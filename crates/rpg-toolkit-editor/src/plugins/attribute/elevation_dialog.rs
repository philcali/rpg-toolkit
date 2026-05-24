//! Inline dialog for setting tile elevation and target elevation values.

use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

use crate::data::Project;
use crate::data::commands::{EditCommand, EditCommandKind};

/// Resource for the elevation input dialog.
#[derive(Resource, Default)]
pub struct ElevationDialog {
    pub open: bool,
    pub layer_index: usize,
    pub tile_x: u32,
    pub tile_y: u32,
    /// The string buffer for the elevation input field.
    pub value_str: String,
    /// The old elevation value (for undo).
    pub old_value: u32,
}

/// Resource for the elevation transition (target elevation) input dialog.
#[derive(Resource, Default)]
pub struct ElevationTransitionDialog {
    pub open: bool,
    pub layer_index: usize,
    pub tile_x: u32,
    pub tile_y: u32,
    /// The string buffer for the target elevation input field.
    pub value_str: String,
    /// The old target elevation value (for undo).
    pub old_value: Option<u32>,
}

/// Egui dialog for setting tile elevation.
pub fn elevation_dialog_ui(
    mut contexts: EguiContexts,
    mut dialog: ResMut<ElevationDialog>,
    mut project: ResMut<Project>,
    mut edit_events: MessageWriter<EditCommand>,
) -> Result {
    if !dialog.open {
        return Ok(());
    }

    let ctx = contexts.ctx_mut()?;

    let mut should_confirm = false;
    let mut should_cancel = false;

    egui::Window::new("Set Elevation")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.label(format!(
                "Tile ({}, {}) — Layer {}",
                dialog.tile_x, dialog.tile_y, dialog.layer_index
            ));
            ui.separator();

            ui.horizontal(|ui| {
                ui.label("Elevation:");
                let response = ui.text_edit_singleline(&mut dialog.value_str);
                if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    should_confirm = true;
                }
            });

            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("OK").clicked() {
                    should_confirm = true;
                }
                if ui.button("Cancel").clicked() {
                    should_cancel = true;
                }
            });
        });

    if should_confirm {
        if let Ok(new_value) = dialog.value_str.trim().parse::<u32>() {
            let old_value = dialog.old_value;
            if new_value != old_value {
                // Apply the change to the map
                if let Some(map) = project.active_map_mut()
                    && let Some(layer) = map.layers.get_mut(dialog.layer_index)
                    && let Some(row) = layer.attributes.cells.get_mut(dialog.tile_y as usize)
                    && let Some(cell) = row.get_mut(dialog.tile_x as usize)
                {
                    cell.elevation = new_value;
                }

                edit_events.write(EditCommand {
                    kind: EditCommandKind::SetElevation {
                        layer_index: dialog.layer_index,
                        x: dialog.tile_x,
                        y: dialog.tile_y,
                        old_value,
                        new_value,
                    },
                });
            }
        }
        dialog.open = false;
    }

    if should_cancel {
        dialog.open = false;
    }

    Ok(())
}

/// Egui dialog for setting tile target elevation (elevation transition).
pub fn elevation_transition_dialog_ui(
    mut contexts: EguiContexts,
    mut dialog: ResMut<ElevationTransitionDialog>,
    mut project: ResMut<Project>,
    mut edit_events: MessageWriter<EditCommand>,
) -> Result {
    if !dialog.open {
        return Ok(());
    }

    let ctx = contexts.ctx_mut()?;

    let mut should_confirm = false;
    let mut should_cancel = false;
    let mut should_clear = false;

    egui::Window::new("Set Elevation Transition")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.label(format!(
                "Tile ({}, {}) — Layer {}",
                dialog.tile_x, dialog.tile_y, dialog.layer_index
            ));
            ui.separator();

            ui.horizontal(|ui| {
                ui.label("Target Elevation:");
                let response = ui.text_edit_singleline(&mut dialog.value_str);
                if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    should_confirm = true;
                }
            });

            ui.label("Leave empty or click Clear to remove the transition.");

            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("OK").clicked() {
                    should_confirm = true;
                }
                if ui.button("Clear").clicked() {
                    should_clear = true;
                }
                if ui.button("Cancel").clicked() {
                    should_cancel = true;
                }
            });
        });

    if should_confirm {
        let new_value = if dialog.value_str.trim().is_empty() {
            None
        } else if let Ok(v) = dialog.value_str.trim().parse::<u32>() {
            Some(v)
        } else {
            // Invalid input — close without applying
            dialog.open = false;
            return Ok(());
        };

        let old_value = dialog.old_value;
        if new_value != old_value {
            // Apply the change to the map
            if let Some(map) = project.active_map_mut()
                && let Some(layer) = map.layers.get_mut(dialog.layer_index)
                && let Some(row) = layer.attributes.cells.get_mut(dialog.tile_y as usize)
                && let Some(cell) = row.get_mut(dialog.tile_x as usize)
            {
                cell.target_elevation = new_value;
            }

            edit_events.write(EditCommand {
                kind: EditCommandKind::SetTargetElevation {
                    layer_index: dialog.layer_index,
                    x: dialog.tile_x,
                    y: dialog.tile_y,
                    old_value,
                    new_value,
                },
            });
        }
        dialog.open = false;
    }

    if should_clear {
        let old_value = dialog.old_value;
        if old_value.is_some() {
            // Apply the change to the map
            if let Some(map) = project.active_map_mut()
                && let Some(layer) = map.layers.get_mut(dialog.layer_index)
                && let Some(row) = layer.attributes.cells.get_mut(dialog.tile_y as usize)
                && let Some(cell) = row.get_mut(dialog.tile_x as usize)
            {
                cell.target_elevation = None;
            }

            edit_events.write(EditCommand {
                kind: EditCommandKind::SetTargetElevation {
                    layer_index: dialog.layer_index,
                    x: dialog.tile_x,
                    y: dialog.tile_y,
                    old_value,
                    new_value: None,
                },
            });
        }
        dialog.open = false;
    }

    if should_cancel {
        dialog.open = false;
    }

    Ok(())
}
