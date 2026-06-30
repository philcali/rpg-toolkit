//! Unit tests for ActionQueue reward action processing.
//! Tests cover GiveCurrency, GiveExperience, GiveItem, LearnAbility,
//! AddPartyMember, branch injection, and non-blocking chaining.

use bevy::prelude::*;
use rpg_toolkit_common::{
    AbilityCategory, AbilityRegistry, AbilitySource, Character, CharacterRegistry, CostType,
    EventAction, Item, ItemCategoryData, ItemRegistry, ProjectFile, Rarity, Stat, TargetType,
    TransferDirection,
};
use std::collections::{HashMap, VecDeque};

use crate::events::{MapChanged, PlayerMoved, ShowDialog};
use crate::resources::{
    ActionQueue, CharacterProgress, CharacterProgressState, CurrencyState, GameState,
    InventoryState, PartyState, RendererProjectData, RendererState, WaitingFor,
};
use crate::systems::triggers::advance_action_queue;

/// Creates a minimal Bevy App configured for reward action testing.
fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(AssetPlugin::default());
    app.init_asset::<Image>();

    // Register messages used by advance_action_queue
    app.add_message::<MapChanged>();
    app.add_message::<PlayerMoved>();
    app.add_message::<ShowDialog>();

    // Initialize resources
    app.init_resource::<RendererState>();
    app.init_resource::<GameState>();
    app.init_resource::<CurrencyState>();
    app.init_resource::<InventoryState>();
    app.init_resource::<CharacterProgressState>();
    app.init_resource::<PartyState>();

    // Add the system under test
    app.add_systems(Update, advance_action_queue);

    app
}

/// Creates a minimal RendererProjectData with empty registries.
fn empty_project_data() -> RendererProjectData {
    let project_file = ProjectFile::new(
        HashMap::new(),
        HashMap::new(),
        None,
        HashMap::new(),
        None,
        HashMap::new(),
        HashMap::new(),
        CharacterRegistry::default(),
        ItemRegistry::default(),
        AbilityRegistry::default(),
        rpg_toolkit_common::EnemyRegistry::default(),
    );
    RendererProjectData {
        project_file,
        tileset_textures: HashMap::new(),
        tileset_atlas_layouts: HashMap::new(),
        spritesheet_textures: HashMap::new(),
        spritesheet_atlas_layouts: HashMap::new(),
    }
}

/// Creates a RendererProjectData with a specific item in the registry.
fn project_data_with_item(id: &str, stackable: bool, stack_limit: u32) -> RendererProjectData {
    let mut data = empty_project_data();
    let item = Item {
        id: id.to_string(),
        display_name: id.to_string(),
        description: String::new(),
        category_data: ItemCategoryData::KeyItem,
        value: 0,
        rarity: Rarity::Common,
        stackable,
        stack_limit,
        stat_modifiers: Vec::new(),
        granted_abilities: Vec::new(),
    };
    data.project_file.items.items.insert(id.to_string(), item);
    data
}

/// Creates a RendererProjectData with a specific ability in the registry.
fn project_data_with_ability(ability_id: &str) -> RendererProjectData {
    let mut data = empty_project_data();
    let ability = rpg_toolkit_common::Ability {
        id: ability_id.to_string(),
        display_name: ability_id.to_string(),
        description: String::new(),
        category: AbilityCategory::Skill,
        cost_type: CostType::MP,
        cost_value: 0,
        power: 0,
        target_type: TargetType::SelfTarget,
        sources: vec![AbilitySource::LevelUp { required_level: 1 }],
    };
    data.project_file
        .abilities
        .abilities
        .insert(ability_id.to_string(), ability);
    data
}

