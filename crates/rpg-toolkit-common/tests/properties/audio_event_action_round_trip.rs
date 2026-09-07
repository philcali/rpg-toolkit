// Feature: audio-system, Property 8: Audio EventAction round-trip
// Validates: Requirements 5.1, 5.2, 5.5, 5.6, 5.8, 5.9, 5.11, 5.12, 5.14, 5.15, 7.1, 7.2, 7.6, 7.7, 7.9, 7.10

use proptest::prelude::*;

use rpg_toolkit_common::map::EventAction;

/// Strategy for a valid audio id (1–128 characters), matching the
/// `MusicLoopId` / `SoundEffectId` validation rules.
fn arb_valid_id() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9_\\-]{1,128}"
}

/// Strategy for a valid fade duration in seconds: finite, 0.0..=10.0.
///
/// Values are drawn from a bounded integer domain and scaled so they are
/// exactly representable and survive a JSON round-trip bit-for-bit (serde_json
/// preserves finite f32 values, but constraining to clean decimals removes any
/// risk of representation flakiness under `PartialEq` on `f32`).
fn arb_fade_seconds() -> impl Strategy<Value = f32> {
    (0u32..=1000u32).prop_map(|n| n as f32 / 100.0)
}

/// Strategy for a valid volume: finite, 0.0..=1.0.
fn arb_volume() -> impl Strategy<Value = f32> {
    (0u32..=100u32).prop_map(|n| n as f32 / 100.0)
}

/// Strategy for a valid `loop_count`: `None` (infinite) or `Some(n)` with n >= 1.
fn arb_loop_count() -> impl Strategy<Value = Option<u32>> {
    prop_oneof![Just(None), (1u32..=1000u32).prop_map(Some)]
}

/// Strategy for a valid `PlayMusic` EventAction.
fn arb_play_music() -> BoxedStrategy<EventAction> {
    (
        arb_valid_id(),
        arb_fade_seconds(),
        arb_loop_count(),
        arb_fade_seconds(),
    )
        .prop_map(
            |(music_loop_id, fade_duration, loop_count, fade_out_duration)| {
                EventAction::PlayMusic {
                    music_loop_id,
                    fade_duration,
                    loop_count,
                    fade_out_duration,
                }
            },
        )
        .boxed()
}

/// Strategy for a valid `PlaySoundEffect` EventAction.
fn arb_play_sound_effect() -> BoxedStrategy<EventAction> {
    (arb_valid_id(), arb_volume())
        .prop_map(|(sound_effect_id, volume)| EventAction::PlaySoundEffect {
            sound_effect_id,
            volume,
        })
        .boxed()
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 100, .. ProptestConfig::default() })]

    /// **Validates: Requirements 5.1, 5.2, 5.5, 5.6, 5.8, 5.9, 5.11, 5.12, 5.14, 5.15, 7.1, 7.2, 7.6, 7.7, 7.9, 7.10**
    ///
    /// Property 8: Audio EventAction round-trip (PlayMusic).
    /// For all valid `PlayMusic` values, serializing to JSON using the
    /// `#[serde(tag = "type")]` format and deserializing the result produces a
    /// value equal to the original, and the JSON carries the `"PlayMusic"` type tag.
    #[test]
    fn play_music_round_trip(action in arb_play_music()) {
        let json = serde_json::to_string(&action).expect("PlayMusic should serialize");

        // The serialized JSON carries the correct `"type"` tag.
        let value: serde_json::Value =
            serde_json::from_str(&json).expect("serialized JSON should parse");
        prop_assert_eq!(value.get("type").and_then(|v| v.as_str()), Some("PlayMusic"));

        // Round-trip produces an equal value.
        let deserialized: EventAction =
            serde_json::from_str(&json).expect("serialized PlayMusic should deserialize");
        prop_assert_eq!(&deserialized, &action);
    }

    /// **Validates: Requirements 5.1, 5.2, 5.5, 5.6, 5.8, 5.9, 5.11, 5.12, 5.14, 5.15, 7.1, 7.2, 7.6, 7.7, 7.9, 7.10**
    ///
    /// Property 8: Audio EventAction round-trip (PlaySoundEffect).
    /// For all valid `PlaySoundEffect` values, serializing to JSON using the
    /// `#[serde(tag = "type")]` format and deserializing the result produces a
    /// value equal to the original, and the JSON carries the `"PlaySoundEffect"` type tag.
    #[test]
    fn play_sound_effect_round_trip(action in arb_play_sound_effect()) {
        let json = serde_json::to_string(&action).expect("PlaySoundEffect should serialize");

        // The serialized JSON carries the correct `"type"` tag.
        let value: serde_json::Value =
            serde_json::from_str(&json).expect("serialized JSON should parse");
        prop_assert_eq!(
            value.get("type").and_then(|v| v.as_str()),
            Some("PlaySoundEffect")
        );

        // Round-trip produces an equal value.
        let deserialized: EventAction =
            serde_json::from_str(&json).expect("serialized PlaySoundEffect should deserialize");
        prop_assert_eq!(&deserialized, &action);
    }
}
