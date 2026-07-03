// Feature: game-state-management, Property 1: SaveFile serialization round-trip with location

use proptest::collection;
use proptest::option;
use proptest::prelude::*;

use rpg_toolkit_common::save::{CharacterProgressData, SaveFile};

/// Strategy for generating arbitrary CharacterProgressData.
fn arb_character_progress_data() -> impl Strategy<Value = CharacterProgressData> {
    (0u64..100_000, collection::vec("[a-z_]{1,16}", 0..=5)).prop_map(
        |(experience, learned_abilities)| CharacterProgressData {
            experience,
            learned_abilities,
        },
    )
}

/// Strategy for generating an arbitrary UUID-like string for map_id.
fn arb_uuid() -> impl Strategy<Value = String> {
    "[a-f0-9]{8}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{12}"
}

/// Strategy for generating an arbitrary SaveFile with all combinations of location fields.
fn arb_save_file() -> impl Strategy<Value = SaveFile> {
    (
        // state: BTreeMap<String, String> with 0-10 entries
        collection::btree_map("[a-z_]{1,8}", "[a-zA-Z0-9 ]{0,16}", 0..=10),
        // currency: u64 (0..1_000_000)
        0u64..1_000_000,
        // inventory: BTreeMap<String, u32> with 0-10 entries
        collection::btree_map("[a-z_]{1,8}", 0u32..100, 0..=10),
        // party: Vec<String> with 0-5 entries
        collection::vec("[a-z_]{1,12}", 0..=5),
        // character_progress: BTreeMap<String, CharacterProgressData> with 0-5 entries
        collection::btree_map("[a-z_]{1,12}", arb_character_progress_data(), 0..=5),
        // map_id: Option<String> (None or Some with UUID-like string)
        option::of(arb_uuid()),
        // position: Option<(u32, u32)> with coords 0-255
        option::of((0u32..=255, 0u32..=255)),
        // elevation: Option<u32> (None or Some(0..10))
        option::of(0u32..10),
    )
        .prop_map(
            |(
                state,
                currency,
                inventory,
                party,
                character_progress,
                map_id,
                position,
                elevation,
            )| {
                SaveFile {
                    state,
                    currency,
                    inventory,
                    party,
                    character_progress,
                    map_id,
                    position,
                    elevation,
                }
            },
        )
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 100, .. ProptestConfig::default() })]

    /// **Validates: Requirements 2.5, 2.6, 3.4, 10.2, 10.3, 10.4**
    ///
    /// Property 1: SaveFile serialization round-trip with location.
    /// For any valid SaveFile (with arbitrary combinations of state flags, currency, inventory,
    /// party, character progress, and location fields where map_id is Option<String>,
    /// position is Option<(u32, u32)> with coordinates 0–255, and elevation is Option<u32>),
    /// serializing to JSON and then deserializing SHALL produce a SaveFile equal to the original.
    #[test]
    fn save_file_round_trip_with_location(save_file in arb_save_file()) {
        let serialized = serde_json::to_string(&save_file).expect("serialization should succeed");
        let deserialized: SaveFile = serde_json::from_str(&serialized).expect("deserialization should succeed");
        prop_assert_eq!(&deserialized, &save_file,
            "Round-trip failed: serialized form was: {}",
            serialized
        );
    }
}
