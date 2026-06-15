// Feature: enemies-editor, Property 4: Operations on non-existent enemy ID return error without modification
//
// For any EnemyId not present in the registry, calling any mutation method SHALL return
// an EnemyValidationError whose message contains the missing ID, and the registry SHALL
// remain unchanged.
//
// **Validates: Requirements 3.2, 4.4, 5.6, 6.9, 7.7, 8.8**

use proptest::prelude::*;
use rpg_toolkit_common::{Element, EnemyRegistry};

/// Strategy for generating a non-existent enemy ID.
fn arb_nonexistent_id() -> impl Strategy<Value = String> {
    "[a-z]{3,8}".prop_map(|s| format!("nonexistent-{s}"))
}

/// Strategy for generating a valid stat name (1-32 chars).
fn arb_stat_name() -> impl Strategy<Value = String> {
    "[a-zA-Z]{1,10}"
}

/// Strategy for generating a valid item ID.
fn arb_item_id() -> impl Strategy<Value = String> {
    "[a-z]{3,8}".prop_map(|s| format!("item-{s}"))
}

/// Strategy for generating a valid ability ID.
fn arb_ability_id() -> impl Strategy<Value = String> {
    "[a-z]{3,8}".prop_map(|s| format!("ability-{s}"))
}

/// Strategy for generating a valid drop/obtain chance in [0.0, 1.0].
fn arb_chance() -> impl Strategy<Value = f64> {
    0.0..=1.0f64
}

