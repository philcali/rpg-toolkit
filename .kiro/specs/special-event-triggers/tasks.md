# Implementation Plan: Special Event Triggers

## Overview

This plan implements five new `EventAction` variants (`ScreenShake`, `StopScreenShake`, `FadeTransition`, `SetState`, `SetPlayerAppearance`) across the common data models, renderer runtime systems, and editor UI. Tasks are ordered: data models first, then pure computation functions with property tests, then runtime systems, then editor UI, and finally integration wiring.

## Tasks

- [x] 1. Extend EventAction data model with new variants and supporting types
  - [x] 1.1 Add `ScreenShakeMode`, `FadeType`, and `PlayerAppearance` enums to `crates/rpg-toolkit-common/src/map.rs`
    - Add `ScreenShakeMode` enum with `Timed` (default) and `Continuous` variants, with `Serialize`/`Deserialize`/`Clone`/`Copy`/`Debug`/`PartialEq`/`Eq` derives
    - Add `FadeType` enum with `FadeIn` and `FadeOut` variants
    - Add `PlayerAppearance` enum with `Hidden`, `Spritesheet { path: String }`, and `Default` variants using `#[serde(tag = "type")]`
    - Add `default_fade_color` helper function returning `[0.0, 0.0, 0.0, 1.0]`
    - _Requirements: 1.1, 1.2, 5.1, 9.1, 9.2_

  - [x] 1.2 Add new variants to the `EventAction` enum in `crates/rpg-toolkit-common/src/map.rs`
    - Add `ScreenShake { intensity: f32, duration: f32, #[serde(default)] mode: ScreenShakeMode }` variant
    - Add `StopScreenShake` variant (no fields)
    - Add `FadeTransition { fade_type: FadeType, duration: f32, #[serde(default = "default_fade_color")] color: [f32; 4] }` variant
    - Add `SetState { key: String, value: String }` variant
    - Add `SetPlayerAppearance { appearance: PlayerAppearance }` variant
    - _Requirements: 1.1, 1.5, 1.8, 3.1, 3.2, 5.1, 5.5, 5.6, 7.1, 7.2, 7.3, 7.4, 9.1, 9.6, 9.7_

  - [ ]* 1.3 Write property test: EventAction serialization round-trip
    - **Property 1: EventAction Serialization Round-Trip**
    - **Validates: Requirements 1.8, 1.9, 3.2, 3.3, 5.6, 7.4, 9.6, 9.7**
    - Add `tests/properties/special_event_triggers.rs` with `[[test]]` entry in `tests/properties/Cargo.toml`
    - Implement `arb_screen_shake_mode`, `arb_fade_type`, `arb_player_appearance`, and `arb_event_action` proptest strategies covering all variants
    - Test: serialize any `EventAction` to JSON, deserialize back, assert equality

  - [ ]* 1.4 Write property test: ProjectFile round-trip with new variants
    - **Property 2: ProjectFile Round-Trip with New Variants**
    - **Validates: Requirements 13.1, 13.3**
    - Generate `ProjectFile` values containing maps with tile attributes that include new `EventAction` variants
    - Test: serialize to JSON, deserialize back, assert equality

- [x] 2. Implement pure computation functions
  - [x] 2.1 Add pure computation functions to a new module `crates/rpg-toolkit-renderer/src/effects.rs`
    - Implement `compute_shake_offset(intensity: f32, seed_x: f32, seed_y: f32) -> (f32, f32)` — returns offset bounded by intensity
    - Implement `is_shake_complete(elapsed: f32, duration: f32, mode: ScreenShakeMode) -> bool` — returns true for Timed when elapsed >= duration, always false for Continuous
    - Implement `compute_fade_opacity(elapsed: f32, duration: f32, fade_type: FadeType) -> f32` — linear interpolation clamped to [0.0, 1.0]
    - Implement `is_fade_complete(elapsed: f32, duration: f32) -> bool` — returns true when elapsed >= duration
    - Implement `is_blocking_action(action: &EventAction) -> bool` — classifies actions as blocking or non-blocking
    - Register the module in `crates/rpg-toolkit-renderer/src/lib.rs`
    - _Requirements: 2.3, 2.6, 2.7, 2.8, 6.2, 6.3, 6.4, 12.2, 12.3_

  - [ ]* 2.2 Write property test: Screen shake offset bounded by intensity
    - **Property 3: Screen Shake Offset Bounded by Intensity**
    - **Validates: Requirements 2.3**
    - For any intensity in [0.0, 50.0] and seeds in [0.0, 1.0], assert |dx| <= intensity and |dy| <= intensity

  - [ ]* 2.3 Write property test: Timed shake completion
    - **Property 4: Timed Shake Completion**
    - **Validates: Requirements 2.6, 2.7**
    - For any duration d in [0.0, 10.0] and elapsed e >= d, assert `is_shake_complete` returns true; for e < d where d > 0, assert false

  - [ ]* 2.4 Write property test: Continuous shake never self-completes
    - **Property 5: Continuous Shake Never Self-Completes**
    - **Validates: Requirements 1.4, 2.8**
    - For any duration and any elapsed time, assert `is_shake_complete(elapsed, duration, Continuous)` returns false

  - [ ]* 2.5 Write property test: Fade opacity interpolation
    - **Property 6: Fade Opacity Interpolation**
    - **Validates: Requirements 6.2, 6.3**
    - For any duration d > 0 and elapsed t in [0.0, d], assert `compute_fade_opacity` returns t/d for FadeOut and 1.0 - t/d for FadeIn (within f32 tolerance), and result is always in [0.0, 1.0]

  - [ ]* 2.6 Write property test: SetState overwrite semantics
    - **Property 7: SetState Overwrite Semantics**
    - **Validates: Requirements 8.1, 8.4**
    - For any sequence of (key, value) pairs applied to a HashMap, the final value for each key equals the last value written

  - [ ]* 2.7 Write property test: Action blocking classification
    - **Property 8: Action Blocking Classification**
    - **Validates: Requirements 2.4, 2.5, 4.2, 6.4, 8.2, 10.5, 12.2, 12.3**
    - For any EventAction, assert `is_blocking_action` returns true iff the action is ScreenShake(Timed, duration > 0), FadeTransition(duration > 0), or ShowDialog

