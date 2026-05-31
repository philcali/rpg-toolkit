use bevy::prelude::*;
use rpg_toolkit_common::{EventAction, FadeType, PlayerAppearance, ScreenShakeMode};
use std::collections::VecDeque;

use crate::components::{FadeOverlay, GameCamera, PlayerCharacter};
use crate::dialog::{
    DialogState, DialogTextRegistry, dialog_config_from_data, dialog_text_from_data,
};
use crate::effects::{
    compute_fade_opacity, compute_shake_offset, is_fade_complete, is_shake_complete,
};
use crate::events::{MapChanged, PlayerMoved, ShowDialog};
use crate::resources::{
    ActionQueue, FadeState, GameState, RendererProjectData, RendererState, ScreenShakeState,
    WaitingFor,
};
use crate::systems::player::grid_to_world;

/// Reacts to `PlayerMoved` events: collects event triggers from all layers at the
/// destination tile and populates the `ActionQueue` for sequential processing.
/// Does nothing if an `ActionQueue` already exists (sequence in progress).
pub fn check_triggers(
    mut player_moved: MessageReader<PlayerMoved>,
    project_data: Res<RendererProjectData>,
    renderer_state: Res<RendererState>,
    action_queue: Option<Res<ActionQueue>>,
    mut commands: Commands,
    mut player_query: Query<&mut PlayerCharacter>,
) {
    for event in player_moved.read() {
        // Apply elevation transition from tile attributes at the destination tile.
        // This runs regardless of whether an action queue exists, since elevation
        // transitions are a passive tile property (not an event action).
        if let Some(map_id) = &renderer_state.active_map_id
            && let Some(map) = project_data.project_file.maps.get(map_id)
        {
            let (x, y) = event.to;
            // Check all layers for a target_elevation at the destination tile
            for layer in &map.layers {
                if let Some(row) = layer.attributes.cells.get(y as usize)
                    && let Some(attrs) = row.get(x as usize)
                    && let Some(target_elev) = attrs.target_elevation
                {
                    // Update the player's elevation
                    if let Ok(mut player) = player_query.single_mut() {
                        player.elevation = target_elev;
                    }
                    // Only apply the first matching transition
                    break;
                }
            }
        }

        // If a sequence is already in progress, ignore new triggers
        if action_queue.is_some() {
            continue;
        }

        let Some(map_id) = &renderer_state.active_map_id else {
            continue;
        };
        let Some(map) = project_data.project_file.maps.get(map_id) else {
            continue;
        };

        let (x, y) = event.to;

        // Collect EventAction entries from all layers at the destination tile
        let mut actions = VecDeque::new();
        for layer in &map.layers {
            let Some(row) = layer.attributes.cells.get(y as usize) else {
                continue;
            };
            let Some(attrs) = row.get(x as usize) else {
                continue;
            };

            for action in &attrs.event_trigger {
                actions.push_back(action.clone());
            }
        }

        // If we collected any actions, insert the ActionQueue resource
        if !actions.is_empty() {
            commands.insert_resource(ActionQueue {
                actions,
                waiting_for: WaitingFor::Nothing,
            });
        }
    }
}

