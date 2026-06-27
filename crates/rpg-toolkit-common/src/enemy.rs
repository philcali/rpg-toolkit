use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::ability::AbilityId;
use crate::element::Element;
use crate::error::CommonError;
use crate::item::ItemId;

pub type EnemyId = String;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EnemyStat {
    pub name: String,
    pub base_value: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ItemDrop {
    pub item_id: ItemId,
    pub drop_chance: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DefeatReward {
    pub exp: u32,
    pub gold: u32,
    pub item_drops: Vec<ItemDrop>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CarriedItem {
    pub item_id: ItemId,
    pub obtain_chance: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ElementalModifier {
    pub element: Element,
    pub multiplier: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Enemy {
    pub id: EnemyId,
    pub display_name: String,
    pub description: String,
    pub stats: Vec<EnemyStat>,
    pub defeat_rewards: DefeatReward,
    pub carried_items: Vec<CarriedItem>,
    pub elemental_modifiers: Vec<ElementalModifier>,
    pub abilities: Vec<AbilityId>,
    #[serde(default)]
    pub portrait: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct EnemyRegistry {
    pub enemies: HashMap<EnemyId, Enemy>,
}

impl EnemyRegistry {
    /// Creates a new enemy with the given display name.
    ///
    /// Validates the name (trimmed, 1–64 chars, at least 1 non-whitespace),
    /// generates a UUID v4 identifier, and initializes with default stats.
    pub fn create_enemy(&mut self, name: &str) -> Result<EnemyId, CommonError> {
        let trimmed = validate_display_name(name)?;

        let id = Uuid::new_v4().to_string();
        let stats = vec![
            EnemyStat {
                name: "HP".to_string(),
                base_value: 10,
            },
            EnemyStat {
                name: "Attack".to_string(),
                base_value: 5,
            },
            EnemyStat {
                name: "Defense".to_string(),
                base_value: 5,
            },
            EnemyStat {
                name: "Speed".to_string(),
                base_value: 5,
            },
        ];

        let enemy = Enemy {
            id: id.clone(),
            display_name: trimmed,
            description: String::new(),
            stats,
            defeat_rewards: DefeatReward {
                exp: 0,
                gold: 0,
                item_drops: vec![],
            },
            carried_items: vec![],
            elemental_modifiers: vec![],
            abilities: vec![],
            portrait: None,
        };

        self.enemies.insert(id.clone(), enemy);
        Ok(id)
    }

    /// Removes an enemy from the registry by ID.
    pub fn delete_enemy(&mut self, id: &EnemyId) -> Result<(), CommonError> {
        if self.enemies.remove(id).is_none() {
            return Err(CommonError::EnemyValidationError(format!(
                "Enemy not found: {id}"
            )));
        }
        Ok(())
    }

    /// Renames an existing enemy.
    ///
    /// Validates the new name (trimmed, 1–64 chars, at least 1 non-whitespace).
    pub fn rename_enemy(&mut self, id: &EnemyId, new_name: &str) -> Result<(), CommonError> {
        let trimmed = validate_display_name(new_name)?;

        let enemy = self
            .enemies
            .get_mut(id)
            .ok_or_else(|| CommonError::EnemyValidationError(format!("Enemy not found: {id}")))?;

        enemy.display_name = trimmed;
        Ok(())
    }

    /// Updates the description of an existing enemy.
    ///
    /// Truncates to 256 Unicode codepoints.
    pub fn update_description(&mut self, id: &EnemyId, desc: &str) -> Result<(), CommonError> {
        let enemy = self
            .enemies
            .get_mut(id)
            .ok_or_else(|| CommonError::EnemyValidationError(format!("Enemy not found: {id}")))?;

        enemy.description = desc.chars().take(256).collect::<String>();
        Ok(())
    }

    /// Sets the portrait file path for an existing enemy.
    ///
    /// Trims the path, validates it is non-empty after trimming, and truncates
    /// to 260 Unicode codepoints.
    pub fn set_portrait(&mut self, id: &EnemyId, path: &str) -> Result<(), CommonError> {
        let trimmed = path.trim();
        if trimmed.is_empty() {
            return Err(CommonError::EnemyValidationError(
                "Portrait path must not be empty or whitespace-only".to_string(),
            ));
        }
        let truncated: String = trimmed.chars().take(260).collect();
        let enemy = self
            .enemies
            .get_mut(id)
            .ok_or_else(|| CommonError::EnemyValidationError(format!("Enemy not found: {id}")))?;
        enemy.portrait = Some(truncated);
        Ok(())
    }

    /// Clears the portrait file path for an existing enemy, setting it to None.
    pub fn clear_portrait(&mut self, id: &EnemyId) -> Result<(), CommonError> {
        let enemy = self
            .enemies
            .get_mut(id)
            .ok_or_else(|| CommonError::EnemyValidationError(format!("Enemy not found: {id}")))?;
        enemy.portrait = None;
        Ok(())
    }

    /// Adds a new stat to an existing enemy.
    ///
    /// Trims the name, validates 1–32 chars, checks for duplicates (case-sensitive),
    /// enforces max 20 stats, and appends with base_value 0.
    pub fn add_stat(&mut self, id: &EnemyId, stat_name: &str) -> Result<(), CommonError> {
        let trimmed = stat_name.trim().to_string();
        if trimmed.is_empty() || trimmed.chars().count() > 32 {
            return Err(CommonError::EnemyValidationError(
                "Stat name must be between 1 and 32 characters".to_string(),
            ));
        }
        let enemy = self
            .enemies
            .get_mut(id)
            .ok_or_else(|| CommonError::EnemyValidationError(format!("Enemy not found: {id}")))?;
        if enemy.stats.iter().any(|s| s.name == trimmed) {
            return Err(CommonError::EnemyValidationError(format!(
                "Duplicate stat name: {trimmed}"
            )));
        }
        if enemy.stats.len() >= 20 {
            return Err(CommonError::EnemyValidationError(
                "Enemy cannot have more than 20 stats".to_string(),
            ));
        }
        enemy.stats.push(EnemyStat {
            name: trimmed,
            base_value: 0,
        });
        Ok(())
    }

    /// Removes a stat from an existing enemy.
    ///
    /// The "HP" stat cannot be removed.
    pub fn remove_stat(&mut self, id: &EnemyId, stat_name: &str) -> Result<(), CommonError> {
        let trimmed = stat_name.trim();
        if trimmed == "HP" {
            return Err(CommonError::EnemyValidationError(
                "Cannot remove required stat: HP".to_string(),
            ));
        }
        let enemy = self
            .enemies
            .get_mut(id)
            .ok_or_else(|| CommonError::EnemyValidationError(format!("Enemy not found: {id}")))?;
        let pos = enemy
            .stats
            .iter()
            .position(|s| s.name == trimmed)
            .ok_or_else(|| {
                CommonError::EnemyValidationError(format!("Stat not found: {trimmed}"))
            })?;
        enemy.stats.remove(pos);
        Ok(())
    }

    /// Updates the base_value of an existing stat on an enemy.
    pub fn update_stat(
        &mut self,
        id: &EnemyId,
        stat_name: &str,
        base_value: u32,
    ) -> Result<(), CommonError> {
        let trimmed = stat_name.trim();
        let enemy = self
            .enemies
            .get_mut(id)
            .ok_or_else(|| CommonError::EnemyValidationError(format!("Enemy not found: {id}")))?;
        let stat = enemy
            .stats
            .iter_mut()
            .find(|s| s.name == trimmed)
            .ok_or_else(|| {
                CommonError::EnemyValidationError(format!("Stat not found: {trimmed}"))
            })?;
        stat.base_value = base_value;
        Ok(())
    }

    /// Updates the experience reward for defeating an enemy.
    pub fn update_exp(&mut self, id: &EnemyId, exp: u32) -> Result<(), CommonError> {
        let enemy = self
            .enemies
            .get_mut(id)
            .ok_or_else(|| CommonError::EnemyValidationError(format!("Enemy not found: {id}")))?;
        enemy.defeat_rewards.exp = exp;
        Ok(())
    }

    /// Updates the gold reward for defeating an enemy.
    pub fn update_gold(&mut self, id: &EnemyId, gold: u32) -> Result<(), CommonError> {
        let enemy = self
            .enemies
            .get_mut(id)
            .ok_or_else(|| CommonError::EnemyValidationError(format!("Enemy not found: {id}")))?;
        enemy.defeat_rewards.gold = gold;
        Ok(())
    }

    /// Adds an item drop to an enemy's defeat rewards.
    ///
    /// Validates non-empty item_id (after trimming), drop_chance in 0.0–1.0 inclusive,
    /// and enforces a maximum of 10 item drops.
    pub fn add_item_drop(
        &mut self,
        id: &EnemyId,
        item_id: &str,
        drop_chance: f64,
    ) -> Result<(), CommonError> {
        let trimmed_item_id = item_id.trim().to_string();
        if trimmed_item_id.is_empty() {
            return Err(CommonError::EnemyValidationError(
                "Item ID must not be empty".to_string(),
            ));
        }
        if !(0.0..=1.0).contains(&drop_chance) {
            return Err(CommonError::EnemyValidationError(
                "Drop chance must be between 0.0 and 1.0".to_string(),
            ));
        }
        let enemy = self
            .enemies
            .get_mut(id)
            .ok_or_else(|| CommonError::EnemyValidationError(format!("Enemy not found: {id}")))?;
        if enemy.defeat_rewards.item_drops.len() >= 10 {
            return Err(CommonError::EnemyValidationError(
                "Enemy cannot have more than 10 item drops".to_string(),
            ));
        }
        enemy.defeat_rewards.item_drops.push(ItemDrop {
            item_id: trimmed_item_id,
            drop_chance,
        });
        Ok(())
    }

    /// Removes an item drop from an enemy's defeat rewards by index.
    pub fn remove_item_drop(&mut self, id: &EnemyId, index: usize) -> Result<(), CommonError> {
        let enemy = self
            .enemies
            .get_mut(id)
            .ok_or_else(|| CommonError::EnemyValidationError(format!("Enemy not found: {id}")))?;
        if index >= enemy.defeat_rewards.item_drops.len() {
            return Err(CommonError::EnemyValidationError(format!(
                "Item drop index out of bounds: {index}"
            )));
        }
        enemy.defeat_rewards.item_drops.remove(index);
        Ok(())
    }

    /// Adds a carried item to an enemy.
    ///
    /// Validates non-empty item_id (after trimming), obtain_chance in 0.0–1.0 inclusive,
    /// and enforces a maximum of 8 carried items.
    pub fn add_carried_item(
        &mut self,
        id: &EnemyId,
        item_id: &str,
        obtain_chance: f64,
    ) -> Result<(), CommonError> {
        let trimmed_item_id = item_id.trim().to_string();
        if trimmed_item_id.is_empty() {
            return Err(CommonError::EnemyValidationError(
                "Item ID must not be empty".to_string(),
            ));
        }
        if !(0.0..=1.0).contains(&obtain_chance) {
            return Err(CommonError::EnemyValidationError(
                "Obtain chance must be between 0.0 and 1.0".to_string(),
            ));
        }
        let enemy = self
            .enemies
            .get_mut(id)
            .ok_or_else(|| CommonError::EnemyValidationError(format!("Enemy not found: {id}")))?;
        if enemy.carried_items.len() >= 8 {
            return Err(CommonError::EnemyValidationError(
                "Enemy cannot have more than 8 carried items".to_string(),
            ));
        }
        enemy.carried_items.push(CarriedItem {
            item_id: trimmed_item_id,
            obtain_chance,
        });
        Ok(())
    }

    /// Removes a carried item from an enemy by index.
    pub fn remove_carried_item(&mut self, id: &EnemyId, index: usize) -> Result<(), CommonError> {
        let enemy = self
            .enemies
            .get_mut(id)
            .ok_or_else(|| CommonError::EnemyValidationError(format!("Enemy not found: {id}")))?;
        if index >= enemy.carried_items.len() {
            return Err(CommonError::EnemyValidationError(format!(
                "Carried item index out of bounds: {index}"
            )));
        }
        enemy.carried_items.remove(index);
        Ok(())
    }

    /// Adds an elemental modifier to an enemy.
    ///
    /// Validates multiplier >= 0.0 and checks for duplicate elements.
    pub fn add_elemental_modifier(
        &mut self,
        id: &EnemyId,
        element: Element,
        multiplier: f64,
    ) -> Result<(), CommonError> {
        if multiplier < 0.0 {
            return Err(CommonError::EnemyValidationError(
                "Multiplier must be greater than or equal to 0.0".to_string(),
            ));
        }
        let enemy = self
            .enemies
            .get_mut(id)
            .ok_or_else(|| CommonError::EnemyValidationError(format!("Enemy not found: {id}")))?;
        if enemy
            .elemental_modifiers
            .iter()
            .any(|m| m.element == element)
        {
            return Err(CommonError::EnemyValidationError(format!(
                "Elemental modifier for {element:?} already exists"
            )));
        }
        enemy.elemental_modifiers.push(ElementalModifier {
            element,
            multiplier,
        });
        Ok(())
    }

    /// Updates the multiplier of an existing elemental modifier on an enemy.
    ///
    /// Validates multiplier >= 0.0 and that the element exists.
    pub fn update_elemental_modifier(
        &mut self,
        id: &EnemyId,
        element: Element,
        multiplier: f64,
    ) -> Result<(), CommonError> {
        if multiplier < 0.0 {
            return Err(CommonError::EnemyValidationError(
                "Multiplier must be greater than or equal to 0.0".to_string(),
            ));
        }
        let enemy = self
            .enemies
            .get_mut(id)
            .ok_or_else(|| CommonError::EnemyValidationError(format!("Enemy not found: {id}")))?;
        let modifier = enemy
            .elemental_modifiers
            .iter_mut()
            .find(|m| m.element == element)
            .ok_or_else(|| {
                CommonError::EnemyValidationError(format!(
                    "Elemental modifier for {element:?} not found"
                ))
            })?;
        modifier.multiplier = multiplier;
        Ok(())
    }

    /// Removes an elemental modifier from an enemy by element.
    pub fn remove_elemental_modifier(
        &mut self,
        id: &EnemyId,
        element: Element,
    ) -> Result<(), CommonError> {
        let enemy = self
            .enemies
            .get_mut(id)
            .ok_or_else(|| CommonError::EnemyValidationError(format!("Enemy not found: {id}")))?;
        let pos = enemy
            .elemental_modifiers
            .iter()
            .position(|m| m.element == element)
            .ok_or_else(|| {
                CommonError::EnemyValidationError(format!(
                    "Elemental modifier for {element:?} not found"
                ))
            })?;
        enemy.elemental_modifiers.remove(pos);
        Ok(())
    }

    /// Adds an ability to an enemy.
    ///
    /// Validates non-empty ability_id (after trimming), checks for duplicates (case-sensitive),
    /// and enforces a maximum of 10 abilities.
    pub fn add_ability(&mut self, id: &EnemyId, ability_id: &str) -> Result<(), CommonError> {
        let trimmed = ability_id.trim().to_string();
        if trimmed.is_empty() {
            return Err(CommonError::EnemyValidationError(
                "Ability ID must not be empty".to_string(),
            ));
        }
        let enemy = self
            .enemies
            .get_mut(id)
            .ok_or_else(|| CommonError::EnemyValidationError(format!("Enemy not found: {id}")))?;
        if enemy.abilities.iter().any(|a| a == &trimmed) {
            return Err(CommonError::EnemyValidationError(format!(
                "Ability already assigned: {trimmed}"
            )));
        }
        if enemy.abilities.len() >= 10 {
            return Err(CommonError::EnemyValidationError(
                "Enemy cannot have more than 10 abilities".to_string(),
            ));
        }
        enemy.abilities.push(trimmed);
        Ok(())
    }

    /// Removes an ability from an enemy by ability ID.
    pub fn remove_ability(&mut self, id: &EnemyId, ability_id: &str) -> Result<(), CommonError> {
        let trimmed = ability_id.trim();
        let enemy = self
            .enemies
            .get_mut(id)
            .ok_or_else(|| CommonError::EnemyValidationError(format!("Enemy not found: {id}")))?;
        let pos = enemy
            .abilities
            .iter()
            .position(|a| a == trimmed)
            .ok_or_else(|| {
                CommonError::EnemyValidationError(format!("Ability not found: {trimmed}"))
            })?;
        enemy.abilities.remove(pos);
        Ok(())
    }

    /// Returns all enemies sorted case-insensitively by display_name.
    ///
    /// Ties (same lowercased name) are broken by byte-order (lexicographic comparison
    /// of the original display_name).
    pub fn sorted_enemies(&self) -> Vec<&Enemy> {
        let mut enemies: Vec<&Enemy> = self.enemies.values().collect();
        enemies.sort_by(|a, b| {
            let cmp = a
                .display_name
                .to_lowercase()
                .cmp(&b.display_name.to_lowercase());
            if cmp == std::cmp::Ordering::Equal {
                a.display_name.cmp(&b.display_name)
            } else {
                cmp
            }
        });
        enemies
    }

    /// Searches enemies by case-insensitive substring match on display_name.
    ///
    /// If the query is empty or whitespace-only, returns the full sorted listing.
    /// Otherwise, filters enemies whose display_name contains the query (case-insensitive)
    /// and returns them sorted the same way as `sorted_enemies`.
    pub fn search_enemies(&self, query: &str) -> Vec<&Enemy> {
        let trimmed = query.trim();
        if trimmed.is_empty() {
            return self.sorted_enemies();
        }
        let lower_query = trimmed.to_lowercase();
        let mut enemies: Vec<&Enemy> = self
            .enemies
            .values()
            .filter(|e| e.display_name.to_lowercase().contains(&lower_query))
            .collect();
        enemies.sort_by(|a, b| {
            let cmp = a
                .display_name
                .to_lowercase()
                .cmp(&b.display_name.to_lowercase());
            if cmp == std::cmp::Ordering::Equal {
                a.display_name.cmp(&b.display_name)
            } else {
                cmp
            }
        });
        enemies
    }
}

/// Validates a display name for enemies.
///
/// Trims the name, checks it is 1–64 chars, and contains at least 1 non-whitespace character.
fn validate_display_name(name: &str) -> Result<String, CommonError> {
    let trimmed = name.trim();
    if trimmed.is_empty() || !trimmed.chars().any(|c| !c.is_whitespace()) {
        return Err(CommonError::EnemyValidationError(
            "Display name must not be empty or whitespace-only".to_string(),
        ));
    }
    if trimmed.chars().count() > 64 {
        return Err(CommonError::EnemyValidationError(
            "Display name must not exceed 64 characters".to_string(),
        ));
    }
    Ok(trimmed.to_string())
}
