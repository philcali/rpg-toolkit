use bevy::prelude::*;

use crate::data::{EditCommand, Project};

/// Plugin that manages undo/redo history and keyboard shortcuts.
pub struct UndoRedoPlugin;

impl Plugin for UndoRedoPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (consume_edit_commands, undo_redo_keyboard));
    }
}

/// Consumes `EditCommand` messages emitted by the painting and layer systems,
/// pushing them onto the active map's undo history.
fn consume_edit_commands(mut project: ResMut<Project>, mut reader: MessageReader<EditCommand>) {
    let Some(active_map_id) = project.active_map_id().cloned() else {
        // No active map — discard any pending commands
        reader.read().for_each(drop);
        return;
    };

    for cmd in reader.read() {
        if let Some(history) = project.undo_histories.get_mut(&active_map_id) {
            history.push_command(cmd.clone());
        }
        project
            .has_unsaved_changes
            .insert(active_map_id.clone(), true);
    }
}

/// Handles Ctrl+Z (undo) and Ctrl+Y (redo) keyboard shortcuts.
fn undo_redo_keyboard(keyboard: Res<ButtonInput<KeyCode>>, mut project: ResMut<Project>) {
    let ctrl = keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight);
    if !ctrl {
        return;
    }

    let Some(active_map_id) = project.active_map_id().cloned() else {
        return;
    };

    let project = &mut *project;

    if keyboard.just_pressed(KeyCode::KeyZ)
        && let (Some(map), Some(history)) = (
            project.maps.get_mut(&active_map_id),
            project.undo_histories.get_mut(&active_map_id),
        )
        && history.undo(map)
    {
        project
            .has_unsaved_changes
            .insert(active_map_id.clone(), true);
    }

    if keyboard.just_pressed(KeyCode::KeyY)
        && let (Some(map), Some(history)) = (
            project.maps.get_mut(&active_map_id),
            project.undo_histories.get_mut(&active_map_id),
        )
        && history.redo(map)
    {
        project
            .has_unsaved_changes
            .insert(active_map_id.clone(), true);
    }
}
