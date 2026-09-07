//! Renderer audio playback: a single cross-fadeable music channel plus a
//! concurrent one-shot sound-effect channel, driven by the `ActionQueue` as
//! non-blocking actions.
//!
//! `PlayMusic` and `PlaySoundEffect` are dispatched from `advance_action_queue`
//! via [`handle_play_music`] and [`handle_play_sound_effect`]; both are
//! non-blocking (the queue pops and continues in the same step). The
//! [`update_music_fades`] system advances cross-fades, fade-ins, and
//! end-of-playback fade-outs each frame, independently of the queue, and counts
//! loop completions for finite `loop_count` playbacks.

use bevy::audio::{PlaybackMode, Volume};
use bevy::prelude::*;

use rpg_toolkit_common::AudioLoader;

use crate::resources::{
    FadeRamp, FadingTrack, MusicChannel, MusicChannelState, MusicCommand, MusicPlayback,
    RendererProjectData, SoundEffectChannel, next_music_command,
};

/// Handles a `PlayMusic` action against the music channel.
///
/// Non-blocking: the caller pops the action and continues in the same step.
///
/// - Looks up `music_loop_id` in `project_file.music_loops`; if missing, warns
///   and leaves the channel unchanged (Req 6.6).
/// - Applies [`next_music_command`]; a same-id request is a no-op (Req 6.3).
/// - `fade_duration == 0.0` → stops the previous track and starts the new one at
///   full volume in the same step (Req 6.2).
/// - `fade_duration > 0.0` with a previous track → cross-fade (Req 6.4).
/// - `fade_duration > 0.0` with no previous track → fade-in (Req 6.5).
/// - Records `loop_count` / `fade_out_duration` bookkeeping (Req 6.1, 6.7–6.9).
#[allow(clippy::too_many_arguments)]
pub fn handle_play_music(
    commands: &mut Commands,
    music_state: &mut MusicChannelState,
    project_data: Option<&RendererProjectData>,
    asset_server: &AssetServer,
    music_loop_id: &str,
    fade_duration: f32,
    loop_count: Option<u32>,
    fade_out_duration: f32,
) {
    let Some(project_data) = project_data else {
        warn!("PlayMusic '{music_loop_id}': no project data available; leaving music unchanged");
        return;
    };

    // Look up the requested loop; missing → warn and leave unchanged (Req 6.6).
    let Some(music_loop) = project_data.project_file.music_loops.get(music_loop_id) else {
        warn!(
            "PlayMusic references music_loop_id '{music_loop_id}' not found in music_loops \
             registry; leaving music channel unchanged"
        );
        return;
    };

    let command = next_music_command(music_state, music_loop_id, fade_duration, true);
    if command == MusicCommand::NoChange {
        // Same id already playing/fading in — continue without restart (Req 6.3).
        return;
    }

    // Classify the audio format through the shared loader path; an unsupported
    // or extension-less path is a load failure — warn and leave unchanged.
    if let Err(err) = AudioLoader::classify_format(&music_loop.relative_path) {
        warn!(
            "PlayMusic '{music_loop_id}': could not load audio '{}': {err}; leaving music \
             channel unchanged",
            music_loop.relative_path
        );
        return;
    }

    let source: Handle<AudioSource> = asset_server.load(music_loop.relative_path.clone());

    // Initial volume: full for an immediate start, zero for a fade-in/cross-fade.
    let start_at_zero = matches!(command, MusicCommand::CrossFade | MusicCommand::FadeIn);
    let initial_volume = if start_at_zero { 0.0 } else { 1.0 };

    // Spawn the new music entity (always looping playback; finite loop counts are
    // enforced by `update_music_fades` observing completions).
    let new_entity = commands
        .spawn((
            AudioPlayer(source),
            PlaybackSettings {
                mode: PlaybackMode::Loop,
                volume: Volume::Linear(initial_volume),
                ..PlaybackSettings::LOOP
            },
            MusicChannel,
        ))
        .id();

    let fade_in = match command {
        MusicCommand::CrossFade | MusicCommand::FadeIn => {
            Some(FadeRamp::new(0.0, 1.0, fade_duration))
        }
        MusicCommand::StartImmediate => None,
        MusicCommand::NoChange => unreachable!(),
    };

    // Retire the previous track.
    if let Some(previous) = music_state.current.take() {
        match command {
            MusicCommand::StartImmediate => {
                // Stop the previous track in the same step (Req 6.2).
                despawn_music_entity(commands, previous.entity);
            }
            MusicCommand::CrossFade => {
                // Keep the previous track alive, fading it to zero (Req 6.4).
                let current = current_entity_volume(&previous);
                music_state.fading_out.push(FadingTrack {
                    entity: previous.entity,
                    ramp: FadeRamp::new(current, 0.0, fade_duration),
                });
            }
            MusicCommand::FadeIn => {
                // FadeIn is only chosen when there is no previous track, but be
                // defensive: retire the old track immediately.
                despawn_music_entity(commands, previous.entity);
            }
            MusicCommand::NoChange => unreachable!(),
        }
    }

    music_state.current = Some(MusicPlayback {
        music_loop_id: music_loop_id.to_string(),
        entity: new_entity,
        fade_in,
        loop_count,
        loops_completed: 0,
        fade_out_duration,
        fade_out: None,
        last_position: 0.0,
        loop_duration: 0.0,
    });
}

