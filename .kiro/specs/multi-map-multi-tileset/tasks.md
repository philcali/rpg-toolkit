# Implementation Plan: Multi-Map Multi-Tileset

## Overview

Transform the RPG Toolkit editor from a single-map, single-tileset model into a project owning collections of maps and tilesets. This involves replacing singleton `Res<MapData>` and `Res<TilesetData>` with a central `Res<Project>` resource, introducing `TileRef` (replacing `TileIndex`), adding Map Tab Bar / Map Browser / Tileset Tab Bar UI components, scoping undo/redo per-map, and updating serialization.

Organized into 4 milestones for incremental review:
1. **Data model foundation** — new types, `Project` resource, serialization
2. **Tileset tab management** — tileset tabs in palette, tile selection producing `TileRef`
3. **Multi-map management** — map tabs, map browser, canvas/painting/undo per active map
4. **Fit and finish** — layer panel, wiring, cleanup, final validation

## Tasks

### Milestone 1: Data Model Foundation

- [x] 1. Introduce `TileRef`, `TilesetId`, `MapId` types and update `Layer`/`MapData`
  - [x] 1.1 Create `TileRef` struct and ID type aliases in `src/data/map.rs`
    - Add `pub type MapId = String;` and `pub type TilesetId = String;` (UUID v4 strings)
    - Add `TileRef { tileset_id: TilesetId, col: u32, row: u32 }` with `Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize`
    - Keep `TileIndex` temporarily for compilation but mark it `#[deprecated]`
    - _Requirements: 4.1_

  - [x] 1.2 Update `Layer` to use `Option<TileRef>` instead of `Option<TileIndex>`
    - Change `tiles: Vec<Vec<Option<TileIndex>>>` → `Vec<Vec<Option<TileRef>>>`
    - _Requirements: 4.1_

  - [x] 1.3 Add `tile_width` and `tile_height` fields to `MapData`
    - Add `pub tile_width: u32` and `pub tile_height: u32` fields
    - Update `MapData::new` to accept `tile_width`/`tile_height` parameters and validate them against `{8, 16, 32, 64}`
    - Remove the `Resource` derive from `MapData` (it will live inside `Project`)
    - _Requirements: 2.1, 2.3_

  - [x] 1.4 Update `EditCommand`/`EditCommandKind` to use `TileRef`
    - Change `PlaceTile.old_tile`/`new_tile` and `EraseTile.old_tile` from `TileIndex`/`Option<TileIndex>` to `TileRef`/`Option<TileRef>`
    - Update `apply` and `apply_inverse` methods accordingly
    - _Requirements: 4.1, 4.2_

  - [ ]* 1.5 Write property test for TileRef storage
    - **Property 7: Placing a tile stores the correct TileRef**
    - **Validates: Requirements 4.1, 4.2, 7.3**

