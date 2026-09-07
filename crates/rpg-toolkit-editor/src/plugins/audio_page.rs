//! Audio database page UI.
//!
//! Provides a dedicated page (reachable via [`AppEditorMode::Audio`]) that lists
//! the project's registered music loops and sound effects. Each list row shows
//! `id — relative_path` and is selectable; an empty-state label is shown when a
//! registry is empty (Requirements 13.1, 13.2, 13.3).
//!
//! The page also hosts registration controls for both music loops and sound
//! effects (task 13.3): a file picker, an identifier text field, and a Register
//! button. Registration reuses the same file-picker, path-normalization, id
//! validation, and duplicate handling as the Project Settings audio panel
//! (`audio_panel.rs`), routing through [`Project::register_music_loop`] /
//! [`Project::register_sound_effect`], which validate the id (1–128 chars,
//! non-empty), reject duplicates leaving existing assets unchanged, and set the
//! `has_unsaved_audio_changes` flag on success. On success the list refreshes
//! automatically (it reads the registry); on invalid/duplicate id an inline
//! error is shown and the registry is left unchanged
//! (Requirements 13.4, 13.5, 13.6, 13.12).
//!
//! Preview playback (task 13.4): when a music loop or sound effect is selected,
//! the page shows Play/Stop controls. Play resolves the selected asset's
//! `relative_path` against the current project root (`EditorState::current_save_path`),
//! loads and decodes it via [`rpg_toolkit_common::audio::AudioLoader`], wraps the
//! bytes in an [`AudioSource`] added to [`Assets<AudioSource>`], and spawns an
//! entity with an [`AudioPlayer`] using [`PlaybackSettings::DESPAWN`] so it stops
//! on its own when finished. The spawned [`Entity`] is stored in
//! [`AudioPageState::preview_entity`]. Before spawning a new preview — and when
//! Stop is pressed or the selection changes — any existing preview entity is
//! despawned, so at most one preview ever plays at a time
//! (Requirements 13.7, 13.8, 13.9, 13.10). When the project root is unknown or
//! resolve/load/decode fails, an inline error is surfaced and the registries are
//! left unchanged (Requirement 13.11).

use bevy::audio::AudioSource;
use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

use rpg_toolkit_common::audio::AudioLoader;
use rpg_toolkit_common::{MusicLoopId, SoundEffectId};

use crate::data::AppEditorMode;
use crate::data::EditorUiSet;
use crate::data::project::Project;
use crate::data::state::EditorState;
use crate::plugins::audio_panel::{
    file_label, music_relative_path, pick_audio_file, sfx_relative_path,
};

/// Plugin that provides the Audio database page UI.
pub struct AudioPagePlugin;

impl Plugin for AudioPagePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AudioPageState>().add_systems(
            EguiPrimaryContextPass,
            audio_page_ui
                .in_set(EditorUiSet::Panels)
                .run_if(resource_equals(AppEditorMode::Audio)),
        );
    }
}

/// UI state for the Audio database page.
#[derive(Resource, Default)]
pub struct AudioPageState {
    /// Currently selected music loop id, if any.
    pub selected_music_loop: Option<MusicLoopId>,
    /// Currently selected sound effect id, if any.
    pub selected_sound_effect: Option<SoundEffectId>,

    /// Absolute path of the audio file picked for the music loop registration.
    pub music_selected_path: Option<String>,
    /// Identifier text currently entered in the music loop registration.
    pub music_id_input: String,
    /// Inline error for the music loop registration control (empty when none).
    pub music_error: String,

    /// Absolute path of the audio file picked for the sound effect registration.
    pub sfx_selected_path: Option<String>,
    /// Identifier text currently entered in the sound effect registration.
    pub sfx_id_input: String,
    /// Inline error for the sound effect registration control (empty when none).
    pub sfx_error: String,

    /// Entity of the currently playing preview, if any. At most one preview
    /// plays at a time; this is despawned on Stop, on switching selection, and
    /// before spawning a new preview (Requirements 13.9, 13.10).
    pub preview_entity: Option<Entity>,
    /// Inline error for the preview controls (empty when none). Set when the
    /// project root is unknown or resolve/load/decode fails (Requirement 13.11).
    pub preview_error: String,
}

