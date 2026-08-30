// Feature: editor-enhancements, Property 6: ParallaxLayer Invalid scroll_factor Rejection

use proptest::prelude::*;

use rpg_toolkit_common::map::ParallaxLayer;

proptest! {
    #![proptest_config(ProptestConfig { cases: 100, .. ProptestConfig::default() })]

    /// **Validates: Requirements 3.6**
    ///
    /// Property 6: ParallaxLayer Invalid scroll_factor Rejection.
    /// For any f32 value strictly less than 0.0 or strictly greater than 1.0,
    /// deserializing a ParallaxLayer with that scroll_factor SHALL return a
    /// deserialization error.
    #[test]
    fn parallax_layer_rejects_invalid_scroll_factor(
        value in prop_oneof![
            // Values below 0.0 (avoid -0.0 edge case)
            (-f32::MAX..=-0.01f32),
            // Values above 1.0
            (1.01f32..=f32::MAX),
        ]
    ) {
        let json = format!(
            r#"{{"image_path": "test.png", "scroll_factor": {}, "z_order": 0}}"#,
            value
        );

        let result = serde_json::from_str::<ParallaxLayer>(&json);
        prop_assert!(
            result.is_err(),
            "Expected deserialization error for scroll_factor={}, but got Ok({:?})",
            value,
            result.unwrap()
        );
    }
}
