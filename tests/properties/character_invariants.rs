// Feature: character-editor, Property 2: Required stats invariant
//
// For any character created via `create_character`, and after any sequence of
// `add_stat` / `remove_stat` / `update_stat` operations, the character SHALL
// always contain exactly one stat named "HP" and exactly one stat named "Level".
//
// Validates: Requirements 1.5, 1.6, 4.5, 5.4

use proptest::prelude::*;
use rpg_toolkit_common::{CharacterRegistry, OPTIONAL_STATS, REQUIRED_STATS};

/// Represents a single stat operation to apply to a character.
#[derive(Clone, Debug)]
enum StatOp {
    Add(String),
    Remove(String),
    Update(String, u32, u32),
}

/// Strategy that generates a random stat operation.
/// Picks from optional stats, required stat names, and arbitrary strings.
fn arb_stat_op() -> impl Strategy<Value = StatOp> {
    // Pool of stat names to use in operations (includes required + optional + arbitrary)
    let stat_names: Vec<String> = REQUIRED_STATS
        .iter()
        .map(|(name, _, _)| name.to_string())
        .chain(OPTIONAL_STATS.iter().map(|s| s.to_string()))
        .collect();

    let stat_name_strategy = prop::sample::select(stat_names);

    prop_oneof![
        // AddStat with a name from the known pool
        stat_name_strategy.clone().prop_map(StatOp::Add),
        // RemoveStat with a name from the known pool (including required stats to test protection)
        stat_name_strategy.clone().prop_map(StatOp::Remove),
        // UpdateStat with a name from the known pool and random values
        (stat_name_strategy, any::<u32>(), any::<u32>())
            .prop_map(|(name, base, growth)| StatOp::Update(name, base, growth)),
    ]
}

/// Strategy that generates a sequence of stat operations.
fn arb_stat_ops() -> impl Strategy<Value = Vec<StatOp>> {
    prop::collection::vec(arb_stat_op(), 1..=20)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn required_stats_invariant(ops in arb_stat_ops()) {
        let mut registry = CharacterRegistry::default();

        // Create a character with a valid name
        let char_id = registry
            .create_character("TestHero")
            .expect("create_character should succeed with a valid name");

        // Apply random sequence of operations, ignoring errors
        for op in &ops {
            match op {
                StatOp::Add(name) => {
                    let _ = registry.add_stat(&char_id, name);
                }
                StatOp::Remove(name) => {
                    let _ = registry.remove_stat(&char_id, name);
                }
                StatOp::Update(name, base, growth) => {
                    let _ = registry.update_stat(&char_id, name, *base, *growth);
                }
            }
        }

        // After all operations, HP and Level must still be present
        let character = registry.characters.get(&char_id)
            .expect("Character should still exist in registry");

        let hp_count = character.stats.iter().filter(|s| s.name == "HP").count();
        let level_count = character.stats.iter().filter(|s| s.name == "Level").count();

        prop_assert_eq!(
            hp_count, 1,
            "Expected exactly 1 HP stat, found {} after ops: {:?}",
            hp_count, ops
        );
        prop_assert_eq!(
            level_count, 1,
            "Expected exactly 1 Level stat, found {} after ops: {:?}",
            level_count, ops
        );
    }
}
