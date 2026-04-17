# Implementation Plan: RPG Map Editor Foundation

## Overview

Milestone-based implementation of the RPG Map Editor using Bevy 0.18 and egui 0.39. Each milestone produces a runnable thick client the user can launch and visually verify. Property-based tests and unit tests are included within the milestone they validate.

## Tasks

### Milestone 1 — Window loads and egui is framing the editor

- [x] 1. Set up project structure and Bevy window
  - [x] 1.1 Initialize Cargo project with directory structure (`src/plugins/`, `src/data/`, `src/systems/`, `tests/properties/`, `tests/unit/`)
    - Configure `Cargo.toml` with all dependencies: `bevy 0.18`, `bevy_egui 0.39`, `serde`, `serde_json`, `rfd 0.15`, `thiserror 2`, and dev-dependency `proptest 1`
    - _Requirements: 1.1, 1.6_

  - [x] 1.2 Create `src/main.rs` with minimal Bevy app that opens a window (title "RPG Map Editor", minimum size 800×600)
    - Register `bevy_egui::EguiPlugin`
    - _Requirements: 1.1, 1.6_

  - [x] 1.3 Create `src/plugins/app_shell.rs` — `AppShellPlugin` with basic layout
    - Render egui menu bar with placeholder File (New Map, Load Tileset, Save Project, Open Project) and Edit (Undo, Redo) menus
    - Render egui central panel (canvas placeholder) and side panel (tile palette placeholder)
    - Create `src/plugins/mod.rs` to re-export the plugin
    - Register `AppShellPlugin` in `main.rs`
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5_

- [ ] 2. Checkpoint — User can `cargo run` and see the editor frame
  - Ensure the app compiles and launches, showing the menu bar, canvas area, and side panel placeholders. Ask the user if questions arise.

### Milestone 2 — Create project, create maps, and load tilesets

- [x] 3. Implement core data models
  - [x] 3.1 Create `src/data/map.rs` — define `TileIndex`, `Layer`, `MapData` structs with serde derives
    - Implement `MapData::new(name, width, height) -> Result<MapData, EditorError>` with dimension validation (1–256) and a single "Ground" layer with empty w×h grid
    - Implement `MapData::validate(&self) -> Result<(), EditorError>` for post-deserialization checks
    - _Requirements: 2.2, 2.3, 2.4, 5.1_

  - [x] 3.2 Create `src/data/tileset.rs` — define `TilesetMeta` and `TilesetData` structs
    - Implement `TilesetMeta::from_image_dimensions(img_w, img_h, tile_w, tile_h) -> Result<..>` that validates tile sizes ∈ {8,16,32,64} and computes columns/rows
    - _Requirements: 3.2, 3.6_

  - [x] 3.3 Create `src/data/editor_state.rs` — define `EditorState`, `ToolMode`, `EditorError` enum
    - Implement zoom clamping logic (0.25..=8.0)
    - _Requirements: 6.2, 8.3_

  - [x] 3.4 Create `src/data/project.rs` — define `ProjectFile` struct with version field
    - Implement `ProjectFile::serialize` and `ProjectFile::deserialize` with validation
    - _Requirements: 7.1, 7.2, 7.4, 7.5, 7.6_

  - [x] 3.5 Create `src/data/mod.rs` to re-export all data modules
    - _Requirements: N/A (structural)_

- [x] 4. Implement New Map dialog and map creation UI
  - [x] 4.1 Add New Map egui dialog to `AppShellPlugin`
    - Dialog prompts for map name, width, height with validation
    - On confirm: create `MapData` resource, display error dialog on invalid input
    - _Requirements: 2.1, 2.3, 2.4_

  - [x] 4.2 Create `src/plugins/canvas.rs` — `CanvasPlugin` with empty grid rendering
    - Spawn 2D camera, render grid overlay aligned to tile boundaries
    - On new map creation, set zoom to fit entire map
    - Register plugin in `main.rs`
    - _Requirements: 1.3, 2.5, 6.4, 6.5_

- [x] 5. Implement tileset loading and tile palette display
  - [x] 5.1 Add Load Tileset dialog to `AppShellPlugin`
    - Use `rfd` for native file dialog filtered to PNG/JPEG
    - On file selection: load image, partition into tile grid using `TilesetMeta`, create `TilesetData` resource
    - Display error dialogs for unsupported formats or corrupted files
    - _Requirements: 3.1, 3.4, 3.5_

  - [x] 5.2 Create `src/plugins/tile_palette.rs` — `TilePalettePlugin`
    - Render egui side panel showing loaded tileset as scrollable tile grid
    - Handle tile selection → update `EditorState.active_brush`
    - Display tile size configuration UI
    - Register plugin in `main.rs`
    - _Requirements: 1.4, 3.3, 3.6, 4.1_

