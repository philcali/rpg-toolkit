use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use rpg_toolkit_common::EntityTarget;

use crate::components::{GameCamera, NpcSprite, NpcSpriteState, PlayerCharacter};
use crate::resources::{
    CameraFollowTarget, CameraPanState, PixelScaleConfig, PixelScaleMode, RendererProjectData,
    RendererState,
};

use super::player::grid_to_world;

/// Startup system: spawns a 2D camera with the `GameCamera` marker.
pub fn spawn_camera(mut commands: Commands) {
    commands.spawn((Camera2d, GameCamera));
}

/// Returns the largest integer `s >= 1` such that
/// `map_pixel_w * s <= win_w` AND `map_pixel_h * s <= win_h`.
/// If no integer greater than 1 satisfies both constraints, returns 1.
pub fn compute_zoom_to_fit(win_w: f32, win_h: f32, map_pixel_w: f32, map_pixel_h: f32) -> u32 {
    if map_pixel_w <= 0.0 || map_pixel_h <= 0.0 || win_w <= 0.0 || win_h <= 0.0 {
        return 1;
    }
    let max_w = (win_w / map_pixel_w).floor() as u32;
    let max_h = (win_h / map_pixel_h).floor() as u32;
    max_w.min(max_h).max(1)
}

/// Computes and applies pixel scaling to the camera projection.
/// Runs after sprite spawning, before camera bounds clamping.
pub fn apply_pixel_scale(
    mut pixel_scale: ResMut<PixelScaleConfig>,
    project_data: Res<RendererProjectData>,
    renderer_state: Res<RendererState>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut camera_query: Query<&mut Projection, With<GameCamera>>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let Ok(mut projection) = camera_query.single_mut() else {
        return;
    };

    match pixel_scale.mode {
        PixelScaleMode::ZoomToFit => {
            let Some(map_id) = &renderer_state.active_map_id else {
                return;
            };
            let Some(map) = project_data.project_file.maps.get(map_id) else {
                return;
            };
            let map_pixel_w = map.width as f32 * map.tile_width as f32;
            let map_pixel_h = map.height as f32 * map.tile_height as f32;
            pixel_scale.effective_scale =
                compute_zoom_to_fit(window.width(), window.height(), map_pixel_w, map_pixel_h);
        }
        PixelScaleMode::Fixed(n) => {
            pixel_scale.effective_scale = n.max(1);
        }
    }

    let scale = 1.0 / pixel_scale.effective_scale as f32;
    if let Projection::Orthographic(ref mut ortho) = *projection {
        ortho.scale = scale;
    }
}

/// Update system: positions the camera based on priority:
/// 1. CameraPanState active → skip (pan system handles positioning)
/// 2. CameraFollowTarget exists → track that entity
/// 3. Default → follow the player character
///
/// Clamps the camera to map bounds so the viewport doesn't show areas outside the map.
#[allow(clippy::too_many_arguments)]
pub fn update_camera(
    project_data: Res<RendererProjectData>,
    renderer_state: Res<RendererState>,
    pixel_scale: Res<PixelScaleConfig>,
    pan_state: Option<Res<CameraPanState>>,
    follow_target: Option<Res<CameraFollowTarget>>,
    player_query: Query<&Transform, (With<PlayerCharacter>, Without<GameCamera>)>,
    npc_query: Query<(&NpcSprite, &NpcSpriteState)>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut camera_query: Query<&mut Transform, (With<GameCamera>, Without<PlayerCharacter>)>,
) {
    // Priority 1: If CameraPanState exists, the pan system handles camera positioning.
    if pan_state.is_some() {
        return;
    }

    let Some(map_id) = &renderer_state.active_map_id else {
        return;
    };
    let Some(map) = project_data.project_file.maps.get(map_id) else {
        return;
    };
    let Ok(mut cam_tf) = camera_query.single_mut() else {
        return;
    };
    let Ok(window) = windows.single() else {
        return;
    };

    // Determine the focus position based on follow target priority.
    let focus_pos = if let Some(ref target_res) = follow_target {
        match &target_res.target {
            EntityTarget::Player => {
                // Follow the player transform
                let Ok(player_tf) = player_query.single() else {
                    return;
                };
                Vec2::new(player_tf.translation.x, player_tf.translation.y)
            }
            EntityTarget::Npc { npc_id } => {
                // Find the NPC by matching npc_index against map.npcs[i].spritesheet_id
                let mut found = None;
                for (npc_sprite, npc_state) in npc_query.iter() {
                    if let Some(npc_instance) = map.npcs.get(npc_sprite.npc_index)
                        && npc_instance.spritesheet_id == *npc_id
                    {
                        // Use the NPC's grid position converted to world coords
                        found = Some(grid_to_world(
                            npc_state.grid_x,
                            npc_state.grid_y,
                            map.tile_width,
                            map.tile_height,
                        ));
                        break;
                    }
                }
                match found {
                    Some(pos) => pos,
                    None => {
                        // NPC not found — log warning and fall back to player tracking
                        warn!(
                            "CameraFollowTarget references NPC '{}' which was not found on the current map",
                            npc_id
                        );
                        let Ok(player_tf) = player_query.single() else {
                            return;
                        };
                        Vec2::new(player_tf.translation.x, player_tf.translation.y)
                    }
                }
            }
        }
    } else {
        // Priority 3: Default — follow the player
        let Ok(player_tf) = player_query.single() else {
            return;
        };
        Vec2::new(player_tf.translation.x, player_tf.translation.y)
    };

    // Apply map bounds clamping
    let scale = pixel_scale.effective_scale as f32;
    let map_pixel_w = map.width as f32 * map.tile_width as f32;
    let map_pixel_h = map.height as f32 * map.tile_height as f32;
    let half_vp_w = window.width() / scale / 2.0;
    let half_vp_h = window.height() / scale / 2.0;

    let mut cam_x = focus_pos.x;
    let mut cam_y = focus_pos.y;

    // Map spans x: [0, map_pixel_w], y: [-map_pixel_h, 0]
    if map_pixel_w <= window.width() / scale {
        // Map is narrower than viewport — center horizontally
        cam_x = map_pixel_w / 2.0;
    } else {
        cam_x = cam_x.clamp(half_vp_w, map_pixel_w - half_vp_w);
    }

    if map_pixel_h <= window.height() / scale {
        // Map is shorter than viewport — center vertically
        cam_y = -map_pixel_h / 2.0;
    } else {
        cam_y = cam_y.clamp(-map_pixel_h + half_vp_h, -half_vp_h);
    }

    cam_tf.translation.x = cam_x;
    cam_tf.translation.y = cam_y;
}
