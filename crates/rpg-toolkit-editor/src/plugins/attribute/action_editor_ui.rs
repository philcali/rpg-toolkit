//! Main action editor UI orchestrator — renders the action list, type selector,
//! and dispatches to per-action form renderers in `action_editor_forms`.

use bevy_egui::egui;

use rpg_toolkit_common::{DialogTextData, TransferDirection};

use crate::data::map::EventAction;
use crate::plugins::searchable_combobox::searchable_combobox;

use super::action_editor::{
    ActionEditorState, ActionType, action_type_display_name, filter_action_categories,
    truncate_preview,
};
use super::action_editor_forms;

/// Context holding registry data and search buffers needed by reward action forms.
/// Passed through `render_action_editor` to avoid widening the main function signature
/// with multiple optional parameters.
pub struct RewardFormContext<'a> {
    /// (id, display_name) pairs for items from ItemRegistry.
    pub items: &'a [(String, String)],
    /// (id, display_name) pairs for abilities from AbilityRegistry.
    pub abilities: &'a [(String, String)],
    /// (id, display_name) pairs for characters from CharacterRegistry.
    pub characters: &'a [(String, String)],
    /// Search buffer for item selectors.
    pub item_search: &'a mut String,
    /// Search buffer for ability selectors.
    pub ability_search: &'a mut String,
    /// Search buffer for character selectors (reward forms).
    pub character_search: &'a mut String,
}

