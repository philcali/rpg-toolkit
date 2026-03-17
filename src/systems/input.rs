use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::plugins::canvas::EditorCamera;

/// Resource tracking the current cursor position in world and tile coordinates.
#[derive(Resource, Default)]
pub struct CursorWorldState {
    /// Cursor position in world space, if over the window.
    pub world_pos: Option<Vec2>,
    /// Tile coordinate (col, row) under the cursor, if within map bounds.
    pub tile_pos: Option<(u32, u32)>,
}

/// Default tile size in pixels (used when no tileset is loaded).
const DEFAULT_TILE_SIZE: f32 = 16.0;

/// System that updates `CursorWorldState` each frame by projecting the screen
/// cursor through the camera into world/tile coordinates.
pub fn update_cursor_state(
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_q: Query<(&Camera, &GlobalTransform), With<EditorCamera>>,
    map: Option<Res<crate::data::MapData>>,
    tileset: Option<Res<crate::data::TilesetData>>,
    mut cursor_state: ResMut<CursorWorldState>,
) {
    cursor_state.world_pos = None;
    cursor_state.tile_pos = None;

    let Ok(window) = windows.single() else { return };
    let Some(cursor_pos) = window.cursor_position() else { return };
    let Ok((camera, cam_transform)) = camera_q.single() else { return };

    let Ok(world_pos) = camera.viewport_to_world_2d(cam_transform, cursor_pos) else {
        return;
    };
    cursor_state.world_pos = Some(world_pos);

    let Some(map) = map else { return };

    let tile_size = tileset
        .as_ref()
        .map(|ts| ts.meta.tile_width as f32)
        .unwrap_or(DEFAULT_TILE_SIZE);

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
