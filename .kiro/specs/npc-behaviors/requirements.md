# Requirements Document

## Introduction

This feature activates the NPC behavior capabilities that were deferred in the character-spritesheets specification (Requirement 9). The existing NPC data model already includes forward-compatible fields for `event_triggers: Vec<EventAction>` and `patrol_path: Vec<(u32, u32)>`, but neither field is read or acted upon at runtime. This specification covers three areas: (1) NPC patrol movement along waypoint paths with walk animation, (2) NPC event triggers fired on player collision or proximity, and (3) editor tooling for configuring patrol paths and event triggers on placed NPCs. The goal is to make NPCs feel like living entities in the game world — walking routes, facing the player, and initiating dialogue or map transitions when approached.

## Glossary

- **NPC_Instance**: A per-map entity placed on a specific tile, referencing a Character_Spritesheet, a Facing_Direction, an event trigger list, and patrol path data. Defined in `rpg-toolkit-common::spritesheet`.
- **Patrol_Path**: An ordered list of waypoint grid positions that an NPC_Instance walks between during gameplay.
- **Patrol_Mode**: The behavior when an NPC reaches the end of its Patrol_Path: `Loop` (return to first waypoint), `PingPong` (reverse direction), or `OneShot` (stop at the last waypoint).
- **Patrol_Config**: The complete patrol behavior configuration for an NPC_Instance, including the Patrol_Path, Patrol_Mode, movement speed, and pause duration at each waypoint.
- **NPC_Movement_Speed**: The duration in seconds for an NPC to move from one tile to an adjacent tile. Analogous to the player's `MovementConfig::move_duration`.
- **Waypoint_Pause**: The duration in seconds an NPC pauses at each waypoint before continuing to the next.
- **NPC_Sprite_State**: A per-NPC component tracking facing direction, animation frame, animation timer, and movement state — analogous to the player's `PlayerSpriteState`.
- **Event_Trigger**: An `EventAction` (ShowDialog or JumpTo) associated with an NPC_Instance, fired when the player interacts with or collides with the NPC.
- **Trigger_Mode**: The condition under which an NPC's Event_Triggers fire: `Collision` (player attempts to move onto the NPC's tile) or `Interaction` (player presses an action key while facing an adjacent NPC).
- **Action_Queue**: The existing renderer resource that processes a sequence of `EventAction` entries (ShowDialog, JumpTo) one at a time.
- **Renderer**: The rpg-toolkit-renderer crate that renders the project as a playable game world.
- **Editor**: The rpg-toolkit-editor crate providing map editing, tileset management, and attribute tooling.
- **Facing_Direction**: One of four cardinal directions (Down, Left, Right, Up) determining which sprite row to display.
- **Animation_Frame**: One of three sequential images in a walk cycle for a given Facing_Direction.
- **Character_Spritesheet**: A project-level image asset containing a 3×4 grid of animation frames (72×128 pixels).

## Requirements

### Requirement 1: Patrol Path Data Model

**User Story:** As a game creator, I want to define patrol paths with configurable behavior for NPCs, so that I can create NPCs that walk predefined routes.

#### Acceptance Criteria

1. THE NPC_Instance data model SHALL store an optional Patrol_Config containing a Patrol_Path (ordered list of waypoint grid positions), a Patrol_Mode (Loop, PingPong, or OneShot), an NPC_Movement_Speed (f32 seconds per tile, default 0.3), and a Waypoint_Pause (f32 seconds, default 0.5).
2. WHEN a Patrol_Config is absent or contains an empty Patrol_Path, THE Renderer SHALL treat the NPC_Instance as stationary.
3. FOR ALL valid NPC_Instance states containing a Patrol_Config, serializing the NPC_Instance to JSON and deserializing the result SHALL produce an equivalent NPC_Instance (round-trip property).
4. WHEN a project file saved before this feature is loaded, THE deserializer SHALL produce NPC_Instances with no Patrol_Config, preserving backward compatibility via `serde(default)`.

### Requirement 2: NPC Patrol Movement

**User Story:** As a game creator, I want NPCs to walk along their patrol paths during gameplay, so that the game world feels alive with moving characters.

#### Acceptance Criteria