/// Creates a RendererProjectData with a specific character in the registry.
fn project_data_with_character(char_id: &str) -> RendererProjectData {
    let mut data = empty_project_data();
    let character = Character {
        id: char_id.to_string(),
        display_name: char_id.to_string(),
        stats: vec![Stat {
            name: "HP".to_string(),
            base_value: 10,
            growth_value: 5,
        }],
        learnable_abilities: Vec::new(),
        visual_assets: Default::default(),
        starting_equipment: Vec::new(),
    };
    data.project_file
        .characters
        .characters
        .insert(char_id.to_string(), character);
    data
}

/// Creates a RendererProjectData with both a character and an ability.
fn project_data_with_character_and_ability(char_id: &str, ability_id: &str) -> RendererProjectData {
    let mut data = project_data_with_ability(ability_id);
    let character = Character {
        id: char_id.to_string(),
        display_name: char_id.to_string(),
        stats: vec![Stat {
            name: "HP".to_string(),
            base_value: 10,
            growth_value: 5,
        }],
        learnable_abilities: Vec::new(),
        visual_assets: Default::default(),
        starting_equipment: Vec::new(),
    };
    data.project_file
        .characters
        .characters
        .insert(char_id.to_string(), character);
    data
}

/// Helper: insert an ActionQueue with given actions.
fn insert_queue(app: &mut App, actions: Vec<EventAction>) {
    app.world_mut().insert_resource(ActionQueue {
        actions: VecDeque::from(actions),
        waiting_for: WaitingFor::Nothing,
    });
}

/// A marker SetState action used to detect branch injection.
fn marker_action(key: &str) -> EventAction {
    EventAction::SetState {
        key: key.to_string(),
        value: "injected".to_string(),
    }
}

// =============================================================================
// GiveCurrency Tests
// =============================================================================

#[test]
fn give_currency_give_adds_to_balance() {
    let mut app = test_app();
    app.insert_resource(empty_project_data());
    app.world_mut().resource_mut::<CurrencyState>().balance = 100;

    insert_queue(
        &mut app,
        vec![EventAction::GiveCurrency {
            amount: 50,
            direction: TransferDirection::Give,
            on_success: vec![],
            on_failure: vec![],
        }],
    );

    app.update();

    let currency = app.world().resource::<CurrencyState>();
    assert_eq!(currency.balance, 150);
    assert!(app.world().get_resource::<ActionQueue>().is_none());
}

#[test]
fn give_currency_give_saturates_at_max() {
    let mut app = test_app();
    app.insert_resource(empty_project_data());
    app.world_mut().resource_mut::<CurrencyState>().balance = u64::MAX - 10;

    insert_queue(
        &mut app,
        vec![EventAction::GiveCurrency {
            amount: 100,
            direction: TransferDirection::Give,
            on_success: vec![],
            on_failure: vec![],
        }],
    );

    app.update();

    let currency = app.world().resource::<CurrencyState>();
    assert_eq!(currency.balance, u64::MAX);
}

#[test]
fn give_currency_take_sufficient_subtracts() {
    let mut app = test_app();
    app.insert_resource(empty_project_data());
    app.world_mut().resource_mut::<CurrencyState>().balance = 200;

    insert_queue(
        &mut app,
        vec![EventAction::GiveCurrency {
            amount: 75,
            direction: TransferDirection::Take,
            on_success: vec![marker_action("success")],
            on_failure: vec![marker_action("failure")],
        }],
    );

    app.update();

    let currency = app.world().resource::<CurrencyState>();
    assert_eq!(currency.balance, 125);
    // on_success branch should have been processed (SetState executed)
    let gs = app.world().resource::<GameState>();
    assert_eq!(gs.flags.get("success"), Some(&"injected".to_string()));
    assert!(!gs.flags.contains_key("failure"));
}

