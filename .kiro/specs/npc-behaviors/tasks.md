# Implementation Plan: NPC Behaviors

## Overview

Incremental implementation activating NPC patrol movement, event triggers, and editor tooling. Starts with data model changes in rpg-toolkit-common, then builds renderer systems for patrol movement, animation, dynamic collision, and trigger handling, followed by editor UI extensions for patrol path and event trigger configuration. Each task builds on the previous, with property-based tests placed close to the code they validate.

## Tasks

- [x] 1. Update NPC data model in rpg-toolkit-common
  - [x] 1.1 Add `PatrolMode`, `PatrolConfig`, and `TriggerMode` types and update `NpcInstance`
    - Add `PatrolMode` enum (`Loop`, `PingPong`, `OneShot` with `Default = Loop`) to `crates/rpg-toolkit-common/src/spritesheet.rs`
    - Add `PatrolConfig` struct with `waypoints: Vec<(u32, u32)>`, `mode: PatrolMode`, `speed: f32` (default 0.3), `pause: f32` (default 0.5) to `crates/rpg-toolkit-common/src/spritesheet.rs`
    - Add `TriggerMode` enum (`Collision`, `Interaction` with `Default = Interaction`) to `crates/rpg-toolkit-common/src/spritesheet.rs`
    - Replace `patrol_path: Vec<(u32, u32)>` with `#[serde(default)] pub patrol_config: Option<PatrolConfig>` on `NpcInstance`
    - Add `#[serde(default)] pub trigger_mode: TriggerMode` to `NpcInstance`
    - Export new types from `crates/rpg-toolkit-common/src/lib.rs`
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 5.1, 6.1, 6.3_

  - [x] 1.2 Add `next_waypoint_index`, `validate_waypoint_bounds`, and `faced_tile` utility functions
    - Add `next_waypoint_index(current: usize, waypoint_count: usize, mode: PatrolMode, forward: bool) -> (usize, bool)` to `crates/rpg-toolkit-common/src/spritesheet.rs` implementing Loop wrap-around, PingPong reversal, and OneShot stop-at-end logic
    - Add `validate_waypoint_bounds(waypoint: (u32, u32), map_width: u32, map_height: u32) -> bool` returning true iff `wx < width && wy < height`
    - Add `faced_tile(player_x: u32, player_y: u32, facing: FacingDirection) -> Option<(u32, u32)>` returning the adjacent tile in the given direction, or `None` at map boundaries
    - Export new functions from `crates/rpg-toolkit-common/src/lib.rs`
    - _Requirements: 2.4, 2.5, 2.6, 9.2, 10.1, 10.2_

  - [ ]* 1.3 Write property test: NpcInstance serialization round-trip (Property 1)
    - **Property 1: NpcInstance serialization round-trip**
    - Generate random `NpcInstance` with optional `PatrolConfig` (0–5 waypoints, random mode/speed/pause), random `TriggerMode`, 0–3 `EventAction` entries
    - Serialize to JSON then deserialize; assert equivalence
    - Add to `tests/properties/project_round_trip.rs`
    - **Validates: Requirements 1.3, 6.2**

  - [ ]* 1.4 Write property test: Backward-compatible deserialization (Property 2)
    - **Property 2: Backward-compatible deserialization**
    - Generate random `NpcInstance` JSON omitting `patrol_config`, `trigger_mode`, and `event_triggers` fields
    - Assert deserialization succeeds with defaults (`patrol_config: None`, `trigger_mode: Interaction`, `event_triggers: []`)
    - Re-serialize and deserialize again; assert stability
    - Add to `tests/properties/project_round_trip.rs`
    - **Validates: Requirements 1.4, 6.3**

  - [ ]* 1.5 Write property test: Patrol mode next waypoint calculation (Property 3)
    - **Property 3: Patrol mode next waypoint calculation**
    - Generate random waypoint count (2–10), random current index within bounds, all three `PatrolMode` variants, both forward/backward directions
    - Assert Loop wraps, PingPong reverses at endpoints, OneShot stops at last
    - Create `tests/properties/npc_patrol_waypoint.rs`
    - **Validates: Requirements 2.4, 2.5, 2.6**

  - [ ]* 1.6 Write property test: Faced tile calculation (Property 12)
    - **Property 12: Faced tile calculation**
    - Generate random player positions (0–255), all four `FacingDirection` variants
    - Assert Up → `(x, y-1)`, Down → `(x, y+1)`, Left → `(x-1, y)`, Right → `(x+1, y)`, and `None` at boundaries
    - Create `tests/properties/npc_faced_tile.rs`
    - **Validates: Requirements 9.2**

  - [ ]* 1.7 Write property test: Waypoint bounds validation (Property 13)
    - **Property 13: Waypoint bounds validation**
    - Generate random map dimensions (1–256), random waypoint positions (0–300)
    - Assert accepted iff `wx < width && wy < height`
    - Create `tests/properties/npc_waypoint_bounds.rs`
    - **Validates: Requirements 10.1, 10.2**

