# Requirements Document

## Introduction

This feature bundle adds four new capabilities to the RPG toolkit: a **Jump event** that allows characters to leap over opaque tiles, **parallax background images** for maps, **hotkey bindings** that fire events on player input, and a **Speed event** that transitions the character from walking to running. These enhancements span data models in `rpg-toolkit-common`, runtime behavior in `rpg-toolkit-renderer`, and editor UI in `rpg-toolkit-editor`.

## Glossary

- **EventAction**: The `#[serde(tag = "type")]` enum in `rpg-toolkit-common` representing a single step in a trigger sequence. Processed sequentially by the ActionQueue.
- **ActionQueue**: The Bevy ECS resource that holds a `VecDeque<EventAction>` and processes actions sequentially, waiting for blocking actions to complete before advancing.
- **Renderer**: The `rpg-toolkit-renderer` crate responsible for running the game world, processing triggers, and rendering visual effects via Bevy.
- **Editor**: The `rpg-toolkit-editor` crate providing the egui-based map editing UI.
- **MovementConfig**: The Bevy ECS resource controlling player movement animation timing, specifically the `move_duration` field (default 0.15 seconds per tile).
- **PlayerCharacter**: The Bevy ECS component holding the player's grid position, move animation state, and elevation.
- **MapData**: The data structure representing a map with layers, tiles, NPCs, and dimensions.
- **TileAttributes**: Per-tile metadata including opacity, event triggers, and elevation.
- **ParallaxLayer**: A new background image layer associated with a map that scrolls at a reduced rate relative to the camera, creating a depth effect.
- **HotkeyBinding**: A player-configurable input mapping that fires a named event when the assigned key is pressed during gameplay.
- **SpeedMultiplier**: A new Bevy ECS resource that scales the player's movement rate. Default value is 1.0 (normal walk speed).

## Requirements

### Requirement 1: Jump EventAction Data Model

**User Story:** As a game designer, I want to define a Jump event action with a distance parameter, so that characters can leap over opaque tiles during scripted sequences.

#### Acceptance Criteria

1. THE EventAction enum SHALL include a `Jump` variant with a required `distance` field of type `u32` representing the number of tiles to leap forward in the player's current facing direction.
2. WHEN the `distance` field is deserialized, THE EventAction parser SHALL accept values in the range 1 to 8 inclusive.
3. IF the `distance` field contains a value less than 1 or greater than 8 during deserialization, THEN THE EventAction parser SHALL return a deserialization error indicating the valid range.
4. IF the `distance` field is missing from the JSON input, THEN THE EventAction parser SHALL return a deserialization error indicating that the field is required.
5. THE EventAction `Jump` variant SHALL serialize to and deserialize from JSON using the existing `#[serde(tag = "type")]` format, producing a JSON object with a `"type"` field set to `"Jump"` and a `"distance"` field with the numeric value.
6. THE EventAction `Jump` variant SHALL satisfy the round-trip property: for all valid distance values (1 to 8), serializing then deserializing SHALL produce an equivalent value.

### Requirement 2: Jump Runtime Behavior

**User Story:** As a player, I want my character to leap over obstacles when a jump event fires, so that scripted sequences can bypass normally impassable terrain.

#### Acceptance Criteria

1. WHEN the ActionQueue advances to a `Jump` action, THE Renderer SHALL compute the landing tile as the tile at `distance` tiles forward from the player's current grid position in the player's current facing direction.
2. IF the landing tile is within map bounds, THEN THE Renderer SHALL animate the player from the current tile to the landing tile, bypassing opacity checks on all intermediate tiles.
3. IF the landing tile is outside map bounds, THEN THE Renderer SHALL clamp the landing tile to the last in-bounds tile along the jump direction.
4. WHEN a Jump action is processed, THE Renderer SHALL apply a parabolic vertical offset to the player sprite during the animation to visually represent the leap.
5. WHILE a Jump animation is in progress, THE Renderer SHALL block player input and ActionQueue advancement until the animation completes (blocking action).
6. WHEN the Jump animation completes, THE Renderer SHALL update the PlayerCharacter grid position to the landing tile coordinates.
7. WHEN a Jump action lands on a tile with an event trigger, THE Renderer SHALL fire that tile's event trigger after the jump animation completes.

### Requirement 3: Parallax Background Data Model

**User Story:** As a game designer, I want to assign parallax background images to maps, so that empty space beyond map edges displays artistic scenery instead of a black void.

