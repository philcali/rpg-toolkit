use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

/// On-disk save file format.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct SaveFile {
    #[serde(default)]
    pub state: BTreeMap<String, String>,
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
