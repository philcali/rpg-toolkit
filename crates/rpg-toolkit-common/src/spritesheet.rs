use serde::{Deserialize, Serialize};

use crate::error::CommonError;

/// Type alias for spritesheet identifiers (UUID v4 strings).
pub type SpritesheetId = String;

/// One of four cardinal directions determining which sprite row to display.
/// Numeric values map directly to spritesheet row indices.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FacingDirection {
    #[default]
    Down = 0,
    Left = 1,
    Right = 2,
    Up = 3,
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

/// A per-map entity placed on a specific tile, referencing a CharacterSpritesheet
/// and a facing direction.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NpcInstance {
    pub spritesheet_id: SpritesheetId,
    pub x: u32,
    pub y: u32,
    pub facing: FacingDirection,
    /// Future-compatible: event triggers (deferred, Requirement 9).
    #[serde(default)]
    pub event_triggers: Vec<crate::map::EventAction>,
    /// Future-compatible: patrol path waypoints (deferred, Requirement 9).
    #[serde(default)]
    pub patrol_path: Vec<(u32, u32)>,
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
