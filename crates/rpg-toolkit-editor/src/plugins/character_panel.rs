use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

use rpg_toolkit_common::{CharacterRegistry, OPTIONAL_STATS, REQUIRED_STATS};

use crate::data::AppEditorMode;
use crate::data::EditorUiSet;

/// Plugin that provides the Character Editor panel UI.
pub struct CharacterPanelPlugin;

impl Plugin for CharacterPanelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CharacterPanelState>().add_systems(
            EguiPrimaryContextPass,
            character_panel_ui
                .in_set(EditorUiSet::Panels)
                .run_if(resource_equals(AppEditorMode::Character)),
        );
    }
}

/// UI state for the Character Editor panel.
#[derive(Resource)]
pub struct CharacterPanelState {
    /// Currently selected character ID.
    pub selected_character: Option<String>,
    /// Whether the "Create Character" dialog is open.
    pub create_dialog_open: bool,
    /// Text buffer for the character name in the create dialog.
    pub create_name_buffer: String,
    /// Validation error for the create dialog.
    pub create_error: Option<String>,
    /// Character ID pending delete confirmation.
    pub delete_confirm_target: Option<String>,
    /// Level used for stat preview (1..=99).
    pub preview_level: u32,
    /// Text buffer for inline name editing.
    pub name_edit_buffer: String,
    /// Validation error for inline name editing.
    pub name_edit_error: Option<String>,
}

impl Default for CharacterPanelState {
    fn default() -> Self {
        Self {
            selected_character: None,
            create_dialog_open: false,
            create_name_buffer: String::new(),
            create_error: None,
            delete_confirm_target: None,
            preview_level: 1,
            name_edit_buffer: String::new(),
            name_edit_error: None,
        }
    }
}

