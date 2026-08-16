# Requirements Document

## Introduction

The Game Intro Narration feature enables game designers to choreograph a cinematic opening sequence that plays automatically when a player starts a new game. Rather than introducing a separate cutscene screen or AppPhase, the system leverages the existing `EventAction` infrastructure: the intro is a list of `EventAction` items stored on the `ProjectManifest` that fire automatically after the player spawns. Four new `EventAction` variants (`MoveEntity`, `CameraFollow`, `CameraPan`, `Wait`) are added to support cinematic choreography, and the editor provides a section in project settings to compose the intro sequence using the existing action editor.

## Glossary

- **ProjectManifest**: The on-disk manifest structure (`manifest.json`) that stores all project metadata including maps, tilesets, registries, and now intro events.
- **EventAction**: A tagged enum representing a single action within an event trigger sequence, serialized as JSON with a `type` discriminator.
- **ActionQueue**: A runtime resource that holds a sequence of `EventAction` items and processes them front-to-back, blocking on blocking actions.
- **WaitingFor**: A runtime enum indicating what the `ActionQueue` is currently waiting on before advancing.
- **NewGameFlag**: A marker resource inserted by the title screen to signal that the current transition to InGame is a fresh new game (not a save load).
- **IntroEventsActive**: A marker resource indicating that intro events are currently playing and player movement input is suppressed.
- **EntityTarget**: A tagged enum distinguishing between the player character (`Player`) and a specific NPC (`Npc { npc_id }`) as the target of movement or camera actions.
- **NPC**: A non-player character placed on a map, identified by a string `npc_id` matching an entry in the map's `npcs` list.
- **ActionEditor**: The existing editor UI component for composing ordered lists of `EventAction` items with type-specific form fields.
- **SpawnPoint**: The project-wide starting position (map, x, y) where the player entity is placed on new game.

## Requirements

### Requirement 1: Intro Events Data Model

**User Story:** As a game designer, I want to store an ordered list of intro event actions on the project manifest, so that the runtime can play them automatically on new game start.

#### Acceptance Criteria

1. THE ProjectManifest SHALL include an `intro_events` field of type `Option<Vec<EventAction>>`.
2. WHEN the `intro_events` field is absent from the manifest JSON, THE ProjectManifest deserialization SHALL default the field to `None`.
3. WHEN the `intro_events` field is present with a list of actions, THE ProjectManifest SHALL store the actions in their declared order.
4. THE ProjectManifest SHALL reject deserialization of `intro_events` containing more than 100 actions, returning an error indicating the maximum was exceeded.
5. WHEN the `intro_events` field is present as an empty array, THE ProjectManifest SHALL deserialize it as `Some` containing an empty `Vec`.

### Requirement 2: MoveEntity Event Action

**User Story:** As a game designer, I want to move either the player character or an NPC to a target grid position with a tile-by-tile walk animation, so that I can choreograph entity movement in cutscenes.

#### Acceptance Criteria

1. THE EventAction enum SHALL include a `MoveEntity` variant with fields: `target` (EntityTarget), `target_x` (u32), `target_y` (u32), and `speed` (f32).
2. THE `EntityTarget` enum SHALL have two variants: `Player` (representing the player character) and `Npc { npc_id: String }` (representing a specific NPC by identifier).
3. THE `MoveEntity` variant SHALL be a blocking action: the ActionQueue SHALL wait until the entity reaches the target position before advancing.
4. THE `MoveEntity` action SHALL animate the entity walking tile-by-tile to the target position (not teleporting).
5. WHEN the `speed` field is omitted from the JSON, THE `MoveEntity` deserialization SHALL default `speed` to 2.0 tiles per second.
6. THE `MoveEntity` deserialization SHALL reject a `speed` value outside the range 0.1 to 10.0 inclusive, returning an error identifying the violated constraint.
7. THE `MoveEntity` deserialization SHALL reject an `Npc` target with an empty `npc_id` string, returning an error indicating the field must not be empty.
8. WHEN a `MoveEntity` action references a nonexistent `npc_id` at runtime, THE ActionQueue SHALL log a warning and skip the action, advancing to the next action.
9. WHEN a `MoveEntity` target position is unreachable due to blocked tiles, THE entity SHALL walk as close as possible to the target, then the action SHALL complete.

### Requirement 3: CameraFollow Event Action

**User Story:** As a game designer, I want to switch which entity the camera follows during the intro sequence, so that I can create cinematic tracking shots that follow NPCs or the player.

#### Acceptance Criteria

1. THE EventAction enum SHALL include a `CameraFollow` variant with a `target` field of type `EntityTarget`.
2. THE `CameraFollow` variant SHALL be a non-blocking action: the ActionQueue SHALL advance to the next action immediately after setting the camera follow target.
3. WHEN a `CameraFollow` action is executed, THE camera SHALL begin tracking the specified entity (player or NPC).
4. THE `CameraFollow` setting SHALL be sticky: it SHALL persist until another `CameraFollow` action changes the target.
5. WHEN a `CameraFollow` action targets `Player`, THE camera SHALL revert to following the player character (default behavior).
6. THE `CameraFollow` deserialization SHALL reject an `Npc` target with an empty `npc_id` string, returning an error indicating the field must not be empty.
7. WHEN a `CameraFollow` action references a nonexistent `npc_id` at runtime, THE system SHALL log a warning and skip the action, advancing to the next action.

### Requirement 4: CameraPan Event Action

**User Story:** As a game designer, I want to smoothly pan the camera to a target grid position during the intro sequence, so that I can direct the player's attention to specific map locations.