#### Acceptance Criteria

1. THE MapData struct SHALL include an optional `parallax_layers` field of type `Vec<ParallaxLayer>` that defaults to an empty list when absent from JSON.
2. THE ParallaxLayer struct SHALL contain an `image_path` field of type `String` (1 to 256 characters) referencing the background image asset file.
3. THE ParallaxLayer struct SHALL contain a `scroll_factor` field of type `f32` in the range 0.0 to 1.0 inclusive, where 0.0 means fully static and 1.0 means scrolling at camera speed.
4. THE ParallaxLayer struct SHALL contain a `z_order` field of type `i32` determining the draw order among parallax layers, where lower values render behind higher values.
5. WHEN `parallax_layers` is deserialized, THE parser SHALL accept layers with `scroll_factor` in the range 0.0 to 1.0 inclusive.
6. IF a `scroll_factor` value is less than 0.0 or greater than 1.0 during deserialization, THEN THE parser SHALL return a deserialization error indicating the valid range.
7. FOR ALL valid MapData values containing parallax layers, serializing then deserializing SHALL produce an equivalent value (round-trip property).

### Requirement 4: Parallax Background Rendering

**User Story:** As a player, I want to see parallax background images scrolling behind the map, so that the game world feels more immersive and visually rich.

#### Acceptance Criteria

1. WHEN a map with non-empty `parallax_layers` is loaded, THE Renderer SHALL spawn one sprite entity per parallax layer, each positioned at a `Transform.translation.z` value less than 0.0 so that all parallax sprites render behind all tile layers (which start at z = 0.0).
2. WHILE the camera moves, THE Renderer SHALL translate each parallax layer sprite by the camera position delta multiplied by that layer's `scroll_factor`, where a `scroll_factor` of 0.0 produces no movement (fully static) and 1.0 produces movement equal to the camera delta.
3. WHEN a parallax layer's `image_path` references a file that does not exist or cannot be loaded, THE Renderer SHALL log a warning identifying the missing path and skip spawning that layer without affecting other parallax layers or tile/NPC rendering.
4. THE Renderer SHALL render parallax layers in ascending `z_order`, assigning lower `z_order` values a lower `Transform.translation.z` (drawn first, further from the camera) and higher `z_order` values a higher `Transform.translation.z` (drawn last, closer to the camera), with all parallax z values remaining below 0.0.
5. WHEN the player transitions between maps (MapChanged event), THE Renderer SHALL despawn all parallax layer entities from the previous map before spawning new parallax layer entities for the target map's `parallax_layers`.
6. WHEN a map with an empty `parallax_layers` list is loaded, THE Renderer SHALL not spawn any parallax entities and SHALL render the map without errors.
7. IF two or more parallax layers share the same `z_order` value, THEN THE Renderer SHALL render them in the order they appear in the `parallax_layers` list (stable sort by list index).

### Requirement 5: Parallax Background Editor Support

**User Story:** As a game designer, I want to configure parallax background images in the editor, so that I can visually set up map backgrounds without editing JSON.

#### Acceptance Criteria

1. THE Editor map properties panel SHALL include a "Parallax Backgrounds" section that lists all configured parallax layers for the active map in their current list order.
2. WHEN the designer clicks an "Add Layer" button, THE Editor SHALL append a new ParallaxLayer entry with default values (empty `image_path`, `scroll_factor` of 0.5, `z_order` of 0).
3. IF the map already contains 16 parallax layers when the designer clicks "Add Layer", THEN THE Editor SHALL disable the "Add Layer" button and not append a new entry.
4. WHEN a parallax layer entry is displayed, THE Editor SHALL show an image file picker for `image_path`, a slider for `scroll_factor` (range 0.0 to 1.0, step 0.05), and a numeric input for `z_order` (range -999 to 999).
5. WHEN the designer clicks a "Remove" button on a parallax layer entry, THE Editor SHALL remove that layer from the map's `parallax_layers` list immediately without a confirmation prompt.
6. IF `image_path` is empty when saving, THEN THE Editor SHALL display a validation warning indicating that the layer has no image assigned and SHALL proceed with saving the map data.

### Requirement 6: Hotkey Binding Data Model

**User Story:** As a game designer, I want to define hotkey bindings that map keyboard keys to named events, so that players can trigger actions like sprinting with a button press.

#### Acceptance Criteria

