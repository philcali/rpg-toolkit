# Implementation Plan: Game Renderer

## Overview

Restructure the RPG Toolkit into a Cargo workspace with four crates (common, renderer, editor, launcher) and implement the `ProjectRendererPlugin` — a Bevy plugin providing map rendering, player movement, collision, event triggers, and a following camera. Tasks are ordered for incremental development: workspace scaffolding → common crate extraction → renderer plugin systems → launcher binary → property-based tests.

## Tasks

- [x] 1. Set up Cargo workspace and crate scaffolding
  - [x] 1.1 Convert root `Cargo.toml` to a workspace manifest
    - Replace the existing `[package]` root `Cargo.toml` with a `[workspace]` manifest
    - Declare members: `crates/rpg-toolkit-common`, `crates/rpg-toolkit-renderer`, `crates/rpg-toolkit-editor`, `crates/rpg-toolkit-launcher`
    - Move shared `[workspace.dependencies]` (bevy, serde, serde_json, thiserror, uuid, proptest) to the workspace manifest
    - _Requirements: 2.1_

  - [x] 1.2 Create `rpg-toolkit-common` crate skeleton
    - Create `crates/rpg-toolkit-common/Cargo.toml` with dependencies: `serde`, `serde_json`, `thiserror`, `uuid`
    - Create `crates/rpg-toolkit-common/src/lib.rs` with module declarations for `error`, `map`, `tileset`, `project`
    - _Requirements: 1.2, 1.3, 2.1_

  - [x] 1.3 Create `rpg-toolkit-renderer` crate skeleton
    - Create `crates/rpg-toolkit-renderer/Cargo.toml` with dependencies: `bevy`, `rpg-toolkit-common`
    - Create `crates/rpg-toolkit-renderer/src/lib.rs` with empty `ProjectRendererPlugin` struct implementing `Plugin`
    - _Requirements: 2.2_

  - [x] 1.4 Create `rpg-toolkit-editor` crate skeleton
    - Create `crates/rpg-toolkit-editor/Cargo.toml` with dependencies: `bevy`, `bevy_egui`, `rpg-toolkit-common`, `rpg-toolkit-renderer`, `rfd`, `uuid`, `image`
    - Move existing `src/` contents into `crates/rpg-toolkit-editor/src/`
    - _Requirements: 2.3_

  - [x] 1.5 Create `rpg-toolkit-launcher` crate skeleton
    - Create `crates/rpg-toolkit-launcher/Cargo.toml` with dependencies: `bevy`, `rpg-toolkit-common`, `rpg-toolkit-renderer`
    - Create `crates/rpg-toolkit-launcher/src/main.rs` with a placeholder `main()` that prints usage
    - _Requirements: 2.4_

- [x] 2. Checkpoint — Workspace compiles
  - Ensure `cargo check --workspace` succeeds with all four crate skeletons, ask the user if questions arise.


