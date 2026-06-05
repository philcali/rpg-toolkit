// Feature: event-branching, Property 1: BranchCondition Evaluation Semantics
//
// For any BranchCondition and any GameState flags map, evaluation SHALL return:
// - When logic is All: true iff every ConditionCheck in checks evaluates to true (or checks is empty).
// - When logic is Any: true iff at least one ConditionCheck in checks evaluates to true (or checks is empty).
//
// Validates: Requirements 1.6, 1.7, 1.8

// Feature: event-branching, Property 2: ConditionCheck Operator Semantics
//
// For any ConditionCheck and any GameState flags map, evaluation SHALL return:
// - Equals: true iff value is Some(v) and flags[key] == v
// - NotEquals: true iff value is None, or key is absent from flags, or flags[key] != v
// - Exists: true iff key is present in flags (regardless of value field)
// - NotExists: true iff key is absent from flags (regardless of value field)
//
// Validates: Requirements 1.5, 2.1, 2.2, 2.3, 2.4, 2.5

use std::collections::HashMap;

use proptest::prelude::*;
use rpg_toolkit_common::condition::{
    BranchCondition, ConditionCheck, ConditionLogic, ConditionOperator,
};

// ---------------------------------------------------------------------------
// Arbitrary generators
// ---------------------------------------------------------------------------

fn arb_operator() -> impl Strategy<Value = ConditionOperator> {
    prop_oneof![
        Just(ConditionOperator::Equals),
        Just(ConditionOperator::NotEquals),
        Just(ConditionOperator::Exists),
        Just(ConditionOperator::NotExists),
    ]
}

fn arb_condition_check() -> impl Strategy<Value = ConditionCheck> {
    (
        "[a-z]{1,5}",
        arb_operator(),
        proptest::option::of("[a-z0-9]{1,5}"),
    )
        .prop_map(|(key, operator, value)| ConditionCheck {
            key,
            operator,
            value,
        })
}

fn arb_logic() -> impl Strategy<Value = ConditionLogic> {
    prop_oneof![Just(ConditionLogic::All), Just(ConditionLogic::Any),]
}

fn arb_branch_condition() -> impl Strategy<Value = BranchCondition> {
    (
        arb_logic(),
        proptest::collection::vec(arb_condition_check(), 0..6),
    )
        .prop_map(|(logic, checks)| BranchCondition { logic, checks })
}

fn arb_flags() -> impl Strategy<Value = HashMap<String, String>> {
    proptest::collection::hash_map("[a-z]{1,5}", "[a-z0-9]{1,5}", 0..8)
}

// ---------------------------------------------------------------------------
// Helper: independently compute expected result of a single ConditionCheck
// ---------------------------------------------------------------------------

fn expected_check_result(check: &ConditionCheck, flags: &HashMap<String, String>) -> bool {
    match check.operator {
        ConditionOperator::Equals => match &check.value {
            Some(v) => flags.get(&check.key) == Some(v),
            None => false,
        },
        ConditionOperator::NotEquals => match &check.value {
            Some(v) => flags.get(&check.key) != Some(v),
            None => true,
        },
        ConditionOperator::Exists => flags.contains_key(&check.key),
        ConditionOperator::NotExists => !flags.contains_key(&check.key),
    }
}

// ---------------------------------------------------------------------------
// Property 1: BranchCondition Evaluation Semantics
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// BranchCondition with `All` logic returns true iff every check passes.
    /// Empty checks list returns true (vacuous truth).
    /// Validates Requirements 1.6, 1.8
    #[test]
    fn branch_condition_all_logic_semantics(
        condition in arb_branch_condition()
            .prop_filter("All logic only", |c| c.logic == ConditionLogic::All),
        flags in arb_flags(),
    ) {
        let result = condition.evaluate(&flags);

        let expected = if condition.checks.is_empty() {
            true
        } else {
            condition.checks.iter().all(|check| expected_check_result(check, &flags))
        };

        prop_assert_eq!(
            result,
            expected,
            "All logic: evaluate() returned {} but expected {} for condition {:?} with flags {:?}",
            result,
            expected,
            condition,
            flags
        );
    }

    /// BranchCondition with `Any` logic returns true iff at least one check passes.
    /// Empty checks list returns true (vacuous truth).
    /// Validates Requirements 1.7, 1.8
    #[test]
    fn branch_condition_any_logic_semantics(
        condition in arb_branch_condition()
            .prop_filter("Any logic only", |c| c.logic == ConditionLogic::Any),
        flags in arb_flags(),
    ) {
        let result = condition.evaluate(&flags);

        let expected = if condition.checks.is_empty() {
            true
        } else {
            condition.checks.iter().any(|check| expected_check_result(check, &flags))
        };

        prop_assert_eq!(
            result,
            expected,
            "Any logic: evaluate() returned {} but expected {} for condition {:?} with flags {:?}",
            result,
            expected,
            condition,
            flags
        );
    }

    /// BranchCondition with empty checks always evaluates to true regardless of logic.
    /// Validates Requirement 1.8
    #[test]
    fn branch_condition_empty_checks_always_true(
        logic in arb_logic(),
        flags in arb_flags(),
    ) {
        let condition = BranchCondition {
            logic,
            checks: vec![],
        };

        let result = condition.evaluate(&flags);

        prop_assert!(
            result,
            "Empty checks should always evaluate to true, got false for logic {:?} with flags {:?}",
            logic,
            flags
        );
    }

    /// BranchCondition evaluation is consistent: calling evaluate twice with the
    /// same inputs produces the same result (determinism).
    /// Validates Requirements 1.6, 1.7
    #[test]
    fn branch_condition_evaluation_is_deterministic(
        condition in arb_branch_condition(),
        flags in arb_flags(),
    ) {
        let result1 = condition.evaluate(&flags);
        let result2 = condition.evaluate(&flags);

        prop_assert_eq!(
            result1,
            result2,
            "evaluate() is non-deterministic: first call returned {}, second returned {}",
            result1,
            result2
        );
    }
}

