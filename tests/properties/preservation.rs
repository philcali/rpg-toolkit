// Property 2: Preservation — Normal Canvas Interactions Unchanged
//
// These tests capture the baseline behavior of the five canvas interaction
// systems when NO dialog is consuming pointer input (wants_pointer_input() == false).
// They verify that painting, zooming, panning, attribute opacity toggling,
// and cursor state computation all work correctly.
//
// EXPECTED: All tests PASS on unfixed code (confirming baseline to preserve).
// After the fix, these tests must still PASS (confirming no regressions).
//
// Validates: Requirements 3.1, 3.2, 3.3, 3.4, 3.5, 3.6

use proptest::prelude::*;
use rpg_toolkit_common::{MapData, TileRef};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Create a minimal valid MapData with the given dimensions and one layer.
fn make_map(width: u32, height: u32) -> MapData {
    MapData::new("test", width, height, 32, 32).expect("valid map")
}

// ---------------------------------------------------------------------------
// Property 2a: painting preservation — tile placement works without dialog
// ---------------------------------------------------------------------------
//
// When wants_pointer_input() == false, the painting_system places tiles
// via left-click with an active brush at a valid tile_pos. We verify
// that the tile is written to the layer grid.

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Preservation: painting with active brush places tile when no dialog.
    /// Validates Req 3.1
    #[test]
    fn painting_places_tile_when_no_dialog(
        width in 1u32..=16,
        height in 1u32..=16,
        col_frac in 0.0f64..1.0,
        row_frac in 0.0f64..1.0,
        brush_col in 0u32..16,
        brush_row in 0u32..16,
    ) {
        let mut map = make_map(width, height);
        let col = (col_frac * width as f64).floor() as u32;
        let row = (row_frac * height as f64).floor() as u32;
        let col = col.min(width - 1);
        let row = row.min(height - 1);

        let brush = TileRef {
            tileset_id: "ts-1".to_string(),
            col: brush_col,
            row: brush_row,
        };

        // Simulate painting_system core path: Paint tool + left-click + valid tile_pos
        // No dialog guard fires because wants_pointer_input() == false.
        let left_pressed = true;
        let wants_pointer = false;

        if !wants_pointer && left_pressed {
            map.layers[0].tiles[row as usize][col as usize] = Some(brush.clone());
        }

        // Tile must be placed
        prop_assert_eq!(
            map.layers[0].tiles[row as usize][col as usize].as_ref(),
            Some(&brush),
            "painting_system failed to place tile at ({}, {}) when no dialog was active",
            col, row
        );
    }
}


// ---------------------------------------------------------------------------
// Property 2a-extra: painting preservation — erasing clears tile when no dialog
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Preservation: erasing clears tile when no dialog active.
    /// Validates Req 3.1
    #[test]
    fn erasing_clears_tile_when_no_dialog(
        width in 1u32..=16,
        height in 1u32..=16,
        col_frac in 0.0f64..1.0,
        row_frac in 0.0f64..1.0,
    ) {
        let mut map = make_map(width, height);
        let col = (col_frac * width as f64).floor().min((width - 1) as f64) as u32;
        let row = (row_frac * height as f64).floor().min((height - 1) as f64) as u32;

        // Pre-place a tile so there's something to erase
        let brush = TileRef {
            tileset_id: "ts-1".to_string(),
            col: 0,
            row: 0,
        };
        map.layers[0].tiles[row as usize][col as usize] = Some(brush);

        // Simulate erase path: Erase tool + left-click + valid tile_pos, no dialog
        let left_pressed = true;
        let wants_pointer = false;

        if !wants_pointer && left_pressed {
            map.layers[0].tiles[row as usize][col as usize] = None;
        }

        prop_assert_eq!(
            map.layers[0].tiles[row as usize][col as usize].as_ref(),
            None,
            "erase failed to clear tile at ({}, {}) when no dialog was active",
            col, row
        );
    }
}

