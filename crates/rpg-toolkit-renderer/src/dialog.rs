use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Vertical placement of the dialog box on screen.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum DialogPosition {
    Top,
    Center,
    #[default]
    Bottom,
}

/// Configuration for how a dialog box behaves.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DialogConfig {
    /// Characters revealed per second. 0 means instant reveal.
    #[serde(default = "default_text_speed")]
    pub text_speed: f32,
    /// Vertical placement on screen.
    #[serde(default)]
    pub position: DialogPosition,
    /// Whether to block player movement while dialog is active.
    #[serde(default = "default_movement_block")]
    pub movement_block: bool,
    /// When true, renders without background/border (floating text).
    #[serde(default)]
    pub attribute_dialog: bool,
    /// Optional face portrait image path (relative to project assets).
    #[serde(default)]
    pub face_portrait: Option<String>,
}

fn default_text_speed() -> f32 {
    30.0
}

fn default_movement_block() -> bool {
    true
}

impl Default for DialogConfig {
    fn default() -> Self {
        Self {
            text_speed: 30.0,
            position: DialogPosition::Bottom,
            movement_block: true,
            attribute_dialog: false,
            face_portrait: None,
        }
    }
}

/// The text content for a dialog event.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum DialogText {
    /// Inline text string.
    Inline(String),
    /// Reference to a text registry entry.
    Id(String),
}

/// A mapping from string IDs to dialog text strings.
/// Loadable from JSON, replaceable at runtime for localization.
#[derive(Resource, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct DialogTextRegistry {
    entries: HashMap<String, String>,
}

impl DialogTextRegistry {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    pub fn from_map(entries: HashMap<String, String>) -> Self {
        Self { entries }
    }

    pub fn insert(&mut self, id: impl Into<String>, text: impl Into<String>) {
        self.entries.insert(id.into(), text.into());
    }

    pub fn get(&self, id: &str) -> Option<&str> {
        self.entries.get(id).map(|s| s.as_str())
    }

    pub fn remove(&mut self, id: &str) -> Option<String> {
        self.entries.remove(id)
    }

    /// Deserialize from a JSON string containing a flat object.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        let entries: HashMap<String, String> = serde_json::from_str(json)?;
        Ok(Self { entries })
    }
}

/// Tracks the active dialog. Present only while a dialog is displayed.
#[derive(Resource)]
pub struct DialogState {
    /// The full text being displayed.
    pub full_text: String,
    /// Total number of characters in the text.
    pub total_chars: usize,
    /// Number of characters currently revealed.
    pub chars_revealed: usize,
    /// Whether all text has been fully revealed.
    pub fully_revealed: bool,
    /// Elapsed time since dialog was spawned (seconds).
    pub elapsed: f32,
    /// Characters per second (from DialogConfig).
    pub text_speed: f32,
    /// Whether player movement is blocked.
    pub movement_blocked: bool,
}

/// Marker for the root dialog box UI entity.
#[derive(Component)]
pub struct DialogBox;

/// Marker for the dialog text UI entity.
#[derive(Component)]
pub struct DialogTextNode;

/// Marker for the inner dialog panel (the bordered/backgrounded box).
#[derive(Component)]
pub struct DialogPanel;

/// Marker for the overflow indicator entity.
#[derive(Component)]
pub struct OverflowIndicator;

/// Marker for the face portrait image entity.
#[derive(Component)]
pub struct FacePortrait;

/// Convert common DialogTextData to renderer DialogText.
pub fn dialog_text_from_data(data: &rpg_toolkit_common::DialogTextData) -> DialogText {
    match data {
        rpg_toolkit_common::DialogTextData::Inline(s) => DialogText::Inline(s.clone()),
        rpg_toolkit_common::DialogTextData::Id(s) => DialogText::Id(s.clone()),
    }
}

/// Convert common DialogConfigData to renderer DialogConfig.
pub fn dialog_config_from_data(data: &rpg_toolkit_common::DialogConfigData) -> DialogConfig {
    DialogConfig {
        text_speed: data.text_speed,
        position: match data.position {
            rpg_toolkit_common::DialogPositionData::Top => DialogPosition::Top,
            rpg_toolkit_common::DialogPositionData::Center => DialogPosition::Center,
            rpg_toolkit_common::DialogPositionData::Bottom => DialogPosition::Bottom,
        },
        movement_block: data.movement_block,
        attribute_dialog: data.attribute_dialog,
        face_portrait: data.face_portrait.clone(),
    }
}

/// Computes the number of visible characters for the typewriter effect.
/// Returns `min(floor(elapsed * text_speed), total_chars)` for speed > 0,
/// or `total_chars` for speed <= 0.
pub fn compute_visible_chars(elapsed: f32, text_speed: f32, total_chars: usize) -> usize {
    if text_speed <= 0.0 {
        return total_chars;
    }
    let computed = (elapsed * text_speed).floor() as usize;
    computed.min(total_chars)
}
