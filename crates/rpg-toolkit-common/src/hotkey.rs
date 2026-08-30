use std::collections::HashSet;

use serde::{Deserialize, Deserializer, Serialize};

use crate::map::EventAction;

/// A hotkey binding that maps a keyboard key to a named event action sequence.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(try_from = "RawHotkeyBinding")]
pub struct HotkeyBinding {
    /// Bevy `KeyCode` variant name (e.g., "ShiftLeft", "KeyZ", "Space"). 1–64 chars.
    pub key_code: String,
    /// Human-readable label for the binding. 1–64 chars.
    pub name: String,
    /// Action sequence fired when the hotkey is pressed. 0–20 entries.
    pub event_actions: Vec<EventAction>,
}

/// Raw helper struct for deserializing `HotkeyBinding` with validation.
#[derive(Deserialize)]
pub struct RawHotkeyBinding {
    pub key_code: String,
    pub name: String,
    #[serde(default)]
    pub event_actions: Vec<EventAction>,
}

impl TryFrom<RawHotkeyBinding> for HotkeyBinding {
    type Error = String;

    fn try_from(raw: RawHotkeyBinding) -> Result<Self, Self::Error> {
        if raw.key_code.is_empty() || raw.key_code.len() > 64 {
            return Err("key_code must be 1 to 64 characters".to_string());
        }
        if raw.name.is_empty() || raw.name.len() > 64 {
            return Err("name must be 1 to 64 characters".to_string());
        }
        if raw.event_actions.len() > 20 {
            return Err("event_actions must have at most 20 entries".to_string());
        }
        Ok(HotkeyBinding {
            key_code: raw.key_code,
            name: raw.name,
            event_actions: raw.event_actions,
        })
    }
}

impl<'de> Deserialize<'de> for HotkeyBinding {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = RawHotkeyBinding::deserialize(deserializer)?;
        HotkeyBinding::try_from(raw).map_err(serde::de::Error::custom)
    }
}

