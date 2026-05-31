//! Reversible edit commands for undo/redo support.
//!
//! This module defines `EditCommand` and `EditCommandKind`, which represent all
//! undoable editing operations in the editor. Each command knows how to apply
//! itself forward (do) and backward (undo) against a `MapData`.

use rpg_toolkit_common::{
    EventAction, Layer, MapData, NpcInstance, SpawnPoint, TileAttributeLayer, TileRef,
};

/// A reversible editing command for undo/redo support.
#[derive(Clone, Debug, bevy::prelude::Message)]
pub struct EditCommand {
    pub kind: EditCommandKind,
}

#[derive(Clone, Debug)]
pub enum EditCommandKind {
    PlaceTile {
        layer_index: usize,
        x: u32,
        y: u32,
        old_tile: Option<TileRef>,
        new_tile: TileRef,
    },
    EraseTile {
        layer_index: usize,
        x: u32,
        y: u32,
        old_tile: Option<TileRef>,
    },
    AddLayer {
        layer_index: usize,
        name: String,
    },
    DeleteLayer {
        layer_index: usize,
        layer_data: Layer,
    },
    SetOpacity {
        layer_index: usize,
        x: u32,
        y: u32,
        old_value: bool,
        new_value: bool,
    },
    SetEventTrigger {
        layer_index: usize,
        x: u32,
        y: u32,
        old_trigger: Vec<EventAction>,
        new_trigger: Vec<EventAction>,
    },
    SetSpawnPoint {
        old_spawn: Option<SpawnPoint>,
        new_spawn: Option<SpawnPoint>,
    },
    PlaceNpc {
        npc_index: Option<usize>, // None = new (appended), Some = edit (replaced at index)
        old_npc: Option<NpcInstance>, // None for new placement, Some for edit
        new_npc: NpcInstance,
    },
    RemoveNpc {
        npc_index: usize,
        removed_npc: NpcInstance,
    },
    SetElevation {
        layer_index: usize,
        x: u32,
        y: u32,
        old_value: u32,
        new_value: u32,
    },
    SetTargetElevation {
        layer_index: usize,
        x: u32,
        y: u32,
        old_value: Option<u32>,
        new_value: Option<u32>,
    },
    InsertDialogText {
        text_id: String,
        text: String,
    },
    UpdateDialogText {
        text_id: String,
        old_text: String,
        new_text: String,
    },
    RemoveDialogText {
        text_id: String,
        old_text: String,
    },
    InsertFacePortrait {
        id: String,
        path: String,
    },
    UpdateFacePortrait {
        id: String,
        old_path: String,
        new_path: String,
    },
    RemoveFacePortrait {
        id: String,
        path: String,
    },
}

impl EditCommand {
    /// Applies this command to the map (forward direction).
    pub fn apply(&self, map: &mut MapData) {
        match &self.kind {
            EditCommandKind::PlaceTile {
                layer_index,
                x,
                y,
                new_tile,
                ..
            } => {
                if let Some(layer) = map.layers.get_mut(*layer_index)
                    && let Some(row) = layer.tiles.get_mut(*y as usize)
                    && let Some(cell) = row.get_mut(*x as usize)
                {
                    *cell = Some(new_tile.clone());
                }
            }
            EditCommandKind::EraseTile {
                layer_index, x, y, ..
            } => {
                if let Some(layer) = map.layers.get_mut(*layer_index)
                    && let Some(row) = layer.tiles.get_mut(*y as usize)
                    && let Some(cell) = row.get_mut(*x as usize)
                {
                    *cell = None;
                }
            }
            EditCommandKind::AddLayer { layer_index, name } => {
                let tiles = vec![vec![None; map.width as usize]; map.height as usize];
                let layer = Layer {
                    name: name.clone(),
                    visible: true,
                    tiles,
                    attributes: TileAttributeLayer::new(map.width, map.height),
                };
                let idx = (*layer_index).min(map.layers.len());
                map.layers.insert(idx, layer);
                map.active_layer_index = idx;
            }
            EditCommandKind::DeleteLayer { layer_index, .. } => {
                if map.layers.len() > 1 && *layer_index < map.layers.len() {
                    map.layers.remove(*layer_index);
                    if map.active_layer_index >= map.layers.len() {
                        map.active_layer_index = map.layers.len() - 1;
                    }
                }
            }
            EditCommandKind::SetOpacity {
                layer_index,
                x,
                y,
                new_value,
                ..
            } => {
                if let Some(layer) = map.layers.get_mut(*layer_index)
                    && let Some(row) = layer.attributes.cells.get_mut(*y as usize)
                    && let Some(cell) = row.get_mut(*x as usize)
                {
                    cell.opacity = *new_value;
                }
            }
            EditCommandKind::SetEventTrigger {
                layer_index,
                x,
                y,
                new_trigger,
                ..
            } => {
                if let Some(layer) = map.layers.get_mut(*layer_index)
                    && let Some(row) = layer.attributes.cells.get_mut(*y as usize)
                    && let Some(cell) = row.get_mut(*x as usize)
                {
                    cell.event_trigger = new_trigger.clone();
                }
            }
            EditCommandKind::SetSpawnPoint { .. } => {
                // No-op on MapData; handled at Project level by undo_redo plugin
            }
            EditCommandKind::PlaceNpc {
                npc_index, new_npc, ..
            } => {
                if let Some(idx) = npc_index {
                    if *idx < map.npcs.len() {
                        map.npcs[*idx] = new_npc.clone();
                    }
                } else {
                    map.npcs.push(new_npc.clone());
                }
            }
            EditCommandKind::RemoveNpc { npc_index, .. } => {
                if *npc_index < map.npcs.len() {
                    map.npcs.remove(*npc_index);
                }
            }
            EditCommandKind::SetElevation {
                layer_index,
                x,
                y,
                new_value,
                ..
            } => {
                if let Some(layer) = map.layers.get_mut(*layer_index)
                    && let Some(row) = layer.attributes.cells.get_mut(*y as usize)
                    && let Some(cell) = row.get_mut(*x as usize)
                {
                    cell.elevation = *new_value;
                }
            }
            EditCommandKind::SetTargetElevation {
                layer_index,
                x,
                y,
                new_value,
                ..
            } => {
                if let Some(layer) = map.layers.get_mut(*layer_index)
                    && let Some(row) = layer.attributes.cells.get_mut(*y as usize)
                    && let Some(cell) = row.get_mut(*x as usize)
                {
                    cell.target_elevation = *new_value;
                }
            }
            EditCommandKind::InsertDialogText { .. }
            | EditCommandKind::UpdateDialogText { .. }
            | EditCommandKind::RemoveDialogText { .. }
            | EditCommandKind::InsertFacePortrait { .. }
            | EditCommandKind::UpdateFacePortrait { .. }
            | EditCommandKind::RemoveFacePortrait { .. } => {
                // No-op on MapData; handled at Project level by undo_redo plugin
            }
        }
    }

