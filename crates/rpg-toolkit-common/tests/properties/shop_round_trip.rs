// Feature: in-game-shops, Property 5: OpenShop action serialization round-trip
// Feature: in-game-shops, Property 13: Shop registry serialization round-trip
// Feature: in-game-shops, Property 14: Shop ID mismatch validation
// Feature: in-game-shops, Property 15: Shop stock persistence round-trip

use std::collections::{BTreeMap, HashMap};

use proptest::collection;
use proptest::prelude::*;

use rpg_toolkit_common::map::EventAction;
use rpg_toolkit_common::project::ProjectFile;
use rpg_toolkit_common::save::SaveFile;
use rpg_toolkit_common::shop::{ShopDefinition, ShopEntry, ShopRegistry};

/// Strategy for generating a non-empty shop ID string (UUID-like).
fn arb_non_empty_shop_id() -> impl Strategy<Value = String> {
    "[a-f0-9]{8}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{12}"
}

/// Strategy for generating a valid shop display name (1-64 chars).
fn arb_valid_shop_name() -> impl Strategy<Value = String> {
    "[a-zA-Z][a-zA-Z0-9 ]{0,63}".prop_filter("must be non-empty after trim", |s| {
        let trimmed = s.trim();
        !trimmed.is_empty() && trimmed.len() <= 64
    })
}

/// Strategy for generating a valid ShopEntry.
fn arb_shop_entry() -> impl Strategy<Value = ShopEntry> {
    (
        "[a-z]{3,8}-[0-9]{1,4}",           // item_id
        0u32..10000,                       // buy_price
        proptest::option::of(0u32..5000),  // sell_price
        proptest::option::of(1u32..=9999), // stock_limit
    )
        .prop_map(|(item_id, buy_price, sell_price, stock_limit)| ShopEntry {
            item_id,
            buy_price,
            sell_price,
            stock_limit,
            condition: None,
        })
}

/// Strategy for generating a valid ShopDefinition with matching ID.
fn arb_shop_definition() -> impl Strategy<Value = (String, ShopDefinition)> {
    (
        arb_non_empty_shop_id(),
        arb_valid_shop_name(),
        collection::vec(arb_shop_entry(), 0..=5),
    )
        .prop_map(|(id, display_name, entries)| {
            // Deduplicate entries by item_id
            let mut seen = std::collections::HashSet::new();
            let unique_entries: Vec<ShopEntry> = entries
                .into_iter()
                .filter(|e| seen.insert(e.item_id.clone()))
                .collect();

            let def = ShopDefinition {
                id: id.clone(),
                display_name,
                entries: unique_entries,
            };
            (id, def)
        })
}

