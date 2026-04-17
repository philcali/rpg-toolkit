# Implementation Plan: Editor Tools

## Overview

Implement a tool mode system with floating toolbar, flood fill, stamp brush, erase, and line painting tools for the RPG map editor. The approach starts with new data types and pure algorithm modules, then builds the toolbar UI, modifies existing systems to gate on the active tool, and wires everything together with preview gizmos.

## Tasks

- [x] 1. Add new data types to editor state
  - [x] 1.1 Add `EditorTool` enum, `StampBrushSelection`, and `LineDragState` to `src/data/editor_state.rs`
    - Add `EditorTool` enum with `Paint` (default), `Erase`, `Pan`, `FloodFill`, `StampBrush` variants, deriving `Resource`, `Default`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Debug`
    - Add `StampBrushSelection` struct with `tileset_id`, `top_left_col`, `top_left_row`, `width`, `height` fields
    - Add `LineDragState` struct with `active: bool` and `start_tile: Option<(u32, u32)>`, deriving `Default`
    - Add `stamp_brush: Option<StampBrushSelection>` and `line_drag: LineDragState` fields to `EditorState`
    - _Requirements: 1.1, 1.2, 6.1, 9.1_
  - [x] 1.2 Update `src/data/mod.rs` to re-export new types
    - Re-export `EditorTool`, `StampBrushSelection`, `LineDragState`
    - _Requirements: 1.1_

- [x] 2. Implement pure algorithm modules
  - [x] 2.1 Create `src/algorithms/mod.rs` with module declarations
    - Declare `pub mod flood_fill;` and `pub mod line_engine;`
    - _Requirements: 7.1, 10.1_
  - [x] 2.2 Implement flood fill in `src/algorithms/flood_fill.rs`
    - Implement `pub fn flood_fill(grid: &[Vec<Option<TileRef>>], start: (u32, u32), target: &Option<TileRef>, replacement: &TileRef) -> Vec<(u32, u32)>`
    - Use BFS with 4-directional adjacency (up, down, left, right)
    - Return empty vec if start is out of bounds or target equals `Some(replacement.clone())`
    - No Bevy dependencies — only use `TileRef` from `crate::data::map`
    - _Requirements: 5.1, 5.2, 5.3, 5.6, 7.1, 7.2, 7.3, 7.4_
  - [ ]* 2.3 Write property tests for flood fill in `tests/properties/flood_fill_props.rs`
    - **Property 1: Flood Fill Completeness** — After applying fill results, no tile adjacent to a filled tile still has the original target value within the connected component
    - **Validates: Requirements 7.2**
    - **Property 2: Flood Fill Correctness** — Every returned coordinate had the target value in the original grid
    - **Validates: Requirements 7.3**
    - **Property 3: Flood Fill Bounds Safety** — All returned coordinates are within grid bounds; out-of-bounds start returns empty
    - **Validates: Requirements 5.3, 7.4**
    - **Property 7: Flood Fill Idempotence** — Running flood fill a second time on the filled grid returns empty
    - **Validates: Requirements 7.2, 7.3**
  - [x] 2.4 Implement Bresenham's line algorithm in `src/algorithms/line_engine.rs`
    - Implement `pub fn bresenham_line(x0: u32, y0: u32, x1: u32, y1: u32) -> Vec<(u32, u32)>`
    - Use `i64` intermediates for unsigned coordinate math
    - Return ordered list from start to end, inclusive
    - No Bevy dependencies
    - _Requirements: 10.1, 10.2, 10.3, 10.4, 10.5_
  - [ ]* 2.5 Write property tests for line engine in `tests/properties/line_engine_props.rs`
    - **Property 4: Line Engine Endpoints** — First element is start, last element is end
    - **Validates: Requirements 10.3**
    - **Property 5: Line Engine Adjacency** — Consecutive coordinates differ by at most 1 in each axis
    - **Validates: Requirements 10.4**
    - **Property 6: Line Engine Single Point** — When start equals end, returns exactly one coordinate
    - **Validates: Requirements 10.5**
  - [x] 2.6 Add `mod algorithms;` declaration in `src/main.rs`
    - _Requirements: 7.1, 10.1_

- [x] 3. Implement toolbar plugin
  - [x] 3.1 Create `src/plugins/toolbar.rs` with `ToolbarPlugin`
    - Implement `ToolbarPlugin` that registers `EditorTool` resource via `init_resource::<EditorTool>()` and adds `toolbar_ui` system in `EguiPrimaryContextPass`
    - Render floating `egui::Window` with `.title_bar(false)`, `.resizable(false)`, `.movable(false)`, `.anchor(egui::Align2::LEFT_TOP, offset)` positioned at top-left of canvas area
    - Display vertical strip of buttons with Unicode icons: `"✏"` (Paint), `"⌫"` (Erase), `"🪣"` (Flood Fill), `"✋"` (Pan), `"⊞"` (Stamp Brush)
    - Use `ui.selectable_label(is_active, icon)` for visual highlighting of active tool
    - Clicking a button writes the corresponding `EditorTool` variant to the resource
    - _Requirements: 1.3, 2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 2.7_
  - [x] 3.2 Register toolbar in `src/plugins/mod.rs` and `src/main.rs`
    - Add `pub mod toolbar;` and `pub use toolbar::ToolbarPlugin;` to `src/plugins/mod.rs`
    - Add `.add_plugins(ToolbarPlugin)` to `src/main.rs`
    - _Requirements: 2.1_

- [ ] 4. Checkpoint — Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [x] 5. Modify painting system for tool modes
  - [x] 5.1 Gate painting system on `EditorTool` in `src/plugins/painting.rs`
    - Add `Res<EditorTool>` and `Res<ButtonInput<KeyCode>>` parameters to `painting_system`
    - Add `ResMut<EditorState>` for `LineDragState` access (already has `Res<EditorState>`, change to `ResMut`)
    - In Paint mode: left-click places tiles (existing behavior), right-click is ignored, Ctrl+left-click starts line drag
    - In Erase mode: left-click erases tiles, left-click-drag erases continuously, Ctrl+left-click starts line erase drag
    - In FloodFill mode: left-click triggers `flood_fill()` on active layer, emit `EditCommand` for each coordinate
    - In StampBrush mode: left-click places full stamp grid from `StampBrushSelection`, skip out-of-bounds tiles, emit `EditCommand` per tile
    - In Pan mode: early return (no painting)
    - On Ctrl+drag release: compute line via `bresenham_line`, place or erase tiles along line, emit `EditCommand` per tile
    - If Ctrl released before mouse button: cancel line operation, clear `LineDragState`
    - _Requirements: 3.1, 3.2, 3.3, 4.3, 5.1, 5.4, 5.5, 5.8, 6.2, 6.3, 6.4, 6.5, 8.1, 8.2, 8.3, 8.4, 8.5, 9.1, 9.2, 9.3, 9.4, 9.5, 9.6, 9.7, 9.8, 9.9_

- [x] 6. Modify camera system for Pan tool
  - [x] 6.1 Extend `pan_system` in `src/systems/camera.rs` to support left-click pan in Pan mode
    - Add `Res<EditorTool>` parameter to `pan_system`
    - When `EditorTool::Pan`, left-mouse-button drag triggers panning using the same delta logic as middle-mouse
    - Middle-mouse panning remains unconditional regardless of active tool
    - _Requirements: 4.1, 4.2, 4.4_

- [x] 7. Modify tile palette for stamp brush selection
  - [x] 7.1 Add click-drag selection to `src/plugins/tile_palette.rs`
    - Track mouse-down start tile position in the tile grid
    - On mouse-up, compute rectangular region from start to current tile
    - Store result as `StampBrushSelection` on `EditorState`
    - Single-click still sets `active_brush` as before
    - _Requirements: 6.1_

- [x] 8. Add canvas preview gizmos
  - [x] 8.1 Add line preview and stamp preview to `src/plugins/canvas.rs`
    - When `LineDragState.active`, compute `bresenham_line` from start to current cursor tile and draw semi-transparent highlight rectangles via gizmos
    - When `EditorTool::StampBrush` and `StampBrushSelection` exists, draw stamp footprint as semi-transparent rectangles at cursor position via gizmos
    - Read `EditorTool`, `EditorState`, and `CursorWorldState` resources
    - _Requirements: 6.6, 9.2, 9.5_

- [ ] 9. Final checkpoint — Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

## Notes

- Tasks marked with `*` are optional and can be skipped for faster MVP
- Each task references specific requirements for traceability
- Checkpoints ensure incremental validation
- Property tests validate universal correctness properties from the design document
- The pure algorithm modules (flood fill, line engine) have no Bevy dependencies, enabling straightforward property-based testing with `proptest`
