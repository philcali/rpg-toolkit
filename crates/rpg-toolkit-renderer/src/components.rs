use bevy::prelude::*;

/// Marker + state for the player character entity.
#[derive(Component)]
pub struct PlayerCharacter {
    /// Current grid position (tile coordinates).
    pub grid_x: u32,
    pub grid_y: u32,
    /// Movement animation state. `Some` while animating between tiles.
    pub move_animation: Option<MoveAnimation>,
}

/// Describes an in-progress tile-to-tile movement animation.
pub struct MoveAnimation {
    /// World-space start position.
    pub from: Vec2,
    /// World-space target position.
    pub to: Vec2,
    /// Grid coordinates before the move.
    pub from_grid: (u32, u32),
    /// Grid coordinates after the move.
    pub to_grid: (u32, u32),
    /// Seconds elapsed since animation started.
    pub elapsed: f32,
    /// Total animation duration in seconds.
    pub duration: f32,
}

/// Marker for tile sprites spawned by the renderer.
#[derive(Component)]
pub struct RendererTileSprite {
    pub layer_index: usize,
    pub x: u32,
    pub y: u32,
}

/// Marker for the game camera (distinct from the editor camera).
#[derive(Component)]
pub struct GameCamera;