#[test]
fn give_currency_take_insufficient_does_not_modify() {
    let mut app = test_app();
    app.insert_resource(empty_project_data());
    app.world_mut().resource_mut::<CurrencyState>().balance = 30;

    insert_queue(
        &mut app,
        vec![EventAction::GiveCurrency {
            amount: 50,
            direction: TransferDirection::Take,
            on_success: vec![marker_action("success")],
            on_failure: vec![marker_action("failure")],
        }],
    );

    app.update();

    let currency = app.world().resource::<CurrencyState>();
    assert_eq!(currency.balance, 30);
    // on_failure branch should have been processed
    let gs = app.world().resource::<GameState>();
    assert_eq!(gs.flags.get("failure"), Some(&"injected".to_string()));
    assert!(!gs.flags.contains_key("success"));
}

// =============================================================================
// GiveExperience Tests
// =============================================================================

#[test]
fn give_experience_give_single_target() {
    let mut app = test_app();
    app.insert_resource(empty_project_data());

    let mut progress = CharacterProgressState::default();
    progress.characters.insert(
        "hero".to_string(),
        CharacterProgress {
            experience: 100,
            learned_abilities: vec![],
        },
    );
    app.insert_resource(progress);

    insert_queue(
        &mut app,
        vec![EventAction::GiveExperience {
            amount: 50,
            target: Some("hero".to_string()),
            direction: TransferDirection::Give,
            on_success: vec![],
            on_failure: vec![],
        }],
    );

    app.update();

    let state = app.world().resource::<CharacterProgressState>();
    assert_eq!(state.characters["hero"].experience, 150);
}

#[test]
fn give_experience_give_all_party() {
    let mut app = test_app();
    app.insert_resource(empty_project_data());

    let mut progress = CharacterProgressState::default();
    progress.characters.insert(
        "hero".to_string(),
        CharacterProgress {
            experience: 10,
            learned_abilities: vec![],
        },
    );
    progress.characters.insert(
        "mage".to_string(),
        CharacterProgress {
            experience: 20,
            learned_abilities: vec![],
        },
    );
    app.insert_resource(progress);
    app.world_mut().resource_mut::<PartyState>().members =
        vec!["hero".to_string(), "mage".to_string()];

    insert_queue(
        &mut app,
        vec![EventAction::GiveExperience {
            amount: 30,
            target: None,
            direction: TransferDirection::Give,
            on_success: vec![],
            on_failure: vec![],
        }],
    );

    app.update();

    let state = app.world().resource::<CharacterProgressState>();
    assert_eq!(state.characters["hero"].experience, 40);
    assert_eq!(state.characters["mage"].experience, 50);
}

#[test]
fn give_experience_take_single_target_sufficient() {
    let mut app = test_app();
    app.insert_resource(empty_project_data());

    let mut progress = CharacterProgressState::default();
    progress.characters.insert(
        "hero".to_string(),
        CharacterProgress {
            experience: 100,
            learned_abilities: vec![],
        },
    );
    app.insert_resource(progress);

    insert_queue(
        &mut app,
        vec![EventAction::GiveExperience {
            amount: 40,
            target: Some("hero".to_string()),
            direction: TransferDirection::Take,
            on_success: vec![marker_action("success")],
            on_failure: vec![marker_action("failure")],
        }],
    );

    app.update();

    let state = app.world().resource::<CharacterProgressState>();
    assert_eq!(state.characters["hero"].experience, 60);
    let gs = app.world().resource::<GameState>();
    assert_eq!(gs.flags.get("success"), Some(&"injected".to_string()));
    assert!(!gs.flags.contains_key("failure"));
}

#[test]
fn give_experience_take_single_target_insufficient() {
    let mut app = test_app();
    app.insert_resource(empty_project_data());

    let mut progress = CharacterProgressState::default();
    progress.characters.insert(
        "hero".to_string(),
        CharacterProgress {
            experience: 20,
            learned_abilities: vec![],
        },
    );
    app.insert_resource(progress);

    insert_queue(
        &mut app,
        vec![EventAction::GiveExperience {
            amount: 50,
            target: Some("hero".to_string()),
            direction: TransferDirection::Take,
            on_success: vec![marker_action("success")],
            on_failure: vec![marker_action("failure")],
        }],
    );

    app.update();

    let state = app.world().resource::<CharacterProgressState>();
    assert_eq!(state.characters["hero"].experience, 20);
    let gs = app.world().resource::<GameState>();
    assert_eq!(gs.flags.get("failure"), Some(&"injected".to_string()));
    assert!(!gs.flags.contains_key("success"));
}

