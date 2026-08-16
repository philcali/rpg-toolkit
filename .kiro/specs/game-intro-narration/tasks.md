# Implementation Plan: Game Intro Narration

## Overview

This plan implements cinematic intro narration for new games by adding four new `EventAction` variants (`MoveEntity`, `CameraFollow`, `CameraPan`, `Wait`), an `intro_events` field on `ProjectManifest`, runtime systems in the renderer for triggering and executing the intro sequence, skip controls, and editor support. The implementation spans three crates: `rpg-toolkit-common`, `rpg-toolkit-renderer`, and `rpg-toolkit-editor`.

## Tasks

- [x] 1. Add EntityTarget enum and new EventAction variants
  - [x] 1.1 Add `EntityTarget` enum and `MoveEntity`, `CameraFollow`, `CameraPan`, `Wait` variants to `EventAction` in `rpg-toolkit-common/src/map.rs`
    - Define `EntityTarget` enum with `Player` and `Npc { npc_id: String }` variants using `#[serde(tag = "type")]`
    - Add `default_entity_move_speed()` helper returning `2.0`
    - Add custom deserialization validators: `deserialize_entity_move_speed` (0.1–10.0), `deserialize_camera_pan_duration` (0.1–10.0), `deserialize_wait_duration` (0.1–30.0), and `deserialize_npc_id` (non-empty)
    - Add `MoveEntity { target: EntityTarget, target_x: u32, target_y: u32, speed: f32 }` variant with `#[serde(default = "default_entity_move_speed")]` on speed and custom deserializer for speed range
    - Add `CameraFollow { target: EntityTarget }` variant
    - Add `CameraPan { target_x: u32, target_y: u32, duration: f32 }` variant with custom deserializer for duration range
    - Add `Wait { duration: f32 }` variant with custom deserializer for duration range
    - Export `EntityTarget` from `lib.rs`
    - _Requirements: 2.1, 2.2, 2.5, 2.6, 2.7, 3.1, 3.6, 4.1, 4.3, 5.1, 5.3_

  - [x] 1.2 Add `intro_events` field to `ProjectManifest` in `rpg-toolkit-common/src/manifest.rs`
    - Add `#[serde(default)] pub intro_events: Option<Vec<EventAction>>` field to `ProjectManifest`
    - Add custom deserialization to reject lists exceeding 100 actions
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5_

  - [ ]* 1.3 Write property test: EventAction serialization round-trip (Property 1)
    - **Property 1: EventAction serialization round-trip (extended)**
    - Create `crates/rpg-toolkit-common/tests/properties/intro_action_round_trip.rs`
    - Generate random valid `MoveEntity`, `CameraFollow`, `CameraPan`, `Wait` with fields within valid ranges
    - Assert `serde_json::from_str(serde_json::to_string(&action)) == action`
    - **Validates: Requirements 9.1, 9.6**

  - [x]* 1.4 Write property test: EntityTarget serialization round-trip (Property 5)
    - **Property 5: EntityTarget serialization round-trip**
    - Create `crates/rpg-toolkit-common/tests/properties/entity_target_round_trip.rs`
    - Generate random `EntityTarget::Player` and `EntityTarget::Npc` with valid non-empty npc_id
    - Assert round-trip equality through JSON serialization
    - **Validates: Requirements 2.2, 9.1**

  - [ ]* 1.5 Write property test: New action variant validation rejects invalid fields (Property 2)
    - **Property 2: New action variant validation rejects invalid fields**
    - Create `crates/rpg-toolkit-common/tests/properties/intro_action_validation.rs`
    - Generate variants with intentionally invalid fields (empty npc_id, speed outside 0.1–10.0, duration outside valid ranges)
    - Assert deserialization returns an error
    - **Validates: Requirements 2.6, 2.7, 3.6, 4.3, 5.3**

  - [x]* 1.6 Write unit tests for new EventAction variants and `intro_events` field
    - Add tests in `rpg-toolkit-common/src/map.rs` `#[cfg(test)]` module for:
      - `MoveEntity` with Player/Npc target serializes/deserializes correctly
      - `MoveEntity` with speed omitted defaults to 2.0
      - `CameraFollow` with Player/Npc target round-trips
      - `CameraPan` with valid fields round-trips
      - `Wait` with valid fields round-trips
      - `EntityTarget::Player` serializes as `{ "type": "Player" }`
      - `EntityTarget::Npc` serializes as `{ "type": "Npc", "npc_id": "..." }`
      - `intro_events` absent in manifest JSON deserializes as `None`
      - `intro_events` with mixed action types round-trips correctly
      - Rejects `MoveEntity` with speed=0.0 and speed=11.0
      - Rejects `CameraPan` with duration=0.0 and duration=11.0
      - Rejects `Wait` with duration=0.0 and duration=31.0
      - Rejects `Npc` target with empty `npc_id`
    - _Requirements: 9.1, 9.2, 9.3, 9.4, 9.5, 9.6, 9.7, 1.2, 2.5, 2.6, 2.7, 3.6, 4.3, 5.3_