- [x] 2. Create `TilesetEntry` and `Project` resource
  - [x] 2.1 Create `TilesetEntry` struct in `src/data/tileset.rs`
    - Add `TilesetEntry { meta: TilesetMeta, texture: Handle<Image>, atlas_layout: Handle<TextureAtlasLayout> }`
    - This replaces the current `TilesetData` resource structure
    - _Requirements: 3.1_

  - [x] 2.2 Create `Project` resource in `src/data/project.rs`
    - Add `Project` struct with fields: `maps: HashMap<MapId, MapData>`, `tilesets: HashMap<TilesetId, TilesetEntry>`, `open_tabs: Vec<MapId>`, `active_tab: Option<usize>`, `undo_histories: HashMap<MapId, UndoHistory>`, `has_unsaved_changes: HashMap<MapId, bool>`, `next_map_name_counter: u32`
    - Implement accessor methods: `active_map_id()`, `active_map()`, `active_map_mut()`, `active_undo_history_mut()`
    - _Requirements: 1.1, 3.1, 5.1, 9.1_

  - [x] 2.3 Implement `Project::add_map` and `Project::remove_map`
    - `add_map(name, w, h, tile_w, tile_h)` generates UUID, creates `MapData`, inserts into `maps`, initializes empty `UndoHistory`, opens tab, returns `MapId`
    - `remove_map(id)` returns error if only 1 map remains, otherwise removes from `maps`, `undo_histories`, `has_unsaved_changes`, and any open tabs
    - _Requirements: 1.1, 1.2, 1.3, 1.4_

  - [ ]* 2.4 Write property tests for map registry operations
    - **Property 1: Adding a map grows the registry and preserves existing maps**
    - **Property 2: Removing a map shrinks the registry**
    - **Validates: Requirements 1.1, 1.2, 1.3**

  - [x] 2.5 Implement `Project::add_tileset` and `Project::remove_tileset`
    - `add_tileset(meta, texture, layout)` generates UUID, creates `TilesetEntry`, inserts into `tilesets`, returns `TilesetId`
    - `remove_tileset(id)` removes from `tilesets` (caller handles confirmation for in-use tilesets)
    - _Requirements: 3.1, 3.2, 3.3_

  - [x] 2.6 Implement `Project::is_tileset_in_use` helper
    - Iterates all maps and all layers to check if any `TileRef` references the given `TilesetId`
    - _Requirements: 3.4_

  - [ ]* 2.7 Write property tests for tileset registry operations
    - **Property 3: Adding a tileset grows the registry and preserves existing tilesets**
    - **Property 4: Removing a tileset shrinks the registry**
    - **Property 5: Tileset-in-use detection**
    - **Validates: Requirements 3.1, 3.2, 3.3, 3.4**

  - [x] 2.8 Implement tab management methods on `Project`
    - `open_map_tab(id)`: adds to `open_tabs` if not present, sets `active_tab` to that index
    - `close_map_tab(idx)`: removes from `open_tabs`, adjusts `active_tab` to nearest remaining or `None`
    - `set_active_tab(idx)`: sets `active_tab` if valid index
    - _Requirements: 5.2, 5.3, 5.4, 5.5_

  - [ ]* 2.9 Write property tests for tab management
    - **Property 9: Setting the active tab selects the correct map**
    - **Property 10: Opening a map adds a tab and activates it**
    - **Property 11: Closing a tab removes it**
    - **Property 12: Closing the active tab activates the nearest remaining tab**
    - **Validates: Requirements 5.2, 5.3, 5.4, 5.5, 6.2, 11.2**

  - [x] 2.10 Update `src/data/mod.rs` exports
    - Export new types: `Project`, `TileRef`, `TilesetEntry`, `MapId`, `TilesetId`
    - Remove old singleton re-exports as needed
    - _Requirements: 1.1, 3.1, 4.1_

- [x] 3. Update `EditorState` and tileset compatibility validation
  - [x] 3.1 Update `EditorState` to use `TileRef` brush and add `active_tileset_tab`
    - Change `active_brush: Option<TileIndex>` → `Option<TileRef>`
    - Add `active_tileset_tab: Option<TilesetId>`
    - Remove `has_unsaved_changes` (now per-map in `Project`)
    - _Requirements: 4.2, 7.3_

  - [x] 3.2 Add tileset compatibility check to `Project` or a helper function
    - Given a `TilesetId` and the active map, verify `tileset.meta.tile_width == map.tile_width && tileset.meta.tile_height == map.tile_height`
    - Return error/warning if mismatched
    - _Requirements: 2.2_

  - [ ]* 3.3 Write property test for tileset compatibility
    - **Property 6: Tileset compatibility validation**
    - **Validates: Requirements 2.2**

- [x] 4. Update serialization for multi-map multi-tileset format
  - [x] 4.1 Rewrite `ProjectFile` for the new format
    - Change `ProjectFile` to contain `maps: HashMap<MapId, MapData>` and `tilesets: HashMap<TilesetId, TilesetMeta>`
    - Update `serialize` and `deserialize` methods
    - Add validation in `deserialize`: validate each map, check `TileRef` tileset IDs exist
    - _Requirements: 10.1, 10.2, 10.4_

  - [x] 4.2 Update `SerializationPlugin` to work with `Res<Project>`
    - Save: extract `ProjectFile` from `Project` (maps + tileset metas only)
    - Load: reconstruct `Project` from `ProjectFile` by re-loading tileset images via asset server, initializing empty undo histories and tab state
    - _Requirements: 10.1, 10.2_

  - [ ]* 4.3 Write property tests for serialization
    - **Property 17: Serialization round-trip**
    - **Property 18: Invalid JSON returns a descriptive error**
    - **Validates: Requirements 10.1, 10.2, 10.3, 10.4**

- [ ] 5. Milestone 1 checkpoint — core data model compiles and serialization round-trips
  - Ensure `cargo build` succeeds and all tests pass. Ask the user to review before proceeding.

### Milestone 2: Tileset Tab Management

