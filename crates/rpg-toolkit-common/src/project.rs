use std::collections::HashMap;

use serde::{Deserialize, Deserializer, Serialize};

use crate::ability::AbilityRegistry;
use crate::character::CharacterRegistry;
use crate::enemy::EnemyRegistry;
use crate::error::CommonError;
use crate::hotkey::{HotkeyBinding, deserialize_hotkey_bindings};
use crate::item::ItemRegistry;
use crate::map::{EventAction, MapData, MapId, SpawnPoint, TilesetId};
use crate::shop::ShopRegistry;
use crate::spritesheet::{CharacterSpritesheet, SpritesheetId};
use crate::tileset::TilesetMeta;

/// Custom deserializer that tolerates malformed JSON types for legacy HashMap fields.
/// If the value is a proper JSON object with string values, it deserializes normally.
/// For any other type (array, number, boolean, null, or object with non-string values),
/// it returns an empty HashMap.
fn deserialize_legacy_hashmap<'de, D>(deserializer: D) -> Result<HashMap<String, String>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    match value {
        serde_json::Value::Object(map) => {
            let mut result = HashMap::new();
            for (k, v) in map {
                if let serde_json::Value::String(s) = v {
                    result.insert(k, s);
                }
                // Non-string values in the map are silently skipped
            }
            Ok(result)
        }
        // Any non-object type (array, number, string, bool, null) → empty
        _ => Ok(HashMap::new()),
    }
}

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
    /// Hidden deserialization sink for legacy dialog_texts field (never serialized).
    #[serde(
        default,
        skip_serializing,
        rename = "dialog_texts",
        deserialize_with = "deserialize_legacy_hashmap"
    )]
    _legacy_dialog_texts: HashMap<String, String>,
    /// Hidden deserialization sink for legacy face_portraits field (never serialized).
    #[serde(
        default,
        skip_serializing,
        rename = "face_portraits",
        deserialize_with = "deserialize_legacy_hashmap"
    )]
    _legacy_face_portraits: HashMap<String, String>,
    /// Character registry: all playable characters defined in this project.
    #[serde(default)]
    pub characters: CharacterRegistry,
    /// Item registry: all items defined in this project.
    #[serde(default)]
    pub items: ItemRegistry,
    /// Ability registry: all abilities defined in this project.
    #[serde(default)]
    pub abilities: AbilityRegistry,
    /// Enemy registry: all enemies defined in this project.
    #[serde(default)]
    pub enemies: EnemyRegistry,
    /// Shop registry: all shops defined in this project.
    #[serde(default)]
    pub shops: ShopRegistry,
    /// Event actions to execute when a new game starts (after player spawns).
    #[serde(default)]
    pub intro_events: Option<Vec<EventAction>>,
    /// Hotkey bindings: keyboard shortcuts mapped to event action sequences.
    #[serde(default, deserialize_with = "deserialize_hotkey_bindings")]
    pub hotkey_bindings: Vec<HotkeyBinding>,
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
        characters: CharacterRegistry,
        items: ItemRegistry,
        abilities: AbilityRegistry,
        enemies: EnemyRegistry,
        shops: ShopRegistry,
    ) -> Self {
        Self {
            maps,
            tilesets,
            spawn_point,
            spritesheets,
            player_spritesheet,
            _legacy_dialog_texts: HashMap::new(),
            _legacy_face_portraits: HashMap::new(),
            characters,
            items,
            abilities,
            enemies,
            shops,
            intro_events: None,
            hotkey_bindings: Vec::new(),
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

        // Validate enemy IDs match their keys in the registry
        for (id, enemy) in &project.enemies.enemies {
            if id != &enemy.id {
                return Err(CommonError::ProjectValidationError(format!(
                    "enemy registry key '{}' does not match enemy id '{}'",
                    id, enemy.id
                )));
            }
        }

        // Validate shop IDs match their keys in the registry
        for (id, shop) in &project.shops.shops {
            if id != &shop.id {
                return Err(CommonError::ProjectValidationError(format!(
                    "shop registry key '{}' does not match shop id '{}'",
                    id, shop.id
                )));
            }
        }

        // Warn about shop entries referencing non-existent items
        for (shop_id, shop) in &project.shops.shops {
            for entry in &shop.entries {
                if !project.items.items.contains_key(&entry.item_id) {
                    eprintln!(
                        "warning: shop '{}' entry references non-existent item '{}'",
                        shop_id, entry.item_id
                    );
                }
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
            dialog_texts: HashMap::new(),
            face_portraits: HashMap::new(),
            characters: self.characters.clone(),
            items: self.items.clone(),
            abilities: self.abilities.clone(),
            enemies: self.enemies.clone(),
            shops: self.shops.clone(),
            intro_events: self.intro_events.clone(),
            hotkey_bindings: self.hotkey_bindings.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hotkey_bindings_defaults_to_empty_when_absent() {
        // A minimal project JSON without the hotkey_bindings field
        let json = r#"{
            "maps": {},
            "tilesets": {}
        }"#;
        let project: ProjectFile = serde_json::from_str(json).unwrap();
        assert!(
            project.hotkey_bindings.is_empty(),
            "hotkey_bindings should default to empty Vec when absent from JSON"
        );
    }

    #[test]
    fn project_with_map_defaults_both_parallax_layers_and_hotkey_bindings() {
        // A project with a map that has no parallax_layers and no hotkey_bindings field.
        // Verifies both fields default correctly when loaded together.
        let json = r#"{
            "maps": {
                "map-1": {
                    "name": "Village",
                    "width": 2,
                    "height": 2,
                    "tile_width": 16,
                    "tile_height": 16,
                    "layers": [{
                        "name": "Ground",
                        "visible": true,
                        "tiles": [[null, null], [null, null]],
                        "attributes": {"cells": [[{"opacity": false}, {"opacity": false}], [{"opacity": false}, {"opacity": false}]]}
                    }],
                    "active_layer_index": 0
                }
            },
            "tilesets": {}
        }"#;
        let project: ProjectFile = serde_json::from_str(json).unwrap();
        assert!(
            project.hotkey_bindings.is_empty(),
            "hotkey_bindings should default to empty Vec when absent"
        );
        let map = project.maps.get("map-1").expect("map-1 should exist");
        assert!(
            map.parallax_layers.is_empty(),
            "parallax_layers should default to empty Vec when absent from map"
        );
    }

    #[test]
    fn project_with_only_pre_existing_action_types_deserializes_without_error() {
        // A project file containing only pre-existing EventAction types (ShowDialog,
        // JumpTo, SetState, FadeTransition, ScreenShake) verifies backward compatibility.
        let json = r#"{
            "maps": {
                "map-1": {
                    "name": "Town",
                    "width": 2,
                    "height": 2,
                    "tile_width": 16,
                    "tile_height": 16,
                    "layers": [{
                        "name": "Ground",
                        "visible": true,
                        "tiles": [[null, null], [null, null]],
                        "attributes": {
                            "cells": [
                                [
                                    {
                                        "opacity": false,
                                        "event_trigger": [
                                            {
                                                "type": "ShowDialog",
                                                "text": {"type": "Inline", "value": "Hello!"},
                                                "config": {}
                                            },
                                            {
                                                "type": "SetState",
                                                "key": "talked",
                                                "value": "true"
                                            }
                                        ]
                                    },
                                    {"opacity": false}
                                ],
                                [
                                    {
                                        "opacity": false,
                                        "event_trigger": [
                                            {
                                                "type": "JumpTo",
                                                "target_map_id": "map-2",
                                                "target_x": 5,
                                                "target_y": 3
                                            }
                                        ]
                                    },
                                    {
                                        "opacity": false,
                                        "event_trigger": [
                                            {
                                                "type": "FadeTransition",
                                                "fade_type": "FadeOut",
                                                "duration": 1.0
                                            }
                                        ]
                                    }
                                ]
                            ]
                        }
                    }],
                    "active_layer_index": 0
                }
            },
            "tilesets": {}
        }"#;
        let project: ProjectFile = serde_json::from_str(json).unwrap();
        let map = project.maps.get("map-1").expect("map-1 should exist");
        // Verify the event triggers deserialized correctly
        let cell_00 = &map.layers[0].attributes.cells[0][0];
        assert_eq!(cell_00.event_trigger.len(), 2);
        let cell_10 = &map.layers[0].attributes.cells[1][0];
        assert_eq!(cell_10.event_trigger.len(), 1);
    }

    #[test]
    fn legacy_dialog_texts_and_face_portraits_absorbed_on_load() {
        // Legacy project with dialog_texts and face_portraits should deserialize fine
        let json = r#"{
            "maps": {},
            "tilesets": {},
            "dialog_texts": {"greeting": "Hello there!", "farewell": "Goodbye!"},
            "face_portraits": {"hero": "assets/hero_face.png"}
        }"#;
        let project: ProjectFile = serde_json::from_str(json).unwrap();
        // Fields are absorbed but not publicly accessible
        assert!(project.maps.is_empty());
        assert!(project.tilesets.is_empty());
    }

    #[test]
    fn dialog_texts_and_face_portraits_not_serialized() {
        // A project file should not contain dialog_texts or face_portraits when serialized
        let project = ProjectFile::new(
            HashMap::new(),
            HashMap::new(),
            None,
            HashMap::new(),
            None,
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
        );
        let json = project.serialize().unwrap();
        assert!(
            !json.contains("dialog_texts"),
            "serialized output should not contain dialog_texts"
        );
        assert!(
            !json.contains("face_portraits"),
            "serialized output should not contain face_portraits"
        );
    }

    #[test]
    fn malformed_dialog_texts_array_tolerated() {
        // dialog_texts as a JSON array should not cause a parse error
        let json = r#"{
            "maps": {},
            "tilesets": {},
            "dialog_texts": [1, 2, 3]
        }"#;
        let project: ProjectFile = serde_json::from_str(json).unwrap();
        assert!(project.maps.is_empty());
    }

    #[test]
    fn malformed_dialog_texts_number_tolerated() {
        // dialog_texts as a number should not cause a parse error
        let json = r#"{
            "maps": {},
            "tilesets": {},
            "dialog_texts": 42
        }"#;
        let project: ProjectFile = serde_json::from_str(json).unwrap();
        assert!(project.maps.is_empty());
    }

    #[test]
    fn malformed_face_portraits_boolean_tolerated() {
        // face_portraits as a boolean should not cause a parse error
        let json = r#"{
            "maps": {},
            "tilesets": {},
            "face_portraits": true
        }"#;
        let project: ProjectFile = serde_json::from_str(json).unwrap();
        assert!(project.maps.is_empty());
    }

    #[test]
    fn malformed_face_portraits_string_tolerated() {
        // face_portraits as a bare string should not cause a parse error
        let json = r#"{
            "maps": {},
            "tilesets": {},
            "face_portraits": "not_an_object"
        }"#;
        let project: ProjectFile = serde_json::from_str(json).unwrap();
        assert!(project.maps.is_empty());
    }

    #[test]
    fn malformed_face_portraits_null_tolerated() {
        // face_portraits as null should not cause a parse error
        let json = r#"{
            "maps": {},
            "tilesets": {},
            "face_portraits": null
        }"#;
        let project: ProjectFile = serde_json::from_str(json).unwrap();
        assert!(project.maps.is_empty());
    }

    #[test]
    fn legacy_project_round_trip_omits_removed_fields() {
        // Load a legacy project with dialog_texts and face_portraits,
        // save it, then reload — the removed fields should be gone
        let json = r#"{
            "maps": {},
            "tilesets": {},
            "dialog_texts": {"key1": "value1"},
            "face_portraits": {"portrait1": "path/to/portrait.png"}
        }"#;
        let project: ProjectFile = serde_json::from_str(json).unwrap();
        let saved_json = project.serialize().unwrap();
        assert!(!saved_json.contains("dialog_texts"));
        assert!(!saved_json.contains("face_portraits"));

        // Reload should work fine
        let reloaded: ProjectFile = serde_json::from_str(&saved_json).unwrap();
        assert!(reloaded.maps.is_empty());
    }
}