- [ ] 3. Checkpoint
  - Ensure all tests pass, ask the user if questions arise.

- [x] 4. Add new resources and components to the renderer
  - [x] 4.1 Add `WaitingFor` enum and new resources to `crates/rpg-toolkit-renderer/src/resources.rs`
    - Add `WaitingFor` enum with variants `Nothing`, `Dialog`, `ScreenShake`, `Fade`
    - Replace `waiting_for_dialog: bool` in `ActionQueue` with `waiting_for: WaitingFor`
    - Add `ScreenShakeState` resource with fields: `intensity: f32`, `mode: ScreenShakeMode`, `duration: f32`, `elapsed: f32`
    - Add `FadeState` resource with fields: `fade_type: FadeType`, `duration: f32`, `elapsed: f32`, `color: [f32; 4]`
    - Add `GameState` resource with `flags: HashMap<String, String>`, derive `Default`
    - Add `PlayerAppearanceState` resource with `original_spritesheet_id: Option<SpritesheetId>`
    - _Requirements: 2.1, 2.2, 6.1, 8.3, 10.1_

  - [x] 4.2 Add `FadeOverlay` marker component to `crates/rpg-toolkit-renderer/src/components.rs`
    - Add `#[derive(Component)] pub struct FadeOverlay;`
    - _Requirements: 6.1_

  - [x] 4.3 Update all existing references to `waiting_for_dialog` in the codebase
    - Update `check_triggers` in `triggers.rs` to use `waiting_for: WaitingFor::Nothing` when creating `ActionQueue`
    - Update `advance_action_queue` to check `waiting_for` enum instead of the boolean
    - Update the `lib.rs` exports to include new resources and components
    - _Requirements: 12.1_

- [x] 5. Implement runtime systems for new effects
  - [x] 5.1 Implement `screen_shake_system` in `crates/rpg-toolkit-renderer/src/systems/triggers.rs` (or a new `effects.rs` systems file)
    - Each frame: increment `ScreenShakeState.elapsed` by `delta_secs()`
    - If `is_shake_complete()`: remove `ScreenShakeState`, set `waiting_for = Nothing`, reset camera offset
    - Otherwise: generate random seeds from elapsed time, call `compute_shake_offset()`, apply offset to `GameCamera` transform
    - _Requirements: 2.1, 2.2, 2.3, 2.6, 2.8_

  - [x] 5.2 Implement `fade_system` in `crates/rpg-toolkit-renderer/src/systems/triggers.rs` (or a new `effects.rs` systems file)
    - Each frame: increment `FadeState.elapsed` by `delta_secs()`
    - Compute opacity via `compute_fade_opacity()`
    - Update `FadeOverlay` entity's `BackgroundColor` alpha
    - If `is_fade_complete()`: for FadeOut, remove `FadeState` but leave overlay; for FadeIn, remove `FadeState` and despawn overlay
    - Set `waiting_for = Nothing` when complete
    - _Requirements: 6.1, 6.2, 6.3, 6.4, 6.5, 6.6, 6.7, 6.8_

  - [x] 5.3 Extend `advance_action_queue` to handle all new action types
    - Handle `ScreenShake(Timed)`: insert `ScreenShakeState`, set `waiting_for = WaitingFor::ScreenShake`
    - Handle `ScreenShake(Continuous)`: insert `ScreenShakeState`, pop action, continue (non-blocking)
    - Handle `StopScreenShake`: remove `ScreenShakeState` if present, reset camera, pop action, continue
    - Handle `FadeTransition`: insert `FadeState`, spawn `FadeOverlay` entity, set `waiting_for = WaitingFor::Fade`
    - Handle `SetState`: insert/update `GameState` resource entry, pop action, continue
    - Handle `SetPlayerAppearance`: apply visibility/spritesheet change, pop action, continue
    - Handle `WaitingFor::ScreenShake` and `WaitingFor::Fade` wait conditions (check resource removal)
    - Handle duration 0.0 edge cases for ScreenShake(Timed) and FadeTransition
    - _Requirements: 2.1, 2.4, 2.5, 2.7, 4.1, 4.2, 4.3, 6.1, 6.4, 6.8, 8.1, 8.2, 8.4, 10.1, 10.2, 10.3, 10.4, 10.5, 10.7, 12.1, 12.2, 12.3_

  - [x] 5.4 Update `handle_map_change` to clean up active effects on JumpTo
    - Remove `ScreenShakeState` if present when JumpTo clears the queue
    - Reset camera offset to zero
    - _Requirements: 12.4, 12.5_

  - [x] 5.5 Register new systems in `ProjectRendererPlugin::build` in `crates/rpg-toolkit-renderer/src/lib.rs`
    - Add `screen_shake_system` after `advance_action_queue` and before `update_camera`
    - Add `fade_system` after `advance_action_queue`
    - Init `GameState` resource as default
    - _Requirements: 8.3_

