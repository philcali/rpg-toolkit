# Implementation Plan: Tile Elevation System

## Overview

This plan implements a tile elevation (z-level) system across three crates: `rpg-toolkit-common` (data model), `rpg-toolkit-renderer` (runtime behavior), and `rpg-toolkit-editor` (authoring tools). Each task builds incrementally, starting with the shared data model, then runtime logic, then editor tooling, and finally wiring everything together.

## Tasks

- [x] 1. Add elevation fields to the common data model
  - [x] 1.1 Add `elevation: u32` and `target_elevation: Option<u32>` to `TileAttributes` in `crates/rpg-toolkit-common/src/map.rs`
    - Add `#[serde(default)]` on both fields for backward compatibility
    - Update `TileAttributes` Default impl (u32 defaults to 0, Option defaults to None)
    - _Requirements: 1.1, 1.2, 1.3, 3.1_
  - [x] 1.2 Add `target_elevation: Option<u32>` to `EventAction::JumpTo` in `crates/rpg-toolkit-common/src/map.rs`
    - Add `#[serde(default)]` for backward compatibility with existing map files
    - _Requirements: 10.1, 10.5_
  - [x] 1.3 Add `elevation: u32` to `NpcInstance` in `crates/rpg-toolkit-common/src/spritesheet.rs`
    - Add `#[serde(default)]` for backward compatibility
    - _Requirements: 9.1_
  - [x] 1.4 Add elevation validation to `MapData::validate` in `crates/rpg-toolkit-common/src/map.rs`
    - Validate that `target_elevation` values (when `Some`) are structurally sound (u32 inherently prevents negatives)
    - Add validation that attribute grid dimensions match for layers containing elevation data
    - _Requirements: 8.1, 8.2, 8.3_
  - [ ]* 1.5 Write property test for TileAttributes round-trip serialization
    - **Property: Round-trip consistency for TileAttributes with elevation fields**
    - **Validates: Requirements 1.4**
    - Create `tests/properties/elevation_round_trip.rs`
    - Generate arbitrary TileAttributes with elevation and target_elevation values
    - Verify serialize → deserialize produces equivalent values

- [ ] 2. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [x] 3. Implement elevation-aware collision in the renderer
  - [x] 3.1 Add `elevation: u32` field to `PlayerCharacter` in `crates/rpg-toolkit-renderer/src/components.rs`
    - Default to 0
    - _Requirements: 2.1, 2.2_
  - [x] 3.2 Update `is_tile_blocked` in `crates/rpg-toolkit-renderer/src/systems/collision.rs`
    - Add `player_elevation: u32` parameter to the function signature
    - Only apply opacity blocking when the tile's elevation matches the player's elevation
    - _Requirements: 4.1, 4.2, 4.3, 4.4_
  - [x] 3.3 Update `NpcPositions` in `crates/rpg-toolkit-renderer/src/resources.rs` to track elevation per NPC
    - Change `positions: Vec<(u32, u32)>` to `positions: Vec<(u32, u32, u32)>` where third element is elevation
    - Update `is_occupied` and `is_occupied_by_other` to accept a `player_elevation` parameter and only match NPCs at the same elevation
    - _Requirements: 9.2, 9.3_
  - [x] 3.4 Update `player_movement` in `crates/rpg-toolkit-renderer/src/systems/player.rs` to pass player elevation to collision checks
    - Read `player.elevation` and pass to `is_tile_blocked`
    - Pass player elevation to NPC position checks
    - _Requirements: 2.3, 4.1, 9.2_
  - [x] 3.5 Update `spawn_player` to initialize `elevation: 0` on the `PlayerCharacter` component
    - _Requirements: 2.2_
  - [ ]* 3.6 Write property test for elevation-aware collision
    - **Property: Opacity only blocks at matching elevation**
    - **Validates: Requirements 4.2, 4.3**
    - Generate maps with tiles at various elevations and opacity settings
    - Verify blocking only occurs when player elevation matches tile elevation

- [x] 4. Implement elevation-aware draw order in the renderer
  - [x] 4.1 Update `sync_map_sprites` in `crates/rpg-toolkit-renderer/src/systems/map_render.rs`
    - Compute Z values based on tile elevation relative to player elevation
    - Tiles with `elevation > player_elevation` get Z above the player sprite
    - Tiles with `elevation <= player_elevation` get Z below the player sprite
    - _Requirements: 5.1, 5.2_
  - [x] 4.2 Add a system to re-sort tile Z values when player elevation changes
    - Query all `RendererTileSprite` entities and update their `Transform.translation.z` based on the new player elevation
    - Ensure this runs within the same frame as the elevation change
    - _Requirements: 5.3_
  - [x] 4.3 Apply elevation-aware draw order to NPC sprites in `spawn_npc_sprites`
    - Use NPC elevation to determine Z ordering relative to the player
    - _Requirements: 9.4_
  - [x] 4.4 Update `init_npc_positions` to populate elevation from `NpcInstance.elevation`
    - _Requirements: 9.1_

