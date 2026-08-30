// Feature: editor-enhancements, Property 15: Speed-Adjusted Move Duration Computation

use proptest::prelude::*;
use rpg_toolkit_renderer::compute_speed_move_duration;

proptest! {
    #![proptest_config(ProptestConfig { cases: 100, .. ProptestConfig::default() })]

    /// **Validates: Requirements 10.2**
    ///
    /// Property 15: For any SpeedMultiplier.value in [0.5, 4.0], the effective
    /// MovementConfig.move_duration SHALL equal 0.15 / value (within f32 epsilon tolerance).
    #[test]
    fn speed_adjusted_move_duration(
        multiplier in 0.5f32..=4.0f32,
    ) {
        let result = compute_speed_move_duration(multiplier);
        let expected = 0.15 / multiplier;

        // Use relative tolerance for floating point comparison
        let diff = (result - expected).abs();
        prop_assert!(
            diff < f32::EPSILON * 10.0,
            "compute_speed_move_duration({}) = {}, expected {}, diff = {}",
            multiplier, result, expected, diff
        );

        // Also verify the result is positive
        prop_assert!(result > 0.0, "move_duration must be positive, got {}", result);
    }
}
