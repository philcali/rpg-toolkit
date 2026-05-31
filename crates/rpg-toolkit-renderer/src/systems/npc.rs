use bevy::prelude::*;
use std::collections::VecDeque;

use crate::components::{
    NpcMoveAnimation, NpcSprite, NpcSpriteState, PlayerCharacter, PlayerSpriteState,
};
use crate::dialog::DialogState;
use crate::resources::{
    ActionQueue, AnimationConfig, GameState, InteractionIntent, NpcCollisionEvent, NpcPositions,
    RendererProjectData, RendererState, WaitingFor,
};
use crate::systems::collision::is_tile_blocked;
use crate::systems::player::grid_to_world;
use rpg_toolkit_common::{
    FacingDirection, PatrolMode, TriggerMode, faced_tile, next_waypoint_index, sprite_atlas_index,
    walk_animation_frame,
};

/// Reads Space/Enter key presses and writes to `InteractionIntent`.
///
/// Resets `pressed` to false each frame, then sets it to true only if:
/// - Space or Enter was just pressed this frame
/// - No dialog is currently active (`DialogState` resource absent)
/// - No action queue is being processed (`ActionQueue` resource absent)
pub fn read_interaction_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut intent: ResMut<InteractionIntent>,
    dialog_state: Option<Res<DialogState>>,
    action_queue: Option<Res<ActionQueue>>,
) {
    // Reset each frame
    intent.pressed = false;

    // Only register interaction if no dialog is active and no action queue is processing
    if dialog_state.is_some() || action_queue.is_some() {
        return;
    }

    if keyboard.just_pressed(KeyCode::Space) || keyboard.just_pressed(KeyCode::Enter) {
        intent.pressed = true;
    }
}

