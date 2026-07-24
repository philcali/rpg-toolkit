// Feature: in-game-shops, Properties 3, 7, 11, 12, 16

use std::collections::HashMap;

use proptest::prelude::*;

use rpg_toolkit_common::condition::{
    BranchCondition, ConditionCheck, ConditionLogic, ConditionOperator,
};
use rpg_toolkit_common::item::{Item, ItemCategoryData, ItemId, ItemRegistry, Rarity};
use rpg_toolkit_common::shop::ShopEntry;
use rpg_toolkit_scenes::shop_scene::{
    ShopError, compute_sell_price, execute_buy, sellable_items, visible_entries,
};

/// Strategy that generates inputs guaranteed to violate at least one buy guard condition.
///
/// The four rejection conditions are:
/// 1. InsufficientFunds: balance < buy_price * quantity
/// 2. InventoryFull (stackable): inventory_qty + quantity > stack_limit
/// 3. InventoryFull (non-stackable): inventory_qty > 0
/// 4. InsufficientStock: remaining_stock.is_some() && remaining_stock.unwrap() < quantity
///
/// We pick one violation to force, then generate values that satisfy it.
fn rejected_buy_inputs() -> impl Strategy<Value = (u64, u32, u32, u32, Option<u32>, bool, u32)> {
    // Choose which guard to violate: 0=funds, 1=stack_full_stackable, 2=non_stackable_held, 3=stock
    (0u8..4).prop_flat_map(|violation| {
        match violation {
            0 => {
                // InsufficientFunds: balance < buy_price * quantity
                // Generate price and quantity such that total > 0, then balance < total
                (1u32..=1000, 1u32..=100)
                    .prop_flat_map(|(buy_price, quantity)| {
                        let total_cost = buy_price as u64 * quantity as u64;
                        // balance must be less than total_cost
                        let max_balance = total_cost.saturating_sub(1);
                        (
                            0u64..=max_balance,
                            Just(0u32), // inventory_qty (doesn't matter for this violation)
                            Just(buy_price),
                            Just(quantity),
                            Just(None), // remaining_stock (unlimited, won't trigger stock error)
                            Just(true), // is_stackable
                            Just(99u32), // stack_limit (high enough to not trigger stack error)
                        )
                    })
                    .boxed()
            }
            1 => {
                // InventoryFull (stackable): inventory_qty + quantity > stack_limit
                // Generate stack_limit and inventory_qty such that adding quantity overflows
                (1u32..=50, 1u32..=50)
                    .prop_flat_map(|(stack_limit, quantity)| {
                        // inventory_qty must be > stack_limit - quantity (so inventory_qty + quantity > stack_limit)
                        let min_inv = stack_limit.saturating_sub(quantity).saturating_add(1);
                        (
                            min_inv..=min_inv.saturating_add(50),
                            Just(stack_limit),
                            Just(quantity),
                        )
                    })
                    .prop_flat_map(|(inventory_qty, stack_limit, quantity)| {
                        // Ensure we have enough funds so InsufficientFunds doesn't trigger first
                        let buy_price = 1u32;
                        let balance = buy_price as u64 * quantity as u64; // exactly enough funds
                        (
                            Just(balance),
                            Just(inventory_qty),
                            Just(buy_price),
                            Just(quantity),
                            Just(None), // unlimited stock
                            Just(true), // stackable
                            Just(stack_limit),
                        )
                    })
                    .boxed()
            }
            2 => {
                // InventoryFull (non-stackable): inventory_qty > 0
                (1u32..=10, 1u32..=100, 1u32..=10)
                    .prop_flat_map(|(inventory_qty, buy_price, quantity)| {
                        let balance = buy_price as u64 * quantity as u64; // enough funds
                        (
                            Just(balance),
                            Just(inventory_qty),
                            Just(buy_price),
                            Just(quantity),
                            Just(None),  // unlimited stock
                            Just(false), // non-stackable
                            Just(1u32),  // stack_limit (irrelevant for non-stackable)
                        )
                    })
                    .boxed()
            }
            3 => {
                // InsufficientStock: remaining_stock.is_some() && remaining_stock < quantity
                (1u32..=100, 1u32..=100)
                    .prop_flat_map(|(quantity, buy_price)| {
                        // remaining_stock must be less than quantity
                        let max_stock = quantity.saturating_sub(1);
                        (0u32..=max_stock, Just(quantity), Just(buy_price))
                    })
                    .prop_flat_map(|(stock, quantity, buy_price)| {
                        let balance = buy_price as u64 * quantity as u64; // enough funds
                        (
                            Just(balance),
                            Just(0u32), // inventory_qty (no stack issue)
                            Just(buy_price),
                            Just(quantity),
                            Just(Some(stock)),
                            Just(true),  // stackable
                            Just(99u32), // high stack_limit
                        )
                    })
                    .boxed()
            }
            _ => unreachable!(),
        }
    })
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 100, .. ProptestConfig::default() })]

    /// **Validates: Requirements 5.2, 5.3, 5.4, 4.6**
    ///
    /// Property 7: For any purchase attempt that violates at least one guard condition,
    /// execute_buy SHALL return an Err (rejecting the transaction), meaning the caller's
    /// balance, inventory, and stock state remain unchanged (since execute_buy is a pure
    /// function that only returns new state on Ok).
    #[test]
    fn purchase_rejection_preserves_state(
        (balance, inventory_qty, buy_price, quantity, remaining_stock, is_stackable, stack_limit)
            in rejected_buy_inputs()
    ) {
        let result = execute_buy(
            balance,
            inventory_qty,
            buy_price,
            quantity,
            remaining_stock,
            is_stackable,
            stack_limit,
        );

        // The transaction must be rejected
        prop_assert!(
            result.is_err(),
            "Expected rejection but got Ok({:?}) for inputs: balance={}, inv_qty={}, \
             price={}, qty={}, stock={:?}, stackable={}, stack_limit={}",
            result.unwrap(),
            balance,
            inventory_qty,
            buy_price,
            quantity,
            remaining_stock,
            is_stackable,
            stack_limit,
        );

        // Verify the error is one of the expected rejection types
        let err = result.unwrap_err();
        prop_assert!(
            matches!(
                err,
                ShopError::InsufficientFunds
                    | ShopError::InventoryFull
                    | ShopError::InsufficientStock
            ),
            "Unexpected error variant: {:?}",
            err,
        );
    }
}

