use bevy::prelude::*;

use crate::resources::{MovementConfig, SpeedMultiplier};

/// Computes the speed-adjusted move duration from the given multiplier.
/// The multiplier is clamped to [0.5, 4.0] before computing the duration.
pub fn compute_speed_move_duration(multiplier: f32) -> f32 {
    0.15 / multiplier.clamp(0.5, 4.0)
}

/// System that adjusts `MovementConfig.move_duration` based on the current
/// `SpeedMultiplier` value. Runs every frame in the `Update` schedule.
pub fn apply_speed_multiplier_system(
    speed_multiplier: Res<SpeedMultiplier>,
    mut movement_config: ResMut<MovementConfig>,
) {
    movement_config.move_duration = compute_speed_move_duration(speed_multiplier.value);
}
