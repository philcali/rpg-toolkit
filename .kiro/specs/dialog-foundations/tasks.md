# Implementation Plan: Dialog Foundations

## Overview

Implement a foundational dialog system for the RPG toolkit's game renderer. The system is event-driven: a `ShowDialog` message spawns a Bevy UI overlay with configurable typewriter text reveal, positional placement, and optional movement blocking. Dialog text can be inline or referenced from a `DialogTextRegistry` for future localization. Implementation proceeds incrementally: data types and pure functions first, then ECS resources and systems, then integration into the existing plugin, and finally property-based tests.

## Tasks

- [x] 1. Define dialog data types and pure functions
  - [x] 1.1 Create the dialog module file and add serde-compatible data types
    - Create `crates/rpg-toolkit-renderer/src/dialog.rs` and register it in `lib.rs`
    - Define `DialogPosition` enum (`Top`, `Center`, `Bottom`) with `Default` (Bottom), `Clone`, `Debug`, `PartialEq`, `Eq`, `Serialize`, `Deserialize`
    - Define `DialogConfig` struct with `text_speed: f32`, `position: DialogPosition`, `movement_block: bool`, serde defaults (`text_speed: 30.0`, `position: Bottom`, `movement_block: true`), and `Default` impl
    - Define `DialogText` enum (`Inline(String)`, `Id(String)`) with serde tagged representation `#[serde(tag = "type", content = "value")]`, `Clone`, `Debug`, `PartialEq`, `Eq`, `Serialize`, `Deserialize`
    - Define `DialogTextRegistry` resource as a newtype around `HashMap<String, String>` with `new()`, `from_map()`, `insert()`, `get()`, `remove()`, `from_json()` methods, deriving `Resource`, `Clone`, `Debug`, `Default`, `PartialEq`, `Serialize`, `Deserialize`
    - Define `compute_visible_chars(elapsed: f32, text_speed: f32, total_chars: usize) -> usize` pure function
    - Add `serde` and `serde_json` dependencies to `crates/rpg-toolkit-renderer/Cargo.toml`
    - _Requirements: 1.1, 1.6, 1.7, 3.4, 3.5, 6.4, 8.1, 8.2, 8.3, 8.4, 8.5, 8.6, 8.7, 8.8, 9.1, 9.2, 9.3, 9.4, 9.5_

  - [ ]* 1.2 Write property tests for `compute_visible_chars`
    - **Property 1: Typewriter visible character computation**
    - Test that for any non-negative elapsed, non-negative text_speed, and total_chars, the function returns `min(floor(elapsed * text_speed), total_chars)` when speed > 0, and `total_chars` when speed == 0, and result is always in `0..=total_chars`
    - Add test file `tests/properties/dialog_typewriter.rs` and register in `tests/properties/Cargo.toml`
    - **Validates: Requirements 3.1, 3.2, 3.4, 3.5**

  - [ ]* 1.3 Write property tests for `DialogConfig` serde round-trip and defaults
    - **Property 5: DialogConfig serde round-trip**
    - Test that for any valid `DialogConfig`, serializing to JSON and deserializing produces an equal value
    - **Property 6: DialogConfig default deserialization**
    - Test that for any subset of fields present in JSON, absent fields use defaults (`text_speed: 30.0`, `position: Bottom`, `movement_block: true`)
    - Add test file `tests/properties/dialog_config_serde.rs` and register in `tests/properties/Cargo.toml`
    - **Validates: Requirements 8.2, 8.3, 8.5**

  - [ ]* 1.4 Write property tests for `DialogTextRegistry` serde round-trip and CRUD semantics
    - **Property 7: DialogTextRegistry serde round-trip**
    - Test that for any valid registry, serializing to JSON and deserializing produces an equal value
    - **Property 8: Registry insert-get-remove semantics**
    - Test that insert/get/remove sequences behave identically to `HashMap<String, String>`
    - Add test file `tests/properties/dialog_registry.rs` and register in `tests/properties/Cargo.toml`
    - **Validates: Requirements 8.7, 9.2, 9.3, 9.4, 9.5, 1.3, 1.4**

- [x] 2. Implement dialog ECS components, resources, and message
  - [x] 2.1 Define `DialogState` resource and marker components
    - Define `DialogState` resource in `dialog.rs` with fields: `full_text: String`, `total_chars: usize`, `chars_revealed: usize`, `fully_revealed: bool`, `elapsed: f32`, `text_speed: f32`, `movement_blocked: bool`
    - Define `DialogBox` marker component and `DialogTextNode` marker component
    - _Requirements: 7.1, 7.2, 7.3, 7.4, 7.5_

  - [x] 2.2 Define `ShowDialog` message type
    - Define `ShowDialog` struct with `text: DialogText` and `config: DialogConfig`, deriving `Message`
    - Register in `events.rs` or `dialog.rs` (follow existing pattern from `MapChanged`/`PlayerMoved`)
    - _Requirements: 1.1, 1.6_

  - [ ]* 2.3 Write property test for `DialogState` fully_revealed flag invariant
    - **Property 9: Dialog state fully_revealed flag invariant**
    - Test that `fully_revealed` is `true` if and only if `chars_revealed >= total_chars`
    - Add to `tests/properties/dialog_typewriter.rs`
    - **Validates: Requirements 7.4**