#[test]
fn give_experience_take_all_party_atomic_sufficient() {
    let mut app = test_app();
    app.insert_resource(empty_project_data());

    let mut progress = CharacterProgressState::default();
    progress.characters.insert(
        "hero".to_string(),
        CharacterProgress {
            experience: 100,
            learned_abilities: vec![],
        },
    );
    progress.characters.insert(
        "mage".to_string(),
        CharacterProgress {
            experience: 80,
            learned_abilities: vec![],
        },
    );
    app.insert_resource(progress);
    app.world_mut().resource_mut::<PartyState>().members =
        vec!["hero".to_string(), "mage".to_string()];

    insert_queue(
        &mut app,
        vec![EventAction::GiveExperience {
            amount: 50,
            target: None,
            direction: TransferDirection::Take,
            on_success: vec![marker_action("success")],
            on_failure: vec![marker_action("failure")],
        }],
    );

    app.update();

    let state = app.world().resource::<CharacterProgressState>();
    assert_eq!(state.characters["hero"].experience, 50);
    assert_eq!(state.characters["mage"].experience, 30);
    let gs = app.world().resource::<GameState>();
    assert_eq!(gs.flags.get("success"), Some(&"injected".to_string()));
}

#[test]
fn give_experience_take_all_party_atomic_insufficient() {
    let mut app = test_app();
    app.insert_resource(empty_project_data());

    let mut progress = CharacterProgressState::default();
    progress.characters.insert(
        "hero".to_string(),
        CharacterProgress {
            experience: 100,
            learned_abilities: vec![],
        },
    );
    progress.characters.insert(
        "mage".to_string(),
        CharacterProgress {
            experience: 20,
            learned_abilities: vec![],
        },
    );
    app.insert_resource(progress);
    app.world_mut().resource_mut::<PartyState>().members =
        vec!["hero".to_string(), "mage".to_string()];

    insert_queue(
        &mut app,
        vec![EventAction::GiveExperience {
            amount: 50,
            target: None,
            direction: TransferDirection::Take,
            on_success: vec![marker_action("success")],
            on_failure: vec![marker_action("failure")],
        }],
    );

    app.update();

    // Atomic: mage has only 20 < 50, so neither changes
    let state = app.world().resource::<CharacterProgressState>();
    assert_eq!(state.characters["hero"].experience, 100);
    assert_eq!(state.characters["mage"].experience, 20);
    let gs = app.world().resource::<GameState>();
    assert_eq!(gs.flags.get("failure"), Some(&"injected".to_string()));
    assert!(!gs.flags.contains_key("success"));
}

// =============================================================================
// GiveItem Tests
// =============================================================================

#[test]
fn give_item_give_new_item() {
    let mut app = test_app();
    app.insert_resource(project_data_with_item("potion", true, 99));

    insert_queue(
        &mut app,
        vec![EventAction::GiveItem {
            item_id: "potion".to_string(),
            quantity: 5,
            direction: TransferDirection::Give,
            on_success: vec![],
            on_failure: vec![],
        }],
    );

    app.update();

    let inv = app.world().resource::<InventoryState>();
    assert_eq!(inv.items.get("potion"), Some(&5));
}

