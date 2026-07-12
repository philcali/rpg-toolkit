// Feature: in-game-shops, Property 1: Shop name validation
// Feature: in-game-shops, Property 2: No duplicate items per shop
// Feature: in-game-shops, Property 4: Shop list case-insensitive sorting

use std::collections::HashSet;

use proptest::collection;
use proptest::prelude::*;

use rpg_toolkit_common::shop::{ShopEntry, ShopRegistry};

/// Strategy for generating arbitrary strings (including whitespace, empty, etc.)
/// to test shop name validation boundaries.
fn arb_shop_name_input() -> impl Strategy<Value = String> {
    prop_oneof![
        // Empty strings
        Just(String::new()),
        // Whitespace-only strings
        "[ \\t\\n]{1,10}".prop_map(|s| s),
        // Valid names (1-64 chars after trim)
        "[a-zA-Z][a-zA-Z0-9 ]{0,63}".prop_map(|s| s),
        // Names with leading/trailing whitespace around valid content
        ("[ \\t]{0,5}", "[a-zA-Z][a-zA-Z0-9 ]{0,63}", "[ \\t]{0,5}")
            .prop_map(|(pre, mid, post)| format!("{}{}{}", pre, mid, post)),
        // Names that are too long (65+ chars after trim)
        "[a-zA-Z0-9]{65,100}".prop_map(|s| s),
    ]
}

/// Strategy for generating distinct item IDs.
fn arb_distinct_item_ids(max_count: usize) -> impl Strategy<Value = Vec<String>> {
    collection::hash_set("[a-z]{3,8}-[0-9]{1,4}", 1..=max_count)
        .prop_map(|set| set.into_iter().collect::<Vec<_>>())
}

