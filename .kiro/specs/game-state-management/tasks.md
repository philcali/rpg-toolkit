# Implementation Plan: Game State Management

## Overview

This plan implements application phase management, save file location persistence, and a title screen for the rpg-toolkit. The work progresses from shared types in `rpg-toolkit-common`, through renderer modifications, a new `rpg-toolkit-scenes` crate, editor support, and finally launcher composition. Property-based tests validate serialization round-trips using the existing `proptest` infrastructure.

## Tasks

- [x] 1. Add AppPhase enum and extend shared types in rpg-toolkit-common
  - [x] 1.1 Add bevy dependency to rpg-toolkit-common and define AppPhase enum
    - Add `bevy` as a workspace dependency in `crates/rpg-toolkit-common/Cargo.toml`
    - Create `src/app_phase.rs` with the `AppPhase` enum implementing `States`, `Clone`, `Debug`, `Default`, `PartialEq`, `Eq`, `Hash`, `Serialize`, `Deserialize`
    - Default variant is `TitleScreen`; other variants: `InGame`, `Battle`, `Shop`, `Status`
    - Re-export `AppPhase` from `src/lib.rs`
    - _Requirements: 1.1, 1.2, 1.3, 1.4_

  - [x] 1.2 Move SaveFile and CharacterProgressData to rpg-toolkit-common with location fields
    - Move `SaveFile` struct and `CharacterProgressData` struct from `rpg-toolkit-renderer/src/save.rs` to a new `src/save.rs` in `rpg-toolkit-common`
    - Add `map_id: Option<String>`, `position: Option<(u32, u32)>`, `elevation: Option<u32>` fields with `#[serde(default)]`
    - Include `SaveFile::load` and `SaveFile::save` methods (filesystem operations)
    - Re-export `SaveFile` and `CharacterProgressData` from `rpg-toolkit-common`'s `lib.rs`
    - Update `rpg-toolkit-renderer/src/save.rs` to re-export from common for backward compatibility
    - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 2.6_

  - [x] 1.3 Add SaveGame and ChangePhase variants to EventAction
    - Add `SaveGame` unit variant to `EventAction` in `rpg-toolkit-common/src/map.rs`
    - Add `ChangePhase { phase: AppPhase }` variant to `EventAction`
    - Ensure both serialize with the existing `#[serde(tag = "type")]` convention
    - _Requirements: 3.1, 4.1, 4.3_

  - [x] 1.4 Write property test for SaveFile round-trip with location fields
    - Create `tests/properties/save_file_location_round_trip.rs` in rpg-toolkit-common
    - Generate arbitrary `SaveFile` instances with all combinations of `Some`/`None` location fields
    - Verify JSON serialize → deserialize produces equal `SaveFile`
    - Register the test in `Cargo.toml` `[[test]]` section
    - **Property 1: SaveFile serialization round-trip with location**
    - **Validates: Requirements 2.5, 2.6, 3.4, 10.2, 10.3, 10.4**

  - [ ]* 1.5 Write property test for EventAction new variants round-trip
    - Create `tests/properties/event_action_new_variants.rs` in rpg-toolkit-common
    - Generate `SaveGame` and `ChangePhase { phase }` for all `AppPhase` variants
    - Verify JSON serialize → deserialize produces equal `EventAction`
    - Register the test in `Cargo.toml` `[[test]]` section
    - **Property 2: New EventAction variants serialization round-trip**
    - **Validates: Requirements 1.3, 4.3, 9.6**

- [x] 2. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [x] 3. Extend save_game function and gate renderer on AppPhase
  - [x] 3.1 Extend save_game() with location parameters
    - Update `save_game()` in `rpg-toolkit-renderer/src/save.rs` to accept `map_id: Option<&str>`, `position: Option<(u32, u32)>`, `elevation: Option<u32>`
    - Populate the new `SaveFile` location fields from these parameters
    - Update existing call sites (if any) to pass `None` for backward compatibility
    - Update existing unit tests for the new signature
    - _Requirements: 10.1, 10.2, 10.3, 10.4_

  - [x] 3.2 Gate ProjectRendererPlugin Update systems on AppPhase::InGame
    - Add `.run_if(in_state(AppPhase::InGame))` to both Update system tuples in `ProjectRendererPlugin::build`
    - Move `fire_initial_map_changed` from `Startup` schedule to `OnEnter(AppPhase::InGame)`
    - Ensure Startup systems (load_spritesheet_assets, spawn_player, spawn_camera) remain ungated
    - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5_

  - [x] 3.3 Handle SaveGame and ChangePhase in advance_action_queue
    - Add match arms for `EventAction::SaveGame` and `EventAction::ChangePhase` in the `advance_action_queue` system
    - `SaveGame`: gather player location from `RendererState` and `PlayerCharacter`, call `save_game()` with location params, log warning on failure, pop and continue
    - `ChangePhase`: if target equals current phase, no-op and continue; otherwise call `next_state.set(phase)`, pop action, and return (stop processing this frame)
    - Handle missing `SavePath` resource gracefully with a warning
    - _Requirements: 3.2, 3.3, 3.4, 3.5, 3.6, 4.2, 4.4, 4.5_