// ---------------------------------------------------------------------------
// Property 2: ConditionCheck Operator Semantics
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Equals operator: true iff value is Some(v) and flags[key] == v.
    /// When key is absent from flags, Equals returns false.
    /// When value is None, Equals returns false.
    /// Validates Requirements 2.1, 2.5
    #[test]
    fn condition_check_equals_operator(
        key in "[a-z]{1,5}",
        value in proptest::option::of("[a-z0-9]{1,5}"),
        flags in arb_flags(),
    ) {
        let check = ConditionCheck {
            key: key.clone(),
            operator: ConditionOperator::Equals,
            value: value.clone(),
        };

        let result = check.evaluate(&flags);

        let expected = match &value {
            Some(v) => flags.get(&key) == Some(v),
            None => false,
        };

        prop_assert_eq!(
            result,
            expected,
            "Equals: evaluate() returned {} but expected {} for key={:?}, value={:?}, flags={:?}",
            result,
            expected,
            key,
            value,
            flags
        );
    }

    /// NotEquals operator: true iff value is None, or key absent, or flags[key] != v.
    /// When value is None, NotEquals always returns true.
    /// Validates Requirements 2.2, 2.5
    #[test]
    fn condition_check_not_equals_operator(
        key in "[a-z]{1,5}",
        value in proptest::option::of("[a-z0-9]{1,5}"),
        flags in arb_flags(),
    ) {
        let check = ConditionCheck {
            key: key.clone(),
            operator: ConditionOperator::NotEquals,
            value: value.clone(),
        };

        let result = check.evaluate(&flags);

        let expected = match &value {
            Some(v) => flags.get(&key) != Some(v),
            None => true,
        };

        prop_assert_eq!(
            result,
            expected,
            "NotEquals: evaluate() returned {} but expected {} for key={:?}, value={:?}, flags={:?}",
            result,
            expected,
            key,
            value,
            flags
        );
    }

    /// Exists operator: true iff key is present in flags (value field ignored).
    /// Validates Requirements 2.3, 1.5
    #[test]
    fn condition_check_exists_operator(
        key in "[a-z]{1,5}",
        value in proptest::option::of("[a-z0-9]{1,5}"),
        flags in arb_flags(),
    ) {
        let check = ConditionCheck {
            key: key.clone(),
            operator: ConditionOperator::Exists,
            value: value.clone(),
        };

        let result = check.evaluate(&flags);
        let expected = flags.contains_key(&key);

        prop_assert_eq!(
            result,
            expected,
            "Exists: evaluate() returned {} but expected {} for key={:?}, value={:?}, flags={:?}",
            result,
            expected,
            key,
            value,
            flags
        );
    }

    /// NotExists operator: true iff key is absent from flags (value field ignored).
    /// Validates Requirements 2.4, 1.5
    #[test]
    fn condition_check_not_exists_operator(
        key in "[a-z]{1,5}",
        value in proptest::option::of("[a-z0-9]{1,5}"),
        flags in arb_flags(),
    ) {
        let check = ConditionCheck {
            key: key.clone(),
            operator: ConditionOperator::NotExists,
            value: value.clone(),
        };

        let result = check.evaluate(&flags);
        let expected = !flags.contains_key(&key);

        prop_assert_eq!(
            result,
            expected,
            "NotExists: evaluate() returned {} but expected {} for key={:?}, value={:?}, flags={:?}",
            result,
            expected,
            key,
            value,
            flags
        );
    }

    /// Equals with value=None always returns false regardless of flags.
    /// Validates Requirement 2.5
    #[test]
    fn condition_check_equals_none_value_always_false(
        key in "[a-z]{1,5}",
        flags in arb_flags(),
    ) {
        let check = ConditionCheck {
            key: key.clone(),
            operator: ConditionOperator::Equals,
            value: None,
        };

        let result = check.evaluate(&flags);

        prop_assert!(
            !result,
            "Equals with value=None should always be false, got true for key={:?}, flags={:?}",
            key,
            flags
        );
    }

    /// NotEquals with value=None always returns true regardless of flags.
    /// Validates Requirement 2.5
    #[test]
    fn condition_check_not_equals_none_value_always_true(
        key in "[a-z]{1,5}",
        flags in arb_flags(),
    ) {
        let check = ConditionCheck {
            key: key.clone(),
            operator: ConditionOperator::NotEquals,
            value: None,
        };

        let result = check.evaluate(&flags);

        prop_assert!(
            result,
            "NotEquals with value=None should always be true, got false for key={:?}, flags={:?}",
            key,
            flags
        );
    }

    /// ConditionCheck evaluation matches the reference implementation for any
    /// arbitrary check and flags combination (all operators covered).
    /// Validates Requirements 1.5, 2.1, 2.2, 2.3, 2.4, 2.5
    #[test]
    fn condition_check_matches_reference_implementation(
        check in arb_condition_check(),
        flags in arb_flags(),
    ) {
        let result = check.evaluate(&flags);
        let expected = expected_check_result(&check, &flags);

        prop_assert_eq!(
            result,
            expected,
            "ConditionCheck evaluate() returned {} but reference returned {} for check {:?} with flags {:?}",
            result,
            expected,
            check,
            flags
        );
    }
}
