//! Shared action editor state and types used by both the
//! Event Trigger Dialog and the NPC Placement Dialog.
//! UI rendering lives in the sibling `action_editor_ui` module.

use crate::data::map::EventAction;
use rpg_toolkit_common::{
    DialogConfigData, DialogPositionData, DialogTextData, FadeType, PlayerAppearance,
    ScreenShakeMode,
};

/// The type of action being added in the Event Trigger Editor.
#[derive(Default, PartialEq)]
pub enum ActionType {
    #[default]
    JumpTo,
    ShowDialog,
    ScreenShake,
    StopScreenShake,
    FadeTransition,
    SetState,
    SetPlayerAppearance,
}

/// The text source mode for a ShowDialog action.
#[derive(Default, PartialEq)]
pub enum DialogTextMode {
    #[default]
    Inline,
    TextId,
}

/// Truncates a string to at most `max_len` characters, appending "…" if truncated.
pub fn truncate_preview(s: &str, max_len: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() > max_len {
        let truncated: String = chars[..max_len].iter().collect();
        format!("{}…", truncated)
    } else {
        s.to_string()
    }
}

/// Consolidated state for editing a single EventAction.
/// Replaces the duplicated field sets in EventTriggerDialog and NpcPlacementDialog.
pub struct ActionEditorState {
    pub action_type: ActionType,
    pub editing_index: Option<usize>,
    // JumpTo fields
    pub target_map_id: String,
    pub target_x: String,
    pub target_y: String,
    // ShowDialog fields
    pub dialog_text_mode: DialogTextMode,
    pub dialog_inline_text: String,
    pub dialog_text_id: String,
    pub dialog_text_speed: String,
    pub dialog_position: DialogPositionData,
    pub dialog_movement_block: bool,
    // ScreenShake fields
    pub shake_mode: ScreenShakeMode,
    pub shake_intensity: String,
    pub shake_duration: String,
    // FadeTransition fields
    pub fade_type: FadeType,
    pub fade_duration: String,
    pub fade_color: [f32; 4],
    // SetState fields
    pub state_key: String,
    pub state_value: String,
    // SetPlayerAppearance fields
    pub appearance: PlayerAppearance,
    pub appearance_path: String,
}

impl Default for ActionEditorState {
    fn default() -> Self {
        Self {
            action_type: ActionType::JumpTo,
            editing_index: None,
            target_map_id: String::new(),
            target_x: "0".to_string(),
            target_y: "0".to_string(),
            dialog_text_mode: DialogTextMode::Inline,
            dialog_inline_text: String::new(),
            dialog_text_id: String::new(),
            dialog_text_speed: "30".to_string(),
            dialog_position: DialogPositionData::Bottom,
            dialog_movement_block: true,
            shake_mode: ScreenShakeMode::Timed,
            shake_intensity: "5.0".to_string(),
            shake_duration: "0.5".to_string(),
            fade_type: FadeType::FadeOut,
            fade_duration: "1.0".to_string(),
            fade_color: [0.0, 0.0, 0.0, 1.0],
            state_key: String::new(),
            state_value: String::new(),
            appearance: PlayerAppearance::Hidden,
            appearance_path: String::new(),
        }
    }
}

