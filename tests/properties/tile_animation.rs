// Feature: editor-ux-improvements, Property 1: Animation Serialization Round-Trip
//
// For any valid TileAnimation (frame_count >= 2, frame_duration_ms > 0, and all
// frame coordinates within tileset bounds), serializing the containing TilesetMeta
// to JSON and then deserializing it SHALL produce an equivalent TilesetMeta with
// identical animation definitions.
//
// Validates: Requirements 1.2, 1.3, 1.4

use proptest::prelude::*;
use rpg_toolkit_common::{
    AnimationFrame, TileAnimation, TilesetMeta, compute_animation_frame_index,
    validate_tile_animation,
};

/// Generates a valid AnimationFrame with coordinates within the given tileset bounds.
fn arb_animation_frame(columns: u32, rows: u32) -> impl Strategy<Value = AnimationFrame> {
    (0..columns, 0..rows).prop_map(|(col, row)| AnimationFrame { col, row })
}

/// Generates a valid TileAnimation with at least 2 frames, positive duration,
/// and all frame coordinates within tileset bounds.
fn arb_tile_animation(columns: u32, rows: u32) -> impl Strategy<Value = TileAnimation> {
    (
        prop::collection::vec(arb_animation_frame(columns, rows), 2..=8),
        1..=2000u32,
    )
        .prop_map(|(frames, frame_duration_ms)| TileAnimation {
            frames,
            frame_duration_ms,
        })
}

/// Generates a valid TilesetMeta with 0–4 animations, all within bounds.
fn arb_tileset_meta() -> impl Strategy<Value = TilesetMeta> {
    // Use reasonable tileset dimensions
    let columns = 1u32..=16;
    let rows = 1u32..=16;
    let tile_size = prop_oneof![Just(8u32), Just(16), Just(32), Just(64)];

    (
        columns,
        rows,
        tile_size.clone(),
        tile_size,
        "[a-z]{3,8}\\.png",
    )
        .prop_flat_map(|(columns, rows, tile_width, tile_height, file_path)| {
            let animations_strategy =
                prop::collection::vec(arb_tile_animation(columns, rows), 0..=4);

            (
                Just(file_path),
                Just(tile_width),
                Just(tile_height),
                Just(columns),
                Just(rows),
                animations_strategy,
            )
                .prop_map(
                    |(file_path, tile_width, tile_height, columns, rows, animations)| TilesetMeta {
                        file_path,
                        tile_width,
                        tile_height,
                        columns,
                        rows,
                        animations,
                    },
                )
        })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn animation_serialization_round_trip(tileset_meta in arb_tileset_meta()) {
        // Serialize to JSON
        let json = serde_json::to_string(&tileset_meta)
            .expect("serialization should succeed for valid TilesetMeta");

        // Deserialize back
        let deserialized: TilesetMeta = serde_json::from_str(&json)
            .expect("deserialization should succeed for valid TilesetMeta JSON");

        // Assert equivalence
        prop_assert_eq!(&tileset_meta, &deserialized);
    }
}

// Feature: editor-ux-improvements, Property 2: Animation Validation Correctness
//
// For any TileAnimation and tileset dimensions (columns, rows),
// validate_tile_animation SHALL return Ok(()) if and only if: the animation has
// at least 2 frames, frame_duration_ms > 0, and every frame satisfies
// col < columns and row < rows. Otherwise it SHALL return an error.
//
// Validates: Requirements 1.5, 1.6, 1.7

/// Generates an arbitrary AnimationFrame (not necessarily within bounds).
fn arb_arbitrary_animation_frame() -> impl Strategy<Value = AnimationFrame> {
    (0..=20u32, 0..=20u32).prop_map(|(col, row)| AnimationFrame { col, row })
}

