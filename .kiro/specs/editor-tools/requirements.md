# Requirements Document

## Introduction

The RPG map editor currently supports single-tile painting via left-click and erasing via right-click, with camera panning limited to middle-mouse-button drag. This feature introduces a toolbar panel for switching between editor tool modes (Paint, Pan, Flood Fill, Stamp Brush, Erase) so that all core editing operations are accessible without a three-button mouse, and adds new tools: flood fill, multi-tile stamp brush, a dedicated erase tool, and line painting via Ctrl+click-drag.

## Glossary

- **Toolbar**: An egui panel displaying selectable tool buttons that controls which Editor_Tool is active.
- **Editor_Tool**: An enum resource representing the currently selected tool mode (Paint, Pan, Flood_Fill, Stamp_Brush, Erase).
- **Canvas**: The central 2D viewport where the tile map is rendered and edited.
- **Painting_System**: The Bevy system that processes mouse input on the Canvas to place or erase tiles.
- **Pan_System**: The Bevy system that translates mouse drag input into camera offset changes.
- **Flood_Fill_Engine**: A pure function that computes the set of tile coordinates to fill given a starting position, a target tile value, and the current layer grid.
- **Line_Engine**: A pure function that computes the ordered list of tile coordinates along a straight line between two points using Bresenham's line algorithm.
- **Stamp_Brush**: A rectangular multi-tile selection from the tile palette used as a compound brush for painting groups of tiles (e.g., trees, houses).
- **Tile_Palette**: The side panel where users select individual tiles or rectangular tile regions from loaded tilesets.
- **Active_Layer**: The layer currently selected for editing in the layer panel.
- **TileRef**: A struct identifying a specific tile within a specific tileset by tileset ID, column, and row.
- **EditorState**: The Bevy resource holding global editor state including the active brush, zoom level, and camera offset.
- **Line_Preview**: A transient visual overlay on the Canvas showing the projected line of tiles from the drag start to the current cursor position during a Ctrl+drag operation.

## Requirements

### Requirement 1: Tool Mode State

**User Story:** As a map editor user, I want the editor to track which tool is currently active, so that mouse input on the canvas is routed to the correct behavior.

#### Acceptance Criteria

1. THE Editor_Tool resource SHALL represent exactly one of the following modes: Paint, Pan, Flood_Fill, Stamp_Brush, Erase.
2. WHEN the editor starts, THE Editor_Tool resource SHALL default to Paint mode.
3. WHEN the user selects a tool from the Toolbar, THE Editor_Tool resource SHALL update to the selected mode.

### Requirement 2: Toolbar Panel

**User Story:** As a laptop user, I want a visible toolbar with buttons for each tool mode, so that I can switch tools without relying on a middle mouse button.

#### Acceptance Criteria

1. THE Toolbar SHALL display one button for each Editor_Tool mode: Paint, Pan, Flood_Fill, Stamp_Brush, Erase.
2. THE Toolbar SHALL visually highlight the currently active tool button.
3. WHEN the user clicks a tool button, THE Toolbar SHALL set the Editor_Tool resource to the corresponding mode.
4. THE Toolbar SHALL be rendered as a floating egui::Window with no title bar, non-resizable, and fixed anchor position overlaying the top-left corner of the Canvas area, without consuming Canvas layout space and without overlapping the Tile_Palette or layer panel.
5. THE Toolbar SHALL display a Unicode text icon label on each tool button: "✏" (pencil) for Paint, "⌫" (erase symbol) for Erase, "🪣" (paint bucket) for Flood_Fill, "✋" (open hand) for Pan, and "⊞" (grid/stamp symbol) for Stamp_Brush, rendered as egui button text since the editor does not load custom icon assets.
6. THE Toolbar SHALL use a vertical strip layout with buttons stacked vertically.
7. THE Toolbar SHALL remain at a fixed position and SHALL NOT be draggable by the user.

### Requirement 3: Paint Tool Mode

**User Story:** As a map editor user, I want the existing single-tile paint behavior to work only when Paint mode is active, so that clicking on the canvas does not accidentally paint while I am using another tool.

#### Acceptance Criteria

1. WHILE the Editor_Tool is set to Paint mode, WHEN the user left-clicks a tile on the Canvas, THE Painting_System SHALL place the active brush TileRef on the Active_Layer at the clicked position.
2. WHILE the Editor_Tool is set to Paint mode, THE Painting_System SHALL ignore right-click input on the Canvas.
3. WHILE the Editor_Tool is set to a mode other than Paint, THE Painting_System SHALL ignore left-click and right-click input on the Canvas for tile placement.