impl ActionEditorState {
    /// Resets all fields to their defaults.
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Populates fields from an existing EventAction for editing.
    pub fn load_from_action(&mut self, action: &EventAction, index: usize) {
        match action {
            EventAction::JumpTo {
                target_map_id,
                target_x,
                target_y,
            } => {
                self.action_type = ActionType::JumpTo;
                self.target_map_id = target_map_id.clone();
                self.target_x = target_x.to_string();
                self.target_y = target_y.to_string();
            }
            EventAction::ShowDialog { text, config } => {
                self.action_type = ActionType::ShowDialog;
                match text {
                    DialogTextData::Inline(s) => {
                        self.dialog_text_mode = DialogTextMode::Inline;
                        self.dialog_inline_text = s.clone();
                        self.dialog_text_id.clear();
                    }
                    DialogTextData::Id(id) => {
                        self.dialog_text_mode = DialogTextMode::TextId;
                        self.dialog_text_id = id.clone();
                        self.dialog_inline_text.clear();
                    }
                }
                self.dialog_text_speed = config.text_speed.to_string();
                self.dialog_position = config.position.clone();
                self.dialog_movement_block = config.movement_block;
            }
            EventAction::ScreenShake {
                intensity,
                duration,
                mode,
            } => {
                self.action_type = ActionType::ScreenShake;
                self.shake_intensity = intensity.to_string();
                self.shake_duration = duration.to_string();
                self.shake_mode = *mode;
            }
            EventAction::StopScreenShake => {
                self.action_type = ActionType::StopScreenShake;
            }
            EventAction::FadeTransition {
                fade_type,
                duration,
                color,
            } => {
                self.action_type = ActionType::FadeTransition;
                self.fade_type = *fade_type;
                self.fade_duration = duration.to_string();
                self.fade_color = *color;
            }
            EventAction::SetState { key, value } => {
                self.action_type = ActionType::SetState;
                self.state_key = key.clone();
                self.state_value = value.clone();
            }
            EventAction::SetPlayerAppearance { appearance } => {
                self.action_type = ActionType::SetPlayerAppearance;
                if let PlayerAppearance::Spritesheet { path } = appearance {
                    self.appearance_path = path.clone();
                } else {
                    self.appearance_path = String::new();
                }
                self.appearance = appearance.clone();
            }
        }
        self.editing_index = Some(index);
    }

    /// Builds an EventAction from the current field values.
    /// Returns None if required fields are empty.
    pub fn build_action(&self) -> Option<EventAction> {
        match self.action_type {
            ActionType::JumpTo => {
                if self.target_map_id.is_empty() {
                    return None;
                }
                let x = self.target_x.trim().parse::<u32>().unwrap_or(0);
                let y = self.target_y.trim().parse::<u32>().unwrap_or(0);
                Some(EventAction::JumpTo {
                    target_map_id: self.target_map_id.clone(),
                    target_x: x,
                    target_y: y,
                })
            }
            ActionType::ShowDialog => {
                let text = match self.dialog_text_mode {
                    DialogTextMode::Inline => {
                        if self.dialog_inline_text.is_empty() {
                            return None;
                        }
                        DialogTextData::Inline(self.dialog_inline_text.clone())
                    }
                    DialogTextMode::TextId => {
                        if self.dialog_text_id.is_empty() {
                            return None;
                        }
                        DialogTextData::Id(self.dialog_text_id.clone())
                    }
                };
                let text_speed = self.dialog_text_speed.trim().parse::<f32>().unwrap_or(30.0);
                let config = DialogConfigData {
                    text_speed,
                    position: self.dialog_position.clone(),
                    movement_block: self.dialog_movement_block,
                };
                Some(EventAction::ShowDialog { text, config })
            }
            ActionType::ScreenShake => {
                let intensity = self
                    .shake_intensity
                    .trim()
                    .parse::<f32>()
                    .unwrap_or(5.0)
                    .clamp(0.0, 50.0);
                let duration = self
                    .shake_duration
                    .trim()
                    .parse::<f32>()
                    .unwrap_or(0.5)
                    .clamp(0.0, 10.0);
                Some(EventAction::ScreenShake {
                    intensity,
                    duration,
                    mode: self.shake_mode,
                })
            }
            ActionType::StopScreenShake => Some(EventAction::StopScreenShake),
            ActionType::FadeTransition => {
                let duration = self
                    .fade_duration
                    .trim()
                    .parse::<f32>()
                    .unwrap_or(1.0)
                    .clamp(0.0, 10.0);
                Some(EventAction::FadeTransition {
                    fade_type: self.fade_type,
                    duration,
                    color: self.fade_color,
                })
            }
            ActionType::SetState => {
                if self.state_key.is_empty() {
                    return None;
                }
                Some(EventAction::SetState {
                    key: self.state_key.clone(),
                    value: self.state_value.clone(),
                })
            }
            ActionType::SetPlayerAppearance => {
                let appearance = match &self.appearance {
                    PlayerAppearance::Spritesheet { .. } => {
                        if self.appearance_path.is_empty() {
                            return None;
                        }
                        PlayerAppearance::Spritesheet {
                            path: self.appearance_path.clone(),
                        }
                    }
                    other => other.clone(),
                };
                Some(EventAction::SetPlayerAppearance { appearance })
            }
        }
    }
}
