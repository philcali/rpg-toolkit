use bevy::prelude::*;
use rpg_toolkit_common::EventAction;
use std::collections::VecDeque;

use crate::components::PlayerCharacter;
use crate::dialog::{
    DialogState, DialogTextRegistry, dialog_config_from_data, dialog_text_from_data,
};
use crate::events::{MapChanged, PlayerMoved, ShowDialog};
use crate::resources::{ActionQueue, RendererProjectData, RendererState};
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
) {
    for event in player_moved.read() {
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
                waiting_for_dialog: false,
            });
        }
    }
}

/// Advances the action queue: fires the next ShowDialog or JumpTo.
/// Waits for dialog dismissal before advancing past ShowDialog actions.
pub fn advance_action_queue(
    mut commands: Commands,
    action_queue: Option<ResMut<ActionQueue>>,
    dialog_state: Option<Res<DialogState>>,
    registry: Option<Res<DialogTextRegistry>>,
    mut renderer_state: ResMut<RendererState>,
    mut show_dialog: MessageWriter<ShowDialog>,
) {
    let Some(mut queue) = action_queue else {
        return;
    };

    // If we're waiting for a dialog to be dismissed...
    if queue.waiting_for_dialog {
        if dialog_state.is_some() {
            // Still waiting — dialog is still active
            return;
        }
        // Dialog was dismissed — pop the completed action and continue
        queue.waiting_for_dialog = false;
        queue.actions.pop_front();
    }

    // If the queue is now empty, remove the resource and return
    if queue.actions.is_empty() {
        commands.remove_resource::<ActionQueue>();
        return;
    }

    // Peek the next action
    let action = queue.actions.front().unwrap().clone();
    match action {
        EventAction::ShowDialog { text, config } => {
            let dialog_text = dialog_text_from_data(&text);
            let dialog_config = dialog_config_from_data(&config);

            // For Id references, check that the registry has the entry
            if let rpg_toolkit_common::DialogTextData::Id(ref id) = text {
                let has_entry = registry.as_ref().is_some_and(|reg| reg.get(id).is_some());
                if !has_entry {
                    warn!(
                        "ShowDialog text ID '{}' not found in DialogTextRegistry; skipping action",
                        id
                    );
                    // Skip this action and try the next one
                    queue.actions.pop_front();
                    if queue.actions.is_empty() {
                        commands.remove_resource::<ActionQueue>();
                    }
                    return;
                }
            }

            show_dialog.write(ShowDialog {
                text: dialog_text,
                config: dialog_config,
            });
            queue.waiting_for_dialog = true;
        }
        EventAction::JumpTo {
            target_map_id,
            target_x,
            target_y,
        } => {
            renderer_state.pending_map_change = Some(target_map_id);
            renderer_state.pending_target_coords = Some((target_x, target_y));
            // Clear the queue and remove the resource — JumpTo terminates the sequence
            commands.remove_resource::<ActionQueue>();
        }
    }
}

/// Handles a pending map change: fires `MapChanged`, updates active map,
/// clamps target coordinates, repositions the player, and clears the pending state.
pub fn handle_map_change(
    mut renderer_state: ResMut<RendererState>,
    project_data: Res<RendererProjectData>,
    mut map_changed: MessageWriter<MapChanged>,
    mut query: Query<(&mut PlayerCharacter, &mut Transform, &mut Sprite)>,
) {
    let Some(new_map_id) = renderer_state.pending_map_change.take() else {
        return;
    };
    let target_coords = renderer_state.pending_target_coords.take();

    let Some(new_map) = project_data.project_file.maps.get(&new_map_id) else {
        warn!(
            "Pending map change to '{}' but map not found; ignoring",
            new_map_id
        );
        return;
    };

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

        let world_pos = grid_to_world(
            clamped_x,
            clamped_y,
            new_map.tile_width,
            new_map.tile_height,
        );
        let z = new_map.layers.len() as f32 + 1.0;

        // Recompute sprite scale and Y offset for the new map's tile dimensions
        let (sprite_scale, y_offset) = project_data
            .project_file
            .player_spritesheet
            .as_ref()
            .and_then(|ss_id| project_data.project_file.spritesheets.get(ss_id))
            .map(|ss| {
                let scale = new_map.tile_width as f32 / ss.sprite_width as f32;
                let scaled_height = ss.sprite_height as f32 * scale;
                let offset = (scaled_height - new_map.tile_height as f32) / 2.0;
                (scale, offset)
            })
            .unwrap_or((1.0, 0.0));

        transform.translation = Vec3::new(world_pos.x, world_pos.y + y_offset, z);
        transform.scale = Vec3::splat(sprite_scale);

        // Update sprite size to match new map's tile dimensions
        sprite.custom_size = Some(Vec2::new(
            new_map.tile_width as f32,
            new_map.tile_height as f32,
        ));
    }

    map_changed.write(MapChanged {
        previous_map_id,
        new_map_id,
    });
}
