use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::ability::AbilityRegistry;
use crate::character::CharacterRegistry;
use crate::error::CommonError;
use crate::item::ItemRegistry;
use crate::map::{DialogTextData, EventAction, MapData, MapId, SpawnPoint, TilesetId};
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
    /// Dialog text entries: Text_Id → text string.
    #[serde(default)]
    pub dialog_texts: HashMap<String, String>,
    /// Face portrait entries: portrait ID → asset path.
    #[serde(default)]
    pub face_portraits: HashMap<String, String>,
    /// Character registry: all playable characters defined in this project.
    #[serde(default)]
    pub characters: CharacterRegistry,
    /// Item registry: all items defined in this project.
    #[serde(default)]
    pub items: ItemRegistry,
    /// Ability registry: all abilities defined in this project.
    #[serde(default)]
    pub abilities: AbilityRegistry,
}

#[allow(clippy::too_many_arguments)]
impl ProjectFile {
    /// Creates a new `ProjectFile`.
    pub fn new(
        maps: HashMap<MapId, MapData>,
        tilesets: HashMap<TilesetId, TilesetMeta>,
        spawn_point: Option<SpawnPoint>,
        spritesheets: HashMap<SpritesheetId, CharacterSpritesheet>,
        player_spritesheet: Option<SpritesheetId>,
        dialog_texts: HashMap<String, String>,
        face_portraits: HashMap<String, String>,
        characters: CharacterRegistry,
        items: ItemRegistry,
        abilities: AbilityRegistry,
    ) -> Self {
        Self {
            maps,
            tilesets,
            spawn_point,
            spritesheets,
            player_spritesheet,
            dialog_texts,
            face_portraits,
            characters,
            items,
            abilities,
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
    ///
    /// Note on duplicate character IDs: Since `CharacterRegistry` uses a `HashMap<CharacterId, Character>`,
    /// serde's default deserialization applies last-wins semantics for duplicate keys. This means
    /// duplicate character IDs in hand-edited JSON will not produce an error but will silently
    /// keep the last entry. This aligns with HashMap's natural deduplication behavior.
    pub fn deserialize(json: &str) -> Result<Self, CommonError> {
        let project: Self = serde_json::from_str(json)
            .map_err(|e| CommonError::ProjectParseError(e.to_string()))?;

        // Validate character IDs match their keys in the registry
        for (id, character) in &project.characters.characters {
            if id != &character.id {
                return Err(CommonError::ProjectValidationError(format!(
                    "character registry key '{}' does not match character id '{}'",
                    id, character.id
                )));
            }
        }

        // Validate item IDs match their keys in the registry
        for (id, item) in &project.items.items {
            if id != &item.id {
                return Err(CommonError::ProjectValidationError(format!(
                    "item registry key '{}' does not match item id '{}'",
                    id, item.id
                )));
            }
        }

        // Validate ability IDs match their keys in the registry
        for (id, ability) in &project.abilities.abilities {
            if id != &ability.id {
                return Err(CommonError::ProjectValidationError(format!(
                    "ability registry key '{}' does not match ability id '{}'",
                    id, ability.id
                )));
            }
        }

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
                                EventAction::ShowDialog {
                                    text: DialogTextData::Id(text_id),
                                    ..
                                } if !project.dialog_texts.contains_key(text_id) => {
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

        if let Some(player_ss) = &self.player_spritesheet
            && player_ss == spritesheet_id
        {
            refs.player_reference = true;
        }

        refs
    }

    /// Serialize project to a directory-based format.
    /// Writes `manifest.json` and `maps/<id>.json` for each map.
    pub fn serialize_to_dir(&self, root: &std::path::Path) -> Result<(), CommonError> {
        let maps_dir = root.join("maps");
        std::fs::create_dir_all(&maps_dir).map_err(|e| {
            CommonError::ProjectParseError(format!("could not create maps directory: {}", e))
        })?;

        for (map_id, map) in &self.maps {
            let map_path = maps_dir.join(format!("{}.json", map_id));
            let json = serde_json::to_string_pretty(map).map_err(|e| {
                CommonError::ProjectParseError(format!(
                    "failed to serialize map '{}': {}",
                    map_id, e
                ))
            })?;
            std::fs::write(&map_path, &json).map_err(|e| {
                CommonError::ProjectParseError(format!("could not write map file: {}", e))
            })?;
        }

        let manifest = self.to_manifest();
        manifest.save_to_dir(root)?;
        Ok(())
    }

    /// Deserialize project from a directory-based format.
    pub fn deserialize_from_dir(root: &std::path::Path) -> Result<Self, CommonError> {
        let manifest = crate::manifest::ProjectManifest::load_from_dir(root)?;

        let errors = manifest.validate_refs(root);
        if !errors.is_empty() {
            return Err(CommonError::ProjectValidationError(errors.join("; ")));
        }

        manifest.into_project_file(root)
    }

    /// Deserialize project from ZIP bytes.
    pub fn deserialize_from_zip(
        zip_data: &[u8],
        temp_dir: &std::path::Path,
    ) -> Result<Self, CommonError> {
        let mut archive = zip::ZipArchive::new(std::io::Cursor::new(zip_data))
            .map_err(|e| CommonError::ZipError(format!("failed to open zip: {}", e)))?;
        archive
            .extract(temp_dir)
            .map_err(|e| CommonError::ZipError(format!("failed to extract zip: {}", e)))?;
        Self::deserialize_from_dir(temp_dir)
    }

    /// Convert to a manifest (maps become just IDs).
    pub fn to_manifest(&self) -> crate::manifest::ProjectManifest {
        crate::manifest::ProjectManifest {
            maps: self.maps.keys().cloned().collect(),
            tilesets: self.tilesets.clone(),
            spawn_point: self.spawn_point.clone(),
            spritesheets: self.spritesheets.clone(),
            player_spritesheet: self.player_spritesheet.clone(),
            dialog_texts: self.dialog_texts.clone(),
            face_portraits: self.face_portraits.clone(),
            characters: self.characters.clone(),
            items: self.items.clone(),
            abilities: self.abilities.clone(),
        }
    }
}
