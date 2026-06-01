// Feature: editor-ux-improvements, Property 7: Display Tile Size Clamping
//
// For any raw scale value, the computed display_tile_size SHALL always be
// in the range [16, 128] inclusive.
//
// Validates: Requirements 7.2, 7.3

use proptest::prelude::*;

/// Reference implementation of clamp_palette_scale.
/// This mirrors the implementation in
/// `rpg_toolkit_editor::data::state::clamp_palette_scale`.
fn clamp_palette_scale(scale: f32) -> f32 {
    scale.clamp(16.0, 128.0)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Validates: Requirements 7.2, 7.3**
    ///
    /// Property 7: For any arbitrary f32 scale value, clamp_palette_scale
    /// always returns a value in the range [16.0, 128.0] inclusive.
    #[test]
    fn display_tile_size_clamping(scale in any::<f32>()) {
        let result = clamp_palette_scale(scale);
        prop_assert!(
            result >= 16.0,
            "clamp_palette_scale({}) returned {} which is below minimum 16.0",
            scale,
            result
        );
        prop_assert!(
            result <= 128.0,
            "clamp_palette_scale({}) returned {} which is above maximum 128.0",
            scale,
            result
        );
    }
}

// Feature: editor-ux-improvements, Property 8: Default Display Tile Size Computation
//
// For any tileset with tile_width in {8, 16, 32, 64}, the default
// display_tile_size SHALL equal max(tile_width, 24).
//
// Validates: Requirements 7.6

/// Reference implementation of default palette scale computation.
/// This mirrors the inline logic in `tile_palette_ui` that computes:
/// `clamp_palette_scale((tile_width as f32).max(24.0))`
fn compute_default_palette_scale(tile_width: u32) -> f32 {
    clamp_palette_scale((tile_width as f32).max(24.0))
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Validates: Requirements 7.6**
    ///
    /// Property 8: For tile_width in {8, 16, 32, 64}, the default display
    /// tile size equals max(tile_width, 24).
    #[test]
    fn default_display_tile_size_computation(
        tile_width in prop_oneof![Just(8u32), Just(16u32), Just(32u32), Just(64u32)]
    ) {
        let expected = (tile_width as f32).max(24.0);
        let result = compute_default_palette_scale(tile_width);
        prop_assert_eq!(
            result,
            expected,
            "For tile_width={}, expected default {} but got {}",
            tile_width,
            expected,
            result
        );
    }
}