- [x] 2. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [x] 3. Implement runtime resources and `is_blocking_action` update
  - [x] 3.1 Add new runtime resources and extend `WaitingFor` in `rpg-toolkit-renderer/src/resources.rs`
    - Add `EntityMove`, `CameraPan`, and `Wait` variants to the `WaitingFor` enum
    - Add `EntityMoveState` resource with target, target_x, target_y, speed, current_x, current_y, complete fields
    - Add `CameraFollowTarget` resource with target field
    - Add `CameraPanState` resource with start_x, start_y, target_x, target_y, duration, elapsed fields
    - Add `WaitState` resource with duration, elapsed fields
    - Add `IntroEventsActive` marker resource
    - Add `NewGameFlag` marker resource
    - _Requirements: 6.1, 6.2, 6.4_

  - [x] 3.2 Update `is_blocking_action` in `rpg-toolkit-renderer/src/effects.rs`
    - Add match arms: `MoveEntity` → `true`, `CameraFollow` → `false`, `CameraPan` → `true`, `Wait` → `true`
    - _Requirements: 2.3, 3.2, 4.2, 5.2_

  - [x]* 3.3 Write property test: is_blocking_action classifies new actions correctly (Property 4)
    - **Property 4: is_blocking_action classifies new actions correctly**
    - Create `crates/rpg-toolkit-common/tests/properties/intro_blocking_classification.rs` (or in renderer test module)
    - Generate random `MoveEntity`, `CameraFollow`, `CameraPan`, `Wait` instances
    - Assert `MoveEntity`/`CameraPan`/`Wait` → `true`, `CameraFollow` → `false`
    - **Validates: Requirements 2.3, 3.2, 4.2, 5.2**

- [x] 4. Implement action handler systems in the renderer
  - [x] 4.1 Implement `CameraFollow` action handler system
    - Create handler in `rpg-toolkit-renderer/src/systems/` (new file or extend triggers.rs)
    - When `CameraFollow` is at the front of the queue: insert/update `CameraFollowTarget` resource, remove `CameraPanState` if active, pop action from queue (non-blocking)
    - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5_

  - [x] 4.2 Implement `CameraPan` action handler system
    - When `CameraPan` is at front of queue and `WaitingFor` is `Nothing`: record start position, insert `CameraPanState`, set `WaitingFor::CameraPan`
    - Each frame while `WaitingFor::CameraPan`: increment elapsed, interpolate camera position (linear lerp bounded by start/target bounding box), when elapsed >= duration mark complete, remove `CameraPanState`, set `WaitingFor::Nothing`
    - Clamp target to map bounds at runtime, log warn if out of bounds
    - _Requirements: 4.1, 4.2, 4.4, 4.5, 4.6_

  - [x] 4.3 Implement `MoveEntity` action handler system
    - When `MoveEntity` is at front of queue and `WaitingFor` is `Nothing`: insert `EntityMoveState`, set `WaitingFor::EntityMove`
    - Each frame while `WaitingFor::EntityMove`: move entity tile-by-tile toward target at configured speed, update walk animation, when target reached or unreachable mark complete, remove `EntityMoveState`, set `WaitingFor::Nothing`
    - If target NPC doesn't exist: log warn, skip action, advance queue
    - If target position unreachable: walk as close as possible, then complete
    - _Requirements: 2.1, 2.3, 2.4, 2.8, 2.9_

  - [x] 4.4 Implement `Wait` action handler system
    - When `Wait` is at front of queue and `WaitingFor` is `Nothing`: insert `WaitState`, set `WaitingFor::Wait`
    - Each frame while `WaitingFor::Wait`: increment elapsed, when elapsed >= duration remove `WaitState`, set `WaitingFor::Nothing`
    - _Requirements: 5.1, 5.2_

  - [x] 4.5 Update camera positioning system to respect `CameraFollowTarget` and `CameraPanState`
    - Modify `rpg-toolkit-renderer/src/systems/camera.rs` to check priority: CameraPanState (interpolating) → CameraFollowTarget (track entity) → default (track player)
    - When `CameraFollowTarget` is `Player`, follow player; when `Npc { npc_id }`, find and follow NPC position
    - If `CameraFollow` references nonexistent NPC at runtime: log warn, skip
    - _Requirements: 3.3, 3.4, 3.5, 3.7, 4.6_

  - [x] 4.6 Write property test: Camera pan interpolation is bounded (Property 3)
    - **Property 3: Camera pan interpolation is bounded**
    - Generate random start/target positions and elapsed times in [0, duration]
    - Assert interpolated position is always within axis-aligned bounding box of start and target
    - **Validates: Requirements 4.4**