- [ ] 6. Property tests for Milestone 2 data models
  - [ ]* 6.1 Write property test for map dimension validation
    - **Property 1: Map dimension validation**
    - **Validates: Requirements 2.3, 2.4**

  - [ ]* 6.2 Write property test for new map initialization
    - **Property 2: New map is correctly initialized**
    - **Validates: Requirements 2.2, 5.1**

  - [ ]* 6.3 Write property test for tileset grid partitioning
    - **Property 3: Tileset grid partitioning**
    - **Validates: Requirements 3.2**

  - [ ]* 6.4 Write property test for zoom level clamping
    - **Property 9: Zoom level clamping**
    - **Validates: Requirements 6.2**

- [x] 7. Checkpoint — User can create a map, load a tileset, and see tiles in the palette
  - Ensure all tests pass. User can `cargo run`, create a new map via the dialog, load a tileset image, and see tiles displayed in the palette panel with the empty grid on the canvas. Ask the user if questions arise.

### Milestone 3 — Paint and erase tiles onto map

- [x] 8. Implement tile painting and erasure
  - [x] 8.1 Implement pure functions for tile placement and erasure on `MapData`
    - `MapData::place_tile(layer_index, x, y, tile_index) -> Result<EditCommand, EditorError>` — sets the tile and returns the command with old value for undo
    - `MapData::erase_tile(layer_index, x, y) -> Result<EditCommand, EditorError>` — clears the tile and returns the command with old value
    - Define `EditCommand`, `EditCommandKind` structs/enums in `src/data/editor_state.rs`
    - _Requirements: 4.2, 4.3, 4.5_

  - [x] 8.2 Create `src/plugins/painting.rs` — `PaintingPlugin`
    - Implement screen-to-tile coordinate conversion
    - Read mouse input over canvas: left-click/drag places active brush tile, right-click erases tile
    - Emit `EditCommand` events for undo/redo tracking
    - Register plugin in `main.rs`
    - _Requirements: 4.2, 4.3, 4.4, 4.5_

  - [x] 8.3 Create `src/systems/render.rs` — tile sprite rendering
    - System to sync sprite transforms and texture atlas indices from `MapData` and `TilesetData`
    - Render placed tiles on the canvas using tileset graphics
    - _Requirements: 4.4, 5.5_

  - [x] 8.4 Create `src/systems/input.rs` — centralized mouse/keyboard input handling
    - Dispatch input events to painting and canvas systems
    - _Requirements: 4.2, 4.3, 4.5, 6.1, 6.3_

- [x] 9. Implement canvas navigation (zoom and pan)
  - [x] 9.1 Add zoom and pan systems to `CanvasPlugin`
    - Mouse wheel zoom centered on cursor position
    - Middle-mouse drag for panning
    - _Requirements: 6.1, 6.2, 6.3_

  - [x] 9.2 Create `src/systems/camera.rs` — camera transform systems
    - Apply zoom and pan transforms to the 2D camera
    - _Requirements: 6.1, 6.2, 6.3, 6.4_

- [ ] 10. Property tests for Milestone 3
  - [ ]* 10.1 Write property test for tile placement
    - **Property 4: Tile placement writes to correct cell**
    - **Validates: Requirements 4.2, 4.3**

  - [ ]* 10.2 Write property test for tile erasure
    - **Property 5: Tile erasure clears the cell**
    - **Validates: Requirements 4.5**

- [ ] 11. Checkpoint — User can select tiles and paint/erase on the map
  - Ensure all tests pass. User can `cargo run`, select a tile from the palette, paint on the canvas with left-click/drag, erase with right-click, and zoom/pan the canvas. Ask the user if questions arise.

### Milestone 4 — Layer support, edit queue (undo/redo)

- [x] 12. Implement layer management
  - [x] 12.1 Implement layer operations on `MapData`
    - `MapData::add_layer(name)` — inserts new empty layer above active layer, returns `EditCommand`
    - `MapData::delete_layer(index)` — removes layer (guards against deleting last layer), returns `EditCommand` with layer data
    - `MapData::toggle_layer_visibility(index)` — flips `visible` flag
    - `MapData::set_active_layer(index)` — validates index
    - _Requirements: 5.3, 5.4, 5.6, 5.7, 5.8_

  - [x] 12.2 Create `src/plugins/layer_panel.rs` — `LayerPanelPlugin`
    - Render egui panel listing layers with name, visibility toggle, selection highlight
    - Add Layer / Delete Layer buttons (disable delete when one layer remains)
    - Wire to `MapData` layer operations
    - Register plugin in `main.rs`
    - _Requirements: 5.2, 5.3, 5.4, 5.5, 5.6, 5.7, 5.8_

  - [x] 12.3 Update `src/systems/render.rs` to render all visible layers composited in stacking order
    - _Requirements: 5.5_

- [x] 13. Implement undo/redo system
  - [x] 13.1 Define `UndoHistory` resource in `src/data/editor_state.rs`
    - `push_command(&mut self, cmd)` — pushes to undo stack, clears redo stack, enforces max 50 entries
    - `undo(&mut self, map) -> bool` — pops undo stack, applies inverse, pushes to redo stack
    - `redo(&mut self, map) -> bool` — pops redo stack, applies command, pushes to undo stack
    - Implement `EditCommand::apply` and `EditCommand::apply_inverse` for all command kinds (tile placement, erasure, layer add, layer delete)
    - _Requirements: 8.1, 8.2, 8.3, 8.4, 8.5_

  - [x] 13.2 Create `src/plugins/undo_redo.rs` — `UndoRedoPlugin`
    - Register `UndoHistory` as Bevy resource
    - System to consume `EditCommand` events and push to history
    - Systems for Ctrl+Z / Ctrl+Y keyboard shortcuts
    - Register plugin in `main.rs`
    - _Requirements: 8.1, 8.2, 8.3, 8.4, 8.5_