- [x] 3. Extract shared types into `rpg-toolkit-common`
  - [x] 3.1 Implement `CommonError` in `crates/rpg-toolkit-common/src/error.rs`
    - Define `CommonError` enum with variants: `InvalidDimensions`, `InvalidTileSize`, `ProjectParseError(String)`, `ProjectValidationError(String)`
    - Derive `thiserror::Error` and `Debug`
    - _Requirements: 1.5_

  - [x] 3.2 Move map types to `crates/rpg-toolkit-common/src/map.rs`
    - Move `MapId`, `TilesetId`, `EventAction`, `TileAttributes`, `TileAttributeLayer`, `SpawnPoint`, `TileRef`, `Layer`, `MapData` from `src/data/map.rs`
    - Replace all `EditorError` references with `CommonError`
    - Preserve `MapData::new()` and `MapData::validate()` methods
    - Preserve all `Serialize`/`Deserialize` derives and `#[serde(tag = "type")]` on `EventAction`
    - Remove `use bevy::prelude::*` — common crate has no Bevy dependency
    - _Requirements: 1.1, 1.3, 1.6, 1.7_

  - [x] 3.3 Move tileset types to `crates/rpg-toolkit-common/src/tileset.rs`
    - Move `TilesetMeta` struct and `from_image_dimensions()` method
    - Replace `EditorError` with `CommonError` (use `CommonError::InvalidTileSize` for unsupported format)
    - Remove Bevy imports
    - _Requirements: 1.1, 1.6_

  - [x] 3.4 Move project types to `crates/rpg-toolkit-common/src/project.rs`
    - Move `ProjectFile` struct with `serialize()` and `deserialize()` methods
    - Replace `EditorError` with `CommonError`
    - Remove `use bevy::prelude::*` (the `warn!` macro for JumpTo warnings can use `eprintln!` or `log` crate instead)
    - Preserve all validation logic: map validation, tileset reference checking, JumpTo target warnings
    - _Requirements: 1.1, 1.6, 1.8_

  - [x] 3.5 Wire up `rpg-toolkit-common/src/lib.rs` re-exports
    - Add `pub use` statements for all public types: `CommonError`, `MapData`, `MapId`, `TilesetId`, `TileRef`, `Layer`, `TileAttributeLayer`, `TileAttributes`, `EventAction`, `SpawnPoint`, `TilesetMeta`, `ProjectFile`
    - _Requirements: 1.1_

- [x] 4. Refactor editor crate to use common imports
  - [x] 4.1 Update editor `data/map.rs` to re-export from common
    - Replace the moved type definitions with `use rpg_toolkit_common::{...}` imports
    - Keep editor-specific methods (e.g., `place_tile`, `erase_tile`, `add_layer`, `delete_layer`) that depend on `EditCommand`/`EditorError` as extension trait or wrapper
    - _Requirements: 1.4, 2.3_

  - [x] 4.2 Update editor `data/tileset.rs` to re-export from common
    - Import `TilesetMeta` from common; keep `TilesetEntry` (has Bevy `Handle` types) in editor
    - _Requirements: 1.4_

  - [x] 4.3 Update editor `data/editor_state.rs` to wrap `CommonError`
    - Keep `EditorError` in editor crate; add a variant or `From<CommonError>` impl to wrap common errors
    - Ensure `UnsupportedFormat` variant remains for editor-specific image format errors
    - _Requirements: 1.4, 1.5_

  - [x] 4.4 Update editor `data/project.rs` to use common types
    - Import `ProjectFile` from common; keep `Project` resource (Bevy `Resource` with `Handle` types) in editor
    - Update `Project` methods to use `CommonError` where appropriate
    - _Requirements: 1.4_

  - [x] 4.5 Update editor `data/mod.rs` and all plugin/system imports
    - Ensure all `use` paths compile after the type migration
    - _Requirements: 1.4_

- [x] 5. Checkpoint — Editor compiles against common crate
  - Ensure `cargo check -p rpg-toolkit-editor` succeeds with all types imported from common, ask the user if questions arise.


