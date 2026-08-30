use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::ability::AbilityId;
use crate::error::CommonError;
use crate::item::ItemId;

/// The type of visual asset on a character.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VisualAssetType {
    Spritesheet,
    FacePortrait,
    StatusPortrait,
}

/// Optional visual asset file paths for a character.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct VisualAssets {
    pub spritesheet: Option<String>,
    pub face_portrait: Option<String>,
    pub status_portrait: Option<String>,
}

/// Type alias for character identifiers (UUID v4 strings).
pub type CharacterId = String;

/// A single stat on a character.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stat {
    pub name: String,
    pub base_value: u32,
    pub growth_value: u32,
}

/// A learnable ability entry linking a character to an ability at a required level.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LearnableAbility {
    pub ability_id: AbilityId,
    pub required_level: u32,
}

/// A playable character with stats and progression.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Character {
    pub id: CharacterId,
    pub display_name: String,
    pub stats: Vec<Stat>,
    #[serde(default)]
    pub learnable_abilities: Vec<LearnableAbility>,
    #[serde(default)]
    pub visual_assets: VisualAssets,
    #[serde(default)]
    pub starting_equipment: Vec<ItemId>,
}

/// The set of all available optional stat names.
pub const OPTIONAL_STATS: &[&str] = &[
    "Strength",
    "Stamina",
    "Speed",
    "Luck",
    "MP",
    "Wisdom",
    "Intelligence",
];

/// Required stats that every character must have: (name, base_value, growth_value).
pub const REQUIRED_STATS: &[(&str, u32, u32)] = &[("HP", 10, 5), ("Level", 1, 0)];

/// Project-level collection of characters.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CharacterRegistry {
    pub characters: HashMap<CharacterId, Character>,
}

impl CharacterRegistry {
    /// Creates a new character with the given display name.
    ///
    /// Validates the name (trimmed, 1–64 chars, at least 1 non-whitespace),
    /// generates a UUID v4 identifier, and initializes with default required stats.
    pub fn create_character(&mut self, name: &str) -> Result<CharacterId, CommonError> {
        let trimmed = name.trim();
        Self::validate_display_name(trimmed)?;

        let id = Uuid::new_v4().to_string();
        let stats = REQUIRED_STATS
            .iter()
            .map(|(stat_name, base, growth)| Stat {
                name: stat_name.to_string(),
                base_value: *base,
                growth_value: *growth,
            })
            .collect();

        let character = Character {
            id: id.clone(),
            display_name: trimmed.to_string(),
            stats,
            learnable_abilities: Vec::new(),
            visual_assets: VisualAssets::default(),
            starting_equipment: Vec::new(),
        };

        self.characters.insert(id.clone(), character);
        Ok(id)
    }

    /// Removes a character from the registry by ID.
    pub fn delete_character(&mut self, id: &CharacterId) -> Result<(), CommonError> {
        if self.characters.remove(id).is_none() {
            return Err(CommonError::CharacterValidationError(format!(
                "Character not found: {id}"
            )));
        }
        Ok(())
    }

    /// Renames an existing character.
    ///
    /// Validates the new name (trimmed, 1–64 chars, at least 1 non-whitespace).
    pub fn rename_character(
        &mut self,
        id: &CharacterId,
        new_name: &str,
    ) -> Result<(), CommonError> {
        let trimmed = new_name.trim();
        Self::validate_display_name(trimmed)?;

        let character = self.characters.get_mut(id).ok_or_else(|| {
            CommonError::CharacterValidationError(format!("Character not found: {id}"))
        })?;

        character.display_name = trimmed.to_string();
        Ok(())
    }

    /// Adds an optional stat to a character with default values (base 0, growth 0).
    ///
    /// Rejects duplicate stat names.
    pub fn add_stat(&mut self, id: &CharacterId, stat_name: &str) -> Result<(), CommonError> {
        if stat_name.is_empty() || stat_name.len() > 32 {
            return Err(CommonError::CharacterValidationError(
                "Stat name must be between 1 and 32 characters".to_string(),
            ));
        }

        let character = self.characters.get_mut(id).ok_or_else(|| {
            CommonError::CharacterValidationError(format!("Character not found: {id}"))
        })?;

        if character.stats.iter().any(|s| s.name == stat_name) {
            return Err(CommonError::CharacterValidationError(format!(
                "Duplicate stat name: {stat_name}"
            )));
        }

        character.stats.push(Stat {
            name: stat_name.to_string(),
            base_value: 0,
            growth_value: 0,
        });

        Ok(())
    }

