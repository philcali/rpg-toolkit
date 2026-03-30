use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::editor_state::{EditCommand, EditCommandKind, EditorError};

/// Type alias for map identifiers (UUID v4 strings).
pub type MapId = String;

/// Type alias for tileset identifiers (UUID v4 strings).
pub type TilesetId = String;

/// Valid tile sizes in pixels.
const VALID_TILE_SIZES: [u32; 4] = [8, 16, 32, 64];

/// A reference to a specific tile within a specific tileset.
///
/// Each placed tile cell stores a `TileRef` so that maps can mix tiles
/// from different tilesets without ambiguity.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TileRef {
    pub tileset_id: TilesetId,
    pub col: u32,
    pub row: u32,
}

/// A single layer of the map, containing a 2D grid of optional tile references.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Layer {
    pub name: String,
    pub visible: bool,
    /// Row-major grid: tiles[y][x]
    pub tiles: Vec<Vec<Option<TileRef>>>,
}

/// The complete map data.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct MapData {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub tile_width: u32,
    pub tile_height: u32,
    pub layers: Vec<Layer>,
    pub active_layer_index: usize,
}

impl MapData {
    /// Creates a new map with the given name, dimensions, and tile size.
    /// Width and height must be in the range 1..=256.
    /// Tile width and height must be in {8, 16, 32, 64}.
    /// The map starts with a single "Ground" layer filled with empty tiles.
    pub fn new(
        name: impl Into<String>,
        width: u32,
        height: u32,
        tile_width: u32,
        tile_height: u32,
    ) -> Result<Self, EditorError> {
        if !(1..=256).contains(&width) || !(1..=256).contains(&height) {
            return Err(EditorError::InvalidDimensions);
        }

        if !VALID_TILE_SIZES.contains(&tile_width) || !VALID_TILE_SIZES.contains(&tile_height) {
            return Err(EditorError::InvalidTileSize);
        }

        let tiles = vec![vec![None; width as usize]; height as usize];
        let ground_layer = Layer {
            name: "Ground".to_string(),
            visible: true,
            tiles,
        };

        Ok(Self {
            name: name.into(),
            width,
            height,
            tile_width,
            tile_height,
            layers: vec![ground_layer],
            active_layer_index: 0,
        })
    }

    /// Validates the map data after deserialization.
    pub fn validate(&self) -> Result<(), EditorError> {
        if !(1..=256).contains(&self.width) || !(1..=256).contains(&self.height) {
            return Err(EditorError::InvalidDimensions);
        }

        if self.layers.is_empty() {
            return Err(EditorError::ProjectValidationError(
                "map must have at least one layer".to_string(),
            ));
        }

        for (i, layer) in self.layers.iter().enumerate() {
            if layer.tiles.len() != self.height as usize {
                return Err(EditorError::ProjectValidationError(format!(
                    "layer {} has {} rows, expected {}",
                    i,
                    layer.tiles.len(),
                    self.height
                )));
            }
            for (y, row) in layer.tiles.iter().enumerate() {
                if row.len() != self.width as usize {
                    return Err(EditorError::ProjectValidationError(format!(
                        "layer {} row {} has {} columns, expected {}",
                        i,
                        y,
                        row.len(),
                        self.width
                    )));
                }
            }
        }

        if self.active_layer_index >= self.layers.len() {
            return Err(EditorError::ProjectValidationError(format!(
                "active_layer_index {} is out of bounds (layers count: {})",
                self.active_layer_index,
                self.layers.len()
            )));
        }

        Ok(())
    }