/// Generates an arbitrary TileAnimation (not necessarily valid).
fn arb_arbitrary_tile_animation() -> impl Strategy<Value = TileAnimation> {
    (
        prop::collection::vec(arb_arbitrary_animation_frame(), 0..=8),
        0..=2000u32,
    )
        .prop_map(|(frames, frame_duration_ms)| TileAnimation {
            frames,
            frame_duration_ms,
        })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn animation_validation_correctness(
        animation in arb_arbitrary_tile_animation(),
        columns in 1..=20u32,
        rows in 1..=20u32,
    ) {
        let result = validate_tile_animation(&animation, columns, rows);

        let has_enough_frames = animation.frames.len() >= 2;
        let has_positive_duration = animation.frame_duration_ms > 0;
        let all_in_bounds = animation.frames.iter().all(|f| f.col < columns && f.row < rows);

        let should_be_valid = has_enough_frames && has_positive_duration && all_in_bounds;

        if should_be_valid {
            prop_assert!(
                result.is_ok(),
                "Expected Ok for valid animation: frames={}, duration={}, columns={}, rows={}, frames={:?}",
                animation.frames.len(),
                animation.frame_duration_ms,
                columns,
                rows,
                animation.frames
            );
        } else {
            prop_assert!(
                result.is_err(),
                "Expected Err for invalid animation: frames={}, duration={}, columns={}, rows={}, frames={:?}",
                animation.frames.len(),
                animation.frame_duration_ms,
                columns,
                rows,
                animation.frames
            );
        }
    }
}

// Feature: editor-ux-improvements, Property 3: Frame Cycling Correctness
//
// For any valid animation with frame_count >= 2 and frame_duration_ms > 0,
// and for any non-negative elapsed_ms, compute_animation_frame_index(elapsed_ms,
// frame_duration_ms, frame_count) SHALL return a value in [0, frame_count) equal
// to (elapsed_ms / frame_duration_ms as u64) % frame_count as u64.
//
// Validates: Requirements 3.1, 3.2, 3.3, 4.1, 4.2, 4.3

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn frame_cycling_correctness(
        elapsed_ms in 0..=u64::MAX,
        frame_duration_ms in 1..=2000u32,
        frame_count in 2..=16usize,
    ) {
        let result = compute_animation_frame_index(elapsed_ms, frame_duration_ms, frame_count);

        // Assert result is in [0, frame_count)
        prop_assert!(
            result < frame_count,
            "Expected result < frame_count ({}), got {}",
            frame_count,
            result
        );

        // Assert result matches the formula
        let expected = ((elapsed_ms / frame_duration_ms as u64) % frame_count as u64) as usize;
        prop_assert_eq!(
            result,
            expected,
            "For elapsed_ms={}, frame_duration_ms={}, frame_count={}: expected {}, got {}",
            elapsed_ms,
            frame_duration_ms,
            frame_count,
            expected,
            result
        );
    }
}

// Feature: editor-ux-improvements, Property 4: Animation Lockstep Synchronization
//
// For any two tile instances referencing the same TileAnimation, given the same
// global elapsed_ms, both SHALL compute the same frame index. This is implied by
// the shared pure function, but we verify it explicitly.
//
// Validates: Requirements 3.4, 4.4

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn animation_lockstep_synchronization(
        elapsed_ms in 0..=u64::MAX,
        frame_duration_ms in 1..=2000u32,
        frame_count in 2..=16usize,
    ) {
        // Simulate two different tile instances referencing the same animation parameters
        let tile_instance_1 = compute_animation_frame_index(elapsed_ms, frame_duration_ms, frame_count);
        let tile_instance_2 = compute_animation_frame_index(elapsed_ms, frame_duration_ms, frame_count);

        // Both tile instances must compute the same frame index
        prop_assert_eq!(
            tile_instance_1,
            tile_instance_2,
            "Two tile instances with same animation params (elapsed_ms={}, frame_duration_ms={}, frame_count={}) \
             must produce the same frame index, but got {} and {}",
            elapsed_ms,
            frame_duration_ms,
            frame_count,
            tile_instance_1,
            tile_instance_2
        );
    }
}
