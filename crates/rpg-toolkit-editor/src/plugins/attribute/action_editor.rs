//! Shared action editor state and types used by both the
//! Event Trigger Dialog and the NPC Placement Dialog.
//! UI rendering lives in the sibling `action_editor_ui` module.

use crate::data::map::EventAction;
use rpg_toolkit_common::{
    AppPhase, BranchCondition, ChoiceData, ConditionCheck, ConditionLogic, DialogConfigData,
    DialogPositionData, DialogTextData, EntityTarget, FadeType, PlayerAppearance, ScreenShakeMode,
    TransferDirection,
};

/// The type of action being added in the Event Trigger Editor.
#[derive(Default, PartialEq, Clone, Copy)]
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
    GiveCurrency,
    GiveExperience,
    GiveItem,
    LearnAbility,
    AddPartyMember,
    SaveGame,
    ChangePhase,
    OpenShop,
    MoveEntity,
    CameraFollow,
    CameraPan,
    Wait,
    Jump,
    SetSpeed,
}

/// A named grouping of action types for the categorized action-type dropdown.
///
/// Each category owns a static list of `(variant, display_name)` pairs. Every
/// `ActionType` variant belongs to exactly one category (see [`ACTION_CATEGORIES`]).
pub struct ActionCategory {
    /// Display name for the category header.
    pub name: &'static str,
    /// Action variants belonging to this category, paired with their display names.
    pub actions: &'static [(ActionType, &'static str)],
}

/// Static mapping of action types into named categories, per Requirement 10.8.
///
/// Every `ActionType` variant appears in exactly one category. The order here
/// determines the order categories and actions are shown in the dropdown.
pub const ACTION_CATEGORIES: &[ActionCategory] = &[
    ActionCategory {
        name: "Dialog",
        actions: &[
            (ActionType::ShowDialog, "Show Dialog"),
            (ActionType::ShowSelection, "Show Selection"),
        ],
    },
    ActionCategory {
        name: "Movement",
        actions: &[
            (ActionType::JumpTo, "Jump To Map"),
            (ActionType::Jump, "Jump"),
            (ActionType::SetSpeed, "Set Speed"),
            (ActionType::MoveEntity, "Move Entity"),
        ],
    },
    ActionCategory {
        name: "Camera",
        actions: &[
            (ActionType::CameraFollow, "Camera Follow"),
            (ActionType::CameraPan, "Camera Pan"),
        ],
    },
    ActionCategory {
        name: "Rewards",
        actions: &[
            (ActionType::GiveCurrency, "Give Currency"),
            (ActionType::GiveExperience, "Give Experience"),
            (ActionType::GiveItem, "Give Item"),
            (ActionType::LearnAbility, "Learn Ability"),
            (ActionType::AddPartyMember, "Add Party Member"),
        ],
    },
    ActionCategory {
        name: "State",
        actions: &[
            (ActionType::SetState, "Set State"),
            (ActionType::StateCheck, "State Check"),
            (ActionType::Branch, "Branch"),
            (ActionType::SaveGame, "Save Game"),
            (ActionType::ChangePhase, "Change Phase"),
        ],
    },
    ActionCategory {
        name: "Visual Effects",
        actions: &[
            (ActionType::ScreenShake, "Screen Shake"),
            (ActionType::StopScreenShake, "Stop Screen Shake"),
            (ActionType::FadeTransition, "Fade Transition"),
            (ActionType::SetPlayerAppearance, "Set Player Appearance"),
        ],
    },
    ActionCategory {
        name: "System",
        actions: &[
            (ActionType::Wait, "Wait"),
            (ActionType::OpenShop, "Open Shop"),
        ],
    },
];

/// Returns the display name for an [`ActionType`] from [`ACTION_CATEGORIES`].
///
/// Falls back to an empty string if the variant is somehow not present (which
/// should never happen since every variant is categorized).
pub fn action_type_display_name(action_type: ActionType) -> &'static str {
    for category in ACTION_CATEGORIES {
        for (variant, name) in category.actions {
            if *variant == action_type {
                return name;
            }
        }
    }
    ""
}

