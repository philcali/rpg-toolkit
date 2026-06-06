use bevy::prelude::*;
use rpg_toolkit_common::map::EventAction;

use crate::input::MovementIntent;
use crate::resources::{ActionQueue, WaitingFor};

/// Tracks an active selection prompt. Present only while the prompt is displayed.
#[derive(Resource)]
pub struct SelectionState {
    /// Index of the currently focused choice (0-based).
    pub cursor_index: usize,
    /// Total number of choices available.
    pub choice_count: usize,
    /// The resolved choice data (labels already resolved from registry).
    pub choices: Vec<ResolvedChoice>,
}

/// A choice with its label resolved to a display string.
pub struct ResolvedChoice {
    pub label: String,
    pub actions: Vec<EventAction>,
}

/// Root UI entity for the selection prompt.
#[derive(Component)]
pub struct SelectionBox;

/// The "▶" cursor indicator.
#[derive(Component)]
pub struct SelectionCursor;

/// A choice label text entity.
#[derive(Component)]
pub struct SelectionLabel {
    pub index: usize,
}

/// Handles keyboard input for navigating and confirming the selection.
///
/// When `SelectionState` is present, this system:
/// - Reads ArrowUp/KeyW for moving the cursor up (with wrapping)
/// - Reads ArrowDown/KeyS for moving the cursor down (with wrapping)
/// - Reads Space/Enter to confirm the currently focused choice
/// - Updates cursor entity visibility so only the active cursor is shown
/// - Blocks player movement by clearing `MovementIntent`
///
/// On confirmation (Space/Enter):
/// - Removes `SelectionState` resource
/// - Despawns all `SelectionBox` entities
/// - Pops the `ShowSelection` action from the front of the `ActionQueue`
/// - Inserts the committed choice's actions at the front of the queue
/// - Clears `waiting_for` to `WaitingFor::Nothing`
pub fn handle_selection_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut selection_state: Option<ResMut<SelectionState>>,
    mut cursor_query: Query<(&SelectionLabel, &mut Visibility), With<SelectionCursor>>,
    mut intent: ResMut<MovementIntent>,
    mut commands: Commands,
    mut action_queue: Option<ResMut<ActionQueue>>,
    selection_boxes: Query<Entity, With<SelectionBox>>,
) {
    let Some(ref mut state) = selection_state else {
        return;
    };

    // Block player movement while selection is active by consuming direction intent
    intent.direction = None;

    // Confirmation: Space or Enter commits the current choice
    if keyboard.just_pressed(KeyCode::Space) || keyboard.just_pressed(KeyCode::Enter) {
        // Clone the selected branch's actions before removing state
        let branch_actions = state.choices[state.cursor_index].actions.clone();

        // Remove SelectionState resource
        commands.remove_resource::<SelectionState>();

        // Despawn all SelectionBox entities
        for entity in selection_boxes.iter() {
            commands.entity(entity).despawn();
        }

        // Update ActionQueue: pop ShowSelection, insert branch actions, clear waiting
        if let Some(ref mut queue) = action_queue {
            // Pop the ShowSelection action from the front of the queue
            queue.actions.pop_front();

            // Insert the committed choice's actions at the front (in order)
            for (i, action) in branch_actions.into_iter().enumerate() {
                queue.actions.insert(i, action);
            }

            // Clear waiting state so advance_action_queue resumes normally
            queue.waiting_for = WaitingFor::Nothing;
        }

        return;
    }

    // Up navigation: ArrowUp or KeyW
    if keyboard.just_pressed(KeyCode::ArrowUp) || keyboard.just_pressed(KeyCode::KeyW) {
        state.cursor_index = (state.cursor_index + state.choice_count - 1) % state.choice_count;
        update_cursor_visibility(state, &mut cursor_query);
    }

    // Down navigation: ArrowDown or KeyS
    if keyboard.just_pressed(KeyCode::ArrowDown) || keyboard.just_pressed(KeyCode::KeyS) {
        state.cursor_index = (state.cursor_index + 1) % state.choice_count;
        update_cursor_visibility(state, &mut cursor_query);
    }
}

