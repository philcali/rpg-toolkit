// Feature: character-spritesheets, Property 2: ProjectFile serialization round-trip
//
// For any valid ProjectFile containing spritesheets, NPC instances, a player
// spritesheet reference, maps, and tilesets, serializing to JSON and then
// deserializing should produce an equivalent ProjectFile.
//
// Validates: Requirements 1.4, 1.5, 1.6, 5.4

use std::collections::HashMap;

use proptest::prelude::*;
use rpg_toolkit_common::{
    CharacterSpritesheet, FacingDirection, Layer, MapData, NpcInstance, ProjectFile,
    TileAttributeLayer,
};

// --- Arbitrary strategies ---

fn arb_facing_direction() -> impl Strategy<Value = FacingDirection> {
    prop_oneof![
        Just(FacingDirection::Down),
        Just(FacingDirection::Left),
        Just(FacingDirection::Right),
        Just(FacingDirection::Up),
    ]
}

fn arb_spritesheet_id(count: usize) -> impl Strategy<Value = String> {
    (0..count).prop_map(|i| format!("ss-{}", i))
}

fn arb_character_spritesheet() -> impl Strategy<Value = CharacterSpritesheet> {
    "[a-z]{3,8}\\.png".prop_map(|file_path| CharacterSpritesheet {
        file_path,
        sprite_width: 24,
        sprite_height: 32,
        frame_count: 3,
        direction_count: 4,
    })
}

fn arb_npc_instance(ss_count: usize, map_w: u32, map_h: u32) -> impl Strategy<Value = NpcInstance> {
    (
        arb_spritesheet_id(ss_count),
        0..map_w,
        0..map_h,
        arb_facing_direction(),
    )
        .prop_map(|(spritesheet_id, x, y, facing)| NpcInstance {
            spritesheet_id,
            x,
            y,
            facing,
            event_triggers: Vec::new(),
            patrol_path: Vec::new(),
        })
}

/// Generates a valid MapData with the given dimensions and NPC count.
/// NPCs reference spritesheets from the pool of `ss_count` spritesheets.
fn arb_map_data(ss_count: usize) -> impl Strategy<Value = MapData> {
    let width = 1u32..=8;
    let height = 1u32..=8;
    let tile_size = prop_oneof![Just(8u32), Just(16), Just(32), Just(64)];

    (width, height, tile_size.clone(), tile_size).prop_flat_map(move |(w, h, tw, th)| {
        let npc_count = if ss_count > 0 { 0usize..=5 } else { 0usize..=0 };
        let npcs_strategy = prop::collection::vec(arb_npc_instance(ss_count, w, h), npc_count);

        (Just(w), Just(h), Just(tw), Just(th), npcs_strategy).prop_map(|(w, h, tw, th, npcs)| {
            let tiles = vec![vec![None; w as usize]; h as usize];
            let attributes = TileAttributeLayer::new(w, h);
            let layer = Layer {
                name: "Ground".to_string(),
                visible: true,
                tiles,
                attributes,
            };
            MapData {
                name: "test-map".to_string(),
                width: w,
                height: h,
                tile_width: tw,
                tile_height: th,
                layers: vec![layer],
                active_layer_index: 0,
                npcs,
            }
        })
    })
}

/// Generates a valid ProjectFile with 0–3 maps, 0–3 spritesheets, 0–5 NPCs
/// per map, and an optional player_spritesheet.
fn arb_project_file() -> impl Strategy<Value = ProjectFile> {
    let ss_count = 0usize..=3;

    ss_count.prop_flat_map(|ss_count| {
        // Generate spritesheets
        let spritesheets_strategy = if ss_count == 0 {
            Just(HashMap::new()).boxed()
        } else {
            prop::collection::hash_map(
                arb_spritesheet_id(ss_count),
                arb_character_spritesheet(),
                ss_count..=ss_count,
            )
            .boxed()
        };

        let map_count = 0usize..=3;
        let maps_strategy = prop::collection::vec(arb_map_data(ss_count), map_count);

        // Optional player spritesheet (only if spritesheets exist)
        let player_ss_strategy = if ss_count > 0 {
            prop_oneof![Just(None), arb_spritesheet_id(ss_count).prop_map(Some),].boxed()
        } else {
            Just(None).boxed()
        };

        (spritesheets_strategy, maps_strategy, player_ss_strategy).prop_map(
            |(spritesheets, maps_vec, player_spritesheet)| {
                let mut maps = HashMap::new();
                for (i, map) in maps_vec.into_iter().enumerate() {
                    maps.insert(format!("map-{}", i), map);
                }

                ProjectFile::new(
                    maps,
                    HashMap::new(), // no tilesets needed for this property
                    None,           // no spawn point needed
                    spritesheets,
                    player_spritesheet,
                    HashMap::new(), // no dialog texts needed for this property
                )
            },
        )
    })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn project_file_round_trip(project in arb_project_file()) {
        // Serialize to JSON
        let json = project.serialize()
            .expect("serialization should succeed for valid ProjectFile");

        // Deserialize back
        let deserialized = ProjectFile::deserialize(&json)
            .expect("deserialization should succeed for valid ProjectFile");

        // Assert equivalence
        prop_assert_eq!(&project, &deserialized);
    }
}