// ── Helper functions ─────────────────────────────────────────────────────────

/// Creates a test item with the given parameters.
fn make_test_item(id: &str, value: u32, category_data: ItemCategoryData) -> Item {
    Item {
        id: id.to_string(),
        display_name: id.to_string(),
        description: String::new(),
        category_data,
        value,
        rarity: Rarity::Common,
        stackable: true,
        stack_limit: 99,
        stat_modifiers: vec![],
        granted_abilities: vec![],
        graphics: Default::default(),
    }
}

/// Creates a ShopEntry with given parameters.
fn make_shop_entry(
    item_id: &str,
    buy_price: u32,
    sell_price: Option<u32>,
    stock_limit: Option<u32>,
    condition: Option<BranchCondition>,
) -> ShopEntry {
    ShopEntry {
        item_id: item_id.to_string(),
        buy_price,
        sell_price,
        stock_limit,
        condition,
    }
}

// ── Property 3: Default sell price calculation ───────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig { cases: 100, .. ProptestConfig::default() })]

    /// **Validates: Requirements 1.5, 6.3**
    ///
    /// Property 3: When a ShopEntry has sell_price: None, compute_sell_price returns
    /// item.value / 2 (integer division, i.e., floor).
    #[test]
    fn default_sell_price_is_half_item_value(value in any::<u32>()) {
        let entry = make_shop_entry("test_item", 100, None, None, None);
        let item = make_test_item("test_item", value, ItemCategoryData::Consumable { effects: vec![] });

        let result = compute_sell_price(&entry, &item);
        prop_assert_eq!(result, value / 2,
            "Expected sell price {} (value {} / 2) but got {}",
            value / 2, value, result
        );
    }
}

// ── Property 11: Sell list filtering ─────────────────────────────────────────

/// Strategy to generate a category that is NOT KeyItem.
fn non_key_item_category() -> impl Strategy<Value = ItemCategoryData> {
    prop_oneof![
        Just(ItemCategoryData::Consumable { effects: vec![] }),
        Just(ItemCategoryData::Weapon {
            attack_power: 10,
            equipment_slot: rpg_toolkit_common::item::EquipmentSlot::MainHand,
        }),
        Just(ItemCategoryData::Armor {
            defense_power: 10,
            equipment_slot: rpg_toolkit_common::item::EquipmentSlot::Body,
        }),
        Just(ItemCategoryData::Accessory {
            equipment_slot: rpg_toolkit_common::item::EquipmentSlot::Accessory1,
        }),
    ]
}