/// Renders the action editor UI into the given egui Ui.
/// Operates on the provided action list and editor state.
/// `depth` controls nesting: at depth >= 1, Branch and StateCheck are excluded
/// from the action type dropdown to prevent deep nesting in the editor.
#[allow(clippy::too_many_arguments)]
pub fn render_action_editor(
    ui: &mut egui::Ui,
    actions: &mut Vec<EventAction>,
    editor_state: &mut ActionEditorState,
    id_salt: &str,
    map_entries: &[(String, String)],
    portrait_entries: &[(String, String)],
    depth: usize,
    reward_ctx: Option<&mut RewardFormContext<'_>>,
    shops: &[(String, String)],
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
                EventAction::ShowSelection { choices, .. } => {
                    format!("{}. ShowSelection — {} choices", i + 1, choices.len())
                }
                // Reward action variants — editor labels added in task 7.x
                EventAction::GiveCurrency { amount, .. } => {
                    format!("{}. GiveCurrency — {}", i + 1, amount)
                }
                EventAction::GiveExperience { amount, .. } => {
                    format!("{}. GiveExperience — {}", i + 1, amount)
                }
                EventAction::GiveItem {
                    item_id, quantity, ..
                } => {
                    format!("{}. GiveItem — {} x{}", i + 1, item_id, quantity)
                }
                EventAction::LearnAbility {
                    ability_id, target, ..
                } => {
                    format!("{}. LearnAbility — {} → {}", i + 1, ability_id, target)
                }
                EventAction::AddPartyMember { character_id, .. } => {
                    format!("{}. AddPartyMember — {}", i + 1, character_id)
                }
                EventAction::SaveGame => {
                    format!("{}. SaveGame", i + 1)
                }
                EventAction::ChangePhase { phase } => {
                    format!("{}. ChangePhase — {:?}", i + 1, phase)
                }
                EventAction::OpenShop { shop_id } => {
                    format!("{}. OpenShop — {}", i + 1, shop_id)
                }
                EventAction::MoveEntity {
                    target,
                    target_x,
                    target_y,
                    speed,
                } => {
                    format!(
                        "{}. MoveEntity — {:?} → ({}, {}) @{}",
                        i + 1,
                        target,
                        target_x,
                        target_y,
                        speed
                    )
                }
                EventAction::CameraFollow { target } => {
                    format!("{}. CameraFollow — {:?}", i + 1, target)
                }
                EventAction::CameraPan {
                    target_x,
                    target_y,
                    duration,
                } => {
                    format!(
                        "{}. CameraPan → ({}, {}) over {}s",
                        i + 1,
                        target_x,
                        target_y,
                        duration
                    )
                }
                EventAction::Wait { duration } => {
                    format!("{}. Wait — {}s", i + 1, duration)
                }
                EventAction::Jump { distance } => {
                    format!("{}. Jump — {} tiles", i + 1, distance)
                }
                EventAction::SetSpeed { multiplier } => {
                    format!("{}. SetSpeed — {}x", i + 1, multiplier)
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
        render_nested_branch_editors(ui, actions, id_salt, map_entries, portrait_entries, shops);
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
        render_action_type_dropdown(ui, editor_state, id_salt, depth, shops);
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
                portrait_entries,
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
        ActionType::ShowSelection => {
            action_editor_forms::render_show_selection_form(
                ui,
                actions,
                editor_state,
                id_salt,
                map_entries,
                portrait_entries,
                shops,
            );
        }
        // Reward action forms
        ActionType::GiveCurrency => {
            action_editor_forms::render_give_currency_form(ui, actions, editor_state, id_salt);
        }
        ActionType::GiveExperience => {
            if let Some(ctx) = reward_ctx {
                action_editor_forms::render_give_experience_form(
                    ui,
                    actions,
                    editor_state,
                    id_salt,
                    ctx.characters,
                );
            } else {
                action_editor_forms::render_give_experience_form(
                    ui,
                    actions,
                    editor_state,
                    id_salt,
                    &[],
                );
            }
        }
        ActionType::GiveItem => {
            if let Some(ctx) = reward_ctx {
                action_editor_forms::render_give_item_form(
                    ui,
                    actions,
                    editor_state,
                    id_salt,
                    ctx.items,
                    ctx.item_search,
                );
            } else {
                let mut fallback = String::new();
                action_editor_forms::render_give_item_form(
                    ui,
                    actions,
                    editor_state,
                    id_salt,
                    &[],
                    &mut fallback,
                );
            }
        }
        ActionType::LearnAbility => {
            if let Some(ctx) = reward_ctx {
                action_editor_forms::render_learn_ability_form(
                    ui,
                    actions,
                    editor_state,
                    id_salt,
                    ctx.abilities,
                    ctx.characters,
                    ctx.ability_search,
                    ctx.character_search,
                );
            } else {
                let mut ab_fallback = String::new();
                let mut ch_fallback = String::new();
                action_editor_forms::render_learn_ability_form(
                    ui,
                    actions,
                    editor_state,
                    id_salt,
                    &[],
                    &[],
                    &mut ab_fallback,
                    &mut ch_fallback,
                );
            }
        }
        ActionType::AddPartyMember => {
            if let Some(ctx) = reward_ctx {
                action_editor_forms::render_add_party_member_form(
                    ui,
                    actions,
                    editor_state,
                    id_salt,
                    ctx.characters,
                    ctx.character_search,
                );
            } else {
                let mut fallback = String::new();
                action_editor_forms::render_add_party_member_form(
                    ui,
                    actions,
                    editor_state,
                    id_salt,
                    &[],
                    &mut fallback,
                );
            }
        }
        ActionType::SaveGame => {
            action_editor_forms::render_save_game_form(ui, actions, editor_state);
        }
        ActionType::ChangePhase => {
            action_editor_forms::render_change_phase_form(ui, actions, editor_state, id_salt);
        }
        ActionType::OpenShop => {
            // Searchable shop selector populated from ShopRegistry
            let current_label = if editor_state.open_shop_id.is_empty() {
                "Select a shop…".to_string()
            } else {
                shops
                    .iter()
                    .find(|(id, _)| *id == editor_state.open_shop_id)
                    .map(|(_, name)| name.clone())
                    .unwrap_or_else(|| editor_state.open_shop_id.clone())
            };

            ui.horizontal(|ui| {
                ui.label("Shop:");
                if let Some(selected_id) = searchable_combobox(
                    ui,
                    &format!("{}_open_shop_selector", id_salt),
                    &current_label,
                    shops,
                    &mut editor_state.shop_search_buffer,
                ) {
                    editor_state.open_shop_id = selected_id;
                }
            });

            let can_add = !editor_state.open_shop_id.is_empty();
            if ui
                .add_enabled(can_add, egui::Button::new("Add Action"))
                .clicked()
                && let Some(action) = editor_state.build_action()
            {
                if let Some(idx) = editor_state.editing_index {
                    actions[idx] = action;
                } else {
                    actions.push(action);
                }
                editor_state.reset();
            }
        }
        ActionType::MoveEntity => {
            action_editor_forms::render_move_entity_form(ui, actions, editor_state, id_salt);
        }
        ActionType::CameraFollow => {
            action_editor_forms::render_camera_follow_form(ui, actions, editor_state, id_salt);
        }
        ActionType::CameraPan => {
            action_editor_forms::render_camera_pan_form(ui, actions, editor_state, id_salt);
        }
        ActionType::Wait => {
            action_editor_forms::render_wait_form(ui, actions, editor_state, id_salt);
        }
        ActionType::Jump => {
            action_editor_forms::render_jump_form(ui, actions, editor_state, id_salt);
        }
        ActionType::SetSpeed => {
            action_editor_forms::render_set_speed_form(ui, actions, editor_state, id_salt);
        }
    }

    // Render nested on_success/on_failure editors for reward actions when direction is Take.
    // This is placed after the form dispatch so it appears below the form fields.
    let is_reward_type = matches!(
        editor_state.action_type,
        ActionType::GiveCurrency
            | ActionType::GiveExperience
            | ActionType::GiveItem
            | ActionType::LearnAbility
            | ActionType::AddPartyMember
    );
    if is_reward_type && editor_state.reward_direction == TransferDirection::Take {
        render_reward_nested_editors(
            ui,
            editor_state,
            id_salt,
            map_entries,
            portrait_entries,
            depth,
            shops,
        );
    }
}

