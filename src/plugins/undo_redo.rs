use bevy::prelude::*;

use crate::data::{EditCommand, EditorState, MapData, UndoHistory};

/// Plugin that manages undo/redo history and keyboard shortcuts.
pub struct UndoRedoPlugin;

impl Plugin for UndoRedoPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UndoHistory>()
            .add_systems(Update, (consume_edit_commands, undo_redo_keyboard));
    }
}

/// Consumes `EditCommand` messages emitted by the painting and layer systems,
/// pushing them onto the undo history.
fn consume_edit_commands(
    mut history: ResMut<UndoHistory>,
    mut reader: MessageReader<EditCommand>,
    mut editor_state: ResMut<EditorState>,
) {
    for cmd in reader.read() {
        history.push_command(cmd.clone());
        editor_state.has_unsaved_changes = true;
    }
}

/// Handles Ctrl+Z (undo) and Ctrl+Y (redo) keyboard shortcuts.
fn undo_redo_keyboard(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut history: ResMut<UndoHistory>,
    mut map: Option<ResMut<MapData>>,
    mut editor_state: ResMut<EditorState>,
) {
    let Some(ref mut map) = map else { return };

    let ctrl = keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight);
    if !ctrl {
        return;
    }

    if keyboard.just_pressed(KeyCode::KeyZ) && history.undo(map) {
        editor_state.has_unsaved_changes = true;
    }

    if keyboard.just_pressed(KeyCode::KeyY) && history.redo(map) {
        editor_state.has_unsaved_changes = true;
    }
}
