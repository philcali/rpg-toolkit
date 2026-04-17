# Implementation Plan: Tile Attributes

## Overview

Add an attribute editing mode to the RPG map editor with opacity toggling, event triggers, and spawn point placement. Implementation proceeds bottom-up: data models first, then undo/redo extensions, serialization, painting gate, attribute plugin UI, overlay rendering, and finally wiring everything together.

## Tasks

- [x] 1. Define core data models and extend Layer/Project
  - [x] 1.1 Add TileAttributes, EventAction, TileAttributeLayer, and SpawnPoint structs to `src/data/map.rs`
    - Add `TileAttributes` struct with `opacity: bool` and `event_trigger: Vec<EventAction>`, deriving `Clone, Debug, Default, PartialEq, Serialize, Deserialize`
    - Add `EventAction` enum with `#[serde(tag = "type")]` and initial `JumpTo { target_map_id: MapId, target_x: u32, target_y: u32 }` variant
    - Add `TileAttributeLayer` struct with `cells: Vec<Vec<TileAttributes>>` and a `new(width, height)` constructor
    - Add `SpawnPoint` struct with `map_id: MapId, x: u32, y: u32`
    - _Requirements: 5.1, 5.2, 5.4, 3.8_

  - [x] 1.2 Add `attributes: TileAttributeLayer` field to `Layer` with `#[serde(default)]`
    - Add the field with `#[serde(default)]` for backward compatibility
    - Implement `Default` for `TileAttributeLayer` that produces an empty grid (for serde default)
    - Update `MapData::new` to initialize the ground layer with `TileAttributeLayer::new(width, height)`
    - Update `MapData::add_layer` to initialize the new layer with `TileAttributeLayer::new(self.width, self.height)`
    - Update `EditCommandKind::AddLayer::apply` to include attribute layer initialization
    - _Requirements: 5.1, 2.5, 2.6, 6.3_

  - [x] 1.3 Add `spawn_point: Option<SpawnPoint>` to `Project` and `ProjectFile`
    - Add field to `Project` struct (defaults to `None`)
    - Add `#[serde(default)]` field to `ProjectFile` struct
    - Update `ProjectFile::new` to accept and store spawn_point
    - Update serialization save/load in `src/plugins/serialization.rs` to include spawn_point
    - _Requirements: 4.3, 4.8, 6.1, 6.2_

  - [ ]* 1.4 Write property tests for data model invariants
    - **Property 2: New layers have all-default attributes**
    - **Validates: Requirements 2.5, 2.6, 3.8**

  - [ ]* 1.5 Write property test for attribute grid dimensions
    - **Property 6: Attribute grid dimensions match tile grid**
    - **Validates: Requirements 5.1**

- [x] 2. Add EditorMode and AttributeTool to EditorState
  - [x] 2.1 Add `EditorMode` enum and `AttributeTool` enum to `src/data/editor_state.rs`
    - Add `EditorMode` enum with `Paint` (default) and `Attribute` variants
    - Add `AttributeTool` enum with `Opacity` (default), `EventTrigger`, and `SpawnPoint` variants
    - Add `editor_mode: EditorMode`, `attribute_tool: AttributeTool`, and `previous_tool: Option<EditorTool>` fields to `EditorState`
    - Export new types from `src/data/mod.rs`
    - _Requirements: 1.1, 1.4_

  - [ ]* 2.2 Write property test for mode toggle tool restoration
    - **Property 13: Mode toggle restores previous tool**
    - **Validates: Requirements 1.4**

- [x] 3. Extend EditCommandKind with attribute variants and update undo/redo
  - [x] 3.1 Add SetOpacity, SetEventTrigger, and SetSpawnPoint variants to `EditCommandKind`
    - Add `SetOpacity { layer_index, x, y, old_value: bool, new_value: bool }`
    - Add `SetEventTrigger { layer_index, x, y, old_trigger: Vec<EventAction>, new_trigger: Vec<EventAction> }`
    - Add `SetSpawnPoint { old_spawn: Option<SpawnPoint>, new_spawn: Option<SpawnPoint> }`
    - Implement `apply` and `apply_inverse` for all three variants in `EditCommand`
    - SetOpacity and SetEventTrigger operate on `map.layers[layer_index].attributes.cells[y][x]`
    - SetSpawnPoint is a no-op on MapData (handled at Project level)
    - _Requirements: 7.1, 7.2, 7.3, 7.4, 7.5_

  - [x] 3.2 Update `consume_edit_commands` in `src/plugins/undo_redo.rs` to handle SetSpawnPoint
    - Before pushing to undo history, check if command is `SetSpawnPoint`
    - If so, apply the spawn point change directly to `project.spawn_point`
    - For undo/redo of SetSpawnPoint, also apply/inverse on `project.spawn_point`
    - _Requirements: 7.3, 7.4, 7.5_

  - [ ]* 3.3 Write property test for attribute undo/redo round-trip
    - **Property 11: Attribute edit undo/redo round-trip**
    - **Validates: Requirements 7.1, 7.2, 7.3, 7.4, 7.5**

- [ ] 4. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [x] 5. Gate painting system and handle map deletion
  - [x] 5.1 Add early return to `painting_system` when in attribute mode
    - Read `EditorState` resource and check `editor_mode == EditorMode::Attribute`
    - If attribute mode, return early before any painting logic
    - _Requirements: 1.2_

  - [x] 5.2 Clear spawn point on map deletion in `Project::remove_map`
    - After removing the map, check if `self.spawn_point` references the deleted map
    - If so, set `self.spawn_point = None`
    - _Requirements: (Error handling table in design)_

  - [ ]* 5.3 Write property test for opacity toggle
    - **Property 1: Opacity toggle inverts value**
    - **Validates: Requirements 2.1, 2.2, 2.3**

