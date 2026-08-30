// Feature: editor-enhancements, Property 12: HotkeyBinding Duplicate key_code Rejection

use proptest::prelude::*;

use rpg_toolkit_common::hotkey::deserialize_hotkey_bindings;

proptest! {
    #![proptest_config(ProptestConfig { cases: 100, .. ProptestConfig::default() })]

    /// **Validates: Requirements 6.9**
    ///
    /// Property 12: HotkeyBinding Duplicate key_code Rejection.
    /// For any list of two or more HotkeyBinding values where at least two
    /// share the same key_code, deserializing the hotkey_bindings field
    /// SHALL return a deserialization error.
    #[test]
    fn duplicate_key_code_is_rejected(
        key_code in "[a-zA-Z]{1,10}",
        name1 in "[a-zA-Z]{1,10}",
        name2 in "[a-zA-Z]{1,10}",
    ) {
        let json = format!(
            r#"[{{"key_code": "{}", "name": "{}", "event_actions": []}}, {{"key_code": "{}", "name": "{}", "event_actions": []}}]"#,
            key_code, name1, key_code, name2
        );
        let mut deserializer = serde_json::Deserializer::from_str(&json);
        let result = deserialize_hotkey_bindings(&mut deserializer);
        prop_assert!(result.is_err(), "Expected error for duplicate key_code '{}', got Ok", key_code);
        let err = result.unwrap_err().to_string();
        prop_assert!(err.contains("duplicate key_code"), "Error should mention 'duplicate key_code', got: {}", err);
    }
}
