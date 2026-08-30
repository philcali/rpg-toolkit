//! Public utility functions for the RPG toolkit editor.
//!
//! These functions are extracted from the editor's internal logic
//! so they can be tested independently via integration tests.

/// Parses a string as a `u32`, defaulting to 2 on failure,
/// then clamps the result to the range [0, 8].
pub fn clamp_jump_distance(input: &str) -> u32 {
    input.trim().parse::<u32>().unwrap_or(2).clamp(0, 8)
}

/// Clamps a speed multiplier value to the range [0.5, 4.0].
pub fn clamp_speed_multiplier(value: f32) -> f32 {
    value.clamp(0.5, 4.0)
}
