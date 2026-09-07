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

/// Clamps an audio fade duration (fade-in / cross-fade / fade-out) to the
/// range [0.0, 10.0]. Used by both `PlayMusic.fade_duration` and
/// `PlayMusic.fade_out_duration`.
///
/// NaN inputs are coerced to `0.0` so the resulting action always carries a
/// finite, in-range value.
pub fn clamp_fade_duration(value: f32) -> f32 {
    if value.is_nan() {
        0.0
    } else {
        value.clamp(0.0, 10.0)
    }
}

/// Clamps an optional loop count for `PlayMusic`.
///
/// - `None` means infinite repetition and is preserved.
/// - A finite `Some(n)` with `n < 1` (i.e. `0`) is clamped up to `1`.
/// - Any other `Some(n)` is preserved unchanged.
pub fn clamp_loop_count(value: Option<u32>) -> Option<u32> {
    value.map(|n| n.max(1))
}

/// Clamps a sound-effect playback volume to the range [0.0, 1.0].
///
/// NaN inputs are coerced to `1.0` (the default volume) so the resulting
/// action always carries a finite, in-range value.
pub fn clamp_volume(value: f32) -> f32 {
    if value.is_nan() {
        1.0
    } else {
        value.clamp(0.0, 1.0)
    }
}