    /// Removes a stat from a character.
    ///
    /// Required stats (HP, Level) cannot be removed.
    pub fn remove_stat(&mut self, id: &CharacterId, stat_name: &str) -> Result<(), CommonError> {
        if REQUIRED_STATS.iter().any(|(name, _, _)| *name == stat_name) {
            return Err(CommonError::CharacterValidationError(format!(
                "Cannot remove required stat: {stat_name}"
            )));
        }

        let character = self.characters.get_mut(id).ok_or_else(|| {
            CommonError::CharacterValidationError(format!("Character not found: {id}"))
        })?;

        let original_len = character.stats.len();
        character.stats.retain(|s| s.name != stat_name);

        if character.stats.len() == original_len {
            return Err(CommonError::CharacterValidationError(format!(
                "Stat not found: {stat_name}"
            )));
        }

        Ok(())
    }

    /// Updates the base and growth values of an existing stat on a character.
    pub fn update_stat(
        &mut self,
        id: &CharacterId,
        stat_name: &str,
        base: u32,
        growth: u32,
    ) -> Result<(), CommonError> {
        let character = self.characters.get_mut(id).ok_or_else(|| {
            CommonError::CharacterValidationError(format!("Character not found: {id}"))
        })?;

        let stat = character
            .stats
            .iter_mut()
            .find(|s| s.name == stat_name)
            .ok_or_else(|| {
                CommonError::CharacterValidationError(format!("Stat not found: {stat_name}"))
            })?;

        stat.base_value = base;
        stat.growth_value = growth;
        Ok(())
    }

    /// Computes the effective stat value at a given level.
    ///
    /// Formula: `base_value + growth_value * (level - 1)`, saturating at `u32::MAX`.
    pub fn compute_stat_value(stat: &Stat, level: u32) -> u32 {
        let level_factor = level.saturating_sub(1);
        stat.base_value
            .saturating_add(stat.growth_value.saturating_mul(level_factor))
    }

    /// Returns characters sorted case-insensitively by display name.
    pub fn sorted_characters(&self) -> Vec<&Character> {
        let mut chars: Vec<&Character> = self.characters.values().collect();
        chars.sort_by(|a, b| {
            a.display_name
                .to_lowercase()
                .cmp(&b.display_name.to_lowercase())
        });
        chars
    }

    /// Adds a learnable ability to a character.
    ///
    /// Validates level is 1–99, rejects duplicate ability IDs, and enforces a max of 20 entries.
    pub fn add_learnable_ability(
        &mut self,
        id: &CharacterId,
        ability_id: AbilityId,
        level: u32,
    ) -> Result<(), CommonError> {
        let clamped_level = level.clamp(1, 99);

        let character = self.characters.get_mut(id).ok_or_else(|| {
            CommonError::CharacterValidationError(format!("Character not found: {id}"))
        })?;

        if character
            .learnable_abilities
            .iter()
            .any(|la| la.ability_id == ability_id)
        {
            return Err(CommonError::CharacterValidationError(format!(
                "Ability already assigned: {ability_id}"
            )));
        }

        if character.learnable_abilities.len() >= 20 {
            return Err(CommonError::CharacterValidationError(
                "Cannot have more than 20 learnable abilities".to_string(),
            ));
        }

        character.learnable_abilities.push(LearnableAbility {
            ability_id,
            required_level: clamped_level,
        });

        Ok(())
    }

    /// Removes a learnable ability from a character by ability ID.
    pub fn remove_learnable_ability(
        &mut self,
        id: &CharacterId,
        ability_id: &AbilityId,
    ) -> Result<(), CommonError> {
        let character = self.characters.get_mut(id).ok_or_else(|| {
            CommonError::CharacterValidationError(format!("Character not found: {id}"))
        })?;

        let original_len = character.learnable_abilities.len();
        character
            .learnable_abilities
            .retain(|la| la.ability_id != *ability_id);

        if character.learnable_abilities.len() == original_len {
            return Err(CommonError::CharacterValidationError(format!(
                "Learnable ability not found: {ability_id}"
            )));
        }

        Ok(())
    }

    /// Updates the required level of a learnable ability, clamping to 1–99.
    pub fn update_learnable_ability_level(
        &mut self,
        id: &CharacterId,
        ability_id: &AbilityId,
        new_level: u32,
    ) -> Result<(), CommonError> {
        let clamped_level = new_level.clamp(1, 99);

        let character = self.characters.get_mut(id).ok_or_else(|| {
            CommonError::CharacterValidationError(format!("Character not found: {id}"))
        })?;

        let entry = character
            .learnable_abilities
            .iter_mut()
            .find(|la| la.ability_id == *ability_id)
            .ok_or_else(|| {
                CommonError::CharacterValidationError(format!(
                    "Learnable ability not found: {ability_id}"
                ))
            })?;

        entry.required_level = clamped_level;
        Ok(())
    }