1. THE project data model SHALL include a `hotkey_bindings` field of type `Vec<HotkeyBinding>` that defaults to an empty list when absent from JSON and contains at most 32 entries.
2. THE HotkeyBinding struct SHALL contain a `key_code` field of type `String` (1 to 64 characters) representing the Bevy `KeyCode` variant name (e.g., "ShiftLeft", "KeyZ", "Space").
3. THE HotkeyBinding struct SHALL contain a `name` field of type `String` (1 to 64 characters) providing a human-readable label for the binding.
4. THE HotkeyBinding struct SHALL contain an `event_actions` field of type `Vec<EventAction>` (0 to 20 entries) representing the action sequence fired when the hotkey is pressed.
5. WHEN the `key_code` field is deserialized, THE parser SHALL accept non-empty string values of at most 64 characters.
6. IF the `key_code` field is empty or exceeds 64 characters during deserialization, THEN THE parser SHALL return a deserialization error indicating the valid length range.
7. IF the `name` field is empty or exceeds 64 characters during deserialization, THEN THE parser SHALL return a deserialization error indicating the valid length range.
8. THE serialization and deserialization process SHALL be lossless such that serializing then deserializing any valid HotkeyBinding value produces an equivalent value (round-trip property).
9. IF the `hotkey_bindings` list contains two or more entries with the same `key_code` value during deserialization, THEN THE parser SHALL return a deserialization error indicating that key_code values must be unique.

### Requirement 7: Hotkey Binding Runtime Behavior

**User Story:** As a player, I want to press a configured hotkey during gameplay and have it fire the associated event, so that I can trigger actions like sprinting without navigating menus.

#### Acceptance Criteria

1. WHEN the player presses a key that matches a configured HotkeyBinding's `key_code` WHILE the AppPhase is InGame and no ActionQueue resource is present and no DialogState is active and no SelectionState is active, THE Renderer SHALL push the binding's `event_actions` sequence to the ActionQueue.
2. WHILE a DialogState resource is present, THE Renderer SHALL ignore hotkey presses.
3. WHILE a SelectionState resource is present, THE Renderer SHALL ignore hotkey presses.
4. WHILE an ActionQueue resource is present, THE Renderer SHALL ignore hotkey presses.
5. WHEN multiple hotkey bindings share the same `key_code`, THE Renderer SHALL fire only the first matching binding in list order.
6. WHEN a hotkey's `event_actions` list is empty, THE Renderer SHALL treat the key press as a no-op.

### Requirement 8: Hotkey Binding Editor Support

**User Story:** As a game designer, I want to configure hotkey bindings in the editor, so that I can assign keys to events without editing JSON.

#### Acceptance Criteria

1. THE Editor SHALL include a "Hotkey Bindings" panel accessible from the project settings area.
2. WHEN the designer clicks "Add Binding", THE Editor SHALL create a new HotkeyBinding entry with default values (empty `key_code`, empty `name`, empty `event_actions`).
3. THE Editor hotkey binding form SHALL display a key capture input for `key_code` that records the next key press as the binding value and displays the captured key name.
4. THE Editor hotkey binding form SHALL display a text input for `name` (1 to 64 characters).
5. THE Editor hotkey binding form SHALL display an event action list editor (reusing the existing Event Trigger Editor pattern) for `event_actions`.
6. WHEN the designer clicks "Remove" on a hotkey binding entry, THE Editor SHALL delete that binding from the project's `hotkey_bindings` list.
7. IF `key_code` or `name` is empty when saving, THEN THE Editor SHALL disable the save button and display a validation message.
8. THE Editor SHALL provide drag-and-drop or arrow button reordering of hotkey binding entries, since binding list order determines first-match-wins priority at runtime.

### Requirement 9: Speed EventAction Data Model

**User Story:** As a game designer, I want to define a Speed event action that changes the player's movement rate, so that scripted sequences or hotkeys can make the character run.

#### Acceptance Criteria

1. THE EventAction enum SHALL include a `SetSpeed` variant with a `multiplier` field of type `f32` representing the movement speed scaling factor.
2. WHEN the `multiplier` field is deserialized, THE EventAction parser SHALL accept values in the range 0.5 to 4.0 inclusive.
3. IF the `multiplier` field contains a value less than 0.5 or greater than 4.0 during deserialization, THEN THE EventAction parser SHALL return a deserialization error indicating the valid range.
4. THE EventAction `SetSpeed` variant SHALL serialize to and deserialize from JSON using the existing `#[serde(tag = "type")]` format, producing a JSON object with a `"type"` field set to `"SetSpeed"` and a `"multiplier"` field with the numeric value.
5. THE EventAction `SetSpeed` variant SHALL satisfy the round-trip property: serializing a valid `SetSpeed` value to JSON and deserializing the resulting JSON SHALL produce a value that is structurally equal (`PartialEq`) to the original.
6. THE EventAction `SetSpeed` variant SHALL default the `multiplier` field to 1.0 when the field is absent from the JSON input during deserialization.

