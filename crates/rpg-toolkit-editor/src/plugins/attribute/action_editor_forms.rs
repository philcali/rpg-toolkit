//! Per-action-type form renderers for the action editor UI.
//! Each function renders the form fields and add/update/cancel buttons
//! for a specific EventAction variant.

use bevy_egui::egui;

use rpg_toolkit_common::{
    AppPhase, ConditionCheck, ConditionLogic, ConditionOperator, DialogPositionData, FadeType,
    PlayerAppearance, ScreenShakeMode, TransferDirection,
};

use crate::data::map::EventAction;
use crate::plugins::searchable_combobox::searchable_combobox;

use super::action_editor::{ActionEditorState, DialogTextMode, EditorChoice};

pub fn render_jumpto_form(
    ui: &mut egui::Ui,
    actions: &mut Vec<EventAction>,
    editor_state: &mut ActionEditorState,
    id_salt: &str,
    map_entries: &[(String, String)],
) {
    let jumpto_form_label = if editor_state.editing_index.is_some() {
        "Edit JumpTo Action:"
    } else {
        "Add JumpTo Action:"
    };
    ui.label(jumpto_form_label);

    ui.horizontal(|ui| {
        ui.label("Target Map:");
        let selected_text = if editor_state.target_map_id.is_empty() {
            "Select map...".to_string()
        } else {
            map_entries
                .iter()
                .find(|(id, _)| *id == editor_state.target_map_id)
                .map(|(_, name)| name.clone())
                .unwrap_or_else(|| editor_state.target_map_id.clone())
        };
        egui::ComboBox::from_id_salt(format!("{}_map_select", id_salt))
            .selected_text(selected_text)
            .show_ui(ui, |ui| {
                for (id, name) in map_entries {
                    ui.selectable_value(&mut editor_state.target_map_id, id.clone(), name);
                }
            });
    });

    ui.horizontal(|ui| {
        ui.label("X:");
        ui.text_edit_singleline(&mut editor_state.target_x);
        ui.label("Y:");
        ui.text_edit_singleline(&mut editor_state.target_y);
    });

    ui.horizontal(|ui| {
        ui.label("Target Elevation (optional):");
        ui.text_edit_singleline(&mut editor_state.target_elevation);
    });

    let jumpto_button_label = if editor_state.editing_index.is_some() {
        "Update JumpTo"
    } else {
        "Add JumpTo"
    };
    if ui.button(jumpto_button_label).clicked()
        && let Some(new_action) = editor_state.build_action()
    {
        if let Some(idx) = editor_state.editing_index {
            if idx < actions.len() {
                actions[idx] = new_action;
            }
            editor_state.editing_index = None;
        } else {
            actions.push(new_action);
        }
        editor_state.target_map_id = String::new();
        editor_state.target_x = "0".to_string();
        editor_state.target_y = "0".to_string();
        editor_state.target_elevation = String::new();
    }
    if editor_state.editing_index.is_some() && ui.button("Cancel Edit").clicked() {
        editor_state.editing_index = None;
        editor_state.target_map_id = String::new();
        editor_state.target_x = "0".to_string();
        editor_state.target_y = "0".to_string();
        editor_state.target_elevation = String::new();
    }
}

pub fn render_show_dialog_form(
    ui: &mut egui::Ui,
    actions: &mut Vec<EventAction>,
    editor_state: &mut ActionEditorState,
    id_salt: &str,
    face_portraits: &std::collections::HashMap<String, String>,
) {
    let show_dialog_form_label = if editor_state.editing_index.is_some() {
        "Edit ShowDialog Action:"
    } else {
        "Add ShowDialog Action:"
    };
    ui.label(show_dialog_form_label);

    // Text source toggle
    ui.horizontal(|ui| {
        ui.label("Text Source:");
        ui.radio_value(
            &mut editor_state.dialog_text_mode,
            DialogTextMode::Inline,
            "Inline",
        );
        ui.radio_value(
            &mut editor_state.dialog_text_mode,
            DialogTextMode::TextId,
            "Text ID",
        );
    });

    match editor_state.dialog_text_mode {
        DialogTextMode::Inline => {
            ui.label("Dialog Text:");
            ui.text_edit_multiline(&mut editor_state.dialog_inline_text);
        }
        DialogTextMode::TextId => {
            ui.horizontal(|ui| {
                ui.label("Text ID:");
                ui.text_edit_singleline(&mut editor_state.dialog_text_id);
            });
        }
    }

    ui.horizontal(|ui| {
        ui.label("Text Speed:");
        ui.add(egui::TextEdit::singleline(&mut editor_state.dialog_text_speed).desired_width(60.0));
    });

    ui.horizontal(|ui| {
        ui.label("Position:");
        egui::ComboBox::from_id_salt(format!("{}_dialog_position_select", id_salt))
            .selected_text(match editor_state.dialog_position {
                DialogPositionData::Top => "Top",
                DialogPositionData::Center => "Center",
                DialogPositionData::Bottom => "Bottom",
            })
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut editor_state.dialog_position,
                    DialogPositionData::Top,
                    "Top",
                );
                ui.selectable_value(
                    &mut editor_state.dialog_position,
                    DialogPositionData::Center,
                    "Center",
                );
                ui.selectable_value(
                    &mut editor_state.dialog_position,
                    DialogPositionData::Bottom,
                    "Bottom",
                );
            });
    });

    ui.checkbox(&mut editor_state.dialog_movement_block, "Movement Block");

    // Face portrait selector
    ui.horizontal(|ui| {
        ui.label("Face Portrait:");
        let selected_text = match &editor_state.dialog_face_portrait {
            Some(id) => id.clone(),
            None => "None".to_string(),
        };
        egui::ComboBox::from_id_salt(format!("{}_face_portrait_select", id_salt))
            .selected_text(&selected_text)
            .show_ui(ui, |ui| {
                if ui
                    .selectable_label(editor_state.dialog_face_portrait.is_none(), "None")
                    .clicked()
                {
                    editor_state.dialog_face_portrait = None;
                }
                let mut portrait_ids: Vec<&String> = face_portraits.keys().collect();
                portrait_ids.sort();
                for portrait_id in portrait_ids {
                    let is_selected =
                        editor_state.dialog_face_portrait.as_ref() == Some(portrait_id);
                    if ui.selectable_label(is_selected, portrait_id).clicked() {
                        editor_state.dialog_face_portrait = Some(portrait_id.clone());
                    }
                }
            });
    });

    let show_dialog_button_label = if editor_state.editing_index.is_some() {
        "Update ShowDialog"
    } else {
        "Add ShowDialog"
    };
    if ui.button(show_dialog_button_label).clicked()
        && let Some(new_action) = editor_state.build_action()
    {
        if let Some(idx) = editor_state.editing_index {
            if idx < actions.len() {
                actions[idx] = new_action;
            }
            editor_state.editing_index = None;
        } else {
            actions.push(new_action);
        }
        // Reset ShowDialog fields
        editor_state.dialog_inline_text = String::new();
        editor_state.dialog_text_id = String::new();
        editor_state.dialog_text_speed = "30".to_string();
        editor_state.dialog_position = DialogPositionData::Bottom;
        editor_state.dialog_movement_block = true;
        editor_state.dialog_face_portrait = None;
    }
    if editor_state.editing_index.is_some() && ui.button("Cancel Edit").clicked() {
        editor_state.editing_index = None;
        editor_state.dialog_inline_text = String::new();
        editor_state.dialog_text_id = String::new();
        editor_state.dialog_text_speed = "30".to_string();
        editor_state.dialog_position = DialogPositionData::Bottom;
        editor_state.dialog_movement_block = true;
        editor_state.dialog_face_portrait = None;
    }
}

