use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

/// Serializable representation of a character's progress for the save file.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CharacterProgressData {
    #[serde(default)]
    pub experience: u64,
    #[serde(default)]
    pub learned_abilities: Vec<String>,
}

/// On-disk save file format.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct SaveFile {
    /// Game state flags (key-value string pairs).
    #[serde(default)]
    pub state: BTreeMap<String, String>,

    /// Player's currency balance.
    #[serde(default)]
    pub currency: u64,

    /// Player's inventory: item_id → quantity.
    #[serde(default)]
    pub inventory: BTreeMap<String, u32>,

    /// Active party member character IDs (ordered).
    #[serde(default)]
    pub party: Vec<String>,

    /// Per-character progress: character_id → progress data.
    #[serde(default)]
    pub character_progress: BTreeMap<String, CharacterProgressData>,

    /// The map the player was on when saving (UUID v4 format).
    #[serde(default)]
    pub map_id: Option<String>,

    /// The player's grid coordinates (column, row) when saving.
    #[serde(default)]
    pub position: Option<(u32, u32)>,

    /// The player's elevation level when saving.
    #[serde(default)]
    pub elevation: Option<u32>,
}

impl SaveFile {
    /// Load a save file from disk, or return default if it doesn't exist.
    pub fn load(path: &Path) -> Self {
        match std::fs::read_to_string(path) {
            Ok(contents) => serde_json::from_str(&contents).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    /// Write the save file to disk.
    pub fn save(&self, path: &Path) -> Result<(), String> {
        let parent = path
            .parent()
            .ok_or_else(|| format!("save path has no parent directory: {}", path.display()))?;
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("could not create save directory: {}", e))?;
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("could not serialize save file: {}", e))?;
        std::fs::write(path, json).map_err(|e| format!("could not write save file: {}", e))
    }
}
