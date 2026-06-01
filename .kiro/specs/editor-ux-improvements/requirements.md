# Requirements Document

## Introduction

This spec bundles four related editor UX improvements for the RPG Toolkit Editor (Bevy + egui):

1. **Tile Animations** — Define animated tile sequences in tileset metadata and render them by cycling frames based on elapsed time.
2. **Map Selector Dropdown with Search** — Replace the unbounded list of map pills in the Map Browser with a searchable dropdown/combobox.
3. **Tileset Palette Dropdown with Search** — Replace the unbounded horizontal-wrapped tileset tab bar with a searchable dropdown/combobox.
4. **Tileset Palette Tile Scaling** — Add a zoom/scale control to the tile palette so tiles remain legible regardless of tileset tile size.

## Glossary

- **Editor**: The `rpg-toolkit-editor` Bevy application providing the map editing UI via egui.
- **Tile_Animation**: A sequence of tile indices within a single tileset that cycle at a defined frame duration to produce an animated tile effect (e.g., fire, water).
- **Animation_Frame**: A single tile index within a Tile_Animation sequence.
- **Frame_Duration**: The time in milliseconds that each Animation_Frame is displayed before advancing to the next frame.
- **Tileset_Meta**: The `TilesetMeta` struct in `rpg-toolkit-common` describing a tileset's grid dimensions and file path.
- **Map_Browser**: The "Maps" section in the left side panel that lists all maps in the project.
- **Tileset_Tab_Bar**: The horizontal-wrapped row of selectable labels in the tile palette panel used to switch between loaded tilesets.
- **Tile_Palette**: The right side panel displaying the tile grid for the active tileset.
- **Display_Tile_Size**: The pixel size at which each tile is rendered in the Tile_Palette grid.
- **Renderer**: The `rpg-toolkit-renderer` crate responsible for rendering the game at runtime.
- **Editor_Render_System**: The `sync_tile_sprites` system in the editor that spawns Bevy sprites for the active map.

## Requirements

### Requirement 1: Tile Animation Data Model

**User Story:** As a game designer, I want to define animated tile sequences in tileset metadata, so that I can create tiles that cycle through frames (fire, torches, waterfalls) without manual per-frame placement.

#### Acceptance Criteria

1. THE Tileset_Meta SHALL include an optional list of Tile_Animation definitions, where each definition contains an ordered sequence of tile grid coordinates and a Frame_Duration value in milliseconds.
2. WHEN a Tile_Animation is serialized, THE serialization system SHALL persist the animation sequence and Frame_Duration to the project file in JSON format.
3. WHEN a project file containing Tile_Animation definitions is loaded, THE serialization system SHALL deserialize the animation data and attach it to the corresponding Tileset_Meta.
4. FOR ALL valid Tile_Animation definitions, serializing then deserializing SHALL produce an equivalent Tile_Animation object (round-trip property).
5. THE Tileset_Meta SHALL validate that each tile coordinate in a Tile_Animation sequence references a valid grid position within the tileset bounds (column < columns, row < rows).
6. THE Tileset_Meta SHALL validate that Frame_Duration is a positive integer greater than zero.
7. IF a Tile_Animation contains fewer than two Animation_Frames, THEN THE Tileset_Meta SHALL reject the animation definition with a descriptive error.

### Requirement 2: Tile Animation Editor UI

**User Story:** As a game designer, I want a UI in the tile palette to select tiles into an animation sequence and set the loop frequency, so that I can author animated tiles visually.

#### Acceptance Criteria

1. THE Tile_Palette SHALL provide an "Animation Editor" mode accessible via a toggle button in the palette panel.
2. WHILE the Animation Editor mode is active, THE Tile_Palette SHALL allow the user to click tiles in sequence to build an ordered list of Animation_Frames.
3. WHILE the Animation Editor mode is active, THE Tile_Palette SHALL display the current animation sequence as an ordered list with tile previews.
4. THE Animation Editor SHALL provide a numeric input field for Frame_Duration with a default value of 200 milliseconds.
5. WHEN the user confirms an animation definition, THE Editor SHALL store the Tile_Animation in the active tileset's metadata.
6. THE Animation Editor SHALL allow the user to remove individual frames from the sequence by clicking a remove button next to each frame entry.
7. THE Animation Editor SHALL allow the user to reorder frames in the sequence via move-up and move-down buttons.
8. WHEN the user cancels the animation editor without confirming, THE Editor SHALL discard the in-progress animation sequence.
9. THE Animation Editor SHALL display a live preview that cycles through the defined frames at the specified Frame_Duration.

