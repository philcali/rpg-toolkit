// Feature: audio-system, Property 6: MapData default_music_loop round-trip

use proptest::prelude::*;

use rpg_toolkit_common::map::MapData;

/// Valid tile sizes accepted by `MapData::new`.
const VALID_TILE_SIZES: [u32; 4] = [8, 16, 32, 64];

/// Strategy for a valid `default_music_loop`: `None` or a 1..=128 character id.
fn arb_default_music_loop() -> impl Strategy<Value = Option<String>> {
    prop_oneof![Just(None), "[a-zA-Z0-9_\\-]{1,128}".prop_map(Some),]
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 100, .. ProptestConfig::default() })]

    /// **Validates: Requirements 4.1, 4.2, 4.3, 4.6, 4.7**
    ///
    /// Property 6: MapData default_music_loop round-trip.
    /// For all valid `MapData` values whose `default_music_loop` is either `None` or a
    /// 1..=128 character id, serializing to JSON and deserializing the result produces a
    /// value equal to the original.
    #[test]
    fn map_default_music_loop_round_trip(
        name in "[a-zA-Z0-9 _\\-]{1,32}",
        width in 1u32..=256,
        height in 1u32..=256,
        tile_width_idx in 0usize..VALID_TILE_SIZES.len(),
        tile_height_idx in 0usize..VALID_TILE_SIZES.len(),
        default_music_loop in arb_default_music_loop(),
    ) {
        let tile_width = VALID_TILE_SIZES[tile_width_idx];
        let tile_height = VALID_TILE_SIZES[tile_height_idx];

        let mut original =
            MapData::new(name, width, height, tile_width, tile_height)
                .expect("MapData::new should succeed for valid dimensions and tile sizes");
        original.default_music_loop = default_music_loop;

        // Serialize to JSON
        let json = serde_json::to_string(&original)
            .expect("serialization should succeed");

        // Deserialize back
        let deserialized: MapData = serde_json::from_str(&json)
            .expect("deserialization should succeed for a valid MapData");

        // Assert equality (round-trip preserves the value, including default_music_loop)
        prop_assert_eq!(&original, &deserialized);
    }
}
