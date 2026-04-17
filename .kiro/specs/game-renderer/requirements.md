# Requirements Document

## Introduction

The RPG Toolkit is being restructured into a Cargo workspace monorepo with three crates: `rpg-toolkit-common` (shared data types), `rpg-toolkit-renderer` (a Bevy plugin for rendering playable game worlds), and `rpg-toolkit-editor` (the existing editor, out of scope for this spec). This spec covers extracting the pure data types into the common crate, building a `ProjectRendererPlugin` that any Bevy app can add to render a project as a playable game, and providing a minimal launcher binary as a test harness. All editor integration (Play/Stop mode switching, UI panel hiding, editor state preservation) is explicitly out of scope.

## Glossary

- **Workspace**: The Cargo workspace containing the three crates: `rpg-toolkit-common`, `rpg-toolkit-renderer`, and `rpg-toolkit-editor`.
- **Common_Crate**: The `rpg-toolkit-common` library crate containing shared, serializable data types with no Bevy dependency (except optionally `bevy_math`).
- **Renderer_Crate**: The `rpg-toolkit-renderer` library crate containing the `ProjectRendererPlugin` Bevy plugin.
- **Editor_Crate**: The `rpg-toolkit-editor` binary crate containing the existing editor application (out of scope for this spec).
- **ProjectRendererPlugin**: A standard Bevy `Plugin` that renders a loaded project as a playable game world with player movement, collision, and event triggers.
- **Launcher_Binary**: A minimal binary in the workspace that loads a `.rpg` project file from a CLI argument, loads tileset images, adds the `ProjectRendererPlugin`, and runs the Bevy app.
- **ProjectFile**: The on-disk JSON format representing a complete project (maps, tileset metadata, spawn point).
- **MapData**: The complete data for a single map including dimensions, tile size, layers, and tile attributes.
- **TilesetMeta**: Metadata about a tileset (file path, tile dimensions, grid dimensions) without runtime texture handles.
- **TileRef**: A reference to a specific tile within a specific tileset (tileset ID, column, row).
- **Layer**: A single map layer containing a 2D tile grid and a parallel tile attribute grid.
- **TileAttributes**: Per-tile data including an opacity flag and an event trigger action list.
- **TileAttributeLayer**: A 2D grid of `TileAttributes` parallel to a layer's tile grid.
- **SpawnPoint**: A project-level coordinate (map ID, x, y) designating where the player character appears.
- **EventAction**: An enum of actions that fire when the player steps on a tile (currently only `JumpTo`).
- **JumpTo_Action**: An `EventAction::JumpTo` variant that transitions the player to a target map at a target coordinate.
- **Player_Character**: A sprite entity controlled by the user via keyboard input during gameplay. Rendered as a solid colored rectangle placeholder until character spritesheet support is implemented in a future feature.
- **Opacity_Tile**: A tile whose `TileAttributes.opacity` flag is `true`, indicating the tile blocks player movement.
- **Active_Map**: The map currently being rendered by the ProjectRendererPlugin.
- **Tile_Grid**: The discrete coordinate system of a map where each cell is one tile wide and one tile tall.
- **Camera**: The Bevy 2D camera that follows the Player_Character during gameplay.

## Requirements

### Requirement 1: Common Crate Extraction

**User Story:** As a developer, I want the shared data types extracted into a standalone crate with minimal dependencies, so that both the renderer and editor can depend on them without pulling in unnecessary libraries.

#### Acceptance Criteria

1. THE Common_Crate SHALL contain the following types extracted from `src/data/`: `MapData`, `MapId`, `TilesetId`, `TileRef`, `Layer`, `TileAttributeLayer`, `TileAttributes`, `EventAction`, `SpawnPoint`, `TilesetMeta`, and `ProjectFile`.
2. THE Common_Crate SHALL depend only on `serde`, `serde_json`, `thiserror`, and `uuid` as runtime dependencies.
3. THE Common_Crate SHALL NOT depend on `bevy` as a runtime dependency.
4. THE Common_Crate SHALL NOT contain editor-specific types including `EditorState`, `EditorTool`, `EditorMode`, `AttributeTool`, `EditCommand`, `EditCommandKind`, `UndoHistory`, `StampBrushSelection`, `LineDragState`, and `EditorError`.
5. THE Common_Crate SHALL define its own error type for serialization and validation failures independent of `EditorError`.
6. THE Common_Crate SHALL preserve all existing `Serialize` and `Deserialize` implementations so that existing `.rpg` project files remain compatible.
7. THE Common_Crate SHALL preserve the `MapData::validate` method for validating map data after deserialization.
8. THE Common_Crate SHALL preserve the `ProjectFile::serialize` and `ProjectFile::deserialize` methods including all validation logic (map validation, tileset reference checking, JumpTo target warnings).

