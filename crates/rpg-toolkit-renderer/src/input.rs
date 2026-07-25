use bevy::prelude::*;
use rpg_toolkit_common::AppPhase;

use crate::dialog::DialogState;
use crate::resources::ActionQueue;
use crate::systems::selection::SelectionState;

/// Cardinal movement direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

/// Resource holding the current frame's movement intent.
#[derive(Resource, Default)]
pub struct MovementIntent {
    pub direction: Option<Direction>,
}

/// Reads keyboard input and writes the corresponding `MovementIntent`.
pub fn read_input(keyboard: Res<ButtonInput<KeyCode>>, mut intent: ResMut<MovementIntent>) {
    intent.direction = if keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp) {
        Some(Direction::Up)
    } else if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
        Some(Direction::Left)
    } else if keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown) {
        Some(Direction::Down)
    } else if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
        Some(Direction::Right)
    } else {
        None
    };
}

/// Opens the status screen when the player presses Escape during free gameplay.
/// Does nothing if a dialog, selection, or action queue is currently active.
pub fn open_status_on_escape(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_phase: ResMut<NextState<AppPhase>>,
    dialog_state: Option<Res<DialogState>>,
    selection_state: Option<Res<SelectionState>>,
    action_queue: Option<Res<ActionQueue>>,
) {
    // Don't open status if a dialog or selection is active
    if dialog_state.is_some() || selection_state.is_some() {
        return;
    }

    // Don't open status if an action queue is being processed
    if action_queue.is_some() {
        return;
    }

    if keyboard.just_pressed(KeyCode::Escape) {
        next_phase.set(AppPhase::Status);
    }
}
