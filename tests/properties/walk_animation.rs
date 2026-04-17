// Feature: character-spritesheets, Property 6: Walk animation frame cycling
//
// For any non-negative elapsed time and positive frame duration, the animation
// system should produce frame index floor(elapsed / frame_duration) % 3,
// cycling through frames 0, 1, 2 continuously while the player is moving.
//
// **Validates: Requirements 4.1**

use proptest::prelude::*;
use rpg_toolkit_common::walk_animation_frame;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn walk_animation_frame_cycling(
        elapsed in 0.0f32..10.0,
        frame_duration in 0.01f32..1.0,
    ) {
        let frame = walk_animation_frame(elapsed, frame_duration);
        let expected = (elapsed / frame_duration).floor() as usize % 3;

        prop_assert_eq!(frame, expected,
            "elapsed={}, frame_duration={}, got frame={}, expected={}",
            elapsed, frame_duration, frame, expected
        );

        // Frame must always be in [0, 3)
        prop_assert!(frame < 3, "frame {} out of range [0, 3)", frame);
    }
}