#[test]
fn give_item_give_stackable_adds_to_existing() {
    let mut app = test_app();
    app.insert_resource(project_data_with_item("potion", true, 99));
    app.world_mut()
        .resource_mut::<InventoryState>()
        .items
        .insert("potion".to_string(), 10);

    insert_queue(
        &mut app,
        vec![EventAction::GiveItem {
            item_id: "potion".to_string(),
            quantity: 5,
            direction: TransferDirection::Give,
            on_success: vec![],
            on_failure: vec![],
        }],
    );

    app.update();

    let inv = app.world().resource::<InventoryState>();
    assert_eq!(inv.items.get("potion"), Some(&15));
}

#[test]
fn give_item_give_unstackable_duplicate_triggers_on_failure() {
    let mut app = test_app();
    app.insert_resource(project_data_with_item("sword", false, 1));
    app.world_mut()
        .resource_mut::<InventoryState>()
        .items
        .insert("sword".to_string(), 1);

    insert_queue(
        &mut app,
        vec![EventAction::GiveItem {
            item_id: "sword".to_string(),
            quantity: 1,
            direction: TransferDirection::Give,
            on_success: vec![marker_action("success")],
            on_failure: vec![marker_action("failure")],
        }],
    );

    app.update();

    let inv = app.world().resource::<InventoryState>();
    assert_eq!(inv.items.get("sword"), Some(&1));
    let gs = app.world().resource::<GameState>();
    assert_eq!(gs.flags.get("failure"), Some(&"injected".to_string()));
    assert!(!gs.flags.contains_key("success"));
}

#[test]
fn give_item_give_stack_cap_triggers_on_failure() {
    let mut app = test_app();
    app.insert_resource(project_data_with_item("potion", true, 10));
    app.world_mut()
        .resource_mut::<InventoryState>()
        .items
        .insert("potion".to_string(), 10);

    insert_queue(
        &mut app,
        vec![EventAction::GiveItem {
            item_id: "potion".to_string(),
            quantity: 5,
            direction: TransferDirection::Give,
            on_success: vec![marker_action("success")],
            on_failure: vec![marker_action("failure")],
        }],
    );

    app.update();

    let inv = app.world().resource::<InventoryState>();
    assert_eq!(inv.items.get("potion"), Some(&10));
    let gs = app.world().resource::<GameState>();
    assert_eq!(gs.flags.get("failure"), Some(&"injected".to_string()));
    assert!(!gs.flags.contains_key("success"));
}

#[test]
fn give_item_take_removes_quantity() {
    let mut app = test_app();
    app.insert_resource(project_data_with_item("potion", true, 99));
    app.world_mut()
        .resource_mut::<InventoryState>()
        .items
        .insert("potion".to_string(), 10);

    insert_queue(
        &mut app,
        vec![EventAction::GiveItem {
            item_id: "potion".to_string(),
            quantity: 3,
            direction: TransferDirection::Take,
            on_success: vec![marker_action("success")],
            on_failure: vec![marker_action("failure")],
        }],
    );

    app.update();

    let inv = app.world().resource::<InventoryState>();
    assert_eq!(inv.items.get("potion"), Some(&7));
    let gs = app.world().resource::<GameState>();
    assert_eq!(gs.flags.get("success"), Some(&"injected".to_string()));
}

#[test]
fn give_item_take_removes_entry_at_zero() {
    let mut app = test_app();
    app.insert_resource(project_data_with_item("potion", true, 99));
    app.world_mut()
        .resource_mut::<InventoryState>()
        .items
        .insert("potion".to_string(), 5);

    insert_queue(
        &mut app,
        vec![EventAction::GiveItem {
            item_id: "potion".to_string(),
            quantity: 5,
            direction: TransferDirection::Take,
            on_success: vec![marker_action("success")],
            on_failure: vec![marker_action("failure")],
        }],
    );

    app.update();

    let inv = app.world().resource::<InventoryState>();
    assert!(
        !inv.items.contains_key("potion"),
        "Entry should be removed when quantity reaches 0"
    );
    let gs = app.world().resource::<GameState>();
    assert_eq!(gs.flags.get("success"), Some(&"injected".to_string()));
}