### Requirement 3: Tile Animation Rendering in Editor

**User Story:** As a game designer, I want animated tiles to play in the editor canvas, so that I can preview how animations look in context while editing the map.

#### Acceptance Criteria

1. WHEN the active map contains tiles that reference a Tile_Animation, THE Editor_Render_System SHALL cycle the displayed sprite atlas index through the animation sequence based on elapsed time.
2. THE Editor_Render_System SHALL advance animation frames at the interval specified by the Tile_Animation's Frame_Duration.
3. WHEN a Tile_Animation reaches its last frame, THE Editor_Render_System SHALL loop back to the first frame.
4. THE Editor_Render_System SHALL synchronize all instances of the same Tile_Animation so they animate in lockstep.

### Requirement 4: Tile Animation Rendering in Game Renderer

**User Story:** As a game designer, I want animated tiles to play at runtime in the game renderer, so that the final game displays the same animations authored in the editor.

#### Acceptance Criteria

1. WHEN the active map contains tiles that reference a Tile_Animation, THE Renderer SHALL cycle the displayed sprite atlas index through the animation sequence based on elapsed time.
2. THE Renderer SHALL advance animation frames at the interval specified by the Tile_Animation's Frame_Duration.
3. WHEN a Tile_Animation reaches its last frame, THE Renderer SHALL loop back to the first frame.
4. THE Renderer SHALL synchronize all instances of the same Tile_Animation so they animate in lockstep.

### Requirement 5: Map Selector Searchable Dropdown

**User Story:** As a game designer, I want to search and select maps from a dropdown instead of scrolling through an unbounded list of pills, so that I can quickly find maps in large projects.

#### Acceptance Criteria

1. THE Map_Browser SHALL display a searchable dropdown (combobox) for map selection instead of a flat scrollable list of selectable labels.
2. WHEN the user types into the search field, THE Map_Browser SHALL filter the displayed map entries to those whose names contain the search text (case-insensitive).
3. WHEN the user selects a map from the dropdown, THE Editor SHALL open that map in a tab and set it as the active map.
4. THE Map_Browser SHALL display the currently active map name as the dropdown's selected value.
5. WHILE the search field is empty, THE Map_Browser SHALL display all maps sorted alphabetically by name.
6. THE Map_Browser SHALL continue to support right-click context menus (Open, Rename, Delete) on each map entry in the dropdown list.

### Requirement 6: Tileset Palette Searchable Dropdown

**User Story:** As a game designer, I want to search and select tilesets from a dropdown instead of scrolling through an unbounded horizontal list of tabs, so that I can quickly switch tilesets in large projects.

#### Acceptance Criteria

1. THE Tileset_Tab_Bar SHALL be replaced with a searchable dropdown (combobox) for tileset selection.
2. WHEN the user types into the search field, THE Tile_Palette SHALL filter the displayed tileset entries to those whose file names contain the search text (case-insensitive).
3. WHEN the user selects a tileset from the dropdown, THE Editor SHALL set it as the active tileset and display its tile grid.
4. THE Tile_Palette SHALL display the currently active tileset's file name as the dropdown's selected value.
5. WHILE the search field is empty, THE Tile_Palette SHALL display all tilesets sorted alphabetically by file name.

### Requirement 7: Tileset Palette Tile Scaling Control

**User Story:** As a game designer, I want to control the display size of tiles in the palette, so that small tiles (8×8) remain legible without making the entire panel excessively large.

#### Acceptance Criteria

1. THE Tile_Palette SHALL provide a zoom slider or scale control that adjusts the Display_Tile_Size independently of the panel width.
2. THE Tile_Palette SHALL enforce a minimum Display_Tile_Size of 16 pixels regardless of the actual tile dimensions in the tileset.
3. THE Tile_Palette SHALL enforce a maximum Display_Tile_Size of 128 pixels.
4. WHEN the user adjusts the zoom control, THE Tile_Palette SHALL immediately re-render the tile grid at the new Display_Tile_Size.
5. THE Tile_Palette SHALL persist the current zoom level across tileset switches within the same editor session.
6. THE Tile_Palette SHALL default the Display_Tile_Size to the larger of the tileset's native tile width or 24 pixels.