- [x] 6. Implement renderer resources, components, and events
  - [x] 6.1 Create renderer resources in `crates/rpg-toolkit-renderer/src/resources.rs`
    - Define `RendererProjectData` resource: `project_file: ProjectFile`, `tileset_textures: HashMap<TilesetId, Handle<Image>>`, `tileset_atlas_layouts: HashMap<TilesetId, Handle<TextureAtlasLayout>>`
    - Define `RendererState` resource with `active_map_id: Option<MapId>`, `pending_map_change: Option<MapId>`
    - Define `MovementConfig` with `move_duration: f32` (default 0.15)
    - Define `PlayerVisual` with `color: Color` (default `Color::srgb(0.2, 0.6, 1.0)`)
    - _Requirements: 9.1, 9.2_

  - [x] 6.2 Create renderer components in `crates/rpg-toolkit-renderer/src/components.rs`
    - Define `PlayerCharacter` component with `grid_x: u32`, `grid_y: u32`, `move_animation: Option<MoveAnimation>`
    - Define `MoveAnimation` struct with `from: Vec2`, `to: Vec2`, `elapsed: f32`, `duration: f32`
    - Define `RendererTileSprite` component with `layer_index: usize`, `x: u32`, `y: u32`
    - Define `GameCamera` marker component
    - _Requirements: 9.3_

  - [x] 6.3 Create renderer events in `crates/rpg-toolkit-renderer/src/events.rs`
    - Define `MapChanged` event with `previous_map_id: Option<MapId>`, `new_map_id: MapId`
    - Define `PlayerMoved` event with `from: (u32, u32)`, `to: (u32, u32)`
    - _Requirements: 9.4, 9.5_

- [x] 7. Implement renderer input system
  - [x] 7.1 Create input reader in `crates/rpg-toolkit-renderer/src/input.rs`
    - Define `Direction` enum: `Up`, `Down`, `Left`, `Right`
    - Define `MovementIntent` resource holding `Option<Direction>`
    - Implement `read_input` system: read `ButtonInput<KeyCode>`, map W/Up→Up, A/Left→Left, S/Down→Down, D/Right→Right, write to `MovementIntent`
    - _Requirements: 5.1_

- [x] 8. Implement player spawning and movement systems
  - [x] 8.1 Create player spawn system in `crates/rpg-toolkit-renderer/src/systems/player.rs`
    - Implement `spawn_player` startup system: read `RendererProjectData` spawn point, clamp coordinates to map bounds, spawn `PlayerCharacter` entity with `Sprite` (solid colored rectangle from `PlayerVisual`), position at world-space center of spawn tile using coordinate formula `(x*tw + tw/2, -(y*th + th/2))`
    - Set initial `RendererState.active_map_id` from spawn point's `map_id`
    - Set Z-ordering above all map layers (`num_layers + 1`)
    - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5_

  - [x] 8.2 Implement `player_movement` system
    - Read `MovementIntent`, compute target tile from current `PlayerCharacter.grid_x/grid_y` + direction
    - Check bounds: reject if target is outside map dimensions
    - Check collision: call `is_tile_blocked` (any layer with `opacity == true` at target)
    - If valid: set `PlayerCharacter.move_animation` with `from`/`to` world positions, update `grid_x`/`grid_y` to target
    - If `move_animation` is already `Some`, ignore input (animation exclusivity)
    - _Requirements: 5.1, 5.2, 5.4, 5.5, 6.1, 6.2, 6.3_

  - [x] 8.3 Implement `animate_player` system
    - Advance `MoveAnimation.elapsed` by `Time::delta_secs()`
    - Lerp `Transform.translation` from `from` to `to` based on `elapsed / duration`
    - When `elapsed >= duration`: snap to `to`, clear `move_animation`, fire `PlayerMoved` event
    - _Requirements: 5.3_

- [x] 9. Implement collision helper
  - [x] 9.1 Create `is_tile_blocked` function in `crates/rpg-toolkit-renderer/src/systems/collision.rs`
    - Implement `is_tile_blocked(map: &MapData, x: u32, y: u32) -> bool`: iterate all layers, check `attributes.cells[y][ ].opacity`
    - Return `true` if any layer has `opacity == true` at that position
    - _Requirements: 6.1, 6.2_

