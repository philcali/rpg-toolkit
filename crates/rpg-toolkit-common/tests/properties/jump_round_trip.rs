// Feature: editor-enhancements, Property 1: Jump EventAction Serialization Round-Trip

use proptest::prelude::*;

use rpg_toolkit_common::map::EventAction;

proptest! {
    #![proptest_config(ProptestConfig { cases: 100, .. ProptestConfig::default() })]

    /// **Validates: Requirements 1.1, 1.2, 1.5, 1.6**
    ///
    /// Property 1: Jump EventAction Serialization Round-Trip.
    /// For any valid EventAction::Jump value with distance in [0, 8],
    /// serializing to JSON and deserializing back shall produce a value
    /// that is PartialEq-equal to the original. The serialized JSON shall
    /// contain "type": "Jump" and a "distance" field.
    #[test]
    fn jump_action_serialization_round_trip(
        distance in 0u32..=8u32,
    ) {
        let action = EventAction::Jump { distance };

        // Serialize to JSON
        let json = serde_json::to_string(&action).unwrap();

        // Verify JSON contains expected fields
        prop_assert!(json.contains("\"type\":\"Jump\"") || json.contains("\"type\": \"Jump\""),
            "Serialized JSON should contain type field: {}", json);
        prop_assert!(json.contains("\"distance\""),
            "Serialized JSON should contain distance field: {}", json);

        // Deserialize back
        let deserialized: EventAction = serde_json::from_str(&json).unwrap();

        // Assert equality
        prop_assert_eq!(&action, &deserialized);
    }
}