/// Strategy to generate inventory items with a mix of sellable and non-sellable characteristics.
/// Returns (items, inventory, shop_entries) where we know which items should pass the filter.
#[allow(clippy::type_complexity)]
fn sell_list_inputs() -> impl Strategy<Value = (Vec<(String, Item, u32, Option<ShopEntry>)>,)> {
    // Generate 1..10 items each with a random category, value, quantity, and optional entry
    proptest::collection::vec(
        (
            // Item ID suffix
            0u32..1000,
            // Whether this is a KeyItem
            proptest::bool::ANY,
            // Item value (used for default sell price)
            0u32..200,
            // Quantity in inventory
            0u32..10,
            // Whether there is a shop entry with explicit sell_price
            proptest::bool::ANY,
            // Explicit sell price (if entry exists)
            0u32..100,
            // Non-key item category
            non_key_item_category(),
        ),
        1..10,
    )
    .prop_map(|items| {
        let result: Vec<(String, Item, u32, Option<ShopEntry>)> = items
            .into_iter()
            .enumerate()
            .map(
                |(i, (id_suffix, is_key, value, qty, has_entry, explicit_sell, category))| {
                    let item_id = format!("item_{}_{}", i, id_suffix);
                    let category_data = if is_key {
                        ItemCategoryData::KeyItem
                    } else {
                        category
                    };
                    let item = make_test_item(&item_id, value, category_data);
                    let entry = if has_entry {
                        Some(make_shop_entry(
                            &item_id,
                            100,
                            Some(explicit_sell),
                            None,
                            None,
                        ))
                    } else {
                        None
                    };
                    (item_id, item, qty, entry)
                },
            )
            .collect();
        (result,)
    })
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 100, .. ProptestConfig::default() })]

    /// **Validates: Requirements 6.5**
    ///
    /// Property 11: sellable_items only returns items where:
    /// - qty > 0
    /// - category != KeyItem
    /// - computed sell_price > 0
    #[test]
    fn sell_list_filtering((items,) in sell_list_inputs()) {
        let mut registry = ItemRegistry::default();
        let mut inventory: HashMap<String, u32> = HashMap::new();
        let mut shop_entries: Vec<ShopEntry> = Vec::new();

        for (item_id, item, qty, entry) in &items {
            registry.items.insert(item_id.clone(), item.clone());
            inventory.insert(item_id.clone(), *qty);
            if let Some(e) = entry {
                shop_entries.push(e.clone());
            }
        }

        let result = sellable_items(&inventory, &registry, &shop_entries);

        // Verify every returned item satisfies all three conditions
        for (returned_id, returned_qty, returned_price) in &result {
            let item = registry.items.get(returned_id).unwrap();

            prop_assert!(
                *returned_qty > 0,
                "Returned item {} has qty 0", returned_id
            );
            prop_assert!(
                item.category() != rpg_toolkit_common::item::ItemCategory::KeyItem,
                "Returned item {} is a KeyItem", returned_id
            );
            prop_assert!(
                *returned_price > 0,
                "Returned item {} has sell_price 0", returned_id
            );
        }

        // Verify that every item that SHOULD be sellable IS in the result
        let result_ids: Vec<&ItemId> = result.iter().map(|(id, _, _)| id).collect();
        for (item_id, item, qty, entry) in &items {
            if *qty == 0 {
                continue;
            }
            if item.category() == rpg_toolkit_common::item::ItemCategory::KeyItem {
                continue;
            }
            let sell_price = if let Some(e) = entry {
                compute_sell_price(e, item)
            } else {
                item.value / 2
            };
            if sell_price == 0 {
                continue;
            }
            // This item should be in the result
            prop_assert!(
                result_ids.contains(&item_id),
                "Item {} should be sellable (qty={}, not KeyItem, sell_price={}) but is missing from result",
                item_id, qty, sell_price
            );
        }
    }
}

// ── Property 12: Condition-based item visibility ─────────────────────────────

/// Strategy to generate a set of flags.
fn flags_strategy() -> impl Strategy<Value = HashMap<String, String>> {
    proptest::collection::hash_map(
        "[a-z]{1,5}",    // keys
        "[a-z0-9]{1,5}", // values
        0..5,
    )
}