pub fn render_screen_shake_form(
    ui: &mut egui::Ui,
    actions: &mut Vec<EventAction>,
    editor_state: &mut ActionEditorState,
    _id_salt: &str,
) {
    ui.horizontal(|ui| {
        ui.label("Mode:");
        ui.radio_value(
            &mut editor_state.shake_mode,
            ScreenShakeMode::Timed,
            "Timed",
        );
        ui.radio_value(
            &mut editor_state.shake_mode,
            ScreenShakeMode::Continuous,
            "Continuous",
        );
    });

    ui.horizontal(|ui| {
        ui.label("Intensity:");
        ui.add(egui::TextEdit::singleline(&mut editor_state.shake_intensity).desired_width(60.0));
        ui.label("(0.0 – 50.0)");
    });

    if editor_state.shake_mode == ScreenShakeMode::Timed {
        ui.horizontal(|ui| {
            ui.label("Duration:");
            ui.add(
                egui::TextEdit::singleline(&mut editor_state.shake_duration).desired_width(60.0),
            );
            ui.label("(0.0 – 10.0)");
        });
    }

    let btn_label = if editor_state.editing_index.is_some() {
        "Update ScreenShake"
    } else {
        "Add ScreenShake"
    };
    if ui.button(btn_label).clicked()
        && let Some(new_action) = editor_state.build_action()
    {
        if let Some(idx) = editor_state.editing_index {
            if idx < actions.len() {
                actions[idx] = new_action;
            }
            editor_state.editing_index = None;
        } else {
            actions.push(new_action);
        }
        editor_state.shake_intensity = "5.0".to_string();
        editor_state.shake_duration = "0.5".to_string();
        editor_state.shake_mode = ScreenShakeMode::Timed;
    }
    if editor_state.editing_index.is_some() && ui.button("Cancel Edit").clicked() {
        editor_state.editing_index = None;
        editor_state.shake_intensity = "5.0".to_string();
        editor_state.shake_duration = "0.5".to_string();
        editor_state.shake_mode = ScreenShakeMode::Timed;
    }
}

pub fn render_stop_screen_shake_form(
    ui: &mut egui::Ui,
    actions: &mut Vec<EventAction>,
    editor_state: &mut ActionEditorState,
) {
    ui.label("No additional configuration needed.");

    let btn_label = if editor_state.editing_index.is_some() {
        "Update StopScreenShake"
    } else {
        "Add StopScreenShake"
    };
    if ui.button(btn_label).clicked() {
        let new_action = EventAction::StopScreenShake;
        if let Some(idx) = editor_state.editing_index {
            if idx < actions.len() {
                actions[idx] = new_action;
            }
            editor_state.editing_index = None;
        } else {
            actions.push(new_action);
        }
    }
    if editor_state.editing_index.is_some() && ui.button("Cancel Edit").clicked() {
        editor_state.editing_index = None;
    }
}

pub fn render_fade_transition_form(
    ui: &mut egui::Ui,
    actions: &mut Vec<EventAction>,
    editor_state: &mut ActionEditorState,
    _id_salt: &str,
) {
    ui.horizontal(|ui| {
        ui.label("Fade Type:");
        ui.radio_value(&mut editor_state.fade_type, FadeType::FadeIn, "FadeIn");
        ui.radio_value(&mut editor_state.fade_type, FadeType::FadeOut, "FadeOut");
    });

    ui.horizontal(|ui| {
        ui.label("Duration:");
        ui.add(egui::TextEdit::singleline(&mut editor_state.fade_duration).desired_width(60.0));
        ui.label("(0.0 – 10.0)");
    });

    ui.horizontal(|ui| {
        ui.label("Color (RGBA):");
        let mut color3 = [
            editor_state.fade_color[0],
            editor_state.fade_color[1],
            editor_state.fade_color[2],
        ];
        if ui.color_edit_button_rgb(&mut color3).changed() {
            editor_state.fade_color[0] = color3[0];
            editor_state.fade_color[1] = color3[1];
            editor_state.fade_color[2] = color3[2];
        }
    });

    let btn_label = if editor_state.editing_index.is_some() {
        "Update FadeTransition"
    } else {
        "Add FadeTransition"
    };
    if ui.button(btn_label).clicked()
        && let Some(new_action) = editor_state.build_action()
    {
        if let Some(idx) = editor_state.editing_index {
            if idx < actions.len() {
                actions[idx] = new_action;
            }
            editor_state.editing_index = None;
        } else {
            actions.push(new_action);
        }
        editor_state.fade_type = FadeType::FadeOut;
        editor_state.fade_duration = "1.0".to_string();
        editor_state.fade_color = [0.0, 0.0, 0.0, 1.0];
    }
    if editor_state.editing_index.is_some() && ui.button("Cancel Edit").clicked() {
        editor_state.editing_index = None;
        editor_state.fade_type = FadeType::FadeOut;
        editor_state.fade_duration = "1.0".to_string();
        editor_state.fade_color = [0.0, 0.0, 0.0, 1.0];
    }
}