#### Acceptance Criteria

1. THE EventAction enum SHALL include a `CameraPan` variant with fields: `target_x` (u32), `target_y` (u32), and `duration` (f32).
2. THE `CameraPan` variant SHALL be a blocking action: the ActionQueue SHALL wait until the pan duration elapses before advancing.
3. THE `CameraPan` deserialization SHALL reject a `duration` value outside the range 0.1 to 10.0 inclusive, returning an error identifying the violated constraint.
4. WHILE a `CameraPan` action is active, THE interpolated camera position SHALL remain within the axis-aligned bounding box defined by the start position and the target position.
5. WHEN the `CameraPan` target is outside the map bounds at runtime, THE system SHALL clamp the target to the map bounds and log a warning.
6. WHEN a `CameraPan` completes, THE camera SHALL remain at the pan target position until a subsequent `CameraFollow` action redirects it.

### Requirement 5: Wait Event Action

**User Story:** As a game designer, I want to pause the action queue for a specified duration, so that I can add timing gaps between intro sequence actions.

#### Acceptance Criteria

1. THE EventAction enum SHALL include a `Wait` variant with a `duration` field (f32) representing seconds.
2. THE `Wait` variant SHALL be a blocking action: the ActionQueue SHALL wait until the specified duration elapses before advancing.
3. THE `Wait` deserialization SHALL reject a `duration` value outside the range 0.1 to 30.0 inclusive, returning an error identifying the violated constraint.

### Requirement 6: Game Start Event Trigger

**User Story:** As a game designer, I want the intro event sequence to fire automatically after the player spawns on a new game, so that the opening narration plays without manual intervention.

#### Acceptance Criteria

1. WHEN the AppPhase transitions to InGame with a NewGameFlag resource present and no ActionQueue exists, THE system SHALL insert the `intro_events` list into a new ActionQueue.
2. WHEN the intro_events ActionQueue is inserted, THE system SHALL insert an IntroEventsActive marker resource.
3. WHEN the intro_events ActionQueue is inserted, THE system SHALL remove the NewGameFlag resource.
4. WHILE the IntroEventsActive marker is present, THE system SHALL suppress player movement input.
5. WHEN all actions in the intro ActionQueue have completed, THE system SHALL remove the IntroEventsActive marker and restore normal player control.
6. WHEN the NewGameFlag is absent during the InGame transition, THE system SHALL not insert any intro ActionQueue.
7. WHEN an ActionQueue already exists during the InGame transition, THE system SHALL not insert the intro ActionQueue, preserving the existing queue.
8. WHEN the `intro_events` field is `None` or contains an empty list, THE system SHALL not insert any ActionQueue and the player SHALL gain control immediately.

### Requirement 7: Player Skip Controls

**User Story:** As a player, I want to skip the intro sequence by pressing Escape, so that I can get to gameplay quickly on repeat playthroughs.

#### Acceptance Criteria

1. WHILE the IntroEventsActive marker is present, WHEN the player presses the Escape key, THE system SHALL drain all remaining actions from the ActionQueue.
2. WHEN the intro is skipped via Escape, THE system SHALL restore the camera to follow the player entity (equivalent to `CameraFollow` with target `Player`).
3. WHEN the intro is skipped via Escape, THE system SHALL remove the IntroEventsActive marker and restore normal player control.

### Requirement 8: Intro Events Editor

**User Story:** As a game designer, I want to compose and edit the intro event sequence in the editor's project settings, so that I can author the opening narration without manually editing JSON.

#### Acceptance Criteria

1. THE editor project settings panel SHALL include a collapsible "Game Start Events" section for editing intro events.
2. THE "Game Start Events" section SHALL reuse the existing ActionEditor component to add, edit, remove, and reorder EventAction items.
3. THE ActionEditor SHALL support the `MoveEntity`, `CameraFollow`, `CameraPan`, and `Wait` action types in its type selector.
4. THE ActionEditor SHALL provide appropriate form fields for each new action type: entity target selector (Player or NPC ID), target X/Y grid inputs, speed slider for MoveEntity; entity target selector for CameraFollow; target X/Y grid inputs and duration slider for CameraPan; duration slider for Wait.
5. WHEN the designer saves project settings, THE editor SHALL write the intro events list to the `intro_events` field of the ProjectManifest.

### Requirement 9: Serialization

**User Story:** As a developer, I want the new EventAction variants to serialize and deserialize correctly, so that project data is reliably persisted and loaded.

#### Acceptance Criteria

1. FOR ALL valid `EventAction` values including `MoveEntity`, `CameraFollow`, `CameraPan`, and `Wait` variants with fields within valid ranges, serializing to JSON then deserializing SHALL produce a value structurally equal to the original (round-trip property).
2. WHEN a `MoveEntity` action is serialized to JSON, THE output SHALL include a `"type": "MoveEntity"` discriminator tag.
3. WHEN a `CameraFollow` action is serialized to JSON, THE output SHALL include a `"type": "CameraFollow"` discriminator tag.
4. WHEN a `CameraPan` action is serialized to JSON, THE output SHALL include a `"type": "CameraPan"` discriminator tag.
5. WHEN a `Wait` action is serialized to JSON, THE output SHALL include a `"type": "Wait"` discriminator tag.
6. WHEN a manifest containing `intro_events` with mixed action types is serialized and deserialized, THE round-trip SHALL preserve the action order and all field values.
7. WHEN deserialization encounters a new action variant with invalid field values, THE system SHALL return a descriptive error identifying the specific violated constraint.
