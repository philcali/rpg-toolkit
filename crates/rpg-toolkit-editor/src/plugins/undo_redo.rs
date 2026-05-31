use bevy::prelude::*;

use crate::data::commands::EditCommandKind;
use crate::data::{EditCommand, Project};
use crate::plugins::dialog_text_panel::{TextIdIndex, update_text_id_index_for_tile};

/// Plugin that manages undo/redo history and keyboard shortcuts.
pub struct UndoRedoPlugin;

impl Plugin for UndoRedoPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (consume_edit_commands, undo_redo_keyboard));
    }
}

/// Consumes `EditCommand` messages emitted by the painting and layer systems,
/// pushing them onto the active map's undo history.
fn consume_edit_commands(
    mut project: ResMut<Project>,
    mut reader: MessageReader<EditCommand>,
    mut text_id_index: ResMut<TextIdIndex>,
) {
    let Some(active_map_id) = project.active_map_id().cloned() else {
        // No active map — discard any pending commands
        reader.read().for_each(drop);
        return;
    };

    for cmd in reader.read() {
        // SetSpawnPoint operates on Project, not MapData
        if let EditCommandKind::SetSpawnPoint { new_spawn, .. } = &cmd.kind {
            project.spawn_point = new_spawn.clone();
        }
        // Dialog text commands operate on Project, not MapData
        match &cmd.kind {
            EditCommandKind::InsertDialogText { text_id, text } => {
                project.dialog_texts.insert(text_id.clone(), text.clone());
            }
            EditCommandKind::UpdateDialogText {
                text_id, new_text, ..
            } => {
                project
                    .dialog_texts
                    .insert(text_id.clone(), new_text.clone());
            }
            EditCommandKind::RemoveDialogText { text_id, .. } => {
                project.dialog_texts.remove(text_id);
            }
            _ => {}
        }
        // Face portrait commands operate on Project, not MapData
        match &cmd.kind {
            EditCommandKind::InsertFacePortrait { id, path } => {
                project.face_portraits.insert(id.clone(), path.clone());
            }
            EditCommandKind::UpdateFacePortrait { id, new_path, .. } => {
                project.face_portraits.insert(id.clone(), new_path.clone());
            }
            EditCommandKind::RemoveFacePortrait { id, .. } => {
                project.face_portraits.remove(id);
            }
            _ => {}
        }
        // Update TextIdIndex when event triggers change (apply direction)
        if let EditCommandKind::SetEventTrigger {
            layer_index,
            x,
            y,
            old_trigger,
            new_trigger,
        } = &cmd.kind
        {
            let map_name = project
                .maps
                .get(&active_map_id)
                .map(|m| m.name.clone())
                .unwrap_or_default();
            update_text_id_index_for_tile(
                &mut text_id_index,
                &active_map_id,
                &map_name,
                *layer_index,
                *x,
                *y,
                old_trigger,
                new_trigger,
            );
        }
        if let Some(history) = project.undo_histories.get_mut(&active_map_id) {
            history.push_command(cmd.clone());
        }
        project
            .has_unsaved_changes
            .insert(active_map_id.clone(), true);
    }
}

