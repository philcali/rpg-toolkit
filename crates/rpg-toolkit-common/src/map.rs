use serde::{Deserialize, Deserializer, Serialize};

use crate::ability::AbilityId;
use crate::character::CharacterId;
use crate::condition::{BranchCondition, ConditionalTrigger};
use crate::error::CommonError;
use crate::item::ItemId;
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
    #[serde(default)]
    pub attribute_dialog: bool,
    #[serde(default)]
    pub face_portrait: Option<String>,
}

impl Default for DialogConfigData {
    fn default() -> Self {
        Self {
            text_speed: 30.0,
            position: DialogPositionData::Bottom,
            movement_block: true,
            attribute_dialog: false,
            face_portrait: None,
        }
    }
}

/// A single choice in a selection prompt.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(try_from = "RawChoiceData")]
pub struct ChoiceData {
    /// Display label for this choice (inline text or registry ID).
    pub label: DialogTextData,
    /// Actions to execute when this choice is selected.
    pub actions: Vec<EventAction>,
}

/// Raw helper struct for deserializing `ChoiceData` with validation.
#[derive(Deserialize)]
struct RawChoiceData {
    label: DialogTextData,
    #[serde(default)]
    actions: Vec<EventAction>,
}

impl TryFrom<RawChoiceData> for ChoiceData {
    type Error = String;

    fn try_from(raw: RawChoiceData) -> Result<Self, Self::Error> {
        // Validate inline label length: must be 1–80 characters
        if let DialogTextData::Inline(ref text) = raw.label {
            if text.is_empty() {
                return Err("choice label must not be empty".to_string());
            }
            if text.len() > 80 {
                return Err(format!(
                    "choice label must be at most 80 characters, got {}",
                    text.len()
                ));
            }
        }
        // Validate actions count: must be ≤ 20
        if raw.actions.len() > 20 {
            return Err(format!(
                "choice actions list must have at most 20 items, got {}",
                raw.actions.len()
            ));
        }
        Ok(ChoiceData {
            label: raw.label,
            actions: raw.actions,
        })
    }
}

impl<'de> Deserialize<'de> for ChoiceData {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = RawChoiceData::deserialize(deserializer)?;
        ChoiceData::try_from(raw).map_err(serde::de::Error::custom)
    }
}

/// Validated wrapper for `Vec<ChoiceData>` ensuring 2–6 choices.
/// Used internally for deserialization validation of ShowSelection.
fn validate_choices(choices: Vec<ChoiceData>) -> Result<Vec<ChoiceData>, String> {
    if choices.len() < 2 {
        return Err(format!(
            "ShowSelection must have at least 2 choices, got {}",
            choices.len()
        ));
    }
    if choices.len() > 6 {
        return Err(format!(
            "ShowSelection must have at most 6 choices, got {}",
            choices.len()
        ));
    }
    Ok(choices)
}

/// Custom deserializer for the `choices` field that enforces 2–6 count validation.
fn deserialize_validated_choices<'de, D>(deserializer: D) -> Result<Vec<ChoiceData>, D::Error>
where
    D: Deserializer<'de>,
{
    let choices = Vec::<ChoiceData>::deserialize(deserializer)?;
    validate_choices(choices).map_err(serde::de::Error::custom)
}

/// Mode for screen shake effect.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScreenShakeMode {
    #[default]
    Timed,
    Continuous,
}

/// Type of fade transition.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FadeType {
    FadeIn,
    FadeOut,
}

/// Player visual appearance state.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PlayerAppearance {
    Hidden,
    Spritesheet { path: String },
    Default,
}

/// Returns the default fade color (opaque black).
pub fn default_fade_color() -> [f32; 4] {
    [0.0, 0.0, 0.0, 1.0]
}

/// Direction of a reward transfer: Give grants to the player, Take removes from the player.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransferDirection {
    #[default]
    Give,
    Take,
}

/// Returns the default quantity of 1 for GiveItem actions.
fn default_quantity() -> u32 {
    1
}

/// Deserializes and validates a reward amount (currency or experience) in the range [1, 9_999_999].
fn deserialize_reward_amount<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    let amount = u64::deserialize(deserializer)?;
    if !(1..=9_999_999).contains(&amount) {
        return Err(serde::de::Error::custom(format!(
            "amount must be between 1 and 9999999 inclusive, got {}",
            amount
        )));
    }
    Ok(amount)
}

/// Deserializes and validates an item quantity in the range [1, 999].
fn deserialize_item_quantity<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    let quantity = u32::deserialize(deserializer)?;
    if !(1..=999).contains(&quantity) {
        return Err(serde::de::Error::custom(format!(
            "quantity must be between 1 and 999 inclusive, got {}",
            quantity
        )));
    }
    Ok(quantity)
}

/// Deserializes and validates a non-empty string field.
fn deserialize_non_empty_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if s.is_empty() {
        return Err(serde::de::Error::custom("field must not be empty"));
    }
    Ok(s)
}

