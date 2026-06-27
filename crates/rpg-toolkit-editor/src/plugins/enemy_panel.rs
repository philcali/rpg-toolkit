use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

use rpg_toolkit_common::{AbilityCategory, Element, EnemyId};

use crate::data::AppEditorMode;
use crate::data::EditorUiSet;
use crate::plugins::searchable_combobox::searchable_combobox;

/// Plugin that provides the Enemy Editor panel UI.
pub struct EnemyPanelPlugin;

impl Plugin for EnemyPanelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EnemyPanelState>().add_systems(
            EguiPrimaryContextPass,
            enemy_panel_ui
                .in_set(EditorUiSet::Panels)
                .run_if(resource_equals(AppEditorMode::Enemy)),
        );
    }
}

/// UI state for the Enemy Editor panel.
#[derive(Resource, Default)]
pub struct EnemyPanelState {
    /// Currently selected enemy ID.
    pub selected_enemy: Option<EnemyId>,
    /// Whether the "Create Enemy" dialog is open.
    pub create_dialog_open: bool,
    /// Text buffer for the enemy name in the create dialog.
    pub create_name_buffer: String,
    /// Validation error for the create dialog.
    pub create_error: Option<String>,
    /// Enemy ID pending delete confirmation.
    pub delete_confirm_target: Option<EnemyId>,
    /// Text buffer for inline name editing.
    pub name_edit_buffer: String,
    /// Validation error for inline name editing.
    pub name_edit_error: Option<String>,
    /// Text buffer for description editing.
    pub description_buffer: String,
    /// Text buffer for the enemy list search.
    pub search_buffer: String,
    /// Text buffer for the ability search/filter.
    pub ability_search_buffer: String,
    /// Buffer for new stat name input.
    pub add_stat_buffer: String,
    /// Error for stat addition.
    pub add_stat_error: Option<String>,
    /// Buffer for new item drop ID.
    pub add_item_drop_id_buffer: String,
    /// Error for item drop addition.
    pub add_item_drop_error: Option<String>,
    /// Buffer for new carried item ID.
    pub add_carried_item_id_buffer: String,
    /// Error for carried item addition.
    pub add_carried_item_error: Option<String>,
    /// Error for modifier addition.
    pub add_modifier_error: Option<String>,
    /// Buffer for new ability ID.
    #[allow(dead_code)]
    pub add_ability_id_buffer: String,
    /// Error for ability addition.
    pub add_ability_error: Option<String>,
    /// Text buffer for portrait path editing.
    pub portrait_buffer: String,
    /// Validation error for portrait path.
    pub portrait_error: Option<String>,
}

