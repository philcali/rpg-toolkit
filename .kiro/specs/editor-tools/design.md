# Design Document

## Overview

This design adds a tool mode system and floating toolbar to the RPG map editor, along with flood fill, stamp brush, erase, and line painting tools. The architecture introduces an `EditorTool` enum resource for mode tracking, a floating egui toolbar overlay, two pure algorithm modules (flood fill and line engine), and modifications to the existing painting and pan systems to gate behavior on the active tool.

## Architecture

### New Data Types

#### `EditorTool` enum (in `src/data/editor_state.rs`)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Resource)]
pub enum EditorTool {
    #[default]
    Paint,
    Erase,
    Pan,
    FloodFill,
    StampBrush,
}
```

Added as a Bevy `Resource` with `Default` deriving to `Paint`. Registered in `CanvasPlugin` or `AppShellPlugin` via `init_resource::<EditorTool>()`.

#### `StampBrushSelection` (in `src/data/editor_state.rs`)

```rust
#[derive(Clone, Debug)]
pub struct StampBrushSelection {
    pub tileset_id: TilesetId,
    pub top_left_col: u32,
    pub top_left_row: u32,
    pub width: u32,   // in tiles
    pub height: u32,  // in tiles
}
```

Stored as `Option<StampBrushSelection>` on `EditorState`.

#### `LineDragState` (in `src/data/editor_state.rs`)

```rust
#[derive(Clone, Debug, Default)]
pub struct LineDragState {
    pub active: bool,
    pub start_tile: Option<(u32, u32)>,
}
```

Stored on `EditorState` to track Ctrl+drag line operations.

### New Modules

#### `src/algorithms/flood_fill.rs`

Pure function module with no Bevy dependencies:

```rust
pub fn flood_fill(
    grid: &[Vec<Option<TileRef>>],
    start: (u32, u32),
    target: &Option<TileRef>,
    replacement: &TileRef,
) -> Vec<(u32, u32)>
```

Uses a BFS queue with four-directional adjacency. Returns coordinates to fill. Returns empty vec if start is out of bounds or target equals replacement.

#### `src/algorithms/line_engine.rs`

Pure function module with no Bevy dependencies:

```rust
pub fn bresenham_line(x0: u32, y0: u32, x1: u32, y1: u32) -> Vec<(u32, u32)>
```

Implements Bresenham's line algorithm using `i64` intermediates to handle unsigned coordinates. Returns ordered list from start to end, inclusive.

#### `src/algorithms/mod.rs`

```rust
pub mod flood_fill;
pub mod line_engine;
```

#### `src/plugins/toolbar.rs`

New plugin that renders the floating toolbar as an `egui::Window`:

```rust
pub struct ToolbarPlugin;
```

Registers the `EditorTool` resource and adds the `toolbar_ui` system in `EguiPrimaryContextPass`.

The `toolbar_ui` system:
- Creates an `egui::Window` with `.title_bar(false)`, `.resizable(false)`, `.movable(false)`, `.anchor(egui::Align2::LEFT_TOP, [layer_panel_width, menu_bar_height])` to position it at the top-left of the canvas area (offset past the left side panel and menu bar).
- Renders buttons vertically using `ui.vertical()`.
- Each button displays a Unicode icon: `"✏"` (Paint), `"⌫"` (Erase), `"🪣"` (Flood Fill), `"✋"` (Pan), `"⊞"` (Stamp Brush).
- The active tool button uses `ui.selectable_label(is_active, icon)` for visual highlighting.
- Clicking a button writes the corresponding `EditorTool` variant to the resource.

### Modified Systems

#### `src/plugins/painting.rs`

The `painting_system` gains a `Res<EditorTool>` parameter and branches on the active tool:

- **Paint mode**: Left-click places tiles (existing behavior). Right-click is ignored. Ctrl+left-click starts a line drag; on release, computes line via `bresenham_line` and places tiles along it.
- **Erase mode**: Left-click erases tiles. Left-click-drag erases continuously. Ctrl+left-click starts a line drag; on release, erases along the line.
- **FloodFill mode**: Left-click triggers `flood_fill()` on the active layer, then emits `EditCommand` for each coordinate.
- **StampBrush mode**: Left-click places the full stamp grid from `StampBrushSelection`, skipping out-of-bounds tiles.
- **Pan mode**: Painting system does nothing (early return).

The system also reads `Res<ButtonInput<KeyCode>>` to detect Ctrl state for line operations, and reads/writes `LineDragState` on `EditorState`.

#### `src/systems/camera.rs` — `pan_system`

Gains a `Res<EditorTool>` parameter. When `EditorTool::Pan`, left-mouse-button drag triggers panning (reusing the existing middle-mouse logic). Middle-mouse panning remains unconditional.

#### `src/plugins/tile_palette.rs`

Extended to support click-and-drag selection for stamp brushes. On mouse-down, records the start tile; on mouse-up, computes the rectangular region and stores it as `StampBrushSelection` on `EditorState`. Single-click still sets `active_brush` as before.

#### `src/plugins/canvas.rs`

Extended to draw:
- **Line preview**: When `LineDragState.active`, compute `bresenham_line` from start to current cursor tile and draw semi-transparent highlight rectangles via gizmos.
- **Stamp preview**: When `EditorTool::StampBrush` and a `StampBrushSelection` exists, draw the stamp footprint as semi-transparent rectangles at the cursor position via gizmos.

### Plugin Registration

In `src/main.rs`, add:
```rust
.add_plugins(ToolbarPlugin)
```

And add `src/algorithms` module declaration in `src/main.rs`:
```rust
mod algorithms;
```

### System Ordering

The toolbar UI runs in `EguiPrimaryContextPass` (same as other egui panels). The painting system and pan system read `EditorTool` during `Update`, which is after egui has processed input for the frame.

## Correctness Properties

### Property 1: Flood Fill Completeness
- **Requirement**: 7.2
- **Description**: After applying the flood fill result to the grid, no tile that is 4-directionally adjacent to a filled tile should still contain the original target value (within the connected component of the start position).
- **Test approach**: Property-based test using `proptest`. Generate random small grids (up to 16×16) with a small set of tile values, pick a random start position, run `flood_fill`, apply the result, and verify the adjacency invariant.

### Property 2: Flood Fill Correctness
- **Requirement**: 7.3
- **Description**: Every coordinate returned by `flood_fill` must have contained the target value in the original (unmodified) grid.
- **Test approach**: Property-based test. Generate random grids and start positions, run `flood_fill`, and verify each returned coordinate had the target value in the original grid.

### Property 3: Flood Fill Bounds Safety
- **Requirement**: 5.3, 7.4
- **Description**: All coordinates returned by `flood_fill` are within the grid bounds. Out-of-bounds start positions return an empty list.
- **Test approach**: Property-based test. Generate grids and arbitrary start positions (including out-of-bounds), verify all returned coordinates are in-bounds.

### Property 4: Line Engine Endpoints
- **Requirement**: 10.3
- **Description**: For all valid start and end coordinates, the first element of the returned list is the start coordinate and the last element is the end coordinate.
- **Test approach**: Property-based test with `proptest`. Generate random coordinate pairs (within a reasonable range, e.g., 0..256), run `bresenham_line`, and assert first == start, last == end.

### Property 5: Line Engine Adjacency
- **Requirement**: 10.4
- **Description**: For all valid start and end coordinates, consecutive coordinates in the returned list differ by at most 1 in each axis.
- **Test approach**: Property-based test. Generate random coordinate pairs, run `bresenham_line`, and verify that for each consecutive pair `(a, b)`, `|a.0 - b.0| <= 1` and `|a.1 - b.1| <= 1`.

### Property 6: Line Engine Single Point
- **Requirement**: 10.5
- **Description**: When start equals end, the line engine returns exactly one coordinate equal to the start.
- **Test approach**: Property-based test. Generate random single coordinates, run `bresenham_line(x, y, x, y)`, assert result length is 1 and result[0] == (x, y).

### Property 7: Flood Fill Idempotence
- **Requirement**: 7.2, 7.3
- **Description**: Running flood fill a second time on the already-filled grid (with the same start and replacement) should return an empty list, since the target value no longer exists at the start position.
- **Test approach**: Property-based test. Generate grid, run flood fill, apply results, run flood fill again with same parameters, assert empty result.

### Property 8: Tool Mode Exclusivity
- **Requirement**: 1.1
- **Description**: The `EditorTool` enum always represents exactly one mode. For any value of `EditorTool`, matching against all five variants is exhaustive.
- **Test approach**: This is enforced by Rust's type system (enum exhaustiveness). Verified at compile time. No runtime test needed, but we can add a property test that round-trips through all variants to confirm Default is Paint.

## File Changes

### New Files
| File | Purpose |
|------|---------|
| `src/algorithms/mod.rs` | Module declaration for algorithm crate |
| `src/algorithms/flood_fill.rs` | Pure flood fill function (BFS, 4-directional) |
| `src/algorithms/line_engine.rs` | Pure Bresenham's line algorithm |
| `src/plugins/toolbar.rs` | Floating toolbar egui::Window plugin |

### Modified Files
| File | Changes |
|------|---------|
| `src/main.rs` | Add `mod algorithms`, register `ToolbarPlugin` |
| `src/data/editor_state.rs` | Add `EditorTool` enum, `StampBrushSelection`, `LineDragState` to `EditorState` |
| `src/data/mod.rs` | Re-export new types |
| `src/plugins/mod.rs` | Add `pub mod toolbar` and re-export `ToolbarPlugin` |
| `src/plugins/painting.rs` | Gate on `EditorTool`, add flood fill / stamp / erase / line logic |
| `src/systems/camera.rs` | Gate left-click pan on `EditorTool::Pan` |
| `src/plugins/tile_palette.rs` | Add click-drag for stamp brush selection |
| `src/plugins/canvas.rs` | Add line preview and stamp preview gizmo rendering |

### New Test Files
| File | Purpose |
|------|---------|
| `tests/properties/flood_fill_props.rs` | Property-based tests for flood fill (completeness, correctness, bounds, idempotence) |
| `tests/properties/line_engine_props.rs` | Property-based tests for line engine (endpoints, adjacency, single point) |