pub fn render_set_state_form(
    ui: &mut egui::Ui,
    actions: &mut Vec<EventAction>,
    editor_state: &mut ActionEditorState,
) {
    ui.horizontal(|ui| {
        ui.label("Key:");
        ui.text_edit_singleline(&mut editor_state.state_key);
    });
    ui.horizontal(|ui| {
        ui.label("Value:");
        ui.text_edit_singleline(&mut editor_state.state_value);
    });

    let btn_label = if editor_state.editing_index.is_some() {
        "Update SetState"
    } else {
        "Add SetState"
    };
    if ui.button(btn_label).clicked()
        && let Some(new_action) = editor_state.build_action()
    {
        if let Some(idx) = editor_state.editing_index {
            if idx < actions.len() {
                actions[idx] = new_action;
            }
            editor_state.editing_index = None;
        } else {
            actions.push(new_action);
        }
        editor_state.state_key = String::new();
        editor_state.state_value = String::new();
    }
    if editor_state.editing_index.is_some() && ui.button("Cancel Edit").clicked() {
        editor_state.editing_index = None;
        editor_state.state_key = String::new();
        editor_state.state_value = String::new();
    }
}

pub fn render_set_player_appearance_form(
    ui: &mut egui::Ui,
    actions: &mut Vec<EventAction>,
    editor_state: &mut ActionEditorState,
    id_salt: &str,
) {
    ui.horizontal(|ui| {
        ui.label("Appearance:");
        let appearance_text = match &editor_state.appearance {
            PlayerAppearance::Hidden => "Hidden",
            PlayerAppearance::Spritesheet { .. } => "Spritesheet",
            PlayerAppearance::Default => "Default",
        };
        egui::ComboBox::from_id_salt(format!("{}_appearance_select", id_salt))
            .selected_text(appearance_text)
            .show_ui(ui, |ui| {
                if ui
                    .selectable_label(
                        matches!(editor_state.appearance, PlayerAppearance::Hidden),
                        "Hidden",
                    )
                    .clicked()
                {
                    editor_state.appearance = PlayerAppearance::Hidden;
                }
                if ui
                    .selectable_label(
                        matches!(
                            editor_state.appearance,
                            PlayerAppearance::Spritesheet { .. }
                        ),
                        "Spritesheet",
                    )
                    .clicked()
                {
                    editor_state.appearance = PlayerAppearance::Spritesheet {
                        path: editor_state.appearance_path.clone(),
                    };
                }
                if ui
                    .selectable_label(
                        matches!(editor_state.appearance, PlayerAppearance::Default),
                        "Default",
                    )
                    .clicked()
                {
                    editor_state.appearance = PlayerAppearance::Default;
                }
            });
    });

    if matches!(
        editor_state.appearance,
        PlayerAppearance::Spritesheet { .. }
    ) {
        ui.horizontal(|ui| {
            ui.label("Path:");
            ui.text_edit_singleline(&mut editor_state.appearance_path);
        });
    }

    let btn_label = if editor_state.editing_index.is_some() {
        "Update SetPlayerAppearance"
    } else {
        "Add SetPlayerAppearance"
    };
    if ui.button(btn_label).clicked()
        && let Some(new_action) = editor_state.build_action()
    {
        if let Some(idx) = editor_state.editing_index {
            if idx < actions.len() {
                actions[idx] = new_action;
            }
            editor_state.editing_index = None;
        } else {
            actions.push(new_action);
        }
        editor_state.appearance = PlayerAppearance::Hidden;
        editor_state.appearance_path = String::new();
    }
    if editor_state.editing_index.is_some() && ui.button("Cancel Edit").clicked() {
        editor_state.editing_index = None;
        editor_state.appearance = PlayerAppearance::Hidden;
        editor_state.appearance_path = String::new();
    }
}

pub fn render_state_check_form(
    ui: &mut egui::Ui,
    actions: &mut Vec<EventAction>,
    editor_state: &mut ActionEditorState,
    _id_salt: &str,
) {
    ui.horizontal(|ui| {
        ui.label("Key:");
        ui.text_edit_singleline(&mut editor_state.state_check_key);
    });
    ui.horizontal(|ui| {
        ui.label("Value (leave empty = key existence):");
        ui.text_edit_singleline(&mut editor_state.state_check_value);
    });
    ui.label("Actions on true/false are managed in the action list above.");

    let btn_label = if editor_state.editing_index.is_some() {
        "Update StateCheck"
    } else {
        "Add StateCheck"
    };
    if ui.button(btn_label).clicked()
        && let Some(new_action) = editor_state.build_action()
    {
        if let Some(idx) = editor_state.editing_index {
            if idx < actions.len() {
                actions[idx] = new_action;
            }
            editor_state.editing_index = None;
        } else {
            actions.push(new_action);
        }
        editor_state.state_check_key = String::new();
        editor_state.state_check_value = String::new();
    }
    if editor_state.editing_index.is_some() && ui.button("Cancel Edit").clicked() {
        editor_state.editing_index = None;
        editor_state.state_check_key = String::new();
        editor_state.state_check_value = String::new();
    }
}