/// Handles a `PlaySoundEffect` action.
///
/// Non-blocking: the caller pops the action and continues in the same step.
///
/// - Looks up `sound_effect_id`; if missing, warns and advances without touching
///   either channel (Req 8.3).
/// - Classifies/loads the audio via [`AudioLoader`]; on failure warns and
///   advances without touching the music channel (Req 8.5).
/// - Spawns a one-shot [`SoundEffectChannel`] at `volume` playing exactly once,
///   concurrently with any already-playing effects, never touching the music
///   channel (Req 8.1, 8.2, 8.6).
pub fn handle_play_sound_effect(
    commands: &mut Commands,
    project_data: Option<&RendererProjectData>,
    asset_server: &AssetServer,
    sound_effect_id: &str,
    volume: f32,
) {
    let Some(project_data) = project_data else {
        warn!("PlaySoundEffect '{sound_effect_id}': no project data available; skipping");
        return;
    };

    let Some(sound_effect) = project_data.project_file.sound_effects.get(sound_effect_id) else {
        warn!(
            "PlaySoundEffect references sound_effect_id '{sound_effect_id}' not found in \
             sound_effects registry; skipping without playing audio"
        );
        return;
    };

    // Load/decode via the shared audio path; on failure warn and advance without
    // touching the music channel (Req 8.5).
    if let Err(err) = AudioLoader::classify_format(&sound_effect.relative_path) {
        warn!(
            "PlaySoundEffect '{sound_effect_id}': could not load audio '{}': {err}; skipping",
            sound_effect.relative_path
        );
        return;
    }

    let source: Handle<AudioSource> = asset_server.load(sound_effect.relative_path.clone());

    // Spawn a one-shot effect that despawns when finished. This plays
    // concurrently with any other sound effects and never touches the music
    // channel (Req 8.1, 8.2, 8.6).
    commands.spawn((
        AudioPlayer(source),
        PlaybackSettings {
            mode: PlaybackMode::Despawn,
            volume: Volume::Linear(volume),
            ..PlaybackSettings::DESPAWN
        },
        SoundEffectChannel,
    ));
}

