//! Confirmation dialog for moving the project spawn point.

use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

use crate::data::Project;
use crate::data::commands::{EditCommand, EditCommandKind};
use crate::data::map::{MapId, SpawnPoint};

/// Resource for the spawn point confirmation dialog.
#[derive(Resource, Default)]
pub struct SpawnPointConfirmDialog {
    pub open: bool,
    pub new_map_id: Option<MapId>,
    pub new_x: u32,
    pub new_y: u32,
}

/// Egui dialog for confirming spawn point relocation.
pub fn spawn_point_confirm_ui(
    mut contexts: EguiContexts,
    mut dialog: ResMut<SpawnPointConfirmDialog>,
    project: Res<Project>,
    mut edit_events: MessageWriter<EditCommand>,
) -> Result {
    if !dialog.open {
        return Ok(());
    }

    let ctx = contexts.ctx_mut()?;

    let mut should_confirm = false;
    let mut should_cancel = false;

    // Build info about the existing spawn point
    let existing_info = if let Some(ref sp) = project.spawn_point {
        let map_name = project
            .maps
            .get(&sp.map_id)
            .map(|m| m.name.as_str())
            .unwrap_or("Unknown");
        format!("Current spawn point: {} ({}, {})", map_name, sp.x, sp.y)
    } else {
        "No existing spawn point.".to_string()
    };

    let new_info = if let Some(ref new_map_id) = dialog.new_map_id {
        let map_name = project
            .maps
            .get(new_map_id)
            .map(|m| m.name.as_str())
            .unwrap_or("Unknown");
        format!(
            "New location: {} ({}, {})",
            map_name, dialog.new_x, dialog.new_y
        )
    } else {
        String::new()
    };

    egui::Window::new("Move Spawn Point?")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.label("A spawn point already exists.");
            ui.label(&existing_info);
            ui.separator();
            ui.label("Do you want to move it?");
            ui.label(&new_info);
            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("Move").clicked() {
                    should_confirm = true;
                }
                if ui.button("Cancel").clicked() {
                    should_cancel = true;
                }
            });
        });

    if should_confirm {
        if let Some(new_map_id) = dialog.new_map_id.take() {
            let old_spawn = project.spawn_point.clone();
            let new_spawn = Some(SpawnPoint {
                map_id: new_map_id,
                x: dialog.new_x,
                y: dialog.new_y,
            });

            edit_events.write(EditCommand {
                kind: EditCommandKind::SetSpawnPoint {
                    old_spawn,
                    new_spawn: new_spawn.clone(),
                },
            });
        }
        dialog.open = false;
    }

    if should_cancel {
        dialog.open = false;
        dialog.new_map_id = None;
    }

    Ok(())
}