/// Advances NPC patrol state machines: handles move animation interpolation,
/// waypoint pause timers, and initiating new tile-to-tile movements along
/// patrol paths. Updates `NpcPositions` for dynamic collision.
#[allow(clippy::too_many_arguments)]
pub fn npc_patrol_movement(
    time: Res<Time>,
    project_data: Res<RendererProjectData>,
    game_state: Res<GameState>,
    renderer_state: Res<RendererState>,
    mut npc_positions: ResMut<NpcPositions>,
    dialog_state: Option<Res<DialogState>>,
    action_queue: Option<Res<ActionQueue>>,
    player_query: Query<&PlayerCharacter>,
    mut npc_query: Query<(&NpcSprite, &mut NpcSpriteState)>,
) {
    // Freeze all NPC movement when a dialog or action queue is active
    if dialog_state.is_some() || action_queue.is_some() {
        return;
    }

    // Resolve the active map
    let Some(map_id) = &renderer_state.active_map_id else {
        return;
    };
    let Some(map) = project_data.project_file.maps.get(map_id) else {
        return;
    };

    let dt = time.delta_secs();
    let tw = map.tile_width;
    let th = map.tile_height;

    for (npc_sprite, mut npc_state) in npc_query.iter_mut() {
        let npc_index = npc_sprite.npc_index;

        // Only process NPCs that have a patrol state
        if npc_state.patrol.is_none() {
            continue;
        }

        // Check required_state condition
        if let Some(npc_instance) = map.npcs.get(npc_index) {
            let state_ok = if let Some(ref required_state) = npc_instance.required_state {
                game_state
                    .flags
                    .get(&required_state.0)
                    .is_some_and(|v| v == &required_state.1)
            } else {
                true
            };
            if !state_ok {
                continue;
            }
        }

        // Look up the patrol config from the map data
        let Some(npc_instance) = map.npcs.get(npc_index) else {
            continue;
        };
        let Some(ref patrol_config) = npc_instance.patrol_config else {
            continue;
        };
        let waypoints = &patrol_config.waypoints;

        // For Random mode, waypoints aren't needed — NPC wanders randomly
        // For Loop mode, we need at least one waypoint
        if patrol_config.mode == PatrolMode::Loop && waypoints.is_empty() {
            continue;
        }

        // (b) If NPC has an active move animation, advance it
        if npc_state.move_animation.is_some() {
            let anim = npc_state.move_animation.as_mut().unwrap();
            anim.elapsed += dt;

            if anim.elapsed >= anim.duration {
                // Move complete: snap to destination grid
                let to_grid = anim.to_grid;
                let pause_duration = patrol_config.pause.max(0.0);

                npc_state.grid_x = to_grid.0;
                npc_state.grid_y = to_grid.1;
                npc_state.move_animation = None;
                npc_state.is_moving = false;

                // Enter pause state
                let patrol = npc_state.patrol.as_mut().unwrap();
                patrol.paused = true;
                patrol.pause_timer = pause_duration;
            }
            continue;
        }

        // (c) If NPC is paused, count down the pause timer
        {
            let patrol = npc_state.patrol.as_mut().unwrap();
            if patrol.paused {
                patrol.pause_timer -= dt;
                if patrol.pause_timer <= 0.0 {
                    patrol.paused = false;
                    // For Loop mode, advance to next waypoint
                    if patrol_config.mode == PatrolMode::Loop && !waypoints.is_empty() {
                        let (next_idx, next_forward) = next_waypoint_index(
                            patrol.current_waypoint_index,
                            waypoints.len(),
                            patrol_config.mode,
                            patrol.forward,
                        );
                        patrol.current_waypoint_index = next_idx;
                        patrol.forward = next_forward;
                    }
                }
                continue;
            }
        }

        // (d) NPC is not paused and not moving — ready to take a step
        let (step_dx, step_dy): (i64, i64) = if patrol_config.mode == PatrolMode::Random {
            // Random mode: pick a random adjacent direction
            use std::hash::{Hash, Hasher};
            let mut hasher = std::hash::DefaultHasher::new();
            // Use NPC index, position, and time as entropy
            npc_index.hash(&mut hasher);
            npc_state.grid_x.hash(&mut hasher);
            npc_state.grid_y.hash(&mut hasher);
            let time_ms = (time.elapsed_secs() * 1000.0) as u64;
            time_ms.hash(&mut hasher);
            let hash = hasher.finish();
            let directions: [(i64, i64); 4] = [(0, -1), (0, 1), (-1, 0), (1, 0)];
            directions[(hash % 4) as usize]
        } else {
            // Loop mode: step toward current waypoint
            let current_wp_idx = npc_state.patrol.as_ref().unwrap().current_waypoint_index;
            let target_wp = waypoints[current_wp_idx];
            let cur_x = npc_state.grid_x;
            let cur_y = npc_state.grid_y;
            let dx = target_wp.0 as i64 - cur_x as i64;
            let dy = target_wp.1 as i64 - cur_y as i64;

            // If already at the target waypoint, advance to next immediately
            if dx == 0 && dy == 0 {
                let patrol = npc_state.patrol.as_mut().unwrap();
                let (next_idx, next_forward) = next_waypoint_index(
                    patrol.current_waypoint_index,
                    waypoints.len(),
                    patrol_config.mode,
                    patrol.forward,
                );
                patrol.current_waypoint_index = next_idx;
                patrol.forward = next_forward;
                continue;
            }

            // Take ONE step toward the target: horizontal first if both dx and dy are non-zero
            if dx != 0 {
                (if dx > 0 { 1 } else { -1 }, 0)
            } else {
                (0, if dy > 0 { 1 } else { -1 })
            }
        };

        let cur_x = npc_state.grid_x;
        let cur_y = npc_state.grid_y;

        let dest_x = cur_x as i64 + step_dx;
        let dest_y = cur_y as i64 + step_dy;

        // Bounds check
        if dest_x < 0 || dest_y < 0 || dest_x >= map.width as i64 || dest_y >= map.height as i64 {
            continue;
        }

        let dest_x = dest_x as u32;
        let dest_y = dest_y as u32;

        // Check if destination is blocked:
        // 1. Opacity blocking (pass None for player_elevation — NPCs don't use elevation filtering)
        if is_tile_blocked(map, dest_x, dest_y, None, None) {
            continue;
        }
        // 2. NPC-NPC collision (exclude self)
        if npc_positions.is_occupied_by_other(dest_x, dest_y, npc_index) {
            continue;
        }
        // 3. Player collision
        let player_blocks = player_query
            .iter()
            .any(|pc| pc.grid_x == dest_x && pc.grid_y == dest_y);
        if player_blocks {
            continue;
        }

        // Destination is clear — initiate movement
        // Update NpcPositions to destination tile immediately
        if let Some(pos) = npc_positions.positions.get_mut(npc_index) {
            let elevation = pos.2;
            *pos = (dest_x, dest_y, elevation);
        }

        let from_world = grid_to_world(cur_x, cur_y, tw, th);
        let to_world = grid_to_world(dest_x, dest_y, tw, th);

        // Clamp speed to minimum 0.01s
        let duration = patrol_config.speed.max(0.01);

        npc_state.move_animation = Some(NpcMoveAnimation {
            from: from_world,
            to: to_world,
            from_grid: (cur_x, cur_y),
            to_grid: (dest_x, dest_y),
            elapsed: 0.0,
            duration,
        });
        npc_state.is_moving = true;

        // Update facing direction to match movement direction
        npc_state.facing = if step_dx > 0 {
            FacingDirection::Right
        } else if step_dx < 0 {
            FacingDirection::Left
        } else if step_dy > 0 {
            FacingDirection::Down
        } else {
            FacingDirection::Up
        };
    }
}

