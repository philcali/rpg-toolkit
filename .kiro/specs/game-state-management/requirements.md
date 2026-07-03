# Requirements Document

## Introduction

This feature adds game state management with save/load mechanics and application phase transitions to the rpg-toolkit. It introduces an `AppPhase` state enum as the coordination primitive between composable scene plugins, extends the save file format to include player location, adds new event actions for saving and phase transitions, gates the renderer on the `InGame` phase, and introduces a new `rpg-toolkit-scenes` crate with a `TitleScreenPlugin` as the first standard scene.

## Glossary

- **AppPhase**: A Bevy `States` enum defined in `rpg-toolkit-common` representing the current application phase. Variants: `TitleScreen`, `InGame`, `Battle`, `Shop`, `Status`.
- **SaveFile**: The on-disk JSON structure representing persisted game progress, defined in `rpg-toolkit-renderer::save`.
- **EventAction**: An enum in `rpg-toolkit-common::map` representing actions triggered by tile events or NPC interactions.
- **ProjectRendererPlugin**: The Bevy plugin in `rpg-toolkit-renderer` that renders the exploration game world.
- **TitleScreenPlugin**: A Bevy plugin in the new `rpg-toolkit-scenes` crate that renders the title screen with New Game and Continue options.
- **Launcher**: The `rpg-toolkit-launcher` binary that composes all plugins and runs the game.
- **Renderer**: The `rpg-toolkit-renderer` crate providing exploration gameplay systems.
- **ActionQueue**: A Bevy resource in the Renderer that processes event action sequences.

## Requirements

### Requirement 1: AppPhase State Enum

**User Story:** As a game developer composing scene plugins, I want a shared application phase enum, so that plugins can coordinate which systems run during each phase.

#### Acceptance Criteria

1. THE AppPhase enum SHALL be defined in the `rpg-toolkit-common` crate with variants: `TitleScreen`, `InGame`, `Battle`, `Shop`, `Status`, and SHALL be publicly re-exported from the crate root.
2. THE `rpg-toolkit-common` crate SHALL depend on `bevy` (workspace dependency) and THE AppPhase enum SHALL implement Bevy's `States` trait so that Bevy's state-based scheduling can gate systems on specific phases.
3. THE AppPhase enum SHALL derive `Clone`, `Debug`, `PartialEq`, `Eq`, `Hash`, `Serialize`, and `Deserialize`, and SHALL serialize each variant as its name string (e.g., `"TitleScreen"`, `"InGame"`).
4. THE AppPhase enum SHALL implement the `Default` trait with `TitleScreen` as the default variant.

### Requirement 2: SaveFile Location Extension

**User Story:** As a player, I want my save file to record my map position, so that continuing a game restores me to the exact location where I saved.

#### Acceptance Criteria

1. THE SaveFile struct SHALL include a `map_id` field of type `Option<String>` representing the map the player was on when saving, where the string value is a valid MapId (UUID v4 format).
2. THE SaveFile struct SHALL include a `position` field of type `Option<(u32, u32)>` representing the player's grid coordinates (column, row) when saving, where each coordinate value is in the range 0 to 255 inclusive (matching the maximum map dimension of 256 tiles).
3. THE SaveFile struct SHALL include an `elevation` field of type `Option<u32>` representing the player's elevation level when saving.
4. WHEN a save file without location fields is deserialized, THE SaveFile parser SHALL default `map_id`, `position`, and `elevation` to `None` using serde `default` attributes to maintain backward compatibility with existing saves that lack these fields.
5. WHEN a SaveFile with location fields populated is serialized and then deserialized, THE SaveFile parser SHALL produce a SaveFile with `map_id`, `position`, and `elevation` values equivalent to the original (round-trip property).
6. WHEN a SaveFile with all location fields set to `None` is serialized and then deserialized, THE SaveFile parser SHALL produce a SaveFile with `map_id`, `position`, and `elevation` equal to `None` (round-trip property for absent location data).

### Requirement 3: EventAction::SaveGame Variant

**User Story:** As a game designer using the editor, I want a SaveGame event action, so that I can place save points in the game world that persist player progress including location.

#### Acceptance Criteria

