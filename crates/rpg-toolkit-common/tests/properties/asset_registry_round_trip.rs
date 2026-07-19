// Feature: unified-asset-management, Property 1: Registration round-trip

use proptest::prelude::*;

use rpg_toolkit_common::asset::{AssetReference, AssetRegistry};

/// Strategy for generating valid asset IDs (1–128 characters).
fn arb_valid_asset_id() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9_\\-]{1,128}"
}

/// Strategy for generating valid relative paths (non-empty, forward-slash separated).
fn arb_valid_relative_path() -> impl Strategy<Value = String> {
    "[a-z]{1,8}(/[a-z]{1,8}){0,3}/[a-z]{1,8}\\.[a-z]{2,4}"
}

/// Strategy for generating valid AssetCategory strings (non-empty).
fn arb_valid_category() -> impl Strategy<Value = String> {
    "[a-z_]{1,32}"
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 100, .. ProptestConfig::default() })]

    /// **Validates: Requirements 1.1, 1.2**
    ///
    /// Property 1: Registration round-trip — For any valid AssetReference with an identifier
    /// of 1–128 characters and any non-empty AssetCategory string, registering it in an
    /// AssetRegistry and then retrieving by the same identifier SHALL return an entry equal
    /// to the original.
    #[test]
    fn registration_round_trip(
        id in arb_valid_asset_id(),
        relative_path in arb_valid_relative_path(),
        category in arb_valid_category(),
    ) {
        let entry = AssetReference {
            id: id.clone(),
            relative_path,
            category,
        };

        let mut registry = AssetRegistry::default();
        registry.register(entry.clone()).expect("registration should succeed for valid entry");

        let retrieved = registry.get(&id).expect("retrieval should succeed for registered entry");

        prop_assert_eq!(
            retrieved, &entry,
            "Retrieved entry should equal the original for id: {:?}",
            id
        );
    }
}
