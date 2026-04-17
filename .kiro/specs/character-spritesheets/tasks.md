# Implementation Plan: Character Spritesheets & NPC Placement

## Overview

Incremental implementation starting from the common data model layer, extending serialization, then building editor UI and renderer features on top. Each task builds on the previous, with property-based tests placed close to the code they validate.

## Tasks

- [x] 1. Define new common types and extend existing data models
  - [x] 1.1 Add `SpritesheetId`, `FacingDirection`, `CharacterSpritesheet`, and `NpcInstance` types to `rpg-toolkit-common`
    - Create `crates/rpg-toolkit-common/src/spritesheet.rs` with `SpritesheetId` type alias, `FacingDirection` enum (Down=0, Left=1, Right=2, Up=3 with Default), `CharacterSpritesheet` struct (file_path, sprite_width, sprite_height, frame_count, direction_count), and `NpcInstance` struct (spritesheet_id, x, y, facing, serde(default) event_triggers, serde(default) patrol_path)
    - Add `validate_spritesheet_dimensions(width: u32, height: u32) -> Result<(), CommonError>` that accepts only 72×128
    - Export new types from `crates/rpg-toolkit-common/src/lib.rs`
    - _Requirements: 1.1, 1.2, 1.3, 5.1, 9.1, 9.2_

  - [x] 1.2 Extend `ProjectFile` and `MapData` with spritesheet and NPC fields
    - Add `#[serde(default)] pub spritesheets: HashMap<SpritesheetId, CharacterSpritesheet>` and `#[serde(default)] pub player_spritesheet: Option<SpritesheetId>` to `ProjectFile` in `crates/rpg-toolkit-common/src/project.rs`
    - Add `#[serde(default)] pub npcs: Vec<NpcInstance>` to `MapData` in `crates/rpg-toolkit-common/src/map.rs`
    - Update `ProjectFile::new()` to accept the new fields
    - Update `ProjectFile::deserialize()` to validate NPC spritesheet references exist in the registry
    - Add `compute_spritesheet_references()` function that returns which NPCs and player reference a given spritesheet
    - _Requirements: 1.4, 1.5, 1.6, 2.1, 5.1, 5.2, 5.3, 5.4_

  - [ ]* 1.3 Write property test: Spritesheet dimension validation (Property 1)
    - **Property 1: Spritesheet dimension validation**
    - Test with random `(u32, u32)` dimensions in range 1..256; assert Ok iff width==72 && height==128
    - Create `tests/properties/spritesheet_validation.rs`
    - **Validates: Requirements 1.2, 1.3**

  - [x] 1.4 Write property test: ProjectFile serialization round-trip (Property 2)
    - **Property 2: ProjectFile serialization round-trip**
    - Generate random `ProjectFile` with 0–3 maps, 0–3 spritesheets, 0–5 NPCs per map, optional player_spritesheet
    - Serialize to JSON then deserialize; assert equivalence
    - Create `tests/properties/project_round_trip.rs`
    - **Validates: Requirements 1.4, 1.5, 1.6, 5.4**

  - [ ]* 1.5 Write property test: Spritesheet reference tracking (Property 3)
    - **Property 3: Spritesheet reference tracking**
    - Generate random project state with 1–3 spritesheets and 0–5 NPCs with random spritesheet references
    - Assert `compute_spritesheet_references()` returns exactly the correct set
    - Create `tests/properties/spritesheet_references.rs`
    - **Validates: Requirements 2.1, 2.3**

  - [ ]* 1.6 Write property test: NPC spritesheet reference validation (Property 9)
    - **Property 9: NPC spritesheet reference validation**
    - Generate `ProjectFile` with some NPCs referencing non-existent spritesheet IDs
    - Assert deserialization/validation returns error for invalid references
    - Create `tests/properties/npc_reference_validation.rs`
    - **Validates: Requirements 5.3**

  - [ ]* 1.7 Write property test: Forward-compatible NPC deserialization (Property 14)
    - **Property 14: Forward-compatible NPC deserialization**
    - Generate random `NpcInstance` JSON omitting event_triggers and patrol_path
    - Assert deserialization succeeds with empty defaults; re-serialize and round-trip is stable
    - Create `tests/properties/npc_forward_compat.rs`
    - **Validates: Requirements 9.1, 9.2**