/// Filters [`ACTION_CATEGORIES`] by a case-insensitive substring match against
/// each action's display name.
///
/// - When `filter` is empty, every category is returned with all of its actions.
/// - When `filter` is non-empty, only actions whose display name contains the
///   filter (case-insensitive) are kept, and categories with no matching actions
///   are omitted entirely.
/// - When `include_predicate` returns `false` for a variant, that action is
///   excluded regardless of the filter (used to hide `StateCheck`/`Branch` at
///   nested depths).
///
/// Returns a list of `(category_name, matching_actions)` pairs preserving the
/// declaration order of [`ACTION_CATEGORIES`].
pub fn filter_action_categories(
    filter: &str,
    include_predicate: impl Fn(ActionType) -> bool,
) -> Vec<(&'static str, Vec<(ActionType, &'static str)>)> {
    let filter_lower = filter.to_lowercase();
    let mut result = Vec::new();
    for category in ACTION_CATEGORIES {
        let matching: Vec<(ActionType, &'static str)> = category
            .actions
            .iter()
            .filter(|(variant, _)| include_predicate(*variant))
            .filter(|(_, name)| {
                filter_lower.is_empty() || name.to_lowercase().contains(&filter_lower)
            })
            .map(|(variant, name)| (*variant, *name))
            .collect();
        if !matching.is_empty() {
            result.push((category.name, matching));
        }
    }
    result
}

/// A single choice in the editor for a ShowSelection action.
pub struct EditorChoice {
    pub label_text: String,
    pub actions: Vec<EventAction>,
    /// Persistent action editor state for this choice's nested action list.
    pub action_editor: ActionEditorState,
}

impl Default for EditorChoice {
    fn default() -> Self {
        Self {
            label_text: String::new(),
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
    /// Search filter buffer for the categorized action-type dropdown.
    pub action_type_search: String,
    pub editing_index: Option<usize>,
    // JumpTo fields
    pub target_map_id: String,
    pub target_x: String,
    pub target_y: String,
    pub target_elevation: String,
    // ShowDialog fields
    pub dialog_inline_text: String,
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
    pub selection_prompt_text: String,
    pub selection_position: DialogPositionData,
    pub selection_face_portrait: Option<String>,
    pub selection_choices: Vec<EditorChoice>,
    // Reward action shared fields
    pub reward_direction: TransferDirection,
    pub reward_on_success: Vec<EventAction>,
    pub reward_on_failure: Vec<EventAction>,
    pub reward_on_success_editor: Option<Box<ActionEditorState>>,
    pub reward_on_failure_editor: Option<Box<ActionEditorState>>,
    // GiveCurrency fields
    pub currency_amount: String,
    // GiveExperience fields
    pub experience_amount: String,
    pub experience_target: Option<String>,
    // GiveItem fields
    pub give_item_id: String,
    pub give_item_quantity: String,
    // LearnAbility fields
    pub learn_ability_id: String,
    pub learn_ability_target: String,
    // AddPartyMember fields
    pub add_party_character_id: String,
    // ChangePhase fields
    pub change_phase_target: AppPhase,
    // OpenShop fields
    pub open_shop_id: String,
    pub shop_search_buffer: String,
    // Portrait search buffer (for ShowDialog and ShowSelection)
    pub portrait_search_buffer: String,
    // MoveEntity fields
    pub move_entity_target_is_player: bool,
    pub move_entity_npc_id: String,
    pub move_target_x: String,
    pub move_target_y: String,
    pub move_speed: f32,
    // CameraFollow fields
    pub camera_follow_target_is_player: bool,
    pub camera_follow_npc_id: String,
    // CameraPan fields
    pub camera_pan_target_x: String,
    pub camera_pan_target_y: String,
    pub camera_pan_duration: f32,
    // Wait fields
    pub wait_duration: f32,
    // Jump fields
    pub jump_distance: String,
    // SetSpeed fields
    pub speed_multiplier: f32,
}

impl Default for ActionEditorState {
    fn default() -> Self {
        Self {
            action_type: ActionType::JumpTo,
            action_type_search: String::new(),
            editing_index: None,
            target_map_id: String::new(),
            target_x: "0".to_string(),
            target_y: "0".to_string(),
            target_elevation: String::new(),
            dialog_inline_text: String::new(),
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
            selection_prompt_text: String::new(),
            selection_position: DialogPositionData::Bottom,
            selection_face_portrait: None,
            selection_choices: vec![EditorChoice::default(), EditorChoice::default()],
            // Reward action shared fields
            reward_direction: TransferDirection::Give,
            reward_on_success: Vec::new(),
            reward_on_failure: Vec::new(),
            reward_on_success_editor: Some(Box::new(ActionEditorState::new_nested())),
            reward_on_failure_editor: Some(Box::new(ActionEditorState::new_nested())),
            // GiveCurrency fields
            currency_amount: "100".to_string(),
            // GiveExperience fields
            experience_amount: "100".to_string(),
            experience_target: None,
            // GiveItem fields
            give_item_id: String::new(),
            give_item_quantity: "1".to_string(),
            // LearnAbility fields
            learn_ability_id: String::new(),
            learn_ability_target: String::new(),
            // AddPartyMember fields
            add_party_character_id: String::new(),
            // ChangePhase fields
            change_phase_target: AppPhase::InGame,
            // OpenShop fields
            open_shop_id: String::new(),
            shop_search_buffer: String::new(),
            // Portrait search buffer
            portrait_search_buffer: String::new(),
            // MoveEntity fields
            move_entity_target_is_player: true,
            move_entity_npc_id: String::new(),
            move_target_x: "0".to_string(),
            move_target_y: "0".to_string(),
            move_speed: 2.0,
            // CameraFollow fields
            camera_follow_target_is_player: true,
            camera_follow_npc_id: String::new(),
            // CameraPan fields
            camera_pan_target_x: "0".to_string(),
            camera_pan_target_y: "0".to_string(),
            camera_pan_duration: 1.0,
            // Wait fields
            wait_duration: 1.0,
            // Jump fields
            jump_distance: "2".to_string(),
            // SetSpeed fields
            speed_multiplier: 1.0,
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
            action_type_search: String::new(),
            editing_index: None,
            target_map_id: String::new(),
            target_x: "0".to_string(),
            target_y: "0".to_string(),
            target_elevation: String::new(),
            dialog_inline_text: String::new(),
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
            selection_prompt_text: String::new(),
            selection_position: DialogPositionData::Bottom,
            selection_face_portrait: None,
            selection_choices: Vec::new(), // Empty — no recursion
            // Reward action shared fields — no recursive Box here
            reward_direction: TransferDirection::Give,
            reward_on_success: Vec::new(),
            reward_on_failure: Vec::new(),
            reward_on_success_editor: None,
            reward_on_failure_editor: None,
            // GiveCurrency fields
            currency_amount: "100".to_string(),
            // GiveExperience fields
            experience_amount: "100".to_string(),
            experience_target: None,
            // GiveItem fields
            give_item_id: String::new(),
            give_item_quantity: "1".to_string(),
            // LearnAbility fields
            learn_ability_id: String::new(),
            learn_ability_target: String::new(),
            // AddPartyMember fields
            add_party_character_id: String::new(),
            // ChangePhase fields
            change_phase_target: AppPhase::InGame,
            // OpenShop fields
            open_shop_id: String::new(),
            shop_search_buffer: String::new(),
            // Portrait search buffer
            portrait_search_buffer: String::new(),
            // MoveEntity fields
            move_entity_target_is_player: true,
            move_entity_npc_id: String::new(),
            move_target_x: "0".to_string(),
            move_target_y: "0".to_string(),
            move_speed: 2.0,
            // CameraFollow fields
            camera_follow_target_is_player: true,
            camera_follow_npc_id: String::new(),
            // CameraPan fields
            camera_pan_target_x: "0".to_string(),
            camera_pan_target_y: "0".to_string(),
            camera_pan_duration: 1.0,
            // Wait fields
            wait_duration: 1.0,
            // Jump fields
            jump_distance: "2".to_string(),
            // SetSpeed fields
            speed_multiplier: 1.0,
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
                        self.dialog_inline_text = s.clone();
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
                        self.selection_prompt_text = s.clone();
                    }
                }
                self.selection_position = config.position.clone();
                self.selection_face_portrait = config.face_portrait.clone();
                self.selection_choices = choices
                    .iter()
                    .map(|choice| {
                        let label_text = match &choice.label {
                            DialogTextData::Inline(s) => s.clone(),
                        };
                        EditorChoice {
                            label_text,
                            actions: choice.actions.clone(),
                            action_editor: ActionEditorState::new_nested(),
                        }
                    })
                    .collect();
            }
            // Reward action variants
            EventAction::GiveCurrency {
                amount,
                direction,
                on_success,
                on_failure,
            } => {
                self.action_type = ActionType::GiveCurrency;
                self.currency_amount = amount.to_string();
                self.reward_direction = *direction;
                self.reward_on_success = on_success.clone();
                self.reward_on_failure = on_failure.clone();
            }
            EventAction::GiveExperience {
                amount,
                target,
                direction,
                on_success,
                on_failure,
            } => {
                self.action_type = ActionType::GiveExperience;
                self.experience_amount = amount.to_string();
                self.experience_target = target.clone();
                self.reward_direction = *direction;
                self.reward_on_success = on_success.clone();
                self.reward_on_failure = on_failure.clone();
            }
            EventAction::GiveItem {
                item_id,
                quantity,
                direction,
                on_success,
                on_failure,
            } => {
                self.action_type = ActionType::GiveItem;
                self.give_item_id = item_id.clone();
                self.give_item_quantity = quantity.to_string();
                self.reward_direction = *direction;
                self.reward_on_success = on_success.clone();
                self.reward_on_failure = on_failure.clone();
            }
            EventAction::LearnAbility {
                ability_id,
                target,
                direction,
                on_success,
                on_failure,
            } => {
                self.action_type = ActionType::LearnAbility;
                self.learn_ability_id = ability_id.clone();
                self.learn_ability_target = target.clone();
                self.reward_direction = *direction;
                self.reward_on_success = on_success.clone();
                self.reward_on_failure = on_failure.clone();
            }
            EventAction::AddPartyMember {
                character_id,
                direction,
                on_success,
                on_failure,
            } => {
                self.action_type = ActionType::AddPartyMember;
                self.add_party_character_id = character_id.clone();
                self.reward_direction = *direction;
                self.reward_on_success = on_success.clone();
                self.reward_on_failure = on_failure.clone();
            }
            EventAction::SaveGame => {
                self.action_type = ActionType::SaveGame;
            }
            EventAction::ChangePhase { phase } => {
                self.action_type = ActionType::ChangePhase;
                self.change_phase_target = phase.clone();
            }
            EventAction::OpenShop { shop_id } => {
                self.action_type = ActionType::OpenShop;
                self.open_shop_id = shop_id.clone();
            }
            // New cinematic action variants
            EventAction::MoveEntity {
                target,
                target_x,
                target_y,
                speed,
            } => {
                self.action_type = ActionType::MoveEntity;
                match target {
                    EntityTarget::Player => {
                        self.move_entity_target_is_player = true;
                        self.move_entity_npc_id.clear();
                    }
                    EntityTarget::Npc { npc_id } => {
                        self.move_entity_target_is_player = false;
                        self.move_entity_npc_id = npc_id.clone();
                    }
                }
                self.move_target_x = target_x.to_string();
                self.move_target_y = target_y.to_string();
                self.move_speed = *speed;
            }
            EventAction::CameraFollow { target } => {
                self.action_type = ActionType::CameraFollow;
                match target {
                    EntityTarget::Player => {
                        self.camera_follow_target_is_player = true;
                        self.camera_follow_npc_id.clear();
                    }
                    EntityTarget::Npc { npc_id } => {
                        self.camera_follow_target_is_player = false;
                        self.camera_follow_npc_id = npc_id.clone();
                    }
                }
            }
            EventAction::CameraPan {
                target_x,
                target_y,
                duration,
            } => {
                self.action_type = ActionType::CameraPan;
                self.camera_pan_target_x = target_x.to_string();
                self.camera_pan_target_y = target_y.to_string();
                self.camera_pan_duration = *duration;
            }
            EventAction::Wait { duration } => {
                self.action_type = ActionType::Wait;
                self.wait_duration = *duration;
            }
            // Jump and SetSpeed — fully integrated
            EventAction::Jump { distance } => {
                self.action_type = ActionType::Jump;
                self.jump_distance = distance.to_string();
            }
            EventAction::SetSpeed { multiplier } => {
                self.action_type = ActionType::SetSpeed;
                self.speed_multiplier = *multiplier;
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
                if self.dialog_inline_text.is_empty() {
                    return None;
                }
                let text = DialogTextData::Inline(self.dialog_inline_text.clone());
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
                    if choice.label_text.is_empty() {
                        return None;
                    }
                }
                // Build prompt
                if self.selection_prompt_text.is_empty() {
                    return None;
                }
                let prompt = DialogTextData::Inline(self.selection_prompt_text.clone());
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
                        let label = DialogTextData::Inline(ec.label_text.clone());
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
            ActionType::GiveCurrency => {
                let amount = self.currency_amount.trim().parse::<u64>().ok()?;
                if amount == 0 {
                    return None;
                }
                let amount = amount.clamp(1, 9_999_999);
                Some(EventAction::GiveCurrency {
                    amount,
                    direction: self.reward_direction,
                    on_success: self.reward_on_success.clone(),
                    on_failure: self.reward_on_failure.clone(),
                })
            }
            ActionType::GiveExperience => {
                let amount = self.experience_amount.trim().parse::<u64>().ok()?;
                if amount == 0 {
                    return None;
                }
                let amount = amount.clamp(1, 9_999_999);
                let target = match &self.experience_target {
                    Some(t) if !t.trim().is_empty() => Some(t.clone()),
                    _ => None,
                };
                Some(EventAction::GiveExperience {
                    amount,
                    target,
                    direction: self.reward_direction,
                    on_success: self.reward_on_success.clone(),
                    on_failure: self.reward_on_failure.clone(),
                })
            }
            ActionType::GiveItem => {
                if self.give_item_id.trim().is_empty() {
                    return None;
                }
                let quantity = self
                    .give_item_quantity
                    .trim()
                    .parse::<u32>()
                    .unwrap_or(1)
                    .clamp(1, 999);
                Some(EventAction::GiveItem {
                    item_id: self.give_item_id.clone(),
                    quantity,
                    direction: self.reward_direction,
                    on_success: self.reward_on_success.clone(),
                    on_failure: self.reward_on_failure.clone(),
                })
            }
            ActionType::LearnAbility => {
                if self.learn_ability_id.trim().is_empty()
                    || self.learn_ability_target.trim().is_empty()
                {
                    return None;
                }
                Some(EventAction::LearnAbility {
                    ability_id: self.learn_ability_id.clone(),
                    target: self.learn_ability_target.clone(),
                    direction: self.reward_direction,
                    on_success: self.reward_on_success.clone(),
                    on_failure: self.reward_on_failure.clone(),
                })
            }
            ActionType::AddPartyMember => {
                let id = self.add_party_character_id.trim();
                if id.is_empty() || id.len() > 64 {
                    return None;
                }
                Some(EventAction::AddPartyMember {
                    character_id: self.add_party_character_id.clone(),
                    direction: self.reward_direction,
                    on_success: self.reward_on_success.clone(),
                    on_failure: self.reward_on_failure.clone(),
                })
            }
            ActionType::SaveGame => Some(EventAction::SaveGame),
            ActionType::ChangePhase => Some(EventAction::ChangePhase {
                phase: self.change_phase_target.clone(),
            }),
            ActionType::OpenShop => {
                if self.open_shop_id.trim().is_empty() {
                    return None;
                }
                Some(EventAction::OpenShop {
                    shop_id: self.open_shop_id.clone(),
                })
            }
            ActionType::MoveEntity => {
                let target = if self.move_entity_target_is_player {
                    EntityTarget::Player
                } else {
                    if self.move_entity_npc_id.trim().is_empty() {
                        return None;
                    }
                    EntityTarget::Npc {
                        npc_id: self.move_entity_npc_id.clone(),
                    }
                };
                let x = self.move_target_x.trim().parse::<u32>().unwrap_or(0);
                let y = self.move_target_y.trim().parse::<u32>().unwrap_or(0);
                let speed = self.move_speed.clamp(0.1, 10.0);
                Some(EventAction::MoveEntity {
                    target,
                    target_x: x,
                    target_y: y,
                    speed,
                })
            }
            ActionType::CameraFollow => {
                let target = if self.camera_follow_target_is_player {
                    EntityTarget::Player
                } else {
                    if self.camera_follow_npc_id.trim().is_empty() {
                        return None;
                    }
                    EntityTarget::Npc {
                        npc_id: self.camera_follow_npc_id.clone(),
                    }
                };
                Some(EventAction::CameraFollow { target })
            }
            ActionType::CameraPan => {
                let x = self.camera_pan_target_x.trim().parse::<u32>().unwrap_or(0);
                let y = self.camera_pan_target_y.trim().parse::<u32>().unwrap_or(0);
                let duration = self.camera_pan_duration.clamp(0.1, 10.0);
                Some(EventAction::CameraPan {
                    target_x: x,
                    target_y: y,
                    duration,
                })
            }
            ActionType::Wait => {
                let duration = self.wait_duration.clamp(0.1, 30.0);
                Some(EventAction::Wait { duration })
            }
            ActionType::Jump => {
                let distance = rpg_toolkit_editor::clamp_jump_distance(&self.jump_distance);
                Some(EventAction::Jump { distance })
            }
            ActionType::SetSpeed => {
                let multiplier = rpg_toolkit_editor::clamp_speed_multiplier(self.speed_multiplier);
                Some(EventAction::SetSpeed { multiplier })
            }
        }
    }
}