- [x] 2. Checkpoint — Ensure common crate changes compile and tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [x] 3. Add renderer components, resources, and NPC spawn changes
  - [x] 3.1 Add `NpcSpriteState`, `NpcMoveAnimation`, `NpcPatrolState` components and `NpcPositions`, `InteractionIntent` resources
    - Add `NpcSpriteState` component (facing, animation_frame, animation_timer, is_moving, grid_x, grid_y, move_animation, patrol, y_offset) to `crates/rpg-toolkit-renderer/src/components.rs`
    - Add `NpcMoveAnimation` struct (from, to, from_grid, to_grid, elapsed, duration) to `crates/rpg-toolkit-renderer/src/components.rs`
    - Add `NpcPatrolState` struct (current_waypoint_index, forward, pause_timer, paused, finished) to `crates/rpg-toolkit-renderer/src/components.rs`
    - Add `NpcPositions` resource with `positions: Vec<(u32, u32)>`, `is_occupied(x, y)`, and `is_occupied_by_other(x, y, exclude_index)` methods to `crates/rpg-toolkit-renderer/src/resources.rs`
    - Add `InteractionIntent` resource with `pressed: bool` to `crates/rpg-toolkit-renderer/src/resources.rs`
    - _Requirements: 3.5, 4.1, 4.2, 9.1_

  - [x] 3.2 Modify `spawn_npc_sprites` to attach `NpcSpriteState` component
    - In `crates/rpg-toolkit-renderer/src/systems/map_render.rs`, extend the NPC entity spawn to include `NpcSpriteState` initialized with the NPC's facing direction, idle frame (1), grid position, y_offset, and optional `NpcPatrolState` built from the NPC's `patrol_config`
    - Compute y_offset using the same logic as the player sprite (scaled sprite height minus tile height divided by 2)
    - _Requirements: 3.1, 3.3, 3.5_

  - [x] 3.3 Add `init_npc_positions` system to rebuild `NpcPositions` on map change
    - Create `init_npc_positions` system in `crates/rpg-toolkit-renderer/src/systems/map_render.rs` that listens for `MapChanged` events and rebuilds the `NpcPositions` resource from the active map's NPC instances
    - _Requirements: 4.1, 4.2_

  - [x] 3.4 Modify `is_tile_blocked` to accept optional `NpcPositions` parameter
    - Change signature of `is_tile_blocked` in `crates/rpg-toolkit-renderer/src/systems/collision.rs` to `is_tile_blocked(map: &MapData, x: u32, y: u32, npc_positions: Option<&NpcPositions>) -> bool`
    - When `npc_positions` is `Some`, use dynamic positions; when `None`, fall back to static `map.npcs` check
    - Update all call sites: `player_movement` passes `Some(&npc_positions)`, other callers pass `None`
    - _Requirements: 4.1, 4.2_

