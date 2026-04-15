use bevy::prelude::*;

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