- [x] 10. Implement event trigger and map change systems
  - [x] 10.1 Create trigger system in `crates/rpg-toolkit-renderer/src/systems/triggers.rs`
    - Implement `check_triggers` system: on `PlayerMoved` event, collect `EventAction` entries from all layers at the destination tile
    - For the first `JumpTo` found: set `RendererState.pending_map_change` to `target_map_id`, store target coordinates
    - If `target_map_id` doesn't exist in `RendererProjectData.project_file.maps`, log warning and skip
    - _Requirements: 7.1, 7.2, 7.4_

  - [x] 10.2 Create map change system in `crates/rpg-toolkit-renderer/src/systems/triggers.rs`
    - Implement `handle_map_change` system: when `RendererState.pending_map_change` is `Some`, fire `MapChanged` event, update `active_map_id`, clamp target coordinates to new map bounds, reposition `PlayerCharacter`, clear `pending_map_change`
    - _Requirements: 7.2, 7.3, 7.5, 3.6_

- [x] 11. Implement map sprite rendering system
  - [x] 11.1 Create `sync_map_sprites` system in `crates/rpg-toolkit-renderer/src/systems/map_render.rs`
    - On `MapChanged` event (or initial load): despawn all entities with `RendererTileSprite` component
    - Spawn tile sprites for the new active map: iterate all visible layers, for each non-`None` tile, resolve `TileRef` to tileset texture + atlas index, spawn `Sprite` entity with `RendererTileSprite` component
    - Position each sprite at world coordinate: `(x * tw + tw/2, -(y * th + th/2))`
    - Set Z-ordering per layer index
    - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5, 3.6_

- [x] 12. Implement camera system
  - [x] 12.1 Create camera setup and follow system in `crates/rpg-toolkit-renderer/src/systems/camera.rs`
    - Spawn a `Camera2d` entity with `GameCamera` marker component at startup
    - Implement `update_camera` system: set camera `Transform.translation` to player world position, then clamp to map bounds so viewport doesn't show outside the map
    - Handle small maps (smaller than viewport): center camera on map
    - _Requirements: 8.1, 8.2, 8.3, 8.4_

- [x] 13. Wire up `ProjectRendererPlugin`
  - [x] 13.1 Complete `ProjectRendererPlugin::build()` in `crates/rpg-toolkit-renderer/src/lib.rs`
    - Register resources: `init_resource::<RendererState>()`, `init_resource::<MovementConfig>()`, `init_resource::<PlayerVisual>()`, `init_resource::<MovementIntent>()`
    - Register events: `add_event::<MapChanged>()`, `add_event::<PlayerMoved>()`
    - Add startup systems: `spawn_player`, `spawn_camera`, initial `sync_map_sprites`
    - Add `Update` systems with explicit ordering: `read_input` → `player_movement` → `animate_player` → `check_triggers` → `handle_map_change` → `sync_map_sprites` → `update_camera`
    - Re-export public API: `RendererProjectData`, `RendererState`, `PlayerCharacter`, `GameCamera`, `MapChanged`, `PlayerMoved`, `MovementConfig`, `PlayerVisual`
    - _Requirements: 9.1, 9.2, 9.3, 9.4, 9.5, 9.6_

- [x] 14. Checkpoint — Renderer crate compiles
  - Ensure `cargo check -p rpg-toolkit-renderer` succeeds with all systems wired, ask the user if questions arise.


- [x] 15. Implement launcher binary
  - [x] 15.1 Implement CLI argument parsing and project loading in `crates/rpg-toolkit-launcher/src/main.rs`
    - Parse first CLI argument as project file path; exit with usage message if missing
    - Read file contents; exit with error if file not found
    - Deserialize `ProjectFile` using `ProjectFile::deserialize()`; exit with error on failure
    - Validate spawn point exists; exit with error if missing
    - _Requirements: 10.1, 10.2, 10.5, 10.6, 10.7_

  - [x] 15.2 Implement tileset loading and Bevy app setup
    - Resolve tileset image paths relative to the project file directory
    - Build Bevy `App` with `DefaultPlugins`, insert `RendererProjectData` resource (with loaded project file — tileset texture loading deferred to Bevy asset system)
    - Add `ProjectRendererPlugin`
    - _Requirements: 10.3, 10.4_

