use bevy::prelude::*;
use rpg_toolkit_common::{
    DialogTextData, EventAction, FadeType, PlayerAppearance, ScreenShakeMode, TransferDirection,
};
use std::collections::VecDeque;

use crate::components::{FadeOverlay, GameCamera, PlayerCharacter};
use crate::dialog::{
    DialogState, DialogTextRegistry, dialog_config_from_data, dialog_text_from_data,
};
use crate::effects::{
    compute_fade_opacity, compute_shake_offset, is_fade_complete, is_shake_complete,
};
use crate::events::{MapChanged, PlayerMoved, ShowDialog};
use crate::resources::{
    ActionQueue, CharacterProgressState, CurrencyState, FadeState, GameState, InventoryState,
    PartyState, RendererProjectData, RendererState, ScreenShakeState, WaitingFor,
};
use crate::systems::player::grid_to_world;
use crate::systems::selection::{ResolvedChoice, SelectionState};

/// Reacts to `PlayerMoved` events: collects event triggers from all layers at the
/// destination tile and populates the `ActionQueue` for sequential processing.
/// Does nothing if an `ActionQueue` already exists (sequence in progress).
pub fn check_triggers(
    mut player_moved: MessageReader<PlayerMoved>,
    project_data: Res<RendererProjectData>,
    renderer_state: Res<RendererState>,
    game_state: Res<GameState>,
    action_queue: Option<Res<ActionQueue>>,
    mut commands: Commands,
    mut player_query: Query<&mut PlayerCharacter>,
) {
    for event in player_moved.read() {
        // Apply elevation transition from tile attributes at the destination tile.
        // This runs regardless of whether an action queue exists, since elevation
        // transitions are a passive tile property (not an event action).
        if let Some(map_id) = &renderer_state.active_map_id
            && let Some(map) = project_data.project_file.maps.get(map_id)
        {
            let (x, y) = event.to;
            // Check all layers for a target_elevation at the destination tile
            for layer in &map.layers {
                if let Some(row) = layer.attributes.cells.get(y as usize)
                    && let Some(attrs) = row.get(x as usize)
                    && let Some(target_elev) = attrs.target_elevation
                {
                    // Update the player's elevation
                    if let Ok(mut player) = player_query.single_mut() {
                        player.elevation = target_elev;
                    }
                    // Only apply the first matching transition
                    break;
                }
            }
        }

        // If a sequence is already in progress, ignore new triggers
        if action_queue.is_some() {
            continue;
        }

        let Some(map_id) = &renderer_state.active_map_id else {
            continue;
        };
        let Some(map) = project_data.project_file.maps.get(map_id) else {
            continue;
        };

        let (x, y) = event.to;

        // Collect EventAction entries from all layers at the destination tile.
        // For each layer, evaluate conditional_triggers first (first match wins),
        // falling through to the default event_trigger if no condition matches.
        let mut actions = VecDeque::new();
        for layer in &map.layers {
            let Some(row) = layer.attributes.cells.get(y as usize) else {
                continue;
            };
            let Some(attrs) = row.get(x as usize) else {
                continue;
            };

            // Evaluate conditional_triggers in order — first match wins
            let mut matched_conditional = false;
            for ct in &attrs.conditional_triggers {
                if ct.condition.evaluate(&game_state.flags) {
                    for action in &ct.actions {
                        actions.push_back(action.clone());
                    }
                    matched_conditional = true;
                    break;
                }
            }

            // Fall through to default event_trigger if no conditional trigger matched
            if !matched_conditional {
                for action in &attrs.event_trigger {
                    actions.push_back(action.clone());
                }
            }
        }

        // If we collected any actions, insert the ActionQueue resource
        if !actions.is_empty() {
            commands.insert_resource(ActionQueue {
                actions,
                waiting_for: WaitingFor::Nothing,
            });
        }
    }
}

