use bevy::prelude::*;

use crate::components::{MoveAnimation, PlayerCharacter, PlayerSpriteState};
use crate::dialog::DialogState;
use crate::events::PlayerMoved;
use crate::input::{Direction, MovementIntent};
use crate::resources::{
    MovementConfig, NpcCollisionEvent, NpcPositions, PlayerVisual, RendererProjectData,
    RendererState,
};
use crate::systems::collision::is_tile_blocked;
use rpg_toolkit_common::{FacingDirection, TriggerMode, sprite_atlas_index};

/// Converts grid coordinates to world-space position using the map's tile dimensions.
pub fn grid_to_world(x: u32, y: u32, tile_width: u32, tile_height: u32) -> Vec2 {
    let wx = x as f32 * tile_width as f32 + tile_width as f32 / 2.0;
    let wy = -(y as f32 * tile_height as f32 + tile_height as f32 / 2.0);
    Vec2::new(wx, wy)
}

/// Startup system: spawns the player character at the project's spawn point.
/// Uses a spritesheet texture atlas if `player_spritesheet` is set and valid,
/// otherwise falls back to a solid-color rectangle.
pub fn spawn_player(
    mut commands: Commands,
    project_data: Res<RendererProjectData>,
    mut renderer_state: ResMut<RendererState>,
    player_visual: Res<PlayerVisual>,
) {
    let Some(spawn) = &project_data.project_file.spawn_point else {
        warn!("No spawn point defined in project; skipping player spawn");
        return;
    };

    let Some(map) = project_data.project_file.maps.get(&spawn.map_id) else {
        warn!(
            "Spawn point references non-existent map '{}'; skipping player spawn",
            spawn.map_id
        );
        return;
    };

    // Clamp spawn coordinates to map bounds
    let grid_x = spawn.x.min(map.width.saturating_sub(1));
    let grid_y = spawn.y.min(map.height.saturating_sub(1));

    // Set the active map
    renderer_state.active_map_id = Some(spawn.map_id.clone());

    let world_pos = grid_to_world(grid_x, grid_y, map.tile_width, map.tile_height);
    let z = map.layers.len() as f32 + 1.0;

    let player = PlayerCharacter {
        grid_x,
        grid_y,
        move_animation: None,
        elevation: 0,
    };

    // Check if a valid player spritesheet is configured
    let has_spritesheet = project_data
        .project_file
        .player_spritesheet
        .as_ref()
        .is_some_and(|ss_id| {
            project_data.spritesheet_textures.contains_key(ss_id)
                && project_data.spritesheet_atlas_layouts.contains_key(ss_id)
        });

    if has_spritesheet {
        let ss_id = project_data
            .project_file
            .player_spritesheet
            .as_ref()
            .unwrap();
        let texture = project_data.spritesheet_textures[ss_id].clone();
        let atlas_layout = project_data.spritesheet_atlas_layouts[ss_id].clone();
        let idle_index = sprite_atlas_index(FacingDirection::Down, 1);

        // Render at 1:1 pixel scale — the character sprite is designed to be
        // larger than the tile and overlap neighbors. The y-offset keeps the
        // feet aligned with the tile bottom.
        let (sprite_scale, y_offset) = project_data
            .project_file
            .spritesheets
            .get(ss_id)
            .map(|ss| {
                let scale = 1.0_f32;
                let scaled_height = ss.sprite_height as f32 * scale;
                let offset = (scaled_height - map.tile_height as f32) / 2.0;
                (scale, offset)
            })
            .unwrap_or((1.0, 0.0));

        commands.spawn((
            player,
            PlayerSpriteState {
                facing: FacingDirection::Down,
                animation_frame: 1,
                animation_timer: 0.0,
                is_moving: false,
                idle_frames: 0,
                y_offset,
            },
            Sprite {
                image: texture,
                texture_atlas: Some(TextureAtlas {
                    layout: atlas_layout,
                    index: idle_index,
                }),
                ..default()
            },
            Transform::from_xyz(world_pos.x, world_pos.y + y_offset, z)
                .with_scale(Vec3::splat(sprite_scale)),
        ));
    } else {
        commands.spawn((
            player,
            Sprite {
                color: player_visual.color,
                custom_size: Some(Vec2::new(map.tile_width as f32, map.tile_height as f32)),
                ..default()
            },
            Transform::from_xyz(world_pos.x, world_pos.y, z),
        ));
    }
}