### Requirement 4: Pan Tool Mode

**User Story:** As a laptop user, I want a Pan tool mode that lets me drag the canvas with left-click, so that I can pan without a middle mouse button.

#### Acceptance Criteria

1. WHILE the Editor_Tool is set to Pan mode, THE Pan_System SHALL initiate camera panning when the user presses and drags with the left mouse button on the Canvas.
2. WHILE the Editor_Tool is set to Pan mode, THE Pan_System SHALL update the camera offset based on the drag delta, matching the existing middle-mouse-button pan behavior.
3. WHILE the Editor_Tool is set to Pan mode, THE Painting_System SHALL not place or erase tiles on left-click.
4. THE Pan_System SHALL continue to support middle-mouse-button panning regardless of the active Editor_Tool mode.

### Requirement 5: Flood Fill Tool

**User Story:** As a map editor user, I want a flood fill tool that fills a contiguous region of matching tiles with the active brush, so that I can quickly paint large uniform areas.

#### Acceptance Criteria

1. WHILE the Editor_Tool is set to Flood_Fill mode, WHEN the user left-clicks a tile on the Canvas, THE Flood_Fill_Engine SHALL compute all contiguous tiles on the Active_Layer that match the clicked tile's value (including empty matching empty).
2. THE Flood_Fill_Engine SHALL use four-directional adjacency (up, down, left, right) to determine contiguity.
3. THE Flood_Fill_Engine SHALL not fill tiles outside the map boundaries.
4. WHEN the Flood_Fill_Engine completes, THE Painting_System SHALL place the active brush TileRef on every tile coordinate in the computed fill set.
5. IF the active brush is not set, THEN THE Painting_System SHALL not perform the flood fill operation.
6. IF the clicked tile already contains the same TileRef as the active brush, THEN THE Flood_Fill_Engine SHALL produce an empty fill set and no tiles SHALL be modified.
7. THE Flood_Fill_Engine SHALL accept a layer grid, a start position, a target tile value, and a replacement TileRef as inputs and return a list of coordinates to fill, with no dependency on Bevy ECS types.
8. WHEN a flood fill operation modifies tiles, THE Painting_System SHALL emit EditCommand messages for each modified tile so that the undo/redo system can reverse the entire fill.

### Requirement 6: Multi-Tile Stamp Brush Selection

**User Story:** As a map editor user, I want to select a rectangular region of tiles from the tile palette to use as a stamp brush, so that I can paint multi-tile structures like trees and houses in a single stroke.

#### Acceptance Criteria

1. WHEN the user clicks and drags across multiple tiles in the Tile_Palette, THE Tile_Palette SHALL capture the rectangular selection as a Stamp_Brush definition containing the tileset ID, the top-left column and row, and the width and height in tiles.
2. WHILE a Stamp_Brush is active and the Editor_Tool is set to Stamp_Brush mode, WHEN the user left-clicks a tile on the Canvas, THE Painting_System SHALL place the full rectangular grid of TileRefs from the Stamp_Brush onto the Active_Layer, anchored at the clicked tile position.
3. THE Painting_System SHALL skip any stamp tile placement that falls outside the map boundaries.
4. WHEN a stamp brush operation modifies tiles, THE Painting_System SHALL emit an EditCommand for each modified tile so that the undo/redo system can reverse the entire stamp.
5. IF no Stamp_Brush selection has been made, THEN THE Painting_System SHALL not perform any stamp operation on left-click in Stamp_Brush mode.
6. WHILE a Stamp_Brush is active and the Editor_Tool is set to Stamp_Brush mode, THE Canvas SHALL display a preview overlay of the stamp brush footprint at the cursor position before the user clicks.

### Requirement 7: Flood Fill Engine Purity

**User Story:** As a developer, I want the flood fill algorithm to be a pure function independent of Bevy, so that I can write property-based tests for correctness.

#### Acceptance Criteria