pub fn render_branch_form(
    ui: &mut egui::Ui,
    actions: &mut Vec<EventAction>,
    editor_state: &mut ActionEditorState,
    id_salt: &str,
) {
    let form_label = if editor_state.editing_index.is_some() {
        "Edit Branch Action:"
    } else {
        "Add Branch Action:"
    };
    ui.label(form_label);

    // Logic selector (All / Any)
    ui.horizontal(|ui| {
        ui.label("Logic:");
        egui::ComboBox::from_id_salt(format!("{}_branch_logic", id_salt))
            .selected_text(match editor_state.branch_logic {
                ConditionLogic::All => "All (AND)",
                ConditionLogic::Any => "Any (OR)",
            })
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut editor_state.branch_logic,
                    ConditionLogic::All,
                    "All (AND)",
                );
                ui.selectable_value(
                    &mut editor_state.branch_logic,
                    ConditionLogic::Any,
                    "Any (OR)",
                );
            });
    });

    // Condition checks list
    ui.label("Conditions:");

    let mut remove_check_idx: Option<usize> = None;
    for (i, check) in editor_state.branch_checks.iter_mut().enumerate() {
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
            egui::ComboBox::from_id_salt(format!("{}_check_op_{}", id_salt, i))
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

            // Value field — disabled for Exists/NotExists
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
                // Clear value when operator doesn't use it
                check.value = None;
            }

            if ui.small_button("✕").clicked() {
                remove_check_idx = Some(i);
            }
        });
    }

    if let Some(idx) = remove_check_idx {
        editor_state.branch_checks.remove(idx);
    }

    if ui.button("Add Condition").clicked() {
        editor_state.branch_checks.push(ConditionCheck {
            key: String::new(),
            operator: ConditionOperator::Equals,
            value: None,
        });
    }

    // Validation hints
    let has_checks = !editor_state.branch_checks.is_empty();
    let all_keys_filled = editor_state.branch_checks.iter().all(|c| !c.key.is_empty());
    if !has_checks {
        ui.label(
            egui::RichText::new("Add at least one condition.")
                .color(egui::Color32::from_rgb(200, 150, 50)),
        );
    } else if !all_keys_filled {
        ui.label(
            egui::RichText::new("All condition keys must be non-empty.")
                .color(egui::Color32::from_rgb(200, 150, 50)),
        );
    }

    let can_save = has_checks && all_keys_filled;

    let btn_label = if editor_state.editing_index.is_some() {
        "Update Branch"
    } else {
        "Add Branch"
    };
    if ui
        .add_enabled(can_save, egui::Button::new(btn_label))
        .clicked()
        && let Some(new_action) = editor_state.build_action()
    {
        if let Some(idx) = editor_state.editing_index {
            if idx < actions.len() {
                actions[idx] = new_action;
            }
            editor_state.editing_index = None;
        } else {
            actions.push(new_action);
        }
        editor_state.branch_logic = ConditionLogic::All;
        editor_state.branch_checks.clear();
        editor_state.branch_on_true.clear();
        editor_state.branch_on_false.clear();
    }
    if editor_state.editing_index.is_some() && ui.button("Cancel Edit").clicked() {
        editor_state.editing_index = None;
        editor_state.branch_logic = ConditionLogic::All;
        editor_state.branch_checks.clear();
        editor_state.branch_on_true.clear();
        editor_state.branch_on_false.clear();
    }
}

