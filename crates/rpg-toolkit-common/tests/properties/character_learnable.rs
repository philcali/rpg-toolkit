// Feature: database-editor-enhancements, Property 4: Learnable ability level invariant

use proptest::collection;
use proptest::prelude::*;

use rpg_toolkit_common::character::CharacterRegistry;

/// Strategy for generating a valid ability_id (UUID-like format).
fn arb_ability_id() -> impl Strategy<Value = String> {
    "[a-f0-9]{8}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{12}"
}

/// Strategy for generating a valid character display name (1-64 chars, non-empty after trim).
fn arb_display_name() -> impl Strategy<Value = String> {
    "[a-zA-Z][a-zA-Z0-9 ]{0,63}".prop_filter("display name must be non-empty after trim", |s| {
        let trimmed = s.trim();
        !trimmed.is_empty() && trimmed.len() <= 64
    })
}

/// Strategy for generating a required_level that is always valid (1..=99).
fn arb_valid_level() -> impl Strategy<Value = u32> {
    1u32..=99
}

/// Strategy for generating an arbitrary u32 level (including out-of-range values).
fn arb_any_level() -> impl Strategy<Value = u32> {
    prop_oneof![
        // Valid range
        1u32..=99,
        // Below valid range (0)
        Just(0u32),
        // Above valid range
        100u32..=1000,
    ]
}

/// Strategy for generating a vec of unique ability IDs with associated levels.
fn arb_learnable_entries(max_entries: usize) -> impl Strategy<Value = Vec<(String, u32)>> {
    collection::vec((arb_ability_id(), arb_valid_level()), 0..=max_entries).prop_map(|entries| {
        // Deduplicate by ability_id
        let mut seen = std::collections::HashSet::new();
        entries
            .into_iter()
            .filter(|(id, _)| seen.insert(id.clone()))
            .collect()
    })
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 100, .. ProptestConfig::default() })]

    /// **Validates: Requirements 3.1, 3.4, 3.7**
    ///
    /// Property 4: Learnable ability level invariant.
    /// For any character with learnable abilities added via the registry methods,
    /// all required_level values are in the range [1, 99] inclusive.
    #[test]
    fn learnable_abilities_have_valid_levels(
        name in arb_display_name(),
        entries in arb_learnable_entries(20),
    ) {
        let mut registry = CharacterRegistry::default();
        let char_id = registry.create_character(&name).unwrap();

        for (ability_id, level) in &entries {
            let _ = registry.add_learnable_ability(&char_id, ability_id.clone(), *level);
        }

        let character = registry.characters.get(&char_id).unwrap();
        for la in &character.learnable_abilities {
            prop_assert!(
                la.required_level >= 1 && la.required_level <= 99,
                "required_level {} is out of range [1, 99] for ability {}",
                la.required_level,
                la.ability_id
            );
        }
    }

    /// **Validates: Requirements 3.1, 3.4, 3.7**
    ///
    /// Property 4 (clamping on add): Adding a learnable ability with an out-of-range level
    /// results in the stored level being clamped to [1, 99].
    #[test]
    fn add_learnable_ability_clamps_level(
        name in arb_display_name(),
        ability_id in arb_ability_id(),
        level in arb_any_level(),
    ) {
        let mut registry = CharacterRegistry::default();
        let char_id = registry.create_character(&name).unwrap();

        let result = registry.add_learnable_ability(&char_id, ability_id.clone(), level);
        prop_assert!(result.is_ok(), "add_learnable_ability should succeed, got: {:?}", result);

        let character = registry.characters.get(&char_id).unwrap();
        let entry = character
            .learnable_abilities
            .iter()
            .find(|la| la.ability_id == ability_id)
            .unwrap();

        let expected = level.clamp(1, 99);
        prop_assert_eq!(
            entry.required_level, expected,
            "Level {} should be clamped to {}, but got {}",
            level, expected, entry.required_level
        );
    }

    /// **Validates: Requirements 3.1, 3.4, 3.7**
    ///
    /// Property 4 (clamping on update): Updating a learnable ability's level with an
    /// out-of-range value results in the stored level being clamped to [1, 99].
    #[test]
    fn update_learnable_ability_clamps_level(
        name in arb_display_name(),
        ability_id in arb_ability_id(),
        initial_level in arb_valid_level(),
        new_level in arb_any_level(),
    ) {
        let mut registry = CharacterRegistry::default();
        let char_id = registry.create_character(&name).unwrap();

        // First add with a valid level
        registry
            .add_learnable_ability(&char_id, ability_id.clone(), initial_level)
            .unwrap();

        // Then update with an arbitrary level
        let result = registry.update_learnable_ability_level(&char_id, &ability_id, new_level);
        prop_assert!(result.is_ok(), "update should succeed, got: {:?}", result);

        let character = registry.characters.get(&char_id).unwrap();
        let entry = character
            .learnable_abilities
            .iter()
            .find(|la| la.ability_id == ability_id)
            .unwrap();

        let expected = new_level.clamp(1, 99);
        prop_assert_eq!(
            entry.required_level, expected,
            "Updated level {} should be clamped to {}, but got {}",
            new_level, expected, entry.required_level
        );
    }
}
