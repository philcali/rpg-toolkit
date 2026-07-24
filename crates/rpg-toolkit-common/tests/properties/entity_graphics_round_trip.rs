// Feature: graphics-database-editor, Property 3: EntityGraphics serialization round-trip

use proptest::prelude::*;

use rpg_toolkit_common::graphics::EntityGraphics;

/// Strategy for generating valid icon path segments (no `.` or `..` components).
/// Uses alphanumeric characters plus underscore, hyphen, and forward slash.
fn arb_icon_segment() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9_\\-]{1,20}"
}

/// Strategy for generating valid icon paths:
/// - 1–260 characters
/// - No `.` or `..` path components
/// - Forward-slash separators
fn arb_valid_icon_path() -> impl Strategy<Value = String> {
    prop::collection::vec(arb_icon_segment(), 1..=5)
        .prop_map(|segments| segments.join("/"))
        .prop_filter("path must be 1–260 characters", |p| {
            !p.is_empty() && p.len() <= 260
        })
}

/// Strategy for generating valid EntityGraphics values:
/// icon is either None or Some(valid path string).
fn arb_entity_graphics() -> impl Strategy<Value = EntityGraphics> {
    prop_oneof![
        // None case
        Just(EntityGraphics::default()),
        // Some case with a valid icon path
        arb_valid_icon_path().prop_map(|path| {
            let mut gfx = EntityGraphics::default();
            gfx.set_icon(&path).expect("valid path should succeed");
            gfx
        }),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 100, .. ProptestConfig::default() })]

    /// **Validates: Requirements 2.5, 10.2**
    ///
    /// Property 3: EntityGraphics serialization round-trip — For any valid
    /// EntityGraphics value (icon is None or Some(string) where string is
    /// 1–260 characters after trimming with no path traversal), serializing
    /// to JSON and deserializing back SHALL produce an identical value.
    #[test]
    fn entity_graphics_serialization_round_trip(
        gfx in arb_entity_graphics(),
    ) {
        let json = serde_json::to_string(&gfx)
            .expect("serialization should succeed");
        let deserialized: EntityGraphics = serde_json::from_str(&json)
            .expect("deserialization should succeed");

        prop_assert_eq!(
            &deserialized, &gfx,
            "Deserialized EntityGraphics should equal original. \
             Original: {:?}, JSON: {:?}, Deserialized: {:?}",
            gfx, json, deserialized
        );
    }
}