### Requirement 2: Workspace Structure

**User Story:** As a developer, I want the project organized as a Cargo workspace, so that the crates can be developed, tested, and versioned together.

#### Acceptance Criteria

1. THE Workspace SHALL contain a root `Cargo.toml` declaring workspace members for `rpg-toolkit-common`, `rpg-toolkit-renderer`, and `rpg-toolkit-editor`.
2. THE Renderer_Crate SHALL depend on the Common_Crate and `bevy`.
3. THE Editor_Crate SHALL depend on the Common_Crate, the Renderer_Crate, `bevy`, and `bevy_egui`.
4. THE Workspace SHALL contain a Launcher_Binary that depends on the Common_Crate, the Renderer_Crate, and `bevy`.

### Requirement 3: ProjectRendererPlugin Map Rendering

**User Story:** As a developer, I want the renderer plugin to display my maps with all visible layers composited correctly, so that the game world looks as I designed it in the editor.

#### Acceptance Criteria

1. WHEN the ProjectRendererPlugin is added to a Bevy App and project data is provided, THE ProjectRendererPlugin SHALL render the Active_Map identified by the SpawnPoint's map ID.
2. THE ProjectRendererPlugin SHALL render all layers of the Active_Map in ascending index order so that higher layers appear on top of lower layers.
3. THE ProjectRendererPlugin SHALL resolve each TileRef to the correct tileset texture and atlas index using the provided tileset registry.
4. THE ProjectRendererPlugin SHALL position each tile sprite at the correct world coordinate based on the tile's grid position and the map's tile dimensions.
5. THE ProjectRendererPlugin SHALL render only layers whose `visible` flag is `true`.
6. WHEN the Active_Map changes via a JumpTo_Action, THE ProjectRendererPlugin SHALL despawn all tile sprites from the previous map and spawn tile sprites for the new Active_Map.

### Requirement 4: Player Character Spawning

**User Story:** As a developer, I want a player character to appear at the designated spawn point when the game starts, so that I can test navigation from the intended starting location.

#### Acceptance Criteria

1. WHEN the ProjectRendererPlugin initializes, THE ProjectRendererPlugin SHALL spawn the Player_Character at the Tile_Grid coordinates specified by the project's SpawnPoint.
2. THE ProjectRendererPlugin SHALL position the Player_Character sprite at the world-space center of the spawn tile.
3. THE ProjectRendererPlugin SHALL render the Player_Character above all map layers.
4. THE ProjectRendererPlugin SHALL render the Player_Character as a solid colored rectangle sized to one tile. Character spritesheet rendering is out of scope and deferred to a future feature.
5. IF the SpawnPoint references a tile coordinate outside the Active_Map bounds, THEN THE ProjectRendererPlugin SHALL clamp the Player_Character position to the nearest valid Tile_Grid cell.

### Requirement 5: Player Movement

**User Story:** As a developer, I want to move the player character around the map using keyboard input, so that I can test walkability and map layout.

#### Acceptance Criteria

1. THE ProjectRendererPlugin SHALL move the Player_Character one tile in the corresponding direction when the user presses an arrow key or WASD key (W=up, A=left, S=down, D=right).
2. THE ProjectRendererPlugin SHALL use grid-based (tile-to-tile) movement for the Player_Character.
3. THE ProjectRendererPlugin SHALL animate the Player_Character's position smoothly between tiles over a configurable duration.
4. WHILE the Player_Character is animating between tiles, THE ProjectRendererPlugin SHALL ignore additional movement input.
5. THE ProjectRendererPlugin SHALL prevent the Player_Character from moving outside the Active_Map boundaries.

### Requirement 6: Collision Detection

**User Story:** As a developer, I want opacity-flagged tiles to block player movement, so that I can test collision boundaries I've painted in the editor.

#### Acceptance Criteria

1. WHEN the Player_Character attempts to move to a Tile_Grid cell, THE ProjectRendererPlugin SHALL check the Opacity_Tile flag on all layers at the target cell.
2. IF any layer at the target cell has an Opacity_Tile flag set to `true`, THEN THE ProjectRendererPlugin SHALL prevent the Player_Character from moving to that cell.
3. THE ProjectRendererPlugin SHALL perform the collision check before starting the movement animation.