### Requirement 10: Speed Runtime Behavior

**User Story:** As a player, I want my character to move faster when a speed event fires, so that sprinting feels responsive and distinct from walking.

#### Acceptance Criteria

1. WHEN the ActionQueue advances to a `SetSpeed` action, THE Renderer SHALL update the SpeedMultiplier resource to the specified `multiplier` value.
2. WHILE SpeedMultiplier is set to a value other than 1.0, THE Renderer SHALL compute the effective `MovementConfig.move_duration` as 0.15 seconds divided by the SpeedMultiplier value, resulting in faster tile transitions for values greater than 1.0 and slower tile transitions for values less than 1.0.
3. WHEN a `SetSpeed` action with a `multiplier` of 1.0 is processed, THE Renderer SHALL restore `MovementConfig.move_duration` to 0.15 seconds.
4. WHEN a `SetSpeed` action is processed, THE ActionQueue SHALL advance immediately to the next action without blocking (non-blocking action).
5. THE SpeedMultiplier resource SHALL be initialized with a value of 1.0 at renderer startup.
6. WHEN a subsequent `SetSpeed` action is processed, THE Renderer SHALL replace the current SpeedMultiplier value with the new `multiplier` value.

### Requirement 11: Editor Support for Jump and Speed Actions

**User Story:** As a game designer, I want to configure Jump and SetSpeed actions in the editor's Event Trigger Editor dialog, so that I can use them in tile triggers and NPC interactions.

#### Acceptance Criteria

1. THE Editor Event Trigger Editor dialog SHALL include `Jump` and `SetSpeed` as selectable action types in the action type dropdown alongside the existing action options.
2. WHEN `Jump` is selected as the action type, THE Editor SHALL display a numeric input field labeled "Distance" with a default value of 2 and an indicated valid range of 1 to 8 (integer, inclusive).
3. WHEN `SetSpeed` is selected as the action type, THE Editor SHALL display a numeric slider labeled "Multiplier" with a default value of 1.0, a range of 0.5 to 4.0 (inclusive), and a step increment of 0.1.
4. IF the user enters a `distance` value less than 1 or greater than 8, THEN THE Editor SHALL clamp the value to the nearest valid bound (1 or 8) when the Add or Update button is clicked.
5. IF the user moves the `multiplier` slider to a value outside the range 0.5 to 4.0, THEN THE Editor SHALL clamp the value to the nearest valid bound (0.5 or 4.0) when the Add or Update button is clicked.
6. WHEN the user clicks the Add or Update button for a `Jump` action with a valid distance value, THEN THE Editor SHALL append or replace the action in the event trigger action list and reset the distance field to the default value of 2.
7. WHEN the user clicks the Add or Update button for a `SetSpeed` action with a valid multiplier value, THEN THE Editor SHALL append or replace the action in the event trigger action list and reset the multiplier slider to the default value of 1.0.

### Requirement 12: Serialization Compatibility

**User Story:** As a game designer, I want my existing project files to continue loading correctly after these enhancements are added, so that I do not lose any work.

#### Acceptance Criteria

1. WHEN a project file containing only pre-existing action types is loaded, THE EventAction parser SHALL deserialize all actions without errors and produce data that satisfies `PartialEq` equality with the original in-memory representation.
2. WHEN a project file containing no `parallax_layers` field on a map is loaded, THE MapData parser SHALL default the field to an empty `Vec<ParallaxLayer>` without error.
3. WHEN a project file containing no `hotkey_bindings` field is loaded, THE project parser SHALL default the field to an empty `Vec<HotkeyBinding>` without error.
4. FOR ALL valid ProjectFile values containing any combination of old and new EventAction variants, serializing then deserializing SHALL produce a value satisfying `PartialEq` equality with the original (round-trip property).
5. WHEN a project file contains an unrecognized `"type"` tag in an EventAction position, THE EventAction parser SHALL return a deserialization error rather than silently discarding the unknown variant.
