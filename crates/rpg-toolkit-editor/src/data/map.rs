// Re-export all common types so existing `use crate::data::map::X` paths continue to work.
pub use rpg_toolkit_common::{
    EventAction, Layer, MapData, MapId, SpawnPoint, TileAttributeLayer, TileRef,
};

use super::commands::{EditCommand, EditCommandKind};
use super::state::EditorError;

/// Editor-specific extension methods for `MapData`.
///
/// These methods depend on `EditCommand` / `EditorError` which live in the editor crate,
/// so they cannot be part of the common crate.
pub trait MapDataEditorExt {
    fn place_tile(
        &mut self,
        layer_index: usize,
        x: u32,
        y: u32,
        tile_ref: TileRef,
    ) -> Result<EditCommand, EditorError>;

    fn erase_tile(
        &mut self,
        layer_index: usize,
        x: u32,
        y: u32,
    ) -> Result<EditCommand, EditorError>;

    fn add_layer(&mut self, name: impl Into<String>) -> EditCommand;

    fn delete_layer(&mut self, index: usize) -> Result<EditCommand, EditorError>;

    fn toggle_layer_visibility(&mut self, index: usize);

    fn set_active_layer(&mut self, index: usize) -> Result<(), EditorError>;
}

impl MapDataEditorExt for MapData {
    /// Places a tile at the given position on the specified layer.
    /// Returns an `EditCommand` capturing the old value for undo.
    fn place_tile(
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
    fn erase_tile(
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
    fn add_layer(&mut self, name: impl Into<String>) -> EditCommand {
        let insert_index = (self.active_layer_index + 1).min(self.layers.len());
        let name = name.into();
        let tiles = vec![vec![None; self.width as usize]; self.height as usize];
        let layer = Layer {
            name: name.clone(),
            visible: true,
            tiles,
            attributes: TileAttributeLayer::new(self.width, self.height),
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
    fn delete_layer(&mut self, index: usize) -> Result<EditCommand, EditorError> {
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
    fn toggle_layer_visibility(&mut self, index: usize) {
        if let Some(layer) = self.layers.get_mut(index) {
            layer.visible = !layer.visible;
        }
    }

    /// Sets the active layer index, validating that it's in bounds.
    fn set_active_layer(&mut self, index: usize) -> Result<(), EditorError> {
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
