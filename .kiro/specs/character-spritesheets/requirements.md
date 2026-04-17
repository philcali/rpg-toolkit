# Requirements Document

## Introduction

This feature introduces character spritesheets and NPC placement into the RPG toolkit. Character spritesheets replace the current solid-color player rectangle with animated, directional sprites using an RPG Maker-style grid layout (3 frames × 4 directions, 24×32 pixels). The system supports loading spritesheets at the project level, skinning the player character at spawn, and placing NPC instances on maps via the editor's attribute tooling. NPCs are grid-snapped entities with a spritesheet reference and facing direction, rendered in the game world and blocking player movement when sharing the same layer.

## Glossary

- **Character_Spritesheet**: A project-level image asset containing a grid of animation frames for a character, organized as 3 columns (frames) × 4 rows (directions: down, left, right, up), with each frame sized 24×32 pixels.
- **Spritesheet_Registry**: The project-level collection of all loaded Character_Spritesheets, analogous to the tileset registry.
- **Collision_Box**: A 16×16 pixel bounding area at the bottom-center of a 24×32 character sprite, used for tile-based collision detection.
- **NPC_Instance**: A per-map entity placed on a specific tile, referencing a Character_Spritesheet and a facing direction.
- **Facing_Direction**: One of four cardinal directions (Down, Left, Right, Up) determining which sprite row to display.
- **Animation_Frame**: One of three sequential images in a walk cycle for a given Facing_Direction.
- **Player_Spritesheet_Reference**: A project-level setting that specifies which Character_Spritesheet is used to render the player character.
- **NPC_Placement_Tool**: An editor attribute tool for placing, selecting, and removing NPC_Instances on the active map.
- **Editor**: The rpg-toolkit-editor crate providing map editing, tileset management, and attribute tooling.
- **Renderer**: The rpg-toolkit-renderer crate that renders the project as a playable game world.
- **Project_File**: The serialized on-disk format containing maps, tilesets, spritesheets, spawn point, and NPC data.

## Requirements

### Requirement 1: Character Spritesheet Loading

**User Story:** As a game creator, I want to load character spritesheet images into my project, so that I can use them for the player and NPCs.

#### Acceptance Criteria

1. WHEN a user imports a character spritesheet image via the Editor, THE Spritesheet_Registry SHALL store a new Character_Spritesheet entry containing the file path, sprite dimensions (24×32), frame count (3), direction count (4), and a unique identifier.
2. WHEN a character spritesheet image is imported, THE Editor SHALL validate that the image dimensions are exactly 72×128 pixels (3 frames × 24 width, 4 directions × 32 height).
3. IF a character spritesheet image does not match the expected 72×128 dimensions, THEN THE Editor SHALL display a descriptive error message and reject the import.
4. THE Spritesheet_Registry SHALL persist all Character_Spritesheet entries in the Project_File using JSON serialization.
5. WHEN a project file containing Character_Spritesheet entries is loaded, THE Editor SHALL deserialize and validate each entry, restoring the Spritesheet_Registry.
6. FOR ALL valid Spritesheet_Registry states, serializing then deserializing the Project_File SHALL produce an equivalent Spritesheet_Registry (round-trip property).

### Requirement 2: Character Spritesheet Metadata

**User Story:** As a game creator, I want to see metadata about each character spritesheet, so that I can manage my sprite assets effectively.

#### Acceptance Criteria

1. THE Spritesheet_Registry SHALL track which NPC_Instances and Player_Spritesheet_Reference are actively using each Character_Spritesheet.
2. WHEN a Character_Spritesheet is selected in the Editor, THE Editor SHALL display the spritesheet's file path, sprite dimensions, and a list of NPC_Instances currently referencing the spritesheet.
3. IF a user attempts to remove a Character_Spritesheet that is referenced by one or more NPC_Instances or the Player_Spritesheet_Reference, THEN THE Editor SHALL display a warning listing all active references and require confirmation before removal.

### Requirement 3: Player Spritesheet Assignment

**User Story:** As a game creator, I want to assign a character spritesheet to the player character, so that the player appears as an animated sprite instead of a colored rectangle.

#### Acceptance Criteria

1. THE Project_File SHALL store an optional Player_Spritesheet_Reference identifying which Character_Spritesheet is used for the player character.
2. WHEN a Player_Spritesheet_Reference is set and the player spawns, THE Renderer SHALL render the player using the referenced Character_Spritesheet instead of a solid-color rectangle.
3. IF no Player_Spritesheet_Reference is set, THEN THE Renderer SHALL fall back to the existing solid-color rectangle rendering for the player.
4. WHEN the player is rendered using a Character_Spritesheet, THE Renderer SHALL use the Collision_Box (16×16 pixels, bottom-center of the 24×32 sprite) for tile-based collision detection.
5. WHEN the player is rendered using a Character_Spritesheet, THE Renderer SHALL display the sprite frame corresponding to the player's current Facing_Direction.

### Requirement 4: Player Sprite Animation

