use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

use crate::algorithms::line_engine::bresenham_line;
use crate::data::map::MapId;
use crate::data::{EditorState, EditorTool, Project};
use crate::systems::camera::{self, PanState};
use crate::systems::input::CursorWorldState;

/// Marker component for the editor's 2D camera.
#[derive(Component)]
pub struct EditorCamera;

/// Tracks which map was last zoom-to-fitted so we only re-fit on actual map switches.
#[derive(Resource, Default)]
struct LastFittedMap(Option<MapId>);

/// Plugin that manages the 2D camera, grid overlay, and zoom-to-fit on map creation.
///
/// TODO: Add a canvas toolbar with selectable tools (Pan, Zoom, Paint, Erase, etc.)
/// so that panning doesn't require a middle-mouse button — important for laptop users
/// without a three-button mouse.
pub struct CanvasPlugin;

impl Plugin for CanvasPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PanState>()
            .init_resource::<LastFittedMap>()
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
                    draw_preview_gizmos.after(draw_grid),
                )
                    .before(crate::systems::input::update_cursor_state),
            )
            .add_systems(EguiPrimaryContextPass, coordinate_tooltip_ui);
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
    mut last_fitted: ResMut<LastFittedMap>,
) {
    let Some(active_id) = project.active_map_id().cloned() else {
        last_fitted.0 = None;
        return;
    };

    // Only re-fit when the active map actually changes, not on every Project mutation.
    if last_fitted.0.as_ref() == Some(&active_id) {
        return;
    }
    last_fitted.0 = Some(active_id.clone());

    let Some(active) = project.maps.get(&active_id) else {
        return;
    };

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
pub(crate) fn draw_grid(project: Res<Project>, mut gizmos: Gizmos) {
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

/// Draw preview gizmos for line drag and stamp brush.
fn draw_preview_gizmos(
    editor: Res<EditorState>,
    tool: Res<EditorTool>,
    cursor: Res<CursorWorldState>,
    project: Res<Project>,
    mut gizmos: Gizmos,
) {
    let Some(map) = project.active_map() else {
        return;
    };
    let tile = map.tile_width as f32;
    let highlight = Color::srgba(1.0, 1.0, 0.0, 0.35);

    // Line preview: when a Ctrl+drag line operation is active
    if editor.line_drag.active
        && let (Some((sx, sy)), Some((cx, cy))) = (editor.line_drag.start_tile, cursor.tile_pos)
    {
        let coords = bresenham_line(sx, sy, cx, cy);
        for (x, y) in coords {
            if x < map.width && y < map.height {
                draw_tile_highlight(&mut gizmos, x, y, tile, highlight);
            }
        }
    }

    // Stamp brush preview: show footprint at cursor when in StampBrush mode
    if *tool == EditorTool::StampBrush
        && let (Some(stamp), Some((cx, cy))) = (&editor.stamp_brush, cursor.tile_pos)
    {
        for dy in 0..stamp.height {
            for dx in 0..stamp.width {
                let tx = cx + dx;
                let ty = cy + dy;
                if tx < map.width && ty < map.height {
                    draw_tile_highlight(&mut gizmos, tx, ty, tile, highlight);
                }
            }
        }
    }
}

/// Draw a semi-transparent highlight rectangle over a single tile.
fn draw_tile_highlight(gizmos: &mut Gizmos, col: u32, row: u32, tile_size: f32, color: Color) {
    let x = col as f32 * tile_size + tile_size / 2.0;
    let y = -(row as f32 * tile_size + tile_size / 2.0);
    gizmos.rect_2d(
        Isometry2d::from_translation(Vec2::new(x, y)),
        Vec2::splat(tile_size),
        color,
    );
}

/// Display a coordinate tooltip showing `(x, y)` at the cursor position when
/// hovering over the map canvas. Shown regardless of which editing tool is active.
fn coordinate_tooltip_ui(mut contexts: EguiContexts, cursor: Res<CursorWorldState>) -> Result {
    let Some((col, row)) = cursor.tile_pos else {
        return Ok(());
    };

    let ctx = contexts.ctx_mut()?;

    // Use a tooltip-style floating label near the cursor
    if let Some(pointer_pos) = ctx.pointer_latest_pos() {
        egui::Area::new(egui::Id::new("coord_tooltip"))
            .fixed_pos(egui::pos2(pointer_pos.x + 16.0, pointer_pos.y + 16.0))
            .order(egui::Order::Tooltip)
            .interactable(false)
            .show(ctx, |ui| {
                egui::Frame::popup(ui.style()).show(ui, |ui| {
                    ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
                    ui.label(format!("({}, {})", col, row));
                });
            });
    }

    Ok(())
}
