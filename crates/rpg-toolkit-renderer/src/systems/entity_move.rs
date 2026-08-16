use bevy::prelude::*;
use rpg_toolkit_common::{EntityTarget, FacingDirection};

use crate::components::{
    MoveAnimation, NpcMoveAnimation, NpcSprite, NpcSpriteState, PlayerCharacter,
};
use crate::resources::{
    ActionQueue, EntityMoveState, NpcPositions, RendererProjectData, RendererState, WaitingFor,
};
use crate::systems::collision::is_tile_blocked;
use crate::systems::player::grid_to_world;

/// Runs each frame while `EntityMoveState` is present.
/// Moves the targeted entity tile-by-tile toward the target position.
/// When the target is reached or no further progress can be made,
/// marks complete and advances the action queue.
#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
pub fn entity_move_system(
    mut commands: Commands,
    time: Res<Time>,
    action_queue: Option<ResMut<ActionQueue>>,
    move_state: Option<ResMut<EntityMoveState>>,
    project_data: Option<Res<RendererProjectData>>,
    renderer_state: Res<RendererState>,
    mut npc_positions: ResMut<NpcPositions>,
    mut player_query: Query<&mut PlayerCharacter>,
    mut npc_query: Query<(&NpcSprite, &mut NpcSpriteState)>,
) {
    // Only run when we're waiting for an entity move
    let Some(mut queue) = action_queue else {
        return;
    };
    if queue.waiting_for != WaitingFor::EntityMove {
        return;
    }
    let Some(mut state) = move_state else {
        return;
    };

    // Get map data
    let Some(ref pd) = project_data else {
        return;
    };
    let Some(map_id) = &renderer_state.active_map_id else {
        return;
    };
    let Some(map) = pd.project_file.maps.get(map_id) else {
        return;
    };

    let dt = time.delta_secs();
    let tw = map.tile_width;
    let th = map.tile_height;
    let duration_per_tile = 1.0 / state.speed.max(0.1);

    match &state.target {
        EntityTarget::Player => {
            let Ok(mut player) = player_query.single_mut() else {
                // Player not found — mark complete
                complete_move(&mut commands, &mut queue);
                return;
            };

            // If the player has an active move animation, advance it
            if let Some(ref mut anim) = player.move_animation {
                anim.elapsed += dt;
                if anim.elapsed >= anim.duration {
                    // Snap to destination grid
                    let to_grid = anim.to_grid;
                    player.grid_x = to_grid.0;
                    player.grid_y = to_grid.1;
                    state.current_x = to_grid.0 as f32;
                    state.current_y = to_grid.1 as f32;
                    player.move_animation = None;
                }
                return;
            }

            // Check if we've reached the target
            if player.grid_x == state.target_x && player.grid_y == state.target_y {
                complete_move(&mut commands, &mut queue);
                return;
            }

            // Compute next step direction (horizontal first, then vertical)
            let (step_dx, step_dy) = compute_step(
                player.grid_x as i64,
                player.grid_y as i64,
                state.target_x as i64,
                state.target_y as i64,
            );

            let dest_x = (player.grid_x as i64 + step_dx) as u32;
            let dest_y = (player.grid_y as i64 + step_dy) as u32;

            // Bounds check
            if dest_x >= map.width || dest_y >= map.height {
                // Can't move further — complete (walked as close as possible)
                complete_move(&mut commands, &mut queue);
                return;
            }

            // Check if tile is blocked (pass None for npc_positions to avoid self-blocking issues
            // with the player; elevation filtering uses the player's elevation)
            if is_tile_blocked(
                map,
                dest_x,
                dest_y,
                Some(player.elevation),
                Some(&npc_positions),
            ) {
                // Blocked — walked as close as possible, complete
                complete_move(&mut commands, &mut queue);
                return;
            }

            // Start a new tile move animation
            let from_world = grid_to_world(player.grid_x, player.grid_y, tw, th);
            let to_world = grid_to_world(dest_x, dest_y, tw, th);

            player.move_animation = Some(MoveAnimation {
                from: from_world,
                to: to_world,
                from_grid: (player.grid_x, player.grid_y),
                to_grid: (dest_x, dest_y),
                elapsed: 0.0,
                duration: duration_per_tile,
            });
        }
        EntityTarget::Npc { npc_id } => {
            // Find the NPC index from map data
            let Some(npc_index) = map
                .npcs
                .iter()
                .position(|npc| &npc.spritesheet_id == npc_id)
            else {
                // NPC not found — complete
                warn!(
                    "entity_move_system: NPC '{}' not found; completing move",
                    npc_id
                );
                complete_move(&mut commands, &mut queue);
                return;
            };

            // Find the NPC entity with matching npc_index
            let mut found = false;
            for (npc_sprite, mut npc_state) in npc_query.iter_mut() {
                if npc_sprite.npc_index != npc_index {
                    continue;
                }
                found = true;

                // If the NPC has an active move animation, advance it
                if let Some(ref mut anim) = npc_state.move_animation {
                    anim.elapsed += dt;
                    if anim.elapsed >= anim.duration {
                        // Snap to destination grid
                        let to_grid = anim.to_grid;
                        npc_state.grid_x = to_grid.0;
                        npc_state.grid_y = to_grid.1;
                        npc_state.move_animation = None;
                        npc_state.is_moving = false;
                        state.current_x = to_grid.0 as f32;
                        state.current_y = to_grid.1 as f32;

                        // Update NpcPositions
                        if let Some(pos) = npc_positions.positions.get_mut(npc_index) {
                            let elevation = pos.2;
                            *pos = (to_grid.0, to_grid.1, elevation);
                        }
                    }
                    return;
                }

                // Check if we've reached the target
                if npc_state.grid_x == state.target_x && npc_state.grid_y == state.target_y {
                    complete_move(&mut commands, &mut queue);
                    return;
                }

                // Compute next step direction (horizontal first, then vertical)
                let (step_dx, step_dy) = compute_step(
                    npc_state.grid_x as i64,
                    npc_state.grid_y as i64,
                    state.target_x as i64,
                    state.target_y as i64,
                );

                let dest_x = (npc_state.grid_x as i64 + step_dx) as u32;
                let dest_y = (npc_state.grid_y as i64 + step_dy) as u32;

                // Bounds check
                if dest_x >= map.width || dest_y >= map.height {
                    complete_move(&mut commands, &mut queue);
                    return;
                }

                // Check if tile is blocked (no elevation filtering for NPC movement)
                if is_tile_blocked(map, dest_x, dest_y, None, None) {
                    complete_move(&mut commands, &mut queue);
                    return;
                }

                // Check NPC-NPC collision (exclude self)
                if npc_positions.is_occupied_by_other(dest_x, dest_y, npc_index) {
                    complete_move(&mut commands, &mut queue);
                    return;
                }

                // Update NpcPositions to destination tile immediately
                if let Some(pos) = npc_positions.positions.get_mut(npc_index) {
                    let elevation = pos.2;
                    *pos = (dest_x, dest_y, elevation);
                }

                // Start a new tile move animation
                let from_world = grid_to_world(npc_state.grid_x, npc_state.grid_y, tw, th);
                let to_world = grid_to_world(dest_x, dest_y, tw, th);

                npc_state.move_animation = Some(NpcMoveAnimation {
                    from: from_world,
                    to: to_world,
                    from_grid: (npc_state.grid_x, npc_state.grid_y),
                    to_grid: (dest_x, dest_y),
                    elapsed: 0.0,
                    duration: duration_per_tile,
                });
                npc_state.is_moving = true;

                // Update facing direction
                npc_state.facing = facing_from_step(step_dx, step_dy);

                break;
            }

            if !found {
                warn!(
                    "entity_move_system: NPC entity with index {} not found; completing move",
                    npc_index
                );
                complete_move(&mut commands, &mut queue);
            }
        }
    }
}

/// Marks the entity move as complete: removes the state resource,
/// resets WaitingFor, and pops the action from the queue.
fn complete_move(commands: &mut Commands, queue: &mut ResMut<ActionQueue>) {
    commands.remove_resource::<EntityMoveState>();
    queue.waiting_for = WaitingFor::Nothing;
    queue.actions.pop_front();
}

/// Computes a single-tile step direction toward the target.
/// Horizontal movement is prioritized over vertical (same as NPC patrol).
fn compute_step(cur_x: i64, cur_y: i64, target_x: i64, target_y: i64) -> (i64, i64) {
    let dx = target_x - cur_x;
    let dy = target_y - cur_y;

    if dx != 0 {
        (if dx > 0 { 1 } else { -1 }, 0)
    } else if dy != 0 {
        (0, if dy > 0 { 1 } else { -1 })
    } else {
        (0, 0)
    }
}

/// Returns the facing direction for a movement step.
fn facing_from_step(dx: i64, dy: i64) -> FacingDirection {
    if dx > 0 {
        FacingDirection::Right
    } else if dx < 0 {
        FacingDirection::Left
    } else if dy > 0 {
        FacingDirection::Down
    } else {
        FacingDirection::Up
    }
}