/// Deserializes and validates an optional string field that, if present, must be non-empty.
fn deserialize_optional_non_empty_string<'de, D>(
    deserializer: D,
) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt = Option::<String>::deserialize(deserializer)?;
    if let Some(ref s) = opt
        && s.is_empty()
    {
        return Err(serde::de::Error::custom(
            "target must not be empty when present",
        ));
    }
    Ok(opt)
}

/// Deserializes and validates a character_id with length 1–64.
fn deserialize_character_id_length<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if s.is_empty() {
        return Err(serde::de::Error::custom("character_id must not be empty"));
    }
    if s.len() > 64 {
        return Err(serde::de::Error::custom(format!(
            "character_id must be at most 64 characters, got {}",
            s.len()
        )));
    }
    Ok(s)
}

/// Helper for deserializing item_quantity with a default of 1, applying validation.
fn deserialize_item_quantity_with_default<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    // This handles the case where the field is present in JSON
    deserialize_item_quantity(deserializer)
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
        /// If set, player elevation is updated to this value after the map transition.
        #[serde(default)]
        target_elevation: Option<u32>,
    },
    ShowDialog {
        text: DialogTextData,
        config: DialogConfigData,
    },
    ScreenShake {
        intensity: f32,
        duration: f32,
        #[serde(default)]
        mode: ScreenShakeMode,
    },
    StopScreenShake,
    FadeTransition {
        fade_type: FadeType,
        duration: f32,
        #[serde(default = "default_fade_color")]
        color: [f32; 4],
    },
    SetState {
        key: String,
        value: String,
    },
    SetPlayerAppearance {
        appearance: PlayerAppearance,
    },
    /// Check a game state flag and execute one of two branches based on the result.
    /// If `value` is `None`, checks only for key existence (key present = true).
    StateCheck {
        key: String,
        value: Option<String>,
        on_true: Vec<EventAction>,
        on_false: Vec<EventAction>,
    },
    /// Evaluate a compound condition and execute one of two branches.
    Branch {
        condition: BranchCondition,
        on_true: Vec<EventAction>,
        on_false: Vec<EventAction>,
    },
    /// Present a selection prompt with multiple choices.
    ShowSelection {
        /// Prompt text displayed above the choices.
        prompt: DialogTextData,
        /// Dialog box configuration (position, portrait, etc.).
        config: DialogConfigData,
        /// Ordered list of choices (2–6 inclusive).
        #[serde(deserialize_with = "deserialize_validated_choices")]
        choices: Vec<ChoiceData>,
    },
    /// Award or deduct currency.
    GiveCurrency {
        #[serde(deserialize_with = "deserialize_reward_amount")]
        amount: u64,
        #[serde(default)]
        direction: TransferDirection,
        #[serde(default)]
        on_success: Vec<EventAction>,
        #[serde(default)]
        on_failure: Vec<EventAction>,
    },
    /// Award or deduct experience points.
    GiveExperience {
        #[serde(deserialize_with = "deserialize_reward_amount")]
        amount: u64,
        #[serde(default, deserialize_with = "deserialize_optional_non_empty_string")]
        target: Option<CharacterId>,
        #[serde(default)]
        direction: TransferDirection,
        #[serde(default)]
        on_success: Vec<EventAction>,
        #[serde(default)]
        on_failure: Vec<EventAction>,
    },
    /// Add or remove an item from inventory.
    GiveItem {
        #[serde(deserialize_with = "deserialize_non_empty_string")]
        item_id: ItemId,
        #[serde(
            default = "default_quantity",
            deserialize_with = "deserialize_item_quantity_with_default"
        )]
        quantity: u32,
        #[serde(default)]
        direction: TransferDirection,
        #[serde(default)]
        on_success: Vec<EventAction>,
        #[serde(default)]
        on_failure: Vec<EventAction>,
    },
    /// Teach or remove an ability from a character.
    LearnAbility {
        #[serde(deserialize_with = "deserialize_non_empty_string")]
        ability_id: AbilityId,
        #[serde(deserialize_with = "deserialize_non_empty_string")]
        target: CharacterId,
        #[serde(default)]
        direction: TransferDirection,
        #[serde(default)]
        on_success: Vec<EventAction>,
        #[serde(default)]
        on_failure: Vec<EventAction>,
    },
    /// Add or remove a character from the active party.
    AddPartyMember {
        #[serde(deserialize_with = "deserialize_character_id_length")]
        character_id: CharacterId,
        #[serde(default)]
        direction: TransferDirection,
        #[serde(default)]
        on_success: Vec<EventAction>,
        #[serde(default)]
        on_failure: Vec<EventAction>,
    },
}

