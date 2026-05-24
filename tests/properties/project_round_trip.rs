// Feature: character-spritesheets, Property 2: ProjectFile serialization round-trip
// Feature: npc-behaviors, Property 1: NpcInstance serialization round-trip
//
// For any valid ProjectFile containing spritesheets, NPC instances, a player
// spritesheet reference, maps, and tilesets, serializing to JSON and then
// deserializing should produce an equivalent ProjectFile.
//
// Validates: Requirements 1.3, 1.4, 1.5, 1.6, 5.4, 6.2

use std::collections::HashMap;

use proptest::prelude::*;
use rpg_toolkit_common::{
    CharacterSpritesheet, DialogConfigData, DialogPositionData, DialogTextData, EventAction,
    FacingDirection, Layer, MapData, NpcInstance, PatrolConfig, PatrolMode, ProjectFile,
    TileAttributeLayer, TriggerMode,
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

fn arb_patrol_mode() -> impl Strategy<Value = PatrolMode> {
    prop_oneof![Just(PatrolMode::Loop), Just(PatrolMode::Random),]
}

fn arb_trigger_mode() -> impl Strategy<Value = TriggerMode> {
    prop_oneof![Just(TriggerMode::Collision), Just(TriggerMode::Interaction),]
}

fn arb_dialog_position() -> impl Strategy<Value = DialogPositionData> {
    prop_oneof![
        Just(DialogPositionData::Top),
        Just(DialogPositionData::Center),
        Just(DialogPositionData::Bottom),
    ]
}

fn arb_dialog_config() -> impl Strategy<Value = DialogConfigData> {
    (10.0f32..100.0, arb_dialog_position(), any::<bool>()).prop_map(
        |(text_speed, position, movement_block)| DialogConfigData {
            text_speed,
            position,
            movement_block,
        },
    )
}

fn arb_dialog_text_data() -> impl Strategy<Value = DialogTextData> {
    prop_oneof![
        "[a-z ]{1,20}".prop_map(DialogTextData::Inline),
        "[a-z\\-]{3,10}".prop_map(DialogTextData::Id),
    ]
}

fn arb_event_action() -> impl Strategy<Value = EventAction> {
    prop_oneof![
        ("[a-z\\-]{3,10}", 0u32..16, 0u32..16).prop_map(|(target_map_id, target_x, target_y)| {
            EventAction::JumpTo {
                target_map_id,
                target_x,
                target_y,
                target_elevation: None,
            }
        }),
        (arb_dialog_text_data(), arb_dialog_config())
            .prop_map(|(text, config)| { EventAction::ShowDialog { text, config } }),
    ]
}

fn arb_patrol_config(map_w: u32, map_h: u32) -> impl Strategy<Value = PatrolConfig> {
    (
        prop::collection::vec((0..map_w, 0..map_h), 0..=5),
        arb_patrol_mode(),
        0.01f32..2.0,
        0.0f32..3.0,
    )
        .prop_map(|(waypoints, mode, speed, pause)| PatrolConfig {
            waypoints,
            mode,
            speed,
            pause,
        })
}

fn arb_optional_patrol_config(
    map_w: u32,
    map_h: u32,
) -> impl Strategy<Value = Option<PatrolConfig>> {
    prop_oneof![Just(None), arb_patrol_config(map_w, map_h).prop_map(Some),]
}

fn arb_npc_instance(ss_count: usize, map_w: u32, map_h: u32) -> impl Strategy<Value = NpcInstance> {
    (
        arb_spritesheet_id(ss_count),
        0..map_w,
        0..map_h,
        arb_facing_direction(),
        arb_optional_patrol_config(map_w, map_h),
        arb_trigger_mode(),
        prop::collection::vec(arb_event_action(), 0..=3),
    )
        .prop_map(
            |(spritesheet_id, x, y, facing, patrol_config, trigger_mode, event_triggers)| {
                NpcInstance {
                    spritesheet_id,
                    x,
                    y,
                    facing,
                    event_triggers,
                    patrol_config,
                    trigger_mode,
                    elevation: 0,
                }
            },
        )
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