/// Updates each NPC's sprite atlas index and transform position based on
/// `NpcSpriteState`. While moving, interpolates position and cycles walk
/// frames. While idle/paused, displays idle frame (frame 1) for the current
/// facing direction.
pub fn npc_patrol_animation(
    time: Res<Time>,
    animation_config: Res<AnimationConfig>,
    project_data: Res<RendererProjectData>,
    renderer_state: Res<RendererState>,
    mut npc_query: Query<(&mut NpcSpriteState, &mut Sprite, &mut Transform)>,
) {
    let Some(map_id) = &renderer_state.active_map_id else {
        return;
    };
    let Some(map) = project_data.project_file.maps.get(map_id) else {
        return;
    };

    let tw = map.tile_width;
    let th = map.tile_height;

    for (mut npc_state, mut sprite, mut transform) in npc_query.iter_mut() {
        let Some(ref mut atlas) = sprite.texture_atlas else {
            continue;
        };

        if let Some(ref anim) = npc_state.move_animation {
            // (a) While moving: interpolate position and cycle walk frames
            let t = (anim.elapsed / anim.duration).clamp(0.0, 1.0);
            let pos = anim.from.lerp(anim.to, t);
            transform.translation.x = pos.x;
            transform.translation.y = pos.y + npc_state.y_offset;

            // Advance animation timer and compute walk frame
            npc_state.animation_timer += time.delta_secs();
            let frame = walk_animation_frame(
                npc_state.animation_timer,
                animation_config.clamped_frame_duration(),
            );
            npc_state.animation_frame = frame;
            atlas.index = sprite_atlas_index(npc_state.facing, frame);
        } else {
            // (b) While idle/paused: display idle frame for current facing direction
            npc_state.animation_timer = 0.0;
            npc_state.animation_frame = 1;
            atlas.index = sprite_atlas_index(npc_state.facing, 1);

            // Keep transform at current grid position + y_offset
            let world_pos = grid_to_world(npc_state.grid_x, npc_state.grid_y, tw, th);
            transform.translation.x = world_pos.x;
            transform.translation.y = world_pos.y + npc_state.y_offset;
        }
    }
}