    /// Sets a visual asset path on a character.
    ///
    /// Trims the path; if empty after trimming, sets to None.
    /// Otherwise truncates to 260 characters and stores.
    pub fn set_visual_asset(
        &mut self,
        id: &CharacterId,
        asset_type: VisualAssetType,
        path: &str,
    ) -> Result<(), CommonError> {
        let character = self.characters.get_mut(id).ok_or_else(|| {
            CommonError::CharacterValidationError(format!("Character not found: {id}"))
        })?;

        let trimmed = path.trim();
        let value = if trimmed.is_empty() {
            None
        } else {
            let truncated: String = trimmed.chars().take(260).collect();
            Some(truncated)
        };

        match asset_type {
            VisualAssetType::Spritesheet => character.visual_assets.spritesheet = value,
            VisualAssetType::FacePortrait => character.visual_assets.face_portrait = value,
            VisualAssetType::StatusPortrait => character.visual_assets.status_portrait = value,
        }

        Ok(())
    }

    /// Clears a visual asset path on a character, setting it to None.
    pub fn clear_visual_asset(
        &mut self,
        id: &CharacterId,
        asset_type: VisualAssetType,
    ) -> Result<(), CommonError> {
        let character = self.characters.get_mut(id).ok_or_else(|| {
            CommonError::CharacterValidationError(format!("Character not found: {id}"))
        })?;

        match asset_type {
            VisualAssetType::Spritesheet => character.visual_assets.spritesheet = None,
            VisualAssetType::FacePortrait => character.visual_assets.face_portrait = None,
            VisualAssetType::StatusPortrait => character.visual_assets.status_portrait = None,
        }

        Ok(())
    }

    /// Adds a starting equipment item to a character.
    ///
    /// Trims the item ID, validates non-empty, rejects duplicates, and enforces a max of 20 entries.
    pub fn add_starting_equipment(
        &mut self,
        id: &CharacterId,
        item_id: &str,
    ) -> Result<(), CommonError> {
        let trimmed = item_id.trim();

        if trimmed.is_empty() {
            return Err(CommonError::CharacterValidationError(
                "Item ID must not be empty or whitespace-only".to_string(),
            ));
        }

        let character = self.characters.get_mut(id).ok_or_else(|| {
            CommonError::CharacterValidationError(format!("Character not found: {id}"))
        })?;

        if character.starting_equipment.iter().any(|e| e == trimmed) {
            return Err(CommonError::CharacterValidationError(format!(
                "Item already assigned: {trimmed}"
            )));
        }

        if character.starting_equipment.len() >= 20 {
            return Err(CommonError::CharacterValidationError(
                "Cannot have more than 20 starting equipment items".to_string(),
            ));
        }

        character.starting_equipment.push(trimmed.to_string());
        Ok(())
    }

    /// Removes a starting equipment item from a character by item ID.
    pub fn remove_starting_equipment(
        &mut self,
        id: &CharacterId,
        item_id: &str,
    ) -> Result<(), CommonError> {
        let character = self.characters.get_mut(id).ok_or_else(|| {
            CommonError::CharacterValidationError(format!("Character not found: {id}"))
        })?;

        let original_len = character.starting_equipment.len();
        character.starting_equipment.retain(|e| e != item_id);

        if character.starting_equipment.len() == original_len {
            return Err(CommonError::CharacterValidationError(format!(
                "Starting equipment item not found: {item_id}"
            )));
        }

        Ok(())
    }

    /// Validates a display name.
    fn validate_display_name(trimmed: &str) -> Result<(), CommonError> {
        if trimmed.is_empty() {
            return Err(CommonError::CharacterValidationError(
                "Display name must not be empty or whitespace-only".to_string(),
            ));
        }
        if trimmed.len() > 64 {
            return Err(CommonError::CharacterValidationError(
                "Display name must not exceed 64 characters".to_string(),
            ));
        }
        if !trimmed.contains(|c: char| !c.is_whitespace()) {
            return Err(CommonError::CharacterValidationError(
                "Display name must contain at least one non-whitespace character".to_string(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_face_portrait_trims_whitespace() {
        let mut registry = CharacterRegistry::default();
        let id = registry.create_character("TestChar").unwrap();

        registry
            .set_visual_asset(
                &id,
                VisualAssetType::FacePortrait,
                "  /path/to/portrait.png  ",
            )
            .unwrap();

        let character = registry.characters.get(&id).unwrap();
        assert_eq!(
            character.visual_assets.face_portrait,
            Some("/path/to/portrait.png".to_string())
        );
    }

    #[test]
    fn set_face_portrait_truncates_to_260_chars() {
        let mut registry = CharacterRegistry::default();
        let id = registry.create_character("TestChar").unwrap();

        let long_path = "a".repeat(300);
        registry
            .set_visual_asset(&id, VisualAssetType::FacePortrait, &long_path)
            .unwrap();

        let character = registry.characters.get(&id).unwrap();
        let stored = character.visual_assets.face_portrait.as_ref().unwrap();
        assert_eq!(stored.len(), 260);
    }
}
