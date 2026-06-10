use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

use rpg_toolkit_common::{
    BuffTargetStat, ConsumableEffect, ConsumableEffectType, CureTargetStatus, EquipmentSlot,
    ItemCategory, ItemCategoryData, ItemId, Rarity, StatModifier, format_modifier_value,
};

use crate::data::AppEditorMode;
use crate::data::EditorUiSet;

/// Plugin that provides the Item Editor panel UI.
pub struct ItemPanelPlugin;

impl Plugin for ItemPanelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ItemPanelState>().add_systems(
            EguiPrimaryContextPass,
            item_panel_ui
                .in_set(EditorUiSet::Panels)
                .run_if(resource_equals(AppEditorMode::Item)),
        );
    }
}

/// UI state for the Item Editor panel.
#[derive(Resource, Default)]
pub struct ItemPanelState {
    /// Currently selected item ID.
    pub selected_item: Option<ItemId>,
    /// Active category filter for the item list.
    pub category_filter: Option<ItemCategory>,
    /// Whether the "Create Item" dialog is open.
    pub create_dialog_open: bool,
    /// Text buffer for the item name in the create dialog.
    pub create_name_buffer: String,
    /// Selected category in the create dialog.
    pub create_category: Option<ItemCategory>,
    /// Validation error for the create dialog.
    pub create_error: Option<String>,
    /// Item ID pending delete confirmation.
    pub delete_confirm_target: Option<ItemId>,
    /// Whether the "Add Stat Modifier" dialog is open.
    pub add_stat_dialog_open: bool,
    /// Text buffer for the stat name in the add stat dialog.
    pub add_stat_name_buffer: String,
    /// Text buffer for the stat value in the add stat dialog.
    pub add_stat_value_buffer: String,
    /// Validation error for the add stat dialog.
    pub add_stat_error: Option<String>,
    /// Text buffer for inline name editing.
    pub name_edit_buffer: String,
    /// Validation error for inline name editing.
    pub name_edit_error: Option<String>,
}

/// Returns the egui color for a given rarity tier.
fn rarity_color(rarity: Rarity) -> egui::Color32 {
    match rarity {
        Rarity::Common => egui::Color32::WHITE,
        Rarity::Uncommon => egui::Color32::from_rgb(0, 255, 0),
        Rarity::Rare => egui::Color32::from_rgb(68, 136, 255),
        Rarity::Epic => egui::Color32::from_rgb(170, 68, 255),
        Rarity::Legendary => egui::Color32::from_rgb(255, 215, 0),
    }
}

/// Returns a human-readable display name for an item category.
fn category_display_name(category: ItemCategory) -> &'static str {
    match category {
        ItemCategory::Weapon => "Weapon",
        ItemCategory::Armor => "Armor",
        ItemCategory::Accessory => "Accessory",
        ItemCategory::Consumable => "Consumable",
        ItemCategory::KeyItem => "Key Item",
    }
}

/// Returns a human-readable display name for an equipment slot.
fn equipment_slot_display(slot: EquipmentSlot) -> &'static str {
    match slot {
        EquipmentSlot::MainHand => "Main Hand",
        EquipmentSlot::OffHand => "Off Hand",
        EquipmentSlot::Head => "Head",
        EquipmentSlot::Body => "Body",
        EquipmentSlot::Legs => "Legs",
        EquipmentSlot::Feet => "Feet",
        EquipmentSlot::Accessory1 => "Accessory 1",
        EquipmentSlot::Accessory2 => "Accessory 2",
    }
}

/// Returns a human-readable display string for a consumable effect type.
fn effect_type_display(effect: &ConsumableEffectType) -> String {
    match effect {
        ConsumableEffectType::RestoreHP => "Restore HP".to_string(),
        ConsumableEffectType::RestoreMP => "Restore MP".to_string(),
        ConsumableEffectType::CureStatus { target_status } => format!("Cure {:?}", target_status),
        ConsumableEffectType::BuffStat {
            target_stat,
            duration,
        } => format!("Buff {:?} ({}t)", target_stat, duration),
    }
}

