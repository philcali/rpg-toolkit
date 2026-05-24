# Requirements Document

## Introduction

This feature introduces a tile elevation (z-level) system that is independent of the existing visual layer system. Elevation allows tiles to exist at different logical heights, enabling scenarios like bridges where a player can walk underneath at ground level or across at bridge level. The feature spans the common data model, editor UI, renderer draw-order logic, and collision system.

## Glossary

- **Elevation_Level**: An integer value (starting at 0) representing the logical height of a tile or entity, independent of the visual layer system. Level 0 is ground level.
- **Tile_Elevation**: The elevation level assigned to a specific tile position on a specific layer, stored in the tile attribute data.
- **Player_Elevation**: The current elevation level at which the player character exists, determining which tiles are collidable and which render above the player.
- **Elevation_Transition**: A tile attribute that changes the player's elevation level when stepped on (e.g., stairs, ramps).
- **Editor**: The Bevy-based map editor application (`rpg-toolkit-editor` crate).
- **Renderer**: The Bevy-based game runtime (`rpg-toolkit-renderer` crate) responsible for draw order and gameplay.
- **Collision_System**: The subsystem within the Renderer that determines whether a tile blocks player movement.
- **Common_Data_Model**: The shared data types in `rpg-toolkit-common` used by both Editor and Renderer.
- **Layer**: A visual painting layer in the map, used for stacking tile graphics (e.g., ground, decorations, canopy).
- **TileAttributes**: The per-tile metadata structure that currently holds opacity and event trigger data.

## Requirements

### Requirement 1: Elevation Field in Tile Attributes

**User Story:** As a map designer, I want each tile position to have an elevation value, so that I can define which logical height a tile exists at independently of its visual layer.

#### Acceptance Criteria

1. THE Common_Data_Model SHALL include an elevation field of type integer in the TileAttributes structure, defaulting to 0.
2. WHEN a map file is deserialized that does not contain elevation data, THE Common_Data_Model SHALL default all tile elevations to 0 without error.
3. WHEN a map file is serialized, THE Common_Data_Model SHALL persist the elevation value for each tile position.
4. FOR ALL valid TileAttributes values, serializing then deserializing SHALL produce an equivalent TileAttributes value (round-trip property).

### Requirement 2: Player Elevation State

**User Story:** As a game designer, I want the player character to have a current elevation level, so that the game can determine which tiles interact with the player.

#### Acceptance Criteria

1. THE Renderer SHALL store a current elevation level as an integer on the PlayerCharacter component, defaulting to 0.
2. WHEN the player spawns, THE Renderer SHALL set the player's elevation level to 0.
3. THE Renderer SHALL make the player's current elevation level available to the Collision_System and the draw-order system.

### Requirement 3: Elevation Transition Tiles

**User Story:** As a map designer, I want to mark tiles as elevation transitions (e.g., stairs), so that the player's elevation changes when they step onto those tiles.

#### Acceptance Criteria

1. THE Common_Data_Model SHALL include an optional target elevation field in TileAttributes, representing the elevation level the player transitions to upon entering the tile.
2. WHEN the player moves onto a tile that has a target elevation value set, THE Renderer SHALL update the player's elevation level to the target elevation value.
3. WHEN the player moves onto a tile that does not have a target elevation value set, THE Renderer SHALL keep the player's elevation level unchanged.
4. THE Renderer SHALL apply the elevation transition after the movement animation completes.

### Requirement 4: Elevation-Aware Collision Detection

**User Story:** As a game designer, I want obstacle tiles to only block the player when the player is at the same elevation, so that a player at ground level can walk under a bridge while a player at bridge level is blocked by bridge railings.

#### Acceptance Criteria

1. WHEN the player attempts to move to a tile, THE Collision_System SHALL compare the player's current elevation level with the tile's elevation level.
2. WHEN a tile has the opacity attribute set and the tile's elevation level equals the player's current elevation level, THE Collision_System SHALL block the player's movement.
3. WHEN a tile has the opacity attribute set and the tile's elevation level does not equal the player's current elevation level, THE Collision_System SHALL allow the player's movement.
4. WHEN a tile does not have the opacity attribute set, THE Collision_System SHALL allow the player's movement regardless of elevation levels.

### Requirement 5: Elevation-Aware Draw Order

**User Story:** As a game designer, I want tiles above the player's current elevation to render on top of the player sprite, so that walking under a bridge looks correct visually.

#### Acceptance Criteria