pub fn render_show_selection_form(
    ui: &mut egui::Ui,
    actions: &mut Vec<EventAction>,
    editor_state: &mut ActionEditorState,
    id_salt: &str,
    map_entries: &[(String, String)],
    face_portraits: &std::collections::HashMap<String, String>,
    shops: &[(String, String)],
) {
    let form_label = if editor_state.editing_index.is_some() {
        "Edit ShowSelection Action:"
    } else {
        "Add ShowSelection Action:"
    };
    ui.label(form_label);

    // --- Prompt text section ---
    ui.horizontal(|ui| {
        ui.label("Prompt Source:");
        ui.radio_value(
            &mut editor_state.selection_prompt_mode,
            DialogTextMode::Inline,
            "Inline",
        );
        ui.radio_value(
            &mut editor_state.selection_prompt_mode,
            DialogTextMode::TextId,
            "Text ID",
        );
    });

    match editor_state.selection_prompt_mode {
        DialogTextMode::Inline => {
            ui.label("Prompt Text:");
            ui.text_edit_multiline(&mut editor_state.selection_prompt_text);
        }
        DialogTextMode::TextId => {
            ui.horizontal(|ui| {
                ui.label("Prompt Text ID:");
                ui.text_edit_singleline(&mut editor_state.selection_prompt_id);
            });
        }
    }

    // --- Position combo box ---
    ui.horizontal(|ui| {
        ui.label("Position:");
        egui::ComboBox::from_id_salt(format!("{}_selection_position_select", id_salt))
            .selected_text(match editor_state.selection_position {
                DialogPositionData::Top => "Top",
                DialogPositionData::Center => "Center",
                DialogPositionData::Bottom => "Bottom",
            })
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut editor_state.selection_position,
                    DialogPositionData::Top,
                    "Top",
                );
                ui.selectable_value(
                    &mut editor_state.selection_position,
                    DialogPositionData::Center,
                    "Center",
                );
                ui.selectable_value(
                    &mut editor_state.selection_position,
                    DialogPositionData::Bottom,
                    "Bottom",
                );
            });
    });

    // --- Face portrait selector ---
    ui.horizontal(|ui| {
        ui.label("Face Portrait:");
        let selected_text = match &editor_state.selection_face_portrait {
            Some(id) => id.clone(),
            None => "None".to_string(),
        };
        egui::ComboBox::from_id_salt(format!("{}_selection_face_portrait_select", id_salt))
            .selected_text(&selected_text)
            .show_ui(ui, |ui| {
                if ui
                    .selectable_label(editor_state.selection_face_portrait.is_none(), "None")
                    .clicked()
                {
                    editor_state.selection_face_portrait = None;
                }
                let mut portrait_ids: Vec<&String> = face_portraits.keys().collect();
                portrait_ids.sort();
                for portrait_id in portrait_ids {
                    let is_selected =
                        editor_state.selection_face_portrait.as_ref() == Some(portrait_id);
                    if ui.selectable_label(is_selected, portrait_id).clicked() {
                        editor_state.selection_face_portrait = Some(portrait_id.clone());
                    }
                }
            });
    });

    ui.separator();

    // --- Choice list ---
    ui.label(egui::RichText::new("Choices:").strong());

    // Validation: at least 2 choices
    let choice_count = editor_state.selection_choices.len();
    if choice_count < 2 {
        ui.label(
            egui::RichText::new("Add at least 2 choices")
                .color(egui::Color32::from_rgb(220, 50, 50)),
        );
    }

    let mut remove_choice_idx: Option<usize> = None;

    for i in 0..editor_state.selection_choices.len() {
        let choice_salt = format!("{}_choice_{}", id_salt, i);

        egui::CollapsingHeader::new(format!("Choice {}", i + 1))
            .id_salt(&choice_salt)
            .default_open(true)
            .show(ui, |ui| {
                // Label mode toggle
                ui.horizontal(|ui| {
                    ui.label("Label Source:");
                    ui.radio_value(
                        &mut editor_state.selection_choices[i].label_mode,
                        DialogTextMode::Inline,
                        "Inline",
                    );
                    ui.radio_value(
                        &mut editor_state.selection_choices[i].label_mode,
                        DialogTextMode::TextId,
                        "Text ID",
                    );
                });

                match editor_state.selection_choices[i].label_mode {
                    DialogTextMode::Inline => {
                        ui.horizontal(|ui| {
                            ui.label("Label:");
                            ui.text_edit_singleline(
                                &mut editor_state.selection_choices[i].label_text,
                            );
                        });
                        // Inline validation for empty label
                        if editor_state.selection_choices[i].label_text.is_empty() {
                            ui.label(
                                egui::RichText::new("Label must not be empty")
                                    .color(egui::Color32::from_rgb(220, 50, 50)),
                            );
                        }
                    }
                    DialogTextMode::TextId => {
                        ui.horizontal(|ui| {
                            ui.label("Label Text ID:");
                            ui.text_edit_singleline(
                                &mut editor_state.selection_choices[i].label_id,
                            );
                        });
                    }
                }

                // Nested action editor for this choice (uses persistent editor state)
                let nested_salt = format!("{}_actions", choice_salt);
                ui.indent(format!("choice_actions_indent_{}", i), |ui| {
                    ui.label(egui::RichText::new(format!("Choice {} Actions:", i + 1)).italics());
                    let choice = &mut editor_state.selection_choices[i];
                    super::action_editor_ui::render_action_editor(
                        ui,
                        &mut choice.actions,
                        &mut choice.action_editor,
                        &nested_salt,
                        map_entries,
                        face_portraits,
                        1,
                        None,
                        shops,
                    );
                });

                // Remove choice button (disabled when only 2 remain)
                let can_remove = editor_state.selection_choices.len() > 2;
                if ui
                    .add_enabled(can_remove, egui::Button::new("Remove Choice"))
                    .clicked()
                {
                    remove_choice_idx = Some(i);
                }
            });
    }

    if let Some(idx) = remove_choice_idx {
        editor_state.selection_choices.remove(idx);
    }

    // Add Choice button (disabled at 6)
    let can_add = editor_state.selection_choices.len() < 6;
    if ui
        .add_enabled(can_add, egui::Button::new("Add Choice"))
        .clicked()
    {
        editor_state.selection_choices.push(EditorChoice::default());
    }

    ui.separator();

    // --- Add/Update and Cancel buttons ---
    let has_valid_choices = editor_state.selection_choices.len() >= 2
        && editor_state
            .selection_choices
            .iter()
            .all(|c| match c.label_mode {
                DialogTextMode::Inline => !c.label_text.is_empty(),
                DialogTextMode::TextId => !c.label_id.is_empty(),
            });
    let has_valid_prompt = match editor_state.selection_prompt_mode {
        DialogTextMode::Inline => !editor_state.selection_prompt_text.is_empty(),
        DialogTextMode::TextId => !editor_state.selection_prompt_id.is_empty(),
    };
    let can_save = has_valid_choices && has_valid_prompt;

    let btn_label = if editor_state.editing_index.is_some() {
        "Update ShowSelection"
    } else {
        "Add ShowSelection"
    };
    if ui
        .add_enabled(can_save, egui::Button::new(btn_label))
        .clicked()
        && let Some(new_action) = editor_state.build_action()
    {
        if let Some(idx) = editor_state.editing_index {
            if idx < actions.len() {
                actions[idx] = new_action;
            }
            editor_state.editing_index = None;
        } else {
            actions.push(new_action);
        }
        // Reset ShowSelection fields
        editor_state.selection_prompt_mode = DialogTextMode::Inline;
        editor_state.selection_prompt_text = String::new();
        editor_state.selection_prompt_id = String::new();
        editor_state.selection_position = DialogPositionData::Bottom;
        editor_state.selection_face_portrait = None;
        editor_state.selection_choices = vec![EditorChoice::default(), EditorChoice::default()];
    }
    if editor_state.editing_index.is_some() && ui.button("Cancel Edit").clicked() {
        editor_state.editing_index = None;
        editor_state.selection_prompt_mode = DialogTextMode::Inline;
        editor_state.selection_prompt_text = String::new();
        editor_state.selection_prompt_id = String::new();
        editor_state.selection_position = DialogPositionData::Bottom;
        editor_state.selection_face_portrait = None;
        editor_state.selection_choices = vec![EditorChoice::default(), EditorChoice::default()];
    }
}

/// Renders the TransferDirection toggle common to all reward action forms.
fn render_transfer_direction_toggle(
    ui: &mut egui::Ui,
    editor_state: &mut ActionEditorState,
    id_salt: &str,
) {
    ui.horizontal(|ui| {
        ui.label("Direction:");
        egui::ComboBox::from_id_salt(format!("{}_reward_direction", id_salt))
            .selected_text(match editor_state.reward_direction {
                TransferDirection::Give => "Give",
                TransferDirection::Take => "Take",
            })
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut editor_state.reward_direction,
                    TransferDirection::Give,
                    "Give",
                );
                ui.selectable_value(
                    &mut editor_state.reward_direction,
                    TransferDirection::Take,
                    "Take",
                );
            });
    });
}