**User Story:** As a game creator, I want the player sprite to animate when walking and idle, so that the game feels polished and alive.

#### Acceptance Criteria

1. WHILE the player is moving between tiles, THE Renderer SHALL cycle through the three Animation_Frames for the current Facing_Direction at a configurable animation speed.
2. WHILE the player is stationary, THE Renderer SHALL display the middle (second) Animation_Frame for the current Facing_Direction as the idle pose.
3. WHEN the player begins moving in a direction, THE Renderer SHALL update the Facing_Direction to match the movement direction before starting the walk animation.
4. WHEN the player completes a tile-to-tile move, THE Renderer SHALL return to the idle pose for the current Facing_Direction.

### Requirement 5: NPC Data Model

**User Story:** As a game creator, I want to define NPC instances on my maps, so that I can populate the game world with characters.

#### Acceptance Criteria

1. THE Project_File SHALL store NPC_Instances per map, where each NPC_Instance contains a grid position (x, y), a Facing_Direction, and a reference to a Character_Spritesheet.
2. THE Project_File SHALL support zero or more NPC_Instances per map.
3. WHEN an NPC_Instance references a Character_Spritesheet that does not exist in the Spritesheet_Registry, THE Editor SHALL report a validation error during project loading.
4. FOR ALL valid Project_File states containing NPC_Instances, serializing then deserializing the Project_File SHALL produce equivalent NPC_Instance data (round-trip property).

### Requirement 6: NPC Placement in the Editor

**User Story:** As a game creator, I want to place NPCs on my maps using the editor, so that I can visually design encounters and populate towns.

#### Acceptance Criteria

1. THE Editor SHALL provide an NPC_Placement_Tool accessible from the attribute editor toolbar.
2. WHEN the NPC_Placement_Tool is active and the user clicks a tile, THE Editor SHALL open a dialog allowing the user to select a Character_Spritesheet from the Spritesheet_Registry and choose a Facing_Direction.
3. WHEN the user confirms the NPC placement dialog, THE Editor SHALL create an NPC_Instance at the clicked tile position with the selected spritesheet and Facing_Direction.
4. WHEN the NPC_Placement_Tool is active, THE Editor SHALL render a visual overlay on tiles that contain NPC_Instances, distinguishing them from empty tiles.
5. WHEN the user clicks a tile that already contains an NPC_Instance while the NPC_Placement_Tool is active, THE Editor SHALL open the dialog pre-populated with the existing NPC_Instance data, allowing editing or removal.
6. WHEN the user removes an NPC_Instance via the dialog, THE Editor SHALL delete the NPC_Instance from the map data.
7. THE Editor SHALL support undo and redo for NPC_Instance placement and removal operations.

### Requirement 7: NPC Rendering in the Renderer

**User Story:** As a game creator, I want NPCs to appear in the game world when I play-test my project, so that I can see how the map feels with characters in it.

#### Acceptance Criteria

1. WHEN a map is loaded in the Renderer, THE Renderer SHALL spawn sprite entities for each NPC_Instance on the map, using the referenced Character_Spritesheet.
2. THE Renderer SHALL render each NPC_Instance using the idle pose (middle Animation_Frame) for the NPC's Facing_Direction.
3. THE Renderer SHALL position each NPC sprite at the NPC_Instance's grid position, using the same grid-to-world coordinate conversion as tile sprites.
4. WHEN an NPC_Instance occupies the same layer as the player, THE Renderer SHALL treat the NPC's tile as blocked for player movement collision.
5. WHEN a map transition occurs, THE Renderer SHALL despawn NPC sprites from the previous map and spawn NPC sprites for the new map.

### Requirement 8: NPC Collision Behavior

**User Story:** As a game creator, I want NPCs to block the player when they share the same layer, so that NPCs feel like physical entities in the world.

#### Acceptance Criteria

1. WHILE an NPC_Instance exists on the same layer as the player, THE collision system SHALL treat the NPC's tile position as blocked, preventing the player from moving onto that tile.
2. WHILE an NPC_Instance exists on a different layer than the player, THE collision system SHALL allow the player to move through the NPC's tile position.
3. IF an NPC_Instance is placed on a tile that is already blocked by tile opacity attributes, THEN THE collision system SHALL maintain the blocked state regardless of the NPC's presence.

### Requirement 9: Future NPC Interaction (Deferred)

**User Story:** As a game creator, I want NPCs to eventually support interaction triggers and behaviors, so that the game world feels interactive.

#### Acceptance Criteria

1. THE NPC_Instance data model SHALL be designed to accommodate future extension with event triggers (passive and collision-triggered) without breaking existing NPC_Instance serialization.
2. THE NPC_Instance data model SHALL be designed to accommodate future extension with patrol paths and movement behaviors without breaking existing NPC_Instance serialization.

*Note: Implementation of NPC interaction triggers, dialogue, and patrol behaviors is deferred to a separate specification. This requirement ensures the data model is forward-compatible.*