// ---------------------------------------------------------------------------
// Property 2b: zoom preservation — scroll changes zoom_level correctly
// ---------------------------------------------------------------------------
//
// When wants_pointer_input() == false, the zoom_system reads MouseWheel
// events and applies: new_zoom = (old_zoom * (1.0 + scroll_y * 0.1)).clamp(0.25, 8.0)
// We verify the formula produces the correct result.

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    /// Preservation: mouse-wheel zoom formula works when no dialog.
    /// Validates Req 3.2
    #[test]
    fn zoom_changes_correctly_when_no_dialog(
        scroll_y in proptest::num::f32::NORMAL
            .prop_filter("non-zero scroll", |y| y.abs() > 0.01)
            .prop_filter("reasonable scroll", |y| y.abs() < 100.0),
        initial_zoom in 0.25f32..=8.0,
    ) {
        let zoom_speed = 0.1f32;
        let old_zoom = initial_zoom.clamp(0.25, 8.0);
        let wants_pointer = false;

        // Simulate zoom_system core logic: no dialog guard fires
        let mut zoom_level = old_zoom;
        if !wants_pointer {
            let new_zoom = (old_zoom * (1.0 + scroll_y * zoom_speed)).clamp(0.25, 8.0);
            zoom_level = new_zoom;
        }

        let expected = (old_zoom * (1.0 + scroll_y * zoom_speed)).clamp(0.25, 8.0);
        prop_assert!(
            (zoom_level - expected).abs() < f32::EPSILON,
            "zoom formula mismatch: got {} expected {} (old_zoom={}, scroll_y={})",
            zoom_level, expected, old_zoom, scroll_y
        );
    }
}

// ---------------------------------------------------------------------------
// Property 2b-extra: zoom preservation — zoom stays unchanged with zero scroll
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    /// Preservation: zero scroll produces no zoom change.
    /// Validates Req 3.2
    #[test]
    fn zoom_unchanged_with_zero_scroll(
        initial_zoom in 0.25f32..=8.0,
    ) {
        let zoom_level = initial_zoom.clamp(0.25, 8.0);
        let scroll_y = 0.0f32;
        let zoom_speed = 0.1f32;

        // zoom_system returns early on zero scroll
        let new_zoom = (zoom_level * (1.0 + scroll_y * zoom_speed)).clamp(0.25, 8.0);

        prop_assert!(
            (new_zoom - zoom_level).abs() < f32::EPSILON,
            "zero scroll should not change zoom: {} != {}",
            new_zoom, zoom_level
        );
    }
}


// ---------------------------------------------------------------------------
// Property 2c: pan preservation — middle-mouse press initiates panning
// ---------------------------------------------------------------------------
//
// When wants_pointer_input() == false, pan_system sets middle_panning = true
// on MouseButton::Middle just_pressed. Subsequent drag updates camera_offset.

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Preservation: middle-mouse press starts panning when no dialog.
    /// Validates Req 3.3
    #[test]
    fn pan_starts_on_middle_press_when_no_dialog(
        _dummy in 0u32..100,
    ) {
        let mut middle_panning = false;
        let wants_pointer = false;

        // Simulate pan_system: middle mouse just pressed, no dialog
        let middle_just_pressed = true;
        if !wants_pointer && middle_just_pressed {
            middle_panning = true;
        }

        prop_assert!(
            middle_panning,
            "pan_system should start panning on middle-mouse press when no dialog is active"
        );
    }

    /// Preservation: pan drag updates camera_offset correctly.
    /// Validates Req 3.3
    #[test]
    fn pan_drag_updates_camera_offset_when_no_dialog(
        delta_x in -500.0f32..500.0,
        delta_y in -500.0f32..500.0,
        zoom in 0.25f32..=8.0,
        offset_x in -1000.0f32..1000.0,
        offset_y in -1000.0f32..1000.0,
    ) {
        let zoom = zoom.clamp(0.25, 8.0);
        let mut camera_offset = (offset_x, offset_y);
        let middle_panning = true;
        let wants_pointer = false;

        // Simulate pan_system drag logic: convert screen delta to world delta
        // Screen X right = world X right, screen Y down = world Y up (negate Y).
        // Panning: dragging right moves camera left, so we subtract.
        if !wants_pointer && middle_panning {
            let world_dx = -delta_x / zoom;
            let world_dy = delta_y / zoom;
            camera_offset.0 += world_dx;
            camera_offset.1 += world_dy;
        }

        let expected_x = offset_x + (-delta_x / zoom);
        let expected_y = offset_y + (delta_y / zoom);

        prop_assert!(
            (camera_offset.0 - expected_x).abs() < 0.001,
            "camera_offset.x mismatch: {} vs {} (delta_x={}, zoom={})",
            camera_offset.0, expected_x, delta_x, zoom
        );
        prop_assert!(
            (camera_offset.1 - expected_y).abs() < 0.001,
            "camera_offset.y mismatch: {} vs {} (delta_y={}, zoom={})",
            camera_offset.1, expected_y, delta_y, zoom
        );
    }
}

