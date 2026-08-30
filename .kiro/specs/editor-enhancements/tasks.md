# Implementation Plan: Editor Enhancements

## Overview

This plan implements four new capabilities across the RPG toolkit workspace: **Jump** event actions, **parallax background layers**, **hotkey bindings**, and **SetSpeed** event actions. The implementation progresses from data models in `rpg-toolkit-common`, through runtime systems in `rpg-toolkit-renderer`, to editor UI in `rpg-toolkit-editor`, ensuring each step builds on the previous and ends fully integrated.

## Tasks

- [x] 1. Add Jump and SetSpeed EventAction variants and validation to rpg-toolkit-common
  - [x] 1.1 Add `Jump` variant to `EventAction` enum with serde validation
    - In `crates/rpg-toolkit-common/src/map.rs`, add `Jump { distance: u32 }` variant to the `EventAction` enum
    - Add `deserialize_jump_distance` validator function that rejects values outside [1, 8] with error message "distance must be between 1 and 8 inclusive, got {value}"
    - Apply `#[serde(deserialize_with = "deserialize_jump_distance")]` to the `distance` field
    - Add unit test verifying missing `distance` field returns deserialization error
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 1.6_

  - [x] 1.2 Add `SetSpeed` variant to `EventAction` enum with serde validation
    - In `crates/rpg-toolkit-common/src/map.rs`, add `SetSpeed { multiplier: f32 }` variant to the `EventAction` enum
    - Add `default_speed_multiplier` function returning `1.0`
    - Add `deserialize_speed_multiplier` validator function that rejects values outside [0.5, 4.0] with error message "multiplier must be between 0.5 and 4.0 inclusive, got {value}"
    - Apply `#[serde(default = "default_speed_multiplier", deserialize_with = "deserialize_speed_multiplier")]` to the `multiplier` field
    - Add unit test verifying `multiplier` defaults to 1.0 when absent from JSON
    - _Requirements: 9.1, 9.2, 9.3, 9.4, 9.5, 9.6_

  - [x] 1.3 Write property test for Jump round-trip serialization
    - **Property 1: Jump EventAction Serialization Round-Trip**
    - **Validates: Requirements 1.1, 1.2, 1.5, 1.6**
    - In `tests/properties/`, create `jump_round_trip.rs` using proptest to generate `distance` in [1, 8], serialize to JSON, deserialize back, and assert `PartialEq` equality

  - [x]* 1.4 Write property test for Jump invalid distance rejection
    - **Property 2: Jump Invalid Distance Rejection**
    - **Validates: Requirements 1.3**
    - In `tests/properties/`, create `jump_invalid_distance.rs` using proptest to generate `u32` values outside [1, 8], attempt deserialization, and assert error is returned

  - [x]* 1.5 Write property test for SetSpeed round-trip serialization
    - **Property 13: SetSpeed Serialization Round-Trip**
    - **Validates: Requirements 9.1, 9.2, 9.4, 9.5**
    - In `tests/properties/`, create `set_speed_round_trip.rs` using proptest to generate `multiplier` in [0.5, 4.0], serialize/deserialize, assert equality

  - [x]* 1.6 Write property test for SetSpeed invalid multiplier rejection
    - **Property 14: SetSpeed Invalid Multiplier Rejection**
    - **Validates: Requirements 9.3**
    - In `tests/properties/`, create `set_speed_invalid_multiplier.rs` using proptest to generate `f32` outside [0.5, 4.0], attempt deserialization, assert error

