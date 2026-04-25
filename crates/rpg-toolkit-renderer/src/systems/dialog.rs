use bevy::prelude::*;

use crate::dialog::{
    DialogBox, DialogConfig, DialogPosition, DialogState, DialogText, DialogTextNode,
    DialogTextRegistry, compute_visible_chars,
};
use crate::events::ShowDialog;

/// Reads `ShowDialog` messages, resolves text, spawns dialog UI, and inserts `DialogState`.
pub fn handle_dialog_event(
    mut show_dialog: MessageReader<ShowDialog>,
    dialog_state: Option<Res<DialogState>>,
    registry: Option<Res<DialogTextRegistry>>,
    mut commands: Commands,
) {
    for event in show_dialog.read() {
        // If a dialog is already active, ignore new events
        if dialog_state.is_some() {
            debug!("Dialog already active, ignoring ShowDialog event");
            return;
        }

        // Resolve the text content
        let resolved_text = match &event.text {
            DialogText::Inline(text) => text.clone(),
            DialogText::Id(id) => {
                let Some(reg) = &registry else {
                    warn!(
                        "ShowDialog with text ID '{}' but no DialogTextRegistry is present; ignoring",
                        id
                    );
                    continue;
                };
                let Some(text) = reg.get(id) else {
                    warn!(
                        "ShowDialog text ID '{}' not found in DialogTextRegistry; ignoring",
                        id
                    );
                    continue;
                };
                text.to_string()
            }
        };

        let total_chars = resolved_text.chars().count();
        let text_speed = event.config.text_speed;
        let fully_revealed = text_speed <= 0.0 || total_chars == 0;

        // Spawn the dialog UI
        spawn_dialog_ui(&mut commands, &resolved_text, &event.config);

        // Insert the DialogState resource
        commands.insert_resource(DialogState {
            full_text: resolved_text,
            total_chars,
            chars_revealed: if fully_revealed { total_chars } else { 0 },
            fully_revealed,
            elapsed: 0.0,
            text_speed,
            movement_blocked: event.config.movement_block,
        });

        // Only handle the first event
        return;
    }
}

/// Spawns the dialog box UI entities.
fn spawn_dialog_ui(commands: &mut Commands, text: &str, config: &DialogConfig) {
    let (justify_content, align_items) = match config.position {
        DialogPosition::Top => (JustifyContent::FlexStart, AlignItems::Center),
        DialogPosition::Center => (JustifyContent::Center, AlignItems::Center),
        DialogPosition::Bottom => (JustifyContent::FlexEnd, AlignItems::Center),
    };

    let padding = match config.position {
        DialogPosition::Top => UiRect::top(Val::Px(20.0)),
        DialogPosition::Center => UiRect::DEFAULT,
        DialogPosition::Bottom => UiRect::bottom(Val::Px(20.0)),
    };

    // Determine initial visible text based on text_speed
    let visible_text = if config.text_speed <= 0.0 || text.is_empty() {
        text.to_string()
    } else {
        String::new()
    };

    // Spawn root dialog container with DialogBox marker
    // flex_direction: Column makes justify_content control the vertical axis,
    // so Top/Center/Bottom position the dialog box vertically as intended.
    commands
        .spawn((
            DialogBox,
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
            // Inner dialog panel with semi-transparent background
            parent
                .spawn((
                    Node {
                        width: Val::Percent(80.0),
                        padding: UiRect::all(Val::Px(16.0)),
                        border_radius: BorderRadius::all(Val::Px(8.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.75)),
                ))
                .with_children(|panel| {
                    // Text node
                    panel.spawn((
                        DialogTextNode,
                        Text::new(visible_text),
                        TextColor(Color::WHITE),
                        TextFont {
                            font_size: 20.0,
                            ..default()
                        },
                    ));
                });
        });
}

/// Reads Space/Enter to advance or dismiss the dialog.
///
/// If the typewriter is still animating, completes it instantly.
/// If the text is fully revealed, dismisses the dialog by despawning all
/// `DialogBox` entities and removing the `DialogState` resource.
pub fn handle_dialog_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut dialog_state: Option<ResMut<DialogState>>,
    dialog_entities: Query<Entity, With<DialogBox>>,
    mut text_query: Query<&mut Text, With<DialogTextNode>>,
    mut commands: Commands,
) {
    // Only respond to Space or Enter
    if !keyboard.just_pressed(KeyCode::Space) && !keyboard.just_pressed(KeyCode::Enter) {
        return;
    }

    let Some(ref mut state) = dialog_state else {
        return;
    };

    if !state.fully_revealed {
        // Complete the typewriter effect instantly
        state.chars_revealed = state.total_chars;
        state.fully_revealed = true;

        // Update the text node to show the full text
        let full_text: String = state.full_text.clone();
        for mut text in text_query.iter_mut() {
            **text = full_text.clone();
        }
    } else {
        // Dismiss the dialog: despawn all DialogBox entities and remove DialogState
        for entity in dialog_entities.iter() {
            commands.entity(entity).despawn();
        }
        commands.remove_resource::<DialogState>();
    }
}

/// Advances the typewriter effect each frame.
///
/// Each frame, increments elapsed time, recomputes the number of visible characters,
/// and updates the `Text` component on the `DialogTextNode` entity to show only
/// the revealed portion of the full text.
pub fn update_dialog_typewriter(
    time: Res<Time>,
    mut dialog_state: Option<ResMut<DialogState>>,
    mut text_query: Query<&mut Text, With<DialogTextNode>>,
) {
    let Some(ref mut state) = dialog_state else {
        return;
    };

    // Handle instant reveal when text_speed <= 0
    if state.text_speed <= 0.0 {
        state.chars_revealed = state.total_chars;
        state.fully_revealed = true;
    } else {
        // Advance elapsed time
        state.elapsed += time.delta_secs();

        // Compute visible characters using the pure function
        state.chars_revealed =
            compute_visible_chars(state.elapsed, state.text_speed, state.total_chars);
        state.fully_revealed = state.chars_revealed >= state.total_chars;
    }

    // Update the Text component on the DialogTextNode entity
    let visible_text: String = state.full_text.chars().take(state.chars_revealed).collect();
    for mut text in text_query.iter_mut() {
        **text = visible_text.clone();
    }
}