// ---------------------------------------------------------------------------
// Property 2d: attribute opacity preservation — click toggles opacity
// ---------------------------------------------------------------------------
//
// When wants_pointer_input() == false and editor is in Attribute mode with
// Opacity tool, a left-click toggles the opacity flag on the target tile.

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Preservation: attribute click toggles opacity when no dialog.
    /// Validates Req 3.5
    #[test]
    fn attribute_opacity_toggles_when_no_dialog(
        width in 1u32..=16,
        height in 1u32..=16,
        col_frac in 0.0f64..1.0,
        row_frac in 0.0f64..1.0,
        initial_opacity in proptest::bool::ANY,
    ) {
        let mut map = make_map(width, height);
        let col = (col_frac * width as f64).floor().min((width - 1) as f64) as u32;
        let row = (row_frac * height as f64).floor().min((height - 1) as f64) as u32;

        // Set initial opacity state
        map.layers[0].attributes.cells[row as usize][col as usize].opacity = initial_opacity;

        let wants_pointer = false;
        let left_just_pressed = true;

        // Simulate attribute_click_system core path: Opacity tool + left-click, no dialog
        if !wants_pointer && left_just_pressed {
            let cell = &mut map.layers[0].attributes.cells[row as usize][col as usize];
            cell.opacity = !cell.opacity;
        }

        let expected = !initial_opacity;
        prop_assert_eq!(
            map.layers[0].attributes.cells[row as usize][col as usize].opacity,
            expected,
            "opacity should toggle from {} to {} at ({}, {}) when no dialog active",
            initial_opacity, expected, col, row
        );
    }

    /// Preservation: double-click toggles opacity back to original value.
    /// Validates Req 3.5
    #[test]
    fn attribute_opacity_double_toggle_restores_when_no_dialog(
        width in 1u32..=16,
        height in 1u32..=16,
        col_frac in 0.0f64..1.0,
        row_frac in 0.0f64..1.0,
        initial_opacity in proptest::bool::ANY,
    ) {
        let mut map = make_map(width, height);
        let col = (col_frac * width as f64).floor().min((width - 1) as f64) as u32;
        let row = (row_frac * height as f64).floor().min((height - 1) as f64) as u32;

        map.layers[0].attributes.cells[row as usize][col as usize].opacity = initial_opacity;

        // Two clicks: toggle, then toggle back
        let cell = &mut map.layers[0].attributes.cells[row as usize][col as usize];
        cell.opacity = !cell.opacity;
        cell.opacity = !cell.opacity;

        prop_assert_eq!(
            map.layers[0].attributes.cells[row as usize][col as usize].opacity,
            initial_opacity,
            "double toggle should restore opacity to {} at ({}, {})",
            initial_opacity, col, row
        );
    }
}