fn item_panel_ui(
    mut contexts: EguiContexts,
    mut panel_state: ResMut<ItemPanelState>,
    mut _project: ResMut<crate::data::Project>,
) -> Result {
    let ctx = contexts.ctx_mut()?;

    // === Left SidePanel: Item List ===
    egui::SidePanel::left("item_list")
        .default_width(220.0)
        .show(ctx, |ui| {
            // "New Item" button at the top
            if ui.button("➕ New Item").clicked() {
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

            egui::ComboBox::from_id_salt("item_category_filter")
                .selected_text(filter_label)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut panel_state.category_filter, None, "All");
                    ui.selectable_value(
                        &mut panel_state.category_filter,
                        Some(ItemCategory::Weapon),
                        "Weapon",
                    );
                    ui.selectable_value(
                        &mut panel_state.category_filter,
                        Some(ItemCategory::Armor),
                        "Armor",
                    );
                    ui.selectable_value(
                        &mut panel_state.category_filter,
                        Some(ItemCategory::Accessory),
                        "Accessory",
                    );
                    ui.selectable_value(
                        &mut panel_state.category_filter,
                        Some(ItemCategory::Consumable),
                        "Consumable",
                    );
                    ui.selectable_value(
                        &mut panel_state.category_filter,
                        Some(ItemCategory::KeyItem),
                        "Key Item",
                    );
                });

            ui.separator();

            // Get filtered items
            let filtered = _project.items.filtered_items(panel_state.category_filter);

            // Handle filter change: if selected item no longer visible, auto-select first or clear
            if panel_state.category_filter != previous_filter
                && let Some(ref selected_id) = panel_state.selected_item
            {
                let still_visible = filtered.iter().any(|item| &item.id == selected_id);
                if !still_visible {
                    panel_state.selected_item = filtered.first().map(|item| item.id.clone());
                }
            }

            if filtered.is_empty() {
                if _project.items.items.is_empty() {
                    ui.label("No items yet. Create one to get started.");
                } else {
                    ui.label("No items match the selected filter.");
                }
            } else {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    // Collect item info to avoid borrow conflicts
                    let item_entries: Vec<(String, String, ItemCategory, Rarity)> = filtered
                        .iter()
                        .map(|item| {
                            (
                                item.id.clone(),
                                item.display_name.clone(),
                                item.category(),
                                item.rarity,
                            )
                        })
                        .collect();

                    for (id, display_name, category, rarity) in &item_entries {
                        let is_selected = panel_state.selected_item.as_ref() == Some(id);

                        ui.horizontal(|ui| {
                            // Rarity color dot
                            let color = rarity_color(*rarity);
                            let (rect, _) =
                                ui.allocate_exact_size(egui::vec2(8.0, 8.0), egui::Sense::hover());
                            ui.painter().circle_filled(rect.center(), 4.0, color);

                            // Selectable label with name and category
                            let label_text =
                                format!("{} ({})", display_name, category_display_name(*category));
                            if ui.selectable_label(is_selected, &label_text).clicked() {
                                panel_state.selected_item = Some(id.clone());
                                // Sync name_edit_buffer with newly selected item
                                panel_state.name_edit_buffer = display_name.clone();
                                panel_state.name_edit_error = None;
                            }

                            // Delete button per item
                            if ui.small_button("🗑").clicked() {
                                panel_state.delete_confirm_target = Some(id.clone());
                            }
                        });
                    }
                });
            }
        });

    // === Right SidePanel: Item Preview ===
    egui::SidePanel::right("item_preview")
        .default_width(250.0)
        .show(ctx, |ui| {
            ui.heading("Item Preview");
            ui.separator();

            if let Some(ref selected_id) = panel_state.selected_item {
                if let Some(item) = _project.items.items.get(selected_id) {
                    // Rarity badge
                    let color = rarity_color(item.rarity);
                    let rarity_text = format!("★ {:?}", item.rarity);
                    ui.colored_label(color, &rarity_text);

                    ui.separator();

                    // Equipment slot (if applicable)
                    match &item.category_data {
                        ItemCategoryData::Weapon { equipment_slot, .. } => {
                            ui.label(format!("Slot: {}", equipment_slot_display(*equipment_slot)));
                        }
                        ItemCategoryData::Armor { equipment_slot, .. } => {
                            ui.label(format!("Slot: {}", equipment_slot_display(*equipment_slot)));
                        }
                        ItemCategoryData::Accessory { equipment_slot } => {
                            ui.label(format!("Slot: {}", equipment_slot_display(*equipment_slot)));
                        }
                        _ => {}
                    }

                    ui.separator();

                    // Stat modifiers section
                    ui.label("Stat Modifiers:");
                    if item.stat_modifiers.is_empty() {
                        ui.label("No stat modifiers");
                    } else {
                        for modifier in &item.stat_modifiers {
                            ui.label(format!(
                                "{}: {}",
                                modifier.stat_name,
                                format_modifier_value(modifier.value)
                            ));
                        }
                    }

                    // Consumable effects section
                    if let ItemCategoryData::Consumable { effects } = &item.category_data {
                        ui.separator();
                        ui.label("Effects:");
                        for effect in effects {
                            ui.label(format!(
                                "{} (potency: {})",
                                effect_type_display(&effect.effect),
                                effect.potency
                            ));
                        }
                    }

                    // Stack limit
                    if item.stackable {
                        ui.separator();
                        ui.label(format!("Stack Limit: {}", item.stack_limit));
                    }
                } else {
                    ui.label("Select an item to preview.");
                }
            } else {
                ui.label("Select an item to preview.");
            }
        });

    // === CentralPanel: Item Detail Editor ===
    egui::CentralPanel::default().show(ctx, |ui| {
        if let Some(selected_id) = panel_state.selected_item.clone() {
            if _project.items.items.contains_key(&selected_id) {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    // --- Display Name Field ---
                    ui.heading("Display Name");
                    let name_response = ui.text_edit_singleline(&mut panel_state.name_edit_buffer);

                    // Truncate to 64 chars
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
                            match _project.items.update_display_name(&selected_id, &trimmed) {
                                Ok(()) => {
                                    panel_state.name_edit_error = None;
                                    panel_state.name_edit_buffer = trimmed;
                                    _project.has_unsaved_item_changes = true;
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
                            .items
                            .items
                            .get(&selected_id)
                            .map(|i| i.description.clone())
                            .unwrap_or_default();
                        let desc_response = ui.add(
                            egui::TextEdit::multiline(&mut desc)
                                .desired_rows(3)
                                .desired_width(f32::INFINITY),
                        );
                        if desc_response.changed() {
                            // Truncate at 256 chars
                            let truncated: String = desc.chars().take(256).collect();
                            let _ = _project.items.update_description(&selected_id, &truncated);
                            _project.has_unsaved_item_changes = true;
                        }
                    }

                    ui.separator();

                    // --- Category ComboBox ---
                    ui.heading("Category");
                    {
                        let current_category = _project
                            .items
                            .items
                            .get(&selected_id)
                            .map(|i| i.category())
                            .unwrap_or(ItemCategory::Weapon);

                        let mut new_category = current_category;
                        egui::ComboBox::from_id_salt("item_category_edit")
                            .selected_text(category_display_name(current_category))
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut new_category,
                                    ItemCategory::Weapon,
                                    "Weapon",
                                );
                                ui.selectable_value(
                                    &mut new_category,
                                    ItemCategory::Armor,
                                    "Armor",
                                );
                                ui.selectable_value(
                                    &mut new_category,
                                    ItemCategory::Accessory,
                                    "Accessory",
                                );
                                ui.selectable_value(
                                    &mut new_category,
                                    ItemCategory::Consumable,
                                    "Consumable",
                                );
                                ui.selectable_value(
                                    &mut new_category,
                                    ItemCategory::KeyItem,
                                    "Key Item",
                                );
                            });

                        if new_category != current_category {
                            let _ = _project.items.change_category(&selected_id, new_category);
                            _project.has_unsaved_item_changes = true;
                        }
                    }

                    ui.separator();

                    // --- Rarity ComboBox ---
                    ui.heading("Rarity");
                    {
                        let current_rarity = _project
                            .items
                            .items
                            .get(&selected_id)
                            .map(|i| i.rarity)
                            .unwrap_or(Rarity::Common);

                        let mut new_rarity = current_rarity;
                        egui::ComboBox::from_id_salt("item_rarity_edit")
                            .selected_text(format!("{:?}", current_rarity))
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut new_rarity, Rarity::Common, "Common");
                                ui.selectable_value(&mut new_rarity, Rarity::Uncommon, "Uncommon");
                                ui.selectable_value(&mut new_rarity, Rarity::Rare, "Rare");
                                ui.selectable_value(&mut new_rarity, Rarity::Epic, "Epic");
                                ui.selectable_value(
                                    &mut new_rarity,
                                    Rarity::Legendary,
                                    "Legendary",
                                );
                            });

                        if new_rarity != current_rarity
                            && let Some(item) = _project.items.items.get_mut(&selected_id)
                        {
                            item.rarity = new_rarity;
                            _project.has_unsaved_item_changes = true;
                        }
                    }

                    ui.separator();

                    // --- Value DragValue ---
                    ui.heading("Value");
                    {
                        let current_category = _project
                            .items
                            .items
                            .get(&selected_id)
                            .map(|i| i.category())
                            .unwrap_or(ItemCategory::Weapon);

                        let mut value = _project
                            .items
                            .items
                            .get(&selected_id)
                            .map(|i| i.value)
                            .unwrap_or(0);

                        let is_key_item = current_category == ItemCategory::KeyItem;

                        let drag = egui::DragValue::new(&mut value).range(0..=u32::MAX);
                        let response = ui.add_enabled(!is_key_item, drag);

                        if is_key_item {
                            ui.label("(Key Items always have value 0)");
                        }

                        if response.changed()
                            && !is_key_item
                            && let Some(item) = _project.items.items.get_mut(&selected_id)
                        {
                            item.value = value;
                            _project.has_unsaved_item_changes = true;
                        }
                    }

                    ui.separator();

                    // --- Stackable Toggle + Stack Limit ---
                    ui.heading("Stacking");
                    {
                        let current_category = _project
                            .items
                            .items
                            .get(&selected_id)
                            .map(|i| i.category())
                            .unwrap_or(ItemCategory::Weapon);

                        let mut stackable = _project
                            .items
                            .items
                            .get(&selected_id)
                            .map(|i| i.stackable)
                            .unwrap_or(false);

                        // Consumable items are always stackable (can't toggle off)
                        // KeyItem items are always non-stackable (can't toggle on)
                        let toggle_enabled = current_category != ItemCategory::Consumable
                            && current_category != ItemCategory::KeyItem;

                        let checkbox = ui.add_enabled(
                            toggle_enabled,
                            egui::Checkbox::new(&mut stackable, "Stackable"),
                        );

                        if checkbox.changed() && toggle_enabled {
                            let _ = _project.items.set_stackable(&selected_id, stackable);
                            _project.has_unsaved_item_changes = true;
                        }

                        if current_category == ItemCategory::Consumable {
                            ui.label("(Consumable items are always stackable)");
                        } else if current_category == ItemCategory::KeyItem {
                            ui.label("(Key Items are never stackable)");
                        }

                        // Show stack_limit input only when stackable
                        let is_stackable = _project
                            .items
                            .items
                            .get(&selected_id)
                            .map(|i| i.stackable)
                            .unwrap_or(false);

                        if is_stackable {
                            let mut stack_limit = _project
                                .items
                                .items
                                .get(&selected_id)
                                .map(|i| i.stack_limit)
                                .unwrap_or(99);

                            ui.horizontal(|ui| {
                                ui.label("Stack Limit:");
                                let drag =
                                    egui::DragValue::new(&mut stack_limit).range(2..=999_u32);
                                if ui.add(drag).changed() {
                                    let _ =
                                        _project.items.set_stack_limit(&selected_id, stack_limit);
                                    _project.has_unsaved_item_changes = true;
                                }
                            });
                        }
                    }

                    ui.separator();

                    // --- Category-Specific Fields ---
                    ui.heading("Category Properties");
                    {
                        let category_data = _project
                            .items
                            .items
                            .get(&selected_id)
                            .map(|i| i.category_data.clone());

                        match category_data {
                            Some(ItemCategoryData::Weapon {
                                attack_power,
                                equipment_slot,
                            }) => {
                                let mut ap = attack_power;
                                ui.horizontal(|ui| {
                                    ui.label("Attack Power:");
                                    if ui
                                        .add(egui::DragValue::new(&mut ap).range(0..=u32::MAX))
                                        .changed()
                                        && let Some(item) =
                                            _project.items.items.get_mut(&selected_id)
                                        && let ItemCategoryData::Weapon {
                                            attack_power: ref mut stored_ap,
                                            ..
                                        } = item.category_data
                                    {
                                        *stored_ap = ap;
                                        _project.has_unsaved_item_changes = true;
                                    }
                                });

                                let mut slot = equipment_slot;
                                ui.horizontal(|ui| {
                                    ui.label("Equipment Slot:");
                                    egui::ComboBox::from_id_salt("weapon_slot_edit")
                                        .selected_text(equipment_slot_display(slot))
                                        .show_ui(ui, |ui| {
                                            ui.selectable_value(
                                                &mut slot,
                                                EquipmentSlot::MainHand,
                                                "Main Hand",
                                            );
                                            ui.selectable_value(
                                                &mut slot,
                                                EquipmentSlot::OffHand,
                                                "Off Hand",
                                            );
                                        });
                                });
                                if slot != equipment_slot
                                    && let Some(item) = _project.items.items.get_mut(&selected_id)
                                    && let ItemCategoryData::Weapon {
                                        equipment_slot: ref mut stored_slot,
                                        ..
                                    } = item.category_data
                                {
                                    *stored_slot = slot;
                                    _project.has_unsaved_item_changes = true;
                                }
                            }
                            Some(ItemCategoryData::Armor {
                                defense_power,
                                equipment_slot,
                            }) => {
                                let mut dp = defense_power;
                                ui.horizontal(|ui| {
                                    ui.label("Defense Power:");
                                    if ui
                                        .add(egui::DragValue::new(&mut dp).range(0..=u32::MAX))
                                        .changed()
                                        && let Some(item) =
                                            _project.items.items.get_mut(&selected_id)
                                        && let ItemCategoryData::Armor {
                                            defense_power: ref mut stored_dp,
                                            ..
                                        } = item.category_data
                                    {
                                        *stored_dp = dp;
                                        _project.has_unsaved_item_changes = true;
                                    }
                                });

                                let mut slot = equipment_slot;
                                ui.horizontal(|ui| {
                                    ui.label("Equipment Slot:");
                                    egui::ComboBox::from_id_salt("armor_slot_edit")
                                        .selected_text(equipment_slot_display(slot))
                                        .show_ui(ui, |ui| {
                                            ui.selectable_value(
                                                &mut slot,
                                                EquipmentSlot::Head,
                                                "Head",
                                            );
                                            ui.selectable_value(
                                                &mut slot,
                                                EquipmentSlot::Body,
                                                "Body",
                                            );
                                            ui.selectable_value(
                                                &mut slot,
                                                EquipmentSlot::Legs,
                                                "Legs",
                                            );
                                            ui.selectable_value(
                                                &mut slot,
                                                EquipmentSlot::Feet,
                                                "Feet",
                                            );
                                        });
                                });
                                if slot != equipment_slot
                                    && let Some(item) = _project.items.items.get_mut(&selected_id)
                                    && let ItemCategoryData::Armor {
                                        equipment_slot: ref mut stored_slot,
                                        ..
                                    } = item.category_data
                                {
                                    *stored_slot = slot;
                                    _project.has_unsaved_item_changes = true;
                                }
                            }
                            Some(ItemCategoryData::Accessory { equipment_slot }) => {
                                let mut slot = equipment_slot;
                                ui.horizontal(|ui| {
                                    ui.label("Equipment Slot:");
                                    egui::ComboBox::from_id_salt("accessory_slot_edit")
                                        .selected_text(equipment_slot_display(slot))
                                        .show_ui(ui, |ui| {
                                            ui.selectable_value(
                                                &mut slot,
                                                EquipmentSlot::Accessory1,
                                                "Accessory 1",
                                            );
                                            ui.selectable_value(
                                                &mut slot,
                                                EquipmentSlot::Accessory2,
                                                "Accessory 2",
                                            );
                                        });
                                });
                                if slot != equipment_slot
                                    && let Some(item) = _project.items.items.get_mut(&selected_id)
                                    && let ItemCategoryData::Accessory {
                                        equipment_slot: ref mut stored_slot,
                                    } = item.category_data
                                {
                                    *stored_slot = slot;
                                    _project.has_unsaved_item_changes = true;
                                }
                            }
                            Some(ItemCategoryData::Consumable { effects }) => {
                                ui.label(format!("Effects ({}/4):", effects.len()));

                                let mut effect_to_remove: Option<usize> = None;
                                let mut effect_updates: Vec<(usize, ConsumableEffect)> = Vec::new();

                                for (idx, effect) in effects.iter().enumerate() {
                                    ui.group(|ui| {
                                        ui.horizontal(|ui| {
                                            ui.label(format!("Effect {}:", idx + 1));
                                            // Remove button (disabled if last effect)
                                            let can_remove = effects.len() > 1;
                                            if ui
                                                .add_enabled(can_remove, egui::Button::new("🗑"))
                                                .clicked()
                                            {
                                                effect_to_remove = Some(idx);
                                            }
                                        });

                                        // Effect type combo
                                        let effect_type_label = match &effect.effect {
                                            ConsumableEffectType::RestoreHP => "Restore HP",
                                            ConsumableEffectType::RestoreMP => "Restore MP",
                                            ConsumableEffectType::CureStatus { .. } => {
                                                "Cure Status"
                                            }
                                            ConsumableEffectType::BuffStat { .. } => "Buff Stat",
                                        };
                                        let _ = effect_type_label; // used below

                                        let current_type_index = match &effect.effect {
                                            ConsumableEffectType::RestoreHP => 0,
                                            ConsumableEffectType::RestoreMP => 1,
                                            ConsumableEffectType::CureStatus { .. } => 2,
                                            ConsumableEffectType::BuffStat { .. } => 3,
                                        };

                                        let type_labels = [
                                            "Restore HP",
                                            "Restore MP",
                                            "Cure Status",
                                            "Buff Stat",
                                        ];

                                        let mut new_type_index = current_type_index;
                                        ui.horizontal(|ui| {
                                            ui.label("Type:");
                                            egui::ComboBox::from_id_salt(format!(
                                                "effect_type_{}",
                                                idx
                                            ))
                                            .selected_text(type_labels[current_type_index])
                                            .show_ui(
                                                ui,
                                                |ui| {
                                                    for (i, label) in type_labels.iter().enumerate()
                                                    {
                                                        ui.selectable_value(
                                                            &mut new_type_index,
                                                            i,
                                                            *label,
                                                        );
                                                    }
                                                },
                                            );
                                        });

                                        // Potency
                                        let mut potency = effect.potency;
                                        ui.horizontal(|ui| {
                                            ui.label("Potency:");
                                            ui.add(
                                                egui::DragValue::new(&mut potency)
                                                    .range(1..=u32::MAX),
                                            );
                                        });

                                        // Type-specific fields
                                        let mut target_status = match &effect.effect {
                                            ConsumableEffectType::CureStatus { target_status } => {
                                                *target_status
                                            }
                                            _ => CureTargetStatus::Poison,
                                        };

                                        let mut target_stat = match &effect.effect {
                                            ConsumableEffectType::BuffStat {
                                                target_stat, ..
                                            } => *target_stat,
                                            _ => BuffTargetStat::Strength,
                                        };

                                        let mut duration = match &effect.effect {
                                            ConsumableEffectType::BuffStat { duration, .. } => {
                                                *duration
                                            }
                                            _ => 1,
                                        };

                                        // Show type-specific fields based on new_type_index
                                        if new_type_index == 2 {
                                            // CureStatus
                                            ui.horizontal(|ui| {
                                                ui.label("Target Status:");
                                                egui::ComboBox::from_id_salt(format!(
                                                    "cure_status_{}",
                                                    idx
                                                ))
                                                .selected_text(format!("{:?}", target_status))
                                                .show_ui(ui, |ui| {
                                                    ui.selectable_value(
                                                        &mut target_status,
                                                        CureTargetStatus::Poison,
                                                        "Poison",
                                                    );
                                                    ui.selectable_value(
                                                        &mut target_status,
                                                        CureTargetStatus::Paralysis,
                                                        "Paralysis",
                                                    );
                                                    ui.selectable_value(
                                                        &mut target_status,
                                                        CureTargetStatus::Sleep,
                                                        "Sleep",
                                                    );
                                                    ui.selectable_value(
                                                        &mut target_status,
                                                        CureTargetStatus::Confusion,
                                                        "Confusion",
                                                    );
                                                    ui.selectable_value(
                                                        &mut target_status,
                                                        CureTargetStatus::Silence,
                                                        "Silence",
                                                    );
                                                    ui.selectable_value(
                                                        &mut target_status,
                                                        CureTargetStatus::All,
                                                        "All",
                                                    );
                                                });
                                            });
                                        } else if new_type_index == 3 {
                                            // BuffStat
                                            ui.horizontal(|ui| {
                                                ui.label("Target Stat:");
                                                egui::ComboBox::from_id_salt(format!(
                                                    "buff_stat_{}",
                                                    idx
                                                ))
                                                .selected_text(format!("{:?}", target_stat))
                                                .show_ui(ui, |ui| {
                                                    ui.selectable_value(
                                                        &mut target_stat,
                                                        BuffTargetStat::Strength,
                                                        "Strength",
                                                    );
                                                    ui.selectable_value(
                                                        &mut target_stat,
                                                        BuffTargetStat::Stamina,
                                                        "Stamina",
                                                    );
                                                    ui.selectable_value(
                                                        &mut target_stat,
                                                        BuffTargetStat::Speed,
                                                        "Speed",
                                                    );
                                                    ui.selectable_value(
                                                        &mut target_stat,
                                                        BuffTargetStat::Luck,
                                                        "Luck",
                                                    );
                                                    ui.selectable_value(
                                                        &mut target_stat,
                                                        BuffTargetStat::Wisdom,
                                                        "Wisdom",
                                                    );
                                                    ui.selectable_value(
                                                        &mut target_stat,
                                                        BuffTargetStat::Intelligence,
                                                        "Intelligence",
                                                    );
                                                });
                                            });
                                            ui.horizontal(|ui| {
                                                ui.label("Duration:");
                                                ui.add(
                                                    egui::DragValue::new(&mut duration)
                                                        .range(1..=99_u32),
                                                );
                                            });
                                        }

                                        // Determine if the effect was changed
                                        let new_effect_type = match new_type_index {
                                            0 => ConsumableEffectType::RestoreHP,
                                            1 => ConsumableEffectType::RestoreMP,
                                            2 => ConsumableEffectType::CureStatus { target_status },
                                            3 => ConsumableEffectType::BuffStat {
                                                target_stat,
                                                duration,
                                            },
                                            _ => ConsumableEffectType::RestoreHP,
                                        };

                                        let new_effect = ConsumableEffect {
                                            effect: new_effect_type,
                                            potency,
                                        };

                                        if new_effect != *effect {
                                            effect_updates.push((idx, new_effect));
                                        }
                                    });
                                }

                                // Apply effect updates
                                for (idx, updated_effect) in effect_updates {
                                    let _ = _project.items.update_consumable_effect(
                                        &selected_id,
                                        idx,
                                        updated_effect,
                                    );
                                    _project.has_unsaved_item_changes = true;
                                }

                                // Apply effect removal
                                if let Some(idx) = effect_to_remove {
                                    let _ =
                                        _project.items.remove_consumable_effect(&selected_id, idx);
                                    _project.has_unsaved_item_changes = true;
                                }

                                // Add effect button (max 4)
                                if effects.len() < 4 && ui.button("➕ Add Effect").clicked() {
                                    let default_effect = ConsumableEffect {
                                        effect: ConsumableEffectType::RestoreHP,
                                        potency: 10,
                                    };
                                    let _ = _project
                                        .items
                                        .add_consumable_effect(&selected_id, default_effect);
                                    _project.has_unsaved_item_changes = true;
                                }
                            }
                            Some(ItemCategoryData::KeyItem) => {
                                ui.label("Key Items have no category-specific properties.");
                            }
                            None => {}
                        }
                    }

                    ui.separator();

                    // --- Stat Modifiers Section ---
                    ui.heading("Stat Modifiers");
                    {
                        let modifiers: Vec<StatModifier> = _project
                            .items
                            .items
                            .get(&selected_id)
                            .map(|i| i.stat_modifiers.clone())
                            .unwrap_or_default();

                        if modifiers.is_empty() {
                            ui.label("No stat modifiers.");
                        } else {
                            let mut modifier_to_remove: Option<String> = None;
                            let mut modifier_updates: Vec<(String, i32)> = Vec::new();

                            egui::Grid::new("stat_modifier_grid")
                                .striped(true)
                                .min_col_width(80.0)
                                .show(ui, |ui| {
                                    ui.strong("Stat");
                                    ui.strong("Value");
                                    ui.strong("");
                                    ui.end_row();

                                    for modifier in &modifiers {
                                        let mut value = modifier.value;

                                        ui.label(&modifier.stat_name);

                                        if ui.add(egui::DragValue::new(&mut value)).changed() {
                                            modifier_updates
                                                .push((modifier.stat_name.clone(), value));
                                        }

                                        if ui.small_button("🗑").clicked() {
                                            modifier_to_remove = Some(modifier.stat_name.clone());
                                        }

                                        ui.end_row();
                                    }
                                });

                            // Apply modifier value updates
                            for (stat_name, value) in modifier_updates {
                                let _ = _project.items.update_stat_modifier(
                                    &selected_id,
                                    &stat_name,
                                    value,
                                );
                                _project.has_unsaved_item_changes = true;
                            }

                            // Apply modifier removal
                            if let Some(stat_name) = modifier_to_remove {
                                let _ = _project
                                    .items
                                    .remove_stat_modifier(&selected_id, &stat_name);
                                _project.has_unsaved_item_changes = true;
                            }
                        }

                        // Add stat modifier button
                        if ui.button("➕ Add Stat Modifier").clicked() {
                            panel_state.add_stat_dialog_open = true;
                            panel_state.add_stat_name_buffer.clear();
                            panel_state.add_stat_value_buffer = "0".to_string();
                            panel_state.add_stat_error = None;
                        }
                    }
                });
            } else {
                ui.label("Selected item not found.");
            }
        } else {
            ui.centered_and_justified(|ui| {
                ui.label("Select an item to edit, or create a new one.");
            });
        }
    });

    // === Add Stat Modifier Dialog ===
    if panel_state.add_stat_dialog_open {
        let mut still_open = true;
        let mut should_add = false;
        let mut should_cancel = false;

        egui::Window::new("Add Stat Modifier")
            .collapsible(false)
            .resizable(false)
            .open(&mut still_open)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Stat Name:");
                    ui.text_edit_singleline(&mut panel_state.add_stat_name_buffer);
                    if panel_state.add_stat_name_buffer.len() > 32 {
                        panel_state.add_stat_name_buffer.truncate(32);
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Value:");
                    ui.text_edit_singleline(&mut panel_state.add_stat_value_buffer);
                });

                if let Some(ref error) = panel_state.add_stat_error {
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
            panel_state.add_stat_dialog_open = false;
            panel_state.add_stat_name_buffer.clear();
            panel_state.add_stat_value_buffer.clear();
            panel_state.add_stat_error = None;
        }

        if should_cancel {
            panel_state.add_stat_dialog_open = false;
            panel_state.add_stat_name_buffer.clear();
            panel_state.add_stat_value_buffer.clear();
            panel_state.add_stat_error = None;
        }

        if should_add && let Some(ref selected_id) = panel_state.selected_item.clone() {
            let stat_name = panel_state.add_stat_name_buffer.trim().to_string();
            if stat_name.is_empty() {
                panel_state.add_stat_error = Some("Stat name must not be empty.".to_string());
            } else {
                match panel_state.add_stat_value_buffer.trim().parse::<i32>() {
                    Ok(value) => {
                        match _project
                            .items
                            .add_stat_modifier(selected_id, &stat_name, value)
                        {
                            Ok(()) => {
                                panel_state.add_stat_dialog_open = false;
                                panel_state.add_stat_name_buffer.clear();
                                panel_state.add_stat_value_buffer.clear();
                                panel_state.add_stat_error = None;
                                _project.has_unsaved_item_changes = true;
                            }
                            Err(e) => {
                                panel_state.add_stat_error = Some(e.to_string());
                            }
                        }
                    }
                    Err(_) => {
                        panel_state.add_stat_error =
                            Some("Value must be a valid integer.".to_string());
                    }
                }
            }
        }
    }

    // === Create Item Dialog ===
    if panel_state.create_dialog_open {
        let mut still_open = true;
        let mut should_create = false;
        let mut should_cancel = false;

        egui::Window::new("New Item")
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
                    egui::ComboBox::from_id_salt("create_item_category")
                        .selected_text(selected_text)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut panel_state.create_category,
                                Some(ItemCategory::Weapon),
                                "Weapon",
                            );
                            ui.selectable_value(
                                &mut panel_state.create_category,
                                Some(ItemCategory::Armor),
                                "Armor",
                            );
                            ui.selectable_value(
                                &mut panel_state.create_category,
                                Some(ItemCategory::Accessory),
                                "Accessory",
                            );
                            ui.selectable_value(
                                &mut panel_state.create_category,
                                Some(ItemCategory::Consumable),
                                "Consumable",
                            );
                            ui.selectable_value(
                                &mut panel_state.create_category,
                                Some(ItemCategory::KeyItem),
                                "Key Item",
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
                    .items
                    .create_item(&panel_state.create_name_buffer, category)
                {
                    Ok(new_id) => {
                        // Auto-select the newly created item
                        let display_name = _project
                            .items
                            .items
                            .get(&new_id)
                            .map(|i| i.display_name.clone())
                            .unwrap_or_default();
                        panel_state.selected_item = Some(new_id);
                        panel_state.name_edit_buffer = display_name;
                        panel_state.name_edit_error = None;
                        panel_state.create_dialog_open = false;
                        panel_state.create_name_buffer.clear();
                        panel_state.create_category = None;
                        panel_state.create_error = None;
                        _project.has_unsaved_item_changes = true;
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
        let item_name = _project
            .items
            .items
            .get(&target_id)
            .map(|i| i.display_name.clone())
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
                    item_name
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
            let _ = _project.items.delete_item(&target_id);
            _project.has_unsaved_item_changes = true;

            // If the deleted item was selected, select the first remaining or clear
            if panel_state.selected_item.as_ref() == Some(&target_id) {
                let sorted = _project.items.sorted_items();
                if let Some(first) = sorted.first() {
                    panel_state.selected_item = Some(first.id.clone());
                    panel_state.name_edit_buffer = first.display_name.clone();
                    panel_state.name_edit_error = None;
                } else {
                    panel_state.selected_item = None;
                    panel_state.name_edit_buffer.clear();
                    panel_state.name_edit_error = None;
                }
            }

            panel_state.delete_confirm_target = None;
        }
    }

    Ok(())
}