/// Strategy for generating an element variant.
fn arb_element() -> impl Strategy<Value = Element> {
    prop::sample::select(Element::all())
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn nonexistent_id_operations_return_error_without_modification(
        nonexistent_id in arb_nonexistent_id(),
        stat_name in arb_stat_name(),
        item_id in arb_item_id(),
        ability_id in arb_ability_id(),
        chance in arb_chance(),
        element in arb_element(),
        base_value in 0u32..1000,
        index in 0usize..10,
    ) {
        // Set up a registry with one enemy so it's not empty
        let mut registry = EnemyRegistry::default();
        let _valid_id = registry.create_enemy("Test Enemy").expect("create should succeed");

        // Clone before operations
        let registry_before = registry.clone();

        // Test delete_enemy
        let result = registry.delete_enemy(&nonexistent_id);
        prop_assert!(result.is_err(), "delete_enemy should fail for nonexistent ID");
        prop_assert!(
            format!("{}", result.unwrap_err()).contains(&nonexistent_id),
            "Error should contain the nonexistent ID"
        );
        prop_assert_eq!(&registry, &registry_before, "Registry should be unchanged after failed delete_enemy");

        // Test rename_enemy
        let result = registry.rename_enemy(&nonexistent_id, "New Name");
        prop_assert!(result.is_err(), "rename_enemy should fail for nonexistent ID");
        prop_assert!(
            format!("{}", result.unwrap_err()).contains(&nonexistent_id),
            "Error should contain the nonexistent ID"
        );
        prop_assert_eq!(&registry, &registry_before, "Registry should be unchanged after failed rename_enemy");

        // Test update_description
        let result = registry.update_description(&nonexistent_id, "Some desc");
        prop_assert!(result.is_err(), "update_description should fail for nonexistent ID");
        prop_assert!(
            format!("{}", result.unwrap_err()).contains(&nonexistent_id),
            "Error should contain the nonexistent ID"
        );
        prop_assert_eq!(&registry, &registry_before, "Registry should be unchanged after failed update_description");

        // Test add_stat
        let result = registry.add_stat(&nonexistent_id, &stat_name);
        prop_assert!(result.is_err(), "add_stat should fail for nonexistent ID");
        prop_assert!(
            format!("{}", result.unwrap_err()).contains(&nonexistent_id),
            "Error should contain the nonexistent ID"
        );
        prop_assert_eq!(&registry, &registry_before, "Registry should be unchanged after failed add_stat");

        // Test remove_stat
        let result = registry.remove_stat(&nonexistent_id, &stat_name);
        prop_assert!(result.is_err(), "remove_stat should fail for nonexistent ID");
        prop_assert!(
            format!("{}", result.unwrap_err()).contains(&nonexistent_id),
            "Error should contain the nonexistent ID"
        );
        prop_assert_eq!(&registry, &registry_before, "Registry should be unchanged after failed remove_stat");

        // Test update_stat
        let result = registry.update_stat(&nonexistent_id, &stat_name, base_value);
        prop_assert!(result.is_err(), "update_stat should fail for nonexistent ID");
        prop_assert!(
            format!("{}", result.unwrap_err()).contains(&nonexistent_id),
            "Error should contain the nonexistent ID"
        );
        prop_assert_eq!(&registry, &registry_before, "Registry should be unchanged after failed update_stat");

        // Test update_exp
        let result = registry.update_exp(&nonexistent_id, base_value);
        prop_assert!(result.is_err(), "update_exp should fail for nonexistent ID");
        prop_assert!(
            format!("{}", result.unwrap_err()).contains(&nonexistent_id),
            "Error should contain the nonexistent ID"
        );
        prop_assert_eq!(&registry, &registry_before, "Registry should be unchanged after failed update_exp");

        // Test update_gold
        let result = registry.update_gold(&nonexistent_id, base_value);
        prop_assert!(result.is_err(), "update_gold should fail for nonexistent ID");
        prop_assert!(
            format!("{}", result.unwrap_err()).contains(&nonexistent_id),
            "Error should contain the nonexistent ID"
        );
        prop_assert_eq!(&registry, &registry_before, "Registry should be unchanged after failed update_gold");

        // Test add_item_drop
        let result = registry.add_item_drop(&nonexistent_id, &item_id, chance);
        prop_assert!(result.is_err(), "add_item_drop should fail for nonexistent ID");
        prop_assert!(
            format!("{}", result.unwrap_err()).contains(&nonexistent_id),
            "Error should contain the nonexistent ID"
        );
        prop_assert_eq!(&registry, &registry_before, "Registry should be unchanged after failed add_item_drop");

        // Test remove_item_drop
        let result = registry.remove_item_drop(&nonexistent_id, index);
        prop_assert!(result.is_err(), "remove_item_drop should fail for nonexistent ID");
        prop_assert!(
            format!("{}", result.unwrap_err()).contains(&nonexistent_id),
            "Error should contain the nonexistent ID"
        );
        prop_assert_eq!(&registry, &registry_before, "Registry should be unchanged after failed remove_item_drop");

        // Test add_carried_item
        let result = registry.add_carried_item(&nonexistent_id, &item_id, chance);
        prop_assert!(result.is_err(), "add_carried_item should fail for nonexistent ID");
        prop_assert!(
            format!("{}", result.unwrap_err()).contains(&nonexistent_id),
            "Error should contain the nonexistent ID"
        );
        prop_assert_eq!(&registry, &registry_before, "Registry should be unchanged after failed add_carried_item");

        // Test remove_carried_item
        let result = registry.remove_carried_item(&nonexistent_id, index);
        prop_assert!(result.is_err(), "remove_carried_item should fail for nonexistent ID");
        prop_assert!(
            format!("{}", result.unwrap_err()).contains(&nonexistent_id),
            "Error should contain the nonexistent ID"
        );
        prop_assert_eq!(&registry, &registry_before, "Registry should be unchanged after failed remove_carried_item");

        // Test add_elemental_modifier
        let result = registry.add_elemental_modifier(&nonexistent_id, element, 1.0);
        prop_assert!(result.is_err(), "add_elemental_modifier should fail for nonexistent ID");
        prop_assert!(
            format!("{}", result.unwrap_err()).contains(&nonexistent_id),
            "Error should contain the nonexistent ID"
        );
        prop_assert_eq!(&registry, &registry_before, "Registry should be unchanged after failed add_elemental_modifier");

        // Test update_elemental_modifier
        let result = registry.update_elemental_modifier(&nonexistent_id, element, 1.0);
        prop_assert!(result.is_err(), "update_elemental_modifier should fail for nonexistent ID");
        prop_assert!(
            format!("{}", result.unwrap_err()).contains(&nonexistent_id),
            "Error should contain the nonexistent ID"
        );
        prop_assert_eq!(&registry, &registry_before, "Registry should be unchanged after failed update_elemental_modifier");

        // Test remove_elemental_modifier
        let result = registry.remove_elemental_modifier(&nonexistent_id, element);
        prop_assert!(result.is_err(), "remove_elemental_modifier should fail for nonexistent ID");
        prop_assert!(
            format!("{}", result.unwrap_err()).contains(&nonexistent_id),
            "Error should contain the nonexistent ID"
        );
        prop_assert_eq!(&registry, &registry_before, "Registry should be unchanged after failed remove_elemental_modifier");

        // Test add_ability
        let result = registry.add_ability(&nonexistent_id, &ability_id);
        prop_assert!(result.is_err(), "add_ability should fail for nonexistent ID");
        prop_assert!(
            format!("{}", result.unwrap_err()).contains(&nonexistent_id),
            "Error should contain the nonexistent ID"
        );
        prop_assert_eq!(&registry, &registry_before, "Registry should be unchanged after failed add_ability");

        // Test remove_ability
        let result = registry.remove_ability(&nonexistent_id, &ability_id);
        prop_assert!(result.is_err(), "remove_ability should fail for nonexistent ID");
        prop_assert!(
            format!("{}", result.unwrap_err()).contains(&nonexistent_id),
            "Error should contain the nonexistent ID"
        );
        prop_assert_eq!(&registry, &registry_before, "Registry should be unchanged after failed remove_ability");
    }
}

