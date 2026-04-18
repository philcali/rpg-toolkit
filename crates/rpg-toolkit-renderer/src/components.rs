use bevy::prelude::*;
use rpg_toolkit_common::FacingDirection;

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

/// Marker for NPC sprite entities spawned by the renderer.
#[derive(Component)]
pub struct NpcSprite {
    pub npc_index: usize,
}

/// Tracks the player's sprite animation state when using a spritesheet.
#[derive(Component)]
pub struct PlayerSpriteState {
    pub facing: FacingDirection,
    pub animation_frame: usize,
    pub animation_timer: f32,
    pub is_moving: bool,
    /// Counts frames since `is_moving` went false.
    /// Used to allow a one-frame grace period between consecutive tile
    /// moves so the walk animation timer isn't reset.
    pub idle_frames: u32,
    /// Y offset applied to the sprite so that the character's feet
    /// align with the bottom of the tile rather than centering the
    /// sprite on the tile. Computed from the height difference between
    /// the scaled sprite and the tile.
    pub y_offset: f32,
}

/// Marker for the game camera (distinct from the editor camera).
#[derive(Component)]
pub struct GameCamera;