- [x] 5. Implement elevation transitions and JumpTo handling in the renderer
  - [x] 5.1 Update `check_triggers` / add elevation transition logic in `crates/rpg-toolkit-renderer/src/systems/triggers.rs`
    - After player movement animation completes onto a tile with `target_elevation`, update `PlayerCharacter.elevation`
    - Apply the transition after the movement animation completes (in `animate_player` or a new system that runs after `PlayerMoved`)
    - _Requirements: 3.2, 3.3, 3.4_
  - [x] 5.2 Add `pending_target_elevation: Option<u32>` to `RendererState` in `crates/rpg-toolkit-renderer/src/resources.rs`
    - _Requirements: 10.2_
  - [x] 5.3 Update `advance_action_queue` JumpTo handling to store `target_elevation`
    - When `EventAction::JumpTo` has `target_elevation: Some(e)`, store it in `renderer_state.pending_target_elevation`
    - _Requirements: 10.2, 10.3_
  - [x] 5.4 Update `handle_map_change` to apply pending target elevation to the player
    - After repositioning the player, set `player.elevation = pending_target_elevation` if present, otherwise preserve current elevation
    - _Requirements: 10.2, 10.3_

- [ ] 6. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [x] 7. Add elevation editor tools - data layer
  - [x] 7.1 Add `Elevation` and `ElevationTransition` variants to `AttributeTool` enum in `crates/rpg-toolkit-editor/src/data/state.rs`
    - _Requirements: 6.1, 7.1_
  - [x] 7.2 Add `SetElevation` and `SetTargetElevation` variants to `EditCommandKind` in `crates/rpg-toolkit-editor/src/data/commands.rs`
    - `SetElevation { layer_index, x, y, old_value: u32, new_value: u32 }`
    - `SetTargetElevation { layer_index, x, y, old_value: Option<u32>, new_value: Option<u32> }`
    - Implement `apply` and `apply_inverse` for both variants
    - _Requirements: 6.4, 7.3_

- [x] 8. Add elevation editor tools - UI and interaction
  - [x] 8.1 Add elevation tool buttons to the toolbar in `crates/rpg-toolkit-editor/src/plugins/toolbar.rs`
    - Add entries for `AttributeTool::Elevation` and `AttributeTool::ElevationTransition` in the attribute tools list
    - _Requirements: 6.1, 7.1_
  - [x] 8.2 Implement click handling for `AttributeTool::Elevation` in `crates/rpg-toolkit-editor/src/plugins/attribute/click.rs`
    - On click, read current elevation value from TileAttributes at the clicked position
    - Open a small input dialog or inline UI to set the elevation value
    - Emit `EditCommand` with `SetElevation` kind
    - _Requirements: 6.2, 6.3, 6.4_
  - [x] 8.3 Implement click handling for `AttributeTool::ElevationTransition` in `crates/rpg-toolkit-editor/src/plugins/attribute/click.rs`
    - On click, read current target_elevation from TileAttributes at the clicked position
    - Open a small input dialog or inline UI to set the target elevation value
    - Emit `EditCommand` with `SetTargetElevation` kind
    - _Requirements: 7.1, 7.2, 7.3_
  - [x] 8.4 Add elevation overlay rendering in `crates/rpg-toolkit-editor/src/plugins/attribute/overlay.rs`
    - When `Elevation` tool is active: draw elevation level numbers/colors on tiles with non-zero elevation
    - When `ElevationTransition` tool is active: draw distinct markers on tiles with `target_elevation` set
    - _Requirements: 6.5, 7.4_

- [x] 9. Add coordinate tooltip and JumpTo form update
  - [x] 9.1 Add coordinate tooltip system in `crates/rpg-toolkit-editor/src/plugins/canvas.rs`
    - Display `(x, y)` at the cursor position when hovering over the map canvas
    - Show regardless of which editing tool is active
    - Hide when mouse leaves the map canvas area
    - Use egui tooltip or gizmo text rendering
    - _Requirements: 11.1, 11.2, 11.3, 11.4_
  - [x] 9.2 Add `target_elevation` field to JumpTo form in `crates/rpg-toolkit-editor/src/plugins/attribute/action_editor_forms.rs`
    - Add a "Target Elevation" input field in `render_jumpto_form`
    - Add `target_elevation` string field to `ActionEditorState` in `action_editor.rs`
    - Update `build_action` to include `target_elevation` in the JumpTo variant
    - Update `load_from_action` to populate the field when editing existing JumpTo actions
    - _Requirements: 10.4_

- [x] 10. Wire everything together and fix compilation
  - [x] 10.1 Update all call sites of `is_tile_blocked` across the codebase
    - Update calls in `player_movement` (player.rs)
    - Update any calls in NPC movement systems (npc.rs)
    - Ensure the new `player_elevation` parameter is passed correctly everywhere
    - _Requirements: 4.1, 9.2_
  - [x] 10.2 Update `EventAction::JumpTo` pattern matches throughout the codebase
    - Update `advance_action_queue` in triggers.rs to destructure `target_elevation`
    - Update `ActionEditorState::load_from_action` and `build_action` for the new field
    - Update any other match arms on `EventAction::JumpTo`
    - _Requirements: 10.1, 10.2, 10.4_
  - [x] 10.3 Register new editor systems and resources
    - Ensure elevation dialog/input resources are initialized in the attribute plugin
    - Wire up any new systems in the plugin's `build` method
    - _Requirements: 6.1, 7.1_

- [x] 11. Final checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

## Notes

- Tasks marked with `*` are optional and can be skipped for faster MVP
- Each task references specific requirements for traceability
- Checkpoints ensure incremental validation
- The design uses Rust throughout, matching the existing workspace
- All new fields use `#[serde(default)]` for backward compatibility with existing map files
- The `u32` type inherently prevents negative elevation values, simplifying validation