// Feature: enemies-editor, Property 8: Validation failure preserves registry state
//
// For any operation violating validation rules (duplicate stat name, out-of-range
// probability/multiplier, capacity overflow, removing required stat "HP"), the operation
// SHALL return an error and the registry SHALL be identical (via PartialEq) to its state
// before the call.
//
// **Validates: Requirements 1.11, 5.3, 5.4, 5.8, 5.9, 6.4, 6.5, 6.6, 7.2, 7.3, 7.4, 8.2, 8.3**

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn validation_failure_preserves_registry_state(
        invalid_chance_low in -10.0f64..-0.001,
        invalid_chance_high in 1.001f64..10.0,
        negative_multiplier in -10.0f64..-0.001,
        element in arb_element(),
    ) {
        let mut registry = EnemyRegistry::default();
        let id = registry.create_enemy("Validation Test Enemy").expect("create should succeed");

        // 1. add_stat with duplicate stat name → error, registry unchanged
        // "HP" is already a default stat
        {
            let before = registry.clone();
            let result = registry.add_stat(&id, "HP");
            prop_assert!(result.is_err(), "add_stat with duplicate name should fail");
            prop_assert_eq!(&registry, &before, "Registry should be unchanged after duplicate stat");
        }

        // 2. add_stat when enemy already has 20 stats → error, registry unchanged
        {
            // Add 16 more stats to reach 20 (enemy starts with 4: HP, Attack, Defense, Speed)
            for i in 0..16 {
                registry.add_stat(&id, &format!("Stat{i}")).expect("adding stat should succeed");
            }
            let before = registry.clone();
            let result = registry.add_stat(&id, "OverflowStat");
            prop_assert!(result.is_err(), "add_stat at capacity should fail");
            prop_assert_eq!(&registry, &before, "Registry should be unchanged after stat capacity overflow");
        }

        // 3. remove_stat for "HP" → error, registry unchanged
        {
            let before = registry.clone();
            let result = registry.remove_stat(&id, "HP");
            prop_assert!(result.is_err(), "remove_stat HP should fail");
            prop_assert_eq!(&registry, &before, "Registry should be unchanged after removing HP");
        }

        // 4. add_item_drop with drop_chance outside 0.0-1.0 (low) → error, registry unchanged
        {
            let before = registry.clone();
            let result = registry.add_item_drop(&id, "some-item", invalid_chance_low);
            prop_assert!(result.is_err(), "add_item_drop with low chance should fail");
            prop_assert_eq!(&registry, &before, "Registry should be unchanged after invalid drop chance (low)");
        }

        // add_item_drop with drop_chance outside 0.0-1.0 (high) → error, registry unchanged
        {
            let before = registry.clone();
            let result = registry.add_item_drop(&id, "some-item", invalid_chance_high);
            prop_assert!(result.is_err(), "add_item_drop with high chance should fail");
            prop_assert_eq!(&registry, &before, "Registry should be unchanged after invalid drop chance (high)");
        }

        // 5. add_item_drop with empty item_id → error, registry unchanged
        {
            let before = registry.clone();
            let result = registry.add_item_drop(&id, "", 0.5);
            prop_assert!(result.is_err(), "add_item_drop with empty item_id should fail");
            prop_assert_eq!(&registry, &before, "Registry should be unchanged after empty item_id drop");
        }

        // 6. add_item_drop when already at 10 item drops → error, registry unchanged
        {
            for i in 0..10 {
                registry.add_item_drop(&id, &format!("drop-item-{i}"), 0.5)
                    .expect("adding item drop should succeed");
            }
            let before = registry.clone();
            let result = registry.add_item_drop(&id, "overflow-item", 0.5);
            prop_assert!(result.is_err(), "add_item_drop at capacity should fail");
            prop_assert_eq!(&registry, &before, "Registry should be unchanged after item drop capacity overflow");
        }

        // 7. add_carried_item with obtain_chance outside 0.0-1.0 → error, registry unchanged
        {
            let before = registry.clone();
            let result = registry.add_carried_item(&id, "some-item", invalid_chance_low);
            prop_assert!(result.is_err(), "add_carried_item with low chance should fail");
            prop_assert_eq!(&registry, &before, "Registry should be unchanged after invalid obtain chance (low)");
        }
        {
            let before = registry.clone();
            let result = registry.add_carried_item(&id, "some-item", invalid_chance_high);
            prop_assert!(result.is_err(), "add_carried_item with high chance should fail");
            prop_assert_eq!(&registry, &before, "Registry should be unchanged after invalid obtain chance (high)");
        }

        // 8. add_carried_item with empty item_id → error, registry unchanged
        {
            let before = registry.clone();
            let result = registry.add_carried_item(&id, "", 0.5);
            prop_assert!(result.is_err(), "add_carried_item with empty item_id should fail");
            prop_assert_eq!(&registry, &before, "Registry should be unchanged after empty item_id carried");
        }

        // 9. add_carried_item when already at 8 carried items → error, registry unchanged
        {
            for i in 0..8 {
                registry.add_carried_item(&id, &format!("carried-item-{i}"), 0.5)
                    .expect("adding carried item should succeed");
            }
            let before = registry.clone();
            let result = registry.add_carried_item(&id, "overflow-carried", 0.5);
            prop_assert!(result.is_err(), "add_carried_item at capacity should fail");
            prop_assert_eq!(&registry, &before, "Registry should be unchanged after carried item capacity overflow");
        }

        // 10. add_elemental_modifier with negative multiplier → error, registry unchanged
        {
            let before = registry.clone();
            let result = registry.add_elemental_modifier(&id, element, negative_multiplier);
            prop_assert!(result.is_err(), "add_elemental_modifier with negative multiplier should fail");
            prop_assert_eq!(&registry, &before, "Registry should be unchanged after negative multiplier");
        }

        // 11. add_elemental_modifier with duplicate element → error, registry unchanged
        {
            // First add the element successfully
            registry.add_elemental_modifier(&id, element, 1.0)
                .expect("adding elemental modifier should succeed");
            let before = registry.clone();
            let result = registry.add_elemental_modifier(&id, element, 2.0);
            prop_assert!(result.is_err(), "add_elemental_modifier with duplicate element should fail");
            prop_assert_eq!(&registry, &before, "Registry should be unchanged after duplicate element");
        }
    }
}