1. THE EventAction enum SHALL include a `SaveGame` variant with no additional fields.
2. WHEN the ActionQueue processes a `SaveGame` action, THE Renderer SHALL call the `save_game` function with the current `GameState`, `CurrencyState`, `InventoryState`, `PartyState`, `CharacterProgressState`, `SavePath`, and the player's current `map_id` (from `RendererState.active_map_id`), grid position (`grid_x`, `grid_y`), and elevation (from `PlayerCharacter`).
3. WHEN the ActionQueue processes a `SaveGame` action, THE Renderer SHALL treat it as non-blocking, pop it from the queue, and continue processing subsequent actions in the same frame.
4. WHEN the `save_game` function is called with location data, THE SaveFile SHALL include the `map_id` as a String field, `position` as a pair of u32 grid-tile coordinates (x, y), and `elevation` as a u32 field in the serialized output.
5. IF the save operation fails, THEN THE Renderer SHALL log a warning indicating the failure reason and continue processing the action queue without crashing.
6. IF the `SavePath` resource is not present when a `SaveGame` action is processed, THEN THE Renderer SHALL log a warning and skip the action without crashing.

### Requirement 4: EventAction::ChangePhase Variant

**User Story:** As a game designer, I want a ChangePhase event action, so that events can trigger transitions to other game phases like battles or shops.

#### Acceptance Criteria

1. THE EventAction enum SHALL include a `ChangePhase` variant containing a `phase` field of type `AppPhase`.
2. WHEN the ActionQueue processes a `ChangePhase` action, THE Renderer SHALL transition the Bevy `AppPhase` state to the specified phase.
3. THE `ChangePhase` action SHALL serialize using the existing `#[serde(tag = "type")]` convention with a `phase` field containing the AppPhase variant name as a string, and deserializing a serialized `ChangePhase` action SHALL produce a value equal to the original (round-trip property).
4. WHEN a `ChangePhase` action transitions away from `InGame`, THE Renderer SHALL pause the ActionQueue, preserving remaining actions, and SHALL resume processing from the next unprocessed action when AppPhase returns to `InGame`.
5. IF the ActionQueue processes a `ChangePhase` action whose target phase equals the current AppPhase, THEN THE Renderer SHALL treat the action as a no-op and continue processing the next action in the queue.

### Requirement 5: Renderer Gated on AppPhase::InGame

**User Story:** As a scene plugin author, I want the renderer to only run during the InGame phase, so that other phases can take over rendering without conflicts.

#### Acceptance Criteria

1. WHILE AppPhase is not `InGame`, THE ProjectRendererPlugin update systems SHALL not execute.
2. WHEN AppPhase transitions to `InGame`, THE ProjectRendererPlugin update systems SHALL begin executing on the next frame.
3. THE ProjectRendererPlugin startup systems SHALL execute once during the Bevy `Startup` schedule regardless of AppPhase, so that resources (spritesheets, camera, player entity) are initialized before the phase transitions to `InGame`.
4. WHEN AppPhase transitions away from `InGame`, THE ProjectRendererPlugin SHALL preserve all spawned entities and resources so that they remain intact when AppPhase returns to `InGame`.
5. WHILE AppPhase is not `InGame`, THE ProjectRendererPlugin startup system `fire_initial_map_changed` SHALL defer emitting the `MapChanged` event until AppPhase first enters `InGame`.

### Requirement 6: TitleScreenPlugin

**User Story:** As a player, I want a title screen with New Game and Continue options, so that I can start a new game or resume from a previous save.

#### Acceptance Criteria

1. WHILE AppPhase is `TitleScreen`, THE TitleScreenPlugin SHALL render a "New Game" option and a "Continue" option.
2. WHILE AppPhase is `TitleScreen` and no save file exists at the configured save path, THE TitleScreenPlugin SHALL display the "Continue" option as disabled and not selectable.
3. WHILE AppPhase is `TitleScreen` and a save file exists at the configured save path, THE TitleScreenPlugin SHALL display the "Continue" option as selectable.
4. WHEN the player selects "New Game", THE TitleScreenPlugin SHALL reset all game state resources to their defaults, set the Renderer's active map and position from the project's spawn point, and transition AppPhase to `InGame`.
5. WHEN the player selects "Continue", THE TitleScreenPlugin SHALL load the save file, populate game state resources from the save data, set the Renderer's active map and position from the save file's `map_id` and `position` fields, and transition AppPhase to `InGame`.
6. IF the save file's `map_id` or `position` is `None`, THEN THE TitleScreenPlugin SHALL fall back to the project's spawn point for position restoration.
7. IF the save file cannot be parsed into a valid SaveFile, THEN THE TitleScreenPlugin SHALL treat it as if no save file exists and display the "Continue" option as disabled.
8. IF the project's spawn point is `None` when "New Game" is selected or when falling back from missing save location data, THEN THE TitleScreenPlugin SHALL not transition to `InGame` and SHALL display an error message indicating that no spawn point is configured.
9. THE TitleScreenPlugin SHALL despawn its UI entities when AppPhase transitions away from `TitleScreen`.

