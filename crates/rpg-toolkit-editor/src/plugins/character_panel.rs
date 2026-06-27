use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

use rpg_toolkit_common::{
    AbilityCategory, CharacterRegistry, ItemCategory, OPTIONAL_STATS, REQUIRED_STATS,
    VisualAssetType,
};

use crate::data::AppEditorMode;
use crate::data::EditorUiSet;
use crate::plugins::searchable_combobox::searchable_combobox;

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
    /// Search buffer for the learnable ability searchable combobox.
    pub add_learnable_search_buffer: String,
    /// Level value for the new learnable ability (default 1).
    pub add_learnable_level: u32,
    /// Error message for learnable ability addition.
    pub add_learnable_error: Option<String>,
    /// Text buffer for spritesheet path editing.
    pub spritesheet_buffer: String,
    /// Text buffer for face portrait path editing.
    pub face_portrait_buffer: String,
    /// Text buffer for status portrait path editing.
    pub status_portrait_buffer: String,
    /// Search buffer for the starting equipment searchable combobox.
    pub starting_equipment_search_buffer: String,
    /// Error message for starting equipment addition.
    pub starting_equipment_error: Option<String>,
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
            add_learnable_search_buffer: String::new(),
            add_learnable_level: 1,
            add_learnable_error: None,
            spritesheet_buffer: String::new(),
            face_portrait_buffer: String::new(),
            status_portrait_buffer: String::new(),
            starting_equipment_search_buffer: String::new(),
            starting_equipment_error: None,
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
                                // Sync visual asset buffers
                                if let Some(character) = project.characters.characters.get(id) {
                                    panel_state.spritesheet_buffer = character
                                        .visual_assets
                                        .spritesheet
                                        .clone()
                                        .unwrap_or_default();
                                    panel_state.face_portrait_buffer = character
                                        .visual_assets
                                        .face_portrait
                                        .clone()
                                        .unwrap_or_default();
                                    panel_state.status_portrait_buffer = character
                                        .visual_assets
                                        .status_portrait
                                        .clone()
                                        .unwrap_or_default();
                                }
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

                ui.separator();

                // --- Learnable Abilities Section ---
                ui.heading("Learnable Abilities");

                if project.abilities.abilities.is_empty() {
                    ui.label("No abilities available");
                } else {
                    // Display existing learnable abilities sorted by required_level ascending
                    let mut learnable_entries: Vec<(String, String, u32)> = project
                        .characters
                        .characters
                        .get(&selected_id)
                        .map(|c| {
                            c.learnable_abilities
                                .iter()
                                .map(|la| {
                                    let display_name = project
                                        .abilities
                                        .abilities
                                        .get(&la.ability_id)
                                        .map(|a| a.display_name.clone())
                                        .unwrap_or_else(|| la.ability_id.clone());
                                    (la.ability_id.clone(), display_name, la.required_level)
                                })
                                .collect()
                        })
                        .unwrap_or_default();
                    learnable_entries.sort_by_key(|(_, _, level)| *level);

                    let mut ability_to_remove: Option<String> = None;
                    let mut level_updates: Vec<(String, u32)> = Vec::new();

                    for (ability_id, display_name, required_level) in &learnable_entries {
                        ui.horizontal(|ui| {
                            ui.label(format!("{} (Lv. {})", display_name, required_level));

                            let mut level = *required_level;
                            if ui
                                .add(egui::DragValue::new(&mut level).range(1..=99))
                                .changed()
                            {
                                level_updates.push((ability_id.clone(), level));
                            }

                            if ui.small_button("🗑").clicked() {
                                ability_to_remove = Some(ability_id.clone());
                            }
                        });
                    }

                    // Apply level updates
                    for (ability_id, new_level) in level_updates {
                        let _ = project.characters.update_learnable_ability_level(
                            &selected_id,
                            &ability_id,
                            new_level,
                        );
                        project.has_unsaved_character_changes = true;
                    }

                    // Apply removal
                    if let Some(ability_id) = ability_to_remove {
                        let _ = project
                            .characters
                            .remove_learnable_ability(&selected_id, &ability_id);
                        project.has_unsaved_character_changes = true;
                    }

                    ui.separator();

                    // Add learnable ability controls
                    ui.horizontal(|ui| {
                        ui.label("Level:");
                        ui.add(
                            egui::DragValue::new(&mut panel_state.add_learnable_level)
                                .range(1..=99),
                        );
                    });

                    // Clamp in case of manual edits
                    panel_state.add_learnable_level = panel_state.add_learnable_level.clamp(1, 99);

                    // Build items list from ability registry (sorted alphabetically by display name)
                    let items: Vec<(String, String)> = project
                        .abilities
                        .filtered_abilities(None)
                        .iter()
                        .map(|ability| {
                            let category_name = match ability.category {
                                AbilityCategory::Skill => "Skill",
                                AbilityCategory::Spell => "Spell",
                                AbilityCategory::SpecialAction => "Special Action",
                                AbilityCategory::Monster => "Monster",
                            };
                            (
                                ability.id.clone(),
                                format!("{} [{}]", ability.display_name, category_name),
                            )
                        })
                        .collect();

                    if let Some(selected_ability_id) = searchable_combobox(
                        ui,
                        "character_add_learnable_ability",
                        "Select ability…",
                        &items,
                        &mut panel_state.add_learnable_search_buffer,
                    ) {
                        match project.characters.add_learnable_ability(
                            &selected_id,
                            selected_ability_id,
                            panel_state.add_learnable_level,
                        ) {
                            Ok(()) => {
                                panel_state.add_learnable_error = None;
                                project.has_unsaved_character_changes = true;
                            }
                            Err(e) => {
                                panel_state.add_learnable_error = Some(e.to_string());
                            }
                        }
                    }

                    if let Some(ref error) = panel_state.add_learnable_error {
                        ui.colored_label(egui::Color32::RED, error);
                    }
                }

                ui.separator();

                // --- Visual Assets Section ---
                ui.heading("Visual Assets");

                // Helper macro-like approach: iterate over the three asset types
                let asset_fields: Vec<(&str, VisualAssetType)> = vec![
                    ("Spritesheet", VisualAssetType::Spritesheet),
                    ("Face Portrait", VisualAssetType::FacePortrait),
                    ("Status Portrait", VisualAssetType::StatusPortrait),
                ];

                for (label, asset_type) in &asset_fields {
                    ui.label(*label);

                    // Check if current value is None to show placeholder
                    let current_value =
                        project
                            .characters
                            .characters
                            .get(&selected_id)
                            .and_then(|c| match asset_type {
                                VisualAssetType::Spritesheet => {
                                    c.visual_assets.spritesheet.as_ref()
                                }
                                VisualAssetType::FacePortrait => {
                                    c.visual_assets.face_portrait.as_ref()
                                }
                                VisualAssetType::StatusPortrait => {
                                    c.visual_assets.status_portrait.as_ref()
                                }
                            });

                    if current_value.is_none() {
                        ui.label("No asset assigned");
                    }

                    // Get mutable reference to the appropriate buffer
                    let buffer = match asset_type {
                        VisualAssetType::Spritesheet => &mut panel_state.spritesheet_buffer,
                        VisualAssetType::FacePortrait => &mut panel_state.face_portrait_buffer,
                        VisualAssetType::StatusPortrait => &mut panel_state.status_portrait_buffer,
                    };

                    let response = ui.text_edit_singleline(buffer);

                    // Truncate to 260 chars
                    let buffer = match asset_type {
                        VisualAssetType::Spritesheet => &mut panel_state.spritesheet_buffer,
                        VisualAssetType::FacePortrait => &mut panel_state.face_portrait_buffer,
                        VisualAssetType::StatusPortrait => &mut panel_state.status_portrait_buffer,
                    };
                    if buffer.chars().count() > 260 {
                        let truncated: String = buffer.chars().take(260).collect();
                        *buffer = truncated;
                    }

                    // On lost focus: trim, if empty → set to None, otherwise store
                    if response.lost_focus() {
                        let buffer = match asset_type {
                            VisualAssetType::Spritesheet => &mut panel_state.spritesheet_buffer,
                            VisualAssetType::FacePortrait => &mut panel_state.face_portrait_buffer,
                            VisualAssetType::StatusPortrait => {
                                &mut panel_state.status_portrait_buffer
                            }
                        };
                        let trimmed = buffer.trim().to_string();
                        if trimmed.is_empty() {
                            let _ = project
                                .characters
                                .clear_visual_asset(&selected_id, *asset_type);
                            buffer.clear();
                        } else {
                            let _ = project.characters.set_visual_asset(
                                &selected_id,
                                *asset_type,
                                &trimmed,
                            );
                            *buffer = trimmed;
                        }
                        project.has_unsaved_character_changes = true;
                    }

                    ui.horizontal(|ui| {
                        // Browse... button — opens native file dialog for image selection
                        if ui.button("Browse...").clicked() {
                            let file = rfd::FileDialog::new()
                                .add_filter("Images", &["png", "jpg", "jpeg"])
                                .pick_file();

                            if let Some(path) = file {
                                let path_str = path.display().to_string();
                                // Truncate to 260 characters
                                let truncated: String = path_str.chars().take(260).collect();

                                // Populate the buffer
                                let buffer = match asset_type {
                                    VisualAssetType::Spritesheet => {
                                        &mut panel_state.spritesheet_buffer
                                    }
                                    VisualAssetType::FacePortrait => {
                                        &mut panel_state.face_portrait_buffer
                                    }
                                    VisualAssetType::StatusPortrait => {
                                        &mut panel_state.status_portrait_buffer
                                    }
                                };
                                *buffer = truncated.clone();

                                // Commit to the character model immediately
                                let _ = project.characters.set_visual_asset(
                                    &selected_id,
                                    *asset_type,
                                    &truncated,
                                );
                                project.has_unsaved_character_changes = true;
                            }
                            // If dialog is cancelled (file is None), leave buffer unchanged
                        }

                        // Clear button
                        if ui.button("Clear").clicked() {
                            let _ = project
                                .characters
                                .clear_visual_asset(&selected_id, *asset_type);
                            let buffer = match asset_type {
                                VisualAssetType::Spritesheet => &mut panel_state.spritesheet_buffer,
                                VisualAssetType::FacePortrait => {
                                    &mut panel_state.face_portrait_buffer
                                }
                                VisualAssetType::StatusPortrait => {
                                    &mut panel_state.status_portrait_buffer
                                }
                            };
                            buffer.clear();
                            project.has_unsaved_character_changes = true;
                        }
                    });

                    ui.separator();
                }

                // --- Starting Equipment Section ---
                ui.heading("Starting Equipment");

                if project.items.items.is_empty() {
                    ui.label("No items available");
                } else {
                    // Display current starting equipment sorted by display name (case-insensitive)
                    let starting_equipment: Vec<String> = project
                        .characters
                        .characters
                        .get(&selected_id)
                        .map(|c| c.starting_equipment.clone())
                        .unwrap_or_default();

                    // Build sorted display entries: (item_id, display_label)
                    let mut display_entries: Vec<(String, String)> = starting_equipment
                        .iter()
                        .map(|item_id| {
                            let label = if let Some(item) = project.items.items.get(item_id) {
                                let category_name = match item.category() {
                                    ItemCategory::Weapon => "Weapon",
                                    ItemCategory::Armor => "Armor",
                                    ItemCategory::Accessory => "Accessory",
                                    ItemCategory::Consumable => "Consumable",
                                    ItemCategory::KeyItem => "Key Item",
                                };
                                format!("{} [{}]", item.display_name, category_name)
                            } else {
                                item_id.clone()
                            };
                            (item_id.clone(), label)
                        })
                        .collect();
                    display_entries.sort_by(|a, b| a.1.to_lowercase().cmp(&b.1.to_lowercase()));

                    let mut item_to_remove: Option<String> = None;

                    for (item_id, label) in &display_entries {
                        ui.horizontal(|ui| {
                            ui.label(label);

                            if ui.small_button("🗑").clicked() {
                                item_to_remove = Some(item_id.clone());
                            }
                        });
                    }

                    if let Some(item_id) = item_to_remove {
                        let _ = project
                            .characters
                            .remove_starting_equipment(&selected_id, &item_id);
                        project.has_unsaved_character_changes = true;
                    }

                    // Add starting equipment via searchable dropdown
                    let items: Vec<(String, String)> = project
                        .items
                        .filtered_items(None)
                        .iter()
                        .map(|item| {
                            let category_name = match item.category() {
                                ItemCategory::Weapon => "Weapon",
                                ItemCategory::Armor => "Armor",
                                ItemCategory::Accessory => "Accessory",
                                ItemCategory::Consumable => "Consumable",
                                ItemCategory::KeyItem => "Key Item",
                            };
                            (
                                item.id.clone(),
                                format!("{} [{}]", item.display_name, category_name),
                            )
                        })
                        .collect();

                    if let Some(selected_item_id) = searchable_combobox(
                        ui,
                        "character_starting_equipment",
                        "Select item…",
                        &items,
                        &mut panel_state.starting_equipment_search_buffer,
                    ) {
                        match project
                            .characters
                            .add_starting_equipment(&selected_id, &selected_item_id)
                        {
                            Ok(()) => {
                                panel_state.starting_equipment_error = None;
                                project.has_unsaved_character_changes = true;
                            }
                            Err(e) => {
                                panel_state.starting_equipment_error = Some(e.to_string());
                            }
                        }
                    }

                    if let Some(ref error) = panel_state.starting_equipment_error {
                        ui.colored_label(egui::Color32::RED, error);
                    }
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
                        // New characters have no visual assets
                        panel_state.spritesheet_buffer.clear();
                        panel_state.face_portrait_buffer.clear();
                        panel_state.status_portrait_buffer.clear();
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
                    panel_state.spritesheet_buffer =
                        first.visual_assets.spritesheet.clone().unwrap_or_default();
                    panel_state.face_portrait_buffer = first
                        .visual_assets
                        .face_portrait
                        .clone()
                        .unwrap_or_default();
                    panel_state.status_portrait_buffer = first
                        .visual_assets
                        .status_portrait
                        .clone()
                        .unwrap_or_default();
                } else {
                    panel_state.selected_character = None;
                    panel_state.name_edit_buffer.clear();
                    panel_state.name_edit_error = None;
                    panel_state.spritesheet_buffer.clear();
                    panel_state.face_portrait_buffer.clear();
                    panel_state.status_portrait_buffer.clear();
                }
            }

            panel_state.delete_confirm_target = None;
        }
    }

    Ok(())
}