/// Renders the categorized, searchable action-type dropdown.
///
/// The dropdown contains a text filter at the top followed by one
/// `CollapsingHeader` per category (per Requirement 10). While a filter is
/// active, only actions whose display name matches the filter are shown and
/// empty categories are hidden. With an empty filter, every category is shown
/// expanded with all of its actions.
///
/// `depth` controls nesting: at depth >= 1, `StateCheck` and `Branch` are
/// excluded to prevent deep nesting. `shops` is used to disable `OpenShop`
/// when no shops exist.
fn render_action_type_dropdown(
    ui: &mut egui::Ui,
    editor_state: &mut ActionEditorState,
    id_salt: &str,
    depth: usize,
    shops: &[(String, String)],
) {
    let selected_text = action_type_display_name(editor_state.action_type);
    let filter_active = !editor_state.action_type_search.trim().is_empty();
    let shops_empty = shops.is_empty();

    egui::ComboBox::from_id_salt(format!("{}_action_type", id_salt))
        .selected_text(selected_text)
        .show_ui(ui, |ui| {
            // Search filter input at the top of the dropdown.
            ui.add(
                egui::TextEdit::singleline(&mut editor_state.action_type_search)
                    .hint_text("Filter actions…")
                    .desired_width(f32::INFINITY),
            );
            ui.separator();

            // Exclude StateCheck/Branch at nested depths.
            let include_predicate = |variant: ActionType| {
                if depth >= 1 {
                    !matches!(variant, ActionType::StateCheck | ActionType::Branch)
                } else {
                    true
                }
            };

            let categories =
                filter_action_categories(&editor_state.action_type_search, include_predicate);

            if categories.is_empty() {
                ui.label("No matching actions");
                return;
            }

            for (category_name, actions) in categories {
                // Categories default to expanded; when a filter is active we
                // force them open so matches are immediately visible.
                egui::CollapsingHeader::new(category_name)
                    .id_salt(format!("{}_cat_{}", id_salt, category_name))
                    .default_open(true)
                    .open(if filter_active { Some(true) } else { None })
                    .show(ui, |ui| {
                        for (variant, display_name) in actions {
                            // OpenShop is disabled (with a tooltip) when there are no shops.
                            if variant == ActionType::OpenShop && shops_empty {
                                ui.add_enabled(
                                    false,
                                    egui::Button::selectable(
                                        editor_state.action_type == variant,
                                        display_name,
                                    ),
                                )
                                .on_disabled_hover_text("Create at least one shop first");
                                continue;
                            }

                            let selected = editor_state.action_type == variant;
                            if ui.selectable_label(selected, display_name).clicked() {
                                editor_state.action_type = variant;
                            }
                        }
                    });
            }
        });
}

/// Renders nested on_success/on_failure action editors for the reward action currently being
/// configured in the editor state. Shown only when direction is Take.
fn render_reward_nested_editors(
    ui: &mut egui::Ui,
    editor_state: &mut ActionEditorState,
    id_salt: &str,
    map_entries: &[(String, String)],
    portrait_entries: &[(String, String)],
    depth: usize,
    shops: &[(String, String)],
) {
    ui.separator();

    // On Success Actions (optional)
    let on_success_count = editor_state.reward_on_success.len();
    let nested_salt_success = format!("{}_reward_on_success", id_salt);
    egui::CollapsingHeader::new(format!(
        "On Success Actions (optional) ({} actions)",
        on_success_count
    ))
    .id_salt(&nested_salt_success)
    .show(ui, |ui| {
        if editor_state.reward_on_success_editor.is_none() {
            editor_state.reward_on_success_editor = Some(Box::new(ActionEditorState::new_nested()));
        }
        if let Some(ref mut nested_editor) = editor_state.reward_on_success_editor {
            render_action_editor(
                ui,
                &mut editor_state.reward_on_success,
                nested_editor,
                &nested_salt_success,
                map_entries,
                portrait_entries,
                depth + 1,
                None,
                shops,
            );
        }
    });

    // On Failure Actions (required)
    let on_failure_count = editor_state.reward_on_failure.len();
    let nested_salt_failure = format!("{}_reward_on_failure", id_salt);
    egui::CollapsingHeader::new(format!(
        "On Failure Actions (required) ({} actions)",
        on_failure_count
    ))
    .id_salt(&nested_salt_failure)
    .show(ui, |ui| {
        if editor_state.reward_on_failure_editor.is_none() {
            editor_state.reward_on_failure_editor = Some(Box::new(ActionEditorState::new_nested()));
        }
        if let Some(ref mut nested_editor) = editor_state.reward_on_failure_editor {
            render_action_editor(
                ui,
                &mut editor_state.reward_on_failure,
                nested_editor,
                &nested_salt_failure,
                map_entries,
                portrait_entries,
                depth + 1,
                None,
                shops,
            );
        }
    });
}