/// Handles NPC collision and interaction triggers, populating `ActionQueue`
/// with the NPC's `event_triggers` when conditions are met.
///
/// - **Collision triggers**: Consumes `NpcCollisionEvent` resource (set by
///   `player_movement` when the player attempts to move onto an NPC tile).
///   If the NPC has `trigger_mode: Collision` and non-empty `event_triggers`,
///   populates `ActionQueue`. Otherwise, default block behavior applies (no-op here).
///
/// - **Interaction triggers**: When `InteractionIntent.pressed` is true, computes
///   the tile the player is facing via `faced_tile`. If an NPC with
///   `trigger_mode: Interaction` and non-empty `event_triggers` occupies that tile,
///   updates the NPC's facing to face the player, then populates `ActionQueue`.
///
/// Skips entirely if an `ActionQueue` already exists (active sequence suppression)
/// or if dialog is active.
#[allow(clippy::too_many_arguments)]
pub fn npc_trigger_system(
    mut commands: Commands,
    action_queue: Option<Res<ActionQueue>>,
    dialog_state: Option<Res<DialogState>>,
    collision_event: Res<NpcCollisionEvent>,
    intent: Res<InteractionIntent>,
    npc_positions: Res<NpcPositions>,
    game_state: Res<GameState>,
    project_data: Res<RendererProjectData>,
    renderer_state: Res<RendererState>,
    player_query: Query<(&PlayerCharacter, Option<&PlayerSpriteState>)>,
    mut npc_query: Query<(&NpcSprite, &mut NpcSpriteState)>,
) {
    // Skip if an ActionQueue is already active (Property 10: active queue suppression)
    if action_queue.is_some() {
        return;
    }

    // Skip if dialog is active
    if dialog_state.is_some() {
        return;
    }

    // Resolve the active map
    let Some(map_id) = &renderer_state.active_map_id else {
        return;
    };
    let Some(map) = project_data.project_file.maps.get(map_id) else {
        return;
    };

    // --- Collision triggers ---
    if let Some(npc_index) = collision_event.npc_index
        && let Some(npc_instance) = map.npcs.get(npc_index)
        && npc_instance.trigger_mode == TriggerMode::Collision
        && !npc_instance.event_triggers.is_empty()
    {
        // Check required_state condition
        let state_ok = if let Some(ref required_state) = npc_instance.required_state {
            game_state
                .flags
                .get(&required_state.0)
                .is_some_and(|v| v == &required_state.1)
        } else {
            true
        };
        if state_ok {
            let actions: VecDeque<_> = npc_instance.event_triggers.iter().cloned().collect();
            commands.insert_resource(ActionQueue {
                actions,
                waiting_for: WaitingFor::Nothing,
            });
            return;
        }
    }
    // If event_triggers is empty, default block behavior applies (no-op)

    // --- Interaction triggers ---
    if !intent.pressed {
        return;
    }

    let Ok((player, player_sprite_state)) = player_query.single() else {
        return;
    };

    // Determine the player's facing direction
    let facing = player_sprite_state
        .map(|ss| ss.facing)
        .unwrap_or(FacingDirection::Down);

    // Compute the tile the player is facing
    let Some((faced_x, faced_y)) = faced_tile(player.grid_x, player.grid_y, facing) else {
        return; // At map boundary, no tile to check
    };

    // Check map bounds for the faced tile
    if faced_x >= map.width || faced_y >= map.height {
        return;
    }

    // Find an NPC at the faced tile with Interaction trigger mode and non-empty triggers
    for (idx, &(nx, ny, _)) in npc_positions.positions.iter().enumerate() {
        if nx != faced_x || ny != faced_y {
            continue;
        }

        let Some(npc_instance) = map.npcs.get(idx) else {
            continue;
        };

        if npc_instance.trigger_mode != TriggerMode::Interaction {
            continue;
        }

        // Check required_state condition
        let state_ok = if let Some(ref required_state) = npc_instance.required_state {
            game_state
                .flags
                .get(&required_state.0)
                .is_some_and(|v| v == &required_state.1)
        } else {
            true
        };
        if !state_ok {
            continue;
        }

        if npc_instance.event_triggers.is_empty() {
            continue;
        }

        // Update NPC facing to face the player before firing triggers (Property 11)
        for (npc_sprite, mut npc_state) in npc_query.iter_mut() {
            if npc_sprite.npc_index == idx {
                // NPC should face toward the player
                // If player is above NPC → NPC faces Up, etc.
                npc_state.facing = opposite_facing(facing);
                break;
            }
        }

        // Populate ActionQueue with the NPC's event triggers
        let actions: VecDeque<_> = npc_instance.event_triggers.iter().cloned().collect();
        commands.insert_resource(ActionQueue {
            actions,
            waiting_for: WaitingFor::Nothing,
        });
        return;
    }
    // If no interactable NPC found on the faced tile, no-op (Requirement 9.3)
}

/// Returns the facing direction that is opposite to the given direction.
/// Used so the NPC faces toward the player: if the player faces Right
/// (meaning the NPC is to the player's right), the NPC should face Left.
fn opposite_facing(facing: FacingDirection) -> FacingDirection {
    match facing {
        FacingDirection::Up => FacingDirection::Down,
        FacingDirection::Down => FacingDirection::Up,
        FacingDirection::Left => FacingDirection::Right,
        FacingDirection::Right => FacingDirection::Left,
    }
}