/// Custom deserializer for `Vec<HotkeyBinding>` that enforces:
/// - At most 32 entries
/// - Unique `key_code` values across all bindings
pub fn deserialize_hotkey_bindings<'de, D>(deserializer: D) -> Result<Vec<HotkeyBinding>, D::Error>
where
    D: Deserializer<'de>,
{
    let bindings = Vec::<HotkeyBinding>::deserialize(deserializer)?;

    if bindings.len() > 32 {
        return Err(serde::de::Error::custom(
            "hotkey_bindings must have at most 32 entries",
        ));
    }

    let mut seen_key_codes = HashSet::new();
    for binding in &bindings {
        if !seen_key_codes.insert(&binding.key_code) {
            return Err(serde::de::Error::custom(format!(
                "duplicate key_code '{}' in hotkey_bindings",
                binding.key_code
            )));
        }
    }

    Ok(bindings)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hotkey_binding_valid() {
        let json = r#"{"key_code": "ShiftLeft", "name": "Sprint", "event_actions": []}"#;
        let result: Result<HotkeyBinding, _> = serde_json::from_str(json);
        assert!(
            result.is_ok(),
            "Valid binding should parse: {:?}",
            result.err()
        );
        let binding = result.unwrap();
        assert_eq!(binding.key_code, "ShiftLeft");
        assert_eq!(binding.name, "Sprint");
        assert!(binding.event_actions.is_empty());
    }

    #[test]
    fn hotkey_binding_rejects_empty_key_code() {
        let json = r#"{"key_code": "", "name": "Sprint", "event_actions": []}"#;
        let result: Result<HotkeyBinding, _> = serde_json::from_str(json);
        assert!(result.is_err(), "Empty key_code should be rejected");
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("key_code must be 1 to 64 characters"),
            "Error: {}",
            err
        );
    }

    #[test]
    fn hotkey_binding_rejects_key_code_over_64() {
        let long_code = "a".repeat(65);
        let json = format!(
            r#"{{"key_code": "{}", "name": "Sprint", "event_actions": []}}"#,
            long_code
        );
        let result: Result<HotkeyBinding, _> = serde_json::from_str(&json);
        assert!(result.is_err(), "key_code over 64 chars should be rejected");
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("key_code must be 1 to 64 characters"),
            "Error: {}",
            err
        );
    }

    #[test]
    fn hotkey_binding_rejects_empty_name() {
        let json = r#"{"key_code": "KeyZ", "name": "", "event_actions": []}"#;
        let result: Result<HotkeyBinding, _> = serde_json::from_str(json);
        assert!(result.is_err(), "Empty name should be rejected");
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("name must be 1 to 64 characters"),
            "Error: {}",
            err
        );
    }

    #[test]
    fn hotkey_binding_rejects_name_over_64() {
        let long_name = "b".repeat(65);
        let json = format!(
            r#"{{"key_code": "KeyZ", "name": "{}", "event_actions": []}}"#,
            long_name
        );
        let result: Result<HotkeyBinding, _> = serde_json::from_str(&json);
        assert!(result.is_err(), "name over 64 chars should be rejected");
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("name must be 1 to 64 characters"),
            "Error: {}",
            err
        );
    }

    #[test]
    fn hotkey_binding_rejects_over_20_event_actions() {
        let actions: Vec<String> = (0..21)
            .map(|i| format!(r#"{{"type": "SetState", "key": "k{}", "value": "v"}}"#, i))
            .collect();
        let json = format!(
            r#"{{"key_code": "KeyZ", "name": "Test", "event_actions": [{}]}}"#,
            actions.join(", ")
        );
        let result: Result<HotkeyBinding, _> = serde_json::from_str(&json);
        assert!(result.is_err(), "21 event_actions should be rejected");
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("event_actions must have at most 20 entries"),
            "Error: {}",
            err
        );
    }

    #[test]
    fn hotkey_binding_event_actions_defaults_to_empty() {
        let json = r#"{"key_code": "Space", "name": "Jump"}"#;
        let result: Result<HotkeyBinding, _> = serde_json::from_str(json);
        assert!(
            result.is_ok(),
            "Missing event_actions should default to empty: {:?}",
            result.err()
        );
        assert!(result.unwrap().event_actions.is_empty());
    }

    #[test]
    fn hotkey_binding_round_trip() {
        let binding = HotkeyBinding {
            key_code: "ShiftLeft".to_string(),
            name: "Sprint".to_string(),
            event_actions: vec![EventAction::SetSpeed { multiplier: 2.0 }],
        };
        let json = serde_json::to_string(&binding).unwrap();
        let deserialized: HotkeyBinding = serde_json::from_str(&json).unwrap();
        assert_eq!(binding, deserialized);
    }

    #[test]
    fn deserialize_hotkey_bindings_rejects_over_32() {
        let entries: Vec<String> = (0..33)
            .map(|i| {
                format!(
                    r#"{{"key_code": "Key{}", "name": "Binding {}", "event_actions": []}}"#,
                    i, i
                )
            })
            .collect();
        let json = format!("[{}]", entries.join(", "));
        let mut deserializer = serde_json::Deserializer::from_str(&json);
        let result = deserialize_hotkey_bindings(&mut deserializer);
        assert!(result.is_err(), "33 bindings should be rejected");
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("hotkey_bindings must have at most 32 entries"),
            "Error: {}",
            err
        );
    }

    #[test]
    fn deserialize_hotkey_bindings_rejects_duplicate_key_code() {
        let json = r#"[
            {"key_code": "KeyZ", "name": "Action A", "event_actions": []},
            {"key_code": "KeyZ", "name": "Action B", "event_actions": []}
        ]"#;
        let mut deserializer = serde_json::Deserializer::from_str(json);
        let result = deserialize_hotkey_bindings(&mut deserializer);
        assert!(result.is_err(), "Duplicate key_code should be rejected");
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("duplicate key_code 'KeyZ' in hotkey_bindings"),
            "Error: {}",
            err
        );
    }

    #[test]
    fn deserialize_hotkey_bindings_accepts_unique_key_codes() {
        let json = r#"[
            {"key_code": "KeyZ", "name": "Action A", "event_actions": []},
            {"key_code": "KeyX", "name": "Action B", "event_actions": []}
        ]"#;
        let mut deserializer = serde_json::Deserializer::from_str(json);
        let result = deserialize_hotkey_bindings(&mut deserializer);
        assert!(
            result.is_ok(),
            "Unique key_codes should be accepted: {:?}",
            result.err()
        );
        assert_eq!(result.unwrap().len(), 2);
    }
}
