use serde::{Deserialize, Serialize};

use crate::error::CommonError;

/// Type alias for spritesheet identifiers (UUID v4 strings).
pub type SpritesheetId = String;

/// One of four cardinal directions determining which sprite row to display.
/// Numeric values map directly to spritesheet row indices.
/// Layout: row 0 = Up, row 1 = Right, row 2 = Down, row 3 = Left.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FacingDirection {
    Up = 0,
    Right = 1,
    #[default]
    Down = 2,
    Left = 3,
}

/// A project-level image asset containing a grid of animation frames for a character.
/// Layout: 3 columns (frames) × 4 rows (directions), each frame 24×32 pixels.
/// Total image size must be 72×128.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CharacterSpritesheet {
    pub file_path: String,
    pub sprite_width: u32,
    pub sprite_height: u32,
    pub frame_count: u32,
    pub direction_count: u32,
}

/// Behavior when an NPC reaches the end of its patrol path.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum PatrolMode {
    /// Follow waypoints in order, looping back to the first when reaching the last.
    #[default]
    Loop,
    /// Wander randomly to adjacent unblocked tiles (no waypoints needed).
    Random,
}

fn default_patrol_speed() -> f32 {
    0.3
}

fn default_patrol_pause() -> f32 {
    0.5
}

/// Complete patrol behavior configuration for an NPC instance.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PatrolConfig {
    /// Ordered waypoint grid positions.
    pub waypoints: Vec<(u32, u32)>,
    /// Behavior at path endpoints.
    #[serde(default)]
    pub mode: PatrolMode,
    /// Seconds per tile movement (default 0.3).
    #[serde(default = "default_patrol_speed")]
    pub speed: f32,
    /// Seconds to pause at each waypoint (default 0.5).
    #[serde(default = "default_patrol_pause")]
    pub pause: f32,
}

/// The condition under which an NPC's event triggers fire.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum TriggerMode {
    Collision,
    #[default]
    Interaction,
}

/// A per-map entity placed on a specific tile, referencing a CharacterSpritesheet
/// and a facing direction.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NpcInstance {
    pub spritesheet_id: SpritesheetId,
    pub x: u32,
    pub y: u32,
    pub facing: FacingDirection,
    #[serde(default)]
    pub event_triggers: Vec<crate::map::EventAction>,
    /// Optional patrol behavior configuration.
    #[serde(default)]
    pub patrol_config: Option<PatrolConfig>,
    #[serde(default)]
    pub trigger_mode: TriggerMode,
    /// Elevation level at which this NPC exists.
    #[serde(default)]
    pub elevation: u32,
}

/// Validates that spritesheet image dimensions are exactly 72×128 pixels.
pub fn validate_spritesheet_dimensions(width: u32, height: u32) -> Result<(), CommonError> {
    if width == 72 && height == 128 {
        Ok(())
    } else {
        Err(CommonError::ProjectValidationError(format!(
            "spritesheet dimensions must be 72×128, got {}×{}",
            width, height
        )))
    }
}

/// Returns the texture atlas index for a given facing direction and animation frame.
///
/// The atlas is laid out as 4 rows (directions) × 3 columns (frames).
/// Index = row * 3 + frame.
pub fn sprite_atlas_index(facing: FacingDirection, frame: usize) -> usize {
    facing as usize * 3 + frame
}

/// Walk cycle pattern: left step, center, right step, center.
const WALK_PATTERN: [usize; 4] = [0, 1, 2, 1];

/// Returns the current walk animation frame (0, 1, or 2) based on elapsed time
/// and frame duration, cycling through the `[0, 1, 2, 1]` pattern continuously.
pub fn walk_animation_frame(elapsed: f32, frame_duration: f32) -> usize {
    let step = (elapsed / frame_duration).floor() as usize % 4;
    WALK_PATTERN[step]
}

/// Computes the next waypoint index for a Loop patrol path.
///
/// Wraps from the last waypoint back to index 0.
///
/// Returns `(next_index, forward)` where `forward` is always `true` for Loop mode.
/// This function is only used for Loop mode; Random mode doesn't use waypoints.
pub fn next_waypoint_index(
    current: usize,
    waypoint_count: usize,
    _mode: PatrolMode,
    _forward: bool,
) -> (usize, bool) {
    // Loop: wrap around
    ((current + 1) % waypoint_count, true)
}

/// Returns `true` if the waypoint is within the map bounds.
pub fn validate_waypoint_bounds(waypoint: (u32, u32), map_width: u32, map_height: u32) -> bool {
    waypoint.0 < map_width && waypoint.1 < map_height
}

/// Returns the grid position of the tile the player is facing, or `None` if
/// that tile would be outside the map (e.g., facing Up at y=0).
pub fn faced_tile(player_x: u32, player_y: u32, facing: FacingDirection) -> Option<(u32, u32)> {
    match facing {
        FacingDirection::Up => player_y.checked_sub(1).map(|y| (player_x, y)),
        FacingDirection::Down => Some((player_x, player_y + 1)),
        FacingDirection::Left => player_x.checked_sub(1).map(|x| (x, player_y)),
        FacingDirection::Right => Some((player_x + 1, player_y)),
    }
}
