use std::collections::HashMap;

use bevy::prelude::*;

// Re-export ProjectFile from common so existing `use crate::data::ProjectFile` paths work.
pub use rpg_toolkit_common::ProjectFile;

use rpg_toolkit_common::{
    AbilityRegistry, CharacterRegistry, CharacterSpritesheet, EnemyRegistry, EventAction,
    ItemRegistry, MapData, MapId, ShopRegistry, SpawnPoint, SpritesheetId, TilesetId,
};

use super::state::EditorError;
use super::tileset::{TilesetEntry, TilesetMeta};
use super::undo::UndoHistory;

/// The central project resource holding all maps, tilesets, and editor state.
#[derive(Resource, Default)]
pub struct Project {
    pub maps: HashMap<MapId, MapData>,
    pub tilesets: HashMap<TilesetId, TilesetEntry>,
    pub open_tabs: Vec<MapId>,
    pub active_tab: Option<usize>,
    pub undo_histories: HashMap<MapId, UndoHistory>,
    pub has_unsaved_changes: HashMap<MapId, bool>,
    pub spawn_point: Option<SpawnPoint>,
    pub spritesheets: HashMap<SpritesheetId, CharacterSpritesheet>,
    pub player_spritesheet: Option<SpritesheetId>,
    /// Dialog text entries: Text_Id → text string.
    pub dialog_texts: HashMap<String, String>,
    /// Face portrait entries: portrait ID → asset path.
    pub face_portraits: HashMap<String, String>,
    /// Character registry: all playable characters defined in this project.
    pub characters: CharacterRegistry,
    /// Whether character data has been modified since the last save.
    pub has_unsaved_character_changes: bool,
    /// Item registry: all items defined in this project.
    pub items: ItemRegistry,
    /// Whether item data has been modified since the last save.
    pub has_unsaved_item_changes: bool,
    /// Ability registry: all abilities defined in this project.
    pub abilities: AbilityRegistry,
    /// Whether ability data has been modified since the last save.
    pub has_unsaved_ability_changes: bool,
    /// Enemy registry: all enemies defined in this project.
    pub enemies: EnemyRegistry,
    /// Whether enemy data has been modified since the last save.
    pub has_unsaved_enemy_changes: bool,
    /// Shop registry: all shops defined in this project.
    pub shops: ShopRegistry,
    /// Whether shop data has been modified since the last save.
    pub has_unsaved_shop_changes: bool,
    /// Intro events: actions to execute when a new game starts.
    pub intro_events: Option<Vec<EventAction>>,
    /// Whether intro events data has been modified since the last save.
    pub has_unsaved_intro_events_changes: bool,
}

impl Project {
    // ── Accessor methods ──

    /// Returns the `MapId` of the currently active tab, if any.
    pub fn active_map_id(&self) -> Option<&MapId> {
        let idx = self.active_tab?;
        self.open_tabs.get(idx)
    }

    /// Returns a reference to the currently active map, if any.
    pub fn active_map(&self) -> Option<&MapData> {
        let id = self.active_map_id()?;
        self.maps.get(id)
    }

    /// Returns a mutable reference to the currently active map, if any.
    pub fn active_map_mut(&mut self) -> Option<&mut MapData> {
        let id = self.active_map_id()?.clone();
        self.maps.get_mut(&id)
    }

    // ── Map operations ──

    /// Adds a new map to the project, initializes its undo history,
    /// opens it in a tab, and returns the generated `MapId`.
    pub fn add_map(
        &mut self,
        name: impl Into<String>,
        width: u32,
        height: u32,
        tile_width: u32,
        tile_height: u32,
    ) -> Result<MapId, EditorError> {
        let map = MapData::new(name, width, height, tile_width, tile_height)?;
        let id: MapId = uuid::Uuid::new_v4().to_string();
        self.maps.insert(id.clone(), map);
        self.undo_histories
            .insert(id.clone(), UndoHistory::default());
        self.has_unsaved_changes.insert(id.clone(), false);
        self.open_map_tab(id.clone());
        Ok(id)
    }