/// Advances the action queue: fires the next action in the sequence.
/// Waits for blocking actions (dialog, screen shake, fade) to complete before advancing.
#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
pub fn advance_action_queue(
    mut commands: Commands,
    action_queue: Option<ResMut<ActionQueue>>,
    dialog_state: Option<Res<DialogState>>,
    selection_state: Option<Res<SelectionState>>,
    registry: Option<Res<DialogTextRegistry>>,
    shake_state: Option<Res<ScreenShakeState>>,
    fade_state: Option<Res<FadeState>>,
    mut game_state: Option<ResMut<GameState>>,
    mut renderer_state: ResMut<RendererState>,
    project_data: Option<Res<RendererProjectData>>,
    asset_server: Res<AssetServer>,
    mut show_dialog: MessageWriter<ShowDialog>,
    mut camera_query: Query<&mut Transform, With<GameCamera>>,
    mut player_query: Query<&mut Visibility, With<PlayerCharacter>>,
    fade_overlay_query: Query<Entity, With<FadeOverlay>>,
    mut reward_state: (
        Option<ResMut<CurrencyState>>,
        Option<ResMut<CharacterProgressState>>,
        Option<ResMut<PartyState>>,
        Option<ResMut<InventoryState>>,
    ),
) {
    // Destructure reward state tuple for convenient access
    let (
        ref mut currency_state,
        ref mut character_progress,
        ref mut party_state,
        ref mut inventory_state,
    ) = reward_state;

    let Some(mut queue) = action_queue else {
        return;
    };

    // If we're waiting for a blocking action to complete...
    match queue.waiting_for {
        WaitingFor::Dialog => {
            if dialog_state.is_some() {
                return;
            }
            queue.waiting_for = WaitingFor::Nothing;
            queue.actions.pop_front();
        }
        WaitingFor::ScreenShake => {
            if shake_state.is_some() {
                return;
            }
            queue.waiting_for = WaitingFor::Nothing;
            queue.actions.pop_front();
        }
        WaitingFor::Fade => {
            if fade_state.is_some() {
                return;
            }
            queue.waiting_for = WaitingFor::Nothing;
            queue.actions.pop_front();
        }
        WaitingFor::Selection => {
            // Selection prompt is active; block until SelectionState is removed.
            if selection_state.is_some() {
                return;
            }
            queue.waiting_for = WaitingFor::Nothing;
            queue.actions.pop_front();
        }
        WaitingFor::Nothing => {}
    }

    // Process actions in a loop to handle non-blocking actions consecutively
    'action_loop: loop {
        if queue.actions.is_empty() {
            commands.remove_resource::<ActionQueue>();
            return;
        }

        let action = queue.actions.front().unwrap().clone();
        match action {
            EventAction::ShowDialog { text, config } => {
                let dialog_text = dialog_text_from_data(&text);
                let dialog_config = dialog_config_from_data(&config);

                if let rpg_toolkit_common::DialogTextData::Id(ref id) = text {
                    let has_entry = registry.as_ref().is_some_and(|reg| reg.get(id).is_some());
                    if !has_entry {
                        warn!(
                            "ShowDialog text ID '{}' not found in DialogTextRegistry; skipping action",
                            id
                        );
                        queue.actions.pop_front();
                        continue;
                    }
                }

                show_dialog.write(ShowDialog {
                    text: dialog_text,
                    config: dialog_config,
                });
                queue.waiting_for = WaitingFor::Dialog;
                return;
            }
            EventAction::JumpTo {
                target_map_id,
                target_x,
                target_y,
                target_elevation,
            } => {
                renderer_state.pending_map_change = Some(target_map_id);
                renderer_state.pending_target_coords = Some((target_x, target_y));
                renderer_state.pending_target_elevation = target_elevation;
                // Pop the JumpTo action itself; remaining actions (e.g. FadeIn)
                // stay in the queue and will continue processing after the map loads.
                queue.actions.pop_front();
                return;
            }
            EventAction::ScreenShake {
                intensity,
                duration,
                mode,
            } => {
                match mode {
                    ScreenShakeMode::Timed => {
                        if duration <= 0.0 {
                            // Instant complete — just pop and continue
                            queue.actions.pop_front();
                            continue;
                        }
                        commands.insert_resource(ScreenShakeState {
                            intensity,
                            mode,
                            duration,
                            elapsed: 0.0,
                        });
                        queue.waiting_for = WaitingFor::ScreenShake;
                        return;
                    }
                    ScreenShakeMode::Continuous => {
                        commands.insert_resource(ScreenShakeState {
                            intensity,
                            mode,
                            duration,
                            elapsed: 0.0,
                        });
                        // Non-blocking — pop and continue
                        queue.actions.pop_front();
                        continue;
                    }
                }
            }
            EventAction::StopScreenShake => {
                commands.remove_resource::<ScreenShakeState>();
                // Reset camera offset
                if let Ok(mut cam_tf) = camera_query.single_mut() {
                    // The update_camera system will reposition next frame;
                    // just zero out any shake offset by letting it run naturally.
                    // We don't need to do anything special here since removing
                    // ScreenShakeState stops the shake system from applying offsets.
                    let _ = &mut cam_tf; // acknowledge the query
                }
                queue.actions.pop_front();
                continue;
            }
            EventAction::FadeTransition {
                fade_type,
                duration,
                color,
            } => {
                if duration <= 0.0 {
                    // Instant — apply final state
                    match fade_type {
                        FadeType::FadeOut => {
                            // Spawn overlay at full opacity
                            commands.spawn((
                                Node {
                                    width: Val::Percent(100.0),
                                    height: Val::Percent(100.0),
                                    position_type: PositionType::Absolute,
                                    ..default()
                                },
                                BackgroundColor(Color::srgba(color[0], color[1], color[2], 1.0)),
                                ZIndex(999),
                                FadeOverlay,
                            ));
                        }
                        FadeType::FadeIn => {
                            // Despawn any existing overlay
                            for entity in fade_overlay_query.iter() {
                                commands.entity(entity).despawn();
                            }
                        }
                    }
                    queue.actions.pop_front();
                    continue;
                }

                // Spawn the fade overlay entity
                let initial_alpha = match fade_type {
                    FadeType::FadeOut => 0.0,
                    FadeType::FadeIn => 1.0,
                };
                commands.spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        position_type: PositionType::Absolute,
                        ..default()
                    },
                    BackgroundColor(Color::srgba(color[0], color[1], color[2], initial_alpha)),
                    ZIndex(999),
                    FadeOverlay,
                ));

                commands.insert_resource(FadeState {
                    fade_type,
                    duration,
                    elapsed: 0.0,
                    color,
                });
                queue.waiting_for = WaitingFor::Fade;
                return;
            }
            EventAction::SetState { key, value } => {
                if key.is_empty() {
                    warn!("SetState with empty key; skipping action");
                    queue.actions.pop_front();
                    continue;
                }
                if let Some(ref mut gs) = game_state {
                    gs.flags.insert(key, value);
                } else {
                    // GameState resource doesn't exist yet — insert it
                    let mut flags = std::collections::HashMap::new();
                    flags.insert(key, value);
                    commands.insert_resource(GameState { flags });
                }
                queue.actions.pop_front();
                continue;
            }
            EventAction::SetPlayerAppearance { appearance } => {
                match appearance {
                    PlayerAppearance::Hidden => {
                        if let Ok(mut vis) = player_query.single_mut() {
                            *vis = Visibility::Hidden;
                        }
                    }
                    PlayerAppearance::Spritesheet { path } => {
                        warn!(
                            "SetPlayerAppearance(Spritesheet) with path '{}': spritesheet swapping is not yet fully implemented",
                            path
                        );
                        // Ensure player is visible
                        if let Ok(mut vis) = player_query.single_mut() {
                            *vis = Visibility::Inherited;
                        }
                    }
                    PlayerAppearance::Default => {
                        if let Ok(mut vis) = player_query.single_mut() {
                            *vis = Visibility::Inherited;
                        }
                    }
                }
                queue.actions.pop_front();
                continue;
            }
            EventAction::StateCheck {
                key,
                value,
                on_true,
                on_false,
            } => {
                let matched = if let Some(ref gs) = game_state {
                    match value {
                        Some(ref expected) => gs.flags.get(&key) == Some(expected),
                        None => {
                            // Check key existence only
                            gs.flags.contains_key(&key)
                        }
                    }
                } else {
                    // No GameState resource — state is effectively empty
                    false
                };

                // Pop the StateCheck action
                queue.actions.pop_front();

                // Push the matching branch to the front so it executes next
                let branch = if matched { on_true } else { on_false };
                for action in branch.into_iter().rev() {
                    queue.actions.push_front(action);
                }
                continue;
            }
            EventAction::Branch {
                condition,
                on_true,
                on_false,
            } => {
                let empty = std::collections::HashMap::new();
                let flags = game_state.as_ref().map(|gs| &gs.flags).unwrap_or(&empty);
                let matched = condition.evaluate(flags);

                // Pop the Branch action
                queue.actions.pop_front();

                // Push the matching branch to the front so it executes next
                let branch = if matched { on_true } else { on_false };
                for action in branch.into_iter().rev() {
                    queue.actions.push_front(action);
                }
                continue;
            }
            EventAction::ShowSelection {
                prompt,
                config: selection_config,
                choices,
            } => {
                // If a selection is already active, don't spawn another
                if selection_state.is_some() {
                    return;
                }

                // Resolve prompt text
                let resolved_prompt = match &prompt {
                    DialogTextData::Inline(text) => text.clone(),
                    DialogTextData::Id(id) => {
                        let resolved = registry.as_ref().and_then(|reg| reg.get(id));
                        match resolved {
                            Some(text) => text.to_string(),
                            None => {
                                warn!(
                                    "ShowSelection prompt ID '{}' not found in DialogTextRegistry; skipping action",
                                    id
                                );
                                queue.actions.pop_front();
                                continue;
                            }
                        }
                    }
                };

                // Resolve all choice labels
                let mut resolved_choices = Vec::with_capacity(choices.len());
                for choice in &choices {
                    let label = match &choice.label {
                        DialogTextData::Inline(text) => text.clone(),
                        DialogTextData::Id(id) => {
                            let resolved = registry.as_ref().and_then(|reg| reg.get(id));
                            match resolved {
                                Some(text) => text.to_string(),
                                None => {
                                    warn!(
                                        "ShowSelection choice label ID '{}' not found in DialogTextRegistry; skipping action",
                                        id
                                    );
                                    queue.actions.pop_front();
                                    continue 'action_loop;
                                }
                            }
                        }
                    };
                    resolved_choices.push(ResolvedChoice {
                        label,
                        actions: choice.actions.clone(),
                    });
                }

                let choice_count = resolved_choices.len();

                // Resolve face portrait path from project data
                let resolved_portrait_path = selection_config.face_portrait.as_ref().and_then(|portrait_id| {
                    let resolved = project_data
                        .as_ref()
                        .and_then(|pd| pd.project_file.face_portraits.get(portrait_id))
                        .cloned();
                    if resolved.is_none() && !portrait_id.is_empty() {
                        warn!(
                            "Face portrait ID '{}' not found in project face_portraits registry; skipping portrait",
                            portrait_id
                        );
                    }
                    resolved
                });

                // Spawn selection UI
                spawn_selection_ui(
                    &mut commands,
                    &resolved_prompt,
                    &resolved_choices,
                    &selection_config,
                    resolved_portrait_path.as_deref(),
                    &asset_server,
                );

                // Insert SelectionState resource
                commands.insert_resource(SelectionState {
                    cursor_index: 0,
                    choice_count,
                    choices: resolved_choices,
                });

                queue.waiting_for = WaitingFor::Selection;
                return;
            }
            // Reward action: GiveCurrency
            EventAction::GiveCurrency {
                amount,
                direction,
                on_success,
                on_failure,
            } => {
                queue.actions.pop_front();
                if let Some(currency) = currency_state {
                    match direction {
                        TransferDirection::Give => {
                            currency.balance = currency.balance.saturating_add(amount);
                        }
                        TransferDirection::Take => {
                            let branch = if currency.balance >= amount {
                                currency.balance -= amount;
                                on_success
                            } else {
                                on_failure
                            };
                            for action in branch.into_iter().rev() {
                                queue.actions.push_front(action);
                            }
                        }
                    }
                }
                continue;
            }
            // Reward action: LearnAbility
            EventAction::LearnAbility {
                ability_id,
                target,
                direction,
                on_success,
                on_failure,
            } => {
                queue.actions.pop_front();

                // Check if ability exists in project's abilities registry
                let ability_exists = project_data.as_ref().is_some_and(|pd| {
                    pd.project_file
                        .abilities
                        .abilities
                        .contains_key(&ability_id)
                });

                if !ability_exists {
                    warn!(
                        "LearnAbility ability_id '{}' not found in AbilityRegistry; skipping",
                        ability_id
                    );
                    continue;
                }

                if let Some(progress) = character_progress {
                    match direction {
                        TransferDirection::Give => {
                            if let Some(entry) = progress.characters.get_mut(&target) {
                                if !entry.learned_abilities.contains(&ability_id) {
                                    entry.learned_abilities.push(ability_id);
                                }
                                // else: already known, no-op
                            } else {
                                warn!(
                                    "LearnAbility target '{}' not found in CharacterProgressState; skipping",
                                    target
                                );
                            }
                        }
                        TransferDirection::Take => {
                            let branch = if let Some(entry) = progress.characters.get_mut(&target) {
                                if let Some(pos) = entry
                                    .learned_abilities
                                    .iter()
                                    .position(|a| a == &ability_id)
                                {
                                    entry.learned_abilities.remove(pos);
                                    on_success
                                } else {
                                    on_failure
                                }
                            } else {
                                warn!(
                                    "LearnAbility target '{}' not found in CharacterProgressState; skipping",
                                    target
                                );
                                // Target not found — treat as failure
                                on_failure
                            };
                            for action in branch.into_iter().rev() {
                                queue.actions.push_front(action);
                            }
                        }
                    }
                }
                continue;
            }
            // Reward action: AddPartyMember
            EventAction::AddPartyMember {
                character_id,
                direction,
                on_success,
                on_failure,
            } => {
                queue.actions.pop_front();

                // Check if character exists in project's character registry
                let character_exists = project_data.as_ref().is_some_and(|pd| {
                    pd.project_file
                        .characters
                        .characters
                        .contains_key(&character_id)
                });

                if !character_exists {
                    warn!(
                        "AddPartyMember character_id '{}' not found in CharacterRegistry; skipping",
                        character_id
                    );
                    continue;
                }

                if let Some(party) = party_state {
                    match direction {
                        TransferDirection::Give => {
                            if !party.members.contains(&character_id) {
                                party.members.push(character_id);
                            }
                            // else: already in party, no-op
                        }
                        TransferDirection::Take => {
                            let branch = if let Some(pos) =
                                party.members.iter().position(|id| id == &character_id)
                            {
                                party.members.remove(pos);
                                on_success
                            } else {
                                on_failure
                            };
                            for action in branch.into_iter().rev() {
                                queue.actions.push_front(action);
                            }
                        }
                    }
                }
                continue;
            }
            // GiveItem reward action
            EventAction::GiveItem {
                item_id,
                quantity,
                direction,
                on_success,
                on_failure,
            } => {
                queue.actions.pop_front();

                // Look up item definition from project data for stackability info
                let item_def = project_data
                    .as_ref()
                    .and_then(|pd| pd.project_file.items.items.get(&item_id));

                if item_def.is_none() {
                    warn!(
                        "GiveItem item_id '{}' not found in ItemRegistry; skipping",
                        item_id
                    );
                    continue;
                }
                let item_def = item_def.unwrap();

                if let Some(inventory) = inventory_state {
                    match direction {
                        TransferDirection::Give => {
                            let success =
                                if let Some(current_qty) = inventory.items.get_mut(&item_id) {
                                    if !item_def.stackable {
                                        // Unstackable item already owned — failure
                                        false
                                    } else if *current_qty >= item_def.stack_limit {
                                        // Already at stack cap — failure
                                        false
                                    } else {
                                        // Add up to stack_limit
                                        *current_qty =
                                            (*current_qty + quantity).min(item_def.stack_limit);
                                        true
                                    }
                                } else {
                                    // New item — always succeeds
                                    inventory
                                        .items
                                        .insert(item_id, quantity.min(item_def.stack_limit));
                                    true
                                };

                            let branch = if success { on_success } else { on_failure };
                            for action in branch.into_iter().rev() {
                                queue.actions.push_front(action);
                            }
                        }
                        TransferDirection::Take => {
                            let branch = if let Some(&current_qty) = inventory.items.get(&item_id) {
                                if current_qty >= quantity {
                                    let new_qty = current_qty - quantity;
                                    if new_qty == 0 {
                                        inventory.items.remove(&item_id);
                                    } else {
                                        inventory.items.insert(item_id, new_qty);
                                    }
                                    on_success
                                } else {
                                    on_failure
                                }
                            } else {
                                on_failure
                            };
                            for action in branch.into_iter().rev() {
                                queue.actions.push_front(action);
                            }
                        }
                    }
                }
                continue;
            }
            // Reward action: GiveExperience
            EventAction::GiveExperience {
                amount,
                target,
                direction,
                on_success,
                on_failure,
            } => {
                queue.actions.pop_front();
                if let Some(progress) = character_progress {
                    match direction {
                        TransferDirection::Give => {
                            match target {
                                Some(char_id) => {
                                    if let Some(entry) = progress.characters.get_mut(&char_id) {
                                        entry.experience = entry.experience.saturating_add(amount);
                                    } else {
                                        warn!(
                                            "GiveExperience target '{}' not found in CharacterProgressState; skipping",
                                            char_id
                                        );
                                    }
                                }
                                None => {
                                    // Give to all party members
                                    if let Some(party) = party_state {
                                        for member_id in &party.members {
                                            if let Some(entry) =
                                                progress.characters.get_mut(member_id)
                                            {
                                                entry.experience =
                                                    entry.experience.saturating_add(amount);
                                            } else {
                                                warn!(
                                                    "GiveExperience party member '{}' not found in CharacterProgressState; skipping member",
                                                    member_id
                                                );
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        TransferDirection::Take => {
                            let sufficient = match target {
                                Some(ref char_id) => progress
                                    .characters
                                    .get(char_id)
                                    .is_some_and(|e| e.experience >= amount),
                                None => {
                                    // All party members must have sufficient (atomic check)
                                    party_state.as_ref().is_some_and(|party| {
                                        party.members.iter().all(|id| {
                                            progress
                                                .characters
                                                .get(id)
                                                .is_some_and(|e| e.experience >= amount)
                                        })
                                    })
                                }
                            };
                            let branch = if sufficient {
                                // Apply subtraction
                                match target {
                                    Some(ref char_id) => {
                                        if let Some(entry) = progress.characters.get_mut(char_id) {
                                            entry.experience -= amount;
                                        }
                                    }
                                    None => {
                                        if let Some(party) = party_state {
                                            for member_id in &party.members {
                                                if let Some(entry) =
                                                    progress.characters.get_mut(member_id)
                                                {
                                                    entry.experience -= amount;
                                                }
                                            }
                                        }
                                    }
                                }
                                on_success
                            } else {
                                on_failure
                            };
                            for action in branch.into_iter().rev() {
                                queue.actions.push_front(action);
                            }
                        }
                    }
                }
                continue;
            }
        }
    }
}

/// Spawns the selection UI entities.
/// Creates a styled UI root with the prompt text, choice labels, cursor indicator,
/// and optional face portrait — consistent with the standard dialog box styling.
fn spawn_selection_ui(
    commands: &mut Commands,
    prompt: &str,
    choices: &[ResolvedChoice],
    config: &rpg_toolkit_common::map::DialogConfigData,
    portrait_path: Option<&str>,
    asset_server: &AssetServer,
) {
    use crate::systems::selection::{SelectionBox, SelectionCursor, SelectionLabel};
    use rpg_toolkit_common::map::DialogPositionData;

    let (justify_content, align_items) = match config.position {
        DialogPositionData::Top => (JustifyContent::FlexStart, AlignItems::Center),
        DialogPositionData::Center => (JustifyContent::Center, AlignItems::Center),
        DialogPositionData::Bottom => (JustifyContent::FlexEnd, AlignItems::Center),
    };

    let padding = match config.position {
        DialogPositionData::Top => UiRect::top(Val::Px(20.0)),
        DialogPositionData::Center => UiRect::DEFAULT,
        DialogPositionData::Bottom => UiRect::bottom(Val::Px(20.0)),
    };

    // Spawn root selection container with SelectionBox marker
    commands
        .spawn((
            SelectionBox,
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content,
                align_items,
                padding,
                ..default()
            },
            GlobalZIndex(100),
        ))
        .with_children(|parent| {
            // Inner panel: auto height, semi-transparent background, border, overflow clip
            parent
                .spawn((
                    Node {
                        width: Val::Percent(80.0),
                        height: Val::Auto,
                        padding: UiRect::all(Val::Px(16.0)),
                        border_radius: BorderRadius::all(Val::Px(8.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        overflow: Overflow::clip(),
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::FlexStart,
                        column_gap: Val::Px(12.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.75)),
                    BorderColor::all(Color::srgba(0.85, 0.85, 0.85, 1.0)),
                ))
                .with_children(|panel| {
                    // Face portrait (if configured)
                    if let Some(path) = portrait_path {
                        let portrait_handle: Handle<Image> = asset_server.load(path.to_string());
                        panel.spawn((
                            ImageNode {
                                image: portrait_handle,
                                ..default()
                            },
                            Node {
                                width: Val::Px(64.0),
                                height: Val::Px(64.0),
                                flex_shrink: 0.0,
                                ..default()
                            },
                        ));
                    }

                    // Content container (prompt text + choice list)
                    panel
                        .spawn(Node {
                            flex_grow: 1.0,
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(8.0),
                            ..default()
                        })
                        .with_children(|content| {
                            // Prompt text
                            content.spawn((
                                Text::new(prompt.to_string()),
                                TextColor(Color::WHITE),
                                TextFont {
                                    font_size: 20.0,
                                    ..default()
                                },
                            ));

                            // Vertical choice list
                            content
                                .spawn(Node {
                                    flex_direction: FlexDirection::Column,
                                    row_gap: Val::Px(4.0),
                                    ..default()
                                })
                                .with_children(|choice_list| {
                                    for (i, choice) in choices.iter().enumerate() {
                                        // Each choice row: cursor + label
                                        choice_list
                                            .spawn(Node {
                                                flex_direction: FlexDirection::Row,
                                                align_items: AlignItems::Center,
                                                column_gap: Val::Px(8.0),
                                                ..default()
                                            })
                                            .with_children(|row| {
                                                // Cursor indicator "▶"
                                                let cursor_visibility = if i == 0 {
                                                    Visibility::Inherited
                                                } else {
                                                    Visibility::Hidden
                                                };
                                                row.spawn((
                                                    SelectionCursor,
                                                    SelectionLabel { index: i },
                                                    Text::new("▶".to_string()),
                                                    TextColor(Color::WHITE),
                                                    TextFont {
                                                        font_size: 20.0,
                                                        ..default()
                                                    },
                                                    cursor_visibility,
                                                ));

                                                // Choice label text
                                                row.spawn((
                                                    Text::new(choice.label.clone()),
                                                    TextColor(Color::WHITE),
                                                    TextFont {
                                                        font_size: 20.0,
                                                        ..default()
                                                    },
                                                ));
                                            });
                                    }
                                });
                        });
                });
        });
}

/// Handles a pending map change: fires `MapChanged`, updates active map,
/// clamps target coordinates, repositions the player, and clears the pending state.
/// Also cleans up any active screen shake effect.
pub fn handle_map_change(
    mut commands: Commands,
    mut renderer_state: ResMut<RendererState>,
    project_data: Res<RendererProjectData>,
    shake_state: Option<Res<ScreenShakeState>>,
    mut map_changed: MessageWriter<MapChanged>,
    mut query: Query<(&mut PlayerCharacter, &mut Transform, &mut Sprite)>,
) {
    let Some(new_map_id) = renderer_state.pending_map_change.take() else {
        return;
    };
    let target_coords = renderer_state.pending_target_coords.take();
    let target_elevation = renderer_state.pending_target_elevation.take();

    let Some(new_map) = project_data.project_file.maps.get(&new_map_id) else {
        warn!(
            "Pending map change to '{}' but map not found; ignoring",
            new_map_id
        );
        return;
    };

    // Clean up active screen shake on map change
    if shake_state.is_some() {
        commands.remove_resource::<ScreenShakeState>();
    }

    let previous_map_id = renderer_state.active_map_id.clone();
    renderer_state.active_map_id = Some(new_map_id.clone());

    // Clamp target coordinates to new map bounds
    let (tx, ty) = target_coords.unwrap_or((0, 0));
    let clamped_x = tx.min(new_map.width.saturating_sub(1));
    let clamped_y = ty.min(new_map.height.saturating_sub(1));

    // Reposition the player
    for (mut player, mut transform, mut sprite) in query.iter_mut() {
        player.grid_x = clamped_x;
        player.grid_y = clamped_y;
        player.move_animation = None; // Cancel any in-progress animation

        // Apply target elevation if specified, otherwise preserve current elevation
        if let Some(elev) = target_elevation {
            player.elevation = elev;
        }

        let world_pos = grid_to_world(
            clamped_x,
            clamped_y,
            new_map.tile_width,
            new_map.tile_height,
        );
        let z = new_map.layers.len() as f32 + 1.0;

        // Compute sprite scale and Y offset for the new map's tile dimensions.
        // Render at 1:1 pixel scale (no shrinking) — the character sprite is
        // designed to be larger than the tile and overlap neighbors.
        let (sprite_scale, y_offset) = project_data
            .project_file
            .player_spritesheet
            .as_ref()
            .and_then(|ss_id| project_data.project_file.spritesheets.get(ss_id))
            .map(|ss| {
                let scale = 1.0_f32;
                let scaled_height = ss.sprite_height as f32 * scale;
                let offset = (scaled_height - new_map.tile_height as f32) / 2.0;
                (scale, offset)
            })
            .unwrap_or((1.0, 0.0));

        transform.translation = Vec3::new(world_pos.x, world_pos.y + y_offset, z);
        transform.scale = Vec3::splat(sprite_scale);

        // Only set custom_size for non-spritesheet players (solid-color fallback).
        // Spritesheet players use transform.scale to fit the tile; setting custom_size
        // would double-scale them.
        if sprite.texture_atlas.is_none() {
            sprite.custom_size = Some(Vec2::new(
                new_map.tile_width as f32,
                new_map.tile_height as f32,
            ));
        }
    }

    map_changed.write(MapChanged {
        previous_map_id,
        new_map_id,
    });
}

/// Runs each frame while `ScreenShakeState` is present.
/// Increments elapsed time, checks for completion, and applies shake offset to camera.
pub fn screen_shake_system(
    mut commands: Commands,
    time: Res<Time>,
    mut shake_state: Option<ResMut<ScreenShakeState>>,
    mut camera_query: Query<&mut Transform, With<GameCamera>>,
) {
    let Some(ref mut state) = shake_state else {
        return;
    };

    state.elapsed += time.delta_secs();

    if is_shake_complete(state.elapsed, state.duration, state.mode) {
        // Shake is done — remove state and reset camera
        commands.remove_resource::<ScreenShakeState>();

        // Camera will be repositioned by update_camera next frame
        return;
    }

    // Generate deterministic pseudo-random seeds from elapsed time
    let seed_x = (state.elapsed * 123.456).fract();
    let seed_y = (state.elapsed * 789.012).fract();

    let (dx, dy) = compute_shake_offset(state.intensity, seed_x, seed_y);

    // Apply offset to camera (this system runs after update_camera,
    // so the offset is applied on top of the base camera position)
    if let Ok(mut cam_tf) = camera_query.single_mut() {
        cam_tf.translation.x += dx;
        cam_tf.translation.y += dy;
    }
}

/// Runs each frame while `FadeState` is present.
/// Increments elapsed time, updates overlay opacity, and handles completion.
pub fn fade_system(
    mut commands: Commands,
    time: Res<Time>,
    mut fade_state: Option<ResMut<FadeState>>,
    mut overlay_query: Query<(Entity, &mut BackgroundColor), With<FadeOverlay>>,
) {
    let Some(ref mut state) = fade_state else {
        return;
    };

    state.elapsed += time.delta_secs();

    let opacity = compute_fade_opacity(state.elapsed, state.duration, state.fade_type);

    // Update overlay color alpha
    for (_, mut bg_color) in overlay_query.iter_mut() {
        bg_color.0 = Color::srgba(state.color[0], state.color[1], state.color[2], opacity);
    }

    if is_fade_complete(state.elapsed, state.duration) {
        let fade_type = state.fade_type;

        // Remove the FadeState resource
        commands.remove_resource::<FadeState>();

        match fade_type {
            FadeType::FadeOut => {
                // Leave overlay at full opacity (screen stays covered)
            }
            FadeType::FadeIn => {
                // Despawn the overlay entity
                for (entity, _) in overlay_query.iter() {
                    commands.entity(entity).despawn();
                }
            }
        }
    }
}
