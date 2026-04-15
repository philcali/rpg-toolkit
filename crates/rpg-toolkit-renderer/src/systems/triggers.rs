use bevy::prelude::*;
use rpg_toolkit_common::EventAction;

use crate::components::PlayerCharacter;
use crate::events::{MapChanged, PlayerMoved};
use crate::resources::{RendererProjectData, RendererState};
use crate::systems::player::grid_to_world;

/// Reacts to `PlayerMoved` events: collects event triggers from all layers at the
/// destination tile and initiates a map change for the first `JumpTo` found.
pub fn check_triggers(
    mut player_moved: MessageReader<PlayerMoved>,
    project_data: Res<RendererProjectData>,
    renderer_state: Res<RendererState>,
    mut commands: Commands,
) {
    for event in player_moved.read() {
        let Some(map_id) = &renderer_state.active_map_id else {
            continue;
        };
        let Some(map) = project_data.project_file.maps.get(map_id) else {
            continue;
        };

        let (x, y) = event.to;

        // Collect EventAction entries from all layers at the destination tile
        for layer in &map.layers {
            let Some(row) = layer.attributes.cells.get(y as usize) else {
                continue;
            };
            let Some(attrs) = row.get(x as usize) else {
                continue;
            };

            for action in &attrs.event_trigger {
                match action {
                    EventAction::JumpTo {
                        target_map_id,
                        target_x,
                        target_y,
                    } => {
                        // Validate target map exists
                        if !project_data.project_file.maps.contains_key(target_map_id) {
                            warn!(
                                "JumpTo references non-existent map '{}'; ignoring",
                                target_map_id
                            );
                            continue;
                        }

                        // Queue the map change via commands to avoid conflicting borrows
                        let target_map = target_map_id.clone();
                        let tx = *target_x;
                        let ty = *target_y;
                        commands.queue(move |world: &mut World| {
                            let mut state = world.resource_mut::<RendererState>();
                            state.pending_map_change = Some(target_map);
                            state.pending_target_coords = Some((tx, ty));
                        });
                        return; // Execute only the first JumpTo found
                    }
                }
            }
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
        transform.translation = Vec3::new(world_pos.x, world_pos.y, z);

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
