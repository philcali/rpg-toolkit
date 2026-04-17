use std::collections::HashMap;

use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

use crate::data::project::Project;
use crate::plugins::app_shell::ErrorDialog;
use rpg_toolkit_common::{
    CharacterSpritesheet, ProjectFile, SpritesheetId, validate_spritesheet_dimensions,
};

/// Plugin that provides the spritesheet management panel.
pub struct SpritesheetPlugin;

impl Plugin for SpritesheetPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SpritesheetPanel>()
            .init_resource::<RemoveSpritesheetDialog>()
            .add_systems(EguiPrimaryContextPass, spritesheet_panel_ui);
    }
}

/// Controls whether the spritesheet management panel is open.
#[derive(Resource, Default)]
pub struct SpritesheetPanel {
    pub open: bool,
}

/// Confirmation dialog shown when removing a spritesheet that has references.
#[derive(Resource, Default)]
pub struct RemoveSpritesheetDialog {
    pub open: bool,
    pub spritesheet_id: Option<SpritesheetId>,
    pub npc_count: usize,
    pub player_reference: bool,
}

#[allow(clippy::too_many_arguments)]
fn spritesheet_panel_ui(
    mut contexts: EguiContexts,
    mut panel: ResMut<SpritesheetPanel>,
    mut remove_dialog: ResMut<RemoveSpritesheetDialog>,
    mut error_dialog: ResMut<ErrorDialog>,
    mut project: ResMut<Project>,
) -> Result {
    let ctx = contexts.ctx_mut()?;

    // Menu bar entry — add "Spritesheets" button to the top menu bar
    egui::TopBottomPanel::top("spritesheet_menu_bar")
        .show_separator_line(false)
        .show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                if ui.button("Spritesheets").clicked() {
                    panel.open = !panel.open;
                }
            });
        });

    // Spritesheet management window
    if panel.open {
        let mut still_open = panel.open;
        egui::Window::new("Spritesheet Manager")
            .collapsible(true)
            .resizable(true)
            .open(&mut still_open)
            .default_width(350.0)
            .show(ctx, |ui| {
                // Import button
                if ui.button("Import Spritesheet").clicked() {
                    let file = rfd::FileDialog::new()
                        .add_filter("Images", &["png", "jpg", "jpeg"])
                        .pick_file();

                    if let Some(path) = file {
                        match image::open(&path) {
                            Ok(img) => {
                                let (img_w, img_h) = (img.width(), img.height());
                                match validate_spritesheet_dimensions(img_w, img_h) {
                                    Ok(()) => {
                                        let id: SpritesheetId = uuid::Uuid::new_v4().to_string();
                                        let spritesheet = CharacterSpritesheet {
                                            file_path: path.to_string_lossy().to_string(),
                                            sprite_width: 24,
                                            sprite_height: 32,
                                            frame_count: 3,
                                            direction_count: 4,
                                        };
                                        project.spritesheets.insert(id, spritesheet);
                                    }
                                    Err(e) => {
                                        error_dialog.open = true;
                                        error_dialog.message = e.to_string();
                                    }
                                }
                            }
                            Err(e) => {
                                error_dialog.open = true;
                                error_dialog.message = format!("Failed to read image: {}", e);
                            }
                        }
                    }
                }

                ui.separator();

                // Player spritesheet assignment dropdown
                ui.horizontal(|ui| {
                    ui.label("Player Spritesheet:");
                    let current_label = match &project.player_spritesheet {
                        Some(id) => project
                            .spritesheets
                            .get(id)
                            .map(|ss| ss.file_path.clone())
                            .unwrap_or_else(|| "Invalid".to_string()),
                        None => "None (solid color)".to_string(),
                    };

                    // We need to collect IDs to avoid borrow issues
                    let spritesheet_ids: Vec<SpritesheetId> =
                        project.spritesheets.keys().cloned().collect();
                    let spritesheet_paths: Vec<String> = spritesheet_ids
                        .iter()
                        .map(|id| {
                            project
                                .spritesheets
                                .get(id)
                                .map(|ss| ss.file_path.clone())
                                .unwrap_or_default()
                        })
                        .collect();

                    let mut selected = project.player_spritesheet.clone();

                    egui::ComboBox::from_id_salt("player_spritesheet_combo")
                        .selected_text(&current_label)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut selected, None, "None (solid color)")
                                .clicked();
                            for (idx, id) in spritesheet_ids.iter().enumerate() {
                                let label = &spritesheet_paths[idx];
                                if ui
                                    .selectable_value(&mut selected, Some(id.clone()), label)
                                    .clicked()
                                {}
                            }
                        });

                    project.player_spritesheet = selected;
                });

                ui.separator();

                // List loaded spritesheets
                ui.label("Loaded Spritesheets:");
                let ids: Vec<SpritesheetId> = project.spritesheets.keys().cloned().collect();

                if ids.is_empty() {
                    ui.label("No spritesheets loaded.");
                }

                let mut remove_request: Option<SpritesheetId> = None;

                for id in &ids {
                    if let Some(ss) = project.spritesheets.get(id) {
                        ui.group(|ui| {
                            ui.label(format!("ID: {}", &id[..8.min(id.len())]));
                            ui.label(format!("File: {}", ss.file_path));
                            ui.label(format!(
                                "Dimensions: {}×{} ({}×{} sprites)",
                                ss.sprite_width * ss.frame_count,
                                ss.sprite_height * ss.direction_count,
                                ss.sprite_width,
                                ss.sprite_height
                            ));
                            if ui.button("Remove").clicked() {
                                remove_request = Some(id.clone());
                            }
                        });
                    }
                }

                // Handle remove request — check references
                if let Some(remove_id) = remove_request {
                    // Build a temporary ProjectFile to compute references
                    let tilesets_meta: HashMap<_, _> = project
                        .tilesets
                        .iter()
                        .map(|(id, entry)| (id.clone(), entry.meta.clone()))
                        .collect();
                    let temp_project_file = ProjectFile::new(
                        project.maps.clone(),
                        tilesets_meta,
                        project.spawn_point.clone(),
                        project.spritesheets.clone(),
                        project.player_spritesheet.clone(),
                    );
                    let refs = temp_project_file.compute_spritesheet_references(&remove_id);

                    if refs.npc_references.is_empty() && !refs.player_reference {
                        // No references — remove directly
                        project.spritesheets.remove(&remove_id);
                        if project.player_spritesheet.as_ref() == Some(&remove_id) {
                            project.player_spritesheet = None;
                        }
                    } else {
                        // Has references — show confirmation dialog
                        remove_dialog.open = true;
                        remove_dialog.spritesheet_id = Some(remove_id);
                        remove_dialog.npc_count = refs.npc_references.len();
                        remove_dialog.player_reference = refs.player_reference;
                    }
                }
            });

        if !still_open {
            panel.open = false;
        }
    }

    // Remove spritesheet confirmation dialog
    if remove_dialog.open {
        let mut still_open = true;
        let mut confirm = false;
        let mut cancel = false;

        egui::Window::new("Remove Spritesheet?")
            .collapsible(false)
            .resizable(false)
            .open(&mut still_open)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.label("This spritesheet is still referenced:");
                if remove_dialog.npc_count > 0 {
                    ui.label(format!(
                        "• {} NPC instance(s) reference this spritesheet",
                        remove_dialog.npc_count
                    ));
                }
                if remove_dialog.player_reference {
                    ui.label("• Player spritesheet is set to this spritesheet");
                }
                ui.separator();
                ui.label("Removing it will leave these references invalid.");
                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("Remove Anyway").clicked() {
                        confirm = true;
                    }
                    if ui.button("Cancel").clicked() {
                        cancel = true;
                    }
                });
            });

        if !still_open || cancel {
            remove_dialog.open = false;
            remove_dialog.spritesheet_id = None;
        }

        if confirm {
            if let Some(ref remove_id) = remove_dialog.spritesheet_id.clone() {
                project.spritesheets.remove(remove_id);
                if project.player_spritesheet.as_ref() == Some(remove_id) {
                    project.player_spritesheet = None;
                }
            }
            remove_dialog.open = false;
            remove_dialog.spritesheet_id = None;
        }
    }

    Ok(())
}
