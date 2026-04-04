use bevy::prelude::*;

use crate::algorithms::flood_fill::flood_fill;
use crate::algorithms::line_engine::bresenham_line;
use crate::data::map::TileRef;
use crate::data::{EditCommand, EditorState, EditorTool, Project};
use crate::systems::input::CursorWorldState;

/// Plugin that handles tile painting and erasure via mouse input on the canvas.
pub struct PaintingPlugin;

impl Plugin for PaintingPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<EditCommand>().add_systems(
            Update,
            painting_system.after(crate::systems::input::update_cursor_state),
        );
    }
}

fn painting_system(
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    cursor_state: Res<CursorWorldState>,
    mut project: ResMut<Project>,
    mut editor_state: ResMut<EditorState>,
    tool: Res<EditorTool>,
    mut edit_events: MessageWriter<EditCommand>,
) {
    // Pan mode: early return, no painting
    if *tool == EditorTool::Pan {
        return;
    }

    let ctrl_held = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);

    // --- Line drag cancellation: Ctrl released before mouse button ---
    if editor_state.line_drag.active && !ctrl_held {
        editor_state.line_drag.active = false;
        editor_state.line_drag.start_tile = None;
        return;
    }

    // --- Line drag commit: mouse released while Ctrl still held ---
    if editor_state.line_drag.active && mouse.just_released(MouseButton::Left) && ctrl_held {
        if let Some(start) = editor_state.line_drag.start_tile
            && let Some((end_col, end_row)) = cursor_state.tile_pos {
                let line = bresenham_line(start.0, start.1, end_col, end_row);

                // Validate tileset compatibility for paint mode
                if *tool == EditorTool::Paint {
                    if let Some(ref brush) = editor_state.active_brush {
                        if let Some(active_map_id) = project.active_map_id().cloned()
                            && project
                                .check_tileset_compatibility(&brush.tileset_id, &active_map_id)
                                .is_err()
                            {
                                editor_state.line_drag.active = false;
                                editor_state.line_drag.start_tile = None;
                                return;
                            }
                    } else {
                        // No brush set — don't commit (Req 9.8)
                        editor_state.line_drag.active = false;
                        editor_state.line_drag.start_tile = None;
                        return;
                    }
                }

                if let Some(map) = project.active_map_mut() {
                    let layer_index = map.active_layer_index;
                    for (col, row) in line {
                        match *tool {
                            EditorTool::Paint => {
                                if let Some(ref brush) = editor_state.active_brush
                                    && let Ok(cmd) =
                                        map.place_tile(layer_index, col, row, brush.clone())
                                    {
                                        edit_events.write(cmd);
                                    }
                            }
                            EditorTool::Erase => {
                                // Only emit if not already empty (Req 8.5)
                                let already_empty = map
                                    .layers
                                    .get(layer_index)
                                    .and_then(|l| l.tiles.get(row as usize))
                                    .and_then(|r| r.get(col as usize))
                                    .and_then(|cell| cell.as_ref())
                                    .is_none();
                                if !already_empty
                                    && let Ok(cmd) = map.erase_tile(layer_index, col, row) {
                                        edit_events.write(cmd);
                                    }
                            }
                            _ => {}
                        }
                    }
                }
            }
        editor_state.line_drag.active = false;
        editor_state.line_drag.start_tile = None;
        return;
    }

    // While line drag is active, don't process normal clicks
    if editor_state.line_drag.active {
        return;
    }

    let left_just_pressed = mouse.just_pressed(MouseButton::Left);
    let left_pressed = mouse.pressed(MouseButton::Left);

    // --- Ctrl+left-click starts line drag (Paint and Erase modes only) ---
    if ctrl_held && left_just_pressed && (*tool == EditorTool::Paint || *tool == EditorTool::Erase)
    {
        if let Some((col, row)) = cursor_state.tile_pos {
            editor_state.line_drag.active = true;
            editor_state.line_drag.start_tile = Some((col, row));
        }
        return;
    }

    // --- Normal tool operations (no Ctrl held) ---
    if !left_pressed {
        return;
    }

    let Some((col, row)) = cursor_state.tile_pos else {
        return;
    };

    match *tool {
        EditorTool::Paint => {
            // Validate tileset compatibility before placing a tile
            if let Some(ref brush) = editor_state.active_brush
                && let Some(active_map_id) = project.active_map_id().cloned()
                    && project
                        .check_tileset_compatibility(&brush.tileset_id, &active_map_id)
                        .is_err()
                {
                    return;
                }

            let Some(map) = project.active_map_mut() else {
                return;
            };
            let layer_index = map.active_layer_index;

            if let Some(ref brush) = editor_state.active_brush {
                let already_set = map
                    .layers
                    .get(layer_index)
                    .and_then(|l| l.tiles.get(row as usize))
                    .and_then(|r| r.get(col as usize))
                    .and_then(|cell| cell.as_ref())
                    == Some(brush);
                if !already_set
                    && let Ok(cmd) = map.place_tile(layer_index, col, row, brush.clone()) {
                        edit_events.write(cmd);
                    }
            }
            // Right-click is ignored in Paint mode (Req 3.2)
        }

        EditorTool::Erase => {
            let Some(map) = project.active_map_mut() else {
                return;
            };
            let layer_index = map.active_layer_index;

            // Erase on left-click and left-click-drag (Req 8.1, 8.3)
            let already_empty = map
                .layers
                .get(layer_index)
                .and_then(|l| l.tiles.get(row as usize))
                .and_then(|r| r.get(col as usize))
                .and_then(|cell| cell.as_ref())
                .is_none();
            if !already_empty
                && let Ok(cmd) = map.erase_tile(layer_index, col, row) {
                    edit_events.write(cmd);
                }
            // Right-click is ignored in Erase mode (Req 8.2)
        }

        EditorTool::FloodFill => {
            // Only trigger on just_pressed, not continuous press
            if !left_just_pressed {
                return;
            }

            let Some(ref brush) = editor_state.active_brush.clone() else {
                // No brush set — don't perform flood fill (Req 5.5)
                return;
            };

            // Validate tileset compatibility
            if let Some(active_map_id) = project.active_map_id().cloned()
                && project
                    .check_tileset_compatibility(&brush.tileset_id, &active_map_id)
                    .is_err()
                {
                    return;
                }

            let Some(map) = project.active_map_mut() else {
                return;
            };
            let layer_index = map.active_layer_index;

            // Build the grid for the active layer
            let grid: &Vec<Vec<Option<TileRef>>> = &map.layers[layer_index].tiles;
            let target = grid
                .get(row as usize)
                .and_then(|r| r.get(col as usize))
                .cloned()
                .unwrap_or(None);

            let coords = flood_fill(grid, (col, row), &target, brush);

            // Apply all fills
            for (fx, fy) in coords {
                if let Ok(cmd) = map.place_tile(layer_index, fx, fy, brush.clone()) {
                    edit_events.write(cmd);
                }
            }
        }

        EditorTool::StampBrush => {
            // Only trigger on just_pressed
            if !left_just_pressed {
                return;
            }

            let Some(ref stamp) = editor_state.stamp_brush.clone() else {
                // No stamp selection — don't perform stamp (Req 6.5)
                return;
            };

            // Validate tileset compatibility
            if let Some(active_map_id) = project.active_map_id().cloned()
                && project
                    .check_tileset_compatibility(&stamp.tileset_id, &active_map_id)
                    .is_err()
                {
                    return;
                }

            let Some(map) = project.active_map_mut() else {
                return;
            };
            let layer_index = map.active_layer_index;

            // Place the full stamp grid anchored at (col, row)
            for dy in 0..stamp.height {
                for dx in 0..stamp.width {
                    let tile_col = col + dx;
                    let tile_row = row + dy;

                    // Skip out-of-bounds tiles (Req 6.3)
                    if tile_col >= map.width || tile_row >= map.height {
                        continue;
                    }

                    let tile_ref = TileRef {
                        tileset_id: stamp.tileset_id.clone(),
                        col: stamp.top_left_col + dx,
                        row: stamp.top_left_row + dy,
                    };

                    if let Ok(cmd) = map.place_tile(layer_index, tile_col, tile_row, tile_ref) {
                        edit_events.write(cmd);
                    }
                }
            }
        }

        EditorTool::Pan => {
            // Already handled above with early return
        }
    }
}
