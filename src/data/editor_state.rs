use bevy::prelude::*;
use std::path::PathBuf;

use super::map::TileIndex;

#[derive(Debug, thiserror::Error)]
pub enum EditorError {
    #[error("Invalid map dimensions: width and height must be between 1 and 256")]
    InvalidDimensions,
    #[error("Unsupported image format. Supported: PNG, JPEG")]
    UnsupportedFormat,
    #[error("Failed to read image: {0}")]
    ImageReadError(String),
    #[error("Failed to parse project file: {0}")]
    ProjectParseError(String),
    #[error("Invalid project data: {0}")]
    ProjectValidationError(String),
}

/// Zoom level boundaries.
const MIN_ZOOM: f32 = 0.25;
const MAX_ZOOM: f32 = 8.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToolMode {
    Paint,
    Erase,
}

#[derive(Resource)]
pub struct EditorState {
    pub active_brush: Option<TileIndex>,
    pub tool_mode: ToolMode,
    pub zoom_level: f32, // 0.25..=8.0
    pub camera_offset: Vec2,
    pub has_unsaved_changes: bool,
    pub current_save_path: Option<PathBuf>,
}

impl Default for EditorState {
    fn default() -> Self {
        Self {
            active_brush: None,
            tool_mode: ToolMode::Paint,
            zoom_level: 1.0,
            camera_offset: Vec2::ZERO,
            has_unsaved_changes: false,
            current_save_path: None,
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
