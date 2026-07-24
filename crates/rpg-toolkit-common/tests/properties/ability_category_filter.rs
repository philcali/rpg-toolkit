// Feature: database-editor-enhancements, Property 1: Category filter returns exactly matching abilities

use std::collections::HashMap;

use proptest::collection;
use proptest::prelude::*;

use rpg_toolkit_common::ability::{
    Ability, AbilityCategory, AbilityRegistry, AbilitySource, CostType, TargetType,
};
use rpg_toolkit_common::graphics::EntityGraphics;

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
                    graphics: EntityGraphics::default(),
                }
            },
        )
}

/// Strategy for generating an ability with a specific category.
fn arb_ability_with_category(category: AbilityCategory) -> impl Strategy<Value = Ability> {
    (
        "[a-f0-9]{8}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{12}",
        arb_display_name(),
        arb_description(),
        arb_cost_type(),
        0u32..1000,
        0u32..1000,
        arb_target_type(),
        collection::vec(arb_source(), 0..=10),
    )
        .prop_map(
            move |(
                id,
                display_name,
                description,
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
                    graphics: EntityGraphics::default(),
                }
            },
        )
}

/// Strategy for generating a valid AbilityRegistry with 0-50 abilities.
fn arb_registry() -> impl Strategy<Value = AbilityRegistry> {
    collection::vec(arb_ability(), 0..=50).prop_map(|abilities| {
        let map: HashMap<String, Ability> =
            abilities.into_iter().map(|a| (a.id.clone(), a)).collect();
        AbilityRegistry { abilities: map }
    })
}

/// Strategy for generating a registry that is guaranteed to contain at least one Monster ability.
fn arb_registry_with_monster() -> impl Strategy<Value = AbilityRegistry> {
    (
        collection::vec(arb_ability(), 0..=20),
        collection::vec(arb_ability_with_category(AbilityCategory::Monster), 1..=5),
    )
        .prop_map(|(mixed, monsters)| {
            let mut map: HashMap<String, Ability> =
                mixed.into_iter().map(|a| (a.id.clone(), a)).collect();
            for a in monsters {
                map.insert(a.id.clone(), a);
            }
            AbilityRegistry { abilities: map }
        })
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 100, .. ProptestConfig::default() })]

    /// **Validates: Requirements 1.4, 1.5**
    ///
    /// Property 1: Category filter returns exactly matching abilities.
    /// For any AbilityRegistry, filtered_abilities(Some(category)) returns only abilities
    /// whose category matches, and filtered_abilities(None) returns all abilities.
    #[test]
    fn filtered_abilities_with_category_returns_only_matching(registry in arb_registry()) {
        // Test for each category variant
        let categories = [
            AbilityCategory::Skill,
            AbilityCategory::Spell,
            AbilityCategory::SpecialAction,
            AbilityCategory::Monster,
        ];

        for cat in &categories {
            let filtered = registry.filtered_abilities(Some(*cat));

            // All returned abilities must match the filter category
            for ability in &filtered {
                prop_assert_eq!(
                    ability.category, *cat,
                    "filtered_abilities(Some({:?})) returned an ability with category {:?}",
                    cat, ability.category
                );
            }

            // No matching abilities should be excluded
            let expected_count = registry
                .abilities
                .values()
                .filter(|a| a.category == *cat)
                .count();
            prop_assert_eq!(
                filtered.len(), expected_count,
                "filtered_abilities(Some({:?})) returned {} abilities but expected {}",
                cat, filtered.len(), expected_count
            );
        }
    }

    /// **Validates: Requirements 1.4, 1.5**
    ///
    /// Property 1 (None filter): filtered_abilities(None) returns all abilities in the registry.
    #[test]
    fn filtered_abilities_with_none_returns_all(registry in arb_registry()) {
        let filtered = registry.filtered_abilities(None);

        // Should return all abilities
        prop_assert_eq!(
            filtered.len(), registry.abilities.len(),
            "filtered_abilities(None) returned {} abilities but registry has {}",
            filtered.len(), registry.abilities.len()
        );

        // Every ability in the registry should be present in the result
        for ability in registry.abilities.values() {
            prop_assert!(
                filtered.iter().any(|a| a.id == ability.id),
                "filtered_abilities(None) excluded ability with id {}",
                ability.id
            );
        }
    }

    /// **Validates: Requirements 1.4, 1.5**
    ///
    /// Property 1 (Monster-specific): When a registry contains Monster abilities,
    /// filtered_abilities(Some(Monster)) returns only those Monster abilities and
    /// filtered_abilities(None) includes them.
    #[test]
    fn filtered_abilities_monster_category_works(registry in arb_registry_with_monster()) {
        let monster_filtered = registry.filtered_abilities(Some(AbilityCategory::Monster));
        let all_filtered = registry.filtered_abilities(None);

        // Monster filter should only return Monster abilities
        for ability in &monster_filtered {
            prop_assert_eq!(
                ability.category, AbilityCategory::Monster,
                "Monster filter returned non-Monster ability: {:?}",
                ability.category
            );
        }

        // Monster filter should return ALL Monster abilities
        let expected_monster_count = registry
            .abilities
            .values()
            .filter(|a| a.category == AbilityCategory::Monster)
            .count();
        prop_assert_eq!(
            monster_filtered.len(), expected_monster_count,
            "Monster filter returned {} but expected {}",
            monster_filtered.len(), expected_monster_count
        );

        // At least one Monster ability should exist (guaranteed by strategy)
        prop_assert!(
            !monster_filtered.is_empty(),
            "Registry with monster strategy should contain at least one Monster ability"
        );

        // None filter should include all Monster abilities
        for monster_ability in &monster_filtered {
            prop_assert!(
                all_filtered.iter().any(|a| a.id == monster_ability.id),
                "None filter excluded Monster ability with id {}",
                monster_ability.id
            );
        }
    }
}
