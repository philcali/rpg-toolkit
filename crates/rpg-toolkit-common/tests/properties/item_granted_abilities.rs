// Feature: database-editor-enhancements, Property 9: Item serialization round-trip with granted abilities

use proptest::prelude::*;

use rpg_toolkit_common::item::{ItemCategory, ItemRegistry};

/// Strategy for generating a valid item display name (1-64 chars, non-empty after trim).
fn arb_display_name() -> impl Strategy<Value = String> {
    "[a-zA-Z][a-zA-Z0-9 ]{0,63}".prop_filter("display name must be non-empty after trim", |s| {
        let trimmed = s.trim();
        !trimmed.is_empty() && trimmed.chars().count() <= 64
    })
}

/// Strategy for generating an arbitrary item category.
fn arb_item_category() -> impl Strategy<Value = ItemCategory> {
    prop_oneof![
        Just(ItemCategory::Weapon),
        Just(ItemCategory::Armor),
        Just(ItemCategory::Accessory),
        Just(ItemCategory::Consumable),
        Just(ItemCategory::KeyItem),
    ]
}

/// Strategy for generating a valid ability ID (non-empty, trimmed).
fn arb_ability_id() -> impl Strategy<Value = String> {
    "[a-zA-Z][a-zA-Z0-9_-]{0,31}"
}

/// Strategy for generating 0-4 unique ability IDs for equippable items.
fn arb_granted_abilities() -> impl Strategy<Value = Vec<String>> {
    proptest::collection::hash_set(arb_ability_id(), 0..=4)
        .prop_map(|set| set.into_iter().collect::<Vec<_>>())
}

/// Strategy for generating item entries: (display_name, category, granted_abilities).
/// Granted abilities are only meaningful for equippable items (Weapon/Armor/Accessory).
fn arb_item_entries() -> impl Strategy<Value = Vec<(String, ItemCategory, Vec<String>)>> {
    proptest::collection::vec(
        (
            arb_display_name(),
            arb_item_category(),
            arb_granted_abilities(),
        ),
        1..=10,
    )
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 100, .. ProptestConfig::default() })]

    /// **Validates: Requirements 5.10**
    ///
    /// Property 9: Item serialization round-trip with granted abilities.
    /// For any valid ItemRegistry containing items of various categories with granted abilities
    /// on equippable items, serializing to JSON and deserializing back shall produce a value
    /// equal to the original.
    #[test]
    fn item_serialization_round_trip_with_granted_abilities(
        entries in arb_item_entries(),
    ) {
        let mut registry = ItemRegistry::default();

        for (name, category, abilities) in &entries {
            let item_id = registry.create_item(name, *category).unwrap();

            // Only add granted abilities to equippable items (Weapon/Armor/Accessory)
            match category {
                ItemCategory::Weapon | ItemCategory::Armor | ItemCategory::Accessory => {
                    for ability_id in abilities {
                        // add_granted_ability rejects duplicates; our strategy generates unique IDs
                        registry.add_granted_ability(&item_id, ability_id).unwrap();
                    }
                }
                _ => {
                    // Consumable and KeyItem cannot have granted abilities; skip
                }
            }
        }

        // Serialize to JSON
        let json = serde_json::to_string(&registry).unwrap();

        // Deserialize back
        let deserialized: ItemRegistry = serde_json::from_str(&json).unwrap();

        // Assert equality
        prop_assert_eq!(&registry, &deserialized);
    }
}