/// Strategy for generating a valid ShopRegistry (IDs match keys).
fn arb_shop_registry() -> impl Strategy<Value = ShopRegistry> {
    collection::vec(arb_shop_definition(), 0..=5).prop_map(|shops_vec| {
        let mut shops = HashMap::new();
        for (id, def) in shops_vec {
            shops.insert(id, def);
        }
        ShopRegistry { shops }
    })
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 100, .. ProptestConfig::default() })]

    /// **Validates: Requirements 3.1**
    ///
    /// Property 5: OpenShop action serialization round-trip — For any non-empty shop ID string,
    /// serializing an EventAction::OpenShop to JSON and deserializing it back SHALL produce
    /// a value equal to the original. Empty strings SHALL be rejected during deserialization.
    #[test]
    fn open_shop_action_round_trip(shop_id in arb_non_empty_shop_id()) {
        let action = EventAction::OpenShop { shop_id: shop_id.clone() };

        // Serialize to JSON
        let json = serde_json::to_string(&action).expect("serialization should succeed");

        // Deserialize back
        let deserialized: EventAction =
            serde_json::from_str(&json).expect("deserialization should succeed");

        prop_assert_eq!(&deserialized, &action,
            "Round-trip failed for shop_id: {:?}, serialized: {}",
            shop_id, json
        );
    }

    /// **Validates: Requirements 3.1**
    ///
    /// Property 5 (rejection): Empty shop_id strings SHALL be rejected during deserialization.
    #[test]
    fn open_shop_action_rejects_empty_shop_id(_dummy in Just(())) {
        let json = r#"{"type":"OpenShop","shop_id":""}"#;
        let result: Result<EventAction, _> = serde_json::from_str(json);
        prop_assert!(
            result.is_err(),
            "Expected deserialization to fail for empty shop_id, but got: {:?}",
            result
        );
    }

    /// **Validates: Requirements 8.1, 8.3**
    ///
    /// Property 13: Shop registry serialization round-trip — For any valid ShopRegistry
    /// (all IDs match keys, names valid, entries valid), serializing as part of a ProjectFile
    /// to JSON and deserializing back SHALL produce a structurally equal ShopRegistry.
    #[test]
    fn shop_registry_project_file_round_trip(registry in arb_shop_registry()) {
        // Create a minimal valid ProjectFile with just the ShopRegistry
        let project = ProjectFile::new(
            HashMap::new(),  // maps
            HashMap::new(),  // tilesets
            None,            // spawn_point
            HashMap::new(),  // spritesheets
            None,            // player_spritesheet
            HashMap::new(),  // dialog_texts
            HashMap::new(),  // face_portraits
            Default::default(), // characters
            Default::default(), // items
            Default::default(), // abilities
            Default::default(), // enemies
            registry.clone(),   // shops
        );

        // Serialize
        let json = project.serialize().expect("serialization should succeed");

        // Deserialize
        let deserialized = ProjectFile::deserialize(&json)
            .expect("deserialization should succeed");

        prop_assert_eq!(
            &deserialized.shops, &registry,
            "ShopRegistry round-trip failed. Serialized form: {}",
            json
        );
    }

    /// **Validates: Requirements 8.2**
    ///
    /// Property 14: Shop ID mismatch validation — For any JSON where a ShopDefinition's `id`
    /// field does not equal its HashMap key in the registry, deserialization SHALL return
    /// a ProjectValidationError.
    #[test]
    fn shop_id_mismatch_returns_validation_error(
        key_id in arb_non_empty_shop_id(),
        def_id in arb_non_empty_shop_id(),
        name in arb_valid_shop_name(),
    ) {
        // Only test when key and definition ID differ
        prop_assume!(key_id != def_id);

        // Construct a ProjectFile JSON with mismatched shop key and id
        let json = format!(
            r#"{{
                "maps": {{}},
                "tilesets": {{}},
                "shops": {{
                    "shops": {{
                        "{}": {{
                            "id": "{}",
                            "display_name": "{}",
                            "entries": []
                        }}
                    }}
                }}
            }}"#,
            key_id, def_id, name
        );

        let result = ProjectFile::deserialize(&json);
        prop_assert!(
            result.is_err(),
            "Expected ProjectValidationError for mismatched shop key '{}' and id '{}', but got Ok",
            key_id, def_id
        );

        // Verify it's specifically a validation error (contains relevant text)
        if let Err(e) = &result {
            let err_str = e.to_string();
            prop_assert!(
                err_str.contains("shop registry key") || err_str.contains("does not match"),
                "Error should mention shop key mismatch, got: {}",
                err_str
            );
        }
    }

    /// **Validates: Requirements 9.1, 9.2, 9.5**
    ///
    /// Property 15: Shop stock persistence round-trip — For any SaveFile containing a
    /// shop_stock map with valid shop IDs and item IDs mapped to u32 remaining stock values,
    /// serializing to JSON and deserializing back SHALL produce a structurally equal shop_stock map.
    #[test]
    fn shop_stock_save_file_round_trip(
        shop_stock in collection::btree_map(
            arb_non_empty_shop_id(),
            collection::btree_map("[a-z]{3,8}-[0-9]{1,4}", 0u32..10000, 0..=10),
            0..=5
        )
    ) {
        let save_file = SaveFile {
            state: BTreeMap::new(),
            currency: 0,
            inventory: BTreeMap::new(),
            party: Vec::new(),
            character_progress: BTreeMap::new(),
            map_id: None,
            position: None,
            elevation: None,
            shop_stock: shop_stock.clone(),
        };

        // Serialize
        let json = serde_json::to_string(&save_file).expect("serialization should succeed");

        // Deserialize
        let deserialized: SaveFile =
            serde_json::from_str(&json).expect("deserialization should succeed");

        prop_assert_eq!(
            &deserialized.shop_stock, &shop_stock,
            "shop_stock round-trip failed. Serialized form: {}",
            json
        );
    }
}