fn character_panel_ui(
    mut contexts: EguiContexts,
    mut panel_state: ResMut<CharacterPanelState>,
    mut project: ResMut<crate::data::Project>,
) -> Result {
    let ctx = contexts.ctx_mut()?;

    // === Left SidePanel: Character List ===
    egui::SidePanel::left("character_list")
        .default_width(200.0)
        .show(ctx, |ui| {
            // "New Character" button at the top
            if ui.button("➕ New Character").clicked() {
                panel_state.create_dialog_open = true;
                panel_state.create_name_buffer.clear();
                panel_state.create_error = None;
            }

            ui.separator();

            let sorted = project.characters.sorted_characters();

            if sorted.is_empty() {
                ui.label("No characters yet. Create one to get started.");
            } else {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    // Collect character info to avoid borrow conflicts
                    let char_entries: Vec<(String, String)> = sorted
                        .iter()
                        .map(|c| (c.id.clone(), c.display_name.clone()))
                        .collect();

                    for (id, display_name) in &char_entries {
                        let is_selected = panel_state.selected_character.as_ref() == Some(id);

                        ui.horizontal(|ui| {
                            if ui.selectable_label(is_selected, display_name).clicked() {
                                panel_state.selected_character = Some(id.clone());
                                // Sync name_edit_buffer with newly selected character
                                panel_state.name_edit_buffer = display_name.clone();
                                panel_state.name_edit_error = None;
                            }

                            // Delete button per character
                            if ui.small_button("🗑").clicked() {
                                panel_state.delete_confirm_target = Some(id.clone());
                            }
                        });
                    }
                });
            }
        });

    // === Right SidePanel: Stat Progression Preview ===
    egui::SidePanel::right("stat_preview")
        .default_width(200.0)
        .show(ctx, |ui| {
            ui.heading("Stat Preview");
            ui.separator();

            // Level input clamped to 1..=99
            ui.horizontal(|ui| {
                ui.label("Preview Level:");
                let drag = egui::DragValue::new(&mut panel_state.preview_level).range(1..=99);
                ui.add(drag);
            });

            // Clamp in case of manual edits
            panel_state.preview_level = panel_state.preview_level.clamp(1, 99);

            ui.separator();

            if let Some(ref selected_id) = panel_state.selected_character.clone() {
                if let Some(character) = project.characters.characters.get(selected_id) {
                    let preview_level = panel_state.preview_level;

                    egui::Grid::new("stat_preview_grid")
                        .striped(true)
                        .show(ui, |ui| {
                            ui.strong("Stat");
                            ui.strong("Value");
                            ui.end_row();

                            for stat in &character.stats {
                                let computed =
                                    CharacterRegistry::compute_stat_value(stat, preview_level);
                                ui.label(&stat.name);
                                ui.label(format!("{}", computed));
                                ui.end_row();
                            }
                        });
                } else {
                    ui.label("Character not found.");
                }
            } else {
                ui.label("Select a character to preview stats.");
            }
        });

    // === CentralPanel: Character Detail Editor ===
    egui::CentralPanel::default().show(ctx, |ui| {
        if let Some(selected_id) = panel_state.selected_character.clone() {
            if project.characters.characters.contains_key(&selected_id) {
                // --- Display Name Field ---
                ui.heading("Display Name");
                let name_response = ui.text_edit_singleline(&mut panel_state.name_edit_buffer);

                // Truncate to 50 chars
                if panel_state.name_edit_buffer.len() > 50 {
                    panel_state.name_edit_buffer.truncate(50);
                }

                // Validate on lost focus or Enter key
                if name_response.lost_focus() {
                    let trimmed = panel_state.name_edit_buffer.trim().to_string();
                    if trimmed.is_empty() || !trimmed.contains(|c: char| !c.is_whitespace()) {
                        panel_state.name_edit_error =
                            Some("Name must not be empty or whitespace-only.".to_string());
                    } else if trimmed.len() > 50 {
                        panel_state.name_edit_error =
                            Some("Name must not exceed 50 characters.".to_string());
                    } else {
                        match project.characters.rename_character(&selected_id, &trimmed) {
                            Ok(()) => {
                                panel_state.name_edit_error = None;
                                // Sync buffer with the actual stored name
                                panel_state.name_edit_buffer = trimmed;
                                project.has_unsaved_character_changes = true;
                            }
                            Err(e) => {
                                panel_state.name_edit_error = Some(e.to_string());
                            }
                        }
                    }
                }

                // Show validation error
                if let Some(ref error) = panel_state.name_edit_error {
                    ui.colored_label(egui::Color32::RED, error);
                }

                ui.separator();

                // --- Stat Table ---
                ui.heading("Stats");

                // Clone stat data to avoid borrow conflicts
                let stats_snapshot: Vec<(String, u32, u32)> = project
                    .characters
                    .characters
                    .get(&selected_id)
                    .map(|c| {
                        c.stats
                            .iter()
                            .map(|s| (s.name.clone(), s.base_value, s.growth_value))
                            .collect()
                    })
                    .unwrap_or_default();

                let mut stat_updates: Vec<(String, u32, u32)> = Vec::new();
                let mut stat_to_remove: Option<String> = None;

                egui::Grid::new("stat_editor_grid")
                    .striped(true)
                    .min_col_width(80.0)
                    .show(ui, |ui| {
                        // Header row
                        ui.strong("Stat");
                        ui.strong("Base Value");
                        ui.strong("Growth Value");
                        ui.strong(""); // Action column
                        ui.end_row();

                        for (stat_name, base_value, growth_value) in &stats_snapshot {
                            let mut base = *base_value;
                            let mut growth = *growth_value;
                            let is_required = REQUIRED_STATS
                                .iter()
                                .any(|(name, _, _)| *name == stat_name.as_str());

                            ui.label(stat_name);

                            let base_changed = ui
                                .add(egui::DragValue::new(&mut base).range(0..=u32::MAX))
                                .changed();

                            let growth_changed = ui
                                .add(egui::DragValue::new(&mut growth).range(0..=u32::MAX))
                                .changed();

                            if base_changed || growth_changed {
                                stat_updates.push((stat_name.clone(), base, growth));
                            }

                            if is_required {
                                // No delete button for required stats
                                ui.label("");
                            } else if ui.small_button("🗑").clicked() {
                                stat_to_remove = Some(stat_name.clone());
                            }

                            ui.end_row();
                        }
                    });

                // Apply stat updates
                for (stat_name, base, growth) in stat_updates {
                    let _ = project
                        .characters
                        .update_stat(&selected_id, &stat_name, base, growth);
                    project.has_unsaved_character_changes = true;
                }

                // Apply stat removal
                if let Some(stat_name) = stat_to_remove {
                    let _ = project.characters.remove_stat(&selected_id, &stat_name);
                    project.has_unsaved_character_changes = true;
                }

                ui.separator();

                // --- Add Stat Section ---
                // Determine which optional stats are not yet assigned
                let assigned_stat_names: Vec<String> = project
                    .characters
                    .characters
                    .get(&selected_id)
                    .map(|c| c.stats.iter().map(|s| s.name.clone()).collect())
                    .unwrap_or_default();

                let available_stats: Vec<&str> = OPTIONAL_STATS
                    .iter()
                    .filter(|s| !assigned_stat_names.contains(&s.to_string()))
                    .copied()
                    .collect();

                if available_stats.is_empty() {
                    ui.label("All stats assigned");
                } else {
                    ui.horizontal(|ui| {
                        ui.label("Add Stat:");
                        let mut selected_stat: Option<&str> = None;
                        egui::ComboBox::from_id_salt("add_stat_combo")
                            .selected_text("Select a stat...")
                            .show_ui(ui, |ui| {
                                for stat in &available_stats {
                                    if ui.selectable_label(false, *stat).clicked() {
                                        selected_stat = Some(*stat);
                                    }
                                }
                            });

                        if let Some(stat_name) = selected_stat {
                            let _ = project.characters.add_stat(&selected_id, stat_name);
                            project.has_unsaved_character_changes = true;
                        }
                    });
                }
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label("Select a character to edit");
                });
            }
        } else {
            ui.centered_and_justified(|ui| {
                ui.label("Select a character to edit");
            });
        }
    });

    // === Create Character Dialog ===
    if panel_state.create_dialog_open {
        let mut still_open = true;
        let mut should_create = false;
        let mut should_cancel = false;

        egui::Window::new("New Character")
            .collapsible(false)
            .resizable(false)
            .open(&mut still_open)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Name:");
                    let response = ui.text_edit_singleline(&mut panel_state.create_name_buffer);
                    // Limit to 50 characters
                    if panel_state.create_name_buffer.len() > 50 {
                        panel_state.create_name_buffer.truncate(50);
                    }
                    // Enter key to confirm
                    if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        should_create = true;
                    }
                });

                // Show error inline if present
                if let Some(ref error) = panel_state.create_error {
                    ui.colored_label(egui::Color32::RED, error);
                }

                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("Create").clicked() {
                        should_create = true;
                    }
                    if ui.button("Cancel").clicked() {
                        should_cancel = true;
                    }
                });
            });

        if !still_open {
            // Window was closed via X button — treat as cancel
            panel_state.create_dialog_open = false;
            panel_state.create_name_buffer.clear();
            panel_state.create_error = None;
        }

        if should_cancel {
            panel_state.create_dialog_open = false;
            panel_state.create_name_buffer.clear();
            panel_state.create_error = None;
        }

        if should_create {
            let name = panel_state.create_name_buffer.trim();
            if name.is_empty() || !name.contains(|c: char| !c.is_whitespace()) {
                panel_state.create_error =
                    Some("Name must not be empty or whitespace-only.".to_string());
            } else {
                match project
                    .characters
                    .create_character(&panel_state.create_name_buffer)
                {
                    Ok(new_id) => {
                        // Auto-select the newly created character
                        let display_name = project
                            .characters
                            .characters
                            .get(&new_id)
                            .map(|c| c.display_name.clone())
                            .unwrap_or_default();
                        panel_state.selected_character = Some(new_id);
                        panel_state.name_edit_buffer = display_name;
                        panel_state.name_edit_error = None;
                        panel_state.create_dialog_open = false;
                        panel_state.create_name_buffer.clear();
                        panel_state.create_error = None;
                        project.has_unsaved_character_changes = true;
                    }
                    Err(e) => {
                        panel_state.create_error = Some(e.to_string());
                    }
                }
            }
        }
    }

    // === Delete Confirmation Dialog ===
    if panel_state.delete_confirm_target.is_some() {
        let target_id = panel_state.delete_confirm_target.clone().unwrap();
        let character_name = project
            .characters
            .characters
            .get(&target_id)
            .map(|c| c.display_name.clone())
            .unwrap_or_else(|| "Unknown".to_string());

        let mut should_delete = false;
        let mut should_cancel = false;
        let mut still_open = true;

        egui::Window::new("Confirm Delete")
            .collapsible(false)
            .resizable(false)
            .open(&mut still_open)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.label(format!(
                    "Are you sure you want to delete \"{}\"?",
                    character_name
                ));
                ui.label("This action cannot be undone.");
                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("Delete").clicked() {
                        should_delete = true;
                    }
                    if ui.button("Cancel").clicked() {
                        should_cancel = true;
                    }
                });
            });

        if !still_open {
            panel_state.delete_confirm_target = None;
        }

        if should_cancel {
            panel_state.delete_confirm_target = None;
        }

        if should_delete {
            let _ = project.characters.delete_character(&target_id);
            project.has_unsaved_character_changes = true;

            // If the deleted character was selected, select the first remaining or clear
            if panel_state.selected_character.as_ref() == Some(&target_id) {
                let sorted = project.characters.sorted_characters();
                if let Some(first) = sorted.first() {
                    panel_state.selected_character = Some(first.id.clone());
                    panel_state.name_edit_buffer = first.display_name.clone();
                    panel_state.name_edit_error = None;
                } else {
                    panel_state.selected_character = None;
                    panel_state.name_edit_buffer.clear();
                    panel_state.name_edit_error = None;
                }
            }

            panel_state.delete_confirm_target = None;
        }
    }

    Ok(())
}
