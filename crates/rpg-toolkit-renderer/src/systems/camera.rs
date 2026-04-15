use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::components::{GameCamera, PlayerCharacter};
use crate::resources::{RendererProjectData, RendererState};

/// Startup system: spawns a 2D camera with the `GameCamera` marker.
pub fn spawn_camera(mut commands: Commands) {
    commands.spawn((Camera2d, GameCamera));
}

/// Update system: follows the player character and clamps to map bounds
/// so the viewport doesn't show areas outside the map.
pub fn update_camera(
    project_data: Res<RendererProjectData>,
    renderer_state: Res<RendererState>,
    player_query: Query<&Transform, (With<PlayerCharacter>, Without<GameCamera>)>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut camera_query: Query<&mut Transform, (With<GameCamera>, Without<PlayerCharacter>)>,
) {
    let Some(map_id) = &renderer_state.active_map_id else {
        return;
    };
    let Some(map) = project_data.project_file.maps.get(map_id) else {
        return;
    };
    let Ok(player_tf) = player_query.single() else {
        return;
    };
    let Ok(mut cam_tf) = camera_query.single_mut() else {
        return;
    };
    let Ok(window) = windows.single() else {
        return;
    };

    let map_pixel_w = map.width as f32 * map.tile_width as f32;
    let map_pixel_h = map.height as f32 * map.tile_height as f32;
    let half_vp_w = window.width() / 2.0;
    let half_vp_h = window.height() / 2.0;

    // Start at the player's position
    let mut cam_x = player_tf.translation.x;
    let mut cam_y = player_tf.translation.y;

    // Map spans x: [0, map_pixel_w], y: [-map_pixel_h, 0]
    if map_pixel_w <= window.width() {
        // Map is narrower than viewport — center horizontally
        cam_x = map_pixel_w / 2.0;
    } else {
        cam_x = cam_x.clamp(half_vp_w, map_pixel_w - half_vp_w);
    }

    if map_pixel_h <= window.height() {
        // Map is shorter than viewport — center vertically
        cam_y = -map_pixel_h / 2.0;
    } else {
        cam_y = cam_y.clamp(-map_pixel_h + half_vp_h, -half_vp_h);
    }

    cam_tf.translation.x = cam_x;
    cam_tf.translation.y = cam_y;
}