### Requirement 7: Event Trigger Execution

**User Story:** As a developer, I want event triggers to fire when the player steps on a tile, so that I can test map transitions and other scripted behaviors.

#### Acceptance Criteria

1. WHEN the Player_Character arrives at a Tile_Grid cell, THE ProjectRendererPlugin SHALL check all layers at that cell for EventAction entries.
2. WHEN a JumpTo_Action is found in the event trigger list, THE ProjectRendererPlugin SHALL transition the Active_Map to the target map specified by `target_map_id`.
3. WHEN a JumpTo_Action is executed, THE ProjectRendererPlugin SHALL reposition the Player_Character at the Tile_Grid cell specified by `target_x` and `target_y`.
4. IF a JumpTo_Action references a `target_map_id` that does not exist in the project, THEN THE ProjectRendererPlugin SHALL log a warning and ignore the action.
5. IF a JumpTo_Action specifies target coordinates outside the target map bounds, THEN THE ProjectRendererPlugin SHALL clamp the Player_Character position to the nearest valid Tile_Grid cell.

### Requirement 8: Game Camera

**User Story:** As a developer, I want the camera to follow the player character, so that I can see the surrounding map as I navigate.

#### Acceptance Criteria

1. THE Camera SHALL center on the Player_Character's world position each frame.
2. THE Camera SHALL update its position to track the Player_Character during movement animation.
3. THE Camera SHALL use a fixed zoom level appropriate for the Active_Map's tile size.
4. THE Camera SHALL clamp its position so that areas outside the Active_Map boundaries are not visible when the map is larger than the viewport.

### Requirement 9: Plugin API Surface

**User Story:** As an advanced developer, I want the renderer plugin to expose Bevy resources, components, and events, so that I can hook into and extend the game world with my own systems.

#### Acceptance Criteria

1. THE ProjectRendererPlugin SHALL accept project data (maps, tileset metadata, and spawn point from ProjectFile) and tileset texture handles as input via Bevy resources.
2. THE ProjectRendererPlugin SHALL expose the Active_Map state as a readable Bevy resource so that consumer systems can query the current map.
3. THE ProjectRendererPlugin SHALL expose the Player_Character's grid position as a readable Bevy component so that consumer systems can query the player location.
4. THE ProjectRendererPlugin SHALL expose a Bevy event when the Active_Map changes (via JumpTo_Action) so that consumer systems can react to map transitions.
5. THE ProjectRendererPlugin SHALL expose a Bevy event when the Player_Character completes a move to a new tile so that consumer systems can react to player movement.
6. THE ProjectRendererPlugin SHALL function as a standard Bevy Plugin that can be added to any Bevy App via `app.add_plugins(ProjectRendererPlugin)`.

### Requirement 10: Launcher Binary

**User Story:** As a developer, I want a minimal launcher that loads a project file and runs the renderer, so that I can test the plugin without the full editor.

#### Acceptance Criteria

1. THE Launcher_Binary SHALL accept a file path to a `.rpg` project file as a command-line argument.
2. WHEN launched, THE Launcher_Binary SHALL deserialize the ProjectFile from the specified path using the Common_Crate's deserialization logic.
3. WHEN launched, THE Launcher_Binary SHALL load all tileset images referenced by the ProjectFile's TilesetMeta entries relative to the project file's directory.
4. WHEN launched, THE Launcher_Binary SHALL add the ProjectRendererPlugin to a Bevy App with the loaded project data and tileset textures.
5. IF the specified project file path does not exist, THEN THE Launcher_Binary SHALL exit with a descriptive error message.
6. IF the project file fails deserialization or validation, THEN THE Launcher_Binary SHALL exit with a descriptive error message.
7. IF the project contains no spawn point, THEN THE Launcher_Binary SHALL exit with a descriptive error message.

### Requirement 11: Serialization Round-Trip

**User Story:** As a developer, I want confidence that serializing and deserializing a project file produces identical data, so that saved projects are faithfully preserved.

#### Acceptance Criteria

1. FOR ALL valid ProjectFile values, serializing to JSON then deserializing SHALL produce an equivalent ProjectFile (round-trip property).
2. FOR ALL valid ProjectFile values, the set of maps, tilesets, spawn point, and tile attributes accessible after deserialization SHALL be identical to those before serialization.
3. THE Common_Crate SHALL produce identical JSON output for the same ProjectFile input across serialization calls (deterministic serialization within a single run).