/// Advances the action queue: fires the next action in the sequence.
/// Waits for blocking actions (dialog, screen shake, fade) to complete before advancing.
#[allow(clippy::too_many_arguments)]
pub fn advance_action_queue(
    mut commands: Commands,
    action_queue: Option<ResMut<ActionQueue>>,
    dialog_state: Option<Res<DialogState>>,
    registry: Option<Res<DialogTextRegistry>>,
    shake_state: Option<Res<ScreenShakeState>>,
    fade_state: Option<Res<FadeState>>,
    mut game_state: Option<ResMut<GameState>>,
    mut renderer_state: ResMut<RendererState>,
    mut show_dialog: MessageWriter<ShowDialog>,
    mut camera_query: Query<&mut Transform, With<GameCamera>>,
    mut player_query: Query<&mut Visibility, With<PlayerCharacter>>,
    fade_overlay_query: Query<Entity, With<FadeOverlay>>,
) {
    let Some(mut queue) = action_queue else {
        return;
    };

    // If we're waiting for a blocking action to complete...
    match queue.waiting_for {
        WaitingFor::Dialog => {
            if dialog_state.is_some() {
                return;
            }
            queue.waiting_for = WaitingFor::Nothing;
            queue.actions.pop_front();
        }
        WaitingFor::ScreenShake => {
            if shake_state.is_some() {
                return;
            }
            queue.waiting_for = WaitingFor::Nothing;
            queue.actions.pop_front();
        }
        WaitingFor::Fade => {
            if fade_state.is_some() {
                return;
            }
            queue.waiting_for = WaitingFor::Nothing;
            queue.actions.pop_front();
        }
        WaitingFor::Nothing => {}
    }

    // Process actions in a loop to handle non-blocking actions consecutively
    loop {
        if queue.actions.is_empty() {
            commands.remove_resource::<ActionQueue>();
            return;
        }

        let action = queue.actions.front().unwrap().clone();
        match action {
            EventAction::ShowDialog { text, config } => {
                let dialog_text = dialog_text_from_data(&text);
                let dialog_config = dialog_config_from_data(&config);

                if let rpg_toolkit_common::DialogTextData::Id(ref id) = text {
                    let has_entry = registry.as_ref().is_some_and(|reg| reg.get(id).is_some());
                    if !has_entry {
                        warn!(
                            "ShowDialog text ID '{}' not found in DialogTextRegistry; skipping action",
                            id
                        );
                        queue.actions.pop_front();
                        continue;
                    }
                }

                show_dialog.write(ShowDialog {
                    text: dialog_text,
                    config: dialog_config,
                });
                queue.waiting_for = WaitingFor::Dialog;
                return;
            }
            EventAction::JumpTo {
                target_map_id,
                target_x,
                target_y,
                target_elevation,
            } => {
                renderer_state.pending_map_change = Some(target_map_id);
                renderer_state.pending_target_coords = Some((target_x, target_y));
                renderer_state.pending_target_elevation = target_elevation;
                // Pop the JumpTo action itself; remaining actions (e.g. FadeIn)
                // stay in the queue and will continue processing after the map loads.
                queue.actions.pop_front();
                return;
            }
            EventAction::ScreenShake {
                intensity,
                duration,
                mode,
            } => {
                match mode {
                    ScreenShakeMode::Timed => {
                        if duration <= 0.0 {
                            // Instant complete — just pop and continue
                            queue.actions.pop_front();
                            continue;
                        }
                        commands.insert_resource(ScreenShakeState {
                            intensity,
                            mode,
                            duration,
                            elapsed: 0.0,
                        });
                        queue.waiting_for = WaitingFor::ScreenShake;
                        return;
                    }
                    ScreenShakeMode::Continuous => {
                        commands.insert_resource(ScreenShakeState {
                            intensity,
                            mode,
                            duration,
                            elapsed: 0.0,
                        });
                        // Non-blocking — pop and continue
                        queue.actions.pop_front();
                        continue;
                    }
                }
            }
            EventAction::StopScreenShake => {
                commands.remove_resource::<ScreenShakeState>();
                // Reset camera offset
                if let Ok(mut cam_tf) = camera_query.single_mut() {
                    // The update_camera system will reposition next frame;
                    // just zero out any shake offset by letting it run naturally.
                    // We don't need to do anything special here since removing
                    // ScreenShakeState stops the shake system from applying offsets.
                    let _ = &mut cam_tf; // acknowledge the query
                }
                queue.actions.pop_front();
                continue;
            }
            EventAction::FadeTransition {
                fade_type,
                duration,
                color,
            } => {
                if duration <= 0.0 {
                    // Instant — apply final state
                    match fade_type {
                        FadeType::FadeOut => {
                            // Spawn overlay at full opacity
                            commands.spawn((
                                Node {
                                    width: Val::Percent(100.0),
                                    height: Val::Percent(100.0),
                                    position_type: PositionType::Absolute,
                                    ..default()
                                },
                                BackgroundColor(Color::srgba(color[0], color[1], color[2], 1.0)),
                                ZIndex(999),
                                FadeOverlay,
                            ));
                        }
                        FadeType::FadeIn => {
                            // Despawn any existing overlay
                            for entity in fade_overlay_query.iter() {
                                commands.entity(entity).despawn();
                            }
                        }
                    }
                    queue.actions.pop_front();
                    continue;
                }

                // Spawn the fade overlay entity
                let initial_alpha = match fade_type {
                    FadeType::FadeOut => 0.0,
                    FadeType::FadeIn => 1.0,
                };
                commands.spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        position_type: PositionType::Absolute,
                        ..default()
                    },
                    BackgroundColor(Color::srgba(color[0], color[1], color[2], initial_alpha)),
                    ZIndex(999),
                    FadeOverlay,
                ));

                commands.insert_resource(FadeState {
                    fade_type,
                    duration,
                    elapsed: 0.0,
                    color,
                });
                queue.waiting_for = WaitingFor::Fade;
                return;
            }
            EventAction::SetState { key, value } => {
                if key.is_empty() {
                    warn!("SetState with empty key; skipping action");
                    queue.actions.pop_front();
                    continue;
                }
                if let Some(ref mut gs) = game_state {
                    gs.flags.insert(key, value);
                } else {
                    // GameState resource doesn't exist yet — insert it
                    let mut flags = std::collections::HashMap::new();
                    flags.insert(key, value);
                    commands.insert_resource(GameState { flags });
                }
                queue.actions.pop_front();
                continue;
            }
            EventAction::SetPlayerAppearance { appearance } => {
                match appearance {
                    PlayerAppearance::Hidden => {
                        if let Ok(mut vis) = player_query.single_mut() {
                            *vis = Visibility::Hidden;
                        }
                    }
                    PlayerAppearance::Spritesheet { path } => {
                        warn!(
                            "SetPlayerAppearance(Spritesheet) with path '{}': spritesheet swapping is not yet fully implemented",
                            path
                        );
                        // Ensure player is visible
                        if let Ok(mut vis) = player_query.single_mut() {
                            *vis = Visibility::Inherited;
                        }
                    }
                    PlayerAppearance::Default => {
                        if let Ok(mut vis) = player_query.single_mut() {
                            *vis = Visibility::Inherited;
                        }
                    }
                }
                queue.actions.pop_front();
                continue;
            }
            EventAction::StateCheck {
                key,
                value,
                on_true,
                on_false,
            } => {
                let matched = if let Some(ref gs) = game_state {
                    match value {
                        Some(ref expected) => {
                            gs.flags.get(&key) == Some(expected)
                        }
                        None => {
                            // Check key existence only
                            gs.flags.contains_key(&key)
                        }
                    }
                } else {
                    // No GameState resource — state is effectively empty
                    false
                };

                // Pop the StateCheck action
                queue.actions.pop_front();

                // Push the matching branch to the front so it executes next
                let branch = if matched { on_true } else { on_false };
                for action in branch.into_iter().rev() {
                    queue.actions.push_front(action);
                }
                continue;
            }
        }
    }
}

