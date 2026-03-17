use bevy::prelude::*;

use crate::data::{EditorState, MapData};

/// Default tile size in pixels (used when no tileset is loaded).
const DEFAULT_TILE_SIZE: f32 = 16.0;

/// Marker component for the editor's 2D camera.
#[derive(Component)]
pub struct EditorCamera;

/// Plugin that manages the 2D camera, grid overlay, and zoom-to-fit on map creation.
pub struct CanvasPlugin;

impl Plugin for CanvasPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera)
            .add_systems(Update, (zoom_to_fit_on_new_map, draw_grid));
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((Camera2d, EditorCamera));
}

/// When a `MapData` resource is first inserted, compute zoom-to-fit and apply it.
fn zoom_to_fit_on_new_map(
    map: Option<Res<MapData>>,
    mut editor: ResMut<EditorState>,
    mut camera_q: Query<&mut Transform, With<EditorCamera>>,
    windows: Query<&Window>,
) {
    let Some(map) = map else { return };
    if !map.is_changed() {
        return;
    }

    let Ok(window) = windows.single() else { return };
    let viewport_w = window.width();
    let viewport_h = window.height();

    let map_pixel_w = map.width as f32 * DEFAULT_TILE_SIZE;
    let map_pixel_h = map.height as f32 * DEFAULT_TILE_SIZE;

    let zoom = (viewport_w / map_pixel_w)
        .min(viewport_h / map_pixel_h)
        .clamp(0.25, 8.0);

    editor.set_zoom(zoom);

    // Center the camera on the middle of the map.
    // The grid spans x: [0, map_pixel_w], y: [-map_pixel_h, 0],
    // so the center is (map_pixel_w / 2, -map_pixel_h / 2).
    let center = Vec2::new(map_pixel_w / 2.0, -map_pixel_h / 2.0);
    editor.camera_offset = center;

    if let Ok(mut transform) = camera_q.single_mut() {
        transform.scale = Vec3::splat(1.0 / zoom);
        transform.translation = Vec3::new(center.x, center.y, 0.0);
    }
}

/// Draw a grid overlay aligned to tile boundaries using gizmos.
fn draw_grid(
    map: Option<Res<MapData>>,
    mut gizmos: Gizmos,
) {
    let Some(map) = map else { return };

    let tile = DEFAULT_TILE_SIZE;
    let cols = map.width as f32;
    let rows = map.height as f32;
    let total_w = cols * tile;
    let total_h = rows * tile;

    // Origin at top-left of the map, so map spans x: [0, total_w], y: [-total_h, 0]
    // (Bevy's Y axis points up, so we go negative for "down")
    let left = 0.0;
    let right = total_w;
    let top = 0.0;
    let bottom = -total_h;

    let grid_color = Color::srgba(1.0, 1.0, 1.0, 0.15);

    // Vertical lines
    for col in 0..=(map.width) {
        let x = left + col as f32 * tile;
        gizmos.line_2d(Vec2::new(x, top), Vec2::new(x, bottom), grid_color);
    }

    // Horizontal lines
    for row in 0..=(map.height) {
        let y = top - row as f32 * tile;
        gizmos.line_2d(Vec2::new(left, y), Vec2::new(right, y), grid_color);
    }
}