- [x] 6. Implement Tileset Tab Bar in Tile Palette
  - [x] 6.1 Update `TilePalettePlugin` to render Tileset Tab Bar
    - Show one tab per tileset in `project.tilesets`
    - Click tab → set `editor_state.active_tileset_tab` and display that tileset's tile grid
    - _Requirements: 7.1, 7.2_

  - [x] 6.2 Update tile selection to produce `TileRef`
    - When user clicks a tile, set `editor_state.active_brush` to `TileRef { tileset_id: active_tileset_tab, col, row }`
    - _Requirements: 7.3_

  - [x] 6.3 Auto-switch to newly loaded tileset tab
    - When a new tileset is added, set `active_tileset_tab` to its ID
    - _Requirements: 7.4_

- [x] 7. Update "Load Tileset" in `AppShellPlugin` to add tileset to `Project`
  - [x] 7.1 Update "Load Tileset" dialog
    - On file load: call `project.add_tileset(meta, texture, layout)` instead of `commands.insert_resource(TilesetData {...})`
    - Set `editor_state.active_tileset_tab` to the new tileset's ID
    - _Requirements: 3.2, 7.4_

- [x] 8. Milestone 2 checkpoint — tileset tabs visible and functional
  - Ensure `cargo build` succeeds and all tests pass. Ask the user to review the tileset tab UI before proceeding.

### Milestone 3: Multi-Map Management

- [x] 9. Update `AppShellPlugin` — New Map dialog changes
  - [x] 9.1 Update "New Map" dialog to add map to `Project`
    - Add tile size picker (tile_width, tile_height) to the New Map dialog
    - On confirm: call `project.add_map(name, w, h, tile_w, tile_h)` instead of `commands.insert_resource(MapData::new(...))`
    - Auto-open the new map in the tab bar and set as active
    - _Requirements: 1.2, 2.1, 11.1, 11.2_

  - [x] 9.2 Update unsaved changes tracking to use per-map `Project.has_unsaved_changes`
    - Check `project.has_unsaved_changes` for the active map when prompting
    - _Requirements: 5.6_

- [x] 10. Implement Map Tab Bar UI
  - [x] 10.1 Add `MapTabBar` UI rendering in `AppShellPlugin` or a new `map_tab_bar.rs` plugin
    - Render a horizontal tab strip above the canvas using `egui::TopBottomPanel`
    - One tab per entry in `project.open_tabs`, showing map name
    - Display modified indicator (●) when `project.has_unsaved_changes[map_id]` is true
    - Click tab → `project.set_active_tab(idx)`
    - Close button (×) on each tab → `project.close_map_tab(idx)`
    - _Requirements: 5.1, 5.2, 5.4, 5.6_

  - [x] 10.2 Handle empty tab state
    - When no tabs are open (`active_tab == None`), show "No map open" on the canvas
    - When closing the active tab, activate nearest remaining tab per `close_map_tab` logic
    - _Requirements: 5.5_

- [x] 11. Implement Map Browser panel
  - [x] 11.1 Create Map Browser panel UI
    - Add a panel (left side, above or integrated with layer panel) listing all maps in `project.maps` by name
    - Double-click entry → `project.open_map_tab(id)`
    - _Requirements: 6.1, 6.2_

  - [x] 11.2 Add right-click context menu with Open, Rename, Delete actions
    - "Open" → `project.open_map_tab(id)`
    - "Rename" → inline text edit, update `project.maps[id].name`
    - "Delete" → confirmation dialog, then `project.remove_map(id)`
    - _Requirements: 6.3, 6.4, 6.5, 1.3, 1.4_

  - [ ]* 11.3 Write property test for map rename
    - **Property 13: Renaming a map updates its name**
    - **Validates: Requirements 6.4**

- [x] 12. Update Canvas and Render systems for active map
  - [x] 12.1 Update `CanvasPlugin` to read from `Project.active_map()`
    - Grid overlay uses `active_map.tile_width`/`tile_height` and `width`/`height`
    - Zoom-to-fit uses the active map's dimensions
    - When no active map, show empty canvas
    - _Requirements: 2.3, 8.1, 8.2_

  - [x] 12.2 Update `sync_tile_sprites` to resolve `TileRef` to correct tileset
    - For each `TileRef` in the active map's visible layers, look up `project.tilesets[tile_ref.tileset_id]`
    - Compute atlas index as `tile_ref.row * tileset.meta.columns + tile_ref.col`
    - Skip tiles whose `tileset_id` is not found in the project (log warning)
    - Despawn/respawn sprites when active map changes
    - _Requirements: 4.3, 8.1, 8.2, 8.3_

  - [ ]* 12.3 Write property tests for render resolution
    - **Property 8: TileRefs with missing tileset IDs produce no sprites**
    - **Property 14: Only the active map's tiles are included in render output**
    - **Property 15: TileRef atlas index resolution**
    - **Validates: Requirements 4.3, 8.1, 8.2, 8.3**

