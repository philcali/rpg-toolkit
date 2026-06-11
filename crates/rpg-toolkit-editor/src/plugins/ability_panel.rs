use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

use rpg_toolkit_common::{AbilityCategory, AbilityId, AbilitySource, CostType, TargetType};

use crate::data::AppEditorMode;
use crate::data::EditorUiSet;

/// Plugin that provides the Ability Editor panel UI.
pub struct AbilityPanelPlugin;

impl Plugin for AbilityPanelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AbilityPanelState>().add_systems(
            EguiPrimaryContextPass,
            ability_panel_ui
                .in_set(EditorUiSet::Panels)
                .run_if(resource_equals(AppEditorMode::Ability)),
        );
    }
}

/// The type of source being added in the "Add Source" dialog.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AbilitySourceType {
    #[default]
    LevelUp,
    LearnedFromItem,
    EquipmentGrant,
    AccessoryGrant,
}

/// UI state for the Ability Editor panel.
#[derive(Resource, Default)]
pub struct AbilityPanelState {
    /// Currently selected ability ID.
    pub selected_ability: Option<AbilityId>,
    /// Active category filter for the ability list.
    pub category_filter: Option<AbilityCategory>,
    /// Whether the "Create Ability" dialog is open.
    pub create_dialog_open: bool,
    /// Text buffer for the ability name in the create dialog.
    pub create_name_buffer: String,
    /// Selected category in the create dialog.
    pub create_category: Option<AbilityCategory>,
    /// Validation error for the create dialog.
    pub create_error: Option<String>,
    /// Ability ID pending delete confirmation.
    pub delete_confirm_target: Option<AbilityId>,
    /// Text buffer for inline name editing.
    pub name_edit_buffer: String,
    /// Validation error for inline name editing.
    pub name_edit_error: Option<String>,
    /// Whether the "Add Source" dialog is open.
    pub add_source_dialog_open: bool,
    /// Selected source type in the add source dialog.
    pub add_source_type: AbilitySourceType,
    /// Text buffer for the level field in the add source dialog.
    pub add_source_level_buffer: String,
    /// Text buffer for the item ID field in the add source dialog.
    pub add_source_item_id_buffer: String,
    /// Validation error for the add source dialog.
    pub add_source_error: Option<String>,
}

/// Returns a human-readable display name for an ability category.
fn category_display_name(cat: AbilityCategory) -> &'static str {
    match cat {
        AbilityCategory::Skill => "Skill",
        AbilityCategory::Spell => "Spell",
        AbilityCategory::SpecialAction => "Special Action",
    }
}

/// Returns a human-readable display name for a target type.
fn target_type_display(tt: TargetType) -> &'static str {
    match tt {
        TargetType::SingleAlly => "Single Ally",
        TargetType::AllAllies => "All Allies",
        TargetType::SingleEnemy => "Single Enemy",
        TargetType::AllEnemies => "All Enemies",
        TargetType::SelfTarget => "Self",
    }
}

/// Returns a human-readable display name for a cost type.
fn cost_type_display(ct: CostType) -> &'static str {
    match ct {
        CostType::MP => "MP",
        CostType::HP => "HP",
    }
}

/// Returns a human-readable display name for an ability source type (for the add dialog).
fn source_type_display(st: &AbilitySourceType) -> &'static str {
    match st {
        AbilitySourceType::LevelUp => "Level Up",
        AbilitySourceType::LearnedFromItem => "Learned From Item",
        AbilitySourceType::EquipmentGrant => "Equipment Grant",
        AbilitySourceType::AccessoryGrant => "Accessory Grant",
    }
}