- [x] 4. Implement NPC patrol movement and animation systems
  - [x] 4.1 Implement `read_interaction_input` system
    - Create `read_interaction_input` system in a new file `crates/rpg-toolkit-renderer/src/systems/npc.rs` that reads Space/Enter key presses and writes to `InteractionIntent` resource
    - Reset `pressed` to false each frame before checking input
    - _Requirements: 9.1, 9.4_

  - [x] 4.2 Implement `npc_patrol_movement` system
    - Create `npc_patrol_movement` system in `crates/rpg-toolkit-renderer/src/systems/npc.rs` that iterates all `NpcSpriteState` entities with patrol state
    - Handle pause timer countdown at waypoints
    - When pause completes, compute next waypoint step using `next_waypoint_index`, check if destination tile is blocked (via `NpcPositions` and opacity), and if clear initiate a `NpcMoveAnimation`
    - Update `NpcPositions` to destination tile at move start
    - Advance `NpcMoveAnimation` each frame using linear interpolation; on completion, update grid position and enter pause state
    - Handle OneShot finished state (stop moving)
    - Clamp speed to minimum 0.01s, pause to minimum 0.0s
    - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 2.7, 4.3, 4.4_

  - [x] 4.3 Implement `npc_patrol_animation` system
    - Create `npc_patrol_animation` system in `crates/rpg-toolkit-renderer/src/systems/npc.rs` that updates each NPC's sprite atlas index based on `NpcSpriteState`
    - While moving: cycle through walk frames using `walk_animation_frame` and `sprite_atlas_index`, update facing direction to match movement direction
    - While idle/paused: display idle frame (frame 1) for current facing direction
    - Update sprite `Transform` position during movement animation (interpolate + y_offset)
    - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5_

  - [ ]* 4.4 Write property test: NPC facing matches movement direction (Property 14)
    - **Property 14: NPC facing matches movement direction**
    - Generate random NPC positions and movement directions
    - Assert facing direction matches movement direction (up→Up, down→Down, left→Left, right→Right)
    - Create `tests/properties/npc_facing_movement.rs`
    - **Validates: Requirements 3.2**

  - [ ]* 4.5 Write property test: Empty or absent patrol config produces no movement (Property 4)
    - **Property 4: Empty or absent patrol config produces no movement**
    - Generate random `NpcInstance` with `patrol_config: None` or empty waypoints
    - Assert NPC remains at initial grid position after patrol system runs
    - Create `tests/properties/npc_patrol_stationary.rs`
    - **Validates: Requirements 1.2**

  - [ ]* 4.6 Write property test: Waypoint pause timing (Property 15)
    - **Property 15: Waypoint pause timing**
    - Generate random pause durations (0.01–5.0) and elapsed times (0.0–10.0)
    - Assert NPC does not move while elapsed < pause duration
    - Create `tests/properties/npc_waypoint_pause.rs`
    - **Validates: Requirements 2.3**

