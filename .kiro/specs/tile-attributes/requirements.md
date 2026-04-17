# Requirements Document

## Introduction

The Tile Attributes feature adds an alternate editor mode where users can define per-tile metadata on each layer. This includes marking tiles as opaque (impassable), attaching event triggers (starting with scene-jump transitions), and placing the player character's initial spawn point on a map. These attributes are stored alongside the existing tile data and serialized with the project file.

## Glossary

- **Attribute_Mode**: An alternate editor view mode (distinct from the painting view) in which the user edits tile attributes instead of painting tiles.
- **Opacity_Attribute**: A per-tile, per-layer boolean flag indicating that a tile is impassable/solid.
- **Event_Trigger**: A per-tile, per-layer ordered list of Event_Actions that execute sequentially when the player character collides with the tile. An empty list means no trigger is assigned.
- **Event_Action**: A single action within an Event_Trigger sequence. Each action has a specific type (e.g., JumpTo). Future action types may include dialog sequences, NPC movement, camera effects, sound playback, etc.
- **JumpTo_Action**: A specific Event_Action type that transitions the player to a target scene at a given (x, y) tile coordinate.
- **Spawn_Point**: A single (map_id, x, y) coordinate on the ground layer (layer index 0) designating the initial position of the player character. Only one Spawn_Point may exist across the entire project at any time. Future versions may allow spawn points on other layers.
- **Tile_Attribute_Layer**: A parallel data structure to the tile grid on each Layer that stores Opacity_Attribute and Event_Trigger values for every cell.
- **Editor**: The RPG Toolkit Bevy/egui application.
- **Canvas**: The central rendering area where the map grid is displayed.
- **Project_File**: The JSON serialization format used to persist all project data to disk.

## Requirements

### Requirement 1: Editor Mode Toggle

**User Story:** As a map designer, I want to switch between painting mode and attribute mode, so that I can edit tile metadata without accidentally painting tiles.

#### Acceptance Criteria

1. THE Editor SHALL provide a toggle control that switches between painting mode and Attribute_Mode.
2. WHEN the user activates Attribute_Mode, THE Editor SHALL disable all painting tools (Paint, Erase, FloodFill, StampBrush).
3. WHEN the user activates Attribute_Mode, THE Canvas SHALL display an attribute overlay on top of the existing tile rendering.
4. WHEN the user deactivates Attribute_Mode, THE Editor SHALL restore the previously active painting tool.
5. WHILE Attribute_Mode is active, THE Editor SHALL display a visual indicator confirming the current mode.

### Requirement 2: Opacity Attribute Editing

**User Story:** As a map designer, I want to mark individual tiles as opaque on a per-layer basis, so that the game engine knows which tiles block character movement.

#### Acceptance Criteria

1. WHILE Attribute_Mode is active, THE Editor SHALL allow the user to toggle the Opacity_Attribute on any tile cell in the active layer by clicking on the Canvas.
2. WHEN the user clicks a tile that has Opacity_Attribute set to false, THE Editor SHALL set the Opacity_Attribute to true for that tile on the active layer.
3. WHEN the user clicks a tile that has Opacity_Attribute set to true, THE Editor SHALL set the Opacity_Attribute to false for that tile on the active layer.
4. WHILE Attribute_Mode is active, THE Canvas SHALL render a distinct visual indicator (e.g., a colored overlay or icon) on tiles where Opacity_Attribute is true.
5. THE Tile_Attribute_Layer SHALL default the Opacity_Attribute to false for every tile cell.
6. WHEN a new layer is added to a map, THE Editor SHALL initialize a corresponding Tile_Attribute_Layer with all Opacity_Attribute values set to false.

### Requirement 3: Event Trigger Assignment

**User Story:** As a map designer, I want to attach event triggers to specific tiles, so that the game engine can respond when the player character collides with those tiles.

#### Acceptance Criteria

1. WHILE Attribute_Mode is active, THE Editor SHALL allow the user to edit the Event_Trigger (action sequence) on any tile cell in the active layer.
2. WHEN the user edits an Event_Trigger on a tile, THE Editor SHALL display a configuration panel showing the ordered list of Event_Actions and allowing the user to add, remove, or reorder actions.
3. THE Editor SHALL support the JumpTo_Action type as the initial Event_Action variant.
4. WHEN the user adds a JumpTo_Action, THE Editor SHALL prompt the user to specify a target map identifier and a target (x, y) tile coordinate.
5. WHEN the user confirms the Event_Trigger configuration, THE Editor SHALL store the ordered list of Event_Actions for that tile on the active layer.
6. WHILE Attribute_Mode is active, THE Canvas SHALL render a distinct visual indicator on tiles that have a non-empty Event_Trigger (one or more Event_Actions).
7. WHEN the user selects a tile that already has Event_Actions, THE Editor SHALL display the existing action sequence for editing or removal.
8. THE Tile_Attribute_Layer SHALL default the Event_Trigger to an empty list for every tile cell.