fn audio_page_ui(
    mut contexts: EguiContexts,
    mut page_state: ResMut<AudioPageState>,
    mut project: ResMut<Project>,
    editor_state: Res<EditorState>,
    mut commands: Commands,
    mut audio_sources: ResMut<Assets<AudioSource>>,
) -> Result {
    let ctx = contexts.ctx_mut()?;

    // Collect preview intents raised by the UI closure; they are acted on after
    // the closure returns so the `Commands`/`Assets` borrows stay disjoint from
    // the `page_state` borrow the widgets hold.
    let mut intent = PreviewIntent::None;

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("Audio");
        ui.separator();

        egui::ScrollArea::vertical().show(ui, |ui| {
            // === Music Loops ===
            ui.label(egui::RichText::new("Music Loops").strong().size(15.0));
            render_music_registration(ui, &mut project, &mut page_state);
            ui.add_space(6.0);
            render_music_list(ui, &project, &mut page_state, &mut intent);
            render_music_preview_controls(ui, &project, &mut page_state, &mut intent);

            ui.add_space(12.0);
            ui.separator();

            // === Sound Effects ===
            ui.label(egui::RichText::new("Sound Effects").strong().size(15.0));
            render_sfx_registration(ui, &mut project, &mut page_state);
            ui.add_space(6.0);
            render_sfx_list(ui, &project, &mut page_state, &mut intent);
            render_sfx_preview_controls(ui, &project, &mut page_state, &mut intent);

            if !page_state.preview_error.is_empty() {
                ui.add_space(4.0);
                ui.colored_label(egui::Color32::RED, &page_state.preview_error);
            }
        });
    });

    apply_preview_intent(
        intent,
        &mut page_state,
        &project,
        &editor_state,
        &mut commands,
        &mut audio_sources,
    );

    Ok(())
}

/// A preview action requested by the UI this frame, applied after the egui
/// closure returns so command/asset borrows do not overlap the widget borrows.
enum PreviewIntent {
    /// No preview action requested.
    None,
    /// Play the given relative path (already resolved from a selection).
    Play(String),
    /// Stop any currently playing preview.
    Stop,
    /// The selection changed; despawn any current preview without starting a new one.
    SelectionChanged,
}

/// Renders the music loop registration control: file picker, id field, and a
/// Register button. On success the entry is registered (which sets
/// `has_unsaved_audio_changes`) and the input is cleared; on invalid/duplicate
/// id an inline error is shown and the registry is left unchanged.
fn render_music_registration(ui: &mut egui::Ui, project: &mut Project, state: &mut AudioPageState) {
    ui.horizontal(|ui| {
        ui.label("Audio file:");
        if ui.button("Browse…").clicked()
            && let Some(path) = pick_audio_file()
        {
            state.music_selected_path = Some(path);
        }
        ui.label(file_label(&state.music_selected_path));
    });

    ui.horizontal(|ui| {
        ui.label("Identifier:");
        ui.text_edit_singleline(&mut state.music_id_input);
    });

    let can_add = state.music_selected_path.is_some() && !state.music_id_input.trim().is_empty();
    if ui
        .add_enabled(can_add, egui::Button::new("Register Music Loop"))
        .clicked()
    {
        let id = state.music_id_input.trim().to_string();
        let abs_path = state.music_selected_path.clone().unwrap_or_default();
        let relative_path = music_relative_path(&abs_path);
        match project.register_music_loop(id, relative_path) {
            Ok(()) => {
                state.music_error.clear();
                state.music_id_input.clear();
                state.music_selected_path = None;
            }
            Err(e) => {
                state.music_error = e.to_string();
            }
        }
    }

    if !state.music_error.is_empty() {
        ui.colored_label(egui::Color32::RED, &state.music_error);
    }
}

