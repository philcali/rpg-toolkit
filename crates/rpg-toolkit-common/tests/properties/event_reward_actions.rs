// Feature: event-rewards, Property 1: Reward action serialization round-trip

use proptest::prelude::*;

use rpg_toolkit_common::map::{EventAction, TransferDirection};

/// Strategy for generating a valid TransferDirection.
fn arb_direction() -> BoxedStrategy<TransferDirection> {
    prop_oneof![Just(TransferDirection::Give), Just(TransferDirection::Take),].boxed()
}

/// Strategy for generating a valid reward amount in [1, 9_999_999].
fn arb_reward_amount() -> BoxedStrategy<u64> {
    (1u64..=9_999_999u64).boxed()
}

/// Strategy for generating a valid item quantity in [1, 999].
fn arb_item_quantity() -> BoxedStrategy<u32> {
    (1u32..=999u32).boxed()
}

/// Strategy for generating a non-empty string (for item_id, ability_id, target).
fn arb_non_empty_string() -> BoxedStrategy<String> {
    "[a-zA-Z][a-zA-Z0-9_-]{0,31}".prop_map(|s| s).boxed()
}

/// Strategy for generating a character_id (1-64 chars).
fn arb_character_id() -> BoxedStrategy<String> {
    "[a-zA-Z][a-zA-Z0-9_-]{0,63}".prop_map(|s| s).boxed()
}

/// Strategy for generating an optional non-empty target string.
fn arb_optional_target() -> BoxedStrategy<Option<String>> {
    prop_oneof![Just(None), arb_non_empty_string().prop_map(Some),].boxed()
}

/// Strategy for generating a simple non-reward EventAction (used as leaf nested actions).
fn arb_leaf_action() -> BoxedStrategy<EventAction> {
    (arb_non_empty_string(), arb_non_empty_string())
        .prop_map(|(key, value)| EventAction::SetState { key, value })
        .boxed()
}

/// Strategy for generating a Vec of nested actions at the given depth.
/// At depth 0, only leaf (non-reward) actions are produced.
/// At depth > 0, reward actions with depth-1 nesting are generated.
fn arb_nested_actions(depth: u32) -> BoxedStrategy<Vec<EventAction>> {
    if depth == 0 {
        proptest::collection::vec(arb_leaf_action(), 0..=2).boxed()
    } else {
        proptest::collection::vec(arb_reward_action(depth - 1), 0..=2).boxed()
    }
}

/// Strategy for generating a valid reward EventAction at the given nesting depth.
/// Depth 2 means reward actions can contain nested reward actions (depth 1),
/// which themselves can contain leaf actions (depth 0).
fn arb_reward_action(depth: u32) -> BoxedStrategy<EventAction> {
    prop_oneof![
        // GiveCurrency
        (
            arb_reward_amount(),
            arb_direction(),
            arb_nested_actions(depth),
            arb_nested_actions(depth),
        )
            .prop_map(|(amount, direction, on_success, on_failure)| {
                EventAction::GiveCurrency {
                    amount,
                    direction,
                    on_success,
                    on_failure,
                }
            }),
        // GiveExperience
        (
            arb_reward_amount(),
            arb_optional_target(),
            arb_direction(),
            arb_nested_actions(depth),
            arb_nested_actions(depth),
        )
            .prop_map(|(amount, target, direction, on_success, on_failure)| {
                EventAction::GiveExperience {
                    amount,
                    target,
                    direction,
                    on_success,
                    on_failure,
                }
            }),
        // GiveItem
        (
            arb_non_empty_string(),
            arb_item_quantity(),
            arb_direction(),
            arb_nested_actions(depth),
            arb_nested_actions(depth),
        )
            .prop_map(|(item_id, quantity, direction, on_success, on_failure)| {
                EventAction::GiveItem {
                    item_id,
                    quantity,
                    direction,
                    on_success,
                    on_failure,
                }
            }),
        // LearnAbility
        (
            arb_non_empty_string(),
            arb_non_empty_string(),
            arb_direction(),
            arb_nested_actions(depth),
            arb_nested_actions(depth),
        )
            .prop_map(|(ability_id, target, direction, on_success, on_failure)| {
                EventAction::LearnAbility {
                    ability_id,
                    target,
                    direction,
                    on_success,
                    on_failure,
                }
            }),
        // AddPartyMember
        (
            arb_character_id(),
            arb_direction(),
            arb_nested_actions(depth),
            arb_nested_actions(depth),
        )
            .prop_map(|(character_id, direction, on_success, on_failure)| {
                EventAction::AddPartyMember {
                    character_id,
                    direction,
                    on_success,
                    on_failure,
                }
            }),
    ]
    .boxed()
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 100, .. ProptestConfig::default() })]

    /// **Validates: Requirements 1.5, 3.8, 5.8, 7.7, 9.5, 13.3, 13.6**
    ///
    /// Property 1: Reward action serialization round-trip.
    /// For any valid EventAction value that is a reward variant (GiveCurrency, GiveExperience,
    /// GiveItem, LearnAbility, or AddPartyMember) with any valid combination of direction,
    /// on_success (containing arbitrary nested EventActions), and on_failure (containing
    /// arbitrary nested EventActions), serializing to JSON and deserializing back shall produce
    /// a value equal to the original.
    #[test]
    fn reward_action_serialization_round_trip(
        action in arb_reward_action(2),
    ) {
        // Serialize to JSON
        let json = serde_json::to_string(&action).unwrap();

        // Deserialize back
        let deserialized: EventAction = serde_json::from_str(&json).unwrap();

        // Assert equality
        prop_assert_eq!(&action, &deserialized);
    }
}