- [ ] 14. Property tests for Milestone 4
  - [ ]* 14.1 Write property test for add layer
    - **Property 6: Add layer increases layer count**
    - **Validates: Requirements 5.3**

  - [ ]* 14.2 Write property test for delete layer
    - **Property 7: Delete layer decreases layer count**
    - **Validates: Requirements 5.7**

  - [ ]* 14.3 Write property test for visibility toggle involution
    - **Property 8: Layer visibility toggle is an involution**
    - **Validates: Requirements 5.6**

  - [ ]* 14.4 Write property test for undo/redo round-trip
    - **Property 12: Undo/redo round-trip preserves map state**
    - **Validates: Requirements 8.1, 8.2, 8.5**

  - [ ]* 14.5 Write property test for undo history max size
    - **Property 13: Undo history respects maximum size**
    - **Validates: Requirements 8.3**

  - [ ]* 14.6 Write property test for new edit clears redo stack
    - **Property 14: New edit clears redo stack**
    - **Validates: Requirements 8.4**

- [ ] 15. Checkpoint — User can manage layers and undo/redo their edits
  - Ensure all tests pass. User can `cargo run`, add/delete layers, toggle layer visibility, select active layer, and undo/redo tile and layer edits with Ctrl+Z/Ctrl+Y. Ask the user if questions arise.

### Milestone 5 — Save / load support

- [x] 16. Implement project save/load and file dialogs
  - [x] 16.1 Create `src/plugins/serialization.rs` — `SerializationPlugin`
    - Save system: serialize `MapData` + tileset meta → JSON via `ProjectFile` with pretty printing, use `rfd` for file dialog
    - Load system: deserialize JSON → validate → populate `MapData`, trigger tileset reload
    - Handle save path tracking and "Save As" vs "Save" logic
    - Register plugin in `main.rs`
    - _Requirements: 7.1, 7.2, 7.3, 7.4, 7.5, 7.6, 7.7_

  - [x] 16.2 Implement unsaved changes prompt
    - When user has unsaved changes and attempts to close or create a new map, prompt to save/discard/cancel
    - Wire into `AppShellPlugin` dialog state
    - _Requirements: 7.8_

  - [x] 16.3 Wire Save Project and Open Project menu items to serialization systems
    - Connect File menu actions to `SerializationPlugin` systems
    - Create `src/systems/mod.rs` to re-export system modules
    - _Requirements: 7.7_

- [ ] 17. Property tests and unit tests for Milestone 5
  - [ ]* 17.1 Write property test for serialization round-trip
    - **Property 10: Project serialization round-trip**
    - **Validates: Requirements 7.1, 7.2, 7.3, 7.5**

  - [ ]* 17.2 Write property test for pretty-printed JSON
    - **Property 11: Pretty-printed JSON contains indentation**
    - **Validates: Requirements 7.4**

  - [ ]* 17.3 Write unit tests for map creation edge cases
    - Test 1×1, 256×256, 0×0, 257×1 dimensions
    - Test default layer name is "Ground"
    - _Requirements: 2.2, 2.3, 2.4, 5.1_

  - [ ]* 17.4 Write unit tests for tileset loading
    - Test each valid tile size (8, 16, 32, 64)
    - Test rejection of invalid tile sizes
    - _Requirements: 3.2, 3.4, 3.5, 3.6_

  - [ ]* 17.5 Write unit tests for painting and layer operations
    - Test specific paint/erase scenarios, out-of-bounds rejection
    - Test last-layer deletion guard
    - _Requirements: 4.2, 4.5, 5.7, 5.8_

  - [ ]* 17.6 Write unit tests for serialization error handling
    - Test malformed JSON, missing fields, invalid data post-deserialization
    - _Requirements: 7.5, 7.6_

  - [ ]* 17.7 Write unit tests for undo/redo sequences
    - Test undo all, redo all, interleaved edits
    - _Requirements: 8.1, 8.2, 8.4, 8.5_

- [ ] 18. Final checkpoint — Ensure all tests pass
  - Ensure all tests pass. User can `cargo run`, save a project to disk, reload it, and verify all map/layer/tileset data is preserved. Unsaved changes prompt works correctly. Ask the user if questions arise.

## Notes

- Tasks marked with `*` are optional and can be skipped for faster MVP
- Each milestone produces a runnable thick client the user can visually test
- Each task references specific requirements for traceability
- Checkpoints at the end of each milestone ensure incremental validation
- Property tests validate universal correctness properties from the design document (all 14 properties covered)
- Unit tests validate specific examples and edge cases