- [x] 13. Update Painting plugin for `TileRef` and tileset compatibility
  - [x] 13.1 Update `PaintingPlugin` to work with `Res<Project>`
    - Read active map from `project.active_map_mut()` instead of `Res<MapData>`
    - Before placing a tile, validate tileset compatibility (tile size match) per Property 6
    - Write `TileRef` into the layer grid instead of `TileIndex`
    - _Requirements: 2.2, 4.1, 4.2_

  - [x] 13.2 Update `EditCommand` emission to use per-map undo history
    - Write `EditCommand` messages scoped to the active map
    - _Requirements: 9.1, 9.2_

- [x] 14. Update Undo/Redo plugin for per-map history
  - [x] 14.1 Update `UndoRedoPlugin` to use `Project.undo_histories`
    - `consume_edit_commands`: push to `project.active_undo_history_mut()`
    - `undo_redo_keyboard`: undo/redo on the active map's history only
    - Mark `project.has_unsaved_changes[active_map_id]` on edits
    - _Requirements: 9.1, 9.2, 9.3_

  - [ ]* 14.2 Write property test for per-map undo isolation
    - **Property 16: Per-map undo isolation**
    - **Validates: Requirements 9.1, 9.2, 9.3**

- [ ] 15. Milestone 3 checkpoint — multi-map UI functional, painting and undo work per-map
  - Ensure `cargo build` succeeds and all tests pass. Ask the user to review map tabs, browser, and painting before proceeding.

### Milestone 4: Fit and Finish

- [x] 16. Update Layer Panel to read from `Project`
  - [x] 16.1 Update `LayerPanelPlugin` to use `Project.active_map()` / `active_map_mut()`
    - Replace `Option<ResMut<MapData>>` with `ResMut<Project>` and access active map
    - All layer operations (add, delete, toggle visibility, select) operate on the active map
    - _Requirements: 8.1_

- [x] 17. Wire everything together in `main.rs`
  - [x] 17.1 Update `main.rs` plugin registration
    - Replace `init_resource::<EditorState>` with `init_resource::<Project>` (or insert default `Project`)
    - Remove old singleton resource insertions (`MapData`, `TilesetData`, `UndoHistory`)
    - Ensure all updated plugins are registered
    - _Requirements: 1.1, 3.1_

  - [x] 17.2 Remove deprecated `TileIndex` and old `TilesetData` resource
    - Delete `TileIndex` struct from `src/data/map.rs`
    - Remove old `TilesetData` resource from `src/data/tileset.rs` (replaced by `TilesetEntry` inside `Project`)
    - Clean up any remaining references
    - _Requirements: 4.1_

- [x] 18. Implement full project management flow in `SerializationPlugin`
  - [x] 18.1 Add `SaveAs` and `NewProject` variants to `SerializationRequest`
    - `SaveAs` always opens a file dialog, updates `current_save_path`
    - `NewProject` creates a fresh `Project` with one default map, clears `current_save_path`. Prompts to save if any map has unsaved changes.
    - _Requirements: 10.1_

  - [x] 18.2 Wire "New Project", "Save As" menu items in `AppShellPlugin`
    - Add "New Project" and "Save As" entries to the File menu
    - Emit the corresponding `SerializationRequest` variants
    - Update "Open" label to "Open Project" for clarity
    - _Requirements: 10.1, 10.2_

  - [x] 18.3 Add unsaved-changes prompt before destructive actions
    - Before "New Project" and "Open Project", check if any map in `project.has_unsaved_changes` is true
    - If so, show a confirmation dialog ("Save changes?") with Save / Discard / Cancel options
    - _Requirements: 5.6, 10.1_

  - [x] 18.4 Remove `Option` wrapper from `Project` parameter in serialization system
    - Once task 17.1 ensures `Project` is always initialized, change `Option<ResMut<Project>>` back to `ResMut<Project>`
    - _Requirements: 1.1_

- [ ] 19. Final checkpoint — full compilation and all tests pass
  - Ensure `cargo build` succeeds and all tests pass. Ask the user for final review.

## Notes

- Tasks marked with `*` are optional property tests that can be skipped for faster MVP
- Each task references specific requirements for traceability
- Milestone checkpoints are review gates — pause for user feedback before continuing
- The `uuid` crate (or `Uuid::new_v4().to_string()`) will be needed — add to `Cargo.toml` dependencies