/// Per-tile attribute data: opacity flag, event trigger list, and elevation.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct TileAttributes {
    pub opacity: bool,
    #[serde(default)]
    pub event_trigger: Vec<EventAction>,
    /// Logical elevation level of this tile (0 = ground level).
    #[serde(default)]
    pub elevation: u32,
    /// If set, stepping on this tile transitions the player to this elevation.
    #[serde(default)]
    pub target_elevation: Option<u32>,
    /// If set, the tile is only visible when the game state key matches the value.
    /// When the condition fails, the tile cell renders as `None` (invisible).
    #[serde(default)]
    pub required_state: Option<(String, String)>,
    /// Condition-gated trigger overrides evaluated before the default `event_trigger`.
    /// First matching condition wins; falls through to `event_trigger` if none match.
    #[serde(default)]
    pub conditional_triggers: Vec<ConditionalTrigger>,
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

        // Validate attribute grid dimensions match layer tile dimensions
        for (i, layer) in self.layers.iter().enumerate() {
            let attr_rows = layer.attributes.cells.len();
            if attr_rows != self.height as usize {
                return Err(CommonError::ProjectValidationError(format!(
                    "layer {} attribute grid has {} rows, expected {}",
                    i, attr_rows, self.height
                )));
            }
            for (y, row) in layer.attributes.cells.iter().enumerate() {
                if row.len() != self.width as usize {
                    return Err(CommonError::ProjectValidationError(format!(
                        "layer {} attribute grid row {} has {} columns, expected {}",
                        i,
                        y,
                        row.len(),
                        self.width
                    )));
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper to build a valid ShowSelection JSON with the given number of choices.
    fn show_selection_json(choice_count: usize) -> String {
        let choices: Vec<String> = (0..choice_count)
            .map(|i| {
                format!(
                    r#"{{"label": {{"type": "Inline", "value": "Choice {}"}}, "actions": []}}"#,
                    i + 1
                )
            })
            .collect();
        format!(
            r#"{{
                "type": "ShowSelection",
                "prompt": {{"type": "Inline", "value": "Pick one"}},
                "config": {{"text_speed": 30.0, "position": "Bottom", "movement_block": true, "attribute_dialog": false, "face_portrait": null}},
                "choices": [{}]
            }}"#,
            choices.join(", ")
        )
    }

    #[test]
    fn show_selection_valid_2_choices() {
        let json = show_selection_json(2);
        let result: Result<EventAction, _> = serde_json::from_str(&json);
        assert!(
            result.is_ok(),
            "2 choices should be valid: {:?}",
            result.err()
        );
    }

    #[test]
    fn show_selection_valid_6_choices() {
        let json = show_selection_json(6);
        let result: Result<EventAction, _> = serde_json::from_str(&json);
        assert!(
            result.is_ok(),
            "6 choices should be valid: {:?}",
            result.err()
        );
    }

    #[test]
    fn show_selection_rejects_1_choice() {
        let json = show_selection_json(1);
        let result: Result<EventAction, _> = serde_json::from_str(&json);
        assert!(result.is_err(), "1 choice should be rejected");
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("at least 2"),
            "Error should mention minimum: {}",
            err
        );
    }

    #[test]
    fn show_selection_rejects_7_choices() {
        let json = show_selection_json(7);
        let result: Result<EventAction, _> = serde_json::from_str(&json);
        assert!(result.is_err(), "7 choices should be rejected");
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("at most 6"),
            "Error should mention maximum: {}",
            err
        );
    }

    #[test]
    fn show_selection_rejects_0_choices() {
        let json = show_selection_json(0);
        let result: Result<EventAction, _> = serde_json::from_str(&json);
        assert!(result.is_err(), "0 choices should be rejected");
    }

    #[test]
    fn choice_data_rejects_empty_inline_label() {
        let json = r#"{"label": {"type": "Inline", "value": ""}, "actions": []}"#;
        let result: Result<ChoiceData, _> = serde_json::from_str(json);
        assert!(result.is_err(), "Empty label should be rejected");
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("must not be empty"),
            "Error should mention empty: {}",
            err
        );
    }

    #[test]
    fn choice_data_rejects_label_over_80_chars() {
        let long_label = "a".repeat(81);
        let json = format!(
            r#"{{"label": {{"type": "Inline", "value": "{}"}}, "actions": []}}"#,
            long_label
        );
        let result: Result<ChoiceData, _> = serde_json::from_str(&json);
        assert!(result.is_err(), "81-char label should be rejected");
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("at most 80"),
            "Error should mention max: {}",
            err
        );
    }

    #[test]
    fn choice_data_accepts_80_char_label() {
        let label = "a".repeat(80);
        let json = format!(
            r#"{{"label": {{"type": "Inline", "value": "{}"}}, "actions": []}}"#,
            label
        );
        let result: Result<ChoiceData, _> = serde_json::from_str(&json);
        assert!(
            result.is_ok(),
            "80-char label should be valid: {:?}",
            result.err()
        );
    }

    #[test]
    fn choice_data_accepts_id_label_without_length_check() {
        // ID labels defer validation to runtime — any length string is accepted
        let json = r#"{"label": {"type": "Id", "value": ""}, "actions": []}"#;
        let result: Result<ChoiceData, _> = serde_json::from_str(json);
        assert!(
            result.is_ok(),
            "Id labels should not be length-checked: {:?}",
            result.err()
        );
    }

    #[test]
    fn choice_data_rejects_over_20_actions() {
        let actions: Vec<String> = (0..21)
            .map(|i| format!(r#"{{"type": "SetState", "key": "k{}", "value": "v"}}"#, i))
            .collect();
        let json = format!(
            r#"{{"label": {{"type": "Inline", "value": "Go"}}, "actions": [{}]}}"#,
            actions.join(", ")
        );
        let result: Result<ChoiceData, _> = serde_json::from_str(&json);
        assert!(result.is_err(), "21 actions should be rejected");
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("at most 20"),
            "Error should mention max actions: {}",
            err
        );
    }

    #[test]
    fn choice_data_accepts_20_actions() {
        let actions: Vec<String> = (0..20)
            .map(|i| format!(r#"{{"type": "SetState", "key": "k{}", "value": "v"}}"#, i))
            .collect();
        let json = format!(
            r#"{{"label": {{"type": "Inline", "value": "Go"}}, "actions": [{}]}}"#,
            actions.join(", ")
        );
        let result: Result<ChoiceData, _> = serde_json::from_str(&json);
        assert!(
            result.is_ok(),
            "20 actions should be valid: {:?}",
            result.err()
        );
    }

    #[test]
    fn show_selection_round_trip() {
        let original = EventAction::ShowSelection {
            prompt: DialogTextData::Inline("What will you do?".to_string()),
            config: DialogConfigData::default(),
            choices: vec![
                ChoiceData {
                    label: DialogTextData::Inline("Fight".to_string()),
                    actions: vec![EventAction::SetState {
                        key: "choice".to_string(),
                        value: "fight".to_string(),
                    }],
                },
                ChoiceData {
                    label: DialogTextData::Id("flee_label".to_string()),
                    actions: vec![],
                },
            ],
        };

        let json = serde_json::to_string(&original).unwrap();
        let deserialized: EventAction = serde_json::from_str(&json).unwrap();
        assert_eq!(original, deserialized);
    }

    #[test]
    fn show_selection_type_tag_present() {
        let action = EventAction::ShowSelection {
            prompt: DialogTextData::Inline("Choose".to_string()),
            config: DialogConfigData::default(),
            choices: vec![
                ChoiceData {
                    label: DialogTextData::Inline("A".to_string()),
                    actions: vec![],
                },
                ChoiceData {
                    label: DialogTextData::Inline("B".to_string()),
                    actions: vec![],
                },
            ],
        };

        let json = serde_json::to_string(&action).unwrap();
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(value["type"], "ShowSelection");
    }

    #[test]
    fn show_selection_missing_prompt_field_errors() {
        let json = r#"{
            "type": "ShowSelection",
            "config": {"text_speed": 30.0, "position": "Bottom", "movement_block": true, "attribute_dialog": false, "face_portrait": null},
            "choices": [
                {"label": {"type": "Inline", "value": "A"}, "actions": []},
                {"label": {"type": "Inline", "value": "B"}, "actions": []}
            ]
        }"#;
        let result: Result<EventAction, _> = serde_json::from_str(json);
        assert!(result.is_err(), "Missing prompt should cause error");
    }

    #[test]
    fn show_selection_missing_choices_field_errors() {
        let json = r#"{
            "type": "ShowSelection",
            "prompt": {"type": "Inline", "value": "Pick"},
            "config": {"text_speed": 30.0, "position": "Bottom", "movement_block": true, "attribute_dialog": false, "face_portrait": null}
        }"#;
        let result: Result<EventAction, _> = serde_json::from_str(json);
        assert!(result.is_err(), "Missing choices should cause error");
    }

    #[test]
    fn choice_data_actions_default_to_empty() {
        let json = r#"{"label": {"type": "Inline", "value": "Go"}}"#;
        let result: Result<ChoiceData, _> = serde_json::from_str(json);
        assert!(
            result.is_ok(),
            "actions should default to empty: {:?}",
            result.err()
        );
        assert_eq!(result.unwrap().actions, vec![]);
    }

    // --- GiveCurrency validation tests ---

    #[test]
    fn give_currency_valid_amount() {
        let json = r#"{"type": "GiveCurrency", "amount": 100}"#;
        let result: Result<EventAction, _> = serde_json::from_str(json);
        assert!(
            result.is_ok(),
            "Valid amount should parse: {:?}",
            result.err()
        );
    }

    #[test]
    fn give_currency_min_amount() {
        let json = r#"{"type": "GiveCurrency", "amount": 1}"#;
        let result: Result<EventAction, _> = serde_json::from_str(json);
        assert!(
            result.is_ok(),
            "Min amount (1) should be valid: {:?}",
            result.err()
        );
    }

    #[test]
    fn give_currency_max_amount() {
        let json = r#"{"type": "GiveCurrency", "amount": 9999999}"#;
        let result: Result<EventAction, _> = serde_json::from_str(json);
        assert!(
            result.is_ok(),
            "Max amount (9999999) should be valid: {:?}",
            result.err()
        );
    }

    #[test]
    fn give_currency_rejects_zero_amount() {
        let json = r#"{"type": "GiveCurrency", "amount": 0}"#;
        let result: Result<EventAction, _> = serde_json::from_str(json);
        assert!(result.is_err(), "Zero amount should be rejected");
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("between 1 and 9999999"),
            "Error should mention range: {}",
            err
        );
    }

    #[test]
    fn give_currency_rejects_amount_over_max() {
        let json = r#"{"type": "GiveCurrency", "amount": 10000000}"#;
        let result: Result<EventAction, _> = serde_json::from_str(json);
        assert!(result.is_err(), "Amount over max should be rejected");
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("between 1 and 9999999"),
            "Error should mention range: {}",
            err
        );
    }

    // --- GiveExperience validation tests ---

    #[test]
    fn give_experience_valid_no_target() {
        let json = r#"{"type": "GiveExperience", "amount": 500}"#;
        let result: Result<EventAction, _> = serde_json::from_str(json);
        assert!(
            result.is_ok(),
            "Valid experience should parse: {:?}",
            result.err()
        );
    }

    #[test]
    fn give_experience_valid_with_target() {
        let json = r#"{"type": "GiveExperience", "amount": 500, "target": "hero_01"}"#;
        let result: Result<EventAction, _> = serde_json::from_str(json);
        assert!(
            result.is_ok(),
            "Valid experience with target should parse: {:?}",
            result.err()
        );
    }

    #[test]
    fn give_experience_rejects_zero_amount() {
        let json = r#"{"type": "GiveExperience", "amount": 0}"#;
        let result: Result<EventAction, _> = serde_json::from_str(json);
        assert!(result.is_err(), "Zero amount should be rejected");
    }

    #[test]
    fn give_experience_rejects_empty_target() {
        let json = r#"{"type": "GiveExperience", "amount": 100, "target": ""}"#;
        let result: Result<EventAction, _> = serde_json::from_str(json);
        assert!(result.is_err(), "Empty target should be rejected");
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("must not be empty"),
            "Error should mention empty: {}",
            err
        );
    }

    // --- GiveItem validation tests ---

    #[test]
    fn give_item_valid() {
        let json = r#"{"type": "GiveItem", "item_id": "potion", "quantity": 5}"#;
        let result: Result<EventAction, _> = serde_json::from_str(json);
        assert!(
            result.is_ok(),
            "Valid GiveItem should parse: {:?}",
            result.err()
        );
    }

    #[test]
    fn give_item_quantity_defaults_to_1() {
        let json = r#"{"type": "GiveItem", "item_id": "potion"}"#;
        let result: Result<EventAction, _> = serde_json::from_str(json);
        assert!(
            result.is_ok(),
            "GiveItem without quantity should default to 1: {:?}",
            result.err()
        );
        if let EventAction::GiveItem { quantity, .. } = result.unwrap() {
            assert_eq!(quantity, 1);
        } else {
            panic!("Expected GiveItem variant");
        }
    }

    #[test]
    fn give_item_rejects_empty_item_id() {
        let json = r#"{"type": "GiveItem", "item_id": "", "quantity": 1}"#;
        let result: Result<EventAction, _> = serde_json::from_str(json);
        assert!(result.is_err(), "Empty item_id should be rejected");
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("must not be empty"),
            "Error should mention empty: {}",
            err
        );
    }

    #[test]
    fn give_item_rejects_quantity_zero() {
        let json = r#"{"type": "GiveItem", "item_id": "potion", "quantity": 0}"#;
        let result: Result<EventAction, _> = serde_json::from_str(json);
        assert!(result.is_err(), "Zero quantity should be rejected");
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("between 1 and 999"),
            "Error should mention range: {}",
            err
        );
    }

    #[test]
    fn give_item_rejects_quantity_over_999() {
        let json = r#"{"type": "GiveItem", "item_id": "potion", "quantity": 1000}"#;
        let result: Result<EventAction, _> = serde_json::from_str(json);
        assert!(result.is_err(), "Quantity over 999 should be rejected");
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("between 1 and 999"),
            "Error should mention range: {}",
            err
        );
    }

    #[test]
    fn give_item_accepts_max_quantity() {
        let json = r#"{"type": "GiveItem", "item_id": "potion", "quantity": 999}"#;
        let result: Result<EventAction, _> = serde_json::from_str(json);
        assert!(
            result.is_ok(),
            "Max quantity (999) should be valid: {:?}",
            result.err()
        );
    }

    // --- LearnAbility validation tests ---

    #[test]
    fn learn_ability_valid() {
        let json = r#"{"type": "LearnAbility", "ability_id": "fireball", "target": "mage_01"}"#;
        let result: Result<EventAction, _> = serde_json::from_str(json);
        assert!(
            result.is_ok(),
            "Valid LearnAbility should parse: {:?}",
            result.err()
        );
    }

    #[test]
    fn learn_ability_rejects_empty_ability_id() {
        let json = r#"{"type": "LearnAbility", "ability_id": "", "target": "mage_01"}"#;
        let result: Result<EventAction, _> = serde_json::from_str(json);
        assert!(result.is_err(), "Empty ability_id should be rejected");
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("must not be empty"),
            "Error should mention empty: {}",
            err
        );
    }

    #[test]
    fn learn_ability_rejects_empty_target() {
        let json = r#"{"type": "LearnAbility", "ability_id": "fireball", "target": ""}"#;
        let result: Result<EventAction, _> = serde_json::from_str(json);
        assert!(result.is_err(), "Empty target should be rejected");
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("must not be empty"),
            "Error should mention empty: {}",
            err
        );
    }

    // --- AddPartyMember validation tests ---

    #[test]
    fn add_party_member_valid() {
        let json = r#"{"type": "AddPartyMember", "character_id": "hero_01"}"#;
        let result: Result<EventAction, _> = serde_json::from_str(json);
        assert!(
            result.is_ok(),
            "Valid AddPartyMember should parse: {:?}",
            result.err()
        );
    }

    #[test]
    fn add_party_member_rejects_empty_character_id() {
        let json = r#"{"type": "AddPartyMember", "character_id": ""}"#;
        let result: Result<EventAction, _> = serde_json::from_str(json);
        assert!(result.is_err(), "Empty character_id should be rejected");
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("must not be empty"),
            "Error should mention empty: {}",
            err
        );
    }

    #[test]
    fn add_party_member_rejects_character_id_over_64() {
        let long_id = "a".repeat(65);
        let json = format!(
            r#"{{"type": "AddPartyMember", "character_id": "{}"}}"#,
            long_id
        );
        let result: Result<EventAction, _> = serde_json::from_str(&json);
        assert!(
            result.is_err(),
            "character_id over 64 chars should be rejected"
        );
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("at most 64"),
            "Error should mention max length: {}",
            err
        );
    }

    #[test]
    fn add_party_member_accepts_64_char_character_id() {
        let id = "a".repeat(64);
        let json = format!(r#"{{"type": "AddPartyMember", "character_id": "{}"}}"#, id);
        let result: Result<EventAction, _> = serde_json::from_str(&json);
        assert!(
            result.is_ok(),
            "64-char character_id should be valid: {:?}",
            result.err()
        );
    }

    // --- Backward compatibility: pre-existing variants still deserialize ---

    #[test]
    fn backward_compat_jump_to_deserializes() {
        let json =
            r#"{"type": "JumpTo", "target_map_id": "map-001", "target_x": 5, "target_y": 10}"#;
        let result: Result<EventAction, _> = serde_json::from_str(json);
        assert!(
            result.is_ok(),
            "JumpTo should still deserialize: {:?}",
            result.err()
        );
        if let EventAction::JumpTo {
            target_map_id,
            target_x,
            target_y,
            target_elevation,
        } = result.unwrap()
        {
            assert_eq!(target_map_id, "map-001");
            assert_eq!(target_x, 5);
            assert_eq!(target_y, 10);
            assert_eq!(target_elevation, None);
        } else {
            panic!("Expected JumpTo variant");
        }
    }

    #[test]
    fn backward_compat_set_state_deserializes() {
        let json = r#"{"type": "SetState", "key": "chest_opened", "value": "true"}"#;
        let result: Result<EventAction, _> = serde_json::from_str(json);
        assert!(
            result.is_ok(),
            "SetState should still deserialize: {:?}",
            result.err()
        );
        if let EventAction::SetState { key, value } = result.unwrap() {
            assert_eq!(key, "chest_opened");
            assert_eq!(value, "true");
        } else {
            panic!("Expected SetState variant");
        }
    }

    #[test]
    fn backward_compat_screen_shake_deserializes() {
        let json = r#"{"type": "ScreenShake", "intensity": 5.0, "duration": 1.5}"#;
        let result: Result<EventAction, _> = serde_json::from_str(json);
        assert!(
            result.is_ok(),
            "ScreenShake should still deserialize: {:?}",
            result.err()
        );
        if let EventAction::ScreenShake {
            intensity,
            duration,
            mode,
        } = result.unwrap()
        {
            assert_eq!(intensity, 5.0);
            assert_eq!(duration, 1.5);
            assert_eq!(mode, ScreenShakeMode::Timed);
        } else {
            panic!("Expected ScreenShake variant");
        }
    }

    #[test]
    fn backward_compat_stop_screen_shake_deserializes() {
        let json = r#"{"type": "StopScreenShake"}"#;
        let result: Result<EventAction, _> = serde_json::from_str(json);
        assert!(
            result.is_ok(),
            "StopScreenShake should still deserialize: {:?}",
            result.err()
        );
        assert!(matches!(result.unwrap(), EventAction::StopScreenShake));
    }

    #[test]
    fn backward_compat_fade_transition_deserializes() {
        let json = r#"{"type": "FadeTransition", "fade_type": "FadeOut", "duration": 0.5}"#;
        let result: Result<EventAction, _> = serde_json::from_str(json);
        assert!(
            result.is_ok(),
            "FadeTransition should still deserialize: {:?}",
            result.err()
        );
        if let EventAction::FadeTransition {
            fade_type,
            duration,
            color,
        } = result.unwrap()
        {
            assert_eq!(fade_type, FadeType::FadeOut);
            assert_eq!(duration, 0.5);
            assert_eq!(color, [0.0, 0.0, 0.0, 1.0]);
        } else {
            panic!("Expected FadeTransition variant");
        }
    }

    // --- Reward variants serialize with correct "type" tag ---

    #[test]
    fn give_currency_serializes_with_correct_type_tag() {
        let action = EventAction::GiveCurrency {
            amount: 500,
            direction: TransferDirection::Give,
            on_success: vec![],
            on_failure: vec![],
        };
        let json = serde_json::to_string(&action).unwrap();
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(value["type"], "GiveCurrency");
        assert_eq!(value["amount"], 500);
    }

    #[test]
    fn give_experience_serializes_with_correct_type_tag() {
        let action = EventAction::GiveExperience {
            amount: 1000,
            target: Some("warrior".to_string()),
            direction: TransferDirection::Take,
            on_success: vec![],
            on_failure: vec![],
        };
        let json = serde_json::to_string(&action).unwrap();
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(value["type"], "GiveExperience");
        assert_eq!(value["amount"], 1000);
        assert_eq!(value["target"], "warrior");
        assert_eq!(value["direction"], "Take");
    }

    #[test]
    fn give_item_serializes_with_correct_type_tag() {
        let action = EventAction::GiveItem {
            item_id: "sword_01".to_string(),
            quantity: 3,
            direction: TransferDirection::Give,
            on_success: vec![],
            on_failure: vec![],
        };
        let json = serde_json::to_string(&action).unwrap();
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(value["type"], "GiveItem");
        assert_eq!(value["item_id"], "sword_01");
        assert_eq!(value["quantity"], 3);
    }

    #[test]
    fn learn_ability_serializes_with_correct_type_tag() {
        let action = EventAction::LearnAbility {
            ability_id: "heal".to_string(),
            target: "cleric_01".to_string(),
            direction: TransferDirection::Give,
            on_success: vec![],
            on_failure: vec![],
        };
        let json = serde_json::to_string(&action).unwrap();
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(value["type"], "LearnAbility");
        assert_eq!(value["ability_id"], "heal");
        assert_eq!(value["target"], "cleric_01");
    }

    #[test]
    fn add_party_member_serializes_with_correct_type_tag() {
        let action = EventAction::AddPartyMember {
            character_id: "npc_ally".to_string(),
            direction: TransferDirection::Take,
            on_success: vec![EventAction::SetState {
                key: "recruited".to_string(),
                value: "true".to_string(),
            }],
            on_failure: vec![],
        };
        let json = serde_json::to_string(&action).unwrap();
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(value["type"], "AddPartyMember");
        assert_eq!(value["character_id"], "npc_ally");
        assert_eq!(value["direction"], "Take");
    }

    // --- direction defaults to Give when absent from JSON ---

    #[test]
    fn give_currency_direction_defaults_to_give() {
        let json = r#"{"type": "GiveCurrency", "amount": 100}"#;
        let action: EventAction = serde_json::from_str(json).unwrap();
        if let EventAction::GiveCurrency { direction, .. } = action {
            assert_eq!(direction, TransferDirection::Give);
        } else {
            panic!("Expected GiveCurrency variant");
        }
    }

    #[test]
    fn give_experience_direction_defaults_to_give() {
        let json = r#"{"type": "GiveExperience", "amount": 200}"#;
        let action: EventAction = serde_json::from_str(json).unwrap();
        if let EventAction::GiveExperience { direction, .. } = action {
            assert_eq!(direction, TransferDirection::Give);
        } else {
            panic!("Expected GiveExperience variant");
        }
    }

    #[test]
    fn give_item_direction_defaults_to_give() {
        let json = r#"{"type": "GiveItem", "item_id": "potion"}"#;
        let action: EventAction = serde_json::from_str(json).unwrap();
        if let EventAction::GiveItem { direction, .. } = action {
            assert_eq!(direction, TransferDirection::Give);
        } else {
            panic!("Expected GiveItem variant");
        }
    }

    #[test]
    fn learn_ability_direction_defaults_to_give() {
        let json = r#"{"type": "LearnAbility", "ability_id": "fireball", "target": "mage"}"#;
        let action: EventAction = serde_json::from_str(json).unwrap();
        if let EventAction::LearnAbility { direction, .. } = action {
            assert_eq!(direction, TransferDirection::Give);
        } else {
            panic!("Expected LearnAbility variant");
        }
    }

    #[test]
    fn add_party_member_direction_defaults_to_give() {
        let json = r#"{"type": "AddPartyMember", "character_id": "hero"}"#;
        let action: EventAction = serde_json::from_str(json).unwrap();
        if let EventAction::AddPartyMember { direction, .. } = action {
            assert_eq!(direction, TransferDirection::Give);
        } else {
            panic!("Expected AddPartyMember variant");
        }
    }

    // --- on_success/on_failure default to empty when absent from JSON ---

    #[test]
    fn give_currency_on_success_on_failure_default_to_empty() {
        let json = r#"{"type": "GiveCurrency", "amount": 50}"#;
        let action: EventAction = serde_json::from_str(json).unwrap();
        if let EventAction::GiveCurrency {
            on_success,
            on_failure,
            ..
        } = action
        {
            assert!(on_success.is_empty(), "on_success should default to empty");
            assert!(on_failure.is_empty(), "on_failure should default to empty");
        } else {
            panic!("Expected GiveCurrency variant");
        }
    }

    #[test]
    fn give_experience_on_success_on_failure_default_to_empty() {
        let json = r#"{"type": "GiveExperience", "amount": 100}"#;
        let action: EventAction = serde_json::from_str(json).unwrap();
        if let EventAction::GiveExperience {
            on_success,
            on_failure,
            ..
        } = action
        {
            assert!(on_success.is_empty(), "on_success should default to empty");
            assert!(on_failure.is_empty(), "on_failure should default to empty");
        } else {
            panic!("Expected GiveExperience variant");
        }
    }

    #[test]
    fn give_item_on_success_on_failure_default_to_empty() {
        let json = r#"{"type": "GiveItem", "item_id": "key"}"#;
        let action: EventAction = serde_json::from_str(json).unwrap();
        if let EventAction::GiveItem {
            on_success,
            on_failure,
            ..
        } = action
        {
            assert!(on_success.is_empty(), "on_success should default to empty");
            assert!(on_failure.is_empty(), "on_failure should default to empty");
        } else {
            panic!("Expected GiveItem variant");
        }
    }

    #[test]
    fn learn_ability_on_success_on_failure_default_to_empty() {
        let json = r#"{"type": "LearnAbility", "ability_id": "heal", "target": "cleric"}"#;
        let action: EventAction = serde_json::from_str(json).unwrap();
        if let EventAction::LearnAbility {
            on_success,
            on_failure,
            ..
        } = action
        {
            assert!(on_success.is_empty(), "on_success should default to empty");
            assert!(on_failure.is_empty(), "on_failure should default to empty");
        } else {
            panic!("Expected LearnAbility variant");
        }
    }

    #[test]
    fn add_party_member_on_success_on_failure_default_to_empty() {
        let json = r#"{"type": "AddPartyMember", "character_id": "ally"}"#;
        let action: EventAction = serde_json::from_str(json).unwrap();
        if let EventAction::AddPartyMember {
            on_success,
            on_failure,
            ..
        } = action
        {
            assert!(on_success.is_empty(), "on_success should default to empty");
            assert!(on_failure.is_empty(), "on_failure should default to empty");
        } else {
            panic!("Expected AddPartyMember variant");
        }
    }

    // --- Invalid direction string produces descriptive error ---

    #[test]
    fn invalid_direction_produces_descriptive_error() {
        let json = r#"{"type": "GiveCurrency", "amount": 100, "direction": "Invalid"}"#;
        let result: Result<EventAction, _> = serde_json::from_str(json);
        assert!(result.is_err(), "Invalid direction should be rejected");
        let err = result.unwrap_err().to_string();
        // Serde should produce an error mentioning the unknown variant
        assert!(
            err.contains("Invalid") || err.contains("unknown variant") || err.contains("expected"),
            "Error should be descriptive about invalid direction: {}",
            err
        );
    }

    #[test]
    fn invalid_direction_on_give_experience_produces_error() {
        let json = r#"{"type": "GiveExperience", "amount": 50, "direction": "Steal"}"#;
        let result: Result<EventAction, _> = serde_json::from_str(json);
        assert!(
            result.is_err(),
            "Invalid direction 'Steal' should be rejected"
        );
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("Steal") || err.contains("unknown variant") || err.contains("expected"),
            "Error should be descriptive about invalid direction: {}",
            err
        );
    }

    #[test]
    fn invalid_direction_on_give_item_produces_error() {
        let json = r#"{"type": "GiveItem", "item_id": "potion", "direction": "Borrow"}"#;
        let result: Result<EventAction, _> = serde_json::from_str(json);
        assert!(
            result.is_err(),
            "Invalid direction 'Borrow' should be rejected"
        );
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("Borrow") || err.contains("unknown variant") || err.contains("expected"),
            "Error should be descriptive about invalid direction: {}",
            err
        );
    }
}