- [x] 3. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [x] 4. Implement dialog systems
  - [x] 4.1 Implement `handle_dialog_event` system
    - Create `crates/rpg-toolkit-renderer/src/systems/dialog.rs` and register in `systems/mod.rs`
    - Read `ShowDialog` messages via `MessageReader<ShowDialog>`
    - If `DialogState` already exists, ignore the event (log at debug level)
    - Resolve text: for `DialogText::Inline`, use the string directly; for `DialogText::Id`, look up in `Option<Res<DialogTextRegistry>>` — if registry missing or key not found, log warning and ignore
    - Spawn Bevy UI entities: root `Node` with `DialogBox` marker (semi-transparent background, ~80% screen width, positioned per `DialogPosition`), child `Text` node with `DialogTextNode` marker
    - Insert `DialogState` resource with initial values (`chars_revealed: 0`, `fully_revealed` set based on whether `text_speed` is 0, `elapsed: 0.0`)
    - Handle empty text: spawn dialog, immediately set `fully_revealed: true`
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 2.1, 2.2, 2.3, 2.4, 2.5, 6.1, 6.2, 6.3, 7.1_

  - [x] 4.2 Implement `update_dialog_typewriter` system
    - Advance `elapsed` by `time.delta_secs()` each frame
    - Compute `chars_revealed` using `compute_visible_chars(elapsed, text_speed, total_chars)`
    - Update `fully_revealed` flag: `chars_revealed >= total_chars`
    - Update the `Text` content on the `DialogTextNode` entity to show only the first `chars_revealed` characters of `full_text`
    - If `text_speed <= 0`, set `chars_revealed = total_chars` and `fully_revealed = true` immediately
    - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5, 7.3, 7.4_

  - [ ]* 4.3 Write property test for advance input completing typewriter
    - **Property 2: Advance input completes typewriter**
    - Test that for any `DialogState` where `chars_revealed < total_chars`, applying the advance action sets `chars_revealed == total_chars` and `fully_revealed == true`
    - Add to `tests/properties/dialog_typewriter.rs`
    - **Validates: Requirements 4.1**

  - [x] 4.4 Implement `handle_dialog_input` system
    - Read `ButtonInput<KeyCode>` for Space and Enter (`just_pressed`)
    - If dialog is active and typewriter is still animating (`!fully_revealed`): set `chars_revealed = total_chars`, `fully_revealed = true`, update text node to show full text
    - If dialog is active and text is fully revealed: despawn all entities with `DialogBox` component (recursively), remove `DialogState` resource
    - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5, 7.2_

- [x] 5. Implement movement blocking
  - [x] 5.1 Add movement blocking guard to `player_movement` system
    - Add `dialog_state: Option<Res<DialogState>>` parameter to `player_movement`
    - At the top of the function, if `dialog_state` is `Some` and `movement_blocked` is `true`, early-return before processing any movement intent
    - Existing movement animation in `animate_player` is unaffected (it does not check `DialogState`)
    - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5_

  - [ ]* 5.2 Write property test for movement blocking respecting config flag
    - **Property 3: Movement blocking respects config flag**
    - Test that movement initiation is suppressed iff dialog is active with `movement_blocked == true`, and processed normally when `movement_blocked == false`
    - Add test file `tests/properties/dialog_movement.rs` and register in `tests/properties/Cargo.toml`
    - **Validates: Requirements 5.1, 5.2**

  - [ ]* 5.3 Write property test for in-progress animation completing despite movement block
    - **Property 4: In-progress animation completes despite movement block**
    - Test that an in-progress `MoveAnimation` continues advancing by delta time and completes when `elapsed >= duration`, regardless of `movement_blocked` state
    - Add to `tests/properties/dialog_movement.rs`
    - **Validates: Requirements 5.3**

- [x] 6. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [x] 7. Register dialog systems in the plugin
  - [x] 7.1 Wire dialog systems and resources into `ProjectRendererPlugin`
    - In `ProjectRendererPlugin::build` in `lib.rs`:
      - Add `.init_resource::<DialogTextRegistry>()`
      - Add `.add_message::<ShowDialog>()`
      - Add dialog systems to `Update` schedule: `handle_dialog_event`, `update_dialog_typewriter.after(handle_dialog_event)`, `handle_dialog_input.after(update_dialog_typewriter)`
    - Update `pub use` exports in `lib.rs` for all new public types and systems
    - _Requirements: 1.1, 1.2, 2.1, 3.1, 4.1, 4.2, 5.1, 6.1, 7.1, 9.1, 9.6_

- [x] 8. Final checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

## Notes

- Tasks marked with `*` are optional and can be skipped for faster MVP
- Each task references specific requirements for traceability
- Checkpoints ensure incremental validation
- Property tests validate universal correctness properties from the design document (Properties 1–9)
- Unit tests validate specific examples and edge cases
- The `tests/properties/` directory already has `rpg-toolkit-renderer` as a dev-dependency pattern to follow; new property test files need `rpg-toolkit-renderer` added to `tests/properties/Cargo.toml` dev-dependencies
- All dialog systems run in the `Update` schedule alongside existing systems, with explicit ordering constraints
- `DialogState` is an optional resource — its presence/absence indicates dialog activity