- [x] 5. Implement NPC trigger system
  - [x] 5.1 Implement `npc_trigger_system`
    - Create `npc_trigger_system` in `crates/rpg-toolkit-renderer/src/systems/npc.rs` that handles both collision and interaction triggers
    - **Collision triggers**: Hook into player movement — when player attempts to move onto an NPC tile with `trigger_mode: Collision` and non-empty `event_triggers`, populate `ActionQueue` with the NPC's triggers instead of blocking silently. If `event_triggers` is empty, apply default block behavior
    - **Interaction triggers**: When `InteractionIntent.pressed` is true, compute `faced_tile` from player position and facing, check if an NPC with `trigger_mode: Interaction` and non-empty `event_triggers` occupies that tile, and if so populate `ActionQueue`. Update NPC facing to face the player before firing triggers
    - Skip if `ActionQueue` already exists (active sequence suppression)
    - Skip if dialog is active
    - _Requirements: 5.2, 5.3, 5.4, 5.5, 5.6, 9.2, 9.3, 9.4_

  - [x] 5.2 Modify `player_movement` to integrate with NPC collision triggers
    - In `crates/rpg-toolkit-renderer/src/systems/player.rs`, add `NpcPositions` resource parameter
    - Pass `Some(&npc_positions)` to `is_tile_blocked` for dynamic collision
    - When movement is blocked by an NPC (not opacity), check if the NPC has `trigger_mode: Collision` and non-empty `event_triggers` — if so, still block movement but signal the trigger system to fire (e.g., via a new `NpcCollisionEvent` message or by checking in `npc_trigger_system` after `player_movement`)
    - _Requirements: 4.1, 4.2, 5.2, 5.4_

  - [ ]* 5.3 Write property test: Collision trigger fires event_triggers (Property 7)
    - **Property 7: Collision trigger fires event_triggers into ActionQueue**
    - Generate random NPCs with `Collision` mode and 1–3 event triggers, random player positions
    - Assert `ActionQueue` is populated with NPC's triggers in order when player moves onto NPC tile
    - Create `tests/properties/npc_collision_trigger.rs`
    - **Validates: Requirements 5.2**

  - [ ]* 5.4 Write property test: Interaction trigger fires event_triggers (Property 8)
    - **Property 8: Interaction trigger fires event_triggers into ActionQueue**
    - Generate random NPCs with `Interaction` mode and 1–3 event triggers, random adjacent player positions
    - Assert `ActionQueue` is populated when player presses action key facing NPC
    - Create `tests/properties/npc_interaction_trigger.rs`
    - **Validates: Requirements 5.3**

  - [ ]* 5.5 Write property test: Empty event_triggers blocks regardless of TriggerMode (Property 9)
    - **Property 9: Empty event_triggers blocks regardless of TriggerMode**
    - Generate random NPCs with empty triggers, both trigger modes
    - Assert player is blocked and no `ActionQueue` is created
    - Create `tests/properties/npc_empty_triggers.rs`
    - **Validates: Requirements 5.4**

  - [ ]* 5.6 Write property test: Active ActionQueue suppresses new NPC triggers (Property 10)
    - **Property 10: Active ActionQueue suppresses new NPC triggers**
    - Generate random existing `ActionQueue` contents and random NPC trigger events
    - Assert existing `ActionQueue` is unchanged when new trigger fires
    - Create `tests/properties/npc_queue_suppression.rs`
    - **Validates: Requirements 5.5**

  - [ ]* 5.7 Write property test: NPC faces player on interaction trigger (Property 11)
    - **Property 11: NPC faces player on interaction trigger**
    - Generate random adjacent player/NPC positions (all four relative directions)
    - Assert NPC facing is updated to face toward the player
    - Create `tests/properties/npc_faces_player.rs`
    - **Validates: Requirements 5.6**

- [x] 6. Wire new systems into renderer plugin
  - [x] 6.1 Register new systems and resources in `ProjectRendererPlugin`
    - In `crates/rpg-toolkit-renderer/src/lib.rs`, register `NpcPositions` and `InteractionIntent` resources
    - Add `init_npc_positions` system after `spawn_npc_sprites`
    - Add `read_interaction_input` system after `read_input`
    - Add `npc_patrol_movement` system after `player_movement`
    - Add `npc_patrol_animation` system after `animate_player_sprite`
    - Add `npc_trigger_system` system after `check_triggers`
    - Update re-exports for new public types and systems
    - Add `mod npc;` to `crates/rpg-toolkit-renderer/src/systems/mod.rs`
    - _Requirements: 2.1, 3.1, 5.2, 5.3, 9.1_