### Requirement 7: rpg-toolkit-scenes Crate

**User Story:** As a toolkit maintainer, I want standard scene plugins in a dedicated crate, so that the launcher can compose them without coupling scene logic to the renderer or editor.

#### Acceptance Criteria

1. THE workspace SHALL include a new `rpg-toolkit-scenes` crate in the `crates/` directory, registered in the workspace `Cargo.toml` members list.
2. THE `rpg-toolkit-scenes` crate SHALL depend on `rpg-toolkit-common` (via relative path) for shared types and `bevy` (via workspace reference) for plugin infrastructure.
3. THE `rpg-toolkit-scenes` crate SHALL publicly re-export the `TitleScreenPlugin` struct from its crate root module so that downstream crates can import it as `rpg_toolkit_scenes::TitleScreenPlugin`.
4. THE `rpg-toolkit-scenes` crate SHALL not depend on `rpg-toolkit-renderer` or `rpg-toolkit-editor` either directly or transitively.
5. WHEN the workspace is built, THE `rpg-toolkit-scenes` crate SHALL compile without errors.

### Requirement 8: Launcher Composition

**User Story:** As a game developer, I want the launcher to compose all plugins including the title screen, so that running the launcher provides the full game experience with phase transitions.

#### Acceptance Criteria

1. THE Launcher SHALL initialize AppPhase to `TitleScreen` as the starting state.
2. THE Launcher SHALL add the `TitleScreenPlugin` from `rpg-toolkit-scenes`.
3. THE Launcher SHALL add the `ProjectRendererPlugin`.
4. THE Launcher SHALL NOT load save file data into game state resources (GameState, CurrencyState, InventoryState, PartyState, CharacterProgressState) at startup; those resources SHALL be inserted with default (empty) values so that plugins can operate without panicking.
5. THE Launcher SHALL accept the `--save` argument to configure the save file path, inserting the `SavePath` resource for use by plugins.
6. IF the `--save` argument is not provided, THEN THE Launcher SHALL derive a default save file path relative to the project's location (e.g., `save.json` alongside the project file or directory).
7. THE Launcher SHALL continue to load the project file data and insert the `RendererProjectData` resource at startup, so that project metadata is available regardless of AppPhase.

### Requirement 9: Editor Support for New Actions

**User Story:** As a game designer using the editor, I want to configure SaveGame and ChangePhase actions in the event editor, so that I can place save points and phase transitions in my game.

#### Acceptance Criteria

1. THE Editor's action editor SHALL display "Save Game" as a selectable action type in the action type list.
2. THE Editor's action editor SHALL display "Change Phase" as a selectable action type in the action type list.
3. WHEN "Save Game" is selected, THE Editor's action editor SHALL require no additional configuration fields and SHALL allow the designer to add the action immediately.
4. WHEN "Change Phase" is selected, THE Editor's action editor SHALL present a dropdown listing all AppPhase variants (`TitleScreen`, `InGame`, `Battle`, `Shop`, `Status`) to choose the target phase.
5. WHEN the designer edits an existing SaveGame or ChangePhase action in the action list, THE Editor's action editor SHALL populate the form fields from the existing action data.
6. THE Editor SHALL serialize configured SaveGame and ChangePhase actions into the project file using the serde tagged JSON format (`"type"` field) consistent with the existing EventAction serialization consumed by the Renderer.

### Requirement 10: save_game Function Extension

**User Story:** As a developer, I want the save_game function to accept location parameters, so that it can persist the player's position alongside other game state.

#### Acceptance Criteria

1. THE `save_game` function SHALL accept additional parameters for `map_id` (Option<&str>), `position` (Option<(u32, u32)>), and `elevation` (Option<u32>).
2. WHEN location parameters are provided, THE `save_game` function SHALL include the location data in the serialized SaveFile.
3. WHEN location parameters are `None`, THE `save_game` function SHALL omit location fields from the serialized SaveFile (maintaining backward compatibility with save files that have no location).
4. FOR ALL combinations of game state resources and location parameters, serializing via `save_game` then deserializing SHALL produce a SaveFile equivalent to the input data (round-trip property).