- [x] 2. Add ParallaxLayer and HotkeyBinding data models to rpg-toolkit-common
  - [x] 2.1 Add `ParallaxLayer` struct with validation
    - In `crates/rpg-toolkit-common/src/map.rs`, add `ParallaxLayer` struct with `image_path: String`, `scroll_factor: f32`, `z_order: i32`
    - Implement `TryFrom<RawParallaxLayer>` with validation: `image_path` 1–256 chars, `scroll_factor` in [0.0, 1.0]
    - Add `#[serde(default)]` `parallax_layers: Vec<ParallaxLayer>` field to `MapData`
    - Add unit test verifying `parallax_layers` defaults to empty Vec when absent
    - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5, 3.6, 3.7, 12.2_

  - [x] 2.2 Add `HotkeyBinding` struct with validation and `hotkey_bindings` field to `ProjectFile`
    - In a new file `crates/rpg-toolkit-common/src/hotkey.rs` (or in `project.rs`), add `HotkeyBinding` struct with `key_code: String`, `name: String`, `event_actions: Vec<EventAction>`
    - Implement `TryFrom<RawHotkeyBinding>` with validation: `key_code` 1–64 chars, `name` 1–64 chars, `event_actions` 0–20 entries
    - Add `deserialize_hotkey_bindings` custom deserializer for `Vec<HotkeyBinding>` that enforces max 32 entries and unique `key_code` values
    - Add `#[serde(default, deserialize_with = "deserialize_hotkey_bindings")] pub hotkey_bindings: Vec<HotkeyBinding>` field to `ProjectFile`
    - Add unit test verifying `hotkey_bindings` defaults to empty Vec when absent
    - _Requirements: 6.1, 6.2, 6.3, 6.4, 6.5, 6.6, 6.7, 6.8, 6.9, 12.3_

  - [x] 2.3 Write property test for ParallaxLayer validation and round-trip
    - **Property 5: ParallaxLayer Validation Acceptance**
    - **Property 7: ParallaxLayer Round-Trip**
    - **Validates: Requirements 3.2, 3.3, 3.5, 3.1, 3.4, 3.7**
    - In `tests/properties/`, create `parallax_layer_round_trip.rs`

  - [x] 2.4 Write property test for ParallaxLayer invalid scroll_factor rejection
    - **Property 6: ParallaxLayer Invalid scroll_factor Rejection**
    - **Validates: Requirements 3.6**
    - In `tests/properties/`, create `parallax_layer_invalid_scroll_factor.rs`

  - [ ]* 2.5 Write property test for HotkeyBinding round-trip
    - **Property 10: HotkeyBinding Serialization Round-Trip**
    - **Validates: Requirements 6.1, 6.2, 6.3, 6.4, 6.5, 6.8**
    - In `tests/properties/`, create `hotkey_binding_round_trip.rs`

  - [ ]* 2.6 Write property test for HotkeyBinding invalid field rejection
    - **Property 11: HotkeyBinding Invalid Field Rejection**
    - **Validates: Requirements 6.6, 6.7**
    - In `tests/properties/`, create `hotkey_binding_invalid_fields.rs`

  - [x] 2.7 Write property test for HotkeyBinding duplicate key_code rejection
    - **Property 12: HotkeyBinding Duplicate key_code Rejection**
    - **Validates: Requirements 6.9**
    - In `tests/properties/`, create `hotkey_binding_duplicate_keycode.rs`

- [~] 3. Checkpoint - Ensure all data model tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [x] 4. Implement Jump runtime systems in rpg-toolkit-renderer
  - [~] 4.1 Add `JumpAnimState` resource and `compute_landing` function
    - In `crates/rpg-toolkit-renderer/src/`, create a `jump.rs` module (or add to existing movement module)
    - Define `JumpAnimState` resource with fields: `start_x`, `start_y`, `landing_x`, `landing_y`, `distance`, `duration`, `elapsed`
    - Implement `compute_landing(grid_x, grid_y, facing, distance, map_width, map_height) -> (u32, u32)` with bounds clamping
    - Implement `jump_arc_offset(t: f32, distance: u32, tile_height: f32) -> f32` using the parabolic formula `4.0 * peak * t * (1.0 - t)`
    - _Requirements: 2.1, 2.3, 2.4_

  - [~] 4.2 Implement `jump_animation_system`
    - Add a Bevy system that runs in `Update` schedule
    - Each frame: advance `elapsed` by delta time, compute `t = elapsed / duration`, apply parabolic offset to player transform
    - On completion (`elapsed >= duration`): update `PlayerCharacter` grid position to landing tile, remove `JumpAnimState` resource, fire landing tile's event trigger if present
    - While `JumpAnimState` exists: block player input and ActionQueue advancement
    - Wire the system into the renderer plugin
    - _Requirements: 2.2, 2.4, 2.5, 2.6, 2.7_

  - [~] 4.3 Integrate Jump action handling into ActionQueue processor
    - In the ActionQueue's action dispatch logic, add a match arm for `EventAction::Jump`
    - On dequeue: compute landing tile via `compute_landing`, insert `JumpAnimState` resource with calculated duration
    - Mark as blocking action so queue waits for animation completion
    - _Requirements: 2.1, 2.5_

  - [ ]* 4.4 Write property test for Jump landing computation with bounds clamping
    - **Property 3: Jump Landing Computation with Bounds Clamping**
    - **Validates: Requirements 2.1, 2.3**
    - In `crates/rpg-toolkit-renderer/tests/properties/`, create `jump_landing_computation.rs`

  - [x] 4.5 Write property test for Jump parabolic offset invariant
    - **Property 4: Jump Parabolic Offset Invariant**
    - **Validates: Requirements 2.4**
    - In `crates/rpg-toolkit-renderer/tests/properties/`, create `jump_arc_offset.rs`

