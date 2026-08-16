// Feature: game-intro-narration, Property 3: Camera pan interpolation is bounded

use proptest::prelude::*;

/// Computes the interpolated camera position for a pan.
/// This mirrors the logic in camera_pan_system.
fn interpolate_pan(
    start_x: f32,
    start_y: f32,
    target_x: f32,
    target_y: f32,
    elapsed: f32,
    duration: f32,
) -> (f32, f32) {
    let t = (elapsed / duration).clamp(0.0, 1.0);
    let x = start_x + (target_x - start_x) * t;
    let y = start_y + (target_y - start_y) * t;
    (x, y)
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 100, .. ProptestConfig::default() })]

    /// **Validates: Requirements 4.4**
    ///
    /// Property 3: For any start/target positions and elapsed time in [0, duration],
    /// the interpolated position is always within the axis-aligned bounding box of
    /// start and target.
    #[test]
    fn pan_interpolation_is_bounded(
        start_x in -1000.0f32..1000.0f32,
        start_y in -1000.0f32..1000.0f32,
        target_x in -1000.0f32..1000.0f32,
        target_y in -1000.0f32..1000.0f32,
        duration in 0.1f32..10.0f32,
        elapsed_fraction in 0.0f32..=1.0f32,
    ) {
        let elapsed = elapsed_fraction * duration;
        let (x, y) = interpolate_pan(start_x, start_y, target_x, target_y, elapsed, duration);

        let min_x = start_x.min(target_x);
        let max_x = start_x.max(target_x);
        let min_y = start_y.min(target_y);
        let max_y = start_y.max(target_y);

        prop_assert!(x >= min_x - f32::EPSILON && x <= max_x + f32::EPSILON,
            "x={} not in [{}, {}]", x, min_x, max_x);
        prop_assert!(y >= min_y - f32::EPSILON && y <= max_y + f32::EPSILON,
            "y={} not in [{}, {}]", y, min_y, max_y);
    }
}
