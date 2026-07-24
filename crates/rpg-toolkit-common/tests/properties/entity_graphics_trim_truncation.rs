// Feature: graphics-database-editor, Property 7: EntityGraphics icon trim and truncation

use proptest::prelude::*;

use rpg_toolkit_common::graphics::EntityGraphics;

/// Strategy for generating arbitrary strings including:
/// - Normal paths
/// - Strings with leading/trailing whitespace
/// - Long strings (>260 chars)
/// - Empty strings
/// - Whitespace-only strings
fn arb_icon_input() -> impl Strategy<Value = String> {
    prop_oneof![
        // Normal path-like strings
        "[a-zA-Z0-9_/\\-\\.]{1,50}",
        // Strings with leading/trailing whitespace
        "\\s{1,5}[a-zA-Z0-9_/\\-\\.]{1,50}\\s{1,5}",
        // Long strings (>260 chars)
        "[a-zA-Z0-9_/]{261,400}",
        // Long strings with whitespace padding
        "\\s{1,10}[a-zA-Z0-9_/]{261,400}\\s{1,10}",
        // Completely arbitrary strings
        ".*",
    ]
}

/// Strategy for generating strings that are empty or whitespace-only after trimming.
fn arb_empty_after_trim() -> impl Strategy<Value = String> {
    prop_oneof![
        Just(String::new()),
        "\\s+",               // whitespace-only (spaces, tabs, newlines)
        "[ \\t\\n\\r]{1,20}", // explicit whitespace characters
    ]
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 100, .. ProptestConfig::default() })]

    /// **Validates: Requirements 3.7, 4.7**
    ///
    /// Property 7: EntityGraphics icon trim and truncation — For any non-empty-after-trim
    /// string input to `set_icon`, the stored icon value SHALL equal the input trimmed of
    /// leading/trailing whitespace and truncated to 260 characters.
    #[test]
    fn set_icon_stores_trimmed_and_truncated(input in arb_icon_input()) {
        let trimmed = input.trim();
        // Skip empty-after-trim inputs (tested separately below)
        prop_assume!(!trimmed.is_empty());

        let mut gfx = EntityGraphics::default();
        let result = gfx.set_icon(&input);

        let expected: String = trimmed.chars().take(260).collect();

        prop_assert!(
            result.is_ok(),
            "set_icon should succeed for non-empty-after-trim input. Input: {:?}, Trimmed: {:?}",
            input, trimmed
        );
        prop_assert_eq!(
            gfx.icon.as_ref().unwrap(),
            &expected,
            "Stored icon should equal trimmed + truncated to 260 chars. Input: {:?}",
            input
        );
    }

    /// **Validates: Requirements 3.7, 4.7**
    ///
    /// Property 7: EntityGraphics icon trim and truncation — If the trimmed input is empty,
    /// the operation SHALL return an error and the icon SHALL remain unchanged.
    #[test]
    fn set_icon_rejects_empty_after_trim(input in arb_empty_after_trim()) {
        // Set up EntityGraphics with a known initial icon state
        let mut gfx = EntityGraphics::default();
        let initial_icon = Some("existing/icon.png".to_string());
        gfx.icon = initial_icon.clone();

        let result = gfx.set_icon(&input);

        prop_assert!(
            result.is_err(),
            "set_icon should return an error for empty-after-trim input. Input: {:?}",
            input
        );
        prop_assert_eq!(
            gfx.icon, initial_icon,
            "Icon should remain unchanged after failed set_icon. Input: {:?}",
            input
        );
    }

    /// **Validates: Requirements 3.7, 4.7**
    ///
    /// Additional: When EntityGraphics starts with no icon (None), rejecting an
    /// empty-after-trim input leaves the icon as None.
    #[test]
    fn set_icon_rejects_empty_after_trim_from_none(input in arb_empty_after_trim()) {
        let mut gfx = EntityGraphics::default();
        prop_assert!(gfx.icon.is_none());

        let result = gfx.set_icon(&input);

        prop_assert!(
            result.is_err(),
            "set_icon should return an error for empty-after-trim input. Input: {:?}",
            input
        );
        prop_assert!(
            gfx.icon.is_none(),
            "Icon should remain None after failed set_icon. Input: {:?}",
            input
        );
    }
}
