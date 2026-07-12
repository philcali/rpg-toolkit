// Feature: enemies-editor, Property 1: Serialization round-trip preserves registry equality
//
// For any valid EnemyRegistry containing 0–50 enemies with 1–20 stats,
// 0–10 item drops, 0–8 carried items, 0–7 elemental modifiers, 0–10
// abilities, and finite f64 values, serializing to JSON (as part of a
// ProjectFile) and then deserializing should produce an equivalent
// EnemyRegistry with identical enemies and field values.
//
// **Validates: Requirements 15.1, 15.4, 1.2, 1.9, 1.10**

use std::collections::HashMap;

use proptest::prelude::*;
use rpg_toolkit_common::{
    AbilityRegistry, CarriedItem, CharacterRegistry, DefeatReward, Element, ElementalModifier,
    Enemy, EnemyId, EnemyRegistry, EnemyStat, ItemDrop, ItemRegistry, ProjectFile,
};

// --- Arbitrary strategies ---

/// Generates a valid display name: 1-64 characters, at least one non-whitespace.
fn arb_display_name() -> impl Strategy<Value = String> {
    "[a-zA-Z][a-zA-Z0-9 ]{0,63}"
        .prop_map(|s| s.trim().to_string())
        .prop_filter("must not be empty after trim", |s| !s.is_empty())
}

/// Generates a description: 0-256 characters.
fn arb_description() -> impl Strategy<Value = String> {
    prop::collection::vec(prop::char::range('a', 'z'), 0..=256)
        .prop_map(|chars| chars.into_iter().collect::<String>())
}

/// Generates a non-empty item/ability ID string.
fn arb_id_string() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9_-]{1,32}"
}

/// Generates an arbitrary Element.
fn arb_element() -> impl Strategy<Value = Element> {
    prop_oneof![
        Just(Element::Fire),
        Just(Element::Ice),
        Just(Element::Lightning),
        Just(Element::Wind),
        Just(Element::Earth),
        Just(Element::Light),
        Just(Element::Dark),
    ]
}

/// Generates a stat name: 1-32 characters.
fn arb_stat_name() -> impl Strategy<Value = String> {
    "[a-zA-Z][a-zA-Z0-9 ]{0,31}"
        .prop_map(|s| s.trim().to_string())
        .prop_filter("must not be empty after trim", |s| !s.is_empty())
}

/// Generates a single EnemyStat.
fn arb_enemy_stat() -> impl Strategy<Value = EnemyStat> {
    (arb_stat_name(), any::<u32>()).prop_map(|(name, base_value)| EnemyStat { name, base_value })
}

/// Generates 1-20 unique stats, always including HP.
fn arb_stats() -> impl Strategy<Value = Vec<EnemyStat>> {
    (
        any::<u32>(),
        prop::collection::vec(arb_enemy_stat(), 0..=19),
    )
        .prop_map(|(hp_value, mut stats)| {
            // Deduplicate by name, keeping first
            let mut seen = std::collections::HashSet::new();
            seen.insert("HP".to_string());
            let mut result = vec![EnemyStat {
                name: "HP".to_string(),
                base_value: hp_value,
            }];
            for stat in stats.drain(..) {
                if stat.name != "HP" && seen.insert(stat.name.clone()) {
                    result.push(stat);
                    if result.len() >= 20 {
                        break;
                    }
                }
            }
            result
        })
}

/// Generates a finite f64 in [0.0, 1.0] for chance fields.
fn arb_chance() -> impl Strategy<Value = f64> {
    (0u32..=1000).prop_map(|n| n as f64 / 1000.0)
}

/// Generates a finite non-negative f64 for multiplier fields.
fn arb_multiplier() -> impl Strategy<Value = f64> {
    (0u32..=10000).prop_map(|n| n as f64 / 100.0)
}

/// Generates an ItemDrop with valid constraints.
fn arb_item_drop() -> impl Strategy<Value = ItemDrop> {
    (arb_id_string(), arb_chance()).prop_map(|(item_id, drop_chance)| ItemDrop {
        item_id,
        drop_chance,
    })
}

/// Generates 0-10 item drops.
fn arb_item_drops() -> impl Strategy<Value = Vec<ItemDrop>> {
    prop::collection::vec(arb_item_drop(), 0..=10)
}