1. WHEN a tile's elevation level is greater than the player's current elevation level, THE Renderer SHALL render that tile's sprite above the player sprite in draw order.
2. WHEN a tile's elevation level is less than or equal to the player's current elevation level, THE Renderer SHALL render that tile's sprite below the player sprite in draw order.
3. WHEN the player's elevation level changes, THE Renderer SHALL update the draw order of all affected tile sprites within the same frame.

### Requirement 6: Editor Elevation Tool

**User Story:** As a map designer, I want a tool in the editor to set the elevation level of tiles, so that I can author maps with multi-level terrain.

#### Acceptance Criteria

1. THE Editor SHALL provide an elevation tool within the attribute editing mode.
2. WHEN the elevation tool is active and the user clicks a tile, THE Editor SHALL display an interface to set the elevation level for that tile position.
3. WHEN the user confirms an elevation value, THE Editor SHALL store the value in the TileAttributes for the active layer at the clicked position.
4. WHEN the user sets an elevation value, THE Editor SHALL generate an undo-able EditCommand capturing the old and new elevation values.
5. THE Editor SHALL display a visual overlay indicating the elevation level of tiles when the elevation tool is active.

### Requirement 7: Editor Elevation Transition Tool

**User Story:** As a map designer, I want to mark tiles as elevation transitions in the editor, so that I can place stairs and ramps that change the player's level.

#### Acceptance Criteria

1. THE Editor SHALL provide a way to set the target elevation value on a tile within the attribute editing mode.
2. WHEN the user sets a target elevation on a tile, THE Editor SHALL store the value in the TileAttributes for the active layer at the clicked position.
3. WHEN the user sets a target elevation value, THE Editor SHALL generate an undo-able EditCommand capturing the old and new target elevation values.
4. THE Editor SHALL display a distinct visual overlay for tiles that have a target elevation value set.

### Requirement 8: Elevation Data Validation

**User Story:** As a developer, I want the elevation data to be validated on load, so that corrupted or out-of-range values are caught early.

#### Acceptance Criteria

1. WHEN a map is validated, THE Common_Data_Model SHALL verify that all elevation values are non-negative integers.
2. WHEN a map is validated, THE Common_Data_Model SHALL verify that all target elevation values (when present) are non-negative integers.
3. IF an elevation value is negative, THEN THE Common_Data_Model SHALL return a validation error identifying the tile position and layer.

### Requirement 9: NPC Elevation Awareness

**User Story:** As a game designer, I want NPCs to exist at a specific elevation level, so that the player only collides with NPCs at the same elevation.

#### Acceptance Criteria

1. THE Common_Data_Model SHALL include an elevation field on the NpcInstance structure, defaulting to 0.
2. WHEN the player attempts to move to a tile occupied by an NPC, THE Collision_System SHALL block movement only when the player's elevation level equals the NPC's elevation level.
3. WHEN the player's elevation level does not equal the NPC's elevation level, THE Collision_System SHALL allow the player to move through the NPC's tile position.
4. THE Renderer SHALL apply elevation-aware draw order to NPC sprites using the same rules as tile sprites.

### Requirement 10: Elevation-Aware JumpTo Events

**User Story:** As a map designer, I want JumpTo events to specify a target elevation, so that the player arrives at the correct z-level when entering or exiting buildings and transitioning between maps.

#### Acceptance Criteria

1. THE Common_Data_Model SHALL include an optional target_elevation field of type integer in the JumpTo EventAction variant.
2. WHEN a JumpTo action is executed and target_elevation is set, THE Renderer SHALL update the player's elevation level to the target_elevation value after the map transition completes.
3. WHEN a JumpTo action is executed and target_elevation is not set, THE Renderer SHALL preserve the player's current elevation level.
4. THE Editor SHALL display a target elevation input field in the JumpTo action editor form.
5. WHEN a map file containing JumpTo actions is deserialized and target_elevation is absent, THE Common_Data_Model SHALL default target_elevation to None without error.

### Requirement 11: Tile Coordinate Tooltip

**User Story:** As a map designer, I want to see the grid coordinates of the tile under my cursor, so that I can easily determine target coordinates when configuring JumpTo events.

#### Acceptance Criteria

1. WHEN the user hovers the mouse over a tile in the map canvas, THE Editor SHALL display a tooltip showing the tile's grid coordinates in `(x, y)` format.
2. WHEN the mouse moves to a different tile, THE Editor SHALL update the tooltip to reflect the new tile's coordinates.
3. WHEN the mouse leaves the map canvas area, THE Editor SHALL hide the coordinate tooltip.
4. THE Editor SHALL display the tooltip regardless of which editing tool is currently active.