- [x] 2. Checkpoint — Ensure all common crate changes compile and tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [x] 3. Add sprite atlas index and animation logic to common/renderer
  - [x] 3.1 Implement `sprite_atlas_index` and animation frame calculation functions
    - Add `sprite_atlas_index(facing: FacingDirection, frame: usize) -> usize` to `crates/rpg-toolkit-common/src/spritesheet.rs` returning `facing as usize * 3 + frame`
    - Add `walk_animation_frame(elapsed: f32, frame_duration: f32) -> usize` returning `(elapsed / frame_duration).floor() as usize % 3`
    - _Requirements: 3.5, 4.1, 7.2_

  - [ ]* 3.2 Write property test: Sprite atlas index correctness (Property 5)
    - **Property 5: Sprite atlas index correctness**
    - Test all `FacingDirection` variants × frame indices 0–2; assert result == facing_row * 3 + frame and in [0, 12)
    - Create `tests/properties/sprite_atlas_index.rs`
    - **Validates: Requirements 3.5, 7.2**

  - [x] 3.3 Write property test: Walk animation frame cycling (Property 6)
    - **Property 6: Walk animation frame cycling**
    - Generate random FacingDirection, elapsed (0.0..10.0), frame_duration (0.01..1.0)
    - Assert frame == floor(elapsed / frame_duration) % 3
    - Create `tests/properties/walk_animation.rs`
    - **Validates: Requirements 4.1**

  - [ ]* 3.4 Write property test: Idle pose is middle frame (Property 7)
    - **Property 7: Idle pose is middle frame**
    - For any FacingDirection, assert idle frame index is 1
    - Create `tests/properties/idle_pose.rs`
    - **Validates: Requirements 4.2, 4.4**

  - [ ]* 3.5 Write property test: Facing direction matches movement (Property 8)
    - **Property 8: Facing direction matches movement direction**
    - For any movement Direction, assert FacingDirection is updated to match
    - Create `tests/properties/facing_direction.rs`
    - **Validates: Requirements 4.3**