1. THE Flood_Fill_Engine SHALL be implemented as a pure function that takes a 2D grid of `Option<TileRef>`, a start coordinate (x, y), a target value `Option<TileRef>`, and a replacement `TileRef`, and returns a `Vec<(u32, u32)>` of coordinates to fill.
2. FOR ALL valid grids and start positions, calling the Flood_Fill_Engine and then applying the returned coordinates SHALL produce a grid where no tile adjacent to a filled tile has the original target value (fill completeness).
3. FOR ALL valid grids and start positions, the Flood_Fill_Engine SHALL return only coordinates that contained the target value in the original grid (fill correctness).
4. WHEN the start coordinate is outside the grid bounds, THE Flood_Fill_Engine SHALL return an empty list.

### Requirement 8: Erase Tool Mode

**User Story:** As a map editor user, I want a dedicated Erase tool mode, so that I can remove tiles by left-clicking without needing to right-click or switch context from the Paint tool.

#### Acceptance Criteria

1. WHILE the Editor_Tool is set to Erase mode, WHEN the user left-clicks a tile on the Canvas, THE Painting_System SHALL erase the tile at the clicked position on the Active_Layer by setting the cell to empty.
2. WHILE the Editor_Tool is set to Erase mode, THE Painting_System SHALL ignore right-click input on the Canvas.
3. WHILE the Editor_Tool is set to Erase mode, WHEN the user left-click-drags across multiple tiles, THE Painting_System SHALL erase each tile the cursor passes over on the Active_Layer.
4. WHEN an erase operation modifies a tile, THE Painting_System SHALL emit an EditCommand for the modified tile so that the undo/redo system can reverse the erasure.
5. IF the tile at the clicked position is already empty, THEN THE Painting_System SHALL not emit an EditCommand for that position.

### Requirement 9: Line Painting (Ctrl+Left-Click Drag)

**User Story:** As a map editor user, I want to hold Ctrl and left-click-drag to draw a straight line of tiles between two points, so that I can quickly create walls, paths, and borders without painting tile by tile.

#### Acceptance Criteria

1. WHILE the Editor_Tool is set to Paint mode, WHEN the user holds Ctrl and presses the left mouse button on a tile, THE Painting_System SHALL record that tile position as the line start point.
2. WHILE the Editor_Tool is set to Paint mode and a Ctrl+left-click drag is in progress, THE Canvas SHALL display a Line_Preview showing the projected line of tiles from the start point to the current cursor tile position.
3. WHILE the Editor_Tool is set to Paint mode, WHEN the user releases the left mouse button after a Ctrl+drag, THE Painting_System SHALL compute the line coordinates using the Line_Engine and place the active brush TileRef on every tile along the line on the Active_Layer.
4. WHILE the Editor_Tool is set to Erase mode, WHEN the user holds Ctrl and presses the left mouse button on a tile, THE Painting_System SHALL record that tile position as the line start point.
5. WHILE the Editor_Tool is set to Erase mode and a Ctrl+left-click drag is in progress, THE Canvas SHALL display a Line_Preview showing the projected line of tiles from the start point to the current cursor tile position.
6. WHILE the Editor_Tool is set to Erase mode, WHEN the user releases the left mouse button after a Ctrl+drag, THE Painting_System SHALL compute the line coordinates using the Line_Engine and erase every tile along the line on the Active_Layer.
7. WHEN a line painting or line erase operation modifies tiles, THE Painting_System SHALL emit an EditCommand for each modified tile so that the undo/redo system can reverse the entire line operation.
8. IF the active brush is not set during a line paint operation in Paint mode, THEN THE Painting_System SHALL not commit any tiles on mouse release.
9. IF the user releases Ctrl before releasing the mouse button, THE Painting_System SHALL cancel the line operation and remove the Line_Preview without committing any tiles.

### Requirement 10: Line Engine Purity

**User Story:** As a developer, I want the line computation algorithm to be a pure function independent of Bevy, so that I can write property-based tests for correctness.

#### Acceptance Criteria

1. THE Line_Engine SHALL be implemented as a pure function that takes a start coordinate (x0, y0) and an end coordinate (x1, y1) and returns a `Vec<(u32, u32)>` of tile coordinates along the line.
2. THE Line_Engine SHALL use Bresenham's line algorithm to compute the tile coordinates.
3. FOR ALL valid start and end coordinates, the first element of the returned list SHALL be the start coordinate and the last element SHALL be the end coordinate.
4. FOR ALL valid start and end coordinates, consecutive coordinates in the returned list SHALL differ by at most 1 in each axis (adjacency property).
5. WHEN the start coordinate equals the end coordinate, THE Line_Engine SHALL return a list containing exactly that single coordinate.
