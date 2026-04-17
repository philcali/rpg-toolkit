use bevy::prelude::*;
use rpg_toolkit_common::{MapId, ProjectFile, SpritesheetId, TilesetId};
use std::collections::HashMap;

/// Input resource: consumers insert this before adding the plugin.
/// Contains the deserialized project data and tileset texture handles.
#[derive(Resource)]
pub struct RendererProjectData {
    pub project_file: ProjectFile,
    pub tileset_textures: HashMap<TilesetId, Handle<Image>>,
    pub tileset_atlas_layouts: HashMap<TilesetId, Handle<TextureAtlasLayout>>,
    pub spritesheet_textures: HashMap<SpritesheetId, Handle<Image>>,
    pub spritesheet_atlas_layouts: HashMap<SpritesheetId, Handle<TextureAtlasLayout>>,
}

/// Runtime state managed by the plugin.
#[derive(Resource, Default)]
pub struct RendererState {
    pub active_map_id: Option<MapId>,
    /// Set to `Some(map_id)` when a map transition is requested.
    pub pending_map_change: Option<MapId>,
    /// Target coordinates for the pending map change (from JumpTo action).
    pub pending_target_coords: Option<(u32, u32)>,
}

/// Configuration for player movement animation.
#[derive(Resource)]
pub struct MovementConfig {
    /// Duration of tile-to-tile animation in seconds.
    pub move_duration: f32,
}

impl Default for MovementConfig {
    fn default() -> Self {
        Self {
            move_duration: 0.15,
        }
    }
}

/// The player's visual representation.
#[derive(Resource)]
pub struct PlayerVisual {
    pub color: Color,
}

impl Default for PlayerVisual {
    fn default() -> Self {
        Self {
            color: Color::srgb(0.2, 0.6, 1.0),
        }
    }
}

/// Configuration for sprite walk animation timing.
#[derive(Resource)]
pub struct AnimationConfig {
    /// Duration of each animation frame in seconds.
    pub frame_duration: f32,
}

impl Default for AnimationConfig {
    fn default() -> Self {
        Self {
            frame_duration: 0.15,
        }
    }
}
