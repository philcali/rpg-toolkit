//! Per-action-type form renderers for the action editor UI.
//! Each function renders the form fields and add/update/cancel buttons
//! for a specific EventAction variant.

use bevy_egui::egui;

use rpg_toolkit_common::{DialogPositionData, FadeType, PlayerAppearance, ScreenShakeMode};

use crate::data::map::EventAction;

use super::action_editor::{ActionEditorState, DialogTextMode};

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
