pub mod animation;
pub mod character;
pub mod condition;
pub mod error;
pub mod item;
pub mod manifest;
pub mod map;
pub mod project;
pub mod spritesheet;
pub mod tileset;

pub use animation::{
    AnimationFrame, TileAnimation, compute_animation_frame_index, validate_tile_animation,
};
pub use character::{
    Character, CharacterId, CharacterRegistry, OPTIONAL_STATS, REQUIRED_STATS, Stat,
};
pub use condition::{
    BranchCondition, ConditionCheck, ConditionLogic, ConditionOperator, ConditionalTrigger,
};
pub use error::CommonError;
pub use item::{
    BuffTargetStat, ConsumableEffect, ConsumableEffectType, CureTargetStatus, EquipmentSlot, Item,
    ItemCategory, ItemCategoryData, ItemId, ItemRegistry, Rarity, StatModifier,
    format_modifier_value,
};
pub use manifest::ProjectManifest;
pub use map::{
    ChoiceData, DialogConfigData, DialogPositionData, DialogTextData, EventAction, FadeType, Layer,
    MapData, MapId, PlayerAppearance, ScreenShakeMode, SpawnPoint, TileAttributeLayer,
    TileAttributes, TileRef, TilesetId,
};
pub use project::{ProjectFile, SpritesheetReferences};
pub use spritesheet::{
    CharacterSpritesheet, FacingDirection, NpcInstance, PatrolConfig, PatrolMode, SpritesheetId,
    TriggerMode, faced_tile, next_waypoint_index, sprite_atlas_index,
    validate_spritesheet_dimensions, validate_waypoint_bounds, walk_animation_frame,
};
pub use tileset::TilesetMeta;