pub fn render_give_currency_form(
    ui: &mut egui::Ui,
    actions: &mut Vec<EventAction>,
    editor_state: &mut ActionEditorState,
    id_salt: &str,
) {
    let form_label = if editor_state.editing_index.is_some() {
        "Edit GiveCurrency Action:"
    } else {
        "Add GiveCurrency Action:"
    };
    ui.label(form_label);

    // TransferDirection toggle
    render_transfer_direction_toggle(ui, editor_state, id_salt);

    // Amount input
    ui.horizontal(|ui| {
        ui.label("Amount:");
        ui.add(egui::TextEdit::singleline(&mut editor_state.currency_amount).desired_width(100.0));
        ui.label("(1 – 9,999,999)");
    });

    // Validation
    let amount_valid = editor_state
        .currency_amount
        .trim()
        .parse::<u64>()
        .is_ok_and(|a| (1..=9_999_999).contains(&a));
    let take_failure_valid = editor_state.reward_direction != TransferDirection::Take
        || !editor_state.reward_on_failure.is_empty();
    let is_valid = amount_valid && take_failure_valid;

    if !is_valid {
        if !amount_valid {
            ui.colored_label(
                egui::Color32::from_rgb(220, 50, 50),
                "Amount must be between 1 and 9,999,999",
            );
        }
        if !take_failure_valid {
            ui.colored_label(
                egui::Color32::from_rgb(220, 50, 50),
                "At least one on_failure action is required for Take direction",
            );
        }
    }

    // Add/Update button
    let btn_label = if editor_state.editing_index.is_some() {
        "Update GiveCurrency"
    } else {
        "Add GiveCurrency"
    };
    if ui
        .add_enabled(is_valid, egui::Button::new(btn_label))
        .clicked()
        && let Some(new_action) = editor_state.build_action()
    {
        if let Some(idx) = editor_state.editing_index {
            if idx < actions.len() {
                actions[idx] = new_action;
            }
            editor_state.editing_index = None;
        } else {
            actions.push(new_action);
        }
        editor_state.currency_amount = "100".to_string();
        editor_state.reward_direction = TransferDirection::Give;
        editor_state.reward_on_success.clear();
        editor_state.reward_on_failure.clear();
    }
    if editor_state.editing_index.is_some() && ui.button("Cancel Edit").clicked() {
        editor_state.editing_index = None;
        editor_state.currency_amount = "100".to_string();
        editor_state.reward_direction = TransferDirection::Give;
        editor_state.reward_on_success.clear();
        editor_state.reward_on_failure.clear();
    }
}

pub fn render_give_experience_form(
    ui: &mut egui::Ui,
    actions: &mut Vec<EventAction>,
    editor_state: &mut ActionEditorState,
    id_salt: &str,
    characters: &[(String, String)],
) {
    let form_label = if editor_state.editing_index.is_some() {
        "Edit GiveExperience Action:"
    } else {
        "Add GiveExperience Action:"
    };
    ui.label(form_label);

    // TransferDirection toggle
    render_transfer_direction_toggle(ui, editor_state, id_salt);

    // Amount input
    ui.horizontal(|ui| {
        ui.label("Amount:");
        ui.add(
            egui::TextEdit::singleline(&mut editor_state.experience_amount).desired_width(100.0),
        );
        ui.label("(1 – 9,999,999)");
    });

    // Target character selector
    ui.horizontal(|ui| {
        ui.label("Target:");
        let selected_text = match &editor_state.experience_target {
            Some(id) if !id.is_empty() => characters
                .iter()
                .find(|(cid, _)| cid == id)
                .map(|(_, name)| name.clone())
                .unwrap_or_else(|| id.clone()),
            _ => "All Party Members".to_string(),
        };
        egui::ComboBox::from_id_salt(format!("{}_exp_target", id_salt))
            .selected_text(selected_text)
            .show_ui(ui, |ui| {
                if ui
                    .selectable_label(
                        editor_state.experience_target.is_none(),
                        "All Party Members",
                    )
                    .clicked()
                {
                    editor_state.experience_target = None;
                }
                for (id, name) in characters {
                    let is_selected = editor_state.experience_target.as_ref() == Some(id);
                    if ui.selectable_label(is_selected, name).clicked() {
                        editor_state.experience_target = Some(id.clone());
                    }
                }
            });
    });

    // Validation
    let amount_valid = editor_state
        .experience_amount
        .trim()
        .parse::<u64>()
        .is_ok_and(|a| (1..=9_999_999).contains(&a));
    let take_failure_valid = editor_state.reward_direction != TransferDirection::Take
        || !editor_state.reward_on_failure.is_empty();
    let is_valid = amount_valid && take_failure_valid;

    if !is_valid {
        if !amount_valid {
            ui.colored_label(
                egui::Color32::from_rgb(220, 50, 50),
                "Amount must be between 1 and 9,999,999",
            );
        }
        if !take_failure_valid {
            ui.colored_label(
                egui::Color32::from_rgb(220, 50, 50),
                "At least one on_failure action is required for Take direction",
            );
        }
    }

    // Add/Update button
    let btn_label = if editor_state.editing_index.is_some() {
        "Update GiveExperience"
    } else {
        "Add GiveExperience"
    };
    if ui
        .add_enabled(is_valid, egui::Button::new(btn_label))
        .clicked()
        && let Some(new_action) = editor_state.build_action()
    {
        if let Some(idx) = editor_state.editing_index {
            if idx < actions.len() {
                actions[idx] = new_action;
            }
            editor_state.editing_index = None;
        } else {
            actions.push(new_action);
        }
        editor_state.experience_amount = "100".to_string();
        editor_state.experience_target = None;
        editor_state.reward_direction = TransferDirection::Give;
        editor_state.reward_on_success.clear();
        editor_state.reward_on_failure.clear();
    }
    if editor_state.editing_index.is_some() && ui.button("Cancel Edit").clicked() {
        editor_state.editing_index = None;
        editor_state.experience_amount = "100".to_string();
        editor_state.experience_target = None;
        editor_state.reward_direction = TransferDirection::Give;
        editor_state.reward_on_success.clear();
        editor_state.reward_on_failure.clear();
    }
}

