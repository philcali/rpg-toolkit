pub mod ability;
pub mod animation;
pub mod app_phase;
pub mod asset;
pub mod character;
pub mod condition;
pub mod element;
pub mod enemy;
pub mod error;
pub mod graphics;
pub mod item;
pub mod manifest;
pub mod map;
pub mod project;
pub mod save;
pub mod shop;
pub mod spritesheet;
pub mod tileset;

pub use ability::{
    Ability, AbilityCategory, AbilityId, AbilityRegistry, AbilitySource, CostType, TargetType,
};
pub use animation::{
    AnimationFrame, TileAnimation, compute_animation_frame_index, validate_tile_animation,
};
pub use app_phase::{AppPhase, NewGameFlag};
pub use asset::{
    AssetCategory, AssetManager, AssetReference, AssetRegistry, AssetValidationError, AssetWarning,
    CATEGORY_FACE_PORTRAIT, CATEGORY_SPRITESHEET, CATEGORY_TILESET, ProjectSource,
};
pub use character::{
    Character, CharacterId, CharacterRegistry, LearnableAbility, OPTIONAL_STATS, REQUIRED_STATS,
    Stat, VisualAssetType,
};
pub use condition::{
    BranchCondition, ConditionCheck, ConditionLogic, ConditionOperator, ConditionalTrigger,
};
pub use element::Element;
pub use enemy::{
    CarriedItem, DefeatReward, ElementalModifier, Enemy, EnemyId, EnemyRegistry, EnemyStat,
    ItemDrop,
};
pub use error::CommonError;
pub use graphics::EntityGraphics;
pub use item::{
    BuffTargetStat, ConsumableEffect, ConsumableEffectType, CureTargetStatus, EquipmentSlot, Item,
    ItemCategory, ItemCategoryData, ItemId, ItemRegistry, Rarity, StatModifier,
    format_modifier_value,
};
pub use manifest::ProjectManifest;
pub use map::{
    ChoiceData, DialogConfigData, DialogPositionData, DialogTextData, EntityTarget, EventAction,
    FadeType, Layer, MapData, MapId, PlayerAppearance, ScreenShakeMode, SpawnPoint,
    TileAttributeLayer, TileAttributes, TileRef, TilesetId, TransferDirection,
};
pub use project::{ProjectFile, SpritesheetReferences};
pub use save::{CharacterProgressData, SaveFile};
pub use shop::{ActiveShopId, ShopDefinition, ShopEntry, ShopId, ShopRegistry};
pub use spritesheet::{
    CharacterSpritesheet, FacingDirection, NpcInstance, PatrolConfig, PatrolMode, SpritesheetId,
    TriggerMode, faced_tile, next_waypoint_index, sprite_atlas_index,
    validate_spritesheet_dimensions, validate_waypoint_bounds, walk_animation_frame,
};
pub use tileset::TilesetMeta;