    /// Removes a map from the project.
    /// Returns an error if it is the only remaining map.
    pub fn remove_map(&mut self, id: &MapId) -> Result<(), EditorError> {
        if self.maps.len() <= 1 {
            return Err(EditorError::ProjectValidationError(
                "cannot delete the last map".to_string(),
            ));
        }
        self.maps.remove(id);

        // Clear spawn point if it references the deleted map
        if let Some(ref sp) = self.spawn_point
            && sp.map_id == *id
        {
            self.spawn_point = None;
        }

        self.undo_histories.remove(id);
        self.has_unsaved_changes.remove(id);

        // Remove from open tabs (may appear at most once)
        if let Some(pos) = self.open_tabs.iter().position(|tab_id| tab_id == id) {
            self.open_tabs.remove(pos);
            // Adjust active_tab
            if self.open_tabs.is_empty() {
                self.active_tab = None;
            } else if let Some(active) = self.active_tab {
                if active >= self.open_tabs.len() {
                    self.active_tab = Some(self.open_tabs.len() - 1);
                } else if pos < active {
                    self.active_tab = Some(active - 1);
                } else if pos == active {
                    // Activate nearest: prefer same index (now the next tab), else last
                    self.active_tab = Some(active.min(self.open_tabs.len() - 1));
                }
            }
        }
        Ok(())
    }

    // ── Tileset operations ──

    /// Adds a tileset to the project and returns the generated `TilesetId`.
    pub fn add_tileset(
        &mut self,
        meta: TilesetMeta,
        texture: Handle<Image>,
        atlas_layout: Handle<TextureAtlasLayout>,
    ) -> TilesetId {
        let id: TilesetId = uuid::Uuid::new_v4().to_string();
        self.tilesets.insert(
            id.clone(),
            TilesetEntry {
                meta,
                texture,
                atlas_layout,
            },
        );
        id
    }

    /// Checks whether a tileset is compatible with a given map (matching tile dimensions).
    /// Returns Ok(()) if compatible, or Err with a descriptive message if not.
    pub fn check_tileset_compatibility(
        &self,
        tileset_id: &TilesetId,
        map_id: &MapId,
    ) -> Result<(), EditorError> {
        let map = self.maps.get(map_id).ok_or_else(|| {
            EditorError::ProjectValidationError(format!("map '{}' not found", map_id))
        })?;
        let tileset = self.tilesets.get(tileset_id).ok_or_else(|| {
            EditorError::ProjectValidationError(format!("tileset '{}' not found", tileset_id))
        })?;
        if tileset.meta.tile_width != map.tile_width || tileset.meta.tile_height != map.tile_height
        {
            return Err(EditorError::ProjectValidationError(format!(
                "tileset tile size ({}x{}) does not match map tile size ({}x{})",
                tileset.meta.tile_width, tileset.meta.tile_height, map.tile_width, map.tile_height
            )));
        }
        Ok(())
    }

    // ── Tab management ──

    /// Opens a map in the tab bar. If already open, just activates it.
    pub fn open_map_tab(&mut self, id: MapId) {
        if let Some(pos) = self.open_tabs.iter().position(|tab_id| tab_id == &id) {
            self.active_tab = Some(pos);
        } else {
            self.open_tabs.push(id);
            self.active_tab = Some(self.open_tabs.len() - 1);
        }
    }

    /// Closes the tab at the given index.
    /// Adjusts `active_tab` to the nearest remaining tab, or `None`.
    pub fn close_map_tab(&mut self, idx: usize) {
        if idx >= self.open_tabs.len() {
            return;
        }
        self.open_tabs.remove(idx);
        if self.open_tabs.is_empty() {
            self.active_tab = None;
        } else if let Some(active) = self.active_tab {
            if idx < active {
                self.active_tab = Some(active - 1);
            } else if idx == active {
                // Activate nearest: prefer same index (now next), clamp to last
                self.active_tab = Some(idx.min(self.open_tabs.len() - 1));
            }
            // If idx > active, active_tab stays the same
        }
    }

    /// Sets the active tab to the given index if it is valid.
    pub fn set_active_tab(&mut self, idx: usize) {
        if idx < self.open_tabs.len() {
            self.active_tab = Some(idx);
        }
    }
}
