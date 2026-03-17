use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::editor_state::{EditCommand, EditCommandKind, EditorError};

/// A coordinate identifying a tile graphic within a tileset image.
///
/// Currently assumes a single tileset per project. To support multiple tilesets
/// per map, this struct should evolve into a `TileRef { tileset_id, col, row }`
/// so each cell can reference tiles from different tilesets. That change would
/// also require a tileset registry, multi-tileset palette UI, and updated
/// serialization. Tracked for a future iteration.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TileIndex {
    pub col: u32,
    pub row: u32,
}

/// A single layer of the map, containing a 2D grid of optional tile references.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Layer {
    pub name: String,
    pub visible: bool,
    /// Row-major grid: tiles[y][x]
    pub tiles: Vec<Vec<Option<TileIndex>>>,
}

/// The complete map data.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Resource)]
pub struct MapData {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub layers: Vec<Layer>,
    pub active_layer_index: usize,
}

impl MapData {
    /// Creates a new map with the given name and dimensions.
    /// Width and height must be in the range 1..=256.
    /// The map starts with a single "Ground" layer filled with empty tiles.
    pub fn new(name: impl Into<String>, width: u32, height: u32) -> Result<Self, EditorError> {
        if !(1..=256).contains(&width) || !(1..=256).contains(&height) {
            return Err(EditorError::InvalidDimensions);
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
        tile_index: TileIndex,
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

        let old_tile = *cell;
        *cell = Some(tile_index);

        Ok(EditCommand {
            kind: EditCommandKind::PlaceTile {
                layer_index,
                x,
                y,
                old_tile,
                new_tile: tile_index,
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

        let old_tile = *cell;
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
}