### Requirement 4: Spawn Point Placement

**User Story:** As a map designer, I want to place a player character spawn point on a map, so that the game engine knows where to position the player when the map loads.

#### Acceptance Criteria

1. WHILE Attribute_Mode is active, THE Editor SHALL provide a spawn-point placement tool.
2. WHEN the user activates the spawn-point tool and clicks a tile on the Canvas, THE Editor SHALL set the Spawn_Point for the project to the current map and that tile coordinate on the ground layer (layer index 0).
3. THE Project SHALL store at most one Spawn_Point across all maps.
4. WHEN the user attempts to place a Spawn_Point and one already exists anywhere in the project, THE Editor SHALL display a confirmation modal informing the user of the existing spawn point location (map name and coordinates) and asking whether to move it.
5. IF the user confirms the modal, THE Editor SHALL move the Spawn_Point to the new location, removing it from the previous map.
6. IF the user cancels the modal, THE Editor SHALL leave the existing Spawn_Point unchanged.
7. WHILE Attribute_Mode is active, THE Canvas SHALL render a distinct visual marker at the Spawn_Point location if it exists on the current map, regardless of which layer is currently active.
8. THE Spawn_Point SHALL default to None when a new project is created.
9. THE Spawn_Point placement SHALL operate on the ground layer (layer index 0) regardless of which layer is currently selected as active.

### Requirement 5: Tile Attribute Data Model

**User Story:** As a developer, I want tile attributes stored in a structured data model parallel to the tile grid, so that attribute data is cleanly separated from tile rendering data.

#### Acceptance Criteria

1. THE Tile_Attribute_Layer SHALL contain a 2D grid matching the dimensions of the corresponding Layer tile grid.
2. Each cell in the Tile_Attribute_Layer SHALL store an Opacity_Attribute (boolean) and an Event_Trigger (ordered list of Event_Actions).
3. WHEN a map is resized, THE Editor SHALL resize all Tile_Attribute_Layer grids to match the new dimensions, preserving existing attribute values within the valid range.
4. THE Event_Action SHALL be represented as a tagged enum. The initial variant SHALL be JumpTo, containing a target map identifier (String) and target coordinates (u32, u32). The enum is designed to be extended with additional action types in the future.

### Requirement 6: Attribute Serialization

**User Story:** As a map designer, I want tile attributes saved and loaded with the project file, so that attribute data persists across editing sessions.

#### Acceptance Criteria

1. WHEN the user saves the project, THE Serialization_System SHALL include all Tile_Attribute_Layer data and Spawn_Point data in the Project_File.
2. WHEN the user loads a project, THE Serialization_System SHALL restore all Tile_Attribute_Layer data and Spawn_Point data from the Project_File.
3. IF a loaded Project_File does not contain Tile_Attribute_Layer data for a layer, THEN THE Serialization_System SHALL initialize default attribute values (Opacity_Attribute false, Event_Trigger empty list) for that layer.
4. IF a loaded Project_File contains a JumpTo_Action referencing a map identifier not present in the project, THEN THE Serialization_System SHALL log a warning and preserve the action data without modification.
5. FOR ALL valid Project data, serializing then deserializing SHALL produce an equivalent Project (round-trip property).

### Requirement 7: Undo/Redo for Attribute Edits

**User Story:** As a map designer, I want to undo and redo attribute changes, so that I can correct mistakes when editing tile attributes.

#### Acceptance Criteria

1. WHEN the user toggles an Opacity_Attribute, THE Editor SHALL record an undoable command capturing the previous and new values.
2. WHEN the user assigns or removes an Event_Trigger, THE Editor SHALL record an undoable command capturing the previous and new trigger values.
3. WHEN the user places or moves a Spawn_Point, THE Editor SHALL record an undoable command capturing the previous and new Spawn_Point values.
4. WHEN the user performs an undo of an attribute edit, THE Editor SHALL restore the attribute to its previous value.
5. WHEN the user performs a redo of an undone attribute edit, THE Editor SHALL reapply the attribute change.
