use bevy::prelude::*;

use crate::data::{EditCommand, EditorState, Project};
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
    cursor_state: Res<CursorWorldState>,
    mut project: ResMut<Project>,
    editor_state: Res<EditorState>,
    mut edit_events: MessageWriter<EditCommand>,
) {
    let left_pressed = mouse.pressed(MouseButton::Left);
    let right_pressed = mouse.pressed(MouseButton::Right);

    if !left_pressed && !right_pressed {
        return;
    }

    let Some((col, row)) = cursor_state.tile_pos else {
        return;
    };

    // Validate tileset compatibility before placing a tile
    if left_pressed && let Some(ref brush) = editor_state.active_brush {
        // Check that the brush's tileset tile size matches the active map's tile size
        if let Some(active_map_id) = project.active_map_id().cloned()
            && project
                .check_tileset_compatibility(&brush.tileset_id, &active_map_id)
                .is_err()
        {
            // Tileset tile size doesn't match the map — reject the paint operation
            return;
        }
    }

    let Some(map) = project.active_map_mut() else {
        return;
    };

    let layer_index = map.active_layer_index;

    if left_pressed {
        if let Some(ref brush) = editor_state.active_brush {
            // Skip if the cell already contains the same tile
            let already_set = map
                .layers
                .get(layer_index)
                .and_then(|l| l.tiles.get(row as usize))
                .and_then(|r| r.get(col as usize))
                .and_then(|cell| cell.as_ref())
                == Some(brush);
            if !already_set && let Ok(cmd) = map.place_tile(layer_index, col, row, brush.clone()) {
                edit_events.write(cmd);
            }
        }
    } else if right_pressed {
        // Skip if the cell is already empty
        let already_empty = map
            .layers
            .get(layer_index)
            .and_then(|l| l.tiles.get(row as usize))
            .and_then(|r| r.get(col as usize))
            .and_then(|cell| cell.as_ref())
            .is_none();
        if !already_empty && let Ok(cmd) = map.erase_tile(layer_index, col, row) {
            edit_events.write(cmd);
        }
    }
}
