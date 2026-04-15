use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::error::CommonError;
use crate::map::{EventAction, MapData, MapId, SpawnPoint, TilesetId};
use crate::tileset::TilesetMeta;

/// The on-disk project format (multi-map, multi-tileset).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ProjectFile {
    pub maps: HashMap<MapId, MapData>,
    pub tilesets: HashMap<TilesetId, TilesetMeta>,
    #[serde(default)]
    pub spawn_point: Option<SpawnPoint>,
}

impl ProjectFile {
    /// Creates a new `ProjectFile`.
    pub fn new(
        maps: HashMap<MapId, MapData>,
        tilesets: HashMap<TilesetId, TilesetMeta>,
        spawn_point: Option<SpawnPoint>,
    ) -> Self {
        Self {
            maps,
            tilesets,
            spawn_point,
        }
    }

    /// Serializes the project to pretty-printed JSON.
    pub fn serialize(&self) -> Result<String, CommonError> {
        serde_json::to_string_pretty(self)
            .map_err(|e| CommonError::ProjectParseError(e.to_string()))
    }

    /// Deserializes a project from a JSON string, validates each map,
    /// and checks that all `TileRef` tileset IDs reference tilesets
    /// present in the project.
    pub fn deserialize(json: &str) -> Result<Self, CommonError> {
        let project: Self = serde_json::from_str(json)
            .map_err(|e| CommonError::ProjectParseError(e.to_string()))?;

        // Validate each map
        for (map_id, map) in &project.maps {
            map.validate().map_err(|e| {
                CommonError::ProjectValidationError(format!(
                    "map '{}' validation failed: {}",
                    map_id, e
                ))
            })?;
        }

        // Check that all TileRef tileset IDs exist in the tilesets registry
        for (map_id, map) in &project.maps {
            for (layer_idx, layer) in map.layers.iter().enumerate() {
                for (y, row) in layer.tiles.iter().enumerate() {
                    for (x, cell) in row.iter().enumerate() {
                        if let Some(tile_ref) = cell
                            && !project.tilesets.contains_key(&tile_ref.tileset_id)
                        {
                            return Err(CommonError::ProjectValidationError(format!(
                                "map '{}' layer {} tile ({},{}) references unknown tileset '{}'",
                                map_id, layer_idx, x, y, tile_ref.tileset_id
                            )));
                        }
                    }
                }
            }
        }

        // Warn about JumpTo actions referencing non-existent maps (preserve data, just log)
        for (map_id, map) in &project.maps {
            for (layer_idx, layer) in map.layers.iter().enumerate() {
                for (y, row) in layer.attributes.cells.iter().enumerate() {
                    for (x, attrs) in row.iter().enumerate() {
                        for action in &attrs.event_trigger {
                            match action {
                                EventAction::JumpTo { target_map_id, .. }
                                    if !project.maps.contains_key(target_map_id) =>
                                {
                                    eprintln!(
                                        "warning: map '{}' layer {} tile ({},{}) has JumpTo referencing non-existent map '{}'",
                                        map_id, layer_idx, x, y, target_map_id
                                    );
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }

        Ok(project)
    }
}
