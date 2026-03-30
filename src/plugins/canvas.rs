use bevy::prelude::*;

use crate::data::{EditorState, Project};
use crate::systems::camera::{self, PanState};

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

/// When a map becomes active via `Project`, compute zoom-to-fit.
///
/// This system only writes to `EditorState`; the `apply_camera_transform` system
/// handles syncing the actual camera `Transform`.
fn zoom_to_fit_on_new_map(
    project: Res<Project>,
    mut editor: ResMut<EditorState>,
    windows: Query<&Window>,
) {
    let Some(active) = project.active_map() else {
        return;
    };

    if !project.is_changed() {
        return;
    }

    let tile_size = active.tile_width as f32;
    let Ok(window) = windows.single() else { return };
    let viewport_w = window.width();
    let viewport_h = window.height();

    let map_pixel_w = active.width as f32 * tile_size;
    let map_pixel_h = active.height as f32 * tile_size;

    let zoom = (viewport_w / map_pixel_w)
        .min(viewport_h / map_pixel_h)
        .clamp(0.25, 8.0);

    editor.set_zoom(zoom);

    let center = Vec2::new(map_pixel_w / 2.0, -map_pixel_h / 2.0);
    editor.camera_offset = center;
}

/// Draw a grid overlay aligned to tile boundaries using gizmos.
fn draw_grid(project: Res<Project>, mut gizmos: Gizmos) {
    let Some(active) = project.active_map() else {
        return;
    };

    let width = active.width;
    let height = active.height;
    let tile = active.tile_width as f32;

    let cols = width as f32;
    let rows = height as f32;
    let total_w = cols * tile;
    let total_h = rows * tile;

    let left = 0.0;
    let right = total_w;
    let top = 0.0;
    let bottom = -total_h;

    let grid_color = Color::srgba(1.0, 1.0, 1.0, 0.15);

    // Vertical lines
    for col in 0..=(width) {
        let x = left + col as f32 * tile;
        gizmos.line_2d(Vec2::new(x, top), Vec2::new(x, bottom), grid_color);
    }

    // Horizontal lines
    for row in 0..=(height) {
        let y = top - row as f32 * tile;
        gizmos.line_2d(Vec2::new(left, y), Vec2::new(right, y), grid_color);
    }
}
