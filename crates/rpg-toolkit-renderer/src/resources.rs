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

impl AnimationConfig {
    /// Returns the frame duration, clamped to a minimum of 0.01 seconds.
    pub fn clamped_frame_duration(&self) -> f32 {
        self.frame_duration.max(0.01)
    }
}

/// Determines how the game world is scaled on screen.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PixelScaleMode {
    /// Automatically compute the largest integer scale where the
    /// entire map fits in the window.
    ZoomToFit,
    /// Use a fixed integer scale (clamped to >= 1).
    Fixed(u32),
}

/// Resource controlling pixel scaling of the game world.
#[derive(Resource)]
pub struct PixelScaleConfig {
    /// The scaling mode: zoom-to-fit or fixed integer.
    pub mode: PixelScaleMode,
    /// The currently computed effective integer scale (always >= 1).
    /// Updated each frame by `apply_pixel_scale`.
    pub effective_scale: u32,
}

impl Default for PixelScaleConfig {
    fn default() -> Self {
        Self {
            mode: PixelScaleMode::ZoomToFit,
            effective_scale: 1,
        }
    }
}