/// Renders the sound effect registration control: file picker, id field, and a
/// Register button. Behaves like [`render_music_registration`] for sound effects.
fn render_sfx_registration(ui: &mut egui::Ui, project: &mut Project, state: &mut AudioPageState) {
    ui.horizontal(|ui| {
        ui.label("Audio file:");
        if ui.button("Browse…").clicked()
            && let Some(path) = pick_audio_file()
        {
            state.sfx_selected_path = Some(path);
        }
        ui.label(file_label(&state.sfx_selected_path));
    });

    ui.horizontal(|ui| {
        ui.label("Identifier:");
        ui.text_edit_singleline(&mut state.sfx_id_input);
    });

    let can_add = state.sfx_selected_path.is_some() && !state.sfx_id_input.trim().is_empty();
    if ui
        .add_enabled(can_add, egui::Button::new("Register Sound Effect"))
        .clicked()
    {
        let id = state.sfx_id_input.trim().to_string();
        let abs_path = state.sfx_selected_path.clone().unwrap_or_default();
        let relative_path = sfx_relative_path(&abs_path);
        match project.register_sound_effect(id, relative_path) {
            Ok(()) => {
                state.sfx_error.clear();
                state.sfx_id_input.clear();
                state.sfx_selected_path = None;
            }
            Err(e) => {
                state.sfx_error = e.to_string();
            }
        }
    }

    if !state.sfx_error.is_empty() {
        ui.colored_label(egui::Color32::RED, &state.sfx_error);
    }
}

/// Renders the selectable list of registered music loops (or an empty-state
/// label). Reads the registry so it reflects newly registered entries.
///
/// Selecting a music loop clears any sound-effect selection and, when it changes
/// the active selection, raises [`PreviewIntent::SelectionChanged`] so the caller
/// despawns any prior preview (at most one plays at a time — Req 13.10).
fn render_music_list(
    ui: &mut egui::Ui,
    project: &Project,
    state: &mut AudioPageState,
    intent: &mut PreviewIntent,
) {
    if project.music_loops.is_empty() {
        ui.label("No music loops registered.");
    } else {
        let mut ids: Vec<MusicLoopId> = project.music_loops.keys().cloned().collect();
        ids.sort();
        for id in ids {
            if let Some(entry) = project.music_loops.get(&id) {
                let is_selected = state.selected_music_loop.as_ref() == Some(&id);
                let label = format!("{} — {}", entry.id, entry.relative_path);
                if ui.selectable_label(is_selected, label).clicked() && !is_selected {
                    state.selected_music_loop = Some(id.clone());
                    state.selected_sound_effect = None;
                    state.preview_error.clear();
                    *intent = PreviewIntent::SelectionChanged;
                }
            }
        }
    }
}

/// Renders the selectable list of registered sound effects (or an empty-state
/// label). Reads the registry so it reflects newly registered entries.
///
/// Selecting a sound effect clears any music-loop selection and, when it changes
/// the active selection, raises [`PreviewIntent::SelectionChanged`] (Req 13.10).
fn render_sfx_list(
    ui: &mut egui::Ui,
    project: &Project,
    state: &mut AudioPageState,
    intent: &mut PreviewIntent,
) {
    if project.sound_effects.is_empty() {
        ui.label("No sound effects registered.");
    } else {
        let mut ids: Vec<SoundEffectId> = project.sound_effects.keys().cloned().collect();
        ids.sort();
        for id in ids {
            if let Some(entry) = project.sound_effects.get(&id) {
                let is_selected = state.selected_sound_effect.as_ref() == Some(&id);
                let label = format!("{} — {}", entry.id, entry.relative_path);
                if ui.selectable_label(is_selected, label).clicked() && !is_selected {
                    state.selected_sound_effect = Some(id.clone());
                    state.selected_music_loop = None;
                    state.preview_error.clear();
                    *intent = PreviewIntent::SelectionChanged;
                }
            }
        }
    }
}

