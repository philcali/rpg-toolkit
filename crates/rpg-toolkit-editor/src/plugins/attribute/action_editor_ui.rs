//! Main action editor UI orchestrator — renders the action list, type selector,
//! and dispatches to per-action form renderers in `action_editor_forms`.

use bevy_egui::egui;

use rpg_toolkit_common::DialogTextData;

use crate::data::map::EventAction;

use super::action_editor::{ActionEditorState, ActionType, truncate_preview};
use super::action_editor_forms;

/// Renders the action editor UI into the given egui Ui.
/// Operates on the provided action list and editor state.
pub fn render_action_editor(
    ui: &mut egui::Ui,
    actions: &mut Vec<EventAction>,
    editor_state: &mut ActionEditorState,
    id_salt: &str,
    map_entries: &[(String, String)],
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
            action_editor_forms::render_show_dialog_form(ui, actions, editor_state, id_salt);
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
    }
}
