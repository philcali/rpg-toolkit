use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::error::CommonError;
use crate::map::{EventAction, MapData, MapId, SpawnPoint, TilesetId};
use crate::spritesheet::{CharacterSpritesheet, SpritesheetId};
use crate::tileset::TilesetMeta;

/// Describes which entities reference a given spritesheet.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SpritesheetReferences {
    /// (map_id, npc_index) pairs for NPCs referencing this spritesheet.
    pub npc_references: Vec<(MapId, usize)>,
    /// Whether the player spritesheet reference points to this spritesheet.
    pub player_reference: bool,
}

/// The on-disk project format (multi-map, multi-tileset).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ProjectFile {
    pub maps: HashMap<MapId, MapData>,
    pub tilesets: HashMap<TilesetId, TilesetMeta>,
    #[serde(default)]
    pub spawn_point: Option<SpawnPoint>,
    #[serde(default)]
    pub spritesheets: HashMap<SpritesheetId, CharacterSpritesheet>,
    #[serde(default)]
    pub player_spritesheet: Option<SpritesheetId>,
}

impl ProjectFile {
    /// Creates a new `ProjectFile`.
    pub fn new(
        maps: HashMap<MapId, MapData>,
        tilesets: HashMap<TilesetId, TilesetMeta>,
        spawn_point: Option<SpawnPoint>,
        spritesheets: HashMap<SpritesheetId, CharacterSpritesheet>,
        player_spritesheet: Option<SpritesheetId>,
    ) -> Self {
        Self {
            maps,
            tilesets,
            spawn_point,
            spritesheets,
            player_spritesheet,
        }
    }

    /// Serializes the project to pretty-printed JSON.
    pub fn serialize(&self) -> Result<String, CommonError> {
        serde_json::to_string_pretty(self)
            .map_err(|e| CommonError::ProjectParseError(e.to_string()))
    }

    /// Deserializes a project from a JSON string, validates each map,
    /// and checks that all `TileRef` tileset IDs and NPC spritesheet IDs
    /// reference entries present in the project.
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

        // Validate NPC spritesheet references
        for (map_id, map) in &project.maps {
            for (npc_idx, npc) in map.npcs.iter().enumerate() {
                if !project.spritesheets.contains_key(&npc.spritesheet_id) {
                    return Err(CommonError::ProjectValidationError(format!(
                        "map '{}' NPC {} references unknown spritesheet '{}'",
                        map_id, npc_idx, npc.spritesheet_id
                    )));
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

    /// Returns which NPCs and player reference a given spritesheet.
    pub fn compute_spritesheet_references(
        &self,
        spritesheet_id: &SpritesheetId,
    ) -> SpritesheetReferences {
        let mut refs = SpritesheetReferences::default();

        for (map_id, map) in &self.maps {
            for (npc_idx, npc) in map.npcs.iter().enumerate() {
                if npc.spritesheet_id == *spritesheet_id {
                    refs.npc_references.push((map_id.clone(), npc_idx));
                }
            }
        }

        if let Some(player_ss) = &self.player_spritesheet {
            if player_ss == spritesheet_id {
                refs.player_reference = true;
            }
        }

        refs
    }
}