/// Update system: reads movement intent and initiates tile-to-tile movement
/// if the target tile is in bounds and not blocked.
/// Also updates `PlayerSpriteState` facing direction and is_moving flag.
#[allow(clippy::too_many_arguments)]
pub fn player_movement(
    intent: Res<MovementIntent>,
    dialog_state: Option<Res<DialogState>>,
    project_data: Res<RendererProjectData>,
    renderer_state: Res<RendererState>,
    movement_config: Res<MovementConfig>,
    npc_positions: Res<NpcPositions>,
    mut collision_event: ResMut<NpcCollisionEvent>,
    mut query: Query<(&mut PlayerCharacter, Option<&mut PlayerSpriteState>)>,
) {
    // Reset collision event each frame
    collision_event.npc_index = None;

    // Block movement if dialog is active with movement_block
    if let Some(ref state) = dialog_state
        && state.movement_blocked
    {
        return;
    }

    let Some(direction) = intent.direction else {
        return;
    };

    let Some(map_id) = &renderer_state.active_map_id else {
        return;
    };

    let Some(map) = project_data.project_file.maps.get(map_id) else {
        return;
    };

    for (mut player, mut sprite_state) in query.iter_mut() {
        // Animation exclusivity: ignore input while animating
        if player.move_animation.is_some() {
            continue;
        }

        let (dx, dy): (i64, i64) = match direction {
            Direction::Up => (0, -1),
            Direction::Down => (0, 1),
            Direction::Left => (-1, 0),
            Direction::Right => (1, 0),
        };

        // Always update facing direction when the player tries to move,
        // even if movement is blocked. This allows the player to turn
        // toward NPCs and obstacles without moving.
        let new_facing = match direction {
            Direction::Up => FacingDirection::Up,
            Direction::Down => FacingDirection::Down,
            Direction::Left => FacingDirection::Left,
            Direction::Right => FacingDirection::Right,
        };
        if let Some(ref mut ss) = sprite_state {
            ss.facing = new_facing;
        }

        let target_x = player.grid_x as i64 + dx;
        let target_y = player.grid_y as i64 + dy;

        // Bounds check
        if target_x < 0
            || target_y < 0
            || target_x >= map.width as i64
            || target_y >= map.height as i64
        {
            continue;
        }

        let target_x = target_x as u32;
        let target_y = target_y as u32;

        // Check opacity blocking first (pass player elevation for elevation-aware collision)
        let opacity_blocked =
            is_tile_blocked(map, target_x, target_y, Some(player.elevation), None);

        if opacity_blocked {
            // Tile is blocked by opacity attributes at the player's elevation
            continue;
        }

        // Check if an NPC at the same elevation occupies the destination tile
        if npc_positions.is_occupied_at_elevation(target_x, target_y, player.elevation) {
            // Find which NPC occupies this tile at the player's elevation and check for collision triggers
            if let Some((npc_index, _)) =
                npc_positions
                    .positions
                    .iter()
                    .enumerate()
                    .find(|(_, (nx, ny, ne))| {
                        *nx == target_x && *ny == target_y && *ne == player.elevation
                    })
            {
                // Check if the NPC has Collision trigger mode and non-empty event_triggers
                if let Some(npc_instance) = map.npcs.get(npc_index)
                    && npc_instance.trigger_mode == TriggerMode::Collision
                    && !npc_instance.event_triggers.is_empty()
                {
                    // Signal the trigger system by writing to NpcCollisionEvent (immediate visibility)
                    collision_event.npc_index = Some(npc_index);
                }
            }
            // Block movement regardless (NPC tile is always blocked)
            continue;
        }

        // Start movement animation
        if let Some(ref mut ss) = sprite_state {
            // Only reset the animation timer if starting from a genuine
            // idle state (idle for more than the 1-frame grace period).
            if ss.idle_frames > 1 {
                ss.animation_timer = 0.0;
            }
            ss.is_moving = true;
            ss.idle_frames = 0;
        }

        let from_grid = (player.grid_x, player.grid_y);
        let from = grid_to_world(from_grid.0, from_grid.1, map.tile_width, map.tile_height);
        let to = grid_to_world(target_x, target_y, map.tile_width, map.tile_height);

        player.grid_x = target_x;
        player.grid_y = target_y;
        player.move_animation = Some(MoveAnimation {
            from,
            to,
            from_grid,
            to_grid: (target_x, target_y),
            elapsed: 0.0,
            duration: movement_config.move_duration,
        });
    }
}