#[test]
fn give_item_take_insufficient_quantity() {
    let mut app = test_app();
    app.insert_resource(project_data_with_item("potion", true, 99));
    app.world_mut()
        .resource_mut::<InventoryState>()
        .items
        .insert("potion".to_string(), 2);

    insert_queue(
        &mut app,
        vec![EventAction::GiveItem {
            item_id: "potion".to_string(),
            quantity: 5,
            direction: TransferDirection::Take,
            on_success: vec![marker_action("success")],
            on_failure: vec![marker_action("failure")],
        }],
    );

    app.update();

    let inv = app.world().resource::<InventoryState>();
    assert_eq!(
        inv.items.get("potion"),
        Some(&2),
        "Quantity should not change"
    );
    let gs = app.world().resource::<GameState>();
    assert_eq!(gs.flags.get("failure"), Some(&"injected".to_string()));
    assert!(!gs.flags.contains_key("success"));
}

// =============================================================================
// LearnAbility Tests
// =============================================================================

#[test]
fn learn_ability_give_learns_new_ability() {
    let mut app = test_app();
    app.insert_resource(project_data_with_character_and_ability("hero", "fireball"));

    let mut progress = CharacterProgressState::default();
    progress.characters.insert(
        "hero".to_string(),
        CharacterProgress {
            experience: 0,
            learned_abilities: vec![],
        },
    );
    app.insert_resource(progress);

    insert_queue(
        &mut app,
        vec![EventAction::LearnAbility {
            ability_id: "fireball".to_string(),
            target: "hero".to_string(),
            direction: TransferDirection::Give,
            on_success: vec![],
            on_failure: vec![],
        }],
    );

    app.update();

    let state = app.world().resource::<CharacterProgressState>();
    assert!(
        state.characters["hero"]
            .learned_abilities
            .contains(&"fireball".to_string())
    );
}

#[test]
fn learn_ability_give_idempotent() {
    let mut app = test_app();
    app.insert_resource(project_data_with_character_and_ability("hero", "fireball"));

    let mut progress = CharacterProgressState::default();
    progress.characters.insert(
        "hero".to_string(),
        CharacterProgress {
            experience: 0,
            learned_abilities: vec!["fireball".to_string()],
        },
    );
    app.insert_resource(progress);

    insert_queue(
        &mut app,
        vec![EventAction::LearnAbility {
            ability_id: "fireball".to_string(),
            target: "hero".to_string(),
            direction: TransferDirection::Give,
            on_success: vec![],
            on_failure: vec![],
        }],
    );

    app.update();

    let state = app.world().resource::<CharacterProgressState>();
    // Should still have exactly one entry — no duplicate
    let count = state.characters["hero"]
        .learned_abilities
        .iter()
        .filter(|a| *a == "fireball")
        .count();
    assert_eq!(
        count, 1,
        "Learning an already-known ability should be idempotent"
    );
}

#[test]
fn learn_ability_take_forgets_known_ability() {
    let mut app = test_app();
    app.insert_resource(project_data_with_character_and_ability("hero", "fireball"));

    let mut progress = CharacterProgressState::default();
    progress.characters.insert(
        "hero".to_string(),
        CharacterProgress {
            experience: 0,
            learned_abilities: vec!["fireball".to_string()],
        },
    );
    app.insert_resource(progress);

    insert_queue(
        &mut app,
        vec![EventAction::LearnAbility {
            ability_id: "fireball".to_string(),
            target: "hero".to_string(),
            direction: TransferDirection::Take,
            on_success: vec![marker_action("success")],
            on_failure: vec![marker_action("failure")],
        }],
    );

    app.update();

    let state = app.world().resource::<CharacterProgressState>();
    assert!(
        !state.characters["hero"]
            .learned_abilities
            .contains(&"fireball".to_string())
    );
    let gs = app.world().resource::<GameState>();
    assert_eq!(gs.flags.get("success"), Some(&"injected".to_string()));
    assert!(!gs.flags.contains_key("failure"));
}

