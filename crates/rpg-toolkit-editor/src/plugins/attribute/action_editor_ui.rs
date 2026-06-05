//! Main action editor UI orchestrator — renders the action list, type selector,
//! and dispatches to per-action form renderers in `action_editor_forms`.

use bevy_egui::egui;

use rpg_toolkit_common::DialogTextData;

use crate::data::map::EventAction;

use super::action_editor::{ActionEditorState, ActionType, truncate_preview};
use super::action_editor_forms;

/// Renders the action editor UI into the given egui Ui.
/// Operates on the provided action list and editor state.
/// `depth` controls nesting: at depth >= 1, Branch and StateCheck are excluded
/// from the action type dropdown to prevent deep nesting in the editor.
pub fn render_action_editor(
    ui: &mut egui::Ui,
    actions: &mut Vec<EventAction>,
    editor_state: &mut ActionEditorState,
    id_salt: &str,
    map_entries: &[(String, String)],
    face_portraits: &std::collections::HashMap<String, String>,
    depth: usize,
) {
    // Display existing actions with remove/reorder/edit controls
    let mut remove_idx: Option<usize> = None;
    let mut swap: Option<(usize, usize)> = None;
    let mut edit_idx: Option<usize> = None;
    let action_count = actions.len();

    for (i, action) in actions.iter().enumerate() {
        let is_being_edited = editor_state.editing_index == Some(i);
        ui.horizontal(|ui| {
            let label = match action {
                EventAction::JumpTo {
                    target_map_id,
                    target_x,
                    target_y,
                    ..
                } => {
                    format!(
                        "{}. JumpTo → map: {}, ({}, {})",
                        i + 1,
                        target_map_id,
                        target_x,
                        target_y
                    )
                }
                EventAction::ShowDialog { text, .. } => {
                    let preview = match text {
                        DialogTextData::Inline(s) => truncate_preview(s, 40),
                        DialogTextData::Id(id) => {
                            format!("ID: {}", id)
                        }
                    };
                    format!("{}. ShowDialog — {}", i + 1, preview)
                }
                EventAction::ScreenShake {
                    intensity,
                    duration,
                    mode,
                } => {
                    format!(
                        "{}. ScreenShake — intensity: {}, duration: {}, mode: {:?}",
                        i + 1,
                        intensity,
                        duration,
                        mode
                    )
                }
                EventAction::StopScreenShake => {
                    format!("{}. StopScreenShake", i + 1)
                }
                EventAction::FadeTransition {
                    fade_type,
                    duration,
                    ..
                } => {
                    format!(
                        "{}. FadeTransition — {:?}, duration: {}",
                        i + 1,
                        fade_type,
                        duration
                    )
                }
                EventAction::SetState { key, value } => {
                    format!("{}. SetState — {}: {}", i + 1, key, value)
                }
                EventAction::SetPlayerAppearance { appearance } => {
                    format!("{}. SetPlayerAppearance — {:?}", i + 1, appearance)
                }
                EventAction::StateCheck {
                    key,
                    value,
                    on_true,
                    on_false,
                } => {
                    let val_str = value.as_deref().unwrap_or("*");
                    format!(
                        "{}. StateCheck — {}: {} | true:{} false:{}",
                        i + 1,
                        key,
                        val_str,
                        on_true.len(),
                        on_false.len()
                    )
                }
                EventAction::Branch {
                    condition,
                    on_true,
                    on_false,
                } => {
                    let logic = match condition.logic {
                        rpg_toolkit_common::ConditionLogic::All => "All",
                        rpg_toolkit_common::ConditionLogic::Any => "Any",
                    };
                    format!(
                        "{}. Branch — {} [{} checks] | true:{} false:{}",
                        i + 1,
                        logic,
                        condition.checks.len(),
                        on_true.len(),
                        on_false.len()
                    )
                }
            };
            if is_being_edited {
                ui.label(
                    egui::RichText::new(label)
                        .strong()
                        .color(egui::Color32::from_rgb(100, 180, 255)),
                );
            } else {
                ui.label(label);
            }

            if i > 0 && ui.small_button("▲").clicked() {
                swap = Some((i, i - 1));
            }
            if i + 1 < action_count && ui.small_button("▼").clicked() {
                swap = Some((i, i + 1));
            }
            if ui
                .small_button("✏")
                .on_hover_text("Edit this action")
                .clicked()
            {
                edit_idx = Some(i);
            }
            if ui.small_button("✕").clicked() {
                remove_idx = Some(i);
            }
        });
    }

    // Render collapsible nested action editors for Branch and StateCheck items
    if depth == 0 {
        render_nested_branch_editors(ui, actions, id_salt, map_entries, face_portraits);
    }

    if let Some(idx) = remove_idx {
        if editor_state.editing_index == Some(idx) {
            editor_state.editing_index = None;
        } else if let Some(ei) = editor_state.editing_index
            && idx < ei
        {
            editor_state.editing_index = Some(ei - 1);
        }
        actions.remove(idx);
    }
    if let Some((a, b)) = swap {
        actions.swap(a, b);
        if editor_state.editing_index == Some(a) {
            editor_state.editing_index = Some(b);
        } else if editor_state.editing_index == Some(b) {
            editor_state.editing_index = Some(a);
        }
    }
    if let Some(idx) = edit_idx
        && let Some(action) = actions.get(idx).cloned()
    {
        editor_state.load_from_action(&action, idx);
    }

    ui.separator();

    // Action type selector
    let is_editing_action = editor_state.editing_index.is_some();
    let form_label = if is_editing_action {
        "Edit Action:"
    } else {
        "Add Action:"
    };
    ui.label(egui::RichText::new(form_label).strong());

    ui.horizontal(|ui| {
        ui.label("Action Type:");
        let action_type_text = match editor_state.action_type {
            ActionType::JumpTo => "JumpTo",
            ActionType::ShowDialog => "ShowDialog",
            ActionType::ScreenShake => "ScreenShake",
            ActionType::StopScreenShake => "StopScreenShake",
            ActionType::FadeTransition => "FadeTransition",
            ActionType::SetState => "SetState",
            ActionType::SetPlayerAppearance => "SetPlayerAppearance",
            ActionType::StateCheck => "StateCheck",
            ActionType::Branch => "Branch",
        };
        egui::ComboBox::from_id_salt(format!("{}_action_type", id_salt))
            .selected_text(action_type_text)
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut editor_state.action_type, ActionType::JumpTo, "JumpTo");
                ui.selectable_value(
                    &mut editor_state.action_type,
                    ActionType::ShowDialog,
                    "ShowDialog",
                );
                ui.selectable_value(
                    &mut editor_state.action_type,
                    ActionType::ScreenShake,
                    "ScreenShake",
                );
                ui.selectable_value(
                    &mut editor_state.action_type,
                    ActionType::StopScreenShake,
                    "StopScreenShake",
                );
                ui.selectable_value(
                    &mut editor_state.action_type,
                    ActionType::FadeTransition,
                    "FadeTransition",
                );
                ui.selectable_value(
                    &mut editor_state.action_type,
                    ActionType::SetState,
                    "SetState",
                );
                ui.selectable_value(
                    &mut editor_state.action_type,
                    ActionType::SetPlayerAppearance,
                    "SetPlayerAppearance",
                );
                // Only show StateCheck and Branch at depth 0
                if depth == 0 {
                    ui.selectable_value(
                        &mut editor_state.action_type,
                        ActionType::StateCheck,
                        "StateCheck",
                    );
                    ui.selectable_value(
                        &mut editor_state.action_type,
                        ActionType::Branch,
                        "Branch",
                    );
                }
            });
    });

    ui.separator();

    // Dispatch to per-action form renderers
    match editor_state.action_type {
        ActionType::JumpTo => {
            action_editor_forms::render_jumpto_form(
                ui,
                actions,
                editor_state,
                id_salt,
                map_entries,
            );
        }
        ActionType::ShowDialog => {
            action_editor_forms::render_show_dialog_form(
                ui,
                actions,
                editor_state,
                id_salt,
                face_portraits,
            );
        }
        ActionType::ScreenShake => {
            action_editor_forms::render_screen_shake_form(ui, actions, editor_state, id_salt);
        }
        ActionType::StopScreenShake => {
            action_editor_forms::render_stop_screen_shake_form(ui, actions, editor_state);
        }
        ActionType::FadeTransition => {
            action_editor_forms::render_fade_transition_form(ui, actions, editor_state, id_salt);
        }
        ActionType::SetState => {
            action_editor_forms::render_set_state_form(ui, actions, editor_state);
        }
        ActionType::SetPlayerAppearance => {
            action_editor_forms::render_set_player_appearance_form(
                ui,
                actions,
                editor_state,
                id_salt,
            );
        }
        ActionType::StateCheck => {
            action_editor_forms::render_state_check_form(ui, actions, editor_state, id_salt);
        }
        ActionType::Branch => {
            action_editor_forms::render_branch_form(ui, actions, editor_state, id_salt);
        }
    }
}