1. WHILE an NPC_Instance has a non-empty Patrol_Path, THE Renderer SHALL move the NPC from its current waypoint toward the next waypoint in the Patrol_Path, one tile at a time along the shortest grid-aligned path (horizontal then vertical, or vertical then horizontal).
2. WHEN an NPC moves between tiles, THE Renderer SHALL animate the movement over NPC_Movement_Speed seconds using linear interpolation, matching the player's tile-to-tile movement system.
3. WHEN an NPC arrives at a waypoint, THE Renderer SHALL pause the NPC for Waypoint_Pause seconds before moving toward the next waypoint.
4. WHEN an NPC reaches the last waypoint and Patrol_Mode is Loop, THE Renderer SHALL set the next target to the first waypoint in the Patrol_Path.
5. WHEN an NPC reaches the last waypoint and Patrol_Mode is PingPong, THE Renderer SHALL reverse the Patrol_Path traversal direction.
6. WHEN an NPC reaches the last waypoint and Patrol_Mode is OneShot, THE Renderer SHALL stop the NPC at the last waypoint and return the NPC to its idle pose.
7. WHILE an NPC is moving between tiles, THE Renderer SHALL update the NPC's grid position in the collision system to the destination tile at the start of the move, preventing the player from moving onto the destination tile during the animation.

### Requirement 3: NPC Walk Animation

**User Story:** As a game creator, I want NPCs to display walk animations while patrolling, so that their movement looks natural and consistent with the player character.

#### Acceptance Criteria

1. WHILE an NPC is moving between tiles, THE Renderer SHALL cycle through the three Animation_Frames for the NPC's current Facing_Direction using the same `walk_animation_frame` function used by the player.
2. WHEN an NPC begins moving toward an adjacent tile, THE Renderer SHALL update the NPC's Facing_Direction to match the movement direction (Up, Down, Left, or Right).
3. WHILE an NPC is stationary (idle or paused at a waypoint), THE Renderer SHALL display the middle Animation_Frame (frame index 1) for the NPC's current Facing_Direction.
4. THE Renderer SHALL use the existing `sprite_atlas_index` function to compute the correct texture atlas index for each NPC's Facing_Direction and Animation_Frame combination.
5. THE Renderer SHALL use a per-NPC NPC_Sprite_State component to track each NPC's animation timer, current frame, facing direction, and movement state independently from the player and other NPCs.

### Requirement 4: NPC Dynamic Collision

**User Story:** As a game creator, I want moving NPCs to block the player at their current position, so that NPCs remain solid entities even while patrolling.

#### Acceptance Criteria

1. THE collision system SHALL check NPC positions dynamically each frame rather than relying on the static positions stored in the map data.
2. WHILE an NPC is animating between tiles, THE collision system SHALL treat the NPC's destination tile as blocked and the NPC's origin tile as unblocked.
3. WHEN an NPC's patrol path would move the NPC onto a tile occupied by the player, THE Renderer SHALL pause the NPC's patrol until the tile is vacated.
4. WHEN an NPC's patrol path would move the NPC onto a tile blocked by opacity attributes or another NPC, THE Renderer SHALL pause the NPC's patrol until the tile is unblocked.

### Requirement 5: NPC Event Trigger System

**User Story:** As a game creator, I want NPCs to trigger events when the player interacts with them, so that NPCs can initiate dialogue and other game actions.

#### Acceptance Criteria

1. THE NPC_Instance data model SHALL store a Trigger_Mode (Collision or Interaction, default Interaction) alongside the existing `event_triggers: Vec<EventAction>` field.
2. WHEN Trigger_Mode is Collision and the player attempts to move onto an NPC's tile, THE Renderer SHALL fire the NPC's event_triggers into the Action_Queue instead of blocking the move silently.
3. WHEN Trigger_Mode is Interaction and the player presses the action key while facing an adjacent NPC, THE Renderer SHALL fire the NPC's event_triggers into the Action_Queue.
4. WHEN an NPC's event_triggers list is empty, THE Renderer SHALL apply the default collision behavior (block the player's movement) regardless of Trigger_Mode.
5. IF an Action_Queue is already active when an NPC trigger fires, THEN THE Renderer SHALL ignore the new trigger until the current Action_Queue completes.
6. WHEN an NPC trigger fires via Interaction mode, THE Renderer SHALL update the NPC's Facing_Direction to face toward the player before executing the event_triggers.

