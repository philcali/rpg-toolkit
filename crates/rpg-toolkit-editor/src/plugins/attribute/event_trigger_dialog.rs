//! Modal dialog for editing event trigger action sequences on a tile.

use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

use super::action_editor::ActionEditorState;
use super::action_editor_ui::render_action_editor;
use crate::data::Project;
use crate::data::commands::{EditCommand, EditCommandKind};
use crate::data::map::EventAction;
use rpg_toolkit_common::{
    BranchCondition, ConditionCheck, ConditionLogic, ConditionOperator, ConditionalTrigger,
};

/// Resource for the event trigger editing dialog.
#[derive(Resource, Default)]
pub struct EventTriggerDialog {
    pub open: bool,
    pub layer_index: usize,
    pub tile_x: u32,
    pub tile_y: u32,
    pub actions: Vec<EventAction>,
    pub original_actions: Vec<EventAction>,
    pub action_editor: ActionEditorState,
    /// Conditional triggers for this tile (condition-gated action overrides).
    pub conditional_triggers: Vec<ConditionalTrigger>,
    pub original_conditional_triggers: Vec<ConditionalTrigger>,
    /// Per-trigger action editor states for nested editing.
    pub conditional_trigger_editors: Vec<ActionEditorState>,
}

/// Egui panel for editing event triggers on a tile.
pub fn event_trigger_panel_ui(
    mut contexts: EguiContexts,
    mut dialog: ResMut<EventTriggerDialog>,
    mut project: ResMut<Project>,
    mut edit_events: MessageWriter<EditCommand>,
) -> Result {
    if !dialog.open {
        return Ok(());
    }

    let ctx = contexts.ctx_mut()?;

    let mut should_close = false;
    let mut should_save = false;

    // Collect map entries for the JumpTo map selector
    let map_entries: Vec<(String, String)> = project
        .maps
        .iter()
        .map(|(id, m)| (id.clone(), m.name.clone()))
        .collect();

    egui::Window::new("Event Trigger Editor")
        .collapsible(false)
        .resizable(true)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.label(format!(
                "Tile ({}, {}) — Layer {}",
                dialog.tile_x, dialog.tile_y, dialog.layer_index
            ));
            ui.separator();

            let dialog = &mut *dialog;

            // === Conditional Triggers section ===
            render_conditional_triggers_panel(
                ui,
                &mut dialog.conditional_triggers,
                &mut dialog.conditional_trigger_editors,
                "evt_cond_trig",
                &map_entries,
                &project.face_portraits,
            );

            ui.separator();
            ui.label(egui::RichText::new("Default Actions").strong());
            ui.label("Used when no conditional trigger matches:");

            render_action_editor(
                ui,
                &mut dialog.actions,
                &mut dialog.action_editor,
                "event_trigger",
                &map_entries,
                &project.face_portraits,
                0,
                None,
            );

            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("Save").clicked() {
                    should_save = true;
                }
                if ui.button("Cancel").clicked() {
                    should_close = true;
                }
            });
        });

    if should_save {
        let layer_index = dialog.layer_index;
        let x = dialog.tile_x;
        let y = dialog.tile_y;
        let old_trigger = dialog.original_actions.clone();
        let new_trigger = dialog.actions.clone();
        let old_conditional_triggers = dialog.original_conditional_triggers.clone();
        let new_conditional_triggers = dialog.conditional_triggers.clone();

        // Apply the change to the map
        if let Some(map) = project.active_map_mut()
            && let Some(layer) = map.layers.get_mut(layer_index)
            && let Some(attr_row) = layer.attributes.cells.get_mut(y as usize)
            && let Some(cell) = attr_row.get_mut(x as usize)
        {
            cell.event_trigger = new_trigger.clone();
            cell.conditional_triggers = new_conditional_triggers.clone();
        }

        edit_events.write(EditCommand {
            kind: EditCommandKind::SetEventTrigger {
                layer_index,
                x,
                y,
                old_trigger,
                new_trigger,
            },
        });

        // Only emit a SetConditionalTriggers command if they actually changed
        if old_conditional_triggers != new_conditional_triggers {
            edit_events.write(EditCommand {
                kind: EditCommandKind::SetConditionalTriggers {
                    layer_index,
                    x,
                    y,
                    old_triggers: old_conditional_triggers,
                    new_triggers: new_conditional_triggers,
                },
            });
        }

        dialog.open = false;
    }

    if should_close {
        dialog.open = false;
    }

    Ok(())
}

