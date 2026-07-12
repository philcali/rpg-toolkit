// Feature: items-editor, Property 1: Item serialization round-trip
//
// For any valid ItemRegistry containing zero or more items with arbitrary
// categories, stat modifiers, consumable effects, rarity values, and stack
// configurations, serializing to JSON (as part of a ProjectFile) and then
// deserializing should produce an equivalent ItemRegistry with identical
// items and field values.
//
// **Validates: Requirements 3.2, 3.6**

use std::collections::HashMap;

use proptest::prelude::*;
use rpg_toolkit_common::{
    BuffTargetStat, CharacterRegistry, ConsumableEffect, ConsumableEffectType, CureTargetStatus,
    EquipmentSlot, Item, ItemCategoryData, ItemId, ItemRegistry, ProjectFile, Rarity, StatModifier,
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

/// Generates an arbitrary Rarity.
fn arb_rarity() -> impl Strategy<Value = Rarity> {
    prop_oneof![
        Just(Rarity::Common),
        Just(Rarity::Uncommon),
        Just(Rarity::Rare),
        Just(Rarity::Epic),
        Just(Rarity::Legendary),
    ]
}

/// Generates an arbitrary EquipmentSlot.
fn arb_equipment_slot() -> impl Strategy<Value = EquipmentSlot> {
    prop_oneof![
        Just(EquipmentSlot::MainHand),
        Just(EquipmentSlot::OffHand),
        Just(EquipmentSlot::Head),
        Just(EquipmentSlot::Body),
        Just(EquipmentSlot::Legs),
        Just(EquipmentSlot::Feet),
        Just(EquipmentSlot::Accessory1),
        Just(EquipmentSlot::Accessory2),
    ]
}

/// Generates an arbitrary BuffTargetStat.
fn arb_buff_target_stat() -> impl Strategy<Value = BuffTargetStat> {
    prop_oneof![
        Just(BuffTargetStat::Strength),
        Just(BuffTargetStat::Stamina),
        Just(BuffTargetStat::Speed),
        Just(BuffTargetStat::Luck),
        Just(BuffTargetStat::Wisdom),
        Just(BuffTargetStat::Intelligence),
    ]
}

/// Generates an arbitrary CureTargetStatus.
fn arb_cure_target_status() -> impl Strategy<Value = CureTargetStatus> {
    prop_oneof![
        Just(CureTargetStatus::Poison),
        Just(CureTargetStatus::Paralysis),
        Just(CureTargetStatus::Sleep),
        Just(CureTargetStatus::Confusion),
        Just(CureTargetStatus::Silence),
        Just(CureTargetStatus::All),
    ]
}

/// Generates an arbitrary ConsumableEffectType.
fn arb_consumable_effect_type() -> impl Strategy<Value = ConsumableEffectType> {
    prop_oneof![
        Just(ConsumableEffectType::RestoreHP),
        Just(ConsumableEffectType::RestoreMP),
        arb_cure_target_status()
            .prop_map(|target_status| ConsumableEffectType::CureStatus { target_status }),
        (arb_buff_target_stat(), 1u32..=100).prop_map(|(target_stat, duration)| {
            ConsumableEffectType::BuffStat {
                target_stat,
                duration,
            }
        }),
    ]
}

/// Generates an arbitrary ConsumableEffect with potency >= 1.
fn arb_consumable_effect() -> impl Strategy<Value = ConsumableEffect> {
    (arb_consumable_effect_type(), 1u32..=1000)
        .prop_map(|(effect, potency)| ConsumableEffect { effect, potency })
}

/// Generates an arbitrary StatModifier.
fn arb_stat_modifier() -> impl Strategy<Value = StatModifier> {
    ("[a-zA-Z]{1,32}", any::<i32>())
        .prop_map(|(stat_name, value)| StatModifier { stat_name, value })
}

/// Generates 0-5 unique stat modifiers.
fn arb_stat_modifiers() -> impl Strategy<Value = Vec<StatModifier>> {
    prop::collection::vec(arb_stat_modifier(), 0..=5).prop_map(|mods| {
        // Deduplicate by stat_name (keep first occurrence)
        let mut seen = std::collections::HashSet::new();
        mods.into_iter()
            .filter(|m| seen.insert(m.stat_name.clone()))
            .collect()
    })
}

/// Generates an arbitrary ItemCategoryData covering all 5 categories.
fn arb_item_category_data() -> impl Strategy<Value = ItemCategoryData> {
    prop_oneof![
        (any::<u32>(), arb_equipment_slot()).prop_map(|(attack_power, equipment_slot)| {
            ItemCategoryData::Weapon {
                attack_power,
                equipment_slot,
            }
        }),
        (any::<u32>(), arb_equipment_slot()).prop_map(|(defense_power, equipment_slot)| {
            ItemCategoryData::Armor {
                defense_power,
                equipment_slot,
            }
        }),
        arb_equipment_slot()
            .prop_map(|equipment_slot| { ItemCategoryData::Accessory { equipment_slot } }),
        prop::collection::vec(arb_consumable_effect(), 1..=4)
            .prop_map(|effects| ItemCategoryData::Consumable { effects }),
        Just(ItemCategoryData::KeyItem),
    ]
}

/// Generates an arbitrary Item with a unique UUID id.
fn arb_item(index: usize) -> impl Strategy<Value = (ItemId, Item)> {
    (
        arb_display_name(),
        arb_description(),
        arb_item_category_data(),
        any::<u32>(),
        arb_rarity(),
        any::<bool>(),
        2u32..=999,
        arb_stat_modifiers(),
    )
        .prop_map(
            move |(
                display_name,
                description,
                category_data,
                value,
                rarity,
                stackable_raw,
                stack_limit_raw,
                stat_modifiers,
            )| {
                let id = format!("item-{:04}", index);

                // Enforce category-specific constraints
                let (value, stackable, stack_limit) = match &category_data {
                    ItemCategoryData::Consumable { .. } => {
                        // Consumable items must be stackable
                        (value, true, stack_limit_raw)
                    }
                    ItemCategoryData::KeyItem => {
                        // Key items must not be stackable and value must be 0
                        (0, false, 1)
                    }
                    _ => {
                        // Other categories respect the generated stackable flag
                        if stackable_raw {
                            (value, true, stack_limit_raw)
                        } else {
                            (value, false, 1)
                        }
                    }
                };

                let item = Item {
                    id: id.clone(),
                    display_name,
                    description,
                    category_data,
                    value,
                    rarity,
                    stackable,
                    stack_limit,
                    stat_modifiers,
                    granted_abilities: Vec::new(),
                };
                (id, item)
            },
        )
}

/// Generates an arbitrary ItemRegistry with 0-8 items.
fn arb_item_registry() -> impl Strategy<Value = ItemRegistry> {
    (0usize..=8).prop_flat_map(|count| {
        let items_strategy: Vec<_> = (0..count).map(arb_item).collect();

        items_strategy.prop_map(|items_vec| {
            let items: HashMap<ItemId, Item> = items_vec.into_iter().collect();
            ItemRegistry { items }
        })
    })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn item_registry_serialization_round_trip(registry in arb_item_registry()) {
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
            registry.clone(),
            rpg_toolkit_common::AbilityRegistry::default(),
            rpg_toolkit_common::EnemyRegistry::default(),
            rpg_toolkit_common::ShopRegistry::default(),
        );

        // Serialize to JSON
        let json = project.serialize()
            .expect("serialization should succeed for valid ProjectFile");

        // Deserialize back
        let deserialized = ProjectFile::deserialize(&json)
            .expect("deserialization should succeed for valid ProjectFile");

        // Assert the item registries are equal
        prop_assert_eq!(&registry, &deserialized.items);
    }
}
