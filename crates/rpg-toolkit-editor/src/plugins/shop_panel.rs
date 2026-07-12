use std::collections::{HashMap, HashSet};

use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

use rpg_toolkit_common::{
    BranchCondition, ConditionCheck, ConditionLogic, ConditionOperator, ShopEntry, ShopId,
};

use crate::data::AppEditorMode;
use crate::data::EditorUiSet;
use crate::plugins::searchable_combobox::searchable_combobox;

/// Plugin that provides the Shop Editor panel UI.
pub struct ShopPanelPlugin;

impl Plugin for ShopPanelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ShopPanelState>().add_systems(
            EguiPrimaryContextPass,
            shop_panel_ui
                .in_set(EditorUiSet::Panels)
                .run_if(resource_equals(AppEditorMode::Shop)),
        );
    }
}

/// Per-entry edit buffers for inline editing of shop entries.
#[derive(Clone, Debug, Default)]
pub struct EntryEditBuffer {
    pub buy_price: String,
    pub sell_price: String,  // empty means "Auto" (None)
    pub stock_limit: String, // empty means "Unlimited" (None)
    pub error: Option<String>,
}

/// UI state for the Shop Editor panel.
#[derive(Resource, Default)]
pub struct ShopPanelState {
    /// Currently selected shop ID.
    pub selected_shop: Option<ShopId>,
    /// Shop ID pending delete confirmation.
    pub delete_confirm_target: Option<ShopId>,
    /// Text buffer for inline name editing.
    pub name_edit_buffer: String,
    /// Validation error for inline name editing.
    pub name_edit_error: Option<String>,
    /// Text buffer for the shop list search.
    pub search_buffer: String,
    /// Search buffer for the add-entry item selector.
    pub item_search_buffer: String,
    /// Selected item ID for new entry.
    pub add_entry_item_id: Option<String>,
    /// Buy price buffer for new entry.
    pub add_entry_buy_price: String,
    /// Error message for add entry operation.
    pub add_entry_error: Option<String>,
    /// Per-entry edit buffers: Map from item_id to EntryEditBuffer.
    pub entry_edit_buffers: HashMap<String, EntryEditBuffer>,
    /// Set of item_ids whose condition editor is expanded.
    pub condition_expanded: HashSet<String>,
}

