use std::collections::HashMap;

use bevy::prelude::*;
use bevy_egui::egui;

use crate::data::project::Project;
use crate::plugins::app_shell::ErrorDialog;
use rpg_toolkit_common::{
    CharacterSpritesheet, ProjectFile, SpritesheetId, validate_spritesheet_dimensions,
};

/// Plugin that provides the spritesheet management panel.
///
/// This plugin only registers resources. The UI is rendered inline by the
/// tile palette plugin via [`spritesheet_section_ui`].
pub struct SpritesheetPlugin;

impl Plugin for SpritesheetPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RemoveSpritesheetDialog>();
    }
}

/// Confirmation dialog shown when removing a spritesheet that has references.
#[derive(Resource, Default)]
pub struct RemoveSpritesheetDialog {
    pub open: bool,
    pub spritesheet_id: Option<SpritesheetId>,
    pub npc_count: usize,
    pub player_reference: bool,
}

/// Renders the spritesheet management section inside the tile palette side panel.
///
/// This is called by `tile_palette_ui` at the bottom of the right panel.
/// It draws a collapsible header (collapsed by default) containing:
/// - Import button
/// - Player spritesheet assignment dropdown
/// - List of loaded spritesheets with remove buttons
/// - A small preview of the currently assigned player sprite
pub fn spritesheet_section_ui(
    ui: &mut egui::Ui,
    project: &mut ResMut<Project>,
    error_dialog: &mut ResMut<ErrorDialog>,
    spritesheet_textures: &HashMap<SpritesheetId, egui::TextureId>,
) {
    let header = egui::CollapsingHeader::new("Spritesheets")
        .default_open(false)
        .show(ui, |ui| {
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
                ui.label("Player:");
                let current_label = match &project.player_spritesheet {
                    Some(id) => short_spritesheet_label(
                        project.spritesheets.get(id).map(|ss| ss.file_path.as_str()),
                    ),
                    None => "None".to_string(),
                };

                let spritesheet_ids: Vec<SpritesheetId> =
                    project.spritesheets.keys().cloned().collect();
                let spritesheet_labels: Vec<String> = spritesheet_ids
                    .iter()
                    .map(|id| {
                        short_spritesheet_label(
                            project.spritesheets.get(id).map(|ss| ss.file_path.as_str()),
                        )
                    })
                    .collect();

                let mut selected = project.player_spritesheet.clone();

                egui::ComboBox::from_id_salt("player_spritesheet_combo")
                    .selected_text(&current_label)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut selected, None, "None (solid color)");
                        for (idx, id) in spritesheet_ids.iter().enumerate() {
                            let label = &spritesheet_labels[idx];
                            ui.selectable_value(&mut selected, Some(id.clone()), label);
                        }
                    });

                project.player_spritesheet = selected;
            });

            // Player sprite preview
            if let Some(ref player_id) = project.player_spritesheet
                && let Some(tex_id) = spritesheet_textures.get(player_id)
                && let Some(ss) = project.spritesheets.get(player_id)
            {
                ui.separator();
                ui.label("Preview:");

                // Show the first frame of the "down" direction (row 0, col 0)
                let sheet_w = (ss.sprite_width * ss.frame_count) as f32;
                let sheet_h = (ss.sprite_height * ss.direction_count) as f32;

                let uv_min = egui::pos2(0.0, 0.0);
                let uv_max = egui::pos2(
                    ss.sprite_width as f32 / sheet_w,
                    ss.sprite_height as f32 / sheet_h,
                );

                // Scale up for visibility (3× native size)
                let preview_w = ss.sprite_width as f32 * 3.0;
                let preview_h = ss.sprite_height as f32 * 3.0;

                let image = egui::Image::new(egui::load::SizedTexture::new(
                    *tex_id,
                    [preview_w, preview_h],
                ))
                .uv(egui::Rect::from_min_max(uv_min, uv_max));

                ui.add(image);
            }

            ui.separator();

            // List loaded spritesheets
            let ids: Vec<SpritesheetId> = project.spritesheets.keys().cloned().collect();

            if ids.is_empty() {
                ui.label("No spritesheets loaded.");
            }

            for id in &ids {
                if let Some(ss) = project.spritesheets.get(id) {
                    ui.group(|ui| {
                        ui.label(short_spritesheet_label(Some(&ss.file_path)));
                        ui.label(format!(
                            "{}×{} ({}×{} sprites)",
                            ss.sprite_width * ss.frame_count,
                            ss.sprite_height * ss.direction_count,
                            ss.sprite_width,
                            ss.sprite_height
                        ));
                        if ui.small_button("Remove").clicked() {
                            ui.memory_mut(|mem| {
                                mem.data.insert_temp(
                                    egui::Id::new("pending_spritesheet_remove"),
                                    id.clone(),
                                );
                            });
                        }
                    });
                }
            }
        });

    // We intentionally ignore `header` — the collapsing state is managed by egui.
    let _ = header;
}

/// Processes a pending spritesheet remove request and shows the confirmation
/// dialog if the spritesheet has references.
///
/// Called by `tile_palette_ui` after the side panel, so the dialog renders
/// as a centered window on top of everything.
pub fn process_spritesheet_remove(
    ctx: &egui::Context,
    project: &mut ResMut<Project>,
    remove_dialog: &mut ResMut<RemoveSpritesheetDialog>,
) {
    // Check for a pending remove request stashed in egui temp memory
    let pending: Option<SpritesheetId> = ctx.memory(|mem| {
        mem.data
            .get_temp::<SpritesheetId>(egui::Id::new("pending_spritesheet_remove"))
    });

    if let Some(remove_id) = pending {
        // Clear the temp so we don't re-process next frame
        ctx.memory_mut(|mem| {
            mem.data
                .remove::<SpritesheetId>(egui::Id::new("pending_spritesheet_remove"));
        });

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
            HashMap::new(),
            HashMap::new(),
            project.characters.clone(),
            project.items.clone(),
            rpg_toolkit_common::AbilityRegistry::default(),
            rpg_toolkit_common::EnemyRegistry::default(),
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
}

/// Extracts a short label from a spritesheet file path (just the filename).
fn short_spritesheet_label(file_path: Option<&str>) -> String {
    match file_path {
        Some(path) => std::path::Path::new(path)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown")
            .to_string(),
        None => "Invalid".to_string(),
    }
}