/// Generates a DefeatReward.
fn arb_defeat_reward() -> impl Strategy<Value = DefeatReward> {
    (any::<u32>(), any::<u32>(), arb_item_drops()).prop_map(|(exp, gold, item_drops)| {
        DefeatReward {
            exp,
            gold,
            item_drops,
        }
    })
}

/// Generates a CarriedItem.
fn arb_carried_item() -> impl Strategy<Value = CarriedItem> {
    (arb_id_string(), arb_chance()).prop_map(|(item_id, obtain_chance)| CarriedItem {
        item_id,
        obtain_chance,
    })
}

/// Generates 0-8 carried items.
fn arb_carried_items() -> impl Strategy<Value = Vec<CarriedItem>> {
    prop::collection::vec(arb_carried_item(), 0..=8)
}

/// Generates an ElementalModifier.
fn arb_elemental_modifier() -> impl Strategy<Value = ElementalModifier> {
    (arb_element(), arb_multiplier()).prop_map(|(element, multiplier)| ElementalModifier {
        element,
        multiplier,
    })
}

/// Generates 0-7 elemental modifiers with no duplicate elements.
fn arb_elemental_modifiers() -> impl Strategy<Value = Vec<ElementalModifier>> {
    prop::collection::vec(arb_elemental_modifier(), 0..=7).prop_map(|mods| {
        let mut seen = std::collections::HashSet::new();
        mods.into_iter()
            .filter(|m| seen.insert(std::mem::discriminant(&m.element)))
            .collect()
    })
}

/// Generates 0-10 unique ability IDs.
fn arb_abilities() -> impl Strategy<Value = Vec<String>> {
    prop::collection::vec(arb_id_string(), 0..=10).prop_map(|ids| {
        let mut seen = std::collections::HashSet::new();
        ids.into_iter()
            .filter(|id| seen.insert(id.clone()))
            .collect()
    })
}

/// Generates an arbitrary Enemy with a given index-based ID.
fn arb_enemy(index: usize) -> impl Strategy<Value = (EnemyId, Enemy)> {
    (
        arb_display_name(),
        arb_description(),
        arb_stats(),
        arb_defeat_reward(),
        arb_carried_items(),
        arb_elemental_modifiers(),
        arb_abilities(),
    )
        .prop_map(
            move |(
                display_name,
                description,
                stats,
                defeat_rewards,
                carried_items,
                elemental_modifiers,
                abilities,
            )| {
                let id = format!("enemy-{:04}", index);
                let enemy = Enemy {
                    id: id.clone(),
                    display_name,
                    description,
                    stats,
                    defeat_rewards,
                    carried_items,
                    elemental_modifiers,
                    abilities,
                    portrait: None,
                };
                (id, enemy)
            },
        )
}

/// Generates an arbitrary EnemyRegistry with 0-50 enemies.
fn arb_enemy_registry() -> impl Strategy<Value = EnemyRegistry> {
    (0usize..=50).prop_flat_map(|count| {
        let enemies_strategy: Vec<_> = (0..count).map(arb_enemy).collect();

        enemies_strategy.prop_map(|enemies_vec| {
            let enemies: HashMap<EnemyId, Enemy> = enemies_vec.into_iter().collect();
            EnemyRegistry { enemies }
        })
    })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn enemy_registry_serialization_round_trip(registry in arb_enemy_registry()) {
        // Wrap in a minimal ProjectFile with empty maps, tilesets, etc.
        let project = ProjectFile::new(
            HashMap::new(), // no maps
            HashMap::new(), // no tilesets
            None,           // no spawn point
            HashMap::new(), // no spritesheets
            None,           // no player spritesheet
            HashMap::new(), // no dialog texts
            HashMap::new(), // no face portraits
            CharacterRegistry::default(),
            ItemRegistry::default(),
            AbilityRegistry::default(),
            registry.clone(),
            rpg_toolkit_common::ShopRegistry::default(),
        );

        // Serialize to JSON
        let json = project.serialize()
            .expect("serialization should succeed for valid ProjectFile");

        // Deserialize back
        let deserialized = ProjectFile::deserialize(&json)
            .expect("deserialization should succeed for valid ProjectFile");

        // Assert the enemy registries are equal
        prop_assert_eq!(&registry, &deserialized.enemies);
    }
}
