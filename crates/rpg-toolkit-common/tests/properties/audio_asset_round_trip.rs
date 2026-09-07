// Feature: audio-system, Property 5: Audio asset serialization round-trip

use proptest::prelude::*;

use rpg_toolkit_common::asset::{
    CATEGORY_MUSIC_LOOP, CATEGORY_SOUND_EFFECT, MusicLoop, SoundEffect,
};

/// Strategy for a valid audio asset id (1–128 characters).
fn arb_valid_id() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9_\\-]{1,128}"
}

/// Strategy for a valid `MusicLoop.relative_path`: non-empty after trimming.
/// The generated value always contains at least one non-whitespace character.
fn arb_music_loop_relative_path() -> impl Strategy<Value = String> {
    "[a-z]{1,8}(/[a-z]{1,8}){0,3}/[a-z]{1,8}\\.[a-z]{2,4}"
}

/// Strategy for a valid `SoundEffect.relative_path`: 1–4096 characters.
fn arb_sound_effect_relative_path() -> impl Strategy<Value = String> {
    "[a-z]{1,8}(/[a-z]{1,8}){0,3}/[a-z]{1,8}\\.[a-z]{2,4}"
}

/// Build a `MusicLoop` from validated inputs by round-tripping through its
/// validated deserializer. The inputs supplied here already satisfy the
/// validation rules, so this is guaranteed to succeed.
fn make_music_loop(id: &str, relative_path: &str) -> MusicLoop {
    let json = serde_json::json!({
        "id": id,
        "relative_path": relative_path,
    })
    .to_string();
    serde_json::from_str(&json).expect("valid MusicLoop inputs should deserialize")
}

/// Build a `SoundEffect` from validated inputs by round-tripping through its
/// validated deserializer.
fn make_sound_effect(id: &str, relative_path: &str) -> SoundEffect {
    let json = serde_json::json!({
        "id": id,
        "relative_path": relative_path,
    })
    .to_string();
    serde_json::from_str(&json).expect("valid SoundEffect inputs should deserialize")
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 100, .. ProptestConfig::default() })]

    /// **Validates: Requirements 1.8, 1.9, 2.8, 2.9**
    ///
    /// Property 5: Audio asset serialization round-trip (MusicLoop).
    /// For all valid `MusicLoop` values, serializing to JSON and deserializing the
    /// result produces a value whose `id`, `relative_path`, and `category` fields
    /// each equal the original's.
    #[test]
    fn music_loop_serialization_round_trip(
        id in arb_valid_id(),
        relative_path in arb_music_loop_relative_path(),
    ) {
        let original = make_music_loop(&id, &relative_path);

        let json = serde_json::to_string(&original).expect("MusicLoop should serialize");
        let deserialized: MusicLoop =
            serde_json::from_str(&json).expect("serialized MusicLoop should deserialize");

        prop_assert_eq!(&deserialized.id, &original.id);
        prop_assert_eq!(&deserialized.relative_path, &original.relative_path);
        prop_assert_eq!(&deserialized.category, &original.category);
        // The category is always coerced to the well-known constant.
        prop_assert_eq!(deserialized.category.as_str(), CATEGORY_MUSIC_LOOP);
    }

    /// **Validates: Requirements 1.8, 1.9, 2.8, 2.9**
    ///
    /// Property 5: Audio asset serialization round-trip (SoundEffect).
    /// For all valid `SoundEffect` values, serializing to JSON and deserializing the
    /// result produces a value whose `id`, `relative_path`, and `category` fields
    /// each equal the original's.
    #[test]
    fn sound_effect_serialization_round_trip(
        id in arb_valid_id(),
        relative_path in arb_sound_effect_relative_path(),
    ) {
        let original = make_sound_effect(&id, &relative_path);

        let json = serde_json::to_string(&original).expect("SoundEffect should serialize");
        let deserialized: SoundEffect =
            serde_json::from_str(&json).expect("serialized SoundEffect should deserialize");

        prop_assert_eq!(&deserialized.id, &original.id);
        prop_assert_eq!(&deserialized.relative_path, &original.relative_path);
        prop_assert_eq!(&deserialized.category, &original.category);
        // The category is always coerced to the well-known constant.
        prop_assert_eq!(deserialized.category.as_str(), CATEGORY_SOUND_EFFECT);
    }
}
