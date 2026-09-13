use rpg_toolkit_common::map::{EventAction, ScreenShakeMode};

/// Computes a shake offset for a given intensity.
/// Returns (dx, dy) where |dx| <= intensity and |dy| <= intensity.
///
/// `seed_x` and `seed_y` should be in [0.0, 1.0] and are mapped to [-intensity, +intensity].
pub fn compute_shake_offset(intensity: f32, seed_x: f32, seed_y: f32) -> (f32, f32) {
    let dx = (seed_x * 2.0 - 1.0) * intensity;
    let dy = (seed_y * 2.0 - 1.0) * intensity;
    (dx, dy)
}

/// Returns true if a shake effect has completed.
///
/// For `Timed` mode, returns true when elapsed >= duration.
/// For `Continuous` mode, always returns false (must be stopped explicitly).
pub fn is_shake_complete(elapsed: f32, duration: f32, mode: ScreenShakeMode) -> bool {
    match mode {
        ScreenShakeMode::Timed => elapsed >= duration,
        ScreenShakeMode::Continuous => false,
    }
}

/// Computes the normalized time factor `t = elapsed / duration`, clamped to
/// `[0.0, 1.0]`.
///
/// This is the raw animation phase; mapping it to the shader `progress` value
/// (which depends on the transition direction) is handled by
/// [`crate::transition::direction_progress`].
pub fn compute_transition_t(elapsed: f32, duration: f32) -> f32 {
    if duration <= 0.0 {
        return 1.0;
    }
    (elapsed / duration).clamp(0.0, 1.0)
}

/// Returns true if a screen transition has completed.
pub fn is_transition_complete(elapsed: f32, duration: f32) -> bool {
    elapsed >= duration
}

/// Classifies an EventAction as blocking or non-blocking.
///
/// Blocking actions prevent the ActionQueue from advancing until they complete:
/// - `ScreenShake` with mode `Timed` and duration > 0
/// - `ScreenTransition` with duration > 0
/// - `ShowDialog`
/// - `ShowSelection`
/// - `MoveEntity`
/// - `CameraPan`
/// - `Wait`
/// - `Jump`
///
/// Non-blocking actions execute immediately:
/// - `CameraFollow`
/// - `SetSpeed`
/// - All other actions
pub fn is_blocking_action(action: &EventAction) -> bool {
    match action {
        EventAction::ScreenShake { mode, duration, .. } => {
            *mode == ScreenShakeMode::Timed && *duration > 0.0
        }
        EventAction::ScreenTransition { duration, .. } => *duration > 0.0,
        EventAction::ShowDialog { .. } => true,
        EventAction::ShowSelection { .. } => true,
        EventAction::MoveEntity { .. } => true,
        EventAction::CameraFollow { .. } => false,
        EventAction::CameraPan { .. } => true,
        EventAction::Wait { .. } => true,
        EventAction::Jump { .. } => true,
        // Audio actions are always non-blocking: the queue advances within the
        // same processing step (Req 6.11, 8.4).
        EventAction::PlayMusic { .. } => false,
        EventAction::PlaySoundEffect { .. } => false,
        _ => false,
    }
}