#[allow(clippy::type_complexity)]
fn shop_panel_ui(
    mut contexts: EguiContexts,
    mut panel_state: ResMut<ShopPanelState>,
    mut project: ResMut<crate::data::Project>,
) -> Result {
    let ctx = contexts.ctx_mut()?;

    // === Left SidePanel: Shop List ===
    egui::SidePanel::left("shop_list")
        .default_width(220.0)
        .show(ctx, |ui| {
            // "Create" button at the top
            if ui.button("➕ Create").clicked()
                && let Ok(new_id) = project.shops.create_shop("New Shop")
            {
                panel_state.selected_shop = Some(new_id.clone());
                panel_state.name_edit_buffer = "New Shop".to_string();
                panel_state.name_edit_error = None;
                project.has_unsaved_shop_changes = true;
            }

            ui.separator();

            // Search field
            ui.horizontal(|ui| {
                ui.label("🔍");
                ui.text_edit_singleline(&mut panel_state.search_buffer);
            });

            ui.separator();

            // Get filtered/sorted shops
            let shops = if panel_state.search_buffer.is_empty() {
                project.shops.sorted_shops()
            } else {
                project.shops.search_shops(&panel_state.search_buffer)
            };

            if shops.is_empty() {
                ui.label("No shops yet. Create one to get started.");
            } else {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    let shop_entries: Vec<(String, String)> = shops
                        .iter()
                        .map(|s| (s.id.clone(), s.display_name.clone()))
                        .collect();

                    for (id, display_name) in &shop_entries {
                        let is_selected = panel_state.selected_shop.as_ref() == Some(id);

                        ui.horizontal(|ui| {
                            if ui.selectable_label(is_selected, display_name).clicked() {
                                panel_state.selected_shop = Some(id.clone());
                                // Sync name buffer from the shop's fields
                                if let Some(shop) = project.shops.shops.get(id) {
                                    panel_state.name_edit_buffer = shop.display_name.clone();
                                }
                                panel_state.name_edit_error = None;
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

    // === Delete Confirmation Dialog ===
    if let Some(ref target_id) = panel_state.delete_confirm_target.clone() {
        let shop_name = project
            .shops
            .shops
            .get(target_id)
            .map(|s| s.display_name.clone())
            .unwrap_or_else(|| "Unknown".to_string());

        let mut still_open = true;
        let mut should_delete = false;

        egui::Window::new("Delete Shop?")
            .collapsible(false)
            .resizable(false)
            .open(&mut still_open)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.label(format!(
                    "Are you sure you want to delete \"{}\"?",
                    shop_name
                ));
                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("Delete").clicked() {
                        should_delete = true;
                    }
                    if ui.button("Cancel").clicked() {
                        panel_state.delete_confirm_target = None;
                    }
                });
            });

        if !still_open {
            panel_state.delete_confirm_target = None;
        }

        if should_delete {
            let _ = project.shops.delete_shop(target_id);
            // Clear selection if we deleted the selected shop
            if panel_state.selected_shop.as_ref() == Some(target_id) {
                panel_state.selected_shop = None;
                panel_state.name_edit_buffer.clear();
                panel_state.name_edit_error = None;
            }
            panel_state.delete_confirm_target = None;
            project.has_unsaved_shop_changes = true;
        }
    }

    // === Central Panel: Shop Details (placeholder for task 7.2) ===
    egui::CentralPanel::default().show(ctx, |ui| {
        if let Some(ref selected_id) = panel_state.selected_shop.clone() {
            if project.shops.shops.contains_key(selected_id) {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    // --- Display Name Field ---
                    ui.heading("Display Name");
                    let name_response =
                        ui.text_edit_singleline(&mut panel_state.name_edit_buffer);

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
                            match project.shops.rename_shop(selected_id, &trimmed) {
                                Ok(()) => {
                                    panel_state.name_edit_error = None;
                                    panel_state.name_edit_buffer = trimmed;
                                    project.has_unsaved_shop_changes = true;
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

                    // === Entry List ===
                    ui.heading("Shop Entries");

                    // Get entries for this shop
                    let entries: Vec<(String, String, u32, Option<u32>, Option<u32>, Option<BranchCondition>)> = project
                        .shops
                        .shops
                        .get(selected_id)
                        .map(|shop| {
                            shop.entries
                                .iter()
                                .map(|entry| {
                                    let item_name = project
                                        .items
                                        .items
                                        .get(&entry.item_id)
                                        .map(|item| item.display_name.clone())
                                        .unwrap_or_else(|| {
                                            format!("[Missing: {}]", entry.item_id)
                                        });
                                    (
                                        entry.item_id.clone(),
                                        item_name,
                                        entry.buy_price,
                                        entry.sell_price,
                                        entry.stock_limit,
                                        entry.condition.clone(),
                                    )
                                })
                                .collect()
                        })
                        .unwrap_or_default();

                    // Initialize edit buffers for entries that don't have them yet
                    for (item_id, _, buy_price, sell_price, stock_limit, _) in &entries {
                        if !panel_state.entry_edit_buffers.contains_key(item_id) {
                            panel_state.entry_edit_buffers.insert(
                                item_id.clone(),
                                EntryEditBuffer {
                                    buy_price: buy_price.to_string(),
                                    sell_price: sell_price
                                        .map(|p| p.to_string())
                                        .unwrap_or_default(),
                                    stock_limit: stock_limit
                                        .map(|l| l.to_string())
                                        .unwrap_or_default(),
                                    error: None,
                                },
                            );
                        }
                    }

                    if entries.is_empty() {
                        ui.label("No entries yet. Add items below.");
                    } else {
                        // Table header
                        egui::Grid::new("shop_entries_grid")
                            .num_columns(5)
                            .spacing([8.0, 4.0])
                            .striped(true)
                            .show(ui, |ui| {
                                ui.strong("Item");
                                ui.strong("Buy Price");
                                ui.strong("Sell Price");
                                ui.strong("Stock Limit");
                                ui.strong(""); // Remove column
                                ui.end_row();

                                let mut removal_target: Option<String> = None;
                                let mut updates: Vec<(
                                    String,
                                    Option<u32>,
                                    Option<Option<u32>>,
                                    Option<Option<u32>>,
                                )> = Vec::new();

                                for (item_id, item_name, _buy_price, _sell_price, _stock_limit, _condition) in
                                    &entries
                                {
                                    // Item name (read-only)
                                    ui.label(item_name);

                                    // Get mutable access to the edit buffer
                                    let buffer = panel_state
                                        .entry_edit_buffers
                                        .get_mut(item_id)
                                        .unwrap();

                                    // Buy Price field
                                    let buy_response = ui.add(
                                        egui::TextEdit::singleline(&mut buffer.buy_price)
                                            .desired_width(80.0),
                                    );

                                    // Sell Price field
                                    let sell_response = ui.add(
                                        egui::TextEdit::singleline(&mut buffer.sell_price)
                                            .desired_width(80.0)
                                            .hint_text("Auto"),
                                    );

                                    // Stock Limit field
                                    let stock_response = ui.add(
                                        egui::TextEdit::singleline(&mut buffer.stock_limit)
                                            .desired_width(80.0)
                                            .hint_text("Unlimited"),
                                    );

                                    // Remove button
                                    if ui.button("🗑").clicked() {
                                        removal_target = Some(item_id.clone());
                                    }

                                    ui.end_row();

                                    // Validate and apply on lost focus
                                    if buy_response.lost_focus()
                                        || sell_response.lost_focus()
                                        || stock_response.lost_focus()
                                    {
                                        let mut new_buy_price: Option<u32> = None;
                                        let mut new_sell_price: Option<Option<u32>> = None;
                                        let mut new_stock_limit: Option<Option<u32>> = None;
                                        let mut error: Option<String> = None;

                                        // Validate buy price
                                        if buy_response.lost_focus() {
                                            match buffer.buy_price.trim().parse::<u32>() {
                                                Ok(val) => {
                                                    new_buy_price = Some(val);
                                                }
                                                Err(_) => {
                                                    error = Some(
                                                        "Buy price must be 0–4,294,967,295"
                                                            .to_string(),
                                                    );
                                                }
                                            }
                                        }

                                        // Validate sell price
                                        if sell_response.lost_focus() {
                                            let trimmed = buffer.sell_price.trim();
                                            if trimmed.is_empty() {
                                                new_sell_price = Some(None);
                                            } else {
                                                match trimmed.parse::<u32>() {
                                                    Ok(val) => {
                                                        new_sell_price = Some(Some(val));
                                                    }
                                                    Err(_) => {
                                                        error = Some(
                                                            "Sell price must be 0–4,294,967,295 or empty for Auto"
                                                                .to_string(),
                                                        );
                                                    }
                                                }
                                            }
                                        }

                                        // Validate stock limit
                                        if stock_response.lost_focus() {
                                            let trimmed = buffer.stock_limit.trim();
                                            if trimmed.is_empty() {
                                                new_stock_limit = Some(None);
                                            } else {
                                                match trimmed.parse::<u32>() {
                                                    Ok(val) if (1..=9999).contains(&val) => {
                                                        new_stock_limit = Some(Some(val));
                                                    }
                                                    _ => {
                                                        error = Some(
                                                            "Stock limit must be 1–9999 or empty for Unlimited"
                                                                .to_string(),
                                                        );
                                                    }
                                                }
                                            }
                                        }

                                        buffer.error = error.clone();

                                        // Only apply if no validation errors
                                        if error.is_none()
                                            && (new_buy_price.is_some()
                                                || new_sell_price.is_some()
                                                || new_stock_limit.is_some())
                                        {
                                            updates.push((
                                                item_id.clone(),
                                                new_buy_price,
                                                new_sell_price,
                                                new_stock_limit,
                                            ));
                                        }
                                    }

                                    // Show per-entry error
                                    if let Some(ref err) = buffer.error {
                                        ui.label(""); // empty first column
                                        ui.colored_label(
                                            egui::Color32::RED,
                                            err,
                                        );
                                        ui.label("");
                                        ui.label("");
                                        ui.label("");
                                        ui.end_row();
                                    }
                                }

                                // Apply updates
                                for (item_id, buy_price, sell_price, stock_limit) in updates {
                                    let _ = project.shops.update_entry(
                                        selected_id,
                                        &item_id,
                                        buy_price,
                                        sell_price,
                                        stock_limit,
                                        None,
                                    );
                                    project.has_unsaved_shop_changes = true;
                                }

                                // Apply removal
                                if let Some(ref remove_id) = removal_target {
                                    let _ =
                                        project.shops.remove_entry(selected_id, remove_id);
                                    panel_state.entry_edit_buffers.remove(remove_id);
                                    panel_state.condition_expanded.remove(remove_id);
                                    project.has_unsaved_shop_changes = true;
                                }
                            });

                        // === Condition Editors (collapsible per entry) ===
                        for (item_id, item_name, _, _, _, condition) in &entries {
                            let is_expanded = panel_state.condition_expanded.contains(item_id);
                            let check_count = condition
                                .as_ref()
                                .map(|c| c.checks.len())
                                .unwrap_or(0);
                            let header_label = if check_count > 0 {
                                format!("📋 Conditions for {} ({} check{})", item_name, check_count, if check_count == 1 { "" } else { "s" })
                            } else {
                                format!("📋 Conditions for {}", item_name)
                            };

                            let toggle_id = item_id.clone();
                            if ui.selectable_label(is_expanded, &header_label).clicked() {
                                if is_expanded {
                                    panel_state.condition_expanded.remove(&toggle_id);
                                } else {
                                    panel_state.condition_expanded.insert(toggle_id);
                                }
                            }

                            if is_expanded {
                                ui.indent(format!("cond_editor_{}", item_id), |ui| {
                                    // Get current condition or default
                                    let mut branch = condition.clone().unwrap_or(BranchCondition {
                                        logic: ConditionLogic::All,
                                        checks: Vec::new(),
                                    });
                                    let mut changed = false;

                                    // Logic mode selector
                                    ui.horizontal(|ui| {
                                        ui.label("Logic:");
                                        let prev_logic = branch.logic;
                                        egui::ComboBox::from_id_salt(format!("cond_logic_{}", item_id))
                                            .selected_text(match branch.logic {
                                                ConditionLogic::All => "All (AND)",
                                                ConditionLogic::Any => "Any (OR)",
                                            })
                                            .show_ui(ui, |ui| {
                                                ui.selectable_value(
                                                    &mut branch.logic,
                                                    ConditionLogic::All,
                                                    "All (AND)",
                                                );
                                                ui.selectable_value(
                                                    &mut branch.logic,
                                                    ConditionLogic::Any,
                                                    "Any (OR)",
                                                );
                                            });
                                        if branch.logic != prev_logic {
                                            changed = true;
                                        }
                                    });

                                    // Condition checks list
                                    let mut remove_check_idx: Option<usize> = None;
                                    for (i, check) in branch.checks.iter_mut().enumerate() {
                                        ui.horizontal(|ui| {
                                            ui.label(format!("{}.", i + 1));
                                            let key_resp = ui.add(
                                                egui::TextEdit::singleline(&mut check.key)
                                                    .desired_width(80.0)
                                                    .hint_text("key"),
                                            );
                                            if key_resp.changed() {
                                                changed = true;
                                            }

                                            let prev_op = check.operator;
                                            let operator_text = match check.operator {
                                                ConditionOperator::Equals => "Equals",
                                                ConditionOperator::NotEquals => "Not Equals",
                                                ConditionOperator::Exists => "Exists",
                                                ConditionOperator::NotExists => "Not Exists",
                                            };
                                            egui::ComboBox::from_id_salt(format!("cond_op_{}_{}", item_id, i))
                                                .selected_text(operator_text)
                                                .width(90.0)
                                                .show_ui(ui, |ui| {
                                                    ui.selectable_value(
                                                        &mut check.operator,
                                                        ConditionOperator::Equals,
                                                        "Equals",
                                                    );
                                                    ui.selectable_value(
                                                        &mut check.operator,
                                                        ConditionOperator::NotEquals,
                                                        "Not Equals",
                                                    );
                                                    ui.selectable_value(
                                                        &mut check.operator,
                                                        ConditionOperator::Exists,
                                                        "Exists",
                                                    );
                                                    ui.selectable_value(
                                                        &mut check.operator,
                                                        ConditionOperator::NotExists,
                                                        "Not Exists",
                                                    );
                                                });
                                            if check.operator != prev_op {
                                                changed = true;
                                            }

                                            // Value field — disabled for Exists/NotExists
                                            let needs_value = matches!(
                                                check.operator,
                                                ConditionOperator::Equals | ConditionOperator::NotEquals
                                            );
                                            if needs_value {
                                                let mut value_str = check.value.clone().unwrap_or_default();
                                                let val_resp = ui.add(
                                                    egui::TextEdit::singleline(&mut value_str)
                                                        .desired_width(80.0)
                                                        .hint_text("value"),
                                                );
                                                if val_resp.changed() {
                                                    check.value = if value_str.is_empty() {
                                                        None
                                                    } else {
                                                        Some(value_str)
                                                    };
                                                    changed = true;
                                                }
                                            } else {
                                                ui.add_enabled(
                                                    false,
                                                    egui::TextEdit::singleline(&mut String::from("—"))
                                                        .desired_width(80.0),
                                                );
                                                if check.value.is_some() {
                                                    check.value = None;
                                                    changed = true;
                                                }
                                            }

                                            if ui.small_button("✕").clicked() {
                                                remove_check_idx = Some(i);
                                            }
                                        });
                                    }

                                    if let Some(idx) = remove_check_idx {
                                        branch.checks.remove(idx);
                                        changed = true;
                                    }

                                    // Add Condition button (max 16)
                                    let at_max = branch.checks.len() >= 16;
                                    ui.horizontal(|ui| {
                                        if ui
                                            .add_enabled(!at_max, egui::Button::new("➕ Add Condition"))
                                            .clicked()
                                        {
                                            branch.checks.push(ConditionCheck {
                                                key: String::new(),
                                                operator: ConditionOperator::Equals,
                                                value: None,
                                            });
                                            changed = true;
                                        }
                                        if at_max {
                                            ui.label(
                                                egui::RichText::new("Max 16 conditions reached.")
                                                    .color(egui::Color32::from_rgb(200, 150, 50)),
                                            );
                                        }
                                    });

                                    // Persist changes
                                    if changed {
                                        let new_condition = if branch.checks.is_empty() {
                                            None
                                        } else {
                                            Some(branch)
                                        };
                                        let _ = project.shops.update_entry(
                                            selected_id,
                                            item_id,
                                            None,
                                            None,
                                            None,
                                            Some(new_condition),
                                        );
                                        project.has_unsaved_shop_changes = true;
                                    }
                                });
                                ui.add_space(4.0);
                            }
                        }
                    }

                    ui.separator();

                    // === Add Entry Section ===
                    ui.heading("Add Entry");

                    // Item selector
                    let items_list: Vec<(String, String)> = project
                        .items
                        .items
                        .values()
                        .map(|item| (item.id.clone(), item.display_name.clone()))
                        .collect();

                    let current_label = panel_state
                        .add_entry_item_id
                        .as_ref()
                        .and_then(|id| project.items.items.get(id))
                        .map(|item| item.display_name.clone())
                        .unwrap_or_else(|| "Select item…".to_string());

                    ui.horizontal(|ui| {
                        ui.label("Item:");
                        if let Some(selected_id) = searchable_combobox(
                            ui,
                            "add_entry_item_selector",
                            &current_label,
                            &items_list,
                            &mut panel_state.item_search_buffer,
                        ) {
                            panel_state.add_entry_item_id = Some(selected_id);
                            panel_state.add_entry_error = None;
                        }
                    });

                    // Buy price input for new entry
                    ui.horizontal(|ui| {
                        ui.label("Buy Price:");
                        ui.add(
                            egui::TextEdit::singleline(&mut panel_state.add_entry_buy_price)
                                .desired_width(100.0),
                        );
                    });

                    // Default buy price if empty
                    if panel_state.add_entry_buy_price.is_empty() {
                        panel_state.add_entry_buy_price = "100".to_string();
                    }

                    // Add Entry button
                    if ui.button("➕ Add Entry").clicked() {
                        if let Some(ref item_id) = panel_state.add_entry_item_id.clone() {
                            // Validate buy price
                            match panel_state.add_entry_buy_price.trim().parse::<u32>() {
                                Ok(buy_price) => {
                                    let entry = ShopEntry {
                                        item_id: item_id.clone(),
                                        buy_price,
                                        sell_price: None,
                                        stock_limit: None,
                                        condition: None,
                                    };
                                    match project.shops.add_entry(selected_id, entry) {
                                        Ok(()) => {
                                            // Clear inputs on success
                                            panel_state.add_entry_item_id = None;
                                            panel_state.add_entry_buy_price =
                                                "100".to_string();
                                            panel_state.item_search_buffer.clear();
                                            panel_state.add_entry_error = None;
                                            project.has_unsaved_shop_changes = true;
                                        }
                                        Err(e) => {
                                            panel_state.add_entry_error =
                                                Some(e.to_string());
                                        }
                                    }
                                }
                                Err(_) => {
                                    panel_state.add_entry_error = Some(
                                        "Buy price must be 0–4,294,967,295".to_string(),
                                    );
                                }
                            }
                        } else {
                            panel_state.add_entry_error =
                                Some("Please select an item first.".to_string());
                        }
                    }

                    // Show add entry error
                    if let Some(ref error) = panel_state.add_entry_error {
                        ui.colored_label(egui::Color32::RED, error);
                    }
                });
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label("Select a shop to edit.");
                });
            }
        } else {
            ui.centered_and_justified(|ui| {
                ui.label("Select a shop to edit.");
            });
        }
    });

    Ok(())
}
