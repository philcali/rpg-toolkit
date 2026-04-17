# Requirements Document

## Introduction

The RPG Toolkit editor currently models a project as a single map tied to a single tileset. This limits real-world usage: even a small scene may draw from multiple tilesets, and a game with many scenes forces the user to juggle separate project files. This feature introduces first-class support for multiple maps and multiple tilesets within a single project, while keeping the canvas focused on one map at a time.

## Glossary

- **Project**: The top-level container persisted as a single JSON file. A Project owns zero or more Maps and zero or more Tilesets.
- **Map**: A named grid of tile layers with fixed dimensions. Each Map may reference tiles from any Tileset registered in the Project.
- **Tileset**: A sprite-sheet image plus its grid metadata (tile size, columns, rows, file path). Identified by a unique Tileset_ID within the Project.
- **Tileset_ID**: A stable string identifier (e.g. UUID or slug) that uniquely identifies a Tileset within a Project.
- **Map_ID**: A stable string identifier that uniquely identifies a Map within a Project.
- **TileRef**: A reference to a specific tile, consisting of a Tileset_ID plus a column and row within that Tileset. Replaces the current `TileIndex` which assumes a single tileset.
- **Active_Map**: The single Map currently displayed on the canvas and receiving edits.
- **Map_Tab_Bar**: A horizontal tab strip showing all Maps the user has opened for editing. Exactly one tab is selected at a time, corresponding to the Active_Map.
- **Tileset_Tab_Bar**: A tab strip inside the Tile Palette panel showing all Tilesets loaded in the Project. The user selects a tab to browse and pick tiles from that Tileset.
- **Map_Browser**: A side panel listing every Map in the Project. Double-clicking or pressing an "Open" action on a Map opens it in the Map_Tab_Bar.
- **Editor**: The RPG Toolkit application.
- **Serializer**: The subsystem responsible for writing Project data to JSON.
- **Deserializer**: The subsystem responsible for reading Project data from JSON.

## Requirements

### Requirement 1: Project contains multiple Maps

**User Story:** As a game developer, I want a single project to hold many maps, so that I can manage all scenes of my game in one place.

#### Acceptance Criteria

1. THE Project SHALL store zero or more Maps, each identified by a unique Map_ID.
2. WHEN the user creates a new Map via the "New Map" dialog, THE Editor SHALL add the Map to the Project and assign it a unique Map_ID.
3. WHEN the user deletes a Map, THE Editor SHALL remove the Map and its Map_ID from the Project.
4. IF the user attempts to delete the last remaining Map in the Project, THEN THE Editor SHALL display an error message and keep the Map.

### Requirement 2: Maps own their tile dimensions

**User Story:** As a game developer, I want each map to define its own tile width and height, so that the map is the authoritative source for scene geometry and I can validate that only compatible tilesets are used.

#### Acceptance Criteria

1. THE MapData SHALL store a `tile_width` and `tile_height` (in pixels), set at creation time via the "New Map" dialog.
2. WHEN the user attempts to paint with a Tileset whose tile size does not match the Active_Map's tile dimensions, THE Editor SHALL prevent the operation and display a warning.
3. THE Canvas SHALL use the Active_Map's `tile_width` and `tile_height` to compute grid spacing and sprite placement.

### Requirement 3: Project contains multiple Tilesets

**User Story:** As a game developer, I want to load several tilesets into one project, so that a single map can use tiles from different sprite sheets.

#### Acceptance Criteria

1. THE Project SHALL store zero or more Tilesets, each identified by a unique Tileset_ID.
2. WHEN the user loads a tileset image via the "Load Tileset" dialog, THE Editor SHALL add the Tileset to the Project and assign it a unique Tileset_ID.
3. WHEN the user removes a Tileset from the Project, THE Editor SHALL remove the Tileset and its Tileset_ID from the Project.
4. IF the user attempts to remove a Tileset that is referenced by tiles placed on any Map, THEN THE Editor SHALL display a confirmation dialog warning that placed tiles referencing the Tileset will become invalid.

### Requirement 4: Tile references include Tileset identity

**User Story:** As a game developer, I want each placed tile to know which tileset it came from, so that maps can mix tiles from different tilesets without ambiguity.

#### Acceptance Criteria

1. THE Map SHALL store each placed tile as a TileRef containing a Tileset_ID, a column, and a row.
2. WHEN the user places a tile, THE Editor SHALL record the Tileset_ID of the currently selected Tileset_Tab along with the tile column and row.
3. IF a TileRef references a Tileset_ID that does not exist in the Project, THEN THE Editor SHALL skip rendering that tile and log a warning.

