//! Shared action editor state and types used by both the
//! Event Trigger Dialog and the NPC Placement Dialog.
//! UI rendering lives in the sibling `action_editor_ui` module.

use crate::data::map::EventAction;
use rpg_toolkit_common::{
    BranchCondition, ChoiceData, ConditionCheck, ConditionLogic, DialogConfigData,
    DialogPositionData, DialogTextData, FadeType, PlayerAppearance, ScreenShakeMode,
};

/// The type of action being added in the Event Trigger Editor.
#[derive(Default, PartialEq)]
pub enum ActionType {
    #[default]
    JumpTo,
    ShowDialog,
    ShowSelection,
    ScreenShake,
    StopScreenShake,
    FadeTransition,
    SetState,
    SetPlayerAppearance,
    StateCheck,
    Branch,
}

/// The text source mode for a ShowDialog action.
#[derive(Default, PartialEq)]
pub enum DialogTextMode {
    #[default]
    Inline,
    TextId,
}

/// A single choice in the editor for a ShowSelection action.
pub struct EditorChoice {
    pub label_mode: DialogTextMode,
    pub label_text: String,
    pub label_id: String,
    pub actions: Vec<EventAction>,
    /// Persistent action editor state for this choice's nested action list.
    pub action_editor: ActionEditorState,
}

impl Default for EditorChoice {
    fn default() -> Self {
        Self {
            label_mode: DialogTextMode::Inline,
            label_text: String::new(),
            label_id: String::new(),
            actions: Vec::new(),
            // Use new_nested() to avoid recursion (no selection_choices inside)
            action_editor: ActionEditorState::new_nested(),
        }
    }
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
    pub target_elevation: String,
    // ShowDialog fields
    pub dialog_text_mode: DialogTextMode,
    pub dialog_inline_text: String,
    pub dialog_text_id: String,
    pub dialog_text_speed: String,
    pub dialog_position: DialogPositionData,
    pub dialog_movement_block: bool,
    pub dialog_face_portrait: Option<String>,
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
    // StateCheck fields
    pub state_check_key: String,
    pub state_check_value: String,
    pub state_check_on_true_idx: usize,
    pub state_check_on_false_idx: usize,
    // Branch fields
    pub branch_logic: ConditionLogic,
    pub branch_checks: Vec<ConditionCheck>,
    pub branch_on_true: Vec<EventAction>,
    pub branch_on_false: Vec<EventAction>,
    // ShowSelection fields
    pub selection_prompt_mode: DialogTextMode,
    pub selection_prompt_text: String,
    pub selection_prompt_id: String,
    pub selection_position: DialogPositionData,
    pub selection_face_portrait: Option<String>,
    pub selection_choices: Vec<EditorChoice>,
}

impl Default for ActionEditorState {
    fn default() -> Self {
        Self {
            action_type: ActionType::JumpTo,
            editing_index: None,
            target_map_id: String::new(),
            target_x: "0".to_string(),
            target_y: "0".to_string(),
            target_elevation: String::new(),
            dialog_text_mode: DialogTextMode::Inline,
            dialog_inline_text: String::new(),
            dialog_text_id: String::new(),
            dialog_text_speed: "30".to_string(),
            dialog_position: DialogPositionData::Bottom,
            dialog_movement_block: true,
            dialog_face_portrait: None,
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
            state_check_key: String::new(),
            state_check_value: String::new(),
            state_check_on_true_idx: 0,
            state_check_on_false_idx: 0,
            branch_logic: ConditionLogic::All,
            branch_checks: Vec::new(),
            branch_on_true: Vec::new(),
            branch_on_false: Vec::new(),
            selection_prompt_mode: DialogTextMode::Inline,
            selection_prompt_text: String::new(),
            selection_prompt_id: String::new(),
            selection_position: DialogPositionData::Bottom,
            selection_face_portrait: None,
            selection_choices: vec![EditorChoice::default(), EditorChoice::default()],
        }
    }
}

impl ActionEditorState {
    /// Resets all fields to their defaults.
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Creates a nested ActionEditorState without default selection choices.
    /// Used by EditorChoice to avoid infinite recursion in defaults.
    pub fn new_nested() -> Self {
        Self {
            action_type: ActionType::JumpTo,
            editing_index: None,
            target_map_id: String::new(),
            target_x: "0".to_string(),
            target_y: "0".to_string(),
            target_elevation: String::new(),
            dialog_text_mode: DialogTextMode::Inline,
            dialog_inline_text: String::new(),
            dialog_text_id: String::new(),
            dialog_text_speed: "30".to_string(),
            dialog_position: DialogPositionData::Bottom,
            dialog_movement_block: true,
            dialog_face_portrait: None,
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
            state_check_key: String::new(),
            state_check_value: String::new(),
            state_check_on_true_idx: 0,
            state_check_on_false_idx: 0,
            branch_logic: ConditionLogic::All,
            branch_checks: Vec::new(),
            branch_on_true: Vec::new(),
            branch_on_false: Vec::new(),
            selection_prompt_mode: DialogTextMode::Inline,
            selection_prompt_text: String::new(),
            selection_prompt_id: String::new(),
            selection_position: DialogPositionData::Bottom,
            selection_face_portrait: None,
            selection_choices: Vec::new(), // Empty — no recursion
        }
    }

