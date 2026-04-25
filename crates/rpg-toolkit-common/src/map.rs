use serde::{Deserialize, Serialize};

use crate::error::CommonError;
use crate::spritesheet::NpcInstance;

/// Type alias for map identifiers (UUID v4 strings).
pub type MapId = String;

/// Type alias for tileset identifiers (UUID v4 strings).
pub type TilesetId = String;

/// Valid tile sizes in pixels.
const VALID_TILE_SIZES: [u32; 4] = [8, 16, 32, 64];

/// Serialization-compatible dialog text data for EventAction.
/// Mirrors rpg_toolkit_renderer::dialog::DialogText.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum DialogTextData {
    Inline(String),
    Id(String),
}

/// Serialization-compatible dialog position for EventAction.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum DialogPositionData {
    Top,
    Center,
    #[default]
    Bottom,
}

fn default_text_speed() -> f32 {
    30.0
}

fn default_movement_block() -> bool {
    true
}

/// Serialization-compatible dialog configuration for EventAction.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DialogConfigData {
    #[serde(default = "default_text_speed")]
    pub text_speed: f32,
    #[serde(default)]
    pub position: DialogPositionData,
    #[serde(default = "default_movement_block")]
    pub movement_block: bool,
}

impl Default for DialogConfigData {
    fn default() -> Self {
        Self {
            text_speed: 30.0,
            position: DialogPositionData::Bottom,
            movement_block: true,
        }
    }
}

/// A single action within an event trigger sequence.
/// Uses `#[serde(tag = "type")]` for clean, forward-compatible JSON.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum EventAction {
    JumpTo {
        target_map_id: MapId,
        target_x: u32,
        target_y: u32,
    },
    ShowDialog {
        text: DialogTextData,
        config: DialogConfigData,
    },
}

/// Per-tile attribute data: opacity flag and event trigger list.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct TileAttributes {
    pub opacity: bool,
    #[serde(default)]
    pub event_trigger: Vec<EventAction>,
}

/// A parallel grid of `TileAttributes` matching a layer's tile dimensions.
/// `cells[y][x]` — row-major, same as `Layer.tiles`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct TileAttributeLayer {
    pub cells: Vec<Vec<TileAttributes>>,
}

impl TileAttributeLayer {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            cells: vec![vec![TileAttributes::default(); width as usize]; height as usize],
        }
    }
}

/// A project-wide spawn point: one per project, always on layer 0.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpawnPoint {
    pub map_id: MapId,
    pub x: u32,
    pub y: u32,
}

/// A reference to a specific tile within a specific tileset.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TileRef {
    pub tileset_id: TilesetId,
    pub col: u32,
    pub row: u32,
}

/// A single layer of the map, containing a 2D grid of optional tile references.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Layer {
    pub name: String,
    pub visible: bool,
    /// Row-major grid: tiles[y][x]
    pub tiles: Vec<Vec<Option<TileRef>>>,
    /// Per-tile attributes grid, parallel to `tiles`.
    #[serde(default)]
    pub attributes: TileAttributeLayer,
}

/// The complete map data.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct MapData {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub tile_width: u32,
    pub tile_height: u32,
    pub layers: Vec<Layer>,
    pub active_layer_index: usize,
    #[serde(default)]
    pub npcs: Vec<NpcInstance>,
}

impl MapData {
    /// Creates a new map with the given name, dimensions, and tile size.
    /// Width and height must be in the range 1..=256.
    /// Tile width and height must be in {8, 16, 32, 64}.
    /// The map starts with a single "Ground" layer filled with empty tiles.
    pub fn new(
        name: impl Into<String>,
        width: u32,
        height: u32,
        tile_width: u32,
        tile_height: u32,
    ) -> Result<Self, CommonError> {
        if !(1..=256).contains(&width) || !(1..=256).contains(&height) {
            return Err(CommonError::InvalidDimensions);
        }

        if !VALID_TILE_SIZES.contains(&tile_width) || !VALID_TILE_SIZES.contains(&tile_height) {
            return Err(CommonError::InvalidTileSize);
        }

        let tiles = vec![vec![None; width as usize]; height as usize];
        let ground_layer = Layer {
            name: "Ground".to_string(),
            visible: true,
            tiles,
            attributes: TileAttributeLayer::new(width, height),
        };

        Ok(Self {
            name: name.into(),
            width,
            height,
            tile_width,
            tile_height,
            layers: vec![ground_layer],
            active_layer_index: 0,
            npcs: Vec::new(),
        })
    }

    /// Validates the map data after deserialization.
    pub fn validate(&self) -> Result<(), CommonError> {
        if !(1..=256).contains(&self.width) || !(1..=256).contains(&self.height) {
            return Err(CommonError::InvalidDimensions);
        }

        if self.layers.is_empty() {
            return Err(CommonError::ProjectValidationError(
                "map must have at least one layer".to_string(),
            ));
        }

        for (i, layer) in self.layers.iter().enumerate() {
            if layer.tiles.len() != self.height as usize {
                return Err(CommonError::ProjectValidationError(format!(
                    "layer {} has {} rows, expected {}",
                    i,
                    layer.tiles.len(),
                    self.height
                )));
            }
            for (y, row) in layer.tiles.iter().enumerate() {
                if row.len() != self.width as usize {
                    return Err(CommonError::ProjectValidationError(format!(
                        "layer {} row {} has {} columns, expected {}",
                        i,
                        y,
                        row.len(),
                        self.width
                    )));
                }
            }
        }

        if self.active_layer_index >= self.layers.len() {
            return Err(CommonError::ProjectValidationError(format!(
                "active_layer_index {} is out of bounds (layers count: {})",
                self.active_layer_index,
                self.layers.len()
            )));
        }

        Ok(())
    }
}
