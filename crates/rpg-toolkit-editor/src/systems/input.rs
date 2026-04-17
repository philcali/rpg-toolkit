use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::data::{AnyDialogOpen, Project};
use crate::plugins::canvas::EditorCamera;
use crate::plugins::toolbar::CanvasRect;

/// Resource tracking the current cursor position in world and tile coordinates.
#[derive(Resource, Default)]
pub struct CursorWorldState {
    /// Cursor position in world space, if over the window.
    pub world_pos: Option<Vec2>,
    /// Tile coordinate (col, row) under the cursor, if within map bounds.
    pub tile_pos: Option<(u32, u32)>,
}

/// System that updates `CursorWorldState` each frame by projecting the screen
/// cursor through the camera into world/tile coordinates.
pub fn update_cursor_state(
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_q: Query<(&Camera, &GlobalTransform), With<EditorCamera>>,
    project: Res<Project>,
    mut cursor_state: ResMut<CursorWorldState>,
    canvas_rect: Res<CanvasRect>,
    any_dialog_open: Res<AnyDialogOpen>,
) {
    cursor_state.world_pos = None;
    cursor_state.tile_pos = None;

    // Block cursor state updates when a modal dialog is open
    if any_dialog_open.0 {
        return;
    }

    let Ok(window) = windows.single() else { return };
    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };
    let Ok((camera, cam_transform)) = camera_q.single() else {
        return;
    };

    let Ok(world_pos) = camera.viewport_to_world_2d(cam_transform, cursor_pos) else {
        return;
    };
    cursor_state.world_pos = Some(world_pos);

    // Only compute tile_pos when the cursor is within the canvas area
    // (past the left panels and below the menu bar). This prevents clicks
    // on egui panels (tile palette, layer panel, toolbar) from being
    // interpreted as canvas interactions.
    if cursor_pos.x < canvas_rect.left
        || cursor_pos.y < canvas_rect.top
        || cursor_pos.x > canvas_rect.right
        || cursor_pos.y > canvas_rect.bottom
    {
        return;
    }

    let Some(map) = project.active_map() else {
        return;
    };

    let tile_size = map.tile_width as f32;

    let col = (world_pos.x / tile_size).floor();
    let row = (-world_pos.y / tile_size).floor();

    if col >= 0.0 && row >= 0.0 {
        let col = col as u32;
        let row = row as u32;
        if col < map.width && row < map.height {
            cursor_state.tile_pos = Some((col, row));
        }
    }
}