- [x] 6. Checkpoint
  - Ensure all tests pass, ask the user if questions arise.

- [x] 7. Add editor UI support for new event actions
  - [x] 7.1 Extend the Event Trigger Editor dialog to include new action types
    - Add `ScreenShake`, `StopScreenShake`, `FadeTransition`, `SetState`, and `SetPlayerAppearance` to the action type selector dropdown
    - _Requirements: 11.1_

  - [x] 7.2 Implement ScreenShake editor fields
    - Add mode selector (Timed/Continuous, default Timed)
    - Add intensity numeric input (default 5.0)
    - Add duration numeric input (default 0.5)
    - Disable/hide duration field when mode is Continuous
    - Validate intensity in [0.0, 50.0] and duration in [0.0, 10.0]
    - _Requirements: 11.2, 11.3, 11.10_

  - [x] 7.3 Implement FadeTransition editor fields
    - Add fade_type selector (FadeIn/FadeOut)
    - Add duration numeric input (default 1.0)
    - Add color picker (default black)
    - Validate duration in [0.0, 10.0]
    - _Requirements: 11.5, 11.10_

  - [x] 7.4 Implement SetState editor fields
    - Add key text input field
    - Add value text input field
    - Validate key is non-empty before save
    - _Requirements: 11.6, 11.11_

  - [x] 7.5 Implement SetPlayerAppearance editor fields
    - Add appearance selector (Hidden/Spritesheet/Default, default Hidden)
    - Show file path picker when Spritesheet is selected
    - Hide extra fields for Hidden and Default
    - Validate path is non-empty for Spritesheet before save
    - _Requirements: 11.7, 11.8, 11.9, 11.12_

  - [x] 7.6 Implement StopScreenShake editor entry (no additional fields)
    - Display no configuration fields when StopScreenShake is selected
    - _Requirements: 11.4_

- [ ] 8. Checkpoint
  - Ensure all tests pass, ask the user if questions arise.

- [ ]* 9. Write unit tests for edge cases and error handling
  - Test `ScreenShakeMode` defaults to `Timed` when not specified in JSON
  - Test fade color defaults to black `[0.0, 0.0, 0.0, 1.0]` when not specified
  - Test `StopScreenShake` is a no-op when no `ScreenShakeState` is present
  - Test `FadeTransition` with duration 0.0 applies final state instantly
  - Test `JumpTo` clears continuous shake
  - Test unknown action type produces a clear deserialization error
  - Test empty key is rejected by editor validation
  - Test empty path is rejected by editor validation for Spritesheet appearance
  - _Requirements: 1.5, 4.3, 5.5, 6.8, 12.5, 13.2, 11.11, 11.12_

- [ ] 10. Final checkpoint
  - Ensure all tests pass, ask the user if questions arise.

## Notes

- Tasks marked with `*` are optional and can be skipped for faster MVP
- Each task references specific requirements for traceability
- Checkpoints ensure incremental validation
- Property tests validate universal correctness properties from the design document
- Unit tests validate specific examples and edge cases
- The design uses Rust throughout, matching the existing codebase
- Property tests use `proptest` with `ProptestConfig::with_cases(100)` minimum, added to `tests/properties/`
- The `WaitingFor` enum replaces the existing `waiting_for_dialog` boolean — this is a refactor that must update all existing references