/// Strategy for generating shop display names that are valid (1-64 trimmed chars).
fn arb_valid_shop_name() -> impl Strategy<Value = String> {
    "[a-zA-Z][a-zA-Z0-9 ]{0,63}".prop_filter("must be non-empty after trim", |s| {
        let trimmed = s.trim();
        !trimmed.is_empty() && trimmed.len() <= 64
    })
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 100, .. ProptestConfig::default() })]

    /// **Validates: Requirements 1.3, 2.10**
    ///
    /// Property 1: Shop name validation — For any string input, name validator accepts
    /// iff trimmed length is between 1 and 64 (inclusive).
    #[test]
    fn shop_name_validation_accepts_iff_trimmed_1_to_64(name in arb_shop_name_input()) {
        let mut registry = ShopRegistry::default();
        let trimmed = name.trim();
        let expected_valid = !trimmed.is_empty() && trimmed.len() <= 64;

        let result = registry.create_shop(&name);

        if expected_valid {
            prop_assert!(
                result.is_ok(),
                "Expected name {:?} (trimmed: {:?}, len: {}) to be accepted, but got error",
                name, trimmed, trimmed.len()
            );
            // Verify the stored name is the trimmed version
            let id = result.unwrap();
            prop_assert_eq!(
                &registry.shops[&id].display_name,
                trimmed,
                "Stored name should be the trimmed input"
            );
        } else {
            prop_assert!(
                result.is_err(),
                "Expected name {:?} (trimmed: {:?}, len: {}) to be rejected, but was accepted",
                name, trimmed, trimmed.len()
            );
            // Registry should remain unchanged
            prop_assert_eq!(
                registry.shops.len(), 0,
                "Registry should not be modified on validation failure"
            );
        }
    }

    /// **Validates: Requirements 1.3, 2.10**
    ///
    /// Property 1 (rename): rename_shop validates name the same way —
    /// accepts iff trimmed length is between 1 and 64.
    #[test]
    fn shop_rename_validation_accepts_iff_trimmed_1_to_64(
        name in arb_shop_name_input()
    ) {
        let mut registry = ShopRegistry::default();
        let shop_id = registry.create_shop("Initial Name").unwrap();
        let trimmed = name.trim();
        let expected_valid = !trimmed.is_empty() && trimmed.len() <= 64;

        let result = registry.rename_shop(&shop_id, &name);

        if expected_valid {
            prop_assert!(
                result.is_ok(),
                "Expected rename to {:?} (trimmed: {:?}, len: {}) to succeed",
                name, trimmed, trimmed.len()
            );
            prop_assert_eq!(
                &registry.shops[&shop_id].display_name,
                trimmed,
                "Stored name should be the trimmed input after rename"
            );
        } else {
            prop_assert!(
                result.is_err(),
                "Expected rename to {:?} (trimmed: {:?}, len: {}) to fail",
                name, trimmed, trimmed.len()
            );
            // Name should remain unchanged
            prop_assert_eq!(
                &registry.shops[&shop_id].display_name,
                "Initial Name",
                "Display name should not change on validation failure"
            );
        }
    }

    /// **Validates: Requirements 1.7, 2.9**
    ///
    /// Property 2: No duplicate items per shop — add_entry sequence with distinct IDs
    /// never produces duplicate ItemIds; adding a duplicate returns an error.
    #[test]
    fn no_duplicate_items_in_shop(item_ids in arb_distinct_item_ids(20)) {
        let mut registry = ShopRegistry::default();
        let shop_id = registry.create_shop("Test Shop").unwrap();

        // Add all distinct items — should all succeed
        for item_id in &item_ids {
            let entry = ShopEntry {
                item_id: item_id.clone(),
                buy_price: 100,
                sell_price: None,
                stock_limit: None,
                condition: None,
            };
            let result = registry.add_entry(&shop_id, entry);
            prop_assert!(
                result.is_ok(),
                "Adding unique item {:?} should succeed",
                item_id
            );
        }

        // Verify no duplicates in the entries
        let entries = &registry.shops[&shop_id].entries;
        let mut seen_ids = HashSet::new();
        for entry in entries {
            prop_assert!(
                seen_ids.insert(&entry.item_id),
                "Found duplicate item_id {:?} in shop entries",
                entry.item_id
            );
        }

        // Verify count matches
        prop_assert_eq!(
            entries.len(),
            item_ids.len(),
            "Entry count should match number of distinct items added"
        );

        // Now try adding duplicates — each should fail
        for item_id in &item_ids {
            let duplicate_entry = ShopEntry {
                item_id: item_id.clone(),
                buy_price: 200,
                sell_price: Some(50),
                stock_limit: Some(10),
                condition: None,
            };
            let entries_before = registry.shops[&shop_id].entries.clone();
            let result = registry.add_entry(&shop_id, duplicate_entry);
            prop_assert!(
                result.is_err(),
                "Adding duplicate item {:?} should return an error",
                item_id
            );
            // Entries should be unchanged after rejected duplicate
            prop_assert_eq!(
                &registry.shops[&shop_id].entries,
                &entries_before,
                "Entry list should remain unchanged after rejected duplicate"
            );
        }
    }

    /// **Validates: Requirements 2.2**
    ///
    /// Property 4: Shop list case-insensitive sorting — sorted output is
    /// non-decreasing in case-insensitive lexicographic order.
    #[test]
    fn sorted_shops_case_insensitive_order(
        names in collection::vec(arb_valid_shop_name(), 0..=30)
    ) {
        let mut registry = ShopRegistry::default();

        for name in &names {
            registry.create_shop(name).unwrap();
        }

        let sorted = registry.sorted_shops();

        // Verify the output is in non-decreasing case-insensitive order
        for window in sorted.windows(2) {
            let a = window[0].display_name.to_lowercase();
            let b = window[1].display_name.to_lowercase();
            prop_assert!(
                a <= b,
                "Sorted shops not in non-decreasing case-insensitive order: {:?} > {:?}",
                window[0].display_name,
                window[1].display_name
            );
        }

        // Verify all shops are present in the sorted output
        prop_assert_eq!(
            sorted.len(),
            registry.shops.len(),
            "sorted_shops() should return all shops"
        );
    }
}