    /// Applies the inverse of this command (undo direction).
    pub fn apply_inverse(&self, map: &mut MapData) {
        match &self.kind {
            EditCommandKind::PlaceTile {
                layer_index,
                x,
                y,
                old_tile,
                ..
            } => {
                if let Some(layer) = map.layers.get_mut(*layer_index)
                    && let Some(row) = layer.tiles.get_mut(*y as usize)
                    && let Some(cell) = row.get_mut(*x as usize)
                {
                    *cell = old_tile.clone();
                }
            }
            EditCommandKind::EraseTile {
                layer_index,
                x,
                y,
                old_tile,
            } => {
                if let Some(layer) = map.layers.get_mut(*layer_index)
                    && let Some(row) = layer.tiles.get_mut(*y as usize)
                    && let Some(cell) = row.get_mut(*x as usize)
                {
                    *cell = old_tile.clone();
                }
            }
            EditCommandKind::AddLayer { layer_index, .. } => {
                // Inverse of add = delete
                if *layer_index < map.layers.len() {
                    map.layers.remove(*layer_index);
                    if map.active_layer_index >= map.layers.len() {
                        map.active_layer_index = map.layers.len().saturating_sub(1);
                    }
                }
            }
            EditCommandKind::DeleteLayer {
                layer_index,
                layer_data,
            } => {
                // Inverse of delete = re-insert
                let idx = (*layer_index).min(map.layers.len());
                map.layers.insert(idx, layer_data.clone());
                map.active_layer_index = idx;
            }
            EditCommandKind::SetOpacity {
                layer_index,
                x,
                y,
                old_value,
                ..
            } => {
                if let Some(layer) = map.layers.get_mut(*layer_index)
                    && let Some(row) = layer.attributes.cells.get_mut(*y as usize)
                    && let Some(cell) = row.get_mut(*x as usize)
                {
                    cell.opacity = *old_value;
                }
            }
            EditCommandKind::SetEventTrigger {
                layer_index,
                x,
                y,
                old_trigger,
                ..
            } => {
                if let Some(layer) = map.layers.get_mut(*layer_index)
                    && let Some(row) = layer.attributes.cells.get_mut(*y as usize)
                    && let Some(cell) = row.get_mut(*x as usize)
                {
                    cell.event_trigger = old_trigger.clone();
                }
            }
            EditCommandKind::SetSpawnPoint { .. } => {
                // No-op on MapData; handled at Project level by undo_redo plugin
            }
            EditCommandKind::PlaceNpc {
                npc_index, old_npc, ..
            } => {
                if let Some(idx) = npc_index {
                    if let Some(old) = old_npc
                        && *idx < map.npcs.len()
                    {
                        map.npcs[*idx] = old.clone();
                    }
                } else {
                    // Was appended, so pop the last element
                    map.npcs.pop();
                }
            }
            EditCommandKind::RemoveNpc {
                npc_index,
                removed_npc,
            } => {
                map.npcs.insert(*npc_index, removed_npc.clone());
            }
            EditCommandKind::SetElevation {
                layer_index,
                x,
                y,
                old_value,
                ..
            } => {
                if let Some(layer) = map.layers.get_mut(*layer_index)
                    && let Some(row) = layer.attributes.cells.get_mut(*y as usize)
                    && let Some(cell) = row.get_mut(*x as usize)
                {
                    cell.elevation = *old_value;
                }
            }
            EditCommandKind::SetTargetElevation {
                layer_index,
                x,
                y,
                old_value,
                ..
            } => {
                if let Some(layer) = map.layers.get_mut(*layer_index)
                    && let Some(row) = layer.attributes.cells.get_mut(*y as usize)
                    && let Some(cell) = row.get_mut(*x as usize)
                {
                    cell.target_elevation = *old_value;
                }
            }
            EditCommandKind::InsertDialogText { .. }
            | EditCommandKind::UpdateDialogText { .. }
            | EditCommandKind::RemoveDialogText { .. }
            | EditCommandKind::InsertFacePortrait { .. }
            | EditCommandKind::UpdateFacePortrait { .. }
            | EditCommandKind::RemoveFacePortrait { .. } => {
                // No-op on MapData; handled at Project level by undo_redo plugin
            }
        }
    }
}