### Requirement 5: Map Tab Bar

**User Story:** As a game developer, I want tabs for the maps I am editing, so that I can quickly switch between scenes.

#### Acceptance Criteria

1. THE Editor SHALL display a Map_Tab_Bar above the canvas area showing one tab per opened Map.
2. WHEN the user clicks a Map tab, THE Editor SHALL set that Map as the Active_Map and display it on the canvas.
3. WHEN the user opens a Map that is not yet in the Map_Tab_Bar, THE Editor SHALL add a new tab for that Map and set it as the Active_Map.
4. WHEN the user closes a Map tab, THE Editor SHALL remove the tab from the Map_Tab_Bar.
5. IF the user closes the tab of the Active_Map, THEN THE Editor SHALL activate the nearest remaining tab, or show an empty canvas if no tabs remain.
6. WHILE a Map has unsaved changes, THE Editor SHALL display a modified indicator on that Map's tab.

### Requirement 6: Map Browser panel

**User Story:** As a game developer, I want a panel listing all maps in my project, so that I can find and open any scene.

#### Acceptance Criteria

1. THE Editor SHALL display a Map_Browser panel listing every Map in the Project by name.
2. WHEN the user double-clicks a Map entry in the Map_Browser, THE Editor SHALL open that Map in the Map_Tab_Bar.
3. WHEN the user right-clicks a Map entry in the Map_Browser, THE Editor SHALL show a context menu with "Open", "Rename", and "Delete" actions.
4. WHEN the user selects "Rename" from the context menu, THE Editor SHALL allow inline editing of the Map name.
5. WHEN the user selects "Delete" from the context menu, THE Editor SHALL prompt for confirmation before deleting the Map.

### Requirement 7: Tileset Tab Bar in Tile Palette

**User Story:** As a game developer, I want tabs on the loaded tilesets, so that I can switch between sprite sheets while painting.

#### Acceptance Criteria

1. THE Tile Palette panel SHALL display a Tileset_Tab_Bar showing one tab per Tileset in the Project.
2. WHEN the user clicks a Tileset tab, THE Tile Palette SHALL display the tile grid for that Tileset.
3. WHEN the user selects a tile from the Tile Palette, THE Editor SHALL set the active brush to a TileRef containing the selected Tileset's Tileset_ID and the tile's column and row.
4. WHEN a new Tileset is loaded into the Project, THE Tile Palette SHALL add a tab for the new Tileset and switch to it.

### Requirement 8: Canvas renders the Active Map only

**User Story:** As a game developer, I want the canvas to show only the map I am working on, so that I can focus on one scene at a time.

#### Acceptance Criteria

1. THE Canvas SHALL render tiles and the grid overlay for the Active_Map only.
2. WHEN the Active_Map changes, THE Canvas SHALL despawn all tile sprites and respawn sprites for the new Active_Map.
3. THE Canvas SHALL resolve each TileRef to the correct Tileset texture and atlas index when rendering sprites.

### Requirement 9: Per-Map undo/redo history

**User Story:** As a game developer, I want undo/redo to be scoped to the map I am editing, so that switching maps does not lose my undo history.

#### Acceptance Criteria

1. THE Editor SHALL maintain a separate undo/redo history for each Map.
2. WHEN the user performs an undo or redo, THE Editor SHALL apply the operation to the Active_Map's history only.
3. WHEN the user switches the Active_Map, THE Editor SHALL preserve the undo/redo history of the previously active Map.

### Requirement 10: Updated project serialization

**User Story:** As a game developer, I want my multi-map, multi-tileset project to save and load correctly, so that I do not lose work.

#### Acceptance Criteria

1. THE Serializer SHALL write all Maps and all Tilesets in the Project to a single JSON file.
2. THE Deserializer SHALL read the JSON file and reconstruct all Maps and Tilesets with their Map_IDs and Tileset_IDs.
3. FOR ALL valid Project values, serializing then deserializing SHALL produce an equivalent Project value (round-trip property).
4. IF the Deserializer encounters invalid or corrupt JSON, THEN THE Deserializer SHALL return a descriptive error without modifying the current Project state.

### Requirement 11: New Map dialog updates

**User Story:** As a game developer, I want the "New Map" dialog to add a map to the current project rather than replacing the whole project.

#### Acceptance Criteria

1. WHEN the user confirms the "New Map" dialog, THE Editor SHALL add the new Map to the existing Project instead of replacing the current Map.
2. WHEN the new Map is added, THE Editor SHALL open the new Map in the Map_Tab_Bar and set it as the Active_Map.
