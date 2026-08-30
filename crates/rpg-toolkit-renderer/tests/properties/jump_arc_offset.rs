// Feature: editor-enhancements, Property 4: Jump Parabolic Offset Invariant

use proptest::prelude::*;
use rpg_toolkit_renderer::jump_arc_offset;

proptest! {
    #![proptest_config(ProptestConfig { cases: 100, .. ProptestConfig::default() })]

    /// **Validates: Requirements 2.4**
    ///
    /// Property 4: For any progress value t in [0.0, 1.0] and any distance in [0, 8],
    /// the parabolic arc offset SHALL be 0.0 at t=0.0 and t=1.0, SHALL be strictly
    /// positive for all t in (0.0, 1.0), and the maximum offset SHALL occur at t=0.5.
    #[test]
    fn jump_arc_offset_parabolic_invariant(
        t in 0.0f32..=1.0f32,
        distance in 0u32..=8u32,
        tile_height in 8.0f32..64.0f32,
    ) {
        let offset = jump_arc_offset(t, distance, tile_height);

        // At t=0.0, offset must be 0.0
        let offset_at_zero = jump_arc_offset(0.0, distance, tile_height);
        prop_assert!((offset_at_zero).abs() < f32::EPSILON,
            "offset at t=0.0 should be 0.0, got {}", offset_at_zero);

        // At t=1.0, offset must be 0.0
        let offset_at_one = jump_arc_offset(1.0, distance, tile_height);
        prop_assert!((offset_at_one).abs() < f32::EPSILON,
            "offset at t=1.0 should be 0.0, got {}", offset_at_one);

        // For t in (0.0, 1.0), offset must be strictly positive
        if t > 0.0 && t < 1.0 {
            prop_assert!(offset > 0.0,
                "offset at t={} should be positive, got {}", t, offset);
        }

        // Maximum offset occurs at t=0.5
        let offset_at_half = jump_arc_offset(0.5, distance, tile_height);
        prop_assert!(offset <= offset_at_half + f32::EPSILON,
            "offset at t={} ({}) should not exceed offset at t=0.5 ({})",
            t, offset, offset_at_half);
    }
}
