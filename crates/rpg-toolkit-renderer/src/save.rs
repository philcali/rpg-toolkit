// Re-export SaveFile and CharacterProgressData from rpg-toolkit-common for backward compatibility.
pub use rpg_toolkit_common::{CharacterProgressData, SaveFile};

use crate::resources::{
    CharacterProgressState, CurrencyState, GameState, InventoryState, PartyState, SavePath,
};

/// Serialize all game state resources into a SaveFile and write to disk.
///
/// This is NOT a Bevy system — it is a standalone function intended to be called
/// by a future "save point" EventAction handler.
#[allow(clippy::too_many_arguments)]
pub fn save_game(
    game_state: &GameState,
    currency: &CurrencyState,
    inventory: &InventoryState,
    party: &PartyState,
    character_progress: &CharacterProgressState,
    save_path: &SavePath,
    map_id: Option<&str>,
    position: Option<(u32, u32)>,
    elevation: Option<u32>,
) -> Result<(), String> {
    let save_file = SaveFile {
        state: game_state
            .flags
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect(),
        currency: currency.balance,
        inventory: inventory
            .items
            .iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect(),
        party: party.members.clone(),
        character_progress: character_progress
            .characters
            .iter()
            .map(|(id, progress)| {
                (
                    id.clone(),
                    CharacterProgressData {
                        experience: progress.experience,
                        learned_abilities: progress.learned_abilities.clone(),
                    },
                )
            })
            .collect(),
        map_id: map_id.map(|s| s.to_string()),
        position,
        elevation,
        shop_stock: std::collections::BTreeMap::new(),
    };

    save_file.save(&save_path.path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resources::{
        CharacterProgress, CharacterProgressState, CurrencyState, GameState, InventoryState,
        PartyState, SavePath,
    };
    use std::collections::HashMap;

    /// Requirement 13.1: Old save file with only `state` field deserializes into
    /// new SaveFile with zeros/empty defaults for new fields.
    #[test]
    fn old_save_file_with_only_state_deserializes_with_defaults() {
        let json = r#"{"state": {"quest_complete": "true", "door_open": "yes"}}"#;
        let save: SaveFile =
            serde_json::from_str(json).expect("should deserialize old save format");

        assert_eq!(save.state.len(), 2);
        assert_eq!(save.state.get("quest_complete"), Some(&"true".to_string()));
        assert_eq!(save.state.get("door_open"), Some(&"yes".to_string()));
        // New fields should default to zero/empty
        assert_eq!(save.currency, 0);
        assert!(save.inventory.is_empty());
        assert!(save.party.is_empty());
        assert!(save.character_progress.is_empty());
    }

    /// Requirement 13.1: Completely empty JSON object deserializes into default SaveFile.
    #[test]
    fn empty_json_object_deserializes_to_defaults() {
        let json = r#"{}"#;
        let save: SaveFile = serde_json::from_str(json).expect("should deserialize empty object");

        assert!(save.state.is_empty());
        assert_eq!(save.currency, 0);
        assert!(save.inventory.is_empty());
        assert!(save.party.is_empty());
        assert!(save.character_progress.is_empty());
    }

    /// Verify `save_game` produces a SaveFile matching input resource state.
    #[test]
    fn save_game_produces_correct_save_file() {
        let tmp_dir = std::env::temp_dir().join("rpg_toolkit_test_save_game");
        let _ = std::fs::create_dir_all(&tmp_dir);
        let save_path_buf = tmp_dir.join("test_save.json");

        let game_state = GameState {
            flags: {
                let mut m = HashMap::new();
                m.insert("boss_defeated".to_string(), "true".to_string());
                m.insert("chapter".to_string(), "3".to_string());
                m
            },
        };
        let currency = CurrencyState { balance: 5000 };
        let inventory = InventoryState {
            items: {
                let mut m = HashMap::new();
                m.insert("potion".to_string(), 5);
                m.insert("sword".to_string(), 1);
                m
            },
        };
        let party = PartyState {
            members: vec!["hero".to_string(), "mage".to_string()],
        };
        let character_progress = CharacterProgressState {
            characters: {
                let mut m = HashMap::new();
                m.insert(
                    "hero".to_string(),
                    CharacterProgress {
                        experience: 1500,
                        learned_abilities: vec!["slash".to_string(), "heal".to_string()],
                    },
                );
                m.insert(
                    "mage".to_string(),
                    CharacterProgress {
                        experience: 1200,
                        learned_abilities: vec!["fireball".to_string()],
                    },
                );
                m
            },
        };
        let save_path = SavePath {
            path: save_path_buf.clone(),
        };

        // Call save_game
        save_game(
            &game_state,
            &currency,
            &inventory,
            &party,
            &character_progress,
            &save_path,
            None,
            None,
            None,
        )
        .expect("save_game should succeed");

        // Load the file back and verify
        let saved: SaveFile = {
            let contents = std::fs::read_to_string(&save_path_buf).expect("should read save file");
            serde_json::from_str(&contents).expect("should parse save file")
        };

        assert_eq!(saved.state.get("boss_defeated"), Some(&"true".to_string()));
        assert_eq!(saved.state.get("chapter"), Some(&"3".to_string()));
        assert_eq!(saved.currency, 5000);
        assert_eq!(saved.inventory.get("potion"), Some(&5));
        assert_eq!(saved.inventory.get("sword"), Some(&1));
        assert_eq!(saved.party, vec!["hero".to_string(), "mage".to_string()]);
        assert_eq!(saved.character_progress.len(), 2);

        let hero_progress = saved
            .character_progress
            .get("hero")
            .expect("hero should exist");
        assert_eq!(hero_progress.experience, 1500);
        assert_eq!(hero_progress.learned_abilities, vec!["slash", "heal"]);

        let mage_progress = saved
            .character_progress
            .get("mage")
            .expect("mage should exist");
        assert_eq!(mage_progress.experience, 1200);
        assert_eq!(mage_progress.learned_abilities, vec!["fireball"]);

        // Cleanup
        let _ = std::fs::remove_file(&save_path_buf);
        let _ = std::fs::remove_dir(&tmp_dir);
    }

    /// Verify CharacterProgressData preserves experience and learned_abilities
    /// through serialization/deserialization.
    #[test]
    fn character_progress_data_round_trip() {
        let data = CharacterProgressData {
            experience: 99999,
            learned_abilities: vec![
                "fireball".to_string(),
                "ice_shield".to_string(),
                "teleport".to_string(),
            ],
        };

        let json = serde_json::to_string(&data).expect("should serialize");
        let deserialized: CharacterProgressData =
            serde_json::from_str(&json).expect("should deserialize");

        assert_eq!(deserialized.experience, 99999);
        assert_eq!(
            deserialized.learned_abilities,
            vec!["fireball", "ice_shield", "teleport"]
        );
        assert_eq!(data, deserialized);
    }

    /// Verify CharacterProgressData defaults when fields are absent.
    #[test]
    fn character_progress_data_defaults_on_empty_json() {
        let json = r#"{}"#;
        let data: CharacterProgressData =
            serde_json::from_str(json).expect("should deserialize empty");

        assert_eq!(data.experience, 0);
        assert!(data.learned_abilities.is_empty());
    }

    /// Verify empty resources produce a valid minimal save file.
    #[test]
    fn empty_resources_produce_valid_minimal_save_file() {
        let tmp_dir = std::env::temp_dir().join("rpg_toolkit_test_empty_save");
        let _ = std::fs::create_dir_all(&tmp_dir);
        let save_path_buf = tmp_dir.join("empty_save.json");

        let game_state = GameState::default();
        let currency = CurrencyState::default();
        let inventory = InventoryState::default();
        let party = PartyState::default();
        let character_progress = CharacterProgressState::default();
        let save_path = SavePath {
            path: save_path_buf.clone(),
        };

        save_game(
            &game_state,
            &currency,
            &inventory,
            &party,
            &character_progress,
            &save_path,
            None,
            None,
            None,
        )
        .expect("save_game with empty resources should succeed");

        // Load back and verify it's a valid minimal save file
        let saved: SaveFile = {
            let contents = std::fs::read_to_string(&save_path_buf).expect("should read save file");
            serde_json::from_str(&contents).expect("should parse save file")
        };

        assert!(saved.state.is_empty());
        assert_eq!(saved.currency, 0);
        assert!(saved.inventory.is_empty());
        assert!(saved.party.is_empty());
        assert!(saved.character_progress.is_empty());

        // Verify the JSON is well-formed
        let contents = std::fs::read_to_string(&save_path_buf).expect("should read");
        let _: serde_json::Value = serde_json::from_str(&contents).expect("should be valid JSON");

        // Cleanup
        let _ = std::fs::remove_file(&save_path_buf);
        let _ = std::fs::remove_dir(&tmp_dir);
    }
}