/// Renders collapsible nested action editors for Branch and StateCheck items in the action list.
/// This allows visual editing of on_true/on_false branches inline.
fn render_nested_branch_editors(
    ui: &mut egui::Ui,
    actions: &mut [EventAction],
    id_salt: &str,
    map_entries: &[(String, String)],
    portrait_entries: &[(String, String)],
    shops: &[(String, String)],
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
                                    portrait_entries,
                                    1,
                                    None,
                                    shops,
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
                                portrait_entries,
                                1,
                                None,
                                shops,
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
                                    portrait_entries,
                                    1,
                                    None,
                                    shops,
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
                                portrait_entries,
                                1,
                                None,
                                shops,
                            );
                        }
                    });
                });
            }
            EventAction::ShowSelection { choices, .. } => {
                ui.indent(format!("selection_indent_{}", i), |ui| {
                    for (ci, choice) in choices.iter_mut().enumerate() {
                        let choice_action_count = choice.actions.len();
                        let nested_salt_choice =
                            format!("{}_selection_{}_choice_{}", id_salt, i, ci);

                        egui::CollapsingHeader::new(format!(
                            "  ↳ Choice {} ({} actions)",
                            ci + 1,
                            choice_action_count
                        ))
                        .id_salt(&nested_salt_choice)
                        .show(ui, |ui| {
                            let mut nested_editor = ActionEditorState::default();
                            render_action_editor(
                                ui,
                                &mut choice.actions,
                                &mut nested_editor,
                                &nested_salt_choice,
                                map_entries,
                                portrait_entries,
                                1,
                                None,
                                shops,
                            );
                        });
                    }
                });
            }
            // Reward action variants — show nested on_success/on_failure when direction is Take
            EventAction::GiveCurrency {
                direction,
                on_success,
                on_failure,
                ..
            }
            | EventAction::GiveExperience {
                direction,
                on_success,
                on_failure,
                ..
            }
            | EventAction::GiveItem {
                direction,
                on_success,
                on_failure,
                ..
            }
            | EventAction::LearnAbility {
                direction,
                on_success,
                on_failure,
                ..
            }
            | EventAction::AddPartyMember {
                direction,
                on_success,
                on_failure,
                ..
            } if *direction == TransferDirection::Take => {
                let on_success_count = on_success.len();
                let on_failure_count = on_failure.len();
                let nested_salt_success = format!("{}_reward_{}_on_success", id_salt, i);
                let nested_salt_failure = format!("{}_reward_{}_on_failure", id_salt, i);

                ui.indent(format!("reward_indent_{}", i), |ui| {
                    egui::CollapsingHeader::new(format!(
                        "  ↳ on_success ({} actions)",
                        on_success_count
                    ))
                    .id_salt(&nested_salt_success)
                    .show(ui, |ui| {
                        let mut nested_editor = ActionEditorState::default();
                        render_action_editor(
                            ui,
                            on_success,
                            &mut nested_editor,
                            &nested_salt_success,
                            map_entries,
                            portrait_entries,
                            1,
                            None,
                            shops,
                        );
                    });

                    egui::CollapsingHeader::new(format!(
                        "  ↳ on_failure ({} actions)",
                        on_failure_count
                    ))
                    .id_salt(&nested_salt_failure)
                    .show(ui, |ui| {
                        let mut nested_editor = ActionEditorState::default();
                        render_action_editor(
                            ui,
                            on_failure,
                            &mut nested_editor,
                            &nested_salt_failure,
                            map_entries,
                            portrait_entries,
                            1,
                            None,
                            shops,
                        );
                    });
                });
            }
            _ => {}
        }
    }
}