/// Advances all music-channel fades each frame and counts loop completions.
///
/// This runs independently of the `ActionQueue` (Req 6.12):
/// - Cross-fade / fade-in ramps drive the current track from 0.0 → full.
/// - Outgoing cross-fade tracks fade to zero and are despawned on completion.
/// - For a finite `loop_count`, loop completions are detected via the sink
///   playback position wrapping back toward the start; on the Nth completion the
///   track either stops immediately (`fade_out_duration == 0.0`, Req 6.8) or
///   fades to zero over `fade_out_duration` (Req 6.9), leaving the channel silent
///   without resuming the map default (Req 6.10).
pub fn update_music_fades(
    time: Res<Time>,
    mut commands: Commands,
    mut music_state: Option<ResMut<MusicChannelState>>,
    mut sinks: Query<&mut AudioSink, With<MusicChannel>>,
) {
    let Some(music_state) = music_state.as_mut() else {
        return;
    };
    let dt = time.delta_secs();

    // --- Advance outgoing cross-fade tracks --------------------------------
    let mut still_fading = Vec::new();
    for mut track in std::mem::take(&mut music_state.fading_out) {
        track.ramp.elapsed += dt;
        let vol = track.ramp.current_volume();
        if let Ok(mut sink) = sinks.get_mut(track.entity) {
            sink.set_volume(Volume::Linear(vol));
        }
        if track.ramp.is_complete() {
            despawn_music_entity(&mut commands, track.entity);
        } else {
            still_fading.push(track);
        }
    }
    music_state.fading_out = still_fading;

    // --- Advance the current track -----------------------------------------
    let mut clear_current = false;
    if let Some(current) = music_state.current.as_mut() {
        // Fade-in / cross-fade-in progress.
        if let Some(ramp) = current.fade_in.as_mut() {
            ramp.elapsed += dt;
            let vol = ramp.current_volume();
            if let Ok(mut sink) = sinks.get_mut(current.entity) {
                sink.set_volume(Volume::Linear(vol));
            }
            if ramp.is_complete() {
                current.fade_in = None;
            }
        }

        // Loop-completion detection and end-of-playback handling for finite
        // loop counts. Bevy loops the sink seamlessly; we observe the playback
        // position wrapping back toward the start to count completed
        // repetitions and to learn the loop's duration.
        if let Some(n) = current.loop_count
            && current.fade_out.is_none()
        {
            let mut pos = current.last_position;
            if let Ok(sink) = sinks.get(current.entity) {
                pos = sink.position().as_secs_f32();
                if pos + 0.05 < current.last_position {
                    // The sink wrapped back to the start: one repetition
                    // completed. The peak position just before the wrap is our
                    // best estimate of the loop's duration (Req 6.9 timing).
                    current.loops_completed = current.loops_completed.saturating_add(1);
                    current.loop_duration = current.loop_duration.max(current.last_position);
                }
                current.last_position = pos;
            }

            if current.loops_completed >= n {
                // The Nth repetition has completed. With no fade-out we stop
                // immediately (Req 6.8); a fade-out that should have started
                // during the final repetition begins now as a fallback so the
                // channel still ends silent (Req 6.9, 6.10).
                if current.fade_out_duration <= 0.0 {
                    if let Ok(sink) = sinks.get(current.entity) {
                        sink.stop();
                    }
                    despawn_music_entity(&mut commands, current.entity);
                    clear_current = true;
                } else {
                    let start = sinks
                        .get(current.entity)
                        .map(|s| s.volume().to_linear())
                        .unwrap_or(1.0);
                    current.fade_out = Some(FadeRamp::new(start, 0.0, current.fade_out_duration));
                }
            } else if current.fade_out_duration > 0.0
                && current.loop_duration > 0.0
                && current.loops_completed + 1 >= n
                && current.loop_duration - pos <= current.fade_out_duration
            {
                // We are within the final (Nth) repetition and close enough to
                // its end that the fade-out must begin now to complete as that
                // repetition ends (Req 6.9). Scale the ramp to the actual time
                // remaining so it lands at zero exactly at the loop boundary.
                let remaining = (current.loop_duration - pos).max(f32::MIN_POSITIVE);
                let start = sinks
                    .get(current.entity)
                    .map(|s| s.volume().to_linear())
                    .unwrap_or(1.0);
                current.fade_out = Some(FadeRamp::new(start, 0.0, remaining));
            }
        }

        // Advance an in-progress end-of-playback fade-out.
        if let Some(ramp) = current.fade_out.as_mut() {
            ramp.elapsed += dt;
            let vol = ramp.current_volume();
            if let Ok(mut sink) = sinks.get_mut(current.entity) {
                sink.set_volume(Volume::Linear(vol));
            }
            if ramp.is_complete() {
                if let Ok(sink) = sinks.get(current.entity) {
                    sink.stop();
                }
                despawn_music_entity(&mut commands, current.entity);
                clear_current = true;
            }
        }
    }

    if clear_current {
        // Leave the channel silent without resuming the map default (Req 6.10).
        music_state.current = None;
    }
}

/// On `MapChanged`, plays the newly entered map's `default_music_loop` (if any)
/// as the internal equivalent of `PlayMusic { default, 0.0, None, 0.0 }`
/// (Req 4.8 runtime, 6.1, 6.2).
///
/// Reuses [`handle_play_music`], so a default that matches the currently playing
/// loop is a no-op and an unregistered/undecodable default is warned and ignored
/// without disturbing the channel.
pub fn play_map_default_music(
    mut map_changed: MessageReader<crate::events::MapChanged>,
    mut commands: Commands,
    project_data: Option<Res<RendererProjectData>>,
    asset_server: Res<AssetServer>,
    mut music_state: Option<ResMut<MusicChannelState>>,
) {
    for event in map_changed.read() {
        let Some(pd) = project_data.as_deref() else {
            continue;
        };
        let Some(map) = pd.project_file.maps.get(&event.new_map_id) else {
            continue;
        };
        let Some(default_id) = map.default_music_loop.clone() else {
            continue;
        };
        let Some(ms) = music_state.as_deref_mut() else {
            warn!("play_map_default_music: MusicChannelState resource not present; skipping");
            continue;
        };
        handle_play_music(
            &mut commands,
            ms,
            Some(pd),
            &asset_server,
            &default_id,
            0.0,
            None,
            0.0,
        );
    }
}

/// Returns the best-known current volume of a playback for cross-fade start.
fn current_entity_volume(playback: &MusicPlayback) -> f32 {
    // If a fade-in is in progress, use its current interpolated value; otherwise
    // assume full volume.
    playback
        .fade_in
        .as_ref()
        .map(|r| r.current_volume())
        .unwrap_or(1.0)
}

/// Despawns a music-channel entity if it still exists.
fn despawn_music_entity(commands: &mut Commands, entity: Entity) {
    if let Ok(mut ec) = commands.get_entity(entity) {
        ec.despawn();
    }
}
