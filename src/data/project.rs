use serde::{Deserialize, Serialize};

use super::editor_state::EditorError;
use super::map::MapData;
use super::tileset::TilesetMeta;

/// The on-disk project format.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ProjectFile {
    pub version: u32, // schema version, starts at 1
    pub map: MapData,
    pub tileset: Option<TilesetMeta>,
}

impl ProjectFile {
    /// Creates a new ProjectFile with version 1.
    pub fn new(map: MapData, tileset: Option<TilesetMeta>) -> Self {
        Self {
            version: 1,
            map,
            tileset,
        }
    }

    /// Serializes the project to pretty-printed JSON.
    pub fn serialize(&self) -> Result<String, EditorError> {
        serde_json::to_string_pretty(self)
            .map_err(|e| EditorError::ProjectParseError(e.to_string()))
    }

    /// Deserializes a project from a JSON string, then validates the map data.
    pub fn deserialize(json: &str) -> Result<Self, EditorError> {
        let project: Self = serde_json::from_str(json)
            .map_err(|e| EditorError::ProjectParseError(e.to_string()))?;
        project.map.validate()?;
        Ok(project)
    }
}