/// Renders Play/Stop controls for the currently selected music loop (if any).
///
/// Play resolves the selected loop's `relative_path` and raises
/// [`PreviewIntent::Play`]; Stop raises [`PreviewIntent::Stop`] (Req 13.7, 13.8).
fn render_music_preview_controls(
    ui: &mut egui::Ui,
    project: &Project,
    state: &mut AudioPageState,
    intent: &mut PreviewIntent,
) {
    let Some(id) = state.selected_music_loop.clone() else {
        return;
    };
    let Some(relative_path) = project
        .music_loops
        .get(&id)
        .map(|e| e.relative_path.clone())
    else {
        return;
    };
    render_preview_controls(ui, &relative_path, state, intent);
}

/// Renders Play/Stop controls for the currently selected sound effect (if any).
fn render_sfx_preview_controls(
    ui: &mut egui::Ui,
    project: &Project,
    state: &mut AudioPageState,
    intent: &mut PreviewIntent,
) {
    let Some(id) = state.selected_sound_effect.clone() else {
        return;
    };
    let Some(relative_path) = project
        .sound_effects
        .get(&id)
        .map(|e| e.relative_path.clone())
    else {
        return;
    };
    render_preview_controls(ui, &relative_path, state, intent);
}

/// Shared Play/Stop row for a selected asset. Play is enabled always (errors are
/// surfaced inline on failure); Stop is enabled only while a preview is playing.
fn render_preview_controls(
    ui: &mut egui::Ui,
    relative_path: &str,
    state: &mut AudioPageState,
    intent: &mut PreviewIntent,
) {
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        if ui.button("▶ Play").clicked() {
            *intent = PreviewIntent::Play(relative_path.to_string());
        }
        let playing = state.preview_entity.is_some();
        if ui
            .add_enabled(playing, egui::Button::new("■ Stop"))
            .clicked()
        {
            *intent = PreviewIntent::Stop;
        }
    });
}

/// Applies a [`PreviewIntent`] raised by the UI this frame.
///
/// Enforces "at most one preview at a time" (Req 13.9, 13.10): every branch that
/// starts, stops, or reacts to a selection change first despawns the existing
/// `preview_entity`. On Play, the asset is resolved against the project root and
/// decoded through [`AudioLoader`]; failures (unknown root, resolve/load/decode)
/// set an inline error and leave the registries and any current preview state
/// untouched beyond the pre-despawn (Req 13.11).
fn apply_preview_intent(
    intent: PreviewIntent,
    state: &mut AudioPageState,
    _project: &Project,
    editor_state: &EditorState,
    commands: &mut Commands,
    audio_sources: &mut Assets<AudioSource>,
) {
    match intent {
        PreviewIntent::None => {}
        PreviewIntent::Stop | PreviewIntent::SelectionChanged => {
            despawn_preview(state, commands);
        }
        PreviewIntent::Play(relative_path) => {
            // Always stop any prior preview first so at most one plays (Req 13.9).
            despawn_preview(state, commands);

            let Some(root) = editor_state.current_save_path.as_ref() else {
                state.preview_error =
                    "cannot preview audio: save the project first to establish its root directory"
                        .to_string();
                return;
            };

            // Resolve + load + decode through the shared loader (Req 13.11).
            let bytes = match AudioLoader::load(root, &relative_path) {
                Ok((_format, bytes)) => bytes,
                Err(err) => {
                    state.preview_error = format!("could not play '{relative_path}': {err}");
                    return;
                }
            };

            // Wrap the raw bytes in an AudioSource asset and play it once,
            // despawning the entity when playback finishes.
            let handle = audio_sources.add(AudioSource {
                bytes: bytes.into(),
            });
            let entity = commands
                .spawn((AudioPlayer(handle), PlaybackSettings::DESPAWN))
                .id();
            state.preview_entity = Some(entity);
            state.preview_error.clear();
        }
    }
}

/// Despawns the current preview entity (if any) and clears `preview_entity`.
fn despawn_preview(state: &mut AudioPageState, commands: &mut Commands) {
    if let Some(entity) = state.preview_entity.take()
        && let Ok(mut ec) = commands.get_entity(entity)
    {
        ec.despawn();
    }
}