- [x] 5. Implement SetSpeed runtime systems in rpg-toolkit-renderer
  - [x] 5.1 Add `SpeedMultiplier` resource and `apply_speed_multiplier_system`
    - Define `SpeedMultiplier` resource with `value: f32` defaulting to 1.0
    - Initialize `SpeedMultiplier` at renderer startup with value 1.0
    - Implement `apply_speed_multiplier_system` in `Update` schedule: sets `MovementConfig.move_duration = 0.15 / speed_multiplier.value.clamp(0.5, 4.0)`
    - Wire into the renderer plugin
    - _Requirements: 10.1, 10.2, 10.3, 10.5_

  - [x] 5.2 Integrate SetSpeed action handling into ActionQueue processor
    - In the ActionQueue's action dispatch logic, add a match arm for `EventAction::SetSpeed`
    - On dequeue: update `SpeedMultiplier.value` to the action's `multiplier` value
    - Mark as non-blocking so queue advances immediately
    - _Requirements: 10.1, 10.4, 10.6_

  - [x] 5.3 Write property test for speed-adjusted move duration computation
    - **Property 15: Speed-Adjusted Move Duration Computation**
    - **Validates: Requirements 10.2**
    - In `crates/rpg-toolkit-renderer/tests/properties/`, create `speed_move_duration.rs`

- [x] 6. Implement Parallax rendering systems in rpg-toolkit-renderer
  - [x] 6.1 Add `ParallaxSprite` component and spawn/despawn systems
    - Define `ParallaxSprite` component with `scroll_factor: f32` and `layer_index: usize`
    - Implement `spawn_parallax_system`: on map load, iterate `MapData.parallax_layers`, attempt to load image at `image_path`, spawn sprite entity with `Transform.translation.z` computed from `z_order` (all < 0.0, stable sort by `(z_order, list_index)`), log warning and skip if image not found
    - Implement `despawn_parallax_system`: on map change, despawn all entities with `ParallaxSprite` component
    - Wire systems into the renderer plugin with appropriate scheduling
    - _Requirements: 4.1, 4.3, 4.4, 4.5, 4.6, 4.7_

  - [x] 6.2 Implement `update_parallax_system`
    - Add a Bevy system that runs in `Update` schedule
    - Each frame: compute camera position delta, for each `ParallaxSprite` entity, translate by `delta * scroll_factor`
    - `scroll_factor` of 0.0 → no movement; `scroll_factor` of 1.0 → full camera movement
    - _Requirements: 4.2_

  - [x] 6.3 Write property test for parallax scroll translation computation
    - **Property 8: Parallax Scroll Translation Computation**
    - **Validates: Requirements 4.2**
    - In `crates/rpg-toolkit-renderer/tests/properties/`, create `parallax_scroll_translation.rs`

  - [ ]* 6.4 Write property test for parallax z-order stable sort
    - **Property 9: Parallax z-order Stable Sort**
    - **Validates: Requirements 4.4, 4.7**
    - In `crates/rpg-toolkit-renderer/tests/properties/`, create `parallax_z_order.rs`

- [x] 7. Implement Hotkey runtime system in rpg-toolkit-renderer
  - [~] 7.1 Implement `hotkey_input_system`
    - Add a Bevy system running in `Update` schedule
    - Guard conditions: only fire when `AppPhase` is `InGame` AND no `ActionQueue` resource AND no `DialogState` resource AND no `SelectionState` resource
    - Each frame: check `ButtonInput<KeyCode>` for `just_pressed` keys matching configured bindings
    - On match: push the binding's `event_actions` into a new `ActionQueue` (first match wins)
    - If `event_actions` is empty, treat as no-op
    - Wire into the renderer plugin
    - _Requirements: 7.1, 7.2, 7.3, 7.4, 7.5, 7.6_