- [x] 6. Implement serialization round-trip and backward compatibility
  - [x] 6.1 Update `ProjectFile` serialization to include attribute data
    - Ensure `Layer.attributes` is serialized (already handled by serde derive + the field addition)
    - Ensure `ProjectFile.spawn_point` is serialized and deserialized
    - Update `load_project_with_dialog` to copy spawn_point from ProjectFile to Project
    - Update `save_project_to_path` to include spawn_point in ProjectFile
    - Add warning log for JumpTo actions referencing non-existent maps on load
    - _Requirements: 6.1, 6.2, 6.3, 6.4, 6.5_

  - [ ]* 6.2 Write property test for serialization round-trip
    - **Property 8: ProjectFile serialization round-trip**
    - **Validates: Requirements 6.1, 6.2, 6.5**

  - [ ]* 6.3 Write property test for backward-compatible deserialization
    - **Property 9: Backward-compatible deserialization defaults**
    - **Validates: Requirements 6.3**

  - [ ]* 6.4 Write property test for dangling map references
    - **Property 10: Dangling map references preserved on load**
    - **Validates: Requirements 6.4**

- [ ] 7. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [x] 8. Implement AttributePlugin with toolbar, overlay, and click handling
  - [x] 8.1 Create `src/plugins/attribute.rs` with `AttributePlugin` struct and register in `main.rs`
    - Create the plugin file with `SpawnPointConfirmDialog` and `EventTriggerDialog` resources
    - Register `AttributePlugin` in `main.rs`
    - Add `pub mod attribute;` and `pub use attribute::AttributePlugin;` to `src/plugins/mod.rs`
    - _Requirements: 1.1_

  - [x] 8.2 Implement mode toggle in toolbar UI
    - Add a Paint/Attribute mode toggle button to the existing toolbar in `src/plugins/toolbar.rs`
    - When switching to Attribute mode: store current `EditorTool` in `editor_state.previous_tool`, set `editor_mode = Attribute`
    - When switching back to Paint mode: restore `editor_state.previous_tool`, set `editor_mode = Paint`
    - When in Attribute mode, show attribute tool buttons (Opacity, EventTrigger, SpawnPoint) instead of paint tools
    - Show a visual indicator for the current mode
    - _Requirements: 1.1, 1.4, 1.5_

  - [x] 8.3 Implement attribute overlay rendering system
    - Add `attribute_overlay_system` that draws gizmos on the canvas when in attribute mode
    - Render colored rectangles on tiles with `opacity == true` (e.g., red semi-transparent)
    - Render a different indicator on tiles with non-empty `event_trigger` (e.g., blue icon/rectangle)
    - Render a spawn point marker if `project.spawn_point` is on the current map
    - Only render when `editor_mode == Attribute`
    - _Requirements: 1.3, 2.4, 3.6, 4.7_

  - [x] 8.4 Implement opacity click system
    - Add `attribute_click_system` that handles left-click in attribute mode
    - When `attribute_tool == Opacity` and user clicks a tile: toggle `opacity` on the active layer's attribute grid
    - Emit `EditCommand` with `SetOpacity` kind for undo/redo
    - _Requirements: 2.1, 2.2, 2.3, 7.1_

  - [x] 8.5 Implement event trigger configuration panel
    - Add `event_trigger_panel_ui` egui system that shows when `attribute_tool == EventTrigger` and user clicks a tile
    - Display ordered list of `EventAction` items with add/remove/reorder controls
    - For JumpTo: show map selector dropdown (from `project.maps`) and x/y coordinate fields
    - On save: emit `EditCommand` with `SetEventTrigger` kind
    - On open: populate panel with existing trigger data for the selected tile
    - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5, 3.7, 7.2_

  - [x] 8.6 Implement spawn point placement system
    - When `attribute_tool == SpawnPoint` and user clicks a tile:
      - If no existing spawn point: set `project.spawn_point` directly, emit `SetSpawnPoint` command
      - If spawn point exists: open `SpawnPointConfirmDialog` with new location info
    - Implement confirm/cancel logic in the dialog
    - Spawn point always targets layer index 0 regardless of active layer
    - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5, 4.6, 4.9, 7.3_

- [-] 9. Wire everything together and add remaining property tests
  - [x] 9.1 Ensure all attribute systems are ordered correctly in the Bevy schedule
    - `attribute_click_system` runs after `update_cursor_state`
    - `attribute_overlay_system` runs after `draw_grid`
    - Attribute UI systems run in `EguiPrimaryContextPass`
    - _Requirements: 1.1, 1.2, 1.3_

  - [ ]* 9.2 Write property test for event trigger round-trip
    - **Property 3: Event trigger storage round-trip**
    - **Validates: Requirements 3.1, 3.5**

  - [ ]* 9.3 Write property test for spawn point placement
    - **Property 4: Spawn point placement stores correct location**
    - **Validates: Requirements 4.2, 4.3, 4.5, 4.9**

- [ ] 10. Final checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

## Notes

- Tasks marked with `*` are optional and can be skipped for faster MVP
- Each task references specific requirements for traceability
- Checkpoints ensure incremental validation
- Property tests validate universal correctness properties from the design document
- SetSpawnPoint is unique among edit commands: it operates on Project.spawn_point, not MapData
- `#[serde(default)]` on Layer.attributes ensures backward compatibility with existing project files