#[test]
fn learn_ability_take_not_known_triggers_failure() {
    let mut app = test_app();
    app.insert_resource(project_data_with_character_and_ability("hero", "fireball"));

    let mut progress = CharacterProgressState::default();
    progress.characters.insert(
        "hero".to_string(),
        CharacterProgress {
            experience: 0,
            learned_abilities: vec![],
        },
    );
    app.insert_resource(progress);

    insert_queue(
        &mut app,
        vec![EventAction::LearnAbility {
            ability_id: "fireball".to_string(),
            target: "hero".to_string(),
            direction: TransferDirection::Take,
            on_success: vec![marker_action("success")],
            on_failure: vec![marker_action("failure")],
        }],
    );

    app.update();

    let state = app.world().resource::<CharacterProgressState>();
    assert!(state.characters["hero"].learned_abilities.is_empty());
    let gs = app.world().resource::<GameState>();
    assert_eq!(gs.flags.get("failure"), Some(&"injected".to_string()));
    assert!(!gs.flags.contains_key("success"));
}

// =============================================================================
// AddPartyMember Tests
// =============================================================================

#[test]
fn add_party_member_give_adds_to_party() {
    let mut app = test_app();
    app.insert_resource(project_data_with_character("hero"));

    insert_queue(
        &mut app,
        vec![EventAction::AddPartyMember {
            character_id: "hero".to_string(),
            direction: TransferDirection::Give,
            on_success: vec![],
            on_failure: vec![],
        }],
    );

    app.update();

    let party = app.world().resource::<PartyState>();
    assert!(party.members.contains(&"hero".to_string()));
}

#[test]
fn add_party_member_give_idempotent() {
    let mut app = test_app();
    app.insert_resource(project_data_with_character("hero"));
    app.world_mut().resource_mut::<PartyState>().members = vec!["hero".to_string()];

    insert_queue(
        &mut app,
        vec![EventAction::AddPartyMember {
            character_id: "hero".to_string(),
            direction: TransferDirection::Give,
            on_success: vec![],
            on_failure: vec![],
        }],
    );

    app.update();

    let party = app.world().resource::<PartyState>();
    let count = party.members.iter().filter(|m| *m == "hero").count();
    assert_eq!(count, 1, "Adding an existing member should be idempotent");
}

#[test]
fn add_party_member_take_removes_member() {
    let mut app = test_app();
    app.insert_resource(project_data_with_character("hero"));
    app.world_mut().resource_mut::<PartyState>().members = vec!["hero".to_string()];

    insert_queue(
        &mut app,
        vec![EventAction::AddPartyMember {
            character_id: "hero".to_string(),
            direction: TransferDirection::Take,
            on_success: vec![marker_action("success")],
            on_failure: vec![marker_action("failure")],
        }],
    );

    app.update();

    let party = app.world().resource::<PartyState>();
    assert!(!party.members.contains(&"hero".to_string()));
    let gs = app.world().resource::<GameState>();
    assert_eq!(gs.flags.get("success"), Some(&"injected".to_string()));
    assert!(!gs.flags.contains_key("failure"));
}

#[test]
fn add_party_member_take_not_in_party_triggers_failure() {
    let mut app = test_app();
    app.insert_resource(project_data_with_character("hero"));
    // Party is empty

    insert_queue(
        &mut app,
        vec![EventAction::AddPartyMember {
            character_id: "hero".to_string(),
            direction: TransferDirection::Take,
            on_success: vec![marker_action("success")],
            on_failure: vec![marker_action("failure")],
        }],
    );

    app.update();

    let party = app.world().resource::<PartyState>();
    assert!(party.members.is_empty());
    let gs = app.world().resource::<GameState>();
    assert_eq!(gs.flags.get("failure"), Some(&"injected".to_string()));
    assert!(!gs.flags.contains_key("success"));
}

