// Feature: dialog-selection, Property 1: Serialization Round-Trip
//
// For any valid ShowSelection action (with 2–6 choices, valid labels, and arbitrarily
// nested EventAction lists up to 3 levels deep including Branch, StateCheck, and
// recursive ShowSelection), serializing to JSON and deserializing back SHALL produce
// a value that is structurally equal to the original via PartialEq.
//
// Validates: Requirements 1.6, 8.1, 8.2

use proptest::prelude::*;
use rpg_toolkit_common::condition::{
    BranchCondition, ConditionCheck, ConditionLogic, ConditionOperator,
};
use rpg_toolkit_common::map::{
    ChoiceData, DialogConfigData, DialogPositionData, DialogTextData, EventAction,
};

// ---------------------------------------------------------------------------
// Arbitrary generators
// ---------------------------------------------------------------------------

fn arb_dialog_text_data() -> impl Strategy<Value = DialogTextData> {
    prop_oneof![
        "[a-zA-Z ]{1,40}".prop_map(DialogTextData::Inline),
        "[a-z\\-]{3,15}".prop_map(DialogTextData::Id),
    ]
}

fn arb_dialog_position() -> impl Strategy<Value = DialogPositionData> {
    prop_oneof![
        Just(DialogPositionData::Top),
        Just(DialogPositionData::Center),
        Just(DialogPositionData::Bottom),
    ]
}

fn arb_dialog_config() -> impl Strategy<Value = DialogConfigData> {
    (
        10.0f32..100.0,
        arb_dialog_position(),
        any::<bool>(),
        any::<bool>(),
        any::<bool>(),
    )
        .prop_map(
            |(text_speed, position, movement_block, attribute_dialog, has_portrait)| {
                DialogConfigData {
                    text_speed,
                    position,
                    movement_block,
                    attribute_dialog,
                    face_portrait: if has_portrait {
                        Some("assets/portrait.png".to_string())
                    } else {
                        None
                    },
                }
            },
        )
}

/// Label strategy for choices: inline labels must be 1–80 characters.
fn arb_choice_label() -> impl Strategy<Value = DialogTextData> {
    prop_oneof![
        "[a-zA-Z ]{1,80}".prop_map(DialogTextData::Inline),
        "[a-z\\-]{3,15}".prop_map(DialogTextData::Id),
    ]
}

fn arb_condition_operator() -> impl Strategy<Value = ConditionOperator> {
    prop_oneof![
        Just(ConditionOperator::Equals),
        Just(ConditionOperator::NotEquals),
        Just(ConditionOperator::Exists),
        Just(ConditionOperator::NotExists),
    ]
}

fn arb_condition_check() -> impl Strategy<Value = ConditionCheck> {
    (
        "[a-z]{1,8}",
        arb_condition_operator(),
        proptest::option::of("[a-z0-9]{1,8}"),
    )
        .prop_map(|(key, operator, value)| ConditionCheck {
            key,
            operator,
            value,
        })
}

fn arb_branch_condition() -> impl Strategy<Value = BranchCondition> {
    (
        prop_oneof![Just(ConditionLogic::All), Just(ConditionLogic::Any)],
        proptest::collection::vec(arb_condition_check(), 1..=3),
    )
        .prop_map(|(logic, checks)| BranchCondition { logic, checks })
}

/// Generate leaf EventAction variants (no nesting).
fn arb_leaf_event_action() -> impl Strategy<Value = EventAction> {
    prop_oneof![
        // ShowDialog
        (arb_dialog_text_data(), arb_dialog_config())
            .prop_map(|(text, config)| EventAction::ShowDialog { text, config }),
        // SetState
        ("[a-z]{1,10}", "[a-z0-9]{1,10}")
            .prop_map(|(key, value)| EventAction::SetState { key, value }),
        // JumpTo (MapTransition)
        ("[a-z\\-]{3,10}", 0u32..16, 0u32..16).prop_map(|(target_map_id, target_x, target_y)| {
            EventAction::JumpTo {
                target_map_id,
                target_x,
                target_y,
                target_elevation: None,
            }
        }),
    ]
}

/// Generate EventAction trees with bounded depth.
/// At depth 0, only leaf actions are produced.
/// At depth > 0, branching actions (Branch, StateCheck, ShowSelection) can appear.
fn arb_event_action(depth: u32) -> BoxedStrategy<EventAction> {
    if depth == 0 {
        arb_leaf_event_action().boxed()
    } else {
        let leaf = arb_leaf_event_action();
        let branch = (
            arb_branch_condition(),
            proptest::collection::vec(arb_event_action(depth - 1), 0..=3),
            proptest::collection::vec(arb_event_action(depth - 1), 0..=3),
        )
            .prop_map(|(condition, on_true, on_false)| EventAction::Branch {
                condition,
                on_true,
                on_false,
            });
        let state_check = (
            "[a-z]{1,8}",
            proptest::option::of("[a-z0-9]{1,8}"),
            proptest::collection::vec(arb_event_action(depth - 1), 0..=3),
            proptest::collection::vec(arb_event_action(depth - 1), 0..=3),
        )
            .prop_map(|(key, value, on_true, on_false)| EventAction::StateCheck {
                key,
                value,
                on_true,
                on_false,
            });
        let show_selection = arb_show_selection_action(depth - 1);

        prop_oneof![
            3 => leaf,
            2 => branch,
            2 => state_check,
            1 => show_selection,
        ]
        .boxed()
    }
}

/// Generate a valid ChoiceData with actions nested to the given depth.
fn arb_choice_data(depth: u32) -> impl Strategy<Value = ChoiceData> {
    (
        arb_choice_label(),
        proptest::collection::vec(arb_event_action(depth), 0..=5),
    )
        .prop_map(|(label, actions)| ChoiceData { label, actions })
}

/// Generate a valid ShowSelection EventAction with choices nested to the given depth.
fn arb_show_selection_action(depth: u32) -> impl Strategy<Value = EventAction> {
    (
        arb_dialog_text_data(),
        arb_dialog_config(),
        proptest::collection::vec(arb_choice_data(depth), 2..=6),
    )
        .prop_map(|(prompt, config, choices)| EventAction::ShowSelection {
            prompt,
            config,
            choices,
        })
}

// ---------------------------------------------------------------------------
// Property 1: Serialization Round-Trip
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// For any valid ShowSelection action with nested EventAction trees up to
    /// 3 levels deep, serializing to JSON and deserializing back produces a
    /// PartialEq-equal value.
    ///
    /// **Validates: Requirements 1.6, 8.1, 8.2**
    #[test]
    fn show_selection_serialization_round_trip(
        action in arb_show_selection_action(3)
    ) {
        // Serialize to JSON
        let json = serde_json::to_string(&action)
            .expect("serialization should succeed for valid ShowSelection");

        // Deserialize back
        let deserialized: EventAction = serde_json::from_str(&json)
            .expect("deserialization should succeed for valid ShowSelection JSON");

        // Assert structural equality
        prop_assert_eq!(
            &action,
            &deserialized,
            "Round-trip failed: original != deserialized.\nJSON: {}",
            json
        );
    }
}
