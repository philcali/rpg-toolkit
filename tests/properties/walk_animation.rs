// Feature: renderer-polish, Property 5: Walk animation frame follows [0, 1, 2, 1] pattern
//
// For any non-negative elapsed time and positive frame duration, the animation
// system should produce frame index [0, 1, 2, 1][floor(elapsed / frame_duration) % 4],
// cycling through the four-step walk pattern continuously while the player is moving.
//
// **Validates: Requirements 3.4, 3.5**

use proptest::prelude::*;
use rpg_toolkit_common::walk_animation_frame;

const WALK_PATTERN: [usize; 4] = [0, 1, 2, 1];

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn walk_animation_frame_cycling(
        elapsed in 0.0f32..10.0,
        frame_duration in 0.01f32..1.0,
    ) {
        let frame = walk_animation_frame(elapsed, frame_duration);
        let step = (elapsed / frame_duration).floor() as usize % 4;
        let expected = WALK_PATTERN[step];

        prop_assert_eq!(frame, expected,
            "elapsed={}, frame_duration={}, got frame={}, expected={}",
            elapsed, frame_duration, frame, expected
        );

        // Frame must always be in [0, 3)
        prop_assert!(frame < 3, "frame {} out of range [0, 3)", frame);
    }
}