/// Updates cursor entity visibility so only the cursor at `state.cursor_index` is visible.
fn update_cursor_visibility(
    state: &SelectionState,
    cursor_query: &mut Query<(&SelectionLabel, &mut Visibility), With<SelectionCursor>>,
) {
    for (label, mut visibility) in cursor_query.iter_mut() {
        if label.index == state.cursor_index {
            *visibility = Visibility::Inherited;
        } else {
            *visibility = Visibility::Hidden;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::app::App;
    use bevy::input::ButtonInput;
    use std::collections::VecDeque;

    /// Creates a minimal Bevy app configured for selection input testing.
    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<ButtonInput<KeyCode>>();
        app.init_resource::<MovementIntent>();
        app.add_systems(Update, handle_selection_input);
        app
    }

    /// Helper to insert a SelectionState with the given choice count and cursor index.
    fn insert_selection_state(app: &mut App, choice_count: usize, cursor_index: usize) {
        app.world_mut().insert_resource(SelectionState {
            cursor_index,
            choice_count,
            choices: (0..choice_count)
                .map(|i| ResolvedChoice {
                    label: format!("Choice {}", i),
                    actions: vec![],
                })
                .collect(),
        });
    }

    /// Helper to insert a SelectionState with specific actions per choice.
    fn insert_selection_state_with_actions(
        app: &mut App,
        choices: Vec<(String, Vec<EventAction>)>,
        cursor_index: usize,
    ) {
        let choice_count = choices.len();
        app.world_mut().insert_resource(SelectionState {
            cursor_index,
            choice_count,
            choices: choices
                .into_iter()
                .map(|(label, actions)| ResolvedChoice { label, actions })
                .collect(),
        });
    }

    /// Helper to insert an ActionQueue with a ShowSelection action at the front.
    fn insert_action_queue_with_show_selection(app: &mut App) {
        use rpg_toolkit_common::map::{DialogConfigData, DialogTextData};
        let mut actions = VecDeque::new();
        actions.push_back(EventAction::ShowSelection {
            prompt: DialogTextData::Inline("Pick one".to_string()),
            config: DialogConfigData::default(),
            choices: vec![],
        });
        app.world_mut().insert_resource(ActionQueue {
            actions,
            waiting_for: WaitingFor::Selection,
        });
    }

    /// Helper to spawn cursor entities with SelectionLabel and SelectionCursor components.
    fn spawn_cursors(app: &mut App, count: usize) {
        for i in 0..count {
            let visibility = if i == 0 {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
            app.world_mut()
                .spawn((SelectionCursor, SelectionLabel { index: i }, visibility));
        }
    }

    /// Helper to spawn a SelectionBox entity.
    fn spawn_selection_box(app: &mut App) -> Entity {
        app.world_mut().spawn(SelectionBox).id()
    }

    /// Helper to press a key in the test app.
    fn press_key(app: &mut App, key: KeyCode) {
        let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        input.press(key);
    }

    #[test]
    fn cursor_moves_down_on_arrow_down() {
        let mut app = test_app();
        insert_selection_state(&mut app, 3, 0);
        spawn_cursors(&mut app, 3);

        press_key(&mut app, KeyCode::ArrowDown);
        app.update();

        let state = app.world().resource::<SelectionState>();
        assert_eq!(state.cursor_index, 1);
    }

    #[test]
    fn cursor_moves_up_on_arrow_up() {
        let mut app = test_app();
        insert_selection_state(&mut app, 3, 1);
        spawn_cursors(&mut app, 3);

        press_key(&mut app, KeyCode::ArrowUp);
        app.update();

        let state = app.world().resource::<SelectionState>();
        assert_eq!(state.cursor_index, 0);
    }

    #[test]
    fn cursor_wraps_down_from_last_to_first() {
        let mut app = test_app();
        insert_selection_state(&mut app, 3, 2);
        spawn_cursors(&mut app, 3);

        press_key(&mut app, KeyCode::ArrowDown);
        app.update();

        let state = app.world().resource::<SelectionState>();
        assert_eq!(state.cursor_index, 0);
    }

    #[test]
    fn cursor_wraps_up_from_first_to_last() {
        let mut app = test_app();
        insert_selection_state(&mut app, 3, 0);
        spawn_cursors(&mut app, 3);

        press_key(&mut app, KeyCode::ArrowUp);
        app.update();

        let state = app.world().resource::<SelectionState>();
        assert_eq!(state.cursor_index, 2);
    }

    #[test]
    fn key_w_moves_cursor_up() {
        let mut app = test_app();
        insert_selection_state(&mut app, 4, 2);
        spawn_cursors(&mut app, 4);

        press_key(&mut app, KeyCode::KeyW);
        app.update();

        let state = app.world().resource::<SelectionState>();
        assert_eq!(state.cursor_index, 1);
    }

    #[test]
    fn key_s_moves_cursor_down() {
        let mut app = test_app();
        insert_selection_state(&mut app, 4, 1);
        spawn_cursors(&mut app, 4);

        press_key(&mut app, KeyCode::KeyS);
        app.update();

        let state = app.world().resource::<SelectionState>();
        assert_eq!(state.cursor_index, 2);
    }

    #[test]
    fn movement_intent_cleared_when_selection_active() {
        let mut app = test_app();
        insert_selection_state(&mut app, 3, 0);
        spawn_cursors(&mut app, 3);

        // Set a movement intent before the system runs
        app.world_mut().resource_mut::<MovementIntent>().direction =
            Some(crate::input::Direction::Up);

        app.update();

        let intent = app.world().resource::<MovementIntent>();
        assert_eq!(intent.direction, None, "MovementIntent should be cleared");
    }

    #[test]
    fn no_crash_without_selection_state() {
        let mut app = test_app();
        // No SelectionState inserted — system should be a no-op
        press_key(&mut app, KeyCode::ArrowDown);
        app.update();
        // If we reach here without panic, the test passes
    }

    #[test]
    fn cursor_visibility_updates_on_navigation() {
        let mut app = test_app();
        insert_selection_state(&mut app, 3, 0);
        spawn_cursors(&mut app, 3);

        press_key(&mut app, KeyCode::ArrowDown);
        app.update();

        // Check visibility: index 1 should be visible, 0 and 2 hidden
        let mut query = app.world_mut().query::<(&SelectionLabel, &Visibility)>();
        let results: Vec<(usize, &Visibility)> = query
            .iter(app.world())
            .map(|(label, vis)| (label.index, vis))
            .collect();

        for (index, vis) in results {
            if index == 1 {
                assert_eq!(
                    *vis,
                    Visibility::Inherited,
                    "Cursor at index 1 should be visible"
                );
            } else {
                assert_eq!(
                    *vis,
                    Visibility::Hidden,
                    "Cursor at index {} should be hidden",
                    index
                );
            }
        }
    }

    // =========================================================================
    // 5.2 Selection confirmation tests
    // =========================================================================

    #[test]
    fn space_confirms_selection_and_removes_state() {
        let mut app = test_app();
        insert_selection_state(&mut app, 3, 1);
        insert_action_queue_with_show_selection(&mut app);
        spawn_cursors(&mut app, 3);
        spawn_selection_box(&mut app);

        press_key(&mut app, KeyCode::Space);
        app.update();

        // SelectionState should be removed
        assert!(
            app.world().get_resource::<SelectionState>().is_none(),
            "SelectionState should be removed after confirmation"
        );
    }

    #[test]
    fn enter_confirms_selection_and_removes_state() {
        let mut app = test_app();
        insert_selection_state(&mut app, 3, 0);
        insert_action_queue_with_show_selection(&mut app);
        spawn_cursors(&mut app, 3);
        spawn_selection_box(&mut app);

        press_key(&mut app, KeyCode::Enter);
        app.update();

        // SelectionState should be removed
        assert!(
            app.world().get_resource::<SelectionState>().is_none(),
            "SelectionState should be removed after Enter confirmation"
        );
    }

    #[test]
    fn confirmation_despawns_selection_box_entities() {
        let mut app = test_app();
        insert_selection_state(&mut app, 2, 0);
        insert_action_queue_with_show_selection(&mut app);
        spawn_cursors(&mut app, 2);
        let box_entity = spawn_selection_box(&mut app);

        press_key(&mut app, KeyCode::Space);
        app.update();

        // The SelectionBox entity should be despawned
        assert!(
            app.world().get_entity(box_entity).is_err(),
            "SelectionBox entity should be despawned after confirmation"
        );
    }

    #[test]
    fn confirmation_pops_show_selection_from_queue() {
        let mut app = test_app();
        insert_selection_state(&mut app, 2, 0);
        insert_action_queue_with_show_selection(&mut app);
        spawn_cursors(&mut app, 2);
        spawn_selection_box(&mut app);

        press_key(&mut app, KeyCode::Space);
        app.update();

        // The ShowSelection action should be popped; queue should be empty
        let queue = app.world().resource::<ActionQueue>();
        assert!(
            queue.actions.is_empty(),
            "ShowSelection should be popped from the queue"
        );
    }

    #[test]
    fn confirmation_inserts_branch_actions_at_front() {
        let mut app = test_app();

        // Set up choices with specific actions for the selected choice
        let branch_actions = vec![EventAction::SetState {
            key: "test_key".to_string(),
            value: "test_value".to_string(),
        }];
        insert_selection_state_with_actions(
            &mut app,
            vec![
                ("Choice 0".to_string(), vec![]),
                ("Choice 1".to_string(), branch_actions.clone()),
                ("Choice 2".to_string(), vec![]),
            ],
            1, // cursor on choice 1
        );
        insert_action_queue_with_show_selection(&mut app);
        spawn_cursors(&mut app, 3);
        spawn_selection_box(&mut app);

        press_key(&mut app, KeyCode::Space);
        app.update();

        // The branch actions should be at the front of the queue
        let queue = app.world().resource::<ActionQueue>();
        assert_eq!(queue.actions.len(), 1);
        assert_eq!(queue.actions[0], branch_actions[0]);
    }

    #[test]
    fn confirmation_clears_waiting_for_to_nothing() {
        let mut app = test_app();
        insert_selection_state(&mut app, 2, 0);
        insert_action_queue_with_show_selection(&mut app);
        spawn_cursors(&mut app, 2);
        spawn_selection_box(&mut app);

        press_key(&mut app, KeyCode::Space);
        app.update();

        let queue = app.world().resource::<ActionQueue>();
        assert_eq!(
            queue.waiting_for,
            WaitingFor::Nothing,
            "waiting_for should be cleared to Nothing after confirmation"
        );
    }

    #[test]
    fn confirmation_without_action_queue_still_removes_state() {
        let mut app = test_app();
        insert_selection_state(&mut app, 2, 0);
        // No ActionQueue inserted
        spawn_cursors(&mut app, 2);
        spawn_selection_box(&mut app);

        press_key(&mut app, KeyCode::Space);
        app.update();

        // SelectionState should still be removed gracefully
        assert!(
            app.world().get_resource::<SelectionState>().is_none(),
            "SelectionState should be removed even without ActionQueue"
        );
    }

    #[test]
    fn confirmation_inserts_multiple_branch_actions_in_order() {
        let mut app = test_app();

        let branch_actions = vec![
            EventAction::SetState {
                key: "first".to_string(),
                value: "1".to_string(),
            },
            EventAction::SetState {
                key: "second".to_string(),
                value: "2".to_string(),
            },
            EventAction::SetState {
                key: "third".to_string(),
                value: "3".to_string(),
            },
        ];
        insert_selection_state_with_actions(
            &mut app,
            vec![
                ("Choice A".to_string(), branch_actions.clone()),
                ("Choice B".to_string(), vec![]),
            ],
            0, // cursor on choice 0
        );
        insert_action_queue_with_show_selection(&mut app);
        spawn_cursors(&mut app, 2);
        spawn_selection_box(&mut app);

        press_key(&mut app, KeyCode::Space);
        app.update();

        // Branch actions should be at the front in original order
        let queue = app.world().resource::<ActionQueue>();
        assert_eq!(queue.actions.len(), 3);
        assert_eq!(queue.actions[0], branch_actions[0]);
        assert_eq!(queue.actions[1], branch_actions[1]);
        assert_eq!(queue.actions[2], branch_actions[2]);
    }
}
