// Feature: database-editor-enhancements, Property 7: Enemy serialization round-trip with portrait

use proptest::prelude::*;

use rpg_toolkit_common::enemy::EnemyRegistry;

/// Strategy for generating a valid enemy display name (1-64 chars, non-empty after trim).
fn arb_display_name() -> impl Strategy<Value = String> {
    "[a-zA-Z][a-zA-Z0-9 ]{0,63}".prop_filter("display name must be non-empty after trim", |s| {
        let trimmed = s.trim();
        !trimmed.is_empty() && trimmed.chars().count() <= 64
    })
}

/// Strategy for generating a valid portrait path (non-empty, trimmed, ≤260 chars).
fn arb_portrait_path() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9_/\\-.]{1,260}".prop_filter("portrait path must be non-empty after trim", |s| {
        let trimmed = s.trim();
        !trimmed.is_empty() && trimmed.chars().count() <= 260
    })
}

/// Strategy for generating an optional portrait (Some with valid path, or None).
fn arb_optional_portrait() -> impl Strategy<Value = Option<String>> {
    prop_oneof![Just(None), arb_portrait_path().prop_map(Some),]
}

/// Strategy for generating a vec of (display_name, optional_portrait) pairs.
fn arb_enemy_entries() -> impl Strategy<Value = Vec<(String, Option<String>)>> {
    proptest::collection::vec((arb_display_name(), arb_optional_portrait()), 1..=10)
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 100, .. ProptestConfig::default() })]

    /// **Validates: Requirements 4.6**
    ///
    /// Property 7: Enemy serialization round-trip with portrait.
    /// For any valid EnemyRegistry containing enemies with and without portrait values,
    /// serializing to JSON and deserializing back shall produce a value equal to the original.
    #[test]
    fn enemy_serialization_round_trip_with_portrait(
        entries in arb_enemy_entries(),
    ) {
        let mut registry = EnemyRegistry::default();

        for (name, portrait) in &entries {
            let enemy_id = registry.create_enemy(name).unwrap();
            if let Some(path) = portrait {
                registry.set_portrait(&enemy_id, path).unwrap();
            }
        }

        // Serialize to JSON
        let json = serde_json::to_string(&registry).unwrap();

        // Deserialize back
        let deserialized: EnemyRegistry = serde_json::from_str(&json).unwrap();

        // Assert equality
        prop_assert_eq!(&registry, &deserialized);
    }
}