// =============================================================================
// Branch Injection Tests
// =============================================================================

#[test]
fn branch_injection_pushes_on_success_to_queue_front() {
    let mut app = test_app();
    app.insert_resource(empty_project_data());
    app.world_mut().resource_mut::<CurrencyState>().balance = 100;

    // GiveCurrency Take (sufficient) followed by another action
    insert_queue(
        &mut app,
        vec![
            EventAction::GiveCurrency {
                amount: 10,
                direction: TransferDirection::Take,
                on_success: vec![
                    marker_action("branch_first"),
                    marker_action("branch_second"),
                ],
                on_failure: vec![marker_action("failure")],
            },
            marker_action("after_reward"),
        ],
    );

    app.update();

    // All non-blocking actions should be processed in single frame
    let gs = app.world().resource::<GameState>();
    assert_eq!(gs.flags.get("branch_first"), Some(&"injected".to_string()));
    assert_eq!(gs.flags.get("branch_second"), Some(&"injected".to_string()));
    assert_eq!(gs.flags.get("after_reward"), Some(&"injected".to_string()));
    assert!(!gs.flags.contains_key("failure"));
}

#[test]
fn branch_injection_pushes_on_failure_to_queue_front() {
    let mut app = test_app();
    app.insert_resource(empty_project_data());
    app.world_mut().resource_mut::<CurrencyState>().balance = 5;

    // GiveCurrency Take (insufficient) followed by another action
    insert_queue(
        &mut app,
        vec![
            EventAction::GiveCurrency {
                amount: 100,
                direction: TransferDirection::Take,
                on_success: vec![marker_action("success")],
                on_failure: vec![marker_action("fail_first"), marker_action("fail_second")],
            },
            marker_action("after_reward"),
        ],
    );

    app.update();

    // on_failure should be injected at front, then "after_reward" follows
    let gs = app.world().resource::<GameState>();
    assert_eq!(gs.flags.get("fail_first"), Some(&"injected".to_string()));
    assert_eq!(gs.flags.get("fail_second"), Some(&"injected".to_string()));
    assert_eq!(gs.flags.get("after_reward"), Some(&"injected".to_string()));
    assert!(!gs.flags.contains_key("success"));
}

// =============================================================================
// Non-Blocking Chaining Tests
// =============================================================================

#[test]
fn non_blocking_actions_chain_within_single_frame() {
    let mut app = test_app();
    app.insert_resource(project_data_with_character("hero"));
    app.world_mut().resource_mut::<CurrencyState>().balance = 0;

    let mut progress = CharacterProgressState::default();
    progress.characters.insert(
        "hero".to_string(),
        CharacterProgress {
            experience: 0,
            learned_abilities: vec![],
        },
    );
    app.insert_resource(progress);

    // Queue multiple non-blocking Give actions
    insert_queue(
        &mut app,
        vec![
            EventAction::GiveCurrency {
                amount: 100,
                direction: TransferDirection::Give,
                on_success: vec![],
                on_failure: vec![],
            },
            EventAction::GiveExperience {
                amount: 50,
                target: Some("hero".to_string()),
                direction: TransferDirection::Give,
                on_success: vec![],
                on_failure: vec![],
            },
            EventAction::AddPartyMember {
                character_id: "hero".to_string(),
                direction: TransferDirection::Give,
                on_success: vec![],
                on_failure: vec![],
            },
        ],
    );

    // Single update processes all non-blocking actions
    app.update();

    let currency = app.world().resource::<CurrencyState>();
    assert_eq!(currency.balance, 100);

    let exp = app.world().resource::<CharacterProgressState>();
    assert_eq!(exp.characters["hero"].experience, 50);

    let party = app.world().resource::<PartyState>();
    assert!(party.members.contains(&"hero".to_string()));

    // Queue fully consumed
    assert!(app.world().get_resource::<ActionQueue>().is_none());
}
