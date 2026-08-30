// Feature: editor-enhancements, Property 8: Parallax Scroll Translation Computation

use bevy::math::Vec2;
use proptest::prelude::*;
use rpg_toolkit_renderer::compute_parallax_translation;

proptest! {
    #![proptest_config(ProptestConfig { cases: 100, .. ProptestConfig::default() })]

    /// **Validates: Requirements 4.2**
    ///
    /// Property 8: For any camera delta vector (dx, dy) and any scroll_factor in [0.0, 1.0],
    /// the parallax layer translation delta SHALL equal (dx * scroll_factor, dy * scroll_factor).
    #[test]
    fn parallax_scroll_translation(
        dx in -10000.0f32..10000.0f32,
        dy in -10000.0f32..10000.0f32,
        scroll_factor in 0.0f32..=1.0f32,
    ) {
        let camera_delta = Vec2::new(dx, dy);
        let result = compute_parallax_translation(camera_delta, scroll_factor);

        let expected_x = dx * scroll_factor;
        let expected_y = dy * scroll_factor;

        let diff_x = (result.x - expected_x).abs();
        let diff_y = (result.y - expected_y).abs();

        prop_assert!(
            diff_x < f32::EPSILON * 100.0,
            "x component: compute_parallax_translation(Vec2({}, {}), {}).x = {}, expected {}, diff = {}",
            dx, dy, scroll_factor, result.x, expected_x, diff_x
        );
        prop_assert!(
            diff_y < f32::EPSILON * 100.0,
            "y component: compute_parallax_translation(Vec2({}, {}), {}).y = {}, expected {}, diff = {}",
            dx, dy, scroll_factor, result.y, expected_y, diff_y
        );

        // scroll_factor of 0.0 should produce zero translation
        if scroll_factor == 0.0 {
            prop_assert_eq!(result, Vec2::ZERO);
        }

        // scroll_factor of 1.0 should produce full camera movement
        if scroll_factor == 1.0 {
            prop_assert!(
                (result.x - dx).abs() < f32::EPSILON,
                "At scroll_factor 1.0, translation should equal camera delta"
            );
            prop_assert!(
                (result.y - dy).abs() < f32::EPSILON,
                "At scroll_factor 1.0, translation should equal camera delta"
            );
        }
    }
}
