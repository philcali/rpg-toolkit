//! Modal dialog for editing event trigger action sequences on a tile.

use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

use super::action_editor::ActionEditorState;
use super::action_editor_ui::render_action_editor;
use crate::data::Project;
use crate::data::commands::{EditCommand, EditCommandKind};
use crate::data::map::EventAction;

/// Resource for the event trigger editing dialog.
#[derive(Resource, Default)]
pub struct EventTriggerDialog {
    pub open: bool,
    pub layer_index: usize,
    pub tile_x: u32,
    pub tile_y: u32,
    pub actions: Vec<EventAction>,
    pub original_actions: Vec<EventAction>,
    pub action_editor: ActionEditorState,
}

/// Egui panel for editing event triggers on a tile.
pub fn event_trigger_panel_ui(
    mut contexts: EguiContexts,
    mut dialog: ResMut<EventTriggerDialog>,
    mut project: ResMut<Project>,
    mut edit_events: MessageWriter<EditCommand>,
) -> Result {
    if !dialog.open {
        return Ok(());
    }

    let ctx = contexts.ctx_mut()?;

    let mut should_close = false;
    let mut should_save = false;

    // Collect map entries for the JumpTo map selector
    let map_entries: Vec<(String, String)> = project
        .maps
        .iter()
        .map(|(id, m)| (id.clone(), m.name.clone()))
        .collect();

    egui::Window::new("Event Trigger Editor")
        .collapsible(false)
        .resizable(true)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.label(format!(
                "Tile ({}, {}) — Layer {}",
                dialog.tile_x, dialog.tile_y, dialog.layer_index
            ));
            ui.separator();

            let dialog = &mut *dialog;
            render_action_editor(
                ui,
                &mut dialog.actions,
                &mut dialog.action_editor,
                "event_trigger",
                &map_entries,
                &project.face_portraits,
            );

            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("Save").clicked() {
                    should_save = true;
                }
                if ui.button("Cancel").clicked() {
                    should_close = true;
                }
            });
        });

    if should_save {
        let layer_index = dialog.layer_index;
        let x = dialog.tile_x;
        let y = dialog.tile_y;
        let old_trigger = dialog.original_actions.clone();
        let new_trigger = dialog.actions.clone();

        // Apply the change to the map
        if let Some(map) = project.active_map_mut()
            && let Some(layer) = map.layers.get_mut(layer_index)
            && let Some(attr_row) = layer.attributes.cells.get_mut(y as usize)
            && let Some(cell) = attr_row.get_mut(x as usize)
        {
            cell.event_trigger = new_trigger.clone();
        }

        edit_events.write(EditCommand {
            kind: EditCommandKind::SetEventTrigger {
                layer_index,
                x,
                y,
                old_trigger,
                new_trigger,
            },
        });

        dialog.open = false;
    }

    if should_close {
        dialog.open = false;
    }

    Ok(())
}
