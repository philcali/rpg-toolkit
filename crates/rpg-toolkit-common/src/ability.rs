use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::CommonError;
use crate::item::ItemId;

pub type AbilityId = String;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AbilityCategory {
    Skill,
    Spell,
    SpecialAction,
    Monster,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TargetType {
    SingleAlly,
    AllAllies,
    SingleEnemy,
    AllEnemies,
    SelfTarget,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CostType {
    MP,
    HP,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "source_type")]
pub enum AbilitySource {
    LevelUp { required_level: u32 },
    LearnedFromItem { item_id: ItemId },
    EquipmentGrant { item_id: ItemId },
    AccessoryGrant { item_id: ItemId },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ability {
    pub id: AbilityId,
    pub display_name: String,
    pub description: String,
    pub category: AbilityCategory,
    pub cost_type: CostType,
    pub cost_value: u32,
    pub power: u32,
    pub target_type: TargetType,
    pub sources: Vec<AbilitySource>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AbilityRegistry {
    pub abilities: HashMap<AbilityId, Ability>,
}

/// Validates a display name: must be 1–64 characters after trimming, with at least one non-whitespace.
fn validate_name(name: &str) -> Result<String, CommonError> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(CommonError::AbilityValidationError(
            "Display name must not be empty or whitespace-only".to_string(),
        ));
    }
    if trimmed.len() > 64 {
        return Err(CommonError::AbilityValidationError(
            "Display name must not exceed 64 characters".to_string(),
        ));
    }
    Ok(trimmed.to_string())
}

/// Validates an AbilitySource for correctness.
fn validate_source(source: &AbilitySource) -> Result<(), CommonError> {
    match source {
        AbilitySource::LevelUp { required_level } => {
            if *required_level < 1 {
                return Err(CommonError::AbilityValidationError(
                    "LevelUp required_level must be at least 1".to_string(),
                ));
            }
        }
        AbilitySource::LearnedFromItem { item_id }
        | AbilitySource::EquipmentGrant { item_id }
        | AbilitySource::AccessoryGrant { item_id } => {
            if item_id.trim().is_empty() {
                return Err(CommonError::AbilityValidationError(
                    "Item ID must not be empty".to_string(),
                ));
            }
        }
    }
    Ok(())
}

impl AbilityRegistry {
    /// Creates a new ability with the given name and category, using sensible defaults.
    /// Returns the generated AbilityId on success.
    pub fn create_ability(
        &mut self,
        name: &str,
        category: AbilityCategory,
    ) -> Result<AbilityId, CommonError> {
        let display_name = validate_name(name)?;
        let id = Uuid::new_v4().to_string();

        let ability = Ability {
            id: id.clone(),
            display_name,
            description: String::new(),
            category,
            cost_type: CostType::MP,
            cost_value: 0,
            power: 0,
            target_type: TargetType::SingleEnemy,
            sources: vec![],
        };

        self.abilities.insert(id.clone(), ability);
        Ok(id)
    }

    /// Removes an ability from the registry. Returns an error if the ability is not found.
    pub fn delete_ability(&mut self, id: &AbilityId) -> Result<(), CommonError> {
        if self.abilities.remove(id).is_none() {
            return Err(CommonError::AbilityValidationError(format!(
                "Ability not found: {}",
                id
            )));
        }
        Ok(())
    }

    /// Updates the display name of an ability. Validates the same rules as creation.
    pub fn update_display_name(&mut self, id: &AbilityId, name: &str) -> Result<(), CommonError> {
        let display_name = validate_name(name)?;
        let ability = self.abilities.get_mut(id).ok_or_else(|| {
            CommonError::AbilityValidationError(format!("Ability not found: {}", id))
        })?;
        ability.display_name = display_name;
        Ok(())
    }

    /// Updates the description of an ability, truncating to the first 256 Unicode codepoints.
    pub fn update_description(&mut self, id: &AbilityId, desc: &str) -> Result<(), CommonError> {
        let ability = self.abilities.get_mut(id).ok_or_else(|| {
            CommonError::AbilityValidationError(format!("Ability not found: {}", id))
        })?;
        let truncated: String = desc.chars().take(256).collect();
        ability.description = truncated;
        Ok(())
    }

    /// Updates the category of an ability.
    pub fn update_category(
        &mut self,
        id: &AbilityId,
        category: AbilityCategory,
    ) -> Result<(), CommonError> {
        let ability = self.abilities.get_mut(id).ok_or_else(|| {
            CommonError::AbilityValidationError(format!("Ability not found: {}", id))
        })?;
        ability.category = category;
        Ok(())
    }

    /// Updates the cost type of an ability.
    pub fn update_cost_type(
        &mut self,
        id: &AbilityId,
        cost_type: CostType,
    ) -> Result<(), CommonError> {
        let ability = self.abilities.get_mut(id).ok_or_else(|| {
            CommonError::AbilityValidationError(format!("Ability not found: {}", id))
        })?;
        ability.cost_type = cost_type;
        Ok(())
    }

    /// Updates the target type of an ability.
    pub fn update_target_type(
        &mut self,
        id: &AbilityId,
        target_type: TargetType,
    ) -> Result<(), CommonError> {
        let ability = self.abilities.get_mut(id).ok_or_else(|| {
            CommonError::AbilityValidationError(format!("Ability not found: {}", id))
        })?;
        ability.target_type = target_type;
        Ok(())
    }

    /// Updates the power value of an ability.
    pub fn update_power(&mut self, id: &AbilityId, power: u32) -> Result<(), CommonError> {
        let ability = self.abilities.get_mut(id).ok_or_else(|| {
            CommonError::AbilityValidationError(format!("Ability not found: {}", id))
        })?;
        ability.power = power;
        Ok(())
    }

    /// Updates the cost value of an ability.
    pub fn update_cost_value(
        &mut self,
        id: &AbilityId,
        cost_value: u32,
    ) -> Result<(), CommonError> {
        let ability = self.abilities.get_mut(id).ok_or_else(|| {
            CommonError::AbilityValidationError(format!("Ability not found: {}", id))
        })?;
        ability.cost_value = cost_value;
        Ok(())
    }

    /// Adds a source to an ability. Validates the source and enforces max 10 sources.
    pub fn add_source(&mut self, id: &AbilityId, source: AbilitySource) -> Result<(), CommonError> {
        validate_source(&source)?;
        let ability = self.abilities.get_mut(id).ok_or_else(|| {
            CommonError::AbilityValidationError(format!("Ability not found: {}", id))
        })?;
        if ability.sources.len() >= 10 {
            return Err(CommonError::AbilityValidationError(
                "Ability cannot have more than 10 sources".to_string(),
            ));
        }
        ability.sources.push(source);
        Ok(())
    }

    /// Removes a source from an ability by index. Returns an error if index is out of bounds.
    pub fn remove_source(&mut self, id: &AbilityId, index: usize) -> Result<(), CommonError> {
        let ability = self.abilities.get_mut(id).ok_or_else(|| {
            CommonError::AbilityValidationError(format!("Ability not found: {}", id))
        })?;
        if index >= ability.sources.len() {
            return Err(CommonError::AbilityValidationError(format!(
                "Source index {} is out of bounds",
                index
            )));
        }
        ability.sources.remove(index);
        Ok(())
    }

    /// Returns abilities filtered by category (or all if None), sorted case-insensitively by display_name.
    pub fn filtered_abilities(&self, category: Option<AbilityCategory>) -> Vec<&Ability> {
        let mut results: Vec<&Ability> = self
            .abilities
            .values()
            .filter(|a| match category {
                Some(cat) => a.category == cat,
                None => true,
            })
            .collect();
        results.sort_by(|a, b| {
            a.display_name
                .to_lowercase()
                .cmp(&b.display_name.to_lowercase())
        });
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::collection;
    use proptest::prelude::*;

    // Feature: abilities-editor, Property 9: Serialization round-trip preserves registry equality

    /// Strategy for generating a valid AbilityCategory.
    fn arb_category() -> impl Strategy<Value = AbilityCategory> {
        prop_oneof![
            Just(AbilityCategory::Skill),
            Just(AbilityCategory::Spell),
            Just(AbilityCategory::SpecialAction),
            Just(AbilityCategory::Monster),
        ]
    }

    /// Strategy for generating a valid TargetType.
    fn arb_target_type() -> impl Strategy<Value = TargetType> {
        prop_oneof![
            Just(TargetType::SingleAlly),
            Just(TargetType::AllAllies),
            Just(TargetType::SingleEnemy),
            Just(TargetType::AllEnemies),
            Just(TargetType::SelfTarget),
        ]
    }

    /// Strategy for generating a valid CostType.
    fn arb_cost_type() -> impl Strategy<Value = CostType> {
        prop_oneof![Just(CostType::MP), Just(CostType::HP),]
    }

    /// Strategy for generating a non-empty item_id.
    fn arb_item_id() -> impl Strategy<Value = String> {
        "[a-zA-Z][a-zA-Z0-9_]{0,31}".prop_map(|s| s)
    }

    /// Strategy for generating a valid AbilitySource.
    fn arb_source() -> impl Strategy<Value = AbilitySource> {
        prop_oneof![
            (1u32..=100).prop_map(|lvl| AbilitySource::LevelUp {
                required_level: lvl
            }),
            arb_item_id().prop_map(|id| AbilitySource::LearnedFromItem { item_id: id }),
            arb_item_id().prop_map(|id| AbilitySource::EquipmentGrant { item_id: id }),
            arb_item_id().prop_map(|id| AbilitySource::AccessoryGrant { item_id: id }),
        ]
    }

    /// Strategy for generating a valid display_name (1-64 chars, non-empty after trim).
    fn arb_display_name() -> impl Strategy<Value = String> {
        "[a-zA-Z][a-zA-Z0-9 ]{0,63}".prop_filter("display name must be non-empty after trim", |s| {
            let trimmed = s.trim();
            !trimmed.is_empty() && trimmed.len() <= 64
        })
    }

    /// Strategy for generating a valid description (0-256 chars).
    fn arb_description() -> impl Strategy<Value = String> {
        "[a-zA-Z0-9 .,!?]{0,256}".prop_map(|s| s)
    }

    /// Strategy for generating a valid Ability.
    fn arb_ability() -> impl Strategy<Value = Ability> {
        (
            "[a-f0-9]{8}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{12}",
            arb_display_name(),
            arb_description(),
            arb_category(),
            arb_cost_type(),
            0u32..1000,
            0u32..1000,
            arb_target_type(),
            collection::vec(arb_source(), 0..=10),
        )
            .prop_map(
                |(
                    id,
                    display_name,
                    description,
                    category,
                    cost_type,
                    cost_value,
                    power,
                    target_type,
                    sources,
                )| {
                    Ability {
                        id,
                        display_name,
                        description,
                        category,
                        cost_type,
                        cost_value,
                        power,
                        target_type,
                        sources,
                    }
                },
            )
    }

    /// Strategy for generating a valid AbilityRegistry with 0-50 abilities.
    fn arb_registry() -> impl Strategy<Value = AbilityRegistry> {
        collection::vec(arb_ability(), 0..=50).prop_map(|abilities| {
            let map: HashMap<AbilityId, Ability> =
                abilities.into_iter().map(|a| (a.id.clone(), a)).collect();
            AbilityRegistry { abilities: map }
        })
    }

    proptest! {
        /// **Validates: Requirements 12.1, 12.4**
        ///
        /// Property 9: Serialization round-trip preserves registry equality.
        /// Any valid AbilityRegistry can be serialized to JSON and deserialized back
        /// to an equal value.
        #[test]
        fn serialization_round_trip_preserves_registry_equality(registry in arb_registry()) {
            let json = serde_json::to_string(&registry)
                .expect("serialization should not fail for valid registry");
            let deserialized: AbilityRegistry = serde_json::from_str(&json)
                .expect("deserialization should not fail for valid JSON");
            prop_assert_eq!(registry, deserialized);
        }
    }
}
