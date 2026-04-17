use bevy::prelude::*;
use std::path::PathBuf;

use rpg_toolkit_common::{
    CommonError, EventAction, Layer, MapData, NpcInstance, SpawnPoint, TileAttributeLayer, TileRef,
    TilesetId,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Resource)]
pub enum EditorTool {
    #[default]
    Paint,
    Erase,
    Pan,
    FloodFill,
    StampBrush,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum EditorMode {
    #[default]
    Paint,
    Attribute,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AttributeTool {
    #[default]
    Opacity,
    EventTrigger,
    SpawnPoint,
    NpcPlacement,
}

#[derive(Clone, Debug)]
pub struct StampBrushSelection {
    pub tileset_id: TilesetId,
    pub top_left_col: u32,
    pub top_left_row: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Debug, Default)]
pub struct LineDragState {
    pub active: bool,
    pub start_tile: Option<(u32, u32)>,
}

#[derive(Debug, thiserror::Error)]
#[allow(dead_code)]
pub enum EditorError {
    #[error("Invalid map dimensions: width and height must be between 1 and 256")]
    InvalidDimensions,
    #[error("Invalid tile size: must be one of 8, 16, 32, 64")]
    InvalidTileSize,
    #[error("Unsupported image format. Supported: PNG, JPEG")]
    UnsupportedFormat,
    #[error("Failed to parse project file: {0}")]
    ProjectParseError(String),
    #[error("Invalid project data: {0}")]
    ProjectValidationError(String),
    #[error(transparent)]
    Common(#[from] CommonError),
}

/// Resource that is `true` whenever any modal dialog window is open.
///
/// This replaces the overly-broad `ctx.wants_pointer_input()` check that
/// was blocking canvas interactions whenever *any* egui widget (including
/// side panels and toolbar) was hovered.  Canvas systems should only be
/// blocked when an actual dialog is in front of the canvas.
#[derive(Resource, Default)]
pub struct AnyDialogOpen(pub bool);

/// Zoom level boundaries.
const MIN_ZOOM: f32 = 0.25;
const MAX_ZOOM: f32 = 8.0;

#[derive(Resource)]
pub struct EditorState {
    pub active_brush: Option<TileRef>,
    pub active_tileset_tab: Option<TilesetId>,
    pub zoom_level: f32, // 0.25..=8.0
    pub camera_offset: Vec2,
    pub current_save_path: Option<PathBuf>,
    pub stamp_brush: Option<StampBrushSelection>,
    pub line_drag: LineDragState,
    pub editor_mode: EditorMode,
    pub attribute_tool: AttributeTool,
    pub previous_tool: Option<EditorTool>,
}

impl Default for EditorState {
    fn default() -> Self {
        Self {
            active_brush: None,
            active_tileset_tab: None,
            zoom_level: 1.0,
            camera_offset: Vec2::ZERO,
            current_save_path: None,
            stamp_brush: None,
            line_drag: LineDragState::default(),
            editor_mode: EditorMode::default(),
            attribute_tool: AttributeTool::default(),
            previous_tool: None,
        }
    }
}

impl EditorState {
    /// Clamps the current zoom level to the valid range [0.25, 8.0].
    pub fn clamp_zoom(&mut self) {
        self.zoom_level = self.zoom_level.clamp(MIN_ZOOM, MAX_ZOOM);
    }

    /// Sets the zoom level, clamping it to [0.25, 8.0].
    pub fn set_zoom(&mut self, zoom: f32) {
        self.zoom_level = zoom;
        self.clamp_zoom();
    }
}

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
        }
    }
}

/// Undo/redo history resource. Maintains two stacks capped at `max_history`.
#[derive(Resource)]
pub struct UndoHistory {
    pub undo_stack: Vec<EditCommand>,
    pub redo_stack: Vec<EditCommand>,
    pub max_history: usize,
}

impl Default for UndoHistory {
    fn default() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_history: 50,
        }
    }
}

impl UndoHistory {
    /// Pushes a command onto the undo stack, clears the redo stack,
    /// and enforces the maximum history size.
    pub fn push_command(&mut self, cmd: EditCommand) {
        self.redo_stack.clear();
        self.undo_stack.push(cmd);
        if self.undo_stack.len() > self.max_history {
            self.undo_stack.remove(0);
        }
    }

    /// Undoes the most recent command. Returns `true` if an undo was performed.
    pub fn undo(&mut self, map: &mut MapData) -> bool {
        if let Some(cmd) = self.undo_stack.pop() {
            cmd.apply_inverse(map);
            self.redo_stack.push(cmd);
            true
        } else {
            false
        }
    }

    /// Redoes the most recently undone command. Returns `true` if a redo was performed.
    pub fn redo(&mut self, map: &mut MapData) -> bool {
        if let Some(cmd) = self.redo_stack.pop() {
            cmd.apply(map);
            self.undo_stack.push(cmd);
            true
        } else {
            false
        }
    }
}