- [x] 4. Extend the renderer for spritesheet-based player and NPC sprites
  - [x] 4.1 Add renderer components and resources for spritesheets
    - Add `NpcSprite { npc_index: usize }` component and `PlayerSpriteState { facing, animation_frame, animation_timer, is_moving }` component to `crates/rpg-toolkit-renderer/src/components.rs`
    - Add `AnimationConfig { frame_duration: f32 }` resource to `crates/rpg-toolkit-renderer/src/resources.rs`
    - Extend `RendererProjectData` with `spritesheet_textures: HashMap<SpritesheetId, Handle<Image>>` and `spritesheet_atlas_layouts: HashMap<SpritesheetId, Handle<TextureAtlasLayout>>`
    - _Requirements: 3.2, 7.1_

  - [x] 4.2 Implement spritesheet atlas loading in the renderer
    - Add `build_spritesheet_atlas()` function that creates `TextureAtlasLayout::from_grid(UVec2::new(24, 32), 3, 4, None, None)`
    - Load spritesheet textures and atlas layouts into `RendererProjectData` during startup
    - _Requirements: 3.2, 7.1_

  - [x] 4.3 Replace solid-color player with spritesheet sprite
    - Modify `spawn_player` in `crates/rpg-toolkit-renderer/src/systems/player.rs` to check `player_spritesheet`; if set and valid, spawn with texture atlas sprite and `PlayerSpriteState` component; otherwise fall back to solid-color rectangle
    - _Requirements: 3.2, 3.3, 3.5_

  - [ ]* 4.4 Write property test: Player rendering mode follows spritesheet presence (Property 4)
    - **Property 4: Player rendering mode follows spritesheet presence**
    - Test that player_spritesheet Some(valid_id) produces atlas sprite, None produces solid-color
    - Create `tests/properties/player_rendering_mode.rs`
    - **Validates: Requirements 3.2, 3.3**

  - [x] 4.5 Implement player walk animation and idle pose systems
    - Add `animate_player_sprite` system that cycles frames while `is_moving` using `walk_animation_frame()`, and resets to idle frame 1 when stationary
    - Update `player_movement` to set `PlayerSpriteState.facing` to match movement direction before starting animation, and set `is_moving` flag
    - Update `animate_player` to clear `is_moving` on animation completion
    - _Requirements: 4.1, 4.2, 4.3, 4.4_

  - [x] 4.6 Spawn NPC sprites on map load
    - Add `spawn_npc_sprites` system that runs after `sync_map_sprites`; for each `NpcInstance` in the active map, spawn a sprite entity with `NpcSprite` marker, texture atlas from the referenced spritesheet, idle frame for the NPC's facing direction, positioned via `grid_to_world`
    - Despawn existing `NpcSprite` entities on map change
    - _Requirements: 7.1, 7.2, 7.3, 7.5_

  - [ ]* 4.7 Write property test: NPC world position matches grid conversion (Property 13)
    - **Property 13: NPC world position matches grid conversion**
    - Generate random grid positions and tile dimensions; assert NPC world position == grid_to_world(x, y, tw, th)
    - Create `tests/properties/npc_world_position.rs`
    - **Validates: Requirements 7.3**

  - [x] 4.8 Extend collision system for NPC blocking
    - Modify `is_tile_blocked` in `crates/rpg-toolkit-renderer/src/systems/collision.rs` to also check `map.npcs.iter().any(|npc| npc.x == x && npc.y == y)`
    - _Requirements: 7.4, 8.1, 8.3_

  - [ ]* 4.9 Write property test: NPC collision blocks tile (Property 12)
    - **Property 12: NPC collision blocks tile**
    - Generate random map with random NPCs and opacity attributes; assert NPC tile is blocked; assert opacity-blocked tile stays blocked with/without NPC
    - Create `tests/properties/npc_collision.rs`
    - **Validates: Requirements 7.4, 8.1, 8.3**

- [x] 5. Checkpoint — Ensure renderer compiles and all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [x] 6. Implement editor spritesheet import and management UI
  - [x] 6.1 Add spritesheet import UI panel to the editor
    - Create spritesheet management panel (egui window) accessible from the app shell that lists loaded spritesheets with file path and dimensions
    - Add "Import Spritesheet" button that opens a file dialog, validates 72×128 dimensions using `validate_spritesheet_dimensions`, and adds to the project's spritesheet registry with a generated `SpritesheetId`
    - Display error message on invalid dimensions
    - _Requirements: 1.1, 1.2, 1.3, 2.2_

  - [x] 6.2 Add player spritesheet assignment UI
    - Add a dropdown in the spritesheet panel to select which spritesheet is assigned to the player (or None for solid-color fallback)
    - Store selection in `Project` as `player_spritesheet: Option<SpritesheetId>`
    - _Requirements: 3.1_

  - [x] 6.3 Add spritesheet removal with reference checking
    - Add "Remove" button per spritesheet that calls `compute_spritesheet_references()` and shows a warning dialog listing active NPC and player references before confirming deletion
    - _Requirements: 2.2, 2.3_

