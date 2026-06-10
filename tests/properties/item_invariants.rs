// Feature: items-editor, Property 3: Category-specific invariants
//
// For any set of items created across all categories, and after any sequence of
// random operations including `change_category`, `set_stackable`, `set_stack_limit`,
// `add_stat_modifier`, `remove_stat_modifier`, the following invariants SHALL hold:
//   - If an item's category is Consumable, then stackable == true
//   - If an item's category is KeyItem, then stackable == false AND value == 0
//
// Validates: Requirements 2.7, 2.8, 2.9

use proptest::prelude::*;
use rpg_toolkit_common::{ItemCategory, ItemRegistry};

/// All possible item categories for generation.
const ALL_CATEGORIES: [ItemCategory; 5] = [
    ItemCategory::Weapon,
    ItemCategory::Armor,
    ItemCategory::Accessory,
    ItemCategory::Consumable,
    ItemCategory::KeyItem,
];

/// Represents a random operation to apply to an item in the registry.
#[derive(Clone, Debug)]
enum ItemOp {
    ChangeCategory(usize, ItemCategory),
    SetStackable(usize, bool),
    SetStackLimit(usize, u32),
    AddStat(usize, String, i32),
    RemoveStat(usize, String),
}

/// Strategy to generate a random item category.
fn arb_category() -> impl Strategy<Value = ItemCategory> {
    prop::sample::select(&ALL_CATEGORIES)
}

/// Strategy to generate a random operation targeting one of the items.
fn arb_item_op(num_items: usize) -> impl Strategy<Value = ItemOp> {
    let item_index = 0..num_items;

    prop_oneof![
        // ChangeCategory to any of the 5 categories
        (item_index.clone(), arb_category())
            .prop_map(|(idx, cat)| ItemOp::ChangeCategory(idx, cat)),
        // SetStackable to true or false
        (item_index.clone(), any::<bool>())
            .prop_map(|(idx, stackable)| ItemOp::SetStackable(idx, stackable)),
        // SetStackLimit with values that may or may not be valid
        (item_index.clone(), 1u32..1000).prop_map(|(idx, limit)| ItemOp::SetStackLimit(idx, limit)),
        // AddStat with a random stat name
        (item_index.clone(), "[a-zA-Z]{1,10}", -100i32..100)
            .prop_map(|(idx, name, value)| ItemOp::AddStat(idx, name, value)),
        // RemoveStat with a random stat name
        (item_index.clone(), "[a-zA-Z]{1,10}")
            .prop_map(|(idx, name)| ItemOp::RemoveStat(idx, name)),
    ]
}

/// Strategy to generate a vector of random operations.
fn arb_item_ops(num_items: usize) -> impl Strategy<Value = Vec<ItemOp>> {
    prop::collection::vec(arb_item_op(num_items), 1..=20)
}

/// Strategy to generate a vector of categories for item creation (1-5 items).
fn arb_initial_categories() -> impl Strategy<Value = Vec<ItemCategory>> {
    prop::collection::vec(arb_category(), 1..=5)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn category_specific_invariants(
        _categories in arb_initial_categories(),
        ops in arb_initial_categories().prop_flat_map(|cats| {
            let n = cats.len();
            (Just(cats), arb_item_ops(n))
        })
    ) {
        let (categories, ops) = ops;
        let mut registry = ItemRegistry::default();

        // Create items with random categories
        let mut item_ids = Vec::new();
        for (i, cat) in categories.iter().enumerate() {
            let name = format!("Item{}", i);
            let id = registry.create_item(&name, *cat).expect("create_item should succeed");
            item_ids.push(id);
        }

        // Apply random sequence of operations, ignoring errors
        for op in &ops {
            match op {
                ItemOp::ChangeCategory(idx, cat) => {
                    let _ = registry.change_category(&item_ids[*idx], *cat);
                }
                ItemOp::SetStackable(idx, stackable) => {
                    let _ = registry.set_stackable(&item_ids[*idx], *stackable);
                }
                ItemOp::SetStackLimit(idx, limit) => {
                    let _ = registry.set_stack_limit(&item_ids[*idx], *limit);
                }
                ItemOp::AddStat(idx, name, value) => {
                    let _ = registry.add_stat_modifier(&item_ids[*idx], name, *value);
                }
                ItemOp::RemoveStat(idx, name) => {
                    let _ = registry.remove_stat_modifier(&item_ids[*idx], name);
                }
            }
        }

        // Assert category-specific invariants for ALL items
        for id in &item_ids {
            let item = registry.items.get(id).expect("Item should still exist");

            match item.category() {
                ItemCategory::Consumable => {
                    prop_assert!(
                        item.stackable,
                        "Consumable item '{}' must have stackable=true, but got false. \
                         Operations: {:?}",
                        item.display_name, ops
                    );
                }
                ItemCategory::KeyItem => {
                    prop_assert!(
                        !item.stackable,
                        "KeyItem '{}' must have stackable=false, but got true. \
                         Operations: {:?}",
                        item.display_name, ops
                    );
                    prop_assert_eq!(
                        item.value, 0,
                        "KeyItem '{}' must have value=0, but got {}. \
                         Operations: {:?}",
                        item.display_name, item.value, ops
                    );
                }
                _ => {
                    // No specific invariant for Weapon, Armor, Accessory
                }
            }
        }
    }
}