/// Renders collapsible nested action editors for Branch and StateCheck items in the action list.
/// This allows visual editing of on_true/on_false branches inline.
fn render_nested_branch_editors(
    ui: &mut egui::Ui,
    actions: &mut [EventAction],
    id_salt: &str,
    map_entries: &[(String, String)],
    face_portraits: &std::collections::HashMap<String, String>,
) {
    // We need indexed mutable access. Use a simple index loop.
    let len = actions.len();
    for (i, action) in actions.iter_mut().enumerate().take(len) {
        match action {
            EventAction::Branch {
                on_true, on_false, ..
            } => {
                let on_true_count = on_true.len();
                let on_false_count = on_false.len();
                let nested_salt_true = format!("{}_branch_{}_true", id_salt, i);
                let nested_salt_false = format!("{}_branch_{}_false", id_salt, i);

                ui.indent(format!("branch_indent_{}", i), |ui| {
                    // on_true collapsible
                    egui::CollapsingHeader::new(format!("  ↳ on_true ({} actions)", on_true_count))
                        .id_salt(&nested_salt_true)
                        .show(ui, |ui| {
                            // Extract on_true for mutable editing
                            if let EventAction::Branch { on_true, .. } = action {
                                let mut nested_editor = ActionEditorState::default();
                                render_action_editor(
                                    ui,
                                    on_true,
                                    &mut nested_editor,
                                    &nested_salt_true,
                                    map_entries,
                                    face_portraits,
                                    1,
                                );
                            }
                        });

                    // on_false collapsible
                    egui::CollapsingHeader::new(format!(
                        "  ↳ on_false ({} actions)",
                        on_false_count
                    ))
                    .id_salt(&nested_salt_false)
                    .show(ui, |ui| {
                        if let EventAction::Branch { on_false, .. } = action {
                            let mut nested_editor = ActionEditorState::default();
                            render_action_editor(
                                ui,
                                on_false,
                                &mut nested_editor,
                                &nested_salt_false,
                                map_entries,
                                face_portraits,
                                1,
                            );
                        }
                    });
                });
            }
            EventAction::StateCheck {
                on_true, on_false, ..
            } => {
                let on_true_count = on_true.len();
                let on_false_count = on_false.len();
                let nested_salt_true = format!("{}_statecheck_{}_true", id_salt, i);
                let nested_salt_false = format!("{}_statecheck_{}_false", id_salt, i);

                ui.indent(format!("statecheck_indent_{}", i), |ui| {
                    egui::CollapsingHeader::new(format!("  ↳ on_true ({} actions)", on_true_count))
                        .id_salt(&nested_salt_true)
                        .show(ui, |ui| {
                            if let EventAction::StateCheck { on_true, .. } = action {
                                let mut nested_editor = ActionEditorState::default();
                                render_action_editor(
                                    ui,
                                    on_true,
                                    &mut nested_editor,
                                    &nested_salt_true,
                                    map_entries,
                                    face_portraits,
                                    1,
                                );
                            }
                        });

                    egui::CollapsingHeader::new(format!(
                        "  ↳ on_false ({} actions)",
                        on_false_count
                    ))
                    .id_salt(&nested_salt_false)
                    .show(ui, |ui| {
                        if let EventAction::StateCheck { on_false, .. } = action {
                            let mut nested_editor = ActionEditorState::default();
                            render_action_editor(
                                ui,
                                on_false,
                                &mut nested_editor,
                                &nested_salt_false,
                                map_entries,
                                face_portraits,
                                1,
                            );
                        }
                    });
                });
            }
            _ => {}
        }
    }
}