/// Handles Ctrl+Z (undo) and Ctrl+Y (redo) keyboard shortcuts.
fn undo_redo_keyboard(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut project: ResMut<Project>,
    mut text_id_index: ResMut<TextIdIndex>,
) {
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
    {
        if let Some(cmd) = history.undo_stack.last()
            && let EditCommandKind::SetSpawnPoint { old_spawn, .. } = &cmd.kind
        {
            project.spawn_point = old_spawn.clone();
        }
        // Handle dialog text undo at Project level
        if let Some(cmd) = history.undo_stack.last() {
            match &cmd.kind {
                EditCommandKind::InsertDialogText { text_id, .. } => {
                    project.dialog_texts.remove(text_id);
                }
                EditCommandKind::UpdateDialogText {
                    text_id, old_text, ..
                } => {
                    project
                        .dialog_texts
                        .insert(text_id.clone(), old_text.clone());
                }
                EditCommandKind::RemoveDialogText { text_id, old_text } => {
                    project
                        .dialog_texts
                        .insert(text_id.clone(), old_text.clone());
                }
                _ => {}
            }
        }
        // Handle face portrait undo at Project level
        if let Some(cmd) = history.undo_stack.last() {
            match &cmd.kind {
                EditCommandKind::InsertFacePortrait { id, .. } => {
                    project.face_portraits.remove(id);
                }
                EditCommandKind::UpdateFacePortrait { id, old_path, .. } => {
                    project.face_portraits.insert(id.clone(), old_path.clone());
                }
                EditCommandKind::RemoveFacePortrait { id, path } => {
                    project.face_portraits.insert(id.clone(), path.clone());
                }
                _ => {}
            }
        }
        // Update TextIdIndex on undo of SetEventTrigger (reverse direction: new→old)
        if let Some(cmd) = history.undo_stack.last()
            && let EditCommandKind::SetEventTrigger {
                layer_index,
                x,
                y,
                old_trigger,
                new_trigger,
            } = &cmd.kind
        {
            let map_name = map.name.clone();
            update_text_id_index_for_tile(
                &mut text_id_index,
                &active_map_id,
                &map_name,
                *layer_index,
                *x,
                *y,
                new_trigger, // undo: the "new" triggers are what's currently applied
                old_trigger, // undo: we're reverting to the "old" triggers
            );
        }
        if history.undo(map) {
            project
                .has_unsaved_changes
                .insert(active_map_id.clone(), true);
        }
    }

    if keyboard.just_pressed(KeyCode::KeyY)
        && let (Some(map), Some(history)) = (
            project.maps.get_mut(&active_map_id),
            project.undo_histories.get_mut(&active_map_id),
        )
    {
        if let Some(cmd) = history.redo_stack.last()
            && let EditCommandKind::SetSpawnPoint { new_spawn, .. } = &cmd.kind
        {
            project.spawn_point = new_spawn.clone();
        }
        // Handle dialog text redo at Project level
        if let Some(cmd) = history.redo_stack.last() {
            match &cmd.kind {
                EditCommandKind::InsertDialogText { text_id, text } => {
                    project.dialog_texts.insert(text_id.clone(), text.clone());
                }
                EditCommandKind::UpdateDialogText {
                    text_id, new_text, ..
                } => {
                    project
                        .dialog_texts
                        .insert(text_id.clone(), new_text.clone());
                }
                EditCommandKind::RemoveDialogText { text_id, .. } => {
                    project.dialog_texts.remove(text_id);
                }
                _ => {}
            }
        }
        // Handle face portrait redo at Project level
        if let Some(cmd) = history.redo_stack.last() {
            match &cmd.kind {
                EditCommandKind::InsertFacePortrait { id, path } => {
                    project.face_portraits.insert(id.clone(), path.clone());
                }
                EditCommandKind::UpdateFacePortrait { id, new_path, .. } => {
                    project.face_portraits.insert(id.clone(), new_path.clone());
                }
                EditCommandKind::RemoveFacePortrait { id, .. } => {
                    project.face_portraits.remove(id);
                }
                _ => {}
            }
        }
        // Update TextIdIndex on redo of SetEventTrigger (forward direction: old→new)
        if let Some(cmd) = history.redo_stack.last()
            && let EditCommandKind::SetEventTrigger {
                layer_index,
                x,
                y,
                old_trigger,
                new_trigger,
            } = &cmd.kind
        {
            let map_name = map.name.clone();
            update_text_id_index_for_tile(
                &mut text_id_index,
                &active_map_id,
                &map_name,
                *layer_index,
                *x,
                *y,
                old_trigger,
                new_trigger,
            );
        }
        if history.redo(map) {
            project
                .has_unsaved_changes
                .insert(active_map_id.clone(), true);
        }
    }
}
