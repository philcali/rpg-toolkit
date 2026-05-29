use std::collections::HashMap;
use std::io::Read;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::CommonError;
use crate::map::{MapData, MapId, SpawnPoint, TilesetId};
use crate::spritesheet::{CharacterSpritesheet, SpritesheetId};
use crate::tileset::TilesetMeta;

/// On-disk manifest: lightweight summary of project contents.
/// Maps are stored as IDs only; full map data is loaded lazily from `maps/<id>.json`.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectManifest {
    /// Map IDs in loading order.
    pub maps: Vec<String>,
    pub tilesets: HashMap<TilesetId, TilesetMeta>,
    #[serde(default)]
    pub spawn_point: Option<SpawnPoint>,
    #[serde(default)]
    pub spritesheets: HashMap<SpritesheetId, CharacterSpritesheet>,
    #[serde(default)]
    pub player_spritesheet: Option<SpritesheetId>,
    #[serde(default)]
    pub dialog_texts: HashMap<String, String>,
}

impl ProjectManifest {
    /// Deserialize from raw JSON bytes.
    pub fn from_bytes(json: &[u8]) -> Result<Self, CommonError> {
        serde_json::from_slice(json).map_err(|e| CommonError::ProjectParseError(e.to_string()))
    }

    /// Serialize to pretty JSON bytes.
    pub fn to_bytes(&self) -> Result<Vec<u8>, CommonError> {
        serde_json::to_vec_pretty(self).map_err(|e| CommonError::ProjectParseError(e.to_string()))
    }

    /// Load manifest from `manifest.json` in a directory.
    pub fn load_from_dir(root: &Path) -> Result<Self, CommonError> {
        let manifest_path = root.join("manifest.json");
        let json = std::fs::read_to_string(&manifest_path).map_err(|e| {
            CommonError::ProjectParseError(format!(
                "could not read manifest.json at {}: {}",
                manifest_path.display(),
                e
            ))
        })?;
        Self::from_bytes(json.as_bytes())
            .map_err(|e| CommonError::ProjectParseError(format!("failed to parse manifest: {}", e)))
    }

    /// Save manifest to `manifest.json` in a directory.
    pub fn save_to_dir(&self, root: &Path) -> Result<(), CommonError> {
        let manifest_path = root.join("manifest.json");
        let json = self.to_bytes()?;
        std::fs::write(&manifest_path, &json).map_err(|e| {
            CommonError::ProjectParseError(format!(
                "could not write manifest.json at {}: {}",
                manifest_path.display(),
                e
            ))
        })
    }

    /// Load manifest from ZIP bytes.
    pub fn load_from_zip(zip_data: &[u8]) -> Result<Self, CommonError> {
        let mut archive = zip::ZipArchive::new(std::io::Cursor::new(zip_data))
            .map_err(|e| CommonError::ZipError(format!("failed to open zip: {}", e)))?;
        let mut manifest_file = archive
            .by_name("manifest.json")
            .map_err(|e| CommonError::ZipError(format!("manifest.json not found in zip: {}", e)))?;
        let mut json = String::new();
        manifest_file.read_to_string(&mut json).map_err(|e| {
            CommonError::ZipError(format!("failed to read manifest.json from zip: {}", e))
        })?;
        Self::from_bytes(json.as_bytes())
    }