pub fn render_give_item_form(
    ui: &mut egui::Ui,
    actions: &mut Vec<EventAction>,
    editor_state: &mut ActionEditorState,
    id_salt: &str,
    items: &[(String, String)],
    item_search_buffer: &mut String,
) {
    let form_label = if editor_state.editing_index.is_some() {
        "Edit GiveItem Action:"
    } else {
        "Add GiveItem Action:"
    };
    ui.label(form_label);

    // TransferDirection toggle
    render_transfer_direction_toggle(ui, editor_state, id_salt);

    // Item selector (searchable)
    ui.horizontal(|ui| {
        ui.label("Item:");
        let current_label = if editor_state.give_item_id.is_empty() {
            "Select item...".to_string()
        } else {
            items
                .iter()
                .find(|(id, _)| *id == editor_state.give_item_id)
                .map(|(_, name)| name.clone())
                .unwrap_or_else(|| editor_state.give_item_id.clone())
        };
        if let Some(selected_id) = searchable_combobox(
            ui,
            &format!("{}_item_select", id_salt),
            &current_label,
            items,
            item_search_buffer,
        ) {
            editor_state.give_item_id = selected_id;
        }
    });

    // Quantity input
    ui.horizontal(|ui| {
        ui.label("Quantity:");
        ui.add(
            egui::TextEdit::singleline(&mut editor_state.give_item_quantity).desired_width(60.0),
        );
        ui.label("(1 – 999)");
    });

    // Validation
    let item_valid = !editor_state.give_item_id.is_empty();
    let quantity_valid = editor_state
        .give_item_quantity
        .trim()
        .parse::<u32>()
        .is_ok_and(|q| (1..=999).contains(&q));
    let take_failure_valid = editor_state.reward_direction != TransferDirection::Take
        || !editor_state.reward_on_failure.is_empty();
    let is_valid = item_valid && quantity_valid && take_failure_valid;

    if !is_valid {
        if !item_valid {
            ui.colored_label(
                egui::Color32::from_rgb(220, 50, 50),
                "An item must be selected",
            );
        }
        if !quantity_valid {
            ui.colored_label(
                egui::Color32::from_rgb(220, 50, 50),
                "Quantity must be between 1 and 999",
            );
        }
        if !take_failure_valid {
            ui.colored_label(
                egui::Color32::from_rgb(220, 50, 50),
                "At least one on_failure action is required for Take direction",
            );
        }
    }

    // Add/Update button
    let btn_label = if editor_state.editing_index.is_some() {
        "Update GiveItem"
    } else {
        "Add GiveItem"
    };
    if ui
        .add_enabled(is_valid, egui::Button::new(btn_label))
        .clicked()
        && let Some(new_action) = editor_state.build_action()
    {
        if let Some(idx) = editor_state.editing_index {
            if idx < actions.len() {
                actions[idx] = new_action;
            }
            editor_state.editing_index = None;
        } else {
            actions.push(new_action);
        }
        editor_state.give_item_id = String::new();
        editor_state.give_item_quantity = "1".to_string();
        editor_state.reward_direction = TransferDirection::Give;
        editor_state.reward_on_success.clear();
        editor_state.reward_on_failure.clear();
    }
    if editor_state.editing_index.is_some() && ui.button("Cancel Edit").clicked() {
        editor_state.editing_index = None;
        editor_state.give_item_id = String::new();
        editor_state.give_item_quantity = "1".to_string();
        editor_state.reward_direction = TransferDirection::Give;
        editor_state.reward_on_success.clear();
        editor_state.reward_on_failure.clear();
    }
}

#[allow(clippy::too_many_arguments)]
pub fn render_learn_ability_form(
    ui: &mut egui::Ui,
    actions: &mut Vec<EventAction>,
    editor_state: &mut ActionEditorState,
    id_salt: &str,
    abilities: &[(String, String)],
    characters: &[(String, String)],
    ability_search_buffer: &mut String,
    character_search_buffer: &mut String,
) {
    let form_label = if editor_state.editing_index.is_some() {
        "Edit LearnAbility Action:"
    } else {
        "Add LearnAbility Action:"
    };
    ui.label(form_label);

    // TransferDirection toggle
    render_transfer_direction_toggle(ui, editor_state, id_salt);

    // Ability selector (searchable)
    ui.horizontal(|ui| {
        ui.label("Ability:");
        let current_label = if editor_state.learn_ability_id.is_empty() {
            "Select ability...".to_string()
        } else {
            abilities
                .iter()
                .find(|(id, _)| *id == editor_state.learn_ability_id)
                .map(|(_, name)| name.clone())
                .unwrap_or_else(|| editor_state.learn_ability_id.clone())
        };
        if let Some(selected_id) = searchable_combobox(
            ui,
            &format!("{}_ability_select", id_salt),
            &current_label,
            abilities,
            ability_search_buffer,
        ) {
            editor_state.learn_ability_id = selected_id;
        }
    });

    // Character target selector (searchable)
    ui.horizontal(|ui| {
        ui.label("Target:");
        let current_label = if editor_state.learn_ability_target.is_empty() {
            "Select character...".to_string()
        } else {
            characters
                .iter()
                .find(|(id, _)| *id == editor_state.learn_ability_target)
                .map(|(_, name)| name.clone())
                .unwrap_or_else(|| editor_state.learn_ability_target.clone())
        };
        if let Some(selected_id) = searchable_combobox(
            ui,
            &format!("{}_learn_char_select", id_salt),
            &current_label,
            characters,
            character_search_buffer,
        ) {
            editor_state.learn_ability_target = selected_id;
        }
    });

    // Validation
    let ability_valid = !editor_state.learn_ability_id.is_empty();
    let target_valid = !editor_state.learn_ability_target.is_empty();
    let take_failure_valid = editor_state.reward_direction != TransferDirection::Take
        || !editor_state.reward_on_failure.is_empty();
    let is_valid = ability_valid && target_valid && take_failure_valid;

    if !is_valid {
        if !ability_valid {
            ui.colored_label(
                egui::Color32::from_rgb(220, 50, 50),
                "An ability must be selected",
            );
        }
        if !target_valid {
            ui.colored_label(
                egui::Color32::from_rgb(220, 50, 50),
                "A target character must be selected",
            );
        }
        if !take_failure_valid {
            ui.colored_label(
                egui::Color32::from_rgb(220, 50, 50),
                "At least one on_failure action is required for Take direction",
            );
        }
    }

    // Add/Update button
    let btn_label = if editor_state.editing_index.is_some() {
        "Update LearnAbility"
    } else {
        "Add LearnAbility"
    };
    if ui
        .add_enabled(is_valid, egui::Button::new(btn_label))
        .clicked()
        && let Some(new_action) = editor_state.build_action()
    {
        if let Some(idx) = editor_state.editing_index {
            if idx < actions.len() {
                actions[idx] = new_action;
            }
            editor_state.editing_index = None;
        } else {
            actions.push(new_action);
        }
        editor_state.learn_ability_id = String::new();
        editor_state.learn_ability_target = String::new();
        editor_state.reward_direction = TransferDirection::Give;
        editor_state.reward_on_success.clear();
        editor_state.reward_on_failure.clear();
    }
    if editor_state.editing_index.is_some() && ui.button("Cancel Edit").clicked() {
        editor_state.editing_index = None;
        editor_state.learn_ability_id = String::new();
        editor_state.learn_ability_target = String::new();
        editor_state.reward_direction = TransferDirection::Give;
        editor_state.reward_on_success.clear();
        editor_state.reward_on_failure.clear();
    }
}