/// Update system: advances movement animation and fires `PlayerMoved` on completion.
/// Clears `is_moving` on `PlayerSpriteState` when the animation finishes.
pub fn animate_player(
    time: Res<Time>,
    mut query: Query<(
        &mut PlayerCharacter,
        &mut Transform,
        Option<&mut PlayerSpriteState>,
    )>,
    mut player_moved: MessageWriter<PlayerMoved>,
) {
    for (mut player, mut transform, sprite_state) in query.iter_mut() {
        let Some(ref mut anim) = player.move_animation else {
            continue;
        };

        let y_offset = sprite_state.as_ref().map_or(0.0, |ss| ss.y_offset);

        anim.elapsed += time.delta_secs();
        let t = (anim.elapsed / anim.duration).clamp(0.0, 1.0);
        let pos = anim.from.lerp(anim.to, t);
        transform.translation.x = pos.x;
        transform.translation.y = pos.y + y_offset;

        if anim.elapsed >= anim.duration {
            // Snap to final position
            transform.translation.x = anim.to.x;
            transform.translation.y = anim.to.y + y_offset;

            let from_grid = anim.from_grid;
            let to_grid = anim.to_grid;
            player.move_animation = None;

            // Clear is_moving so the sprite returns to idle pose
            if let Some(mut ss) = sprite_state {
                ss.is_moving = false;
            }

            player_moved.write(PlayerMoved {
                from: from_grid,
                to: to_grid,
            });
        }
    }
}
/// Update system: cycles the player's sprite animation frames while moving,
/// and resets to idle frame (frame 1) when stationary.
pub fn animate_player_sprite(
    time: Res<Time>,
    animation_config: Res<crate::resources::AnimationConfig>,
    mut query: Query<(&mut PlayerSpriteState, &mut Sprite)>,
) {
    for (mut state, mut sprite) in query.iter_mut() {
        let Some(ref mut atlas) = sprite.texture_atlas else {
            continue;
        };

        if state.is_moving {
            state.idle_frames = 0;
            state.animation_timer += time.delta_secs();
            let frame = rpg_toolkit_common::walk_animation_frame(
                state.animation_timer,
                animation_config.clamped_frame_duration(),
            );
            state.animation_frame = frame;
            atlas.index = sprite_atlas_index(state.facing, frame);
        } else {
            state.idle_frames = state.idle_frames.saturating_add(1);
            if state.idle_frames > 1 {
                // Genuinely idle — show idle pose and reset timer.
                state.animation_frame = 1;
                state.animation_timer = 0.0;
                atlas.index = sprite_atlas_index(state.facing, 1);
            } else {
                // Grace frame between consecutive tile moves —
                // keep showing the current walk frame so the cycle
                // isn't interrupted.
                state.animation_timer += time.delta_secs();
                let frame = rpg_toolkit_common::walk_animation_frame(
                    state.animation_timer,
                    animation_config.clamped_frame_duration(),
                );
                state.animation_frame = frame;
                atlas.index = sprite_atlas_index(state.facing, frame);
            }
        }
    }
}