- [~] 8. Checkpoint - Ensure all renderer tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [x] 9. Add Jump and SetSpeed editor forms in rpg-toolkit-editor
  - [x] 9.1 Add `Jump` and `SetSpeed` to action type dropdown and implement form renderers
    - In `crates/rpg-toolkit-editor/src/plugins/attribute/action_editor.rs`, add `Jump` and `SetSpeed` entries to the action type enum/dropdown
    - Add `jump_distance: String` field (default "2") and `speed_multiplier: f32` field (default 1.0) to `ActionEditorState`
    - In `action_editor_forms.rs`, implement `render_jump_form`: numeric input for distance with range hint [1, 8], clamp on Add/Update, reset to 2 after submission
    - In `action_editor_forms.rs`, implement `render_set_speed_form`: slider for multiplier [0.5, 4.0] step 0.1, clamp on Add/Update, reset to 1.0 after submission
    - _Requirements: 11.1, 11.2, 11.3, 11.4, 11.5, 11.6, 11.7_

  - [x] 9.2 Write property test for editor value clamping
    - **Property 16: Editor Value Clamping**
    - **Validates: Requirements 11.4, 11.5**
    - In `crates/rpg-toolkit-editor/tests/properties/` (or `tests/properties/`), create `editor_value_clamping.rs`

- [x] 10. Add Parallax Panel to rpg-toolkit-editor
  - [~] 10.1 Implement `ParallaxPanel` plugin in the editor
    - Create `crates/rpg-toolkit-editor/src/plugins/parallax_panel.rs`
    - Display list of parallax layers for the active map in order
    - "Add Layer" button: appends default entry (empty `image_path`, `scroll_factor` 0.5, `z_order` 0), disabled at 16 layers
    - Per-row controls: file picker for `image_path`, slider for `scroll_factor` (0.0–1.0, step 0.05), DragValue for `z_order` (-999–999)
    - "Remove" button per row, no confirmation
    - Validation warning when `image_path` is empty on save (non-blocking)
    - Register plugin in the editor app
    - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5, 5.6_

- [x] 11. Add Hotkey Bindings Panel to rpg-toolkit-editor
  - [x] 11.1 Implement `HotkeyBindingsPanel` plugin in the editor
    - Create `crates/rpg-toolkit-editor/src/plugins/hotkey_panel.rs`
    - Display in project settings area, list all hotkey bindings
    - "Add Binding" button: creates entry with empty defaults
    - Per-entry: key capture input for `key_code`, text input for `name` (64 char limit), embedded event action list editor (reuse existing pattern)
    - Arrow buttons (↑/↓) or drag-and-drop for reordering
    - "Remove" button per entry
    - Save disabled when `key_code` or `name` is empty, show validation message
    - Register plugin in the editor app
    - _Requirements: 8.1, 8.2, 8.3, 8.4, 8.5, 8.6, 8.7, 8.8_

- [x] 12. Comprehensive serialization compatibility integration
  - [x] 12.1 Add unit tests for backward compatibility and unknown variant handling
    - Test loading a project file with no `parallax_layers` or `hotkey_bindings` fields (verify defaults)
    - Test loading project with only pre-existing action types (verify no errors)
    - Test that an unrecognized EventAction `"type"` tag returns a deserialization error
    - _Requirements: 12.1, 12.2, 12.3, 12.5_

  - [ ]* 12.2 Write property test for ProjectFile comprehensive round-trip
    - **Property 17: ProjectFile Comprehensive Round-Trip**
    - **Validates: Requirements 12.1, 12.4**
    - In `tests/properties/`, create `project_file_comprehensive_round_trip.rs` testing full project serialization with mixed old and new variants, parallax layers, and hotkey bindings

- [~] 13. Final checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

## Notes

- Tasks marked with `*` are optional and can be skipped for faster MVP
- Each task references specific requirements for traceability
- Checkpoints ensure incremental validation
- Property tests validate universal correctness properties from the design document
- Unit tests validate specific examples and edge cases
- The implementation language is Rust, matching the existing workspace
- `proptest` is already a workspace dependency — no additional test framework setup needed
- Property tests go in `tests/properties/` directories following the existing pattern in `rpg-toolkit-common`

## Task Dependency Graph

```json
{
  "waves": [
    { "id": 0, "tasks": ["1.1", "1.2"] },
    { "id": 1, "tasks": ["1.3", "1.4", "1.5", "1.6", "2.1", "2.2"] },
    { "id": 2, "tasks": ["2.3", "2.4", "2.5", "2.6", "2.7"] },
    { "id": 3, "tasks": ["4.1", "5.1"] },
    { "id": 4, "tasks": ["4.2", "4.3", "4.4", "4.5", "5.2", "5.3", "6.1"] },
    { "id": 5, "tasks": ["6.2", "6.3", "6.4", "7.1"] },
    { "id": 6, "tasks": ["9.1", "10.1", "11.1"] },
    { "id": 7, "tasks": ["9.2", "12.1", "12.2"] }
  ]
}
```
