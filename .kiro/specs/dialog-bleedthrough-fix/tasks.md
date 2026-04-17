# Implementation Plan

- [x] 1. Write bug condition exploration test
  - **Property 1: Bug Condition** - Dialog Bleedthrough to Canvas Systems
  - **CRITICAL**: This test MUST FAIL on unfixed code — failure confirms the bug exists
  - **DO NOT attempt to fix the test or the code when it fails**
  - **NOTE**: This test encodes the expected behavior — it will validate the fix when it passes after implementation
  - **GOAL**: Surface counterexamples that demonstrate canvas systems process mouse input when egui is consuming it
  - **Scoped PBT Approach**: For each of the five affected systems, scope the property to concrete failing cases where `wants_pointer_input() == true` and canvas state should not mutate
  - Create a property-based test in `tests/properties/` that:
    - Sets up a minimal Bevy app with the editor systems registered
    - For `painting_system`: with an active brush and valid tile_pos, simulate left-click while `wants_pointer_input()` is true → assert no tile placement occurs
    - For `zoom_system`: fire a `MouseWheel` event while `wants_pointer_input()` is true → assert zoom_level is unchanged
    - For `pan_system`: simulate `MouseButton::Middle` press while `wants_pointer_input()` is true → assert `PanState.middle_panning` remains false
    - For `attribute_click_system`: in attribute mode with opacity tool, simulate left-click while a non-attribute dialog (e.g. Load Tileset) is open → assert no opacity toggle
    - For `update_cursor_state`: simulate cursor over canvas while `wants_pointer_input()` is true → assert `tile_pos` remains `None`
  - The test assertions should match the Expected Behavior from the design: when `isBugCondition(input)` is true, canvas state must not change
  - Run test on UNFIXED code
  - **EXPECTED OUTCOME**: Test FAILS (this is correct — it proves the bug exists, i.e., canvas state mutates when it shouldn't)
  - Document counterexamples found to understand root cause
  - Mark task complete when test is written, run, and failure is documented
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 1.6_

- [x] 2. Write preservation property tests (BEFORE implementing fix)
  - **Property 2: Preservation** - Normal Canvas Interactions Unchanged
  - **IMPORTANT**: Follow observation-first methodology
  - Observe behavior on UNFIXED code for non-buggy inputs (where `wants_pointer_input() == false`):
    - Observe: painting with active brush + left-click + valid tile_pos places tiles
    - Observe: mouse-wheel scroll changes zoom_level proportionally
    - Observe: middle-mouse press initiates panning, drag updates camera_offset
    - Observe: attribute mode left-click toggles opacity on the target tile
    - Observe: cursor over canvas with no dialog sets tile_pos correctly
  - Write property-based tests in `tests/properties/` capturing observed behavior:
    - For all `(EditorTool, brush, tile_pos)` combinations with `wants_pointer_input() == false`, painting produces expected tile mutations
    - For all `(scroll_delta, zoom_level)` with no dialog active, zoom changes match `(old_zoom * (1.0 + scroll_y * 0.1)).clamp(0.25, 8.0)`
    - For all middle-mouse drags with no dialog active, `PanState` and `camera_offset` update correctly
    - For all attribute clicks with no dialog active, opacity toggles as expected
    - For all cursor positions within canvas bounds with no dialog active, `tile_pos` is computed correctly
  - Verify tests pass on UNFIXED code
  - **EXPECTED OUTCOME**: Tests PASS (this confirms baseline behavior to preserve)
  - Mark task complete when tests are written, run, and passing on unfixed code
  - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5, 3.6_

- [x] 3. Fix dialog bleedthrough in canvas interaction systems

  - [x] 3.1 Add `wants_pointer_input()` guard to `painting_system`
    - In `crates/rpg-toolkit-editor/src/plugins/painting.rs`, add `EguiContexts` parameter to `painting_system`
    - Add early return at the top of the function (before any `ButtonInput<MouseButton>` reads): `if let Ok(ctx) = contexts.ctx_mut() && ctx.wants_pointer_input() { return; }`
    - This blocks tile painting, erasing, flood fill, and stamp brush when any egui dialog consumes pointer input
    - _Bug_Condition: isBugCondition(input) where wants_pointer_input() == true AND painting_system processes the click_
    - _Expected_Behavior: painting_system returns early, no canvas tile mutations_
    - _Preservation: When wants_pointer_input() == false, painting behavior is identical to unfixed code_
    - _Requirements: 1.1, 1.2, 1.3, 2.1, 2.2, 2.3_

  - [x] 3.2 Add `wants_pointer_input()` guard to `zoom_system`
    - In `crates/rpg-toolkit-editor/src/systems/camera.rs`, add `EguiContexts` parameter to `zoom_system`
    - Add early return at the top of the function (before reading `MouseWheel` events): `if let Ok(ctx) = contexts.ctx_mut() && ctx.wants_pointer_input() { return; }`
    - This blocks scroll-wheel zoom when any egui dialog consumes pointer input
    - _Bug_Condition: isBugCondition(input) where wants_pointer_input() == true AND zoom_system processes the scroll_
    - _Expected_Behavior: zoom_system returns early, zoom_level unchanged_
    - _Preservation: When wants_pointer_input() == false, zoom behavior is identical to unfixed code_
    - _Requirements: 1.4, 2.4_

  - [x] 3.3 Add `wants_pointer_input()` guard to `pan_system`
    - In `crates/rpg-toolkit-editor/src/systems/camera.rs`, add `EguiContexts` parameter to `pan_system`
    - Add early return at the top of the function (before reading `ButtonInput<MouseButton>`): `if let Ok(ctx) = contexts.ctx_mut() && ctx.wants_pointer_input() { ... }`
    - When egui wants pointer input and a pan is in progress, cancel the active pan by resetting `PanState` (prevents stuck pan state)
    - When egui wants pointer input and no pan is in progress, return early
    - _Bug_Condition: isBugCondition(input) where wants_pointer_input() == true AND pan_system processes the drag_
    - _Expected_Behavior: pan_system returns early, PanState not activated, active pan cancelled_
    - _Preservation: When wants_pointer_input() == false, pan behavior is identical to unfixed code_
    - _Requirements: 1.5, 2.5_

  - [x] 3.4 Replace partial guard in `attribute_click_system` with universal `wants_pointer_input()` check
    - In `crates/rpg-toolkit-editor/src/plugins/attribute.rs`, replace the existing partial guard (which checks `event_trigger_dialog.open || spawn_confirm_dialog.open || npc_placement_dialog.open` combined with `ctx.is_pointer_over_area()`) with a single `if let Ok(ctx) = contexts.ctx_mut() && ctx.wants_pointer_input() { return; }` at the top of the function
    - Remove the `EguiContexts` parameter that was only used for the old partial guard (it stays but the usage changes)
    - This covers ALL current and future egui dialogs without enumerating them
    - _Bug_Condition: isBugCondition(input) where wants_pointer_input() == true AND attribute_click_system processes the click_
    - _Expected_Behavior: attribute_click_system returns early, no opacity toggle / event trigger open / spawn point / NPC placement_
    - _Preservation: When wants_pointer_input() == false, attribute behavior is identical to unfixed code_
    - _Requirements: 1.6, 2.6_

  - [x] 3.5 Add `wants_pointer_input()` guard to `update_cursor_state`
    - In `crates/rpg-toolkit-editor/src/systems/input.rs`, add `EguiContexts` parameter to `update_cursor_state`
    - Add early return after resetting `cursor_state` fields to `None`: `if let Ok(ctx) = contexts.ctx_mut() && ctx.wants_pointer_input() { return; }`
    - This prevents `tile_pos` from being set when egui consumes pointer input, so downstream systems cannot act on stale cursor state
    - _Bug_Condition: isBugCondition(input) where wants_pointer_input() == true AND update_cursor_state sets tile_pos_
    - _Expected_Behavior: update_cursor_state returns early, tile_pos stays None_
    - _Preservation: When wants_pointer_input() == false, cursor state behavior is identical to unfixed code. Existing CanvasRect gating is preserved._
    - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 3.4_

  - [x] 3.6 Verify bug condition exploration test now passes
    - **Property 1: Expected Behavior** - Dialog Blocks Canvas Mouse Input
    - **IMPORTANT**: Re-run the SAME test from task 1 — do NOT write a new test
    - The test from task 1 encodes the expected behavior: when `wants_pointer_input()` is true, canvas state must not change
    - When this test passes, it confirms the expected behavior is satisfied for all five systems
    - Run bug condition exploration test from step 1
    - **EXPECTED OUTCOME**: Test PASSES (confirms bug is fixed)
    - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 2.6_

  - [x] 3.7 Verify preservation tests still pass
    - **Property 2: Preservation** - Normal Canvas Interactions Unchanged
    - **IMPORTANT**: Re-run the SAME tests from task 2 — do NOT write new tests
    - Run preservation property tests from step 2
    - **EXPECTED OUTCOME**: Tests PASS (confirms no regressions)
    - Confirm all normal canvas interactions (painting, zoom, pan, attribute, cursor) still work identically
    - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5, 3.6_

- [x] 4. Checkpoint — Ensure all tests pass
  - Run the full test suite to confirm both property tests pass and no other tests are broken
  - Ensure all tests pass, ask the user if questions arise
