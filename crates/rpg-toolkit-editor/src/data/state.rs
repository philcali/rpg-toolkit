//! Editor state resources and enums.
//!
//! This module holds the primary editor state resource (`EditorState`), tool and mode
//! enums, brush selection types, and the `AnyDialogOpen` flag used to gate canvas
//! interactions when modal dialogs are visible.

use bevy::prelude::*;
use std::path::PathBuf;

use rpg_toolkit_common::{AnimationFrame, CommonError, TileRef, TilesetId};

/// System sets for ordering egui panel rendering.
/// Panels must render in order: Shell (top bar) → Panels (side panels) → Overlay (toolbar, tooltips).
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum EditorUiSet {
    /// Top-level shell: menu bar, tab bar, central panel.
    Shell,
    /// Side panels: layer panel, tile palette, character panel.
    Panels,
    /// Floating overlays: toolbar, coordinate tooltip.
    Overlay,
}

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

/// Top-level application editor mode. Controls which set of plugins
/// renders in the viewport.
///
/// This is separate from the existing `EditorMode` enum which toggles
/// Paint/Attribute within the map editor.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Resource)]
pub enum AppEditorMode {
    #[default]
    Map,
    Character,
    Item,
    Ability,
    Enemy,
}

/// Resource that is `true` whenever any modal dialog window is open.
///
/// This replaces the overly-broad `ctx.wants_pointer_input()` check that
/// was blocking canvas interactions whenever *any* egui widget (including
/// side panels and toolbar) was hovered.  Canvas systems should only be
/// blocked when an actual dialog is in front of the canvas.
#[derive(Resource, Default)]
pub struct AnyDialogOpen(pub bool);

/// State for the animation editor UI mode.
///
/// Tracks whether the animation editor panel is active, the in-progress
/// frame sequence, and the frame duration setting.
#[derive(Resource)]
pub struct AnimationEditorState {
    pub active: bool,
    pub frames: Vec<AnimationFrame>,
    pub frame_duration_ms: u32,
    /// Inline error message shown when validation fails on confirm.
    pub error_message: Option<String>,
}

impl Default for AnimationEditorState {
    fn default() -> Self {
        Self {
            active: false,
            frames: Vec::new(),
            frame_duration_ms: 200,
            error_message: None,
        }
    }
}

/// Zoom level boundaries.
const MIN_ZOOM: f32 = 0.25;
const MAX_ZOOM: f32 = 8.0;

/// Palette tile scale boundaries.
const MIN_PALETTE_SCALE: f32 = 16.0;
const MAX_PALETTE_SCALE: f32 = 128.0;

/// Clamps a palette tile scale value to the valid range [16.0, 128.0].
pub fn clamp_palette_scale(scale: f32) -> f32 {
    scale.clamp(MIN_PALETTE_SCALE, MAX_PALETTE_SCALE)
}

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
    /// If the project was loaded from a ZIP file, this is the original path.
    pub original_zip_path: Option<std::path::PathBuf>,
    /// Display tile size for the palette grid (pixels). Clamped to [16, 128].
    pub palette_tile_scale: f32,
    /// Search buffer for the tileset searchable combobox.
    pub tileset_search_buffer: String,
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
            original_zip_path: None,
            palette_tile_scale: 24.0,
            tileset_search_buffer: String::new(),
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