    /// Convert manifest to full `ProjectFile` by loading all map files.
    /// Validates each map and checks cross-references.
    pub fn into_project_file(self, root: &Path) -> Result<crate::ProjectFile, CommonError> {
        let maps = self.load_maps(root)?;

        // Validate each map
        for (map_id, map) in &maps {
            map.validate().map_err(|e| {
                CommonError::ProjectValidationError(format!(
                    "map '{}' validation failed: {}",
                    map_id, e
                ))
            })?;
        }

        // Check that all TileRef tileset IDs exist in the tilesets registry
        for (map_id, map) in &maps {
            for (layer_idx, layer) in map.layers.iter().enumerate() {
                for (y, row) in layer.tiles.iter().enumerate() {
                    for (x, cell) in row.iter().enumerate() {
                        if let Some(tile_ref) = cell
                            && !self.tilesets.contains_key(&tile_ref.tileset_id)
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
        for (map_id, map) in &maps {
            for (npc_idx, npc) in map.npcs.iter().enumerate() {
                if !self.spritesheets.contains_key(&npc.spritesheet_id) {
                    return Err(CommonError::ProjectValidationError(format!(
                        "map '{}' NPC {} references unknown spritesheet '{}'",
                        map_id, npc_idx, npc.spritesheet_id
                    )));
                }
            }
        }

        // Warn about JumpTo actions referencing non-existent maps (preserve data, just log)
        for (map_id, map) in &maps {
            for (layer_idx, layer) in map.layers.iter().enumerate() {
                for (y, row) in layer.attributes.cells.iter().enumerate() {
                    for (x, attrs) in row.iter().enumerate() {
                        for action in &attrs.event_trigger {
                            match action {
                                crate::map::EventAction::JumpTo { target_map_id, .. }
                                    if !self.maps.contains(target_map_id) =>
                                {
                                    eprintln!(
                                        "warning: map '{}' layer {} tile ({},{}) has JumpTo referencing non-existent map '{}'",
                                        map_id, layer_idx, x, y, target_map_id
                                    );
                                }
                                crate::map::EventAction::ShowDialog {
                                    text: crate::map::DialogTextData::Id(text_id),
                                    ..
                                } if !self.dialog_texts.contains_key(text_id) => {
                                    eprintln!(
                                        "warning: map '{}' layer {} tile ({},{}) has ShowDialog referencing non-existent text ID '{}'",
                                        map_id, layer_idx, x, y, text_id
                                    );
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }

        Ok(crate::ProjectFile::new(
            maps,
            self.tilesets,
            self.spawn_point,
            self.spritesheets,
            self.player_spritesheet,
            self.dialog_texts,
        ))
    }

    /// Load all map files from `maps/` directory.
    fn load_maps(&self, root: &Path) -> Result<HashMap<MapId, MapData>, CommonError> {
        let mut maps = HashMap::new();
        let maps_dir = root.join("maps");
        for map_id in &self.maps {
            let map_path = maps_dir.join(format!("{}.json", map_id));
            let json = std::fs::read_to_string(&map_path).map_err(|e| {
                CommonError::ProjectParseError(format!(
                    "could not read map file {}: {}",
                    map_path.display(),
                    e
                ))
            })?;
            let map: MapData = serde_json::from_str(&json).map_err(|e| {
                CommonError::ProjectParseError(format!("failed to parse map {}: {}", map_id, e))
            })?;
            maps.insert(map_id.clone(), map);
        }
        Ok(maps)
    }

    /// Validate all referenced files exist.
    /// Returns a list of error messages (empty = all valid).
    pub fn validate_refs(&self, root: &Path) -> Vec<String> {
        let mut errors = Vec::new();
        let maps_dir = root.join("maps");

        for map_id in &self.maps {
            if !maps_dir.join(format!("{}.json", map_id)).exists() {
                errors.push(format!("map file missing: maps/{}.json", map_id));
            }
        }

        for (id, meta) in &self.tilesets {
            let path = root.join(&meta.file_path);
            if !path.exists() {
                errors.push(format!(
                    "tileset file missing: {} (referenced by {})",
                    meta.file_path, id
                ));
            }
        }

        for (id, ss) in &self.spritesheets {
            let path = root.join(&ss.file_path);
            if !path.exists() {
                errors.push(format!(
                    "spritesheet file missing: {} (referenced by {})",
                    ss.file_path, id
                ));
            }
        }

        if let Some(ref sp) = self.spawn_point
            && !self.maps.contains(&sp.map_id)
        {
            errors.push(format!(
                "spawn point references non-existent map: {}",
                sp.map_id
            ));
        }

        errors
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn manifest_round_trip() {
        let mut tilesets = HashMap::new();
        tilesets.insert(
            "ts-1".to_string(),
            TilesetMeta {
                file_path: "tilesets/base.png".to_string(),
                tile_width: 16,
                tile_height: 16,
                columns: 16,
                rows: 16,
            },
        );

        let mut spritesheets = HashMap::new();
        spritesheets.insert(
            "ss-1".to_string(),
            CharacterSpritesheet {
                file_path: "data/hero.png".to_string(),
                sprite_width: 24,
                sprite_height: 32,
                frame_count: 3,
                direction_count: 4,
            },
        );

        let manifest = ProjectManifest {
            maps: vec!["map-1".to_string(), "map-2".to_string()],
            tilesets,
            spawn_point: Some(SpawnPoint {
                map_id: "map-1".to_string(),
                x: 5,
                y: 5,
            }),
            spritesheets,
            player_spritesheet: Some("ss-1".to_string()),
            dialog_texts: HashMap::new(),
        };

        let bytes = manifest.to_bytes().unwrap();
        let loaded = ProjectManifest::from_bytes(&bytes).unwrap();
        assert_eq!(manifest, loaded);
    }

    #[test]
    fn dir_serialize_deserialize() {
        let tmp = std::env::temp_dir().join("rpg-toolkit-test-dir-project");
        let _ = fs::remove_dir_all(&tmp);

        let map = MapData::new("test", 8, 8, 16, 16).unwrap();
        let mut maps = HashMap::new();
        maps.insert("map-1".to_string(), map);

        let project_file = crate::ProjectFile::new(
            maps,
            HashMap::new(),
            None,
            HashMap::new(),
            None,
            HashMap::new(),
        );

        project_file.serialize_to_dir(&tmp).unwrap();
        assert!(tmp.join("manifest.json").exists());
        assert!(tmp.join("maps").is_dir());
        assert!(tmp.join("maps/map-1.json").exists());

        let loaded = crate::ProjectFile::deserialize_from_dir(&tmp).unwrap();
        assert_eq!(loaded.maps.len(), 1);
        assert!(loaded.maps.contains_key("map-1"));

        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn validate_refs_finds_missing() {
        let tmp = std::env::temp_dir().join("rpg-toolkit-test-validate");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(tmp.join("maps")).ok();

        let manifest = ProjectManifest {
            maps: vec!["map-1".to_string(), "map-missing".to_string()],
            tilesets: {
                let mut m = HashMap::new();
                m.insert(
                    "ts-1".to_string(),
                    TilesetMeta {
                        file_path: "tilesets/nonexistent.png".to_string(),
                        tile_width: 16,
                        tile_height: 16,
                        columns: 16,
                        rows: 16,
                    },
                );
                m
            },
            spawn_point: Some(SpawnPoint {
                map_id: "map-1".to_string(),
                x: 0,
                y: 0,
            }),
            spritesheets: HashMap::new(),
            player_spritesheet: None,
            dialog_texts: HashMap::new(),
        };

        let errors = manifest.validate_refs(&tmp);
        assert!(errors.len() >= 2); // missing map file + missing tileset
        assert!(errors.iter().any(|e| e.contains("map-missing")));
        assert!(errors.iter().any(|e| e.contains("nonexistent")));

        let _ = fs::remove_dir_all(&tmp);
    }
}
