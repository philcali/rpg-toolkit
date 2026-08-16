// Feature: game-intro-narration, Property 1: EventAction serialization round-trip (extended)

use proptest::prelude::*;

use rpg_toolkit_common::map::{EntityTarget, EventAction};

/// Strategy for generating a non-empty NPC ID string.
fn arb_npc_id() -> BoxedStrategy<String> {
    "[a-zA-Z][a-zA-Z0-9_-]{0,31}".prop_map(|s| s).boxed()
}

/// Strategy for generating a valid EntityTarget.
fn arb_entity_target() -> BoxedStrategy<EntityTarget> {
    prop_oneof![
        Just(EntityTarget::Player),
        arb_npc_id().prop_map(|npc_id| EntityTarget::Npc { npc_id }),
    ]
    .boxed()
}

/// Strategy for generating a valid MoveEntity action.
fn arb_move_entity() -> BoxedStrategy<EventAction> {
    (
        arb_entity_target(),
        any::<u32>(),
        any::<u32>(),
        0.1f32..=10.0f32,
    )
        .prop_map(
            |(target, target_x, target_y, speed)| EventAction::MoveEntity {
                target,
                target_x,
                target_y,
                speed,
            },
        )
        .boxed()
}

/// Strategy for generating a valid CameraFollow action.
fn arb_camera_follow() -> BoxedStrategy<EventAction> {
    arb_entity_target()
        .prop_map(|target| EventAction::CameraFollow { target })
        .boxed()
}

/// Strategy for generating a valid CameraPan action.
fn arb_camera_pan() -> BoxedStrategy<EventAction> {
    (any::<u32>(), any::<u32>(), 0.1f32..=10.0f32)
        .prop_map(|(target_x, target_y, duration)| EventAction::CameraPan {
            target_x,
            target_y,
            duration,
        })
        .boxed()
}

/// Strategy for generating a valid Wait action.
fn arb_wait() -> BoxedStrategy<EventAction> {
    (0.1f32..=30.0f32)
        .prop_map(|duration| EventAction::Wait { duration })
        .boxed()
}

/// Strategy for generating any of the new intro-related EventAction variants.
fn arb_intro_action() -> BoxedStrategy<EventAction> {
    prop_oneof![
        arb_move_entity(),
        arb_camera_follow(),
        arb_camera_pan(),
        arb_wait(),
    ]
    .boxed()
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 100, .. ProptestConfig::default() })]

    /// **Validates: Requirements 9.1, 9.6**
    ///
    /// Property 1: EventAction serialization round-trip (extended).
    /// For any valid EventAction value that is a new intro variant (MoveEntity,
    /// CameraFollow, CameraPan, or Wait) with fields within their valid ranges,
    /// serializing to JSON and deserializing back shall produce a value equal
    /// to the original.
    #[test]
    fn intro_action_serialization_round_trip(
        action in arb_intro_action(),
    ) {
        // Serialize to JSON
        let json = serde_json::to_string(&action).unwrap();

        // Deserialize back
        let deserialized: EventAction = serde_json::from_str(&json).unwrap();

        // Assert equality
        prop_assert_eq!(&action, &deserialized);
    }
}