pub fn render_add_party_member_form(
    ui: &mut egui::Ui,
    actions: &mut Vec<EventAction>,
    editor_state: &mut ActionEditorState,
    id_salt: &str,
    characters: &[(String, String)],
    character_search_buffer: &mut String,
) {
    let form_label = if editor_state.editing_index.is_some() {
        "Edit AddPartyMember Action:"
    } else {
        "Add AddPartyMember Action:"
    };
    ui.label(form_label);

    // TransferDirection toggle
    render_transfer_direction_toggle(ui, editor_state, id_salt);

    // Character selector (searchable)
    ui.horizontal(|ui| {
        ui.label("Character:");
        let current_label = if editor_state.add_party_character_id.is_empty() {
            "Select character...".to_string()
        } else {
            characters
                .iter()
                .find(|(id, _)| *id == editor_state.add_party_character_id)
                .map(|(_, name)| name.clone())
                .unwrap_or_else(|| editor_state.add_party_character_id.clone())
        };
        if let Some(selected_id) = searchable_combobox(
            ui,
            &format!("{}_party_char_select", id_salt),
            &current_label,
            characters,
            character_search_buffer,
        ) {
            editor_state.add_party_character_id = selected_id;
        }
    });

    // Validation
    let character_valid = !editor_state.add_party_character_id.is_empty()
        && editor_state.add_party_character_id.len() <= 64;
    let take_failure_valid = editor_state.reward_direction != TransferDirection::Take
        || !editor_state.reward_on_failure.is_empty();
    let is_valid = character_valid && take_failure_valid;

    if !is_valid {
        if !character_valid {
            ui.colored_label(
                egui::Color32::from_rgb(220, 50, 50),
                "A character must be selected (1–64 characters)",
            );
        }
        if !take_failure_valid {
            ui.colored_label(
                egui::Color32::from_rgb(220, 50, 50),
                "At least one on_failure action is required for Take direction",
            );
        }
    }

    // Add/Update button
    let btn_label = if editor_state.editing_index.is_some() {
        "Update AddPartyMember"
    } else {
        "Add AddPartyMember"
    };
    if ui
        .add_enabled(is_valid, egui::Button::new(btn_label))
        .clicked()
        && let Some(new_action) = editor_state.build_action()
    {
        if let Some(idx) = editor_state.editing_index {
            if idx < actions.len() {
                actions[idx] = new_action;
            }
            editor_state.editing_index = None;
        } else {
            actions.push(new_action);
        }
        editor_state.add_party_character_id = String::new();
        editor_state.reward_direction = TransferDirection::Give;
        editor_state.reward_on_success.clear();
        editor_state.reward_on_failure.clear();
    }
    if editor_state.editing_index.is_some() && ui.button("Cancel Edit").clicked() {
        editor_state.editing_index = None;
        editor_state.add_party_character_id = String::new();
        editor_state.reward_direction = TransferDirection::Give;
        editor_state.reward_on_success.clear();
        editor_state.reward_on_failure.clear();
    }
}

pub fn render_save_game_form(
    ui: &mut egui::Ui,
    actions: &mut Vec<EventAction>,
    editor_state: &mut ActionEditorState,
) {
    ui.label("Save Game requires no additional configuration.");

    let is_editing = editor_state.editing_index.is_some();
    let btn_label = if is_editing {
        "Update Action"
    } else {
        "Add Action"
    };

    if ui.button(btn_label).clicked()
        && let Some(action) = editor_state.build_action()
    {
        if let Some(idx) = editor_state.editing_index {
            actions[idx] = action;
        } else {
            actions.push(action);
        }
        editor_state.reset();
    }
    if is_editing && ui.button("Cancel Edit").clicked() {
        editor_state.editing_index = None;
    }
}

pub fn render_change_phase_form(
    ui: &mut egui::Ui,
    actions: &mut Vec<EventAction>,
    editor_state: &mut ActionEditorState,
    id_salt: &str,
) {
    ui.horizontal(|ui| {
        ui.label("Target Phase:");
        let phase_text = format!("{:?}", editor_state.change_phase_target);
        egui::ComboBox::from_id_salt(format!("{}_change_phase_target", id_salt))
            .selected_text(&phase_text)
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut editor_state.change_phase_target,
                    AppPhase::TitleScreen,
                    "TitleScreen",
                );
                ui.selectable_value(
                    &mut editor_state.change_phase_target,
                    AppPhase::InGame,
                    "InGame",
                );
                ui.selectable_value(
                    &mut editor_state.change_phase_target,
                    AppPhase::Battle,
                    "Battle",
                );
                ui.selectable_value(
                    &mut editor_state.change_phase_target,
                    AppPhase::Shop,
                    "Shop",
                );
                ui.selectable_value(
                    &mut editor_state.change_phase_target,
                    AppPhase::Status,
                    "Status",
                );
            });
    });

    let is_editing = editor_state.editing_index.is_some();
    let btn_label = if is_editing {
        "Update Action"
    } else {
        "Add Action"
    };

    if ui.button(btn_label).clicked()
        && let Some(action) = editor_state.build_action()
    {
        if let Some(idx) = editor_state.editing_index {
            actions[idx] = action;
        } else {
            actions.push(action);
        }
        editor_state.reset();
    }
    if is_editing && ui.button("Cancel Edit").clicked() {
        editor_state.editing_index = None;
    }
}