/// Handles a pending map change: fires `MapChanged`, updates active map,
/// clamps target coordinates, repositions the player, and clears the pending state.
/// Also cleans up any active screen shake effect.
pub fn handle_map_change(
    mut commands: Commands,
    mut renderer_state: ResMut<RendererState>,
    project_data: Res<RendererProjectData>,
    shake_state: Option<Res<ScreenShakeState>>,
    mut map_changed: MessageWriter<MapChanged>,
    mut query: Query<(&mut PlayerCharacter, &mut Transform, &mut Sprite)>,
) {
    let Some(new_map_id) = renderer_state.pending_map_change.take() else {
        return;
    };
    let target_coords = renderer_state.pending_target_coords.take();
    let target_elevation = renderer_state.pending_target_elevation.take();

    let Some(new_map) = project_data.project_file.maps.get(&new_map_id) else {
        warn!(
            "Pending map change to '{}' but map not found; ignoring",
            new_map_id
        );
        return;
    };

    // Clean up active screen shake on map change
    if shake_state.is_some() {
        commands.remove_resource::<ScreenShakeState>();
    }

    let previous_map_id = renderer_state.active_map_id.clone();
    renderer_state.active_map_id = Some(new_map_id.clone());

    // Clamp target coordinates to new map bounds
    let (tx, ty) = target_coords.unwrap_or((0, 0));
    let clamped_x = tx.min(new_map.width.saturating_sub(1));
    let clamped_y = ty.min(new_map.height.saturating_sub(1));

    // Reposition the player
    for (mut player, mut transform, mut sprite) in query.iter_mut() {
        player.grid_x = clamped_x;
        player.grid_y = clamped_y;
        player.move_animation = None; // Cancel any in-progress animation

        // Apply target elevation if specified, otherwise preserve current elevation
        if let Some(elev) = target_elevation {
            player.elevation = elev;
        }

        let world_pos = grid_to_world(
            clamped_x,
            clamped_y,
            new_map.tile_width,
            new_map.tile_height,
        );
        let z = new_map.layers.len() as f32 + 1.0;

        // Compute sprite scale and Y offset for the new map's tile dimensions.
        // Render at 1:1 pixel scale (no shrinking) — the character sprite is
        // designed to be larger than the tile and overlap neighbors.
        let (sprite_scale, y_offset) = project_data
            .project_file
            .player_spritesheet
            .as_ref()
            .and_then(|ss_id| project_data.project_file.spritesheets.get(ss_id))
            .map(|ss| {
                let scale = 1.0_f32;
                let scaled_height = ss.sprite_height as f32 * scale;
                let offset = (scaled_height - new_map.tile_height as f32) / 2.0;
                (scale, offset)
            })
            .unwrap_or((1.0, 0.0));

        transform.translation = Vec3::new(world_pos.x, world_pos.y + y_offset, z);
        transform.scale = Vec3::splat(sprite_scale);

        // Only set custom_size for non-spritesheet players (solid-color fallback).
        // Spritesheet players use transform.scale to fit the tile; setting custom_size
        // would double-scale them.
        if sprite.texture_atlas.is_none() {
            sprite.custom_size = Some(Vec2::new(
                new_map.tile_width as f32,
                new_map.tile_height as f32,
            ));
        }
    }

    map_changed.write(MapChanged {
        previous_map_id,
        new_map_id,
    });
}