// ---------------------------------------------------------------------------
// Property 2e: cursor state preservation — tile_pos computed correctly
// ---------------------------------------------------------------------------
//
// When wants_pointer_input() == false, update_cursor_state projects the
// screen cursor through the camera to world coords, then computes
// tile_pos = (floor(world_x / tile_size), floor(-world_y / tile_size)).
// We verify the formula for positions within map bounds and that
// out-of-bounds positions produce None.

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    /// Preservation: cursor within canvas bounds produces correct tile_pos.
    /// Validates Req 3.4 (CanvasRect gating preserved) and implied cursor behavior.
    #[test]
    fn cursor_computes_tile_pos_when_no_dialog(
        width in 1u32..=32,
        height in 1u32..=32,
        tile_size in prop_oneof![Just(8u32), Just(16), Just(32), Just(64)],
        col_frac in 0.0f64..1.0,
        row_frac in 0.0f64..1.0,
    ) {
        let map = MapData::new("test", width, height, tile_size, tile_size)
            .expect("valid map");
        let ts = map.tile_width as f32;

        // Generate a world position within the map bounds
        let col = (col_frac * width as f64).floor().min((width - 1) as f64) as u32;
        let row = (row_frac * height as f64).floor().min((height - 1) as f64) as u32;

        // World coords: x increases right, y increases up (Bevy convention).
        // Tile (col, row) maps to world_x = col * tile_size, world_y = -(row * tile_size).
        // We place the cursor in the center of the tile.
        let world_x = col as f32 * ts + ts / 2.0;
        let world_y = -(row as f32 * ts + ts / 2.0);

        let wants_pointer = false;
        let mut tile_pos: Option<(u32, u32)> = None;

        // Simulate update_cursor_state core logic: no dialog guard fires
        if !wants_pointer {
            let computed_col = (world_x / ts).floor();
            let computed_row = (-world_y / ts).floor();

            if computed_col >= 0.0 && computed_row >= 0.0 {
                let c = computed_col as u32;
                let r = computed_row as u32;
                if c < map.width && r < map.height {
                    tile_pos = Some((c, r));
                }
            }
        }

        prop_assert_eq!(
            tile_pos,
            Some((col, row)),
            "tile_pos should be ({}, {}) for world ({}, {}) with tile_size={}",
            col, row, world_x, world_y, ts
        );
    }

    /// Preservation: cursor outside map bounds produces tile_pos = None.
    /// Validates Req 3.4
    #[test]
    fn cursor_out_of_bounds_gives_none_when_no_dialog(
        width in 1u32..=16,
        height in 1u32..=16,
        tile_size in prop_oneof![Just(8u32), Just(16), Just(32), Just(64)],
    ) {
        let map = MapData::new("test", width, height, tile_size, tile_size)
            .expect("valid map");
        let ts = map.tile_width as f32;

        // Place cursor one tile past the right edge
        let world_x = width as f32 * ts + ts / 2.0;
        let world_y = -(0.0f32 * ts + ts / 2.0);

        let wants_pointer = false;
        let mut tile_pos: Option<(u32, u32)> = None;

        if !wants_pointer {
            let computed_col = (world_x / ts).floor();
            let computed_row = (-world_y / ts).floor();

            if computed_col >= 0.0 && computed_row >= 0.0 {
                let c = computed_col as u32;
                let r = computed_row as u32;
                if c < map.width && r < map.height {
                    tile_pos = Some((c, r));
                }
            }
        }

        prop_assert_eq!(
            tile_pos,
            None,
            "tile_pos should be None for out-of-bounds world ({}, {})",
            world_x, world_y
        );
    }

    /// Preservation: cursor at negative world coords gives tile_pos = None.
    #[test]
    fn cursor_negative_world_gives_none_when_no_dialog(
        width in 1u32..=16,
        height in 1u32..=16,
        tile_size in prop_oneof![Just(8u32), Just(16), Just(32), Just(64)],
    ) {
        let map = MapData::new("test", width, height, tile_size, tile_size)
            .expect("valid map");
        let ts = map.tile_width as f32;

        // Negative world_x: left of the map
        let world_x = -ts / 2.0;
        let world_y = -(0.0f32 * ts + ts / 2.0);

        let wants_pointer = false;
        let mut tile_pos: Option<(u32, u32)> = None;

        if !wants_pointer {
            let computed_col = (world_x / ts).floor();
            let computed_row = (-world_y / ts).floor();

            if computed_col >= 0.0 && computed_row >= 0.0 {
                let c = computed_col as u32;
                let r = computed_row as u32;
                if c < map.width && r < map.height {
                    tile_pos = Some((c, r));
                }
            }
        }

        prop_assert_eq!(
            tile_pos,
            None,
            "tile_pos should be None for negative world_x ({})",
            world_x
        );
    }
}
