//! Editor state resources and enums.
//!
//! This module holds the primary editor state resource (`EditorState`), tool and mode
//! enums, brush selection types, and the `AnyDialogOpen` flag used to gate canvas
//! interactions when modal dialogs are visible.

use bevy::prelude::*;
use std::path::PathBuf;

use rpg_toolkit_common::{CommonError, TileRef, TilesetId};

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
    Elevation,
    ElevationTransition,
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
