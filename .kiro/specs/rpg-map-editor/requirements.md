# Requirements Document

## Introduction

This document defines the requirements for the first iteration of a retro-style RPG Map Editor, built as a cross-platform thick client using Bevy and bevy_egui. This iteration focuses on establishing the application foundation, map creation, tileset loading, and project serialization. Future iterations will add NPC characters, dialogs, scene transitions, and storyboard features.

## Glossary

- **Editor**: The RPG Map Editor application built with Bevy and bevy_egui
- **Map**: A 2D grid of tiles representing a game scene, defined by a width and height in tile units
- **Tile**: A single cell in the Map grid, referencing a specific region of a Tileset
- **Tileset**: An image file containing a grid of fixed-size tile graphics used to paint the Map
- **Tile_Index**: A coordinate pair (column, row) identifying a specific tile graphic within a Tileset
- **Layer**: A named, ordered drawing surface within a Map; multiple Layers stack to compose the final scene
- **Canvas**: The 2D viewport area in the Editor where the Map is rendered and edited
- **Tile_Palette**: The UI panel displaying available tiles from the currently loaded Tileset for selection
- **Project**: The collection of Map data, Tileset references, and Layer configuration that constitutes a saved editor session
- **Project_File**: The serialized on-disk representation of a Project
- **Serializer**: The component responsible for writing Project data to a Project_File
- **Deserializer**: The component responsible for reading a Project_File back into Project data
- **Pretty_Printer**: The component responsible for formatting Project data into a human-readable Project_File representation

## Requirements

### Requirement 1: Application Shell

**User Story:** As a game designer, I want a cross-platform desktop application with a responsive UI, so that I can use the editor on Windows, macOS, and Linux.

#### Acceptance Criteria

1. THE Editor SHALL launch a native window using Bevy with an embedded bevy_egui UI layer
2. THE Editor SHALL display a menu bar with File and Edit menus
3. THE Editor SHALL display the Canvas in the central area of the window
4. THE Editor SHALL display the Tile_Palette in a side panel
5. WHEN the user resizes the application window, THE Editor SHALL reflow the UI layout to fit the new dimensions
6. THE Editor SHALL maintain a minimum window size of 800x600 pixels

### Requirement 2: Map Creation

**User Story:** As a game designer, I want to create new maps with configurable dimensions, so that I can define the playable area for a game scene.

#### Acceptance Criteria

1. WHEN the user selects "New Map" from the File menu, THE Editor SHALL display a dialog prompting for map name, width (in tiles), and height (in tiles)
2. WHEN the user confirms the New Map dialog with valid dimensions, THE Editor SHALL create a Map with the specified width and height, filled with empty Tiles
3. THE Editor SHALL support Map dimensions from 1x1 to 256x256 tiles
4. IF the user enters dimensions outside the supported range, THEN THE Editor SHALL display an error message indicating the valid range
5. WHEN a new Map is created, THE Canvas SHALL display the empty Map grid

### Requirement 3: Tileset Loading

**User Story:** As a game designer, I want to load tileset images into the editor, so that I can use tile graphics to paint my maps.

#### Acceptance Criteria

1. WHEN the user selects "Load Tileset" from the File menu, THE Editor SHALL open a native file dialog filtered to PNG and JPEG image formats
2. WHEN the user selects a valid image file, THE Editor SHALL load the image and partition it into a grid of tiles based on a configurable tile size (default 16x16 pixels)
3. WHEN a Tileset is loaded, THE Tile_Palette SHALL display all tiles from the Tileset in a scrollable grid
4. IF the user selects an unsupported file format, THEN THE Editor SHALL display an error message listing the supported formats
5. IF the selected image file cannot be read or is corrupted, THEN THE Editor SHALL display an error message describing the failure
6. THE Editor SHALL allow the user to configure the tile size (8x8, 16x16, 32x32, or 64x64 pixels) when loading a Tileset

### Requirement 4: Tile Painting

**User Story:** As a game designer, I want to paint tiles onto the map by selecting from the palette and clicking on the canvas, so that I can visually compose game scenes.

#### Acceptance Criteria

