use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::editor_state::EditorError;

/// Valid tile sizes in pixels.
const VALID_TILE_SIZES: [u32; 4] = [8, 16, 32, 64];

/// Metadata about a loaded tileset.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TilesetMeta {
    pub file_path: String,
    pub tile_width: u32,  // 8, 16, 32, or 64
    pub tile_height: u32, // 8, 16, 32, or 64
    pub columns: u32,
    pub rows: u32,
}

impl TilesetMeta {
    /// Computes tileset grid dimensions from image and tile sizes.
    ///
    /// Validates that `tile_w` and `tile_h` are each in {8, 16, 32, 64},
    /// and that the resulting columns and rows are both at least 1.
    pub fn from_image_dimensions(
        img_w: u32,
        img_h: u32,
        tile_w: u32,
        tile_h: u32,
    ) -> Result<Self, EditorError> {
        if !VALID_TILE_SIZES.contains(&tile_w) || !VALID_TILE_SIZES.contains(&tile_h) {
            return Err(EditorError::UnsupportedFormat);
        }

        let columns = img_w / tile_w;
        let rows = img_h / tile_h;

        if columns == 0 || rows == 0 {
            return Err(EditorError::UnsupportedFormat);
        }

        Ok(Self {
            file_path: String::new(),
            tile_width: tile_w,
            tile_height: tile_h,
            columns,
            rows,
        })
    }
}

/// Runtime tileset data (not serialized — the texture handle is runtime-only).
pub struct TilesetData {
    pub meta: TilesetMeta,
    pub texture: Handle<Image>,
    /// Pre-computed atlas layout for each tile
    pub atlas_layout: Handle<TextureAtlasLayout>,
}