fn ability_panel_ui(
    mut contexts: EguiContexts,
    mut panel_state: ResMut<AbilityPanelState>,
    mut _project: ResMut<crate::data::Project>,
) -> Result {
    let ctx = contexts.ctx_mut()?;

    // === Left SidePanel: Ability List ===
    egui::SidePanel::left("ability_list")
        .default_width(220.0)
        .show(ctx, |ui| {
            // "Create" button at the top
            if ui.button("➕ Create").clicked() {
                panel_state.create_dialog_open = true;
                panel_state.create_name_buffer.clear();
                panel_state.create_category = None;
                panel_state.create_error = None;
            }

            ui.separator();

            // Category filter combo box
            let filter_label = match panel_state.category_filter {
                None => "All",
                Some(cat) => category_display_name(cat),
            };

            let previous_filter = panel_state.category_filter;

            egui::ComboBox::from_id_salt("ability_category_filter")
                .selected_text(filter_label)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut panel_state.category_filter, None, "All");
                    ui.selectable_value(
                        &mut panel_state.category_filter,
                        Some(AbilityCategory::Skill),
                        "Skill",
                    );
                    ui.selectable_value(
                        &mut panel_state.category_filter,
                        Some(AbilityCategory::Spell),
                        "Spell",
                    );
                    ui.selectable_value(
                        &mut panel_state.category_filter,
                        Some(AbilityCategory::SpecialAction),
                        "Special Action",
                    );
                });

            ui.separator();

            // Get filtered abilities
            let filtered = _project
                .abilities
                .filtered_abilities(panel_state.category_filter);

            // Handle filter change: if selected ability no longer visible, auto-select first or clear
            if panel_state.category_filter != previous_filter
                && let Some(ref selected_id) = panel_state.selected_ability
            {
                let still_visible = filtered.iter().any(|a| &a.id == selected_id);
                if !still_visible {
                    panel_state.selected_ability = filtered.first().map(|a| a.id.clone());
                    // Sync buffers
                    if let Some(ref new_id) = panel_state.selected_ability
                        && let Some(ability) = _project.abilities.abilities.get(new_id)
                    {
                        panel_state.name_edit_buffer = ability.display_name.clone();
                        panel_state.name_edit_error = None;
                    }
                }
            }

            if filtered.is_empty() {
                ui.label("No abilities yet. Create one to get started.");
            } else {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    // Collect ability info to avoid borrow conflicts
                    let ability_entries: Vec<(String, String, AbilityCategory)> = filtered
                        .iter()
                        .map(|a| (a.id.clone(), a.display_name.clone(), a.category))
                        .collect();

                    for (id, display_name, category) in &ability_entries {
                        let is_selected = panel_state.selected_ability.as_ref() == Some(id);

                        ui.horizontal(|ui| {
                            // Selectable label with name and category
                            let label_text =
                                format!("{} ({})", display_name, category_display_name(*category));
                            if ui.selectable_label(is_selected, &label_text).clicked() {
                                panel_state.selected_ability = Some(id.clone());
                                // Sync name_edit_buffer with newly selected ability
                                panel_state.name_edit_buffer = display_name.clone();
                                panel_state.name_edit_error = None;
                            }

                            // Delete button per ability
                            if ui.small_button("🗑").clicked() {
                                panel_state.delete_confirm_target = Some(id.clone());
                            }
                        });
                    }
                });
            }
        });

    // === Right SidePanel: Ability Preview ===
    egui::SidePanel::right("ability_preview")
        .default_width(250.0)
        .show(ctx, |ui| {
            ui.heading("Ability Preview");
            ui.separator();

            if let Some(ref selected_id) = panel_state.selected_ability {
                if let Some(ability) = _project.abilities.abilities.get(selected_id) {
                    ui.label(format!("Name: {}", ability.display_name));
                    ui.label(format!(
                        "Category: {}",
                        category_display_name(ability.category)
                    ));
                    ui.label(format!(
                        "Cost: {} {}",
                        ability.cost_value,
                        cost_type_display(ability.cost_type)
                    ));
                    ui.label(format!("Power: {}", ability.power));
                    ui.label(format!(
                        "Target: {}",
                        target_type_display(ability.target_type)
                    ));

                    ui.separator();
                    ui.label("Sources:");
                    if ability.sources.is_empty() {
                        ui.label("No sources defined.");
                    } else {
                        for source in &ability.sources {
                            match source {
                                AbilitySource::LevelUp { required_level } => {
                                    ui.label(format!("Level Up (level {})", required_level));
                                }
                                AbilitySource::LearnedFromItem { item_id } => {
                                    ui.label(format!("Learned From Item ({})", item_id));
                                }
                                AbilitySource::EquipmentGrant { item_id } => {
                                    ui.label(format!("Equipment Grant ({})", item_id));
                                }
                                AbilitySource::AccessoryGrant { item_id } => {
                                    ui.label(format!("Accessory Grant ({})", item_id));
                                }
                            }
                        }
                    }
                } else {
                    ui.label("Select an ability to preview.");
                }
            } else {
                ui.label("Select an ability to preview.");
            }
        });

    // === CentralPanel: Ability Detail Editor ===
    egui::CentralPanel::default().show(ctx, |ui| {
        if let Some(selected_id) = panel_state.selected_ability.clone() {
            if _project.abilities.abilities.contains_key(&selected_id) {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    // --- Display Name Field ---
                    ui.heading("Display Name");
                    let name_response = ui.text_edit_singleline(&mut panel_state.name_edit_buffer);

                    // Truncate to 64 chars as user types
                    if panel_state.name_edit_buffer.len() > 64 {
                        panel_state.name_edit_buffer.truncate(64);
                    }

                    // Validate on lost focus
                    if name_response.lost_focus() {
                        let trimmed = panel_state.name_edit_buffer.trim().to_string();
                        if trimmed.is_empty() || !trimmed.contains(|c: char| !c.is_whitespace()) {
                            panel_state.name_edit_error = Some(
                                "Name must contain at least 1 non-whitespace character."
                                    .to_string(),
                            );
                        } else if trimmed.len() > 64 {
                            panel_state.name_edit_error =
                                Some("Name must not exceed 64 characters.".to_string());
                        } else {
                            match _project
                                .abilities
                                .update_display_name(&selected_id, &trimmed)
                            {
                                Ok(()) => {
                                    panel_state.name_edit_error = None;
                                    panel_state.name_edit_buffer = trimmed;
                                    _project.has_unsaved_ability_changes = true;
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

                    // --- Description Field ---
                    ui.heading("Description");
                    {
                        let mut desc = _project
                            .abilities
                            .abilities
                            .get(&selected_id)
                            .map(|a| a.description.clone())
                            .unwrap_or_default();
                        let desc_response = ui.add(
                            egui::TextEdit::multiline(&mut desc)
                                .desired_rows(3)
                                .desired_width(f32::INFINITY),
                        );
                        if desc_response.changed() {
                            // Truncate at 256 chars
                            let truncated: String = desc.chars().take(256).collect();
                            let _ = _project
                                .abilities
                                .update_description(&selected_id, &truncated);
                            _project.has_unsaved_ability_changes = true;
                        }
                    }

                    ui.separator();

                    // --- Category ComboBox ---
                    ui.heading("Category");
                    {
                        let current_category = _project
                            .abilities
                            .abilities
                            .get(&selected_id)
                            .map(|a| a.category)
                            .unwrap_or(AbilityCategory::Skill);

                        let mut new_category = current_category;
                        egui::ComboBox::from_id_salt("ability_category_edit")
                            .selected_text(category_display_name(current_category))
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut new_category,
                                    AbilityCategory::Skill,
                                    "Skill",
                                );
                                ui.selectable_value(
                                    &mut new_category,
                                    AbilityCategory::Spell,
                                    "Spell",
                                );
                                ui.selectable_value(
                                    &mut new_category,
                                    AbilityCategory::SpecialAction,
                                    "Special Action",
                                );
                            });

                        if new_category != current_category {
                            let _ = _project
                                .abilities
                                .update_category(&selected_id, new_category);
                            _project.has_unsaved_ability_changes = true;
                        }
                    }

                    ui.separator();

                    // --- Cost Type ComboBox ---
                    ui.heading("Cost Type");
                    {
                        let current_cost_type = _project
                            .abilities
                            .abilities
                            .get(&selected_id)
                            .map(|a| a.cost_type)
                            .unwrap_or(CostType::MP);

                        let mut new_cost_type = current_cost_type;
                        egui::ComboBox::from_id_salt("ability_cost_type_edit")
                            .selected_text(cost_type_display(current_cost_type))
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut new_cost_type, CostType::MP, "MP");
                                ui.selectable_value(&mut new_cost_type, CostType::HP, "HP");
                            });

                        if new_cost_type != current_cost_type {
                            let _ = _project
                                .abilities
                                .update_cost_type(&selected_id, new_cost_type);
                            _project.has_unsaved_ability_changes = true;
                        }
                    }

                    ui.separator();

                    // --- Cost Value DragValue ---
                    ui.heading("Cost Value");
                    {
                        let mut cost_value = _project
                            .abilities
                            .abilities
                            .get(&selected_id)
                            .map(|a| a.cost_value)
                            .unwrap_or(0);

                        if ui
                            .add(egui::DragValue::new(&mut cost_value).range(0..=u32::MAX))
                            .changed()
                        {
                            let _ = _project
                                .abilities
                                .update_cost_value(&selected_id, cost_value);
                            _project.has_unsaved_ability_changes = true;
                        }
                    }

                    ui.separator();

                    // --- Power DragValue ---
                    ui.heading("Power");
                    {
                        let mut power = _project
                            .abilities
                            .abilities
                            .get(&selected_id)
                            .map(|a| a.power)
                            .unwrap_or(0);

                        if ui
                            .add(egui::DragValue::new(&mut power).range(0..=u32::MAX))
                            .changed()
                        {
                            let _ = _project.abilities.update_power(&selected_id, power);
                            _project.has_unsaved_ability_changes = true;
                        }
                    }

                    ui.separator();

                    // --- Target Type ComboBox ---
                    ui.heading("Target Type");
                    {
                        let current_target_type = _project
                            .abilities
                            .abilities
                            .get(&selected_id)
                            .map(|a| a.target_type)
                            .unwrap_or(TargetType::SingleEnemy);

                        let mut new_target_type = current_target_type;
                        egui::ComboBox::from_id_salt("ability_target_type_edit")
                            .selected_text(target_type_display(current_target_type))
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut new_target_type,
                                    TargetType::SingleAlly,
                                    "Single Ally",
                                );
                                ui.selectable_value(
                                    &mut new_target_type,
                                    TargetType::AllAllies,
                                    "All Allies",
                                );
                                ui.selectable_value(
                                    &mut new_target_type,
                                    TargetType::SingleEnemy,
                                    "Single Enemy",
                                );
                                ui.selectable_value(
                                    &mut new_target_type,
                                    TargetType::AllEnemies,
                                    "All Enemies",
                                );
                                ui.selectable_value(
                                    &mut new_target_type,
                                    TargetType::SelfTarget,
                                    "Self",
                                );
                            });

                        if new_target_type != current_target_type {
                            let _ = _project
                                .abilities
                                .update_target_type(&selected_id, new_target_type);
                            _project.has_unsaved_ability_changes = true;
                        }
                    }

                    ui.separator();

                    // --- Sources Section ---
                    ui.heading("Sources");
                    {
                        let sources: Vec<AbilitySource> = _project
                            .abilities
                            .abilities
                            .get(&selected_id)
                            .map(|a| a.sources.clone())
                            .unwrap_or_default();

                        if sources.is_empty() {
                            ui.label("No sources defined.");
                        } else {
                            let mut source_to_remove: Option<usize> = None;

                            for (idx, source) in sources.iter().enumerate() {
                                ui.horizontal(|ui| {
                                    let label = match source {
                                        AbilitySource::LevelUp { required_level } => {
                                            format!("Level Up (level {})", required_level)
                                        }
                                        AbilitySource::LearnedFromItem { item_id } => {
                                            format!("Learned From Item ({})", item_id)
                                        }
                                        AbilitySource::EquipmentGrant { item_id } => {
                                            format!("Equipment Grant ({})", item_id)
                                        }
                                        AbilitySource::AccessoryGrant { item_id } => {
                                            format!("Accessory Grant ({})", item_id)
                                        }
                                    };
                                    ui.label(&label);
                                    if ui.small_button("🗑").clicked() {
                                        source_to_remove = Some(idx);
                                    }
                                });
                            }

                            if let Some(idx) = source_to_remove {
                                let _ = _project.abilities.remove_source(&selected_id, idx);
                                _project.has_unsaved_ability_changes = true;
                            }
                        }

                        // Add source button
                        if ui.button("➕ Add Source").clicked() {
                            panel_state.add_source_dialog_open = true;
                            panel_state.add_source_type = AbilitySourceType::LevelUp;
                            panel_state.add_source_level_buffer = "1".to_string();
                            panel_state.add_source_item_id_buffer.clear();
                            panel_state.add_source_error = None;
                        }
                    }
                });
            } else {
                ui.label("Selected ability not found.");
            }
        } else {
            ui.centered_and_justified(|ui| {
                ui.label("Select an ability to edit, or create a new one.");
            });
        }
    });

    // === Add Source Dialog ===
    if panel_state.add_source_dialog_open {
        let mut still_open = true;
        let mut should_add = false;
        let mut should_cancel = false;

        egui::Window::new("Add Source")
            .collapsible(false)
            .resizable(false)
            .open(&mut still_open)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Source Type:");
                    egui::ComboBox::from_id_salt("add_source_type")
                        .selected_text(source_type_display(&panel_state.add_source_type))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut panel_state.add_source_type,
                                AbilitySourceType::LevelUp,
                                "Level Up",
                            );
                            ui.selectable_value(
                                &mut panel_state.add_source_type,
                                AbilitySourceType::LearnedFromItem,
                                "Learned From Item",
                            );
                            ui.selectable_value(
                                &mut panel_state.add_source_type,
                                AbilitySourceType::EquipmentGrant,
                                "Equipment Grant",
                            );
                            ui.selectable_value(
                                &mut panel_state.add_source_type,
                                AbilitySourceType::AccessoryGrant,
                                "Accessory Grant",
                            );
                        });
                });

                // Show type-specific fields
                match panel_state.add_source_type {
                    AbilitySourceType::LevelUp => {
                        ui.horizontal(|ui| {
                            ui.label("Required Level:");
                            ui.text_edit_singleline(&mut panel_state.add_source_level_buffer);
                        });
                    }
                    _ => {
                        ui.horizontal(|ui| {
                            ui.label("Item ID:");
                            ui.text_edit_singleline(&mut panel_state.add_source_item_id_buffer);
                        });
                    }
                }

                if let Some(ref error) = panel_state.add_source_error {
                    ui.colored_label(egui::Color32::RED, error);
                }

                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("Add").clicked() {
                        should_add = true;
                    }
                    if ui.button("Cancel").clicked() {
                        should_cancel = true;
                    }
                });
            });

        if !still_open {
            panel_state.add_source_dialog_open = false;
            panel_state.add_source_level_buffer.clear();
            panel_state.add_source_item_id_buffer.clear();
            panel_state.add_source_error = None;
        }

        if should_cancel {
            panel_state.add_source_dialog_open = false;
            panel_state.add_source_level_buffer.clear();
            panel_state.add_source_item_id_buffer.clear();
            panel_state.add_source_error = None;
        }

        if should_add && let Some(ref selected_id) = panel_state.selected_ability.clone() {
            let source_result = match panel_state.add_source_type {
                AbilitySourceType::LevelUp => {
                    match panel_state.add_source_level_buffer.trim().parse::<u32>() {
                        Ok(level) => {
                            let source = AbilitySource::LevelUp {
                                required_level: level,
                            };
                            Some(_project.abilities.add_source(selected_id, source))
                        }
                        Err(_) => {
                            panel_state.add_source_error =
                                Some("Level must be a valid positive number.".to_string());
                            None
                        }
                    }
                }
                AbilitySourceType::LearnedFromItem => {
                    let item_id = panel_state.add_source_item_id_buffer.trim().to_string();
                    let source = AbilitySource::LearnedFromItem { item_id };
                    Some(_project.abilities.add_source(selected_id, source))
                }
                AbilitySourceType::EquipmentGrant => {
                    let item_id = panel_state.add_source_item_id_buffer.trim().to_string();
                    let source = AbilitySource::EquipmentGrant { item_id };
                    Some(_project.abilities.add_source(selected_id, source))
                }
                AbilitySourceType::AccessoryGrant => {
                    let item_id = panel_state.add_source_item_id_buffer.trim().to_string();
                    let source = AbilitySource::AccessoryGrant { item_id };
                    Some(_project.abilities.add_source(selected_id, source))
                }
            };

            if let Some(result) = source_result {
                match result {
                    Ok(()) => {
                        panel_state.add_source_dialog_open = false;
                        panel_state.add_source_level_buffer.clear();
                        panel_state.add_source_item_id_buffer.clear();
                        panel_state.add_source_error = None;
                        _project.has_unsaved_ability_changes = true;
                    }
                    Err(e) => {
                        panel_state.add_source_error = Some(e.to_string());
                    }
                }
            }
        }
    }

    // === Create Ability Dialog ===
    if panel_state.create_dialog_open {
        let mut still_open = true;
        let mut should_create = false;
        let mut should_cancel = false;

        egui::Window::new("New Ability")
            .collapsible(false)
            .resizable(false)
            .open(&mut still_open)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Name:");
                    ui.text_edit_singleline(&mut panel_state.create_name_buffer);
                    // Limit to 64 characters
                    if panel_state.create_name_buffer.len() > 64 {
                        panel_state.create_name_buffer.truncate(64);
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Category:");
                    let selected_text = match panel_state.create_category {
                        Some(cat) => category_display_name(cat),
                        None => "Select category...",
                    };
                    egui::ComboBox::from_id_salt("create_ability_category")
                        .selected_text(selected_text)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut panel_state.create_category,
                                Some(AbilityCategory::Skill),
                                "Skill",
                            );
                            ui.selectable_value(
                                &mut panel_state.create_category,
                                Some(AbilityCategory::Spell),
                                "Spell",
                            );
                            ui.selectable_value(
                                &mut panel_state.create_category,
                                Some(AbilityCategory::SpecialAction),
                                "Special Action",
                            );
                        });
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
            panel_state.create_category = None;
            panel_state.create_error = None;
        }

        if should_cancel {
            panel_state.create_dialog_open = false;
            panel_state.create_name_buffer.clear();
            panel_state.create_category = None;
            panel_state.create_error = None;
        }

        if should_create {
            let name = panel_state.create_name_buffer.trim();
            if name.is_empty() || !name.contains(|c: char| !c.is_whitespace()) {
                panel_state.create_error =
                    Some("Name must not be empty or whitespace-only.".to_string());
            } else if panel_state.create_category.is_none() {
                panel_state.create_error = Some("Please select a category.".to_string());
            } else {
                let category = panel_state.create_category.unwrap();
                match _project
                    .abilities
                    .create_ability(&panel_state.create_name_buffer, category)
                {
                    Ok(new_id) => {
                        // Auto-select the newly created ability
                        let display_name = _project
                            .abilities
                            .abilities
                            .get(&new_id)
                            .map(|a| a.display_name.clone())
                            .unwrap_or_default();
                        panel_state.selected_ability = Some(new_id);
                        panel_state.name_edit_buffer = display_name;
                        panel_state.name_edit_error = None;
                        panel_state.create_dialog_open = false;
                        panel_state.create_name_buffer.clear();
                        panel_state.create_category = None;
                        panel_state.create_error = None;
                        _project.has_unsaved_ability_changes = true;
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
        let ability_name = _project
            .abilities
            .abilities
            .get(&target_id)
            .map(|a| a.display_name.clone())
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
                    ability_name
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
            let _ = _project.abilities.delete_ability(&target_id);
            _project.has_unsaved_ability_changes = true;

            // If the deleted ability was selected, select the first remaining or clear
            if panel_state.selected_ability.as_ref() == Some(&target_id) {
                let sorted = _project.abilities.filtered_abilities(None);
                if let Some(first) = sorted.first() {
                    panel_state.selected_ability = Some(first.id.clone());
                    panel_state.name_edit_buffer = first.display_name.clone();
                    panel_state.name_edit_error = None;
                } else {
                    panel_state.selected_ability = None;
                    panel_state.name_edit_buffer.clear();
                    panel_state.name_edit_error = None;
                }
            }

            panel_state.delete_confirm_target = None;
        }
    }

    Ok(())
}
