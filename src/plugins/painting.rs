use bevy::prelude::*;

use crate::data::{EditCommand, EditorState, MapData};
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
    mut map: Option<ResMut<MapData>>,
    editor_state: Res<EditorState>,
    mut edit_events: MessageWriter<EditCommand>,
) {
    let Some(ref mut map) = map else { return };

    let left_pressed = mouse.pressed(MouseButton::Left);
    let right_pressed = mouse.pressed(MouseButton::Right);

    if !left_pressed && !right_pressed {
        return;
    }

    let Some((col, row)) = cursor_state.tile_pos else {
        return;
    };

    let layer_index = map.active_layer_index;

    if left_pressed {
        if let Some(brush) = editor_state.active_brush {
            // Skip if the cell already contains the same tile (avoids duplicate undo entries from held clicks)
            let already_set = map
                .layers
                .get(layer_index)
                .and_then(|l| l.tiles.get(row as usize))
                .and_then(|r| r.get(col as usize))
                .copied()
                .flatten()
                == Some(brush);
            if !already_set {
                if let Ok(cmd) = map.place_tile(layer_index, col, row, brush) {
                    edit_events.write(cmd);
                }
            }
        }
    } else if right_pressed {
        // Skip if the cell is already empty
        let already_empty = map
            .layers
            .get(layer_index)
            .and_then(|l| l.tiles.get(row as usize))
            .and_then(|r| r.get(col as usize))
            .copied()
            .flatten()
            .is_none();
        if !already_empty {
            if let Ok(cmd) = map.erase_tile(layer_index, col, row) {
                edit_events.write(cmd);
            }
        }
    }
}