    /// Places a tile at the given position on the specified layer.
    /// Returns an `EditCommand` capturing the old value for undo.
    pub fn place_tile(
        &mut self,
        layer_index: usize,
        x: u32,
        y: u32,
        tile_ref: TileRef,
    ) -> Result<EditCommand, EditorError> {
        let layer = self
            .layers
            .get_mut(layer_index)
            .ok_or(EditorError::ProjectValidationError(format!(
                "layer index {} out of bounds",
                layer_index
            )))?;

        let row = layer
            .tiles
            .get_mut(y as usize)
            .ok_or(EditorError::ProjectValidationError(format!(
                "y={} out of bounds (height={})",
                y, self.height
            )))?;

        let cell = row
            .get_mut(x as usize)
            .ok_or(EditorError::ProjectValidationError(format!(
                "x={} out of bounds (width={})",
                x, self.width
            )))?;

        let old_tile = cell.clone();
        *cell = Some(tile_ref.clone());

        Ok(EditCommand {
            kind: EditCommandKind::PlaceTile {
                layer_index,
                x,
                y,
                old_tile,
                new_tile: tile_ref,
            },
        })
    }

    /// Erases the tile at the given position on the specified layer.
    /// Returns an `EditCommand` capturing the old value for undo.
    pub fn erase_tile(
        &mut self,
        layer_index: usize,
        x: u32,
        y: u32,
    ) -> Result<EditCommand, EditorError> {
        let layer = self
            .layers
            .get_mut(layer_index)
            .ok_or(EditorError::ProjectValidationError(format!(
                "layer index {} out of bounds",
                layer_index
            )))?;

        let row = layer
            .tiles
            .get_mut(y as usize)
            .ok_or(EditorError::ProjectValidationError(format!(
                "y={} out of bounds (height={})",
                y, self.height
            )))?;

        let cell = row
            .get_mut(x as usize)
            .ok_or(EditorError::ProjectValidationError(format!(
                "x={} out of bounds (width={})",
                x, self.width
            )))?;

        let old_tile = cell.clone();
        *cell = None;

        Ok(EditCommand {
            kind: EditCommandKind::EraseTile {
                layer_index,
                x,
                y,
                old_tile,
            },
        })
    }

    /// Adds a new empty layer above the active layer.
    /// Returns an `EditCommand` for undo support.
    pub fn add_layer(&mut self, name: impl Into<String>) -> EditCommand {
        let insert_index = (self.active_layer_index + 1).min(self.layers.len());
        let name = name.into();
        let tiles = vec![vec![None; self.width as usize]; self.height as usize];
        let layer = Layer {
            name: name.clone(),
            visible: true,
            tiles,
        };
        self.layers.insert(insert_index, layer);
        self.active_layer_index = insert_index;

        EditCommand {
            kind: EditCommandKind::AddLayer {
                layer_index: insert_index,
                name,
            },
        }
    }

    /// Deletes the layer at the given index.
    /// Returns `Err` if it's the last remaining layer.
    /// Returns an `EditCommand` containing the removed layer data for undo.
    pub fn delete_layer(&mut self, index: usize) -> Result<EditCommand, EditorError> {
        if self.layers.len() <= 1 {
            return Err(EditorError::ProjectValidationError(
                "cannot delete the last layer".to_string(),
            ));
        }
        if index >= self.layers.len() {
            return Err(EditorError::ProjectValidationError(format!(
                "layer index {} out of bounds (count: {})",
                index,
                self.layers.len()
            )));
        }

        let removed = self.layers.remove(index);

        // Adjust active layer index
        if self.active_layer_index >= self.layers.len() {
            self.active_layer_index = self.layers.len() - 1;
        }

        Ok(EditCommand {
            kind: EditCommandKind::DeleteLayer {
                layer_index: index,
                layer_data: removed,
            },
        })
    }

    /// Toggles the visibility of the layer at the given index.
    pub fn toggle_layer_visibility(&mut self, index: usize) {
        if let Some(layer) = self.layers.get_mut(index) {
            layer.visible = !layer.visible;
        }
    }

    /// Sets the active layer index, validating that it's in bounds.
    pub fn set_active_layer(&mut self, index: usize) -> Result<(), EditorError> {
        if index >= self.layers.len() {
            return Err(EditorError::ProjectValidationError(format!(
                "layer index {} out of bounds (count: {})",
                index,
                self.layers.len()
            )));
        }
        self.active_layer_index = index;
        Ok(())
    }
}