- [x] 16. Checkpoint — Full workspace compiles and launcher runs
  - Ensure `cargo check --workspace` succeeds, ask the user if questions arise.

- [ ] 17. Property-based tests
  - [ ]* 17.1 Write property test for serialization round-trip
    - **Property 1: Serialization Round-Trip**
    - Generate arbitrary valid `ProjectFile` values (1–4 maps, dimensions 1–16, 1–3 layers, 1–3 tilesets, valid tile refs, random attributes, optional spawn point referencing valid maps)
    - Assert `ProjectFile::deserialize(pf.serialize().unwrap()).unwrap() == pf`
    - Test location: `tests/properties/serialization_roundtrip.rs`
    - **Validates: Requirement 11.1, 11.2**

  - [ ]* 17.2 Write property test for MapData validation consistency
    - **Property 2: MapData Validation Consistency**
    - Generate valid dimension ranges (1–256) and tile sizes from {8, 16, 32, 64}
    - Assert `MapData::new(name, w, h, tw, th).unwrap().validate().is_ok()`
    - Test location: `tests/properties/map_validation.rs`
    - **Validates: Requirement 1.7**

  - [ ]* 17.3 Write property test for player stays in bounds
    - **Property 3: Player Stays In Bounds**
    - Generate a valid map (2–16 dimensions), valid spawn position, random sequence of 1–100 movement directions
    - Simulate movement logic (with bounds checking), assert `grid_x < width && grid_y < height` after all moves
    - Test location: `tests/properties/player_bounds.rs`
    - **Validates: Requirement 5.5**

  - [ ]* 17.4 Write property test for collision blocks movement
    - **Property 4: Collision Blocks Movement**
    - Generate a small map with random opacity flags, place player adjacent to an opaque tile, attempt move into it
    - Assert player position unchanged after move attempt
    - Test location: `tests/properties/collision.rs`
    - **Validates: Requirement 6.2**

  - [ ]* 17.5 Write property test for spawn point clamping
    - **Property 5: Spawn Point Clamping**
    - Generate valid maps and spawn points with coordinates potentially exceeding map dimensions
    - Assert initial player position equals `(min(spawn_x, width-1), min(spawn_y, height-1))`
    - Test location: `tests/properties/spawn_clamping.rs`
    - **Validates: Requirement 4.5**

  - [ ]* 17.6 Write property test for JumpTo target clamping
    - **Property 6: JumpTo Target Clamping**
    - Generate a project with two maps, place a JumpTo trigger with potentially out-of-bounds target coordinates
    - Assert player position after JumpTo equals clamped coordinates on target map
    - Test location: `tests/properties/jumpto_clamping.rs`
    - **Validates: Requirement 7.5**

  - [ ]* 17.7 Write property test for tile position correctness
    - **Property 7: Tile Position Correctness**
    - Generate small maps with random tile placements
    - For each tile at grid `(x, y)` with tile dimensions `(tw, th)`, assert world position equals `(x*tw + tw/2, -(y*th + th/2))`
    - Test location: `tests/properties/tile_positioning.rs`
    - **Validates: Requirement 3.4**

  - [ ]* 17.8 Write property test for movement animation exclusivity
    - **Property 8: Movement Animation Exclusivity**
    - Start a movement (set `move_animation` to `Some`), inject additional direction input
    - Assert `move_animation.to` remains unchanged after the second input attempt
    - Test location: `tests/properties/movement_exclusivity.rs`
    - **Validates: Requirement 5.4**

- [ ] 18. Final checkpoint — All tests pass
  - Ensure `cargo test --workspace` succeeds, ask the user if questions arise.

## Notes

- Tasks marked with `*` are optional and can be skipped for faster MVP
- Each task references specific requirements for traceability
- Checkpoints ensure incremental validation after each major phase
- Property tests validate the 8 correctness properties from the design document using `proptest`
- The editor crate refactoring (tasks 4.x) preserves compilation but does not change editor behavior