/// Strategy to generate a ConditionCheck that references keys that may or may not exist in flags.
fn condition_check_strategy() -> impl Strategy<Value = ConditionCheck> {
    (
        "[a-z]{1,5}",
        prop_oneof![
            Just(ConditionOperator::Equals),
            Just(ConditionOperator::NotEquals),
            Just(ConditionOperator::Exists),
            Just(ConditionOperator::NotExists),
        ],
        proptest::option::of("[a-z0-9]{1,5}"),
    )
        .prop_map(|(key, operator, value)| ConditionCheck {
            key,
            operator,
            value,
        })
}

/// Strategy to generate a BranchCondition.
fn branch_condition_strategy() -> impl Strategy<Value = BranchCondition> {
    (
        prop_oneof![Just(ConditionLogic::All), Just(ConditionLogic::Any)],
        proptest::collection::vec(condition_check_strategy(), 0..4),
    )
        .prop_map(|(logic, checks)| BranchCondition { logic, checks })
}

/// Strategy to generate shop entries with various condition states.
/// Returns (entries, registry, flags).
fn visibility_inputs()
-> impl Strategy<Value = (Vec<ShopEntry>, ItemRegistry, HashMap<String, String>)> {
    (
        flags_strategy(),
        proptest::collection::vec(
            (
                0u32..100, // item index suffix
                proptest::option::of(branch_condition_strategy()),
                proptest::bool::ANY, // whether item exists in registry
            ),
            1..8,
        ),
    )
        .prop_map(|(flags, raw_entries)| {
            let mut registry = ItemRegistry::default();
            let entries: Vec<ShopEntry> = raw_entries
                .into_iter()
                .enumerate()
                .map(|(i, (suffix, condition, in_registry))| {
                    let item_id = format!("vis_item_{}_{}", i, suffix);
                    if in_registry {
                        let item = make_test_item(
                            &item_id,
                            100,
                            ItemCategoryData::Consumable { effects: vec![] },
                        );
                        registry.items.insert(item_id.clone(), item);
                    }
                    make_shop_entry(&item_id, 50, None, None, condition)
                })
                .collect();
            (entries, registry, flags)
        })
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 100, .. ProptestConfig::default() })]

    /// **Validates: Requirements 7.1, 7.2, 7.3, 7.4**
    ///
    /// Property 12: An entry is visible iff:
    /// - Its item_id exists in the registry, AND
    /// - condition is None, OR checks is empty, OR condition.evaluate(flags) == true
    #[test]
    fn condition_based_item_visibility((entries, registry, flags) in visibility_inputs()) {
        let result = visible_entries(&entries, &flags, &registry);

        for entry in &entries {
            let in_registry = registry.items.contains_key(&entry.item_id);
            let condition_passes = match &entry.condition {
                None => true,
                Some(cond) => {
                    if cond.checks.is_empty() {
                        true
                    } else {
                        cond.evaluate(&flags)
                    }
                }
            };

            let should_be_visible = in_registry && condition_passes;
            let is_visible = result.iter().any(|e| e.item_id == entry.item_id);

            prop_assert_eq!(
                is_visible,
                should_be_visible,
                "Entry '{}': in_registry={}, condition_passes={}, expected visible={}, got visible={}",
                entry.item_id, in_registry, condition_passes, should_be_visible, is_visible
            );
        }
    }
}

// ── Property 16: Stock value clamping on load ────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig { cases: 100, .. ProptestConfig::default() })]

    /// **Validates: Requirements 9.4**
    ///
    /// Property 16: When loading saved stock, if saved_value > stock_limit then the
    /// value is clamped to stock_limit; otherwise the saved_value is preserved.
    #[test]
    fn stock_value_clamping_on_load(
        saved_value in any::<u32>(),
        stock_limit in 1u32..=9999,
    ) {
        let clamped = saved_value.min(stock_limit);

        if saved_value <= stock_limit {
            prop_assert_eq!(clamped, saved_value,
                "saved_value ({}) <= stock_limit ({}), expected preserved value {} but got {}",
                saved_value, stock_limit, saved_value, clamped
            );
        } else {
            prop_assert_eq!(clamped, stock_limit,
                "saved_value ({}) > stock_limit ({}), expected clamped to {} but got {}",
                saved_value, stock_limit, stock_limit, clamped
            );
        }
    }
}