- [x] 5. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [x] 6. Implement intro event trigger and skip systems
  - [x] 6.1 Modify title screen to insert `NewGameFlag` on new game
    - In the title screen system (likely `rpg-toolkit-renderer/src/systems/` or a scenes crate), when new game is selected: `commands.insert_resource(NewGameFlag)` before transitioning to `AppPhase::InGame`
    - _Requirements: 6.1_

  - [x] 6.2 Implement `trigger_intro_events` system
    - Add system to renderer that runs in `InGame` phase
    - Conditions: `NewGameFlag` present, no `ActionQueue` exists, `intro_events` is `Some` with non-empty vec
    - Actions: insert `ActionQueue` with intro_events, insert `IntroEventsActive`, remove `NewGameFlag`
    - If `intro_events` is `None` or empty, just remove `NewGameFlag` without inserting queue
    - _Requirements: 6.1, 6.2, 6.3, 6.6, 6.7, 6.8_

  - [x] 6.3 Suppress player movement while `IntroEventsActive` is present
    - Modify player input system to check for `IntroEventsActive` resource; if present, ignore movement input
    - When ActionQueue drains (all actions complete), remove `IntroEventsActive` marker
    - _Requirements: 6.4, 6.5_

  - [x] 6.4 Implement intro skip handler (Escape key)
    - When Escape is pressed and `IntroEventsActive` is present: drain `ActionQueue`, remove `IntroEventsActive`, remove `CameraPanState`/`EntityMoveState`/`WaitState` if active, reset `CameraFollowTarget` to `Player`
    - Restore normal player control
    - _Requirements: 7.1, 7.2, 7.3_

- [~] 7. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [x] 8. Add editor support for new action types
  - [~] 8.1 Add `MoveEntity`, `CameraFollow`, `CameraPan`, `Wait` variants to `ActionType` enum in `rpg-toolkit-editor/src/plugins/attribute/action_editor.rs`
    - Add the four new variants to the `ActionType` enum
    - Update any match arms or display logic that enumerate `ActionType`
    - _Requirements: 8.3_

  - [~] 8.2 Implement form fields for new action types in `action_editor_forms.rs`
    - `MoveEntity`: Entity target selector (Player radio / NPC ID dropdown), target_x/target_y u32 inputs, speed slider (0.1–10.0, default 2.0)
    - `CameraFollow`: Entity target selector (Player radio / NPC ID dropdown)
    - `CameraPan`: target_x/target_y u32 inputs, duration slider (0.1–10.0)
    - `Wait`: duration slider (0.1–30.0)
    - _Requirements: 8.4_

  - [~] 8.3 Add "Game Start Events" section in project settings panel
    - Add a collapsible section in the project settings editor panel
    - Reuse existing `ActionEditor` component to manage `intro_events` list
    - Support add, edit, remove, and reorder of EventAction items
    - Save changes to `manifest.intro_events` on project save
    - _Requirements: 8.1, 8.2, 8.5_

- [~] 9. Final checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

## Notes

- Tasks marked with `*` are optional and can be skipped for faster MVP
- Each task references specific requirements for traceability
- Checkpoints ensure incremental validation
- Property tests validate universal correctness properties from the design document
- Unit tests validate specific examples and edge cases
- The implementation uses Rust with serde for serialization and Bevy ECS for runtime systems
- Property tests use `proptest` (already a workspace dev-dependency) in `crates/rpg-toolkit-common/tests/properties/`

## Task Dependency Graph

```json
{
  "waves": [
    { "id": 0, "tasks": ["1.1"] },
    { "id": 1, "tasks": ["1.2", "1.3", "1.4", "1.5", "1.6"] },
    { "id": 2, "tasks": ["3.1", "3.2"] },
    { "id": 3, "tasks": ["3.3", "4.1", "4.2", "4.3", "4.4"] },
    { "id": 4, "tasks": ["4.5", "4.6"] },
    { "id": 5, "tasks": ["6.1", "6.2"] },
    { "id": 6, "tasks": ["6.3", "6.4"] },
    { "id": 7, "tasks": ["8.1"] },
    { "id": 8, "tasks": ["8.2", "8.3"] }
  ]
}
```