/// Renders the "Conditional Triggers" panel with add/remove/reorder and inline editors.
/// Shared between the event trigger dialog and NPC dialog.
pub fn render_conditional_triggers_panel(
    ui: &mut egui::Ui,
    conditional_triggers: &mut Vec<ConditionalTrigger>,
    editors: &mut Vec<ActionEditorState>,
    id_salt: &str,
    map_entries: &[(String, String)],
    face_portraits: &std::collections::HashMap<String, String>,
) {
    ui.label(egui::RichText::new("Conditional Triggers").strong());
    ui.label("First matching condition overrides the default actions:");

    // Ensure editors vec matches triggers count
    while editors.len() < conditional_triggers.len() {
        editors.push(ActionEditorState::default());
    }
    editors.truncate(conditional_triggers.len());

    let mut remove_idx: Option<usize> = None;
    let mut swap: Option<(usize, usize)> = None;

    let trigger_count = conditional_triggers.len();
    for i in 0..trigger_count {
        let trigger = &conditional_triggers[i];
        let logic_str = match trigger.condition.logic {
            ConditionLogic::All => "All",
            ConditionLogic::Any => "Any",
        };
        let summary = format!(
            "Condition {} — {} [{} checks], {} actions",
            i + 1,
            logic_str,
            trigger.condition.checks.len(),
            trigger.actions.len()
        );

        let header_id = format!("{}_{}", id_salt, i);
        egui::CollapsingHeader::new(&summary)
            .id_salt(&header_id)
            .show(ui, |ui| {
                // Condition editor inline
                render_condition_editor(
                    ui,
                    &mut conditional_triggers[i].condition,
                    &format!("{}_cond_{}", id_salt, i),
                );

                ui.separator();
                ui.label("Actions:");

                // Nested action editor at depth 1
                render_action_editor(
                    ui,
                    &mut conditional_triggers[i].actions,
                    &mut editors[i],
                    &format!("{}_actions_{}", id_salt, i),
                    map_entries,
                    face_portraits,
                    1,
                    None,
                );
            });

        // Reorder and remove buttons
        ui.horizontal(|ui| {
            ui.add_space(20.0);
            if i > 0 && ui.small_button("▲").on_hover_text("Move up").clicked() {
                swap = Some((i, i - 1));
            }
            if i + 1 < trigger_count && ui.small_button("▼").on_hover_text("Move down").clicked()
            {
                swap = Some((i, i + 1));
            }
            if ui
                .small_button("✕")
                .on_hover_text("Remove trigger")
                .clicked()
            {
                remove_idx = Some(i);
            }
        });
    }

    if let Some((a, b)) = swap {
        conditional_triggers.swap(a, b);
        editors.swap(a, b);
    }
    if let Some(idx) = remove_idx {
        conditional_triggers.remove(idx);
        editors.remove(idx);
    }

    if ui.button("Add Conditional Trigger").clicked() {
        conditional_triggers.push(ConditionalTrigger {
            condition: BranchCondition {
                logic: ConditionLogic::All,
                checks: Vec::new(),
            },
            actions: Vec::new(),
        });
        editors.push(ActionEditorState::default());
    }
}

/// Renders a condition editor (logic selector + condition checks list) for a BranchCondition.
fn render_condition_editor(ui: &mut egui::Ui, condition: &mut BranchCondition, id_salt: &str) {
    // Logic selector
    ui.horizontal(|ui| {
        ui.label("Logic:");
        egui::ComboBox::from_id_salt(format!("{}_logic", id_salt))
            .selected_text(match condition.logic {
                ConditionLogic::All => "All (AND)",
                ConditionLogic::Any => "Any (OR)",
            })
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut condition.logic, ConditionLogic::All, "All (AND)");
                ui.selectable_value(&mut condition.logic, ConditionLogic::Any, "Any (OR)");
            });
    });

    // Condition checks list
    let mut remove_check_idx: Option<usize> = None;
    for (i, check) in condition.checks.iter_mut().enumerate() {
        ui.horizontal(|ui| {
            ui.label(format!("{}.", i + 1));
            ui.add(
                egui::TextEdit::singleline(&mut check.key)
                    .desired_width(80.0)
                    .hint_text("key"),
            );

            let operator_text = match check.operator {
                ConditionOperator::Equals => "Equals",
                ConditionOperator::NotEquals => "Not Equals",
                ConditionOperator::Exists => "Exists",
                ConditionOperator::NotExists => "Not Exists",
            };
            egui::ComboBox::from_id_salt(format!("{}_op_{}", id_salt, i))
                .selected_text(operator_text)
                .width(90.0)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut check.operator, ConditionOperator::Equals, "Equals");
                    ui.selectable_value(
                        &mut check.operator,
                        ConditionOperator::NotEquals,
                        "Not Equals",
                    );
                    ui.selectable_value(&mut check.operator, ConditionOperator::Exists, "Exists");
                    ui.selectable_value(
                        &mut check.operator,
                        ConditionOperator::NotExists,
                        "Not Exists",
                    );
                });

            let needs_value = matches!(
                check.operator,
                ConditionOperator::Equals | ConditionOperator::NotEquals
            );
            if needs_value {
                let mut value_str = check.value.clone().unwrap_or_default();
                if ui
                    .add(
                        egui::TextEdit::singleline(&mut value_str)
                            .desired_width(80.0)
                            .hint_text("value"),
                    )
                    .changed()
                {
                    check.value = if value_str.is_empty() {
                        None
                    } else {
                        Some(value_str)
                    };
                }
            } else {
                ui.add_enabled(
                    false,
                    egui::TextEdit::singleline(&mut String::from("—")).desired_width(80.0),
                );
                check.value = None;
            }

            if ui.small_button("✕").clicked() {
                remove_check_idx = Some(i);
            }
        });
    }

    if let Some(idx) = remove_check_idx {
        condition.checks.remove(idx);
    }

    if ui.button("Add Condition").clicked() {
        condition.checks.push(ConditionCheck {
            key: String::new(),
            operator: ConditionOperator::Equals,
            value: None,
        });
    }
}
