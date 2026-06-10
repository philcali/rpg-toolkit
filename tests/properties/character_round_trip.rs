// Feature: character-editor, Property 1: Character serialization round-trip
//
// For any valid CharacterRegistry containing zero or more characters with
// arbitrary stat configurations, serializing to JSON (as part of a ProjectFile)
// and then deserializing should produce an equivalent CharacterRegistry with
// identical characters, stats, and field values.
//
// **Validates: Requirements 2.1, 2.2**

use std::collections::HashMap;

use proptest::prelude::*;
use rpg_toolkit_common::{
    Character, CharacterId, CharacterRegistry, OPTIONAL_STATS, ProjectFile, REQUIRED_STATS, Stat,
};

// --- Arbitrary strategies ---

/// Generates a display name: 1-64 non-empty characters (at least one non-whitespace).
fn arb_display_name() -> impl Strategy<Value = String> {
    "[a-zA-Z][a-zA-Z0-9 ]{0,63}"
        .prop_map(|s| s.trim().to_string())
        .prop_filter("must not be empty after trim", |s| !s.is_empty())
}

/// Generates a stat with arbitrary base/growth u32 values.
fn arb_stat(name: String) -> impl Strategy<Value = Stat> {
    (any::<u32>(), any::<u32>()).prop_map(move |(base_value, growth_value)| Stat {
        name: name.clone(),
        base_value,
        growth_value,
    })
}

/// Generates 0-7 optional stats chosen from OPTIONAL_STATS.
fn arb_optional_stats() -> impl Strategy<Value = Vec<Stat>> {
    // Pick a subset of optional stats (0 to 7)
    (0usize..=7).prop_flat_map(|count| {
        let count = count.min(OPTIONAL_STATS.len());
        // Use a shuffled selection by generating a bool mask
        prop::collection::vec(any::<bool>(), OPTIONAL_STATS.len()).prop_flat_map(move |mask| {
            // Take the first `count` trues as selected stats
            let selected: Vec<String> = mask
                .iter()
                .enumerate()
                .filter(|(_, b)| **b)
                .take(count)
                .map(|(i, _)| OPTIONAL_STATS[i].to_string())
                .collect();

            // Generate stats for selected names
            selected.into_iter().map(arb_stat).collect::<Vec<_>>()
        })
    })
}

/// Generates a full set of stats: required stats (HP, Level) with arbitrary values + optional stats.
fn arb_stats() -> impl Strategy<Value = Vec<Stat>> {
    // Generate required stats with arbitrary base/growth values
    let required = REQUIRED_STATS
        .iter()
        .map(|(name, _, _)| arb_stat(name.to_string()))
        .collect::<Vec<_>>();

    (required, arb_optional_stats()).prop_map(|(req_stats, opt_stats)| {
        let mut all_stats = req_stats;
        all_stats.extend(opt_stats);
        all_stats
    })
}

/// Generates a single Character with a fixed ID format.
fn arb_character(index: usize) -> impl Strategy<Value = (CharacterId, Character)> {
    (arb_display_name(), arb_stats()).prop_map(move |(display_name, stats)| {
        let id = format!("char-{}", index);
        let character = Character {
            id: id.clone(),
            display_name,
            stats,
        };
        (id, character)
    })
}

/// Generates an arbitrary CharacterRegistry with 0-5 characters.
fn arb_character_registry() -> impl Strategy<Value = CharacterRegistry> {
    (0usize..=5).prop_flat_map(|count| {
        let characters_strategy: Vec<_> = (0..count).map(arb_character).collect();

        characters_strategy.prop_map(|chars| {
            let characters: HashMap<CharacterId, Character> = chars.into_iter().collect();
            CharacterRegistry { characters }
        })
    })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn character_registry_serialization_round_trip(registry in arb_character_registry()) {
        // Wrap in a minimal ProjectFile
        let project = ProjectFile::new(
            HashMap::new(), // no maps
            HashMap::new(), // no tilesets
            None,           // no spawn point
            HashMap::new(), // no spritesheets
            None,           // no player spritesheet
            HashMap::new(), // no dialog texts
            HashMap::new(), // no face portraits
            registry.clone(),
            rpg_toolkit_common::ItemRegistry::default(),
        );

        // Serialize to JSON
        let json = project.serialize()
            .expect("serialization should succeed for valid ProjectFile");

        // Deserialize back
        let deserialized = ProjectFile::deserialize(&json)
            .expect("deserialization should succeed for valid ProjectFile");

        // Assert the character registries are equal
        prop_assert_eq!(&registry, &deserialized.characters);
    }
}