- [x] 7. Implement NPC placement tool in the editor
  - [x] 7.1 Add `NpcPlacement` variant to `AttributeTool` and wire into toolbar
    - Add `AttributeTool::NpcPlacement` variant to `EditorState` in `crates/rpg-toolkit-editor/src/data/editor_state.rs`
    - Add NPC placement icon to the attribute toolbar in `crates/rpg-toolkit-editor/src/plugins/toolbar.rs`
    - _Requirements: 6.1_

  - [x] 7.2 Implement NPC placement dialog and click handler
    - Create `NpcPlacementDialog` resource with fields for spritesheet selection, facing direction, and optional existing NPC data
    - In `attribute_click_system`, when `AttributeTool::NpcPlacement` is active and user clicks a tile: if tile has existing NPC, open dialog pre-populated for editing; otherwise open empty dialog for new placement
    - On dialog confirm: create `NpcInstance` and add to `map.npcs`; on remove: delete from `map.npcs`
    - _Requirements: 6.2, 6.3, 6.5, 6.6_

  - [x] 7.3 Add NPC overlay rendering in the editor
    - In `attribute_overlay_system`, when in NPC placement mode, draw gizmo overlays on tiles containing NPC instances (distinct color from opacity/event overlays)
    - _Requirements: 6.4_

  - [x] 7.4 Add undo/redo support for NPC operations
    - Add `EditCommandKind::PlaceNpc` and `EditCommandKind::RemoveNpc` variants to `EditCommandKind`
    - Implement `apply()` and `apply_inverse()` for both variants
    - Emit `EditCommand` from NPC placement/removal actions
    - _Requirements: 6.7_

  - [ ]* 7.5 Write property test: NPC placement creates correct instance (Property 10)
    - **Property 10: NPC placement creates correct instance**
    - Generate random valid position, spritesheet ID, and facing; assert NPC is added with correct values and count increases by 1
    - Create `tests/properties/npc_placement.rs`
    - **Validates: Requirements 6.3**

  - [ ]* 7.6 Write property test: NPC undo/redo round-trip (Property 11)
    - **Property 11: NPC undo/redo round-trip**
    - Generate random NPC placement/removal operations; assert undo restores original state, undo+redo equals apply
    - Create `tests/properties/npc_undo_redo.rs`
    - **Validates: Requirements 6.6, 6.7**

- [x] 8. Wire editor serialization for new spritesheet and NPC data
  - [x] 8.1 Update editor serialization plugin to include spritesheets and NPCs
    - Update `save_project_to_path` in `crates/rpg-toolkit-editor/src/plugins/serialization.rs` to include `spritesheets`, `player_spritesheet` from the Project resource when constructing `ProjectFile`
    - Update `load_project_with_dialog` to restore spritesheets from the loaded `ProjectFile` into the `Project` resource
    - NPCs are already part of `MapData` so they serialize automatically
    - _Requirements: 1.4, 1.5_

  - [x] 8.2 Extend editor `Project` resource with spritesheet fields
    - Add `spritesheets: HashMap<SpritesheetId, CharacterSpritesheet>` and `player_spritesheet: Option<SpritesheetId>` to the `Project` struct in `crates/rpg-toolkit-editor/src/data/project.rs`
    - _Requirements: 1.1, 3.1_

- [x] 9. Register new systems in the renderer plugin
  - [x] 9.1 Wire new renderer systems into `ProjectRendererPlugin`
    - Register `AnimationConfig` resource in `ProjectRendererPlugin::build()`
    - Add `animate_player_sprite` system after `animate_player`
    - Add `spawn_npc_sprites` system after `sync_map_sprites`
    - Update re-exports in `crates/rpg-toolkit-renderer/src/lib.rs`
    - _Requirements: 3.2, 4.1, 7.1_

- [x] 10. Final checkpoint — Ensure full build compiles and all tests pass
  - Ensure all tests pass, ask the user if questions arise.

## Notes

- Tasks marked with `*` are optional and can be skipped for faster MVP
- Each task references specific requirements for traceability
- Checkpoints ensure incremental validation
- Property tests validate universal correctness properties from the design document
- Unit tests validate specific examples and edge cases
- The `proptest` crate is already declared in `[workspace.dependencies]`
- All property tests go in `tests/properties/` as workspace-level integration tests
