use bevy::prelude::*;

// Re-export TilesetMeta from common so existing `use crate::data::tileset::TilesetMeta` paths work.
pub use rpg_toolkit_common::TilesetMeta;

/// A tileset entry stored inside the `Project` resource.
/// Replaces the singleton `TilesetData` resource for multi-tileset support.
pub struct TilesetEntry {
    pub meta: TilesetMeta,
    pub texture: Handle<Image>,
    pub atlas_layout: Handle<TextureAtlasLayout>,
}