1. WHEN the user clicks a tile in the Tile_Palette, THE Editor SHALL set that tile as the active brush
2. WHEN the user clicks a cell on the Canvas while a brush is active, THE Editor SHALL place the active brush Tile at the clicked Map cell on the current Layer
3. WHEN the user clicks and drags across the Canvas while a brush is active, THE Editor SHALL paint the active brush Tile on each Map cell the cursor passes over
4. THE Canvas SHALL render each placed Tile using the corresponding graphic from the Tileset
5. WHEN the user right-clicks a cell on the Canvas, THE Editor SHALL erase the Tile at that cell on the current Layer, setting it to empty

### Requirement 5: Layer Management

**User Story:** As a game designer, I want to organize my map into multiple layers, so that I can separate ground, objects, and overlay elements.

#### Acceptance Criteria

1. WHEN a new Map is created, THE Editor SHALL create a default Layer named "Ground"
2. THE Editor SHALL display a Layer list panel showing all Layers in their stacking order
3. WHEN the user clicks "Add Layer" in the Layer list panel, THE Editor SHALL create a new empty Layer above the currently selected Layer
4. WHEN the user selects a Layer in the Layer list panel, THE Editor SHALL set that Layer as the active drawing target
5. THE Canvas SHALL render all visible Layers composited in their stacking order, with higher Layers drawn on top of lower Layers
6. WHEN the user toggles the visibility icon on a Layer, THE Editor SHALL show or hide that Layer on the Canvas
7. WHEN the user clicks "Delete Layer" in the Layer list panel, THE Editor SHALL remove the selected Layer and its tile data
8. IF only one Layer remains, THEN THE Editor SHALL disable the Delete Layer action

### Requirement 6: Canvas Navigation

**User Story:** As a game designer, I want to pan and zoom the map canvas, so that I can navigate large maps and work at different detail levels.

#### Acceptance Criteria

1. WHEN the user scrolls the mouse wheel over the Canvas, THE Editor SHALL zoom the Canvas view in or out, centered on the cursor position
2. THE Editor SHALL support zoom levels from 25% to 800%
3. WHEN the user holds the middle mouse button and drags over the Canvas, THE Editor SHALL pan the Canvas view in the drag direction
4. WHEN a new Map is created, THE Editor SHALL set the Canvas zoom to fit the entire Map within the visible area
5. THE Canvas SHALL display a grid overlay aligned to tile boundaries at all zoom levels

### Requirement 7: Project Serialization

**User Story:** As a game designer, I want to save and load my map projects, so that I can preserve my work and continue editing later.

#### Acceptance Criteria

1. WHEN the user selects "Save Project" from the File menu, THE Serializer SHALL write the current Project to a Project_File in JSON format
2. THE Project_File SHALL contain the Map dimensions, Layer definitions, Tile data for each Layer, and Tileset file path references
3. WHEN the user selects "Open Project" from the File menu, THE Deserializer SHALL read a Project_File and restore the Map, Layers, and Tileset references
4. THE Pretty_Printer SHALL format Project_File JSON with indentation and readable field names
5. FOR ALL valid Project data, serializing then deserializing SHALL produce a Project equivalent to the original (round-trip property)
6. IF the Project_File is malformed or contains invalid data, THEN THE Deserializer SHALL display an error message describing the issue
7. WHEN the user selects "Save Project" and no save path exists, THE Editor SHALL open a file dialog for the user to choose a save location
8. WHEN the user has unsaved changes and attempts to close the Editor or create a new Map, THE Editor SHALL prompt the user to save, discard, or cancel the action

### Requirement 8: Undo and Redo

**User Story:** As a game designer, I want to undo and redo my editing actions, so that I can correct mistakes without starting over.

#### Acceptance Criteria

1. WHEN the user presses Ctrl+Z (Cmd+Z on macOS), THE Editor SHALL undo the most recent editing action
2. WHEN the user presses Ctrl+Y (Cmd+Shift+Z on macOS), THE Editor SHALL redo the most recently undone action
3. THE Editor SHALL maintain an undo history of at least 50 actions
4. WHEN the user performs a new editing action after undoing, THE Editor SHALL discard the redo history
5. THE Editor SHALL support undo and redo for tile placement, tile erasure, Layer creation, and Layer deletion
