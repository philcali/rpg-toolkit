use bevy::prelude::*;

use crate::data::{EditorState, MapData, TilesetData};
use crate::systems::camera::{self, PanState};

/// Default tile size in pixels (used when no tileset is loaded).
const DEFAULT_TILE_SIZE: f32 = 16.0;

/// Marker component for the editor's 2D camera.
#[derive(Component)]
pub struct EditorCamera;

/// Plugin that manages the 2D camera, grid overlay, and zoom-to-fit on map creation.
///
/// TODO: Add a canvas toolbar with selectable tools (Pan, Zoom, Paint, Erase, etc.)
/// so that panning doesn't require a middle-mouse button — important for laptop users
/// without a three-button mouse.
pub struct CanvasPlugin;

impl Plugin for CanvasPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PanState>()
            .add_systems(Startup, spawn_camera)
            .add_systems(
                Update,
                (
                    zoom_to_fit_on_new_map,
                    camera::zoom_system.after(zoom_to_fit_on_new_map),
                    camera::pan_system.after(zoom_to_fit_on_new_map),
                    camera::apply_camera_transform
                        .after(camera::zoom_system)
                        .after(camera::pan_system),
                    draw_grid.after(camera::apply_camera_transform),
                )
                    .before(crate::systems::input::update_cursor_state),
            );
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((Camera2d, EditorCamera));
}

/// When a `MapData` resource is first inserted, compute zoom-to-fit and apply it.
///
/// This system only writes to `EditorState`; the `apply_camera_transform` system
/// handles syncing the actual camera `Transform`.
fn zoom_to_fit_on_new_map(
    map: Option<Res<MapData>>,
    tileset: Option<Res<TilesetData>>,
    mut editor: ResMut<EditorState>,
    windows: Query<&Window>,
) {
    let Some(map) = map else { return };
    if !map.is_changed() {
        return;
    }

    let tile_size = tileset
        .as_ref()
        .map(|ts| ts.meta.tile_width as f32)
        .unwrap_or(DEFAULT_TILE_SIZE);

    let Ok(window) = windows.single() else { return };
    let viewport_w = window.width();
    let viewport_h = window.height();

    let map_pixel_w = map.width as f32 * tile_size;
    let map_pixel_h = map.height as f32 * tile_size;

    let zoom = (viewport_w / map_pixel_w)
        .min(viewport_h / map_pixel_h)
        .clamp(0.25, 8.0);

    editor.set_zoom(zoom);

    // Center the camera on the middle of the map.
    // The grid spans x: [0, map_pixel_w], y: [-map_pixel_h, 0],
    // so the center is (map_pixel_w / 2, -map_pixel_h / 2).
    let center = Vec2::new(map_pixel_w / 2.0, -map_pixel_h / 2.0);
    editor.camera_offset = center;
}

/// Draw a grid overlay aligned to tile boundaries using gizmos.
fn draw_grid(map: Option<Res<MapData>>, tileset: Option<Res<TilesetData>>, mut gizmos: Gizmos) {
    let Some(map) = map else { return };

    let tile = tileset
        .as_ref()
        .map(|ts| ts.meta.tile_width as f32)
        .unwrap_or(DEFAULT_TILE_SIZE);
    let cols = map.width as f32;
    let rows = map.height as f32;
    let total_w = cols * tile;
    let total_h = rows * tile;

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