fn enemy_panel_ui(
    mut contexts: EguiContexts,
    mut panel_state: ResMut<EnemyPanelState>,
    mut _project: ResMut<crate::data::Project>,
) -> Result {
    let ctx = contexts.ctx_mut()?;

    // === Left SidePanel: Enemy List ===
    egui::SidePanel::left("enemy_list")
        .default_width(220.0)
        .show(ctx, |ui| {
            // "Create" button at the top
            if ui.button("➕ Create").clicked() {
                panel_state.create_dialog_open = true;
                panel_state.create_name_buffer.clear();
                panel_state.create_error = None;
            }

            ui.separator();

            // Search field
            ui.horizontal(|ui| {
                ui.label("🔍");
                ui.text_edit_singleline(&mut panel_state.search_buffer);
            });

            ui.separator();

            // Get filtered/sorted enemies
            let enemies = _project.enemies.search_enemies(&panel_state.search_buffer);

            if enemies.is_empty() {
                ui.label("No enemies yet. Create one to get started.");
            } else {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    let enemy_entries: Vec<(String, String)> = enemies
                        .iter()
                        .map(|e| (e.id.clone(), e.display_name.clone()))
                        .collect();

                    for (id, display_name) in &enemy_entries {
                        let is_selected = panel_state.selected_enemy.as_ref() == Some(id);

                        ui.horizontal(|ui| {
                            if ui.selectable_label(is_selected, display_name).clicked() {
                                panel_state.selected_enemy = Some(id.clone());
                                // Sync buffers from the enemy's fields
                                if let Some(enemy) = _project.enemies.enemies.get(id) {
                                    panel_state.name_edit_buffer = enemy.display_name.clone();
                                    panel_state.description_buffer = enemy.description.clone();
                                    panel_state.portrait_buffer =
                                        enemy.portrait.clone().unwrap_or_default();
                                }
                                panel_state.name_edit_error = None;
                                panel_state.portrait_error = None;
                            }

                            // Delete button per entry
                            if ui.small_button("🗑").clicked() {
                                panel_state.delete_confirm_target = Some(id.clone());
                            }
                        });
                    }
                });
            }
        });

    // === Right SidePanel: Enemy Preview ===
    egui::SidePanel::right("enemy_preview")
        .default_width(250.0)
        .show(ctx, |ui| {
            ui.heading("Enemy Preview");
            ui.separator();

            if let Some(ref selected_id) = panel_state.selected_enemy {
                if let Some(enemy) = _project.enemies.enemies.get(selected_id) {
                    ui.label(format!("Name: {}", enemy.display_name));

                    ui.separator();
                    ui.label("Stats:");
                    if enemy.stats.is_empty() {
                        ui.label("No stats defined.");
                    } else {
                        for stat in &enemy.stats {
                            ui.label(format!("  {}: {}", stat.name, stat.base_value));
                        }
                    }

                    ui.separator();
                    ui.label("Defeat Rewards:");
                    ui.label(format!("  EXP: {}", enemy.defeat_rewards.exp));
                    ui.label(format!("  Gold: {}", enemy.defeat_rewards.gold));
                    ui.label(format!(
                        "  Item Drops: {}",
                        enemy.defeat_rewards.item_drops.len()
                    ));

                    ui.separator();
                    ui.label(format!("Carried Items: {}", enemy.carried_items.len()));

                    ui.separator();
                    ui.label("Elemental Modifiers:");
                    if enemy.elemental_modifiers.is_empty() {
                        ui.label("No elemental modifiers.");
                    } else {
                        for modifier in &enemy.elemental_modifiers {
                            ui.label(format!(
                                "  {:?}: x{}",
                                modifier.element, modifier.multiplier
                            ));
                        }
                    }
                } else {
                    ui.label("Select an enemy to preview.");
                }
            } else {
                ui.label("Select an enemy to preview.");
            }
        });

    // === Central Panel: Enemy Details ===
    egui::CentralPanel::default().show(ctx, |ui| {
        if let Some(selected_id) = panel_state.selected_enemy.clone() {
            if _project.enemies.enemies.contains_key(&selected_id) {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    // --- Display Name Field ---
                    ui.heading("Display Name");
                    let name_response = ui.text_edit_singleline(&mut panel_state.name_edit_buffer);

                    // Truncate to 64 chars as user types
                    if panel_state.name_edit_buffer.chars().count() > 64 {
                        let truncated: String =
                            panel_state.name_edit_buffer.chars().take(64).collect();
                        panel_state.name_edit_buffer = truncated;
                    }

                    // Validate on lost focus
                    if name_response.lost_focus() {
                        let trimmed = panel_state.name_edit_buffer.trim().to_string();
                        if trimmed.is_empty() || !trimmed.contains(|c: char| !c.is_whitespace()) {
                            panel_state.name_edit_error = Some(
                                "Name must contain at least 1 non-whitespace character."
                                    .to_string(),
                            );
                        } else if trimmed.chars().count() > 64 {
                            panel_state.name_edit_error =
                                Some("Name must not exceed 64 characters.".to_string());
                        } else {
                            match _project.enemies.rename_enemy(&selected_id, &trimmed) {
                                Ok(()) => {
                                    panel_state.name_edit_error = None;
                                    panel_state.name_edit_buffer = trimmed;
                                    _project.has_unsaved_enemy_changes = true;
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
                    let desc_response = ui.add(
                        egui::TextEdit::multiline(&mut panel_state.description_buffer)
                            .desired_rows(3)
                            .desired_width(f32::INFINITY),
                    );
                    if desc_response.changed() {
                        // Truncate at 256 chars
                        if panel_state.description_buffer.chars().count() > 256 {
                            let truncated: String =
                                panel_state.description_buffer.chars().take(256).collect();
                            panel_state.description_buffer = truncated.clone();
                            let _ = _project
                                .enemies
                                .update_description(&selected_id, &truncated);
                        } else {
                            let _ = _project
                                .enemies
                                .update_description(&selected_id, &panel_state.description_buffer);
                        }
                        _project.has_unsaved_enemy_changes = true;
                    }

                    ui.separator();

                    // --- Portrait Section ---
                    ui.heading("Portrait");
                    {
                        let has_portrait = _project
                            .enemies
                            .enemies
                            .get(&selected_id)
                            .and_then(|e| e.portrait.as_ref())
                            .is_some();

                        if !has_portrait {
                            ui.label("No portrait assigned");
                        }

                        // Text input and Browse button on the same row
                        let portrait_response = ui
                            .horizontal(|ui| {
                                let response =
                                    ui.text_edit_singleline(&mut panel_state.portrait_buffer);

                                // Browse... button — opens native file dialog for image selection
                                if ui.button("Browse...").clicked() {
                                    let file = rfd::FileDialog::new()
                                        .add_filter("Images", &["png", "jpg", "jpeg"])
                                        .pick_file();

                                    if let Some(path) = file {
                                        let path_str = path.display().to_string();
                                        // Truncate to 260 characters
                                        let truncated: String =
                                            path_str.chars().take(260).collect();
                                        let trimmed = truncated.trim().to_string();

                                        if trimmed.is_empty() {
                                            panel_state.portrait_error = Some(
                                            "Portrait path must not be empty or whitespace-only."
                                                .to_string(),
                                        );
                                        } else {
                                            match _project
                                                .enemies
                                                .set_portrait(&selected_id, &trimmed)
                                            {
                                                Ok(()) => {
                                                    panel_state.portrait_buffer = trimmed;
                                                    panel_state.portrait_error = None;
                                                    _project.has_unsaved_enemy_changes = true;
                                                }
                                                Err(e) => {
                                                    panel_state.portrait_error =
                                                        Some(e.to_string());
                                                }
                                            }
                                        }
                                    }
                                    // If dialog is cancelled (file is None), leave buffer unchanged
                                }

                                response
                            })
                            .inner;

                        // Truncate to 260 chars as user types
                        if panel_state.portrait_buffer.chars().count() > 260 {
                            let truncated: String =
                                panel_state.portrait_buffer.chars().take(260).collect();
                            panel_state.portrait_buffer = truncated;
                        }

                        // Validate on lost focus
                        if portrait_response.lost_focus() {
                            let trimmed = panel_state.portrait_buffer.trim().to_string();
                            if trimmed.is_empty() {
                                panel_state.portrait_error = Some(
                                    "Portrait path must not be empty or whitespace-only."
                                        .to_string(),
                                );
                            } else {
                                match _project.enemies.set_portrait(&selected_id, &trimmed) {
                                    Ok(()) => {
                                        panel_state.portrait_error = None;
                                        panel_state.portrait_buffer = trimmed;
                                        _project.has_unsaved_enemy_changes = true;
                                    }
                                    Err(e) => {
                                        panel_state.portrait_error = Some(e.to_string());
                                    }
                                }
                            }
                        }

                        // Show validation error
                        if let Some(ref error) = panel_state.portrait_error {
                            ui.colored_label(egui::Color32::RED, error);
                        }

                        // Clear button
                        if ui.button("Clear").clicked() {
                            let _ = _project.enemies.clear_portrait(&selected_id);
                            panel_state.portrait_buffer.clear();
                            panel_state.portrait_error = None;
                            _project.has_unsaved_enemy_changes = true;
                        }
                    }

                    ui.separator();

                    // --- Stats Section ---
                    ui.heading("Stats");
                    {
                        let stats: Vec<(String, u32)> = _project
                            .enemies
                            .enemies
                            .get(&selected_id)
                            .map(|e| {
                                e.stats
                                    .iter()
                                    .map(|s| (s.name.clone(), s.base_value))
                                    .collect()
                            })
                            .unwrap_or_default();

                        let mut stat_to_remove: Option<String> = None;

                        egui::Grid::new("enemy_stats_grid")
                            .num_columns(3)
                            .spacing([10.0, 4.0])
                            .show(ui, |ui| {
                                for (stat_name, base_value) in &stats {
                                    ui.label(stat_name);
                                    let mut value = *base_value;
                                    if ui
                                        .add(egui::DragValue::new(&mut value).range(0..=u32::MAX))
                                        .changed()
                                    {
                                        let _ = _project.enemies.update_stat(
                                            &selected_id,
                                            stat_name,
                                            value,
                                        );
                                        _project.has_unsaved_enemy_changes = true;
                                    }
                                    // Delete button (disabled for "HP")
                                    if stat_name == "HP" {
                                        ui.add_enabled(false, egui::Button::new("🗑"));
                                    } else if ui.small_button("🗑").clicked() {
                                        stat_to_remove = Some(stat_name.clone());
                                    }
                                    ui.end_row();
                                }
                            });

                        if let Some(stat_name) = stat_to_remove {
                            let _ = _project.enemies.remove_stat(&selected_id, &stat_name);
                            _project.has_unsaved_enemy_changes = true;
                        }

                        // Add Stat
                        ui.horizontal(|ui| {
                            ui.text_edit_singleline(&mut panel_state.add_stat_buffer);
                            if ui.button("Add Stat").clicked() {
                                let trimmed = panel_state.add_stat_buffer.trim().to_string();
                                match _project.enemies.add_stat(&selected_id, &trimmed) {
                                    Ok(()) => {
                                        panel_state.add_stat_buffer.clear();
                                        panel_state.add_stat_error = None;
                                        _project.has_unsaved_enemy_changes = true;
                                    }
                                    Err(e) => {
                                        panel_state.add_stat_error = Some(e.to_string());
                                    }
                                }
                            }
                        });
                        if let Some(ref error) = panel_state.add_stat_error {
                            ui.colored_label(egui::Color32::RED, error);
                        }
                    }

                    ui.separator();

                    // --- Defeat Rewards Section ---
                    ui.heading("Defeat Rewards");
                    {
                        let mut exp = _project
                            .enemies
                            .enemies
                            .get(&selected_id)
                            .map(|e| e.defeat_rewards.exp)
                            .unwrap_or(0);
                        let mut gold = _project
                            .enemies
                            .enemies
                            .get(&selected_id)
                            .map(|e| e.defeat_rewards.gold)
                            .unwrap_or(0);

                        ui.horizontal(|ui| {
                            ui.label("EXP:");
                            if ui
                                .add(egui::DragValue::new(&mut exp).range(0..=u32::MAX))
                                .changed()
                            {
                                let _ = _project.enemies.update_exp(&selected_id, exp);
                                _project.has_unsaved_enemy_changes = true;
                            }
                        });

                        ui.horizontal(|ui| {
                            ui.label("Gold:");
                            if ui
                                .add(egui::DragValue::new(&mut gold).range(0..=u32::MAX))
                                .changed()
                            {
                                let _ = _project.enemies.update_gold(&selected_id, gold);
                                _project.has_unsaved_enemy_changes = true;
                            }
                        });

                        ui.separator();
                        ui.label("Item Drops:");

                        let item_drops: Vec<(String, f64)> = _project
                            .enemies
                            .enemies
                            .get(&selected_id)
                            .map(|e| {
                                e.defeat_rewards
                                    .item_drops
                                    .iter()
                                    .map(|d| (d.item_id.clone(), d.drop_chance))
                                    .collect()
                            })
                            .unwrap_or_default();

                        let mut drop_to_remove: Option<usize> = None;

                        for (idx, (item_id, drop_chance)) in item_drops.iter().enumerate() {
                            ui.horizontal(|ui| {
                                ui.label(item_id);
                                let mut chance = *drop_chance;
                                if ui
                                    .add(
                                        egui::DragValue::new(&mut chance)
                                            .range(0.0..=1.0)
                                            .speed(0.01),
                                    )
                                    .changed()
                                {
                                    // Update by directly mutating the drop chance
                                    if let Some(enemy) =
                                        _project.enemies.enemies.get_mut(&selected_id)
                                        && let Some(drop) =
                                            enemy.defeat_rewards.item_drops.get_mut(idx)
                                    {
                                        drop.drop_chance = chance;
                                    }
                                    _project.has_unsaved_enemy_changes = true;
                                }
                                if ui.small_button("🗑").clicked() {
                                    drop_to_remove = Some(idx);
                                }
                            });
                        }

                        if let Some(idx) = drop_to_remove {
                            let _ = _project.enemies.remove_item_drop(&selected_id, idx);
                            _project.has_unsaved_enemy_changes = true;
                        }

                        // Add Item Drop
                        ui.horizontal(|ui| {
                            ui.text_edit_singleline(&mut panel_state.add_item_drop_id_buffer);
                            if ui.button("Add Item Drop").clicked() {
                                match _project.enemies.add_item_drop(
                                    &selected_id,
                                    &panel_state.add_item_drop_id_buffer,
                                    0.5,
                                ) {
                                    Ok(()) => {
                                        panel_state.add_item_drop_id_buffer.clear();
                                        panel_state.add_item_drop_error = None;
                                        _project.has_unsaved_enemy_changes = true;
                                    }
                                    Err(e) => {
                                        panel_state.add_item_drop_error = Some(e.to_string());
                                    }
                                }
                            }
                        });
                        if let Some(ref error) = panel_state.add_item_drop_error {
                            ui.colored_label(egui::Color32::RED, error);
                        }
                    }

                    ui.separator();

                    // --- Carried Items Section ---
                    ui.heading("Carried Items");
                    {
                        let carried_items: Vec<(String, f64)> = _project
                            .enemies
                            .enemies
                            .get(&selected_id)
                            .map(|e| {
                                e.carried_items
                                    .iter()
                                    .map(|c| (c.item_id.clone(), c.obtain_chance))
                                    .collect()
                            })
                            .unwrap_or_default();

                        let mut carried_to_remove: Option<usize> = None;

                        for (idx, (item_id, obtain_chance)) in carried_items.iter().enumerate() {
                            ui.horizontal(|ui| {
                                ui.label(item_id);
                                let mut chance = *obtain_chance;
                                if ui
                                    .add(
                                        egui::DragValue::new(&mut chance)
                                            .range(0.0..=1.0)
                                            .speed(0.01),
                                    )
                                    .changed()
                                {
                                    if let Some(enemy) =
                                        _project.enemies.enemies.get_mut(&selected_id)
                                        && let Some(carried) = enemy.carried_items.get_mut(idx)
                                    {
                                        carried.obtain_chance = chance;
                                    }
                                    _project.has_unsaved_enemy_changes = true;
                                }
                                if ui.small_button("🗑").clicked() {
                                    carried_to_remove = Some(idx);
                                }
                            });
                        }

                        if let Some(idx) = carried_to_remove {
                            let _ = _project.enemies.remove_carried_item(&selected_id, idx);
                            _project.has_unsaved_enemy_changes = true;
                        }

                        // Add Carried Item
                        ui.horizontal(|ui| {
                            ui.text_edit_singleline(&mut panel_state.add_carried_item_id_buffer);
                            if ui.button("Add Carried Item").clicked() {
                                match _project.enemies.add_carried_item(
                                    &selected_id,
                                    &panel_state.add_carried_item_id_buffer,
                                    0.5,
                                ) {
                                    Ok(()) => {
                                        panel_state.add_carried_item_id_buffer.clear();
                                        panel_state.add_carried_item_error = None;
                                        _project.has_unsaved_enemy_changes = true;
                                    }
                                    Err(e) => {
                                        panel_state.add_carried_item_error = Some(e.to_string());
                                    }
                                }
                            }
                        });
                        if let Some(ref error) = panel_state.add_carried_item_error {
                            ui.colored_label(egui::Color32::RED, error);
                        }
                    }

                    ui.separator();

                    // --- Elemental Modifiers Section ---
                    ui.heading("Elemental Modifiers");
                    {
                        let modifiers: Vec<(Element, f64)> = _project
                            .enemies
                            .enemies
                            .get(&selected_id)
                            .map(|e| {
                                e.elemental_modifiers
                                    .iter()
                                    .map(|m| (m.element, m.multiplier))
                                    .collect()
                            })
                            .unwrap_or_default();

                        let mut modifier_to_remove: Option<Element> = None;

                        for (element, multiplier) in &modifiers {
                            ui.horizontal(|ui| {
                                ui.label(format!("{:?}", element));
                                let mut mult = *multiplier;
                                if ui
                                    .add(
                                        egui::DragValue::new(&mut mult)
                                            .range(0.0..=f64::MAX)
                                            .speed(0.1),
                                    )
                                    .changed()
                                {
                                    let _ = _project.enemies.update_elemental_modifier(
                                        &selected_id,
                                        *element,
                                        mult,
                                    );
                                    _project.has_unsaved_enemy_changes = true;
                                }
                                if ui.small_button("🗑").clicked() {
                                    modifier_to_remove = Some(*element);
                                }
                            });
                        }

                        if let Some(element) = modifier_to_remove {
                            let _ = _project
                                .enemies
                                .remove_elemental_modifier(&selected_id, element);
                            _project.has_unsaved_enemy_changes = true;
                        }

                        // Add Modifier with Element combo box
                        ui.horizontal(|ui| {
                            let mut selected_element = Element::Fire;
                            egui::ComboBox::from_id_salt("add_element_modifier")
                                .selected_text(format!("{:?}", selected_element))
                                .show_ui(ui, |ui| {
                                    for elem in Element::all() {
                                        ui.selectable_value(
                                            &mut selected_element,
                                            *elem,
                                            format!("{:?}", elem),
                                        );
                                    }
                                });
                            if ui.button("Add Modifier").clicked() {
                                match _project.enemies.add_elemental_modifier(
                                    &selected_id,
                                    selected_element,
                                    1.0,
                                ) {
                                    Ok(()) => {
                                        panel_state.add_modifier_error = None;
                                        _project.has_unsaved_enemy_changes = true;
                                    }
                                    Err(e) => {
                                        panel_state.add_modifier_error = Some(e.to_string());
                                    }
                                }
                            }
                        });
                        if let Some(ref error) = panel_state.add_modifier_error {
                            ui.colored_label(egui::Color32::RED, error);
                        }
                    }

                    ui.separator();

                    // --- Abilities Section ---
                    ui.heading("Abilities");
                    {
                        let abilities: Vec<String> = _project
                            .enemies
                            .enemies
                            .get(&selected_id)
                            .map(|e| e.abilities.clone())
                            .unwrap_or_default();

                        let mut ability_to_remove: Option<String> = None;

                        for ability_id in &abilities {
                            // Display with name and category if found in registry
                            let display_label = if let Some(ability) =
                                _project.abilities.abilities.get(ability_id)
                            {
                                let category_name = match ability.category {
                                    AbilityCategory::Skill => "Skill",
                                    AbilityCategory::Spell => "Spell",
                                    AbilityCategory::SpecialAction => "Special Action",
                                    AbilityCategory::Monster => "Monster",
                                };
                                format!("{} [{}]", ability.display_name, category_name)
                            } else {
                                ability_id.clone()
                            };
                            ui.horizontal(|ui| {
                                ui.label(&display_label);
                                if ui.small_button("🗑").clicked() {
                                    ability_to_remove = Some(ability_id.clone());
                                }
                            });
                        }

                        if let Some(ability_id) = ability_to_remove {
                            let _ = _project.enemies.remove_ability(&selected_id, &ability_id);
                            _project.has_unsaved_enemy_changes = true;
                        }

                        // Add Ability via searchable dropdown
                        if _project.abilities.abilities.is_empty() {
                            ui.label("No abilities available");
                        } else {
                            // Build items list from ability registry
                            let items: Vec<(String, String)> = _project
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
                                "enemy_add_ability",
                                "Select ability…",
                                &items,
                                &mut panel_state.ability_search_buffer,
                            ) {
                                match _project
                                    .enemies
                                    .add_ability(&selected_id, &selected_ability_id)
                                {
                                    Ok(()) => {
                                        panel_state.add_ability_error = None;
                                        _project.has_unsaved_enemy_changes = true;
                                    }
                                    Err(e) => {
                                        panel_state.add_ability_error = Some(e.to_string());
                                    }
                                }
                            }
                        }

                        if let Some(ref error) = panel_state.add_ability_error {
                            ui.colored_label(egui::Color32::RED, error);
                        }
                    }
                });
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label("Select an enemy to edit, or create a new one.");
                });
            }
        } else {
            ui.centered_and_justified(|ui| {
                ui.label("Select an enemy to edit, or create a new one.");
            });
        }
    });

    // === Create Enemy Dialog ===
    if panel_state.create_dialog_open {
        let mut still_open = true;
        let mut should_create = false;
        let mut should_cancel = false;

        egui::Window::new("New Enemy")
            .collapsible(false)
            .resizable(false)
            .open(&mut still_open)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Name:");
                    ui.text_edit_singleline(&mut panel_state.create_name_buffer);
                    // Limit to 64 characters
                    if panel_state.create_name_buffer.chars().count() > 64 {
                        let truncated: String =
                            panel_state.create_name_buffer.chars().take(64).collect();
                        panel_state.create_name_buffer = truncated;
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
            let trimmed = panel_state.create_name_buffer.trim().to_string();
            if trimmed.is_empty() || !trimmed.contains(|c: char| !c.is_whitespace()) {
                panel_state.create_error =
                    Some("Name must not be empty or whitespace-only.".to_string());
            } else if trimmed.chars().count() > 64 {
                panel_state.create_error = Some("Name must not exceed 64 characters.".to_string());
            } else {
                match _project.enemies.create_enemy(&trimmed) {
                    Ok(new_id) => {
                        // Auto-select the newly created enemy
                        if let Some(enemy) = _project.enemies.enemies.get(&new_id) {
                            panel_state.name_edit_buffer = enemy.display_name.clone();
                            panel_state.description_buffer = enemy.description.clone();
                            panel_state.portrait_buffer =
                                enemy.portrait.clone().unwrap_or_default();
                        }
                        panel_state.selected_enemy = Some(new_id);
                        panel_state.name_edit_error = None;
                        panel_state.portrait_error = None;
                        panel_state.create_dialog_open = false;
                        panel_state.create_name_buffer.clear();
                        panel_state.create_error = None;
                        _project.has_unsaved_enemy_changes = true;
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
        let enemy_name = _project
            .enemies
            .enemies
            .get(&target_id)
            .map(|e| e.display_name.clone())
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
                    enemy_name
                ));
                ui.label("This action cannot be undone.");
                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("Confirm").clicked() {
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
            let _ = _project.enemies.delete_enemy(&target_id);
            _project.has_unsaved_enemy_changes = true;

            // If the deleted enemy was selected, clear selection
            if panel_state.selected_enemy.as_ref() == Some(&target_id) {
                panel_state.selected_enemy = None;
                panel_state.name_edit_buffer.clear();
                panel_state.description_buffer.clear();
                panel_state.name_edit_error = None;
                panel_state.portrait_buffer.clear();
                panel_state.portrait_error = None;
            }

            panel_state.delete_confirm_target = None;
        }
    }

    Ok(())
}