/// Runs each frame while `ScreenShakeState` is present.
/// Increments elapsed time, checks for completion, and applies shake offset to camera.
pub fn screen_shake_system(
    mut commands: Commands,
    time: Res<Time>,
    mut shake_state: Option<ResMut<ScreenShakeState>>,
    mut camera_query: Query<&mut Transform, With<GameCamera>>,
) {
    let Some(ref mut state) = shake_state else {
        return;
    };

    state.elapsed += time.delta_secs();

    if is_shake_complete(state.elapsed, state.duration, state.mode) {
        // Shake is done — remove state and reset camera
        commands.remove_resource::<ScreenShakeState>();

        // Camera will be repositioned by update_camera next frame
        return;
    }

    // Generate deterministic pseudo-random seeds from elapsed time
    let seed_x = (state.elapsed * 123.456).fract();
    let seed_y = (state.elapsed * 789.012).fract();

    let (dx, dy) = compute_shake_offset(state.intensity, seed_x, seed_y);

    // Apply offset to camera (this system runs after update_camera,
    // so the offset is applied on top of the base camera position)
    if let Ok(mut cam_tf) = camera_query.single_mut() {
        cam_tf.translation.x += dx;
        cam_tf.translation.y += dy;
    }
}

/// Runs each frame while `FadeState` is present.
/// Increments elapsed time, updates overlay opacity, and handles completion.
pub fn fade_system(
    mut commands: Commands,
    time: Res<Time>,
    mut fade_state: Option<ResMut<FadeState>>,
    mut overlay_query: Query<(Entity, &mut BackgroundColor), With<FadeOverlay>>,
) {
    let Some(ref mut state) = fade_state else {
        return;
    };

    state.elapsed += time.delta_secs();

    let opacity = compute_fade_opacity(state.elapsed, state.duration, state.fade_type);

    // Update overlay color alpha
    for (_, mut bg_color) in overlay_query.iter_mut() {
        bg_color.0 = Color::srgba(state.color[0], state.color[1], state.color[2], opacity);
    }

    if is_fade_complete(state.elapsed, state.duration) {
        let fade_type = state.fade_type;

        // Remove the FadeState resource
        commands.remove_resource::<FadeState>();

        match fade_type {
            FadeType::FadeOut => {
                // Leave overlay at full opacity (screen stays covered)
            }
            FadeType::FadeIn => {
                // Despawn the overlay entity
                for (entity, _) in overlay_query.iter() {
                    commands.entity(entity).despawn();
                }
            }
        }
    }
}