### Requirement 6: NPC Event Trigger Data Model

**User Story:** As a game creator, I want the NPC event trigger configuration to be saved in the project file, so that trigger settings persist across editor sessions.

#### Acceptance Criteria

1. THE NPC_Instance data model SHALL store the Trigger_Mode as a serializable field with a default value of Interaction.
2. FOR ALL valid NPC_Instance states containing event_triggers and a Trigger_Mode, serializing to JSON and deserializing SHALL produce an equivalent NPC_Instance (round-trip property).
3. WHEN a project file saved before this feature is loaded, THE deserializer SHALL produce NPC_Instances with Trigger_Mode defaulting to Interaction and event_triggers defaulting to an empty list, preserving backward compatibility.

### Requirement 7: Editor Patrol Path Configuration

**User Story:** As a game creator, I want to define NPC patrol paths visually in the editor, so that I can design NPC movement routes on the map.

#### Acceptance Criteria

1. WHEN the user selects an NPC_Instance in the Editor via the NPC_Placement_Tool, THE Editor SHALL display a patrol path configuration panel showing the current waypoints, Patrol_Mode, NPC_Movement_Speed, and Waypoint_Pause.
2. WHEN the user clicks map tiles while the patrol path configuration panel is open, THE Editor SHALL append the clicked tile position as a new waypoint to the Patrol_Path.
3. WHEN the user removes a waypoint from the patrol path configuration panel, THE Editor SHALL remove the waypoint from the Patrol_Path and update the display.
4. THE Editor SHALL render the Patrol_Path as a connected line overlay on the map canvas, with numbered markers at each waypoint position.
5. THE Editor SHALL support undo and redo for all patrol path editing operations (add waypoint, remove waypoint, change Patrol_Mode, change speed, change pause duration).
6. WHEN the user changes the Patrol_Mode, NPC_Movement_Speed, or Waypoint_Pause in the configuration panel, THE Editor SHALL update the NPC_Instance's Patrol_Config immediately.

### Requirement 8: Editor Event Trigger Configuration

**User Story:** As a game creator, I want to configure NPC event triggers in the editor, so that I can define what happens when the player interacts with an NPC.

#### Acceptance Criteria

1. WHEN the user selects an NPC_Instance in the Editor via the NPC_Placement_Tool, THE Editor SHALL display an event trigger configuration panel showing the current Trigger_Mode and the list of event_triggers.
2. THE Editor SHALL allow the user to add ShowDialog and JumpTo actions to an NPC's event_triggers list using the same action configuration interface used for tile event triggers.
3. THE Editor SHALL allow the user to reorder, edit, and remove individual event_triggers from an NPC's list.
4. THE Editor SHALL allow the user to select the Trigger_Mode (Collision or Interaction) for each NPC_Instance.
5. THE Editor SHALL support undo and redo for all event trigger editing operations (add action, remove action, reorder actions, change Trigger_Mode).

### Requirement 9: Interaction Input

**User Story:** As a game creator, I want the player to have an action key for interacting with NPCs, so that Interaction-mode triggers can be activated intentionally.

#### Acceptance Criteria

1. THE Renderer SHALL recognize a configurable action key (default: Space or Enter) as the interaction input.
2. WHEN the player presses the action key, THE Renderer SHALL check whether the tile the player is facing contains an NPC_Instance with Trigger_Mode set to Interaction and a non-empty event_triggers list.
3. IF no interactable NPC is found on the faced tile, THEN THE Renderer SHALL take no action.
4. WHILE a dialog is active or an Action_Queue is being processed, THE Renderer SHALL ignore action key presses.

### Requirement 10: Patrol Path Validation

**User Story:** As a game creator, I want the editor to validate patrol paths, so that I can avoid placing waypoints that would cause NPCs to get stuck.

#### Acceptance Criteria

1. WHEN the user adds a waypoint to a Patrol_Path, THE Editor SHALL validate that the waypoint is within the map bounds.
2. IF a waypoint is placed outside the map bounds, THEN THE Editor SHALL reject the waypoint and display a descriptive error message.
3. WHEN a map is loaded containing NPC_Instances with Patrol_Paths, THE Editor SHALL validate that all waypoints are within the map bounds and report any out-of-bounds waypoints as warnings.