use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::CommonError;

pub type ItemId = String;

/// Rarity tier for items.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
}

/// Equipment slots available for items.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EquipmentSlot {
    MainHand,
    OffHand,
    Head,
    Body,
    Legs,
    Feet,
    Accessory1,
    Accessory2,
}

/// A named stat modifier on an item.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatModifier {
    pub stat_name: String,
    pub value: i32,
}

/// Target stat for buff effects.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BuffTargetStat {
    Strength,
    Stamina,
    Speed,
    Luck,
    Wisdom,
    Intelligence,
}

/// Target status for cure effects.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CureTargetStatus {
    Poison,
    Paralysis,
    Sleep,
    Confusion,
    Silence,
    All,
}

/// The type of effect a consumable applies.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "effect_type")]
pub enum ConsumableEffectType {
    RestoreHP,
    RestoreMP,
    CureStatus {
        target_status: CureTargetStatus,
    },
    BuffStat {
        target_stat: BuffTargetStat,
        duration: u32,
    },
}

/// A single consumable effect with type and potency.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConsumableEffect {
    pub effect: ConsumableEffectType,
    pub potency: u32,
}

/// Category-specific data, stored as a serde-tagged enum.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "category")]
pub enum ItemCategoryData {
    Weapon {
        attack_power: u32,
        equipment_slot: EquipmentSlot,
    },
    Armor {
        defense_power: u32,
        equipment_slot: EquipmentSlot,
    },
    Accessory {
        equipment_slot: EquipmentSlot,
    },
    Consumable {
        effects: Vec<ConsumableEffect>,
    },
    KeyItem,
}

/// Enum used for filtering and creation UI (not stored on the item).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ItemCategory {
    Weapon,
    Armor,
    Accessory,
    Consumable,
    KeyItem,
}

/// A game item with all its properties.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Item {
    pub id: ItemId,
    pub display_name: String,
    pub description: String,
    pub category_data: ItemCategoryData,
    pub value: u32,
    pub rarity: Rarity,
    pub stackable: bool,
    pub stack_limit: u32,
    pub stat_modifiers: Vec<StatModifier>,
}

impl Item {
    /// Returns the category enum for this item.
    pub fn category(&self) -> ItemCategory {
        match &self.category_data {
            ItemCategoryData::Weapon { .. } => ItemCategory::Weapon,
            ItemCategoryData::Armor { .. } => ItemCategory::Armor,
            ItemCategoryData::Accessory { .. } => ItemCategory::Accessory,
            ItemCategoryData::Consumable { .. } => ItemCategory::Consumable,
            ItemCategoryData::KeyItem => ItemCategory::KeyItem,
        }
    }
}

/// Project-level collection of items.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemRegistry {
    pub items: HashMap<ItemId, Item>,
}

/// Validates a display name: must be 1–64 non-whitespace characters after trimming.
fn validate_name(name: &str) -> Result<String, CommonError> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(CommonError::ItemValidationError(
            "Display name must contain at least 1 non-whitespace character".to_string(),
        ));
    }
    if trimmed.len() > 64 {
        return Err(CommonError::ItemValidationError(
            "Display name must not exceed 64 characters".to_string(),
        ));
    }
    Ok(trimmed.to_string())
}

/// Returns the default `ItemCategoryData` for a given category.
fn default_category_data(category: ItemCategory) -> ItemCategoryData {
    match category {
        ItemCategory::Weapon => ItemCategoryData::Weapon {
            attack_power: 0,
            equipment_slot: EquipmentSlot::MainHand,
        },
        ItemCategory::Armor => ItemCategoryData::Armor {
            defense_power: 0,
            equipment_slot: EquipmentSlot::Body,
        },
        ItemCategory::Accessory => ItemCategoryData::Accessory {
            equipment_slot: EquipmentSlot::Accessory1,
        },
        ItemCategory::Consumable => ItemCategoryData::Consumable {
            effects: vec![ConsumableEffect {
                effect: ConsumableEffectType::RestoreHP,
                potency: 10,
            }],
        },
        ItemCategory::KeyItem => ItemCategoryData::KeyItem,
    }
}

impl ItemRegistry {
    /// Creates a new item with the given name and category, using category defaults.
    /// Returns the generated ItemId on success.
    pub fn create_item(
        &mut self,
        name: &str,
        category: ItemCategory,
    ) -> Result<ItemId, CommonError> {
        let display_name = validate_name(name)?;
        let id = Uuid::new_v4().to_string();

        let (stackable, stack_limit) = match category {
            ItemCategory::Consumable => (true, 99),
            _ => (false, 1),
        };

        let item = Item {
            id: id.clone(),
            display_name,
            description: String::new(),
            category_data: default_category_data(category),
            value: 0,
            rarity: Rarity::Common,
            stackable,
            stack_limit,
            stat_modifiers: Vec::new(),
        };

        self.items.insert(id.clone(), item);
        Ok(id)
    }

    /// Removes an item from the registry. Returns an error if the item is not found.
    pub fn delete_item(&mut self, id: &ItemId) -> Result<(), CommonError> {
        if self.items.remove(id).is_none() {
            return Err(CommonError::ItemValidationError(format!(
                "Item with id '{}' not found",
                id
            )));
        }
        Ok(())
    }

    /// Updates an item's display name after validation.
    pub fn update_display_name(&mut self, id: &ItemId, name: &str) -> Result<(), CommonError> {
        let display_name = validate_name(name)?;
        let item = self.items.get_mut(id).ok_or_else(|| {
            CommonError::ItemValidationError(format!("Item with id '{}' not found", id))
        })?;
        item.display_name = display_name;
        Ok(())
    }

    /// Updates an item's description, truncating at 256 characters.
    pub fn update_description(&mut self, id: &ItemId, desc: &str) -> Result<(), CommonError> {
        let item = self.items.get_mut(id).ok_or_else(|| {
            CommonError::ItemValidationError(format!("Item with id '{}' not found", id))
        })?;
        let truncated: String = desc.chars().take(256).collect();
        item.description = truncated;
        Ok(())
    }

    /// Changes an item's category, replacing category_data with defaults and enforcing
    /// category constraints. Preserves display_name, description, stat_modifiers, and rarity.
    /// Preserves value except when changing to KeyItem (forces value=0).
    pub fn change_category(
        &mut self,
        id: &ItemId,
        new_category: ItemCategory,
    ) -> Result<(), CommonError> {
        let item = self.items.get_mut(id).ok_or_else(|| {
            CommonError::ItemValidationError(format!("Item with id '{}' not found", id))
        })?;

        item.category_data = default_category_data(new_category);

        match new_category {
            ItemCategory::Consumable => {
                item.stackable = true;
                item.stack_limit = 99;
            }
            ItemCategory::KeyItem => {
                item.stackable = false;
                item.stack_limit = 1;
                item.value = 0;
            }
            _ => {
                // Preserve current stackable/stack_limit unless invariant would be broken
                if item.stackable && item.stack_limit < 2 {
                    item.stack_limit = 99;
                } else if !item.stackable {
                    item.stack_limit = 1;
                }
            }
        }

        Ok(())
    }

    /// Toggles the stackable flag on an item.
    /// If setting to true and current stack_limit is 1, sets stack_limit to 99.
    /// If setting to false, sets stack_limit to 1.
    /// Rejects setting stackable=false on Consumable items (must always be stackable).
    /// Rejects setting stackable=true on KeyItem items (must never be stackable).
    pub fn set_stackable(&mut self, id: &ItemId, stackable: bool) -> Result<(), CommonError> {
        let item = self.items.get_mut(id).ok_or_else(|| {
            CommonError::ItemValidationError(format!("Item with id '{}' not found", id))
        })?;

        // Enforce category-specific constraints
        match item.category() {
            ItemCategory::Consumable if !stackable => {
                return Err(CommonError::ItemValidationError(
                    "Consumable items must always be stackable".to_string(),
                ));
            }
            ItemCategory::KeyItem if stackable => {
                return Err(CommonError::ItemValidationError(
                    "Key items cannot be stackable".to_string(),
                ));
            }
            _ => {}
        }

        item.stackable = stackable;
        if stackable {
            if item.stack_limit == 1 {
                item.stack_limit = 99;
            }
        } else {
            item.stack_limit = 1;
        }

        Ok(())
    }

    /// Sets the stack_limit for a stackable item. Must be in range [2, 999].
    /// Only valid when the item is stackable.
    pub fn set_stack_limit(&mut self, id: &ItemId, limit: u32) -> Result<(), CommonError> {
        let item = self.items.get_mut(id).ok_or_else(|| {
            CommonError::ItemValidationError(format!("Item with id '{}' not found", id))
        })?;

        if !item.stackable {
            return Err(CommonError::ItemValidationError(
                "Cannot set stack_limit on a non-stackable item".to_string(),
            ));
        }

        if !(2..=999).contains(&limit) {
            return Err(CommonError::ItemValidationError(
                "Stack limit must be between 2 and 999".to_string(),
            ));
        }

        item.stack_limit = limit;
        Ok(())
    }

    /// Adds a stat modifier to an item. Validates stat_name (1–32 chars, at least 1 non-whitespace),
    /// rejects duplicates (case-sensitive on trimmed name), and enforces max 20 modifiers.
    pub fn add_stat_modifier(
        &mut self,
        id: &ItemId,
        stat_name: &str,
        value: i32,
    ) -> Result<(), CommonError> {
        let trimmed = stat_name.trim();
        if trimmed.is_empty() {
            return Err(CommonError::ItemValidationError(
                "Stat name must contain at least 1 non-whitespace character".to_string(),
            ));
        }
        if trimmed.len() > 32 {
            return Err(CommonError::ItemValidationError(
                "Stat name must not exceed 32 characters".to_string(),
            ));
        }

        let item = self.items.get_mut(id).ok_or_else(|| {
            CommonError::ItemValidationError(format!("Item with id '{}' not found", id))
        })?;

        if item.stat_modifiers.len() >= 20 {
            return Err(CommonError::ItemValidationError(
                "Item cannot have more than 20 stat modifiers".to_string(),
            ));
        }

        if item.stat_modifiers.iter().any(|m| m.stat_name == trimmed) {
            return Err(CommonError::ItemValidationError(format!(
                "Stat modifier '{}' already exists on this item",
                trimmed
            )));
        }

        item.stat_modifiers.push(StatModifier {
            stat_name: trimmed.to_string(),
            value,
        });

        Ok(())
    }

    /// Removes a stat modifier from an item by stat name.
    pub fn remove_stat_modifier(
        &mut self,
        id: &ItemId,
        stat_name: &str,
    ) -> Result<(), CommonError> {
        let trimmed = stat_name.trim();
        let item = self.items.get_mut(id).ok_or_else(|| {
            CommonError::ItemValidationError(format!("Item with id '{}' not found", id))
        })?;

        let pos = item
            .stat_modifiers
            .iter()
            .position(|m| m.stat_name == trimmed)
            .ok_or_else(|| {
                CommonError::ItemValidationError(format!(
                    "Stat modifier '{}' not found on this item",
                    trimmed
                ))
            })?;

        item.stat_modifiers.remove(pos);
        Ok(())
    }

    /// Updates an existing stat modifier's value by stat name.
    pub fn update_stat_modifier(
        &mut self,
        id: &ItemId,
        stat_name: &str,
        value: i32,
    ) -> Result<(), CommonError> {
        let trimmed = stat_name.trim();
        let item = self.items.get_mut(id).ok_or_else(|| {
            CommonError::ItemValidationError(format!("Item with id '{}' not found", id))
        })?;

        let modifier = item
            .stat_modifiers
            .iter_mut()
            .find(|m| m.stat_name == trimmed)
            .ok_or_else(|| {
                CommonError::ItemValidationError(format!(
                    "Stat modifier '{}' not found on this item",
                    trimmed
                ))
            })?;

        modifier.value = value;
        Ok(())
    }

    /// Returns all items sorted case-insensitively by display name.
    pub fn sorted_items(&self) -> Vec<&Item> {
        let mut items: Vec<&Item> = self.items.values().collect();
        items.sort_by(|a, b| {
            a.display_name
                .to_lowercase()
                .cmp(&b.display_name.to_lowercase())
        });
        items
    }

    /// Adds a consumable effect to a consumable item.
    /// Validates potency ≥ 1, enforces max 4 effects, rejects if item is not Consumable.
    pub fn add_consumable_effect(
        &mut self,
        id: &ItemId,
        effect: ConsumableEffect,
    ) -> Result<(), CommonError> {
        let item = self.items.get_mut(id).ok_or_else(|| {
            CommonError::ItemValidationError(format!("Item with id '{}' not found", id))
        })?;

        let effects = match &mut item.category_data {
            ItemCategoryData::Consumable { effects } => effects,
            _ => {
                return Err(CommonError::ItemValidationError(
                    "Item is not a Consumable".to_string(),
                ));
            }
        };

        if effect.potency < 1 {
            return Err(CommonError::ItemValidationError(
                "Consumable effect potency must be at least 1".to_string(),
            ));
        }

        if effects.len() >= 4 {
            return Err(CommonError::ItemValidationError(
                "Consumable item cannot have more than 4 effects".to_string(),
            ));
        }

        effects.push(effect);
        Ok(())
    }

    /// Removes a consumable effect at the given index.
    /// Rejects removal of the last effect.
    pub fn remove_consumable_effect(
        &mut self,
        id: &ItemId,
        index: usize,
    ) -> Result<(), CommonError> {
        let item = self.items.get_mut(id).ok_or_else(|| {
            CommonError::ItemValidationError(format!("Item with id '{}' not found", id))
        })?;

        let effects = match &mut item.category_data {
            ItemCategoryData::Consumable { effects } => effects,
            _ => {
                return Err(CommonError::ItemValidationError(
                    "Item is not a Consumable".to_string(),
                ));
            }
        };

        if effects.len() <= 1 {
            return Err(CommonError::ItemValidationError(
                "Cannot remove the last consumable effect".to_string(),
            ));
        }

        if index >= effects.len() {
            return Err(CommonError::ItemValidationError(format!(
                "Effect index {} is out of bounds",
                index
            )));
        }

        effects.remove(index);
        Ok(())
    }

    /// Updates a consumable effect at the given index.
    /// Validates potency ≥ 1.
    pub fn update_consumable_effect(
        &mut self,
        id: &ItemId,
        index: usize,
        effect: ConsumableEffect,
    ) -> Result<(), CommonError> {
        let item = self.items.get_mut(id).ok_or_else(|| {
            CommonError::ItemValidationError(format!("Item with id '{}' not found", id))
        })?;

        let effects = match &mut item.category_data {
            ItemCategoryData::Consumable { effects } => effects,
            _ => {
                return Err(CommonError::ItemValidationError(
                    "Item is not a Consumable".to_string(),
                ));
            }
        };

        if effect.potency < 1 {
            return Err(CommonError::ItemValidationError(
                "Consumable effect potency must be at least 1".to_string(),
            ));
        }

        if index >= effects.len() {
            return Err(CommonError::ItemValidationError(format!(
                "Effect index {} is out of bounds",
                index
            )));
        }

        effects[index] = effect;
        Ok(())
    }

    /// Returns items filtered by category (if Some) and sorted case-insensitively by display name.
    pub fn filtered_items(&self, category: Option<ItemCategory>) -> Vec<&Item> {
        let mut items: Vec<&Item> = match category {
            Some(cat) => self
                .items
                .values()
                .filter(|i| i.category() == cat)
                .collect(),
            None => self.items.values().collect(),
        };
        items.sort_by(|a, b| {
            a.display_name
                .to_lowercase()
                .cmp(&b.display_name.to_lowercase())
        });
        items
    }
}

/// Formats a stat modifier value with sign prefix: "+N", "-N", or "+0".
pub fn format_modifier_value(value: i32) -> String {
    if value >= 0 {
        format!("+{}", value)
    } else {
        format!("{}", value)
    }
}
