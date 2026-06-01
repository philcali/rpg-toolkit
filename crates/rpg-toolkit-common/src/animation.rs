use serde::{Deserialize, Serialize};

use crate::error::CommonError;

/// A single frame in a tile animation sequence.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnimationFrame {
    pub col: u32,
    pub row: u32,
}

/// A tile animation definition: an ordered sequence of frames with a shared duration.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TileAnimation {
    /// Ordered frames in the animation cycle. Must contain >= 2 frames.
    pub frames: Vec<AnimationFrame>,
    /// Duration of each frame in milliseconds. Must be > 0.
    pub frame_duration_ms: u32,
}

/// Validates a TileAnimation against tileset bounds.
///
/// Checks that:
/// - The animation has at least 2 frames
/// - `frame_duration_ms` is greater than zero
/// - All frame coordinates are within the tileset bounds (col < columns, row < rows)
///
/// Returns `Ok(())` if valid, `Err(CommonError)` with a descriptive message otherwise.
pub fn validate_tile_animation(
    animation: &TileAnimation,
    columns: u32,
    rows: u32,
) -> Result<(), CommonError> {
    if animation.frames.len() < 2 {
        return Err(CommonError::AnimationTooFewFrames);
    }

    if animation.frame_duration_ms == 0 {
        return Err(CommonError::AnimationInvalidDuration);
    }

    for frame in &animation.frames {
        if frame.col >= columns || frame.row >= rows {
            return Err(CommonError::AnimationFrameOutOfBounds {
                col: frame.col,
                row: frame.row,
            });
        }
    }

    Ok(())
}

/// Computes which frame index to display given elapsed time.
///
/// Returns an index into the animation's frames vec, cycling through frames
/// at the rate of one frame per `frame_duration_ms` milliseconds.
///
/// Pure function — no side effects, no state.
pub fn compute_animation_frame_index(
    elapsed_ms: u64,
    frame_duration_ms: u32,
    frame_count: usize,
) -> usize {
    ((elapsed_ms / frame_duration_ms as u64) % frame_count as u64) as usize
}