    /// Populates fields from an existing EventAction for editing.
    pub fn load_from_action(&mut self, action: &EventAction, index: usize) {
        match action {
            EventAction::JumpTo {
                target_map_id,
                target_x,
                target_y,
                target_elevation,
                ..
            } => {
                self.action_type = ActionType::JumpTo;
                self.target_map_id = target_map_id.clone();
                self.target_x = target_x.to_string();
                self.target_y = target_y.to_string();
                self.target_elevation = target_elevation.map(|e| e.to_string()).unwrap_or_default();
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
                self.dialog_face_portrait = config.face_portrait.clone();
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
            EventAction::StateCheck {
                key,
                value,
                on_true,
                on_false,
            } => {
                self.action_type = ActionType::StateCheck;
                self.state_check_key = key.clone();
                self.state_check_value = value.clone().unwrap_or_default();
                self.state_check_on_true_idx = on_true.len();
                self.state_check_on_false_idx = on_false.len();
            }
            EventAction::Branch {
                condition,
                on_true,
                on_false,
            } => {
                self.action_type = ActionType::Branch;
                self.branch_logic = condition.logic;
                self.branch_checks = condition.checks.clone();
                self.branch_on_true = on_true.clone();
                self.branch_on_false = on_false.clone();
            }
            EventAction::ShowSelection {
                prompt,
                config,
                choices,
            } => {
                self.action_type = ActionType::ShowSelection;
                match prompt {
                    DialogTextData::Inline(s) => {
                        self.selection_prompt_mode = DialogTextMode::Inline;
                        self.selection_prompt_text = s.clone();
                        self.selection_prompt_id.clear();
                    }
                    DialogTextData::Id(id) => {
                        self.selection_prompt_mode = DialogTextMode::TextId;
                        self.selection_prompt_id = id.clone();
                        self.selection_prompt_text.clear();
                    }
                }
                self.selection_position = config.position.clone();
                self.selection_face_portrait = config.face_portrait.clone();
                self.selection_choices = choices
                    .iter()
                    .map(|choice| {
                        let (label_mode, label_text, label_id) = match &choice.label {
                            DialogTextData::Inline(s) => {
                                (DialogTextMode::Inline, s.clone(), String::new())
                            }
                            DialogTextData::Id(id) => {
                                (DialogTextMode::TextId, String::new(), id.clone())
                            }
                        };
                        EditorChoice {
                            label_mode,
                            label_text,
                            label_id,
                            actions: choice.actions.clone(),
                            action_editor: ActionEditorState::new_nested(),
                        }
                    })
                    .collect();
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
                let target_elevation = self.target_elevation.trim().parse::<u32>().ok();
                Some(EventAction::JumpTo {
                    target_map_id: self.target_map_id.clone(),
                    target_x: x,
                    target_y: y,
                    target_elevation,
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
                    attribute_dialog: false,
                    face_portrait: self.dialog_face_portrait.clone(),
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
            ActionType::StateCheck => {
                let value = if self.state_check_value.is_empty() {
                    None
                } else {
                    Some(self.state_check_value.clone())
                };
                Some(EventAction::StateCheck {
                    key: self.state_check_key.clone(),
                    value,
                    on_true: Vec::new(),
                    on_false: Vec::new(),
                })
            }
            ActionType::Branch => {
                // Validate: at least one check with a non-empty key
                if self.branch_checks.is_empty() {
                    return None;
                }
                if self.branch_checks.iter().any(|c| c.key.is_empty()) {
                    return None;
                }
                let condition = BranchCondition {
                    logic: self.branch_logic,
                    checks: self.branch_checks.clone(),
                };
                Some(EventAction::Branch {
                    condition,
                    on_true: self.branch_on_true.clone(),
                    on_false: self.branch_on_false.clone(),
                })
            }
            ActionType::ShowSelection => {
                // Validate: at least 2 choices
                if self.selection_choices.len() < 2 {
                    return None;
                }
                // Validate: each choice must have a non-empty label
                for choice in &self.selection_choices {
                    match choice.label_mode {
                        DialogTextMode::Inline => {
                            if choice.label_text.is_empty() {
                                return None;
                            }
                        }
                        DialogTextMode::TextId => {
                            if choice.label_id.is_empty() {
                                return None;
                            }
                        }
                    }
                }
                // Build prompt
                let prompt = match self.selection_prompt_mode {
                    DialogTextMode::Inline => {
                        if self.selection_prompt_text.is_empty() {
                            return None;
                        }
                        DialogTextData::Inline(self.selection_prompt_text.clone())
                    }
                    DialogTextMode::TextId => {
                        if self.selection_prompt_id.is_empty() {
                            return None;
                        }
                        DialogTextData::Id(self.selection_prompt_id.clone())
                    }
                };
                // Build config
                let config = DialogConfigData {
                    text_speed: 30.0,
                    position: self.selection_position.clone(),
                    movement_block: true,
                    attribute_dialog: false,
                    face_portrait: self.selection_face_portrait.clone(),
                };
                // Build choices
                let choices: Vec<ChoiceData> = self
                    .selection_choices
                    .iter()
                    .map(|ec| {
                        let label = match ec.label_mode {
                            DialogTextMode::Inline => DialogTextData::Inline(ec.label_text.clone()),
                            DialogTextMode::TextId => DialogTextData::Id(ec.label_id.clone()),
                        };
                        ChoiceData {
                            label,
                            actions: ec.actions.clone(),
                        }
                    })
                    .collect();
                Some(EventAction::ShowSelection {
                    prompt,
                    config,
                    choices,
                })
            }
        }
    }
}