- [x] 7. Checkpoint — Ensure renderer compiles and all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [x] 8. Extend editor NPC dialog with patrol path configuration
  - [x] 8.1 Add `UpdateNpcPatrol` and `UpdateNpcTriggers` variants to `EditCommandKind`
    - Add `UpdateNpcPatrol { npc_index: usize, old_config: Option<PatrolConfig>, new_config: Option<PatrolConfig> }` to `EditCommandKind` in `crates/rpg-toolkit-editor/src/data/editor_state.rs`
    - Add `UpdateNpcTriggers { npc_index: usize, old_trigger_mode: TriggerMode, new_trigger_mode: TriggerMode, old_event_triggers: Vec<EventAction>, new_event_triggers: Vec<EventAction> }` to `EditCommandKind`
    - Implement `apply()` and `apply_inverse()` for both new variants
    - _Requirements: 7.5, 8.5_

  - [x] 8.2 Extend `NpcPlacementDialog` with patrol path configuration fields
    - Add patrol config fields to `NpcPlacementDialog` resource in `crates/rpg-toolkit-editor/src/plugins/attribute.rs`: `patrol_waypoints: Vec<(u32, u32)>`, `patrol_mode: PatrolMode`, `patrol_speed: String`, `patrol_pause: String`, `adding_waypoints: bool`
    - Pre-populate from existing NPC's `patrol_config` when editing
    - Add patrol path panel UI to `npc_placement_dialog_ui`: waypoint list with remove buttons, PatrolMode radio buttons, speed/pause text fields, "Add Waypoints" toggle button
    - When "Add Waypoints" is active and user clicks map tiles, append clicked position to waypoints (with bounds validation using `validate_waypoint_bounds`)
    - On dialog confirm, build `PatrolConfig` from fields and emit `UpdateNpcPatrol` edit command
    - _Requirements: 7.1, 7.2, 7.3, 7.5, 7.6, 10.1, 10.2_

  - [x] 8.3 Add patrol path overlay rendering in the editor
    - In `attribute_overlay_system` in `crates/rpg-toolkit-editor/src/plugins/attribute.rs`, when an NPC with a patrol config is selected or when in NPC placement mode, draw the patrol path as connected line segments between waypoints with numbered markers at each waypoint position
    - Use a distinct color (e.g., yellow/orange) for patrol path lines
    - _Requirements: 7.4_

  - [x] 8.4 Extend `NpcPlacementDialog` with event trigger configuration
    - Add event trigger fields to `NpcPlacementDialog`: `trigger_mode: TriggerMode`, `event_triggers: Vec<EventAction>`, plus the same action editing fields used by `EventTriggerDialog` (action type, JumpTo fields, ShowDialog fields)
    - Pre-populate from existing NPC's `trigger_mode` and `event_triggers` when editing
    - Add trigger config panel UI to `npc_placement_dialog_ui`: TriggerMode radio buttons (Collision/Interaction), action list with add/edit/remove/reorder controls reusing the same UI pattern as `event_trigger_panel_ui`
    - On dialog confirm, emit `UpdateNpcTriggers` edit command
    - _Requirements: 8.1, 8.2, 8.3, 8.4, 8.5_

  - [x] 8.5 Wire undo/redo for new `UpdateNpcPatrol` and `UpdateNpcTriggers` commands
    - In `crates/rpg-toolkit-editor/src/plugins/undo_redo.rs`, ensure `consume_edit_commands` handles the new variants (they operate on `MapData` so the existing `apply`/`apply_inverse` flow handles them automatically)
    - Verify undo/redo keyboard shortcuts work for the new command types
    - _Requirements: 7.5, 8.5_

- [x] 9. Update existing property tests for NpcInstance field changes
  - [x] 9.1 Update `project_round_trip.rs` to generate new NpcInstance fields
    - Update the `NpcInstance` generator in `tests/properties/project_round_trip.rs` to generate `patrol_config: Option<PatrolConfig>` (with random waypoints, mode, speed, pause) and `trigger_mode: TriggerMode` instead of the old `patrol_path` field
    - Ensure the existing round-trip property test covers the new fields
    - _Requirements: 1.3, 6.2_

  - [ ]* 9.2 Write property test: NPC grid position updates to destination (Property 5)
    - **Property 5: NPC grid position updates to destination at move start**
    - Generate random NPC index, random from/to grid positions
    - Assert `NpcPositions` reflects destination tile from move start
    - Create `tests/properties/npc_position_update.rs`
    - **Validates: Requirements 2.7, 4.2**

  - [ ]* 9.3 Write property test: NPC patrol pauses when blocked (Property 6)
    - **Property 6: NPC patrol pauses when next tile is blocked**
    - Generate random map with opacity attributes and NPC positions, random patrol paths
    - Assert NPC does not move when next tile is blocked
    - Create `tests/properties/npc_patrol_blocked.rs`
    - **Validates: Requirements 4.3, 4.4**

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
- The old `patrol_path` field on `NpcInstance` was never written by any code path, so removing it is safe
- All new fields use `#[serde(default)]` for backward compatibility with existing project files
- The existing `walk_animation_frame` and `sprite_atlas_index` functions are reused for NPC animation
- The existing `ActionQueue` and `advance_action_queue` system are reused for NPC event triggers