- [ ] 4. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [x] 5. Create rpg-toolkit-scenes crate with TitleScreenPlugin
  - [x] 5.1 Scaffold rpg-toolkit-scenes crate
    - Create `crates/rpg-toolkit-scenes/Cargo.toml` with dependencies on `rpg-toolkit-common` (path) and `bevy` (workspace)
    - Create `crates/rpg-toolkit-scenes/src/lib.rs` that publicly re-exports `TitleScreenPlugin`
    - Add `"crates/rpg-toolkit-scenes"` to workspace members in root `Cargo.toml`
    - Ensure the crate does NOT depend on `rpg-toolkit-renderer` or `rpg-toolkit-editor`
    - _Requirements: 7.1, 7.2, 7.3, 7.4, 7.5_

  - [x] 5.2 Implement TitleScreenPlugin with New Game and Continue
    - Create `crates/rpg-toolkit-scenes/src/title_screen.rs` with `TitleScreenPlugin` struct implementing `Plugin`
    - Register systems on `OnEnter(AppPhase::TitleScreen)` (spawn UI), `OnExit(AppPhase::TitleScreen)` (despawn UI), and `Update` gated on `in_state(AppPhase::TitleScreen)` (input handling)
    - Render "New Game" and "Continue" options; disable "Continue" when no save file exists or save file is unparseable
    - "New Game": reset game state resources to defaults, set active map/position from project spawn point, transition to `InGame`
    - "Continue": load save file, populate game state resources, set active map/position from save's `map_id`/`position`, transition to `InGame`
    - Fall back to spawn point if save file lacks `map_id`/`position`; show error if no spawn point configured
    - Despawn all title screen entities on `OnExit`
    - _Requirements: 6.1, 6.2, 6.3, 6.4, 6.5, 6.6, 6.7, 6.8, 6.9_

- [ ] 6. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [x] 7. Refactor launcher and add editor support
  - [x] 7.1 Refactor launcher to compose plugins with AppPhase state
    - Add `rpg-toolkit-scenes` dependency to `rpg-toolkit-launcher/Cargo.toml`
    - Call `app.init_state::<AppPhase>()` to register the state (defaults to `TitleScreen`)
    - Insert game state resources with defaults (no save file loading at startup)
    - Add `TitleScreenPlugin` and `ProjectRendererPlugin` via `app.add_plugins(...)`
    - Keep `SavePath` resource insertion and project data loading at startup
    - Remove the current save-file-at-startup loading logic
    - _Requirements: 8.1, 8.2, 8.3, 8.4, 8.5, 8.6, 8.7_

  - [x] 7.2 Add SaveGame and ChangePhase support to editor action_editor.rs
    - Add `SaveGame` and `ChangePhase` variants to the `ActionType` enum in `action_editor.rs`
    - Add a `change_phase_target: AppPhase` field (defaulting to `InGame`) to `ActionEditorState`
    - Implement `load_from_action` for both new variants
    - Implement `build_action` for `SaveGame` (no fields needed) and `ChangePhase` (uses `change_phase_target`)
    - Update `action_editor_ui.rs` to render "Save Game" (no config) and "Change Phase" (dropdown of AppPhase variants)
    - _Requirements: 9.1, 9.2, 9.3, 9.4, 9.5, 9.6_

- [ ] 8. Final checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

## Notes

- Tasks marked with `*` are optional and can be skipped for faster MVP
- Each task references specific requirements for traceability
- Checkpoints ensure incremental validation
- Property tests validate universal correctness properties (round-trip serialization)
- `SaveFile` moves to `rpg-toolkit-common` but is re-exported from `rpg-toolkit-renderer` for backward compatibility
- The `save_game()` function stays in the renderer (it depends on Bevy resources)
- `rpg-toolkit-scenes` depends only on `rpg-toolkit-common` and `bevy`, not on the renderer

## Task Dependency Graph

```json
{
  "waves": [
    { "id": 0, "tasks": ["1.1"] },
    { "id": 1, "tasks": ["1.2", "1.3"] },
    { "id": 2, "tasks": ["1.4", "1.5"] },
    { "id": 3, "tasks": ["3.1", "3.2"] },
    { "id": 4, "tasks": ["3.3"] },
    { "id": 5, "tasks": ["5.1"] },
    { "id": 6, "tasks": ["5.2"] },
    { "id": 7, "tasks": ["7.1", "7.2"] }
  ]
}
```
