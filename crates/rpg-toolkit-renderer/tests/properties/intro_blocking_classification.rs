// Feature: game-intro-narration, Property 4: is_blocking_action classifies new actions correctly

use proptest::prelude::*;

use rpg_toolkit_common::map::{EntityTarget, EventAction};
use rpg_toolkit_renderer::is_blocking_action;

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

/// Strategy for generating a valid MoveEntity action (blocking).
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

/// Strategy for generating a valid CameraFollow action (non-blocking).
fn arb_camera_follow() -> BoxedStrategy<EventAction> {
    arb_entity_target()
        .prop_map(|target| EventAction::CameraFollow { target })
        .boxed()
}

/// Strategy for generating a valid CameraPan action (blocking).
fn arb_camera_pan() -> BoxedStrategy<EventAction> {
    (any::<u32>(), any::<u32>(), 0.1f32..=10.0f32)
        .prop_map(|(target_x, target_y, duration)| EventAction::CameraPan {
            target_x,
            target_y,
            duration,
        })
        .boxed()
}

/// Strategy for generating a valid Wait action (blocking).
fn arb_wait() -> BoxedStrategy<EventAction> {
    (0.1f32..=30.0f32)
        .prop_map(|duration| EventAction::Wait { duration })
        .boxed()
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 100, .. ProptestConfig::default() })]

    /// **Validates: Requirements 2.3, 3.2, 4.2, 5.2**
    ///
    /// Property 4: MoveEntity is always classified as blocking.
    #[test]
    fn move_entity_is_blocking(action in arb_move_entity()) {
        prop_assert!(
            is_blocking_action(&action),
            "MoveEntity should be blocking but was not: {:?}",
            action
        );
    }

    /// **Validates: Requirements 2.3, 3.2, 4.2, 5.2**
    ///
    /// Property 4: CameraPan is always classified as blocking.
    #[test]
    fn camera_pan_is_blocking(action in arb_camera_pan()) {
        prop_assert!(
            is_blocking_action(&action),
            "CameraPan should be blocking but was not: {:?}",
            action
        );
    }

    /// **Validates: Requirements 2.3, 3.2, 4.2, 5.2**
    ///
    /// Property 4: Wait is always classified as blocking.
    #[test]
    fn wait_is_blocking(action in arb_wait()) {
        prop_assert!(
            is_blocking_action(&action),
            "Wait should be blocking but was not: {:?}",
            action
        );
    }

    /// **Validates: Requirements 2.3, 3.2, 4.2, 5.2**
    ///
    /// Property 4: CameraFollow is always classified as non-blocking.
    #[test]
    fn camera_follow_is_non_blocking(action in arb_camera_follow()) {
        prop_assert!(
            !is_blocking_action(&action),
            "CameraFollow should be non-blocking but was blocking: {:?}",
            action
        );
    }
}
