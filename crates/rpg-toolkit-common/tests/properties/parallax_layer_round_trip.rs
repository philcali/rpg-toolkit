// Feature: editor-enhancements, Property 5: ParallaxLayer Validation Acceptance
// Feature: editor-enhancements, Property 7: ParallaxLayer Round-Trip

use proptest::prelude::*;

use rpg_toolkit_common::map::ParallaxLayer;

proptest! {
    #![proptest_config(ProptestConfig { cases: 100, .. ProptestConfig::default() })]

    /// **Validates: Requirements 3.2, 3.3, 3.5**
    ///
    /// Property 5: ParallaxLayer Validation Acceptance.
    /// For any `image_path` string of length 1 to 256, `scroll_factor` in [0.0, 1.0],
    /// and any `i32` `z_order`, deserializing a ParallaxLayer SHALL succeed without error.
    #[test]
    fn parallax_layer_validation_acceptance(
        image_path in "[a-zA-Z0-9_/.-]{1,256}",
        scroll_factor in 0.0f32..=1.0f32,
        z_order in proptest::num::i32::ANY,
    ) {
        let json = serde_json::json!({
            "image_path": image_path,
            "scroll_factor": scroll_factor,
            "z_order": z_order,
        });

        let result: Result<ParallaxLayer, _> = serde_json::from_value(json);
        prop_assert!(result.is_ok(), "Expected successful deserialization, got error: {:?}", result.err());
    }

    /// **Validates: Requirements 3.1, 3.4, 3.7**
    ///
    /// Property 7: ParallaxLayer Round-Trip.
    /// For any valid `ParallaxLayer` value (image_path 1–256 chars, scroll_factor in [0.0, 1.0],
    /// any i32 z_order), serializing to JSON and deserializing back SHALL produce a
    /// `PartialEq`-equal value.
    #[test]
    fn parallax_layer_round_trip(
        image_path in "[a-zA-Z0-9_/.-]{1,256}",
        scroll_factor in 0.0f32..=1.0f32,
        z_order in proptest::num::i32::ANY,
    ) {
        let original = ParallaxLayer {
            image_path,
            scroll_factor,
            z_order,
        };

        // Serialize to JSON
        let json = serde_json::to_string(&original).unwrap();

        // Deserialize back
        let deserialized: ParallaxLayer = serde_json::from_str(&json).unwrap();

        // Assert equality
        prop_assert_eq!(&original, &deserialized);
    }
}
