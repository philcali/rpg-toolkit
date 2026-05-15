# Requirements Document

## Introduction

This feature extends the existing event trigger system with five new `EventAction` variants: **ScreenShake** (earthquake effect with timed or continuous mode), **StopScreenShake** (stops a continuous screen shake), **FadeTransition** (fade in/out for screen transitions), **SetState** (persistent game state flags), and **SetPlayerAppearance** (change the player's visual appearance — hide, swap spritesheet, or restore default). These actions integrate into the existing `ActionQueue` sequential processing pipeline and are configurable through the editor's Event Trigger Editor dialog.

## Glossary

- **ActionQueue**: The Bevy ECS resource that holds a `VecDeque<EventAction>` and processes actions sequentially, waiting for blocking actions to complete before advancing.
- **EventAction**: The `#[serde(tag = "type")]` enum in `rpg-toolkit-common` representing a single step in a trigger sequence. Currently has `JumpTo` and `ShowDialog` variants.
- **Renderer**: The `rpg-toolkit-renderer` crate responsible for running the game world, processing triggers, and rendering visual effects.
- **Editor**: The `rpg-toolkit-editor` crate providing the map editing UI including the Event Trigger Editor dialog.
- **GameState**: A new Bevy ECS resource that holds a `HashMap<String, String>` of key-value pairs representing persistent game flags (e.g., `"talked_to_elder" → "true"`).
- **ScreenShake**: A visual effect that displaces the game camera by random offsets each frame to simulate an earthquake. Supports two modes: Timed (runs for a fixed duration) and Continuous (runs indefinitely until explicitly stopped).
- **ScreenShakeMode**: An enum (`Timed`, `Continuous`) that determines whether a ScreenShake effect runs for a fixed duration or indefinitely until a `StopScreenShake` action is processed.
- **StopScreenShake**: An EventAction variant that stops any active continuous screen shake effect.
- **FadeTransition**: A full-screen overlay that transitions between transparent and opaque (or vice versa) over a configurable duration.
- **PlayerSprite**: The entity with the `PlayerCharacter` component representing the player's visual presence in the game world.
- **PlayerAppearance**: A sub-enum describing the player's visual state. Variants: `Hidden` (sprite invisible), `Spritesheet(path)` (swap to a different spritesheet file), `Default` (restore original appearance and visibility).
- **SetPlayerAppearance**: An EventAction variant that changes the player's visual appearance by applying a `PlayerAppearance` value.

## Requirements

### Requirement 1: ScreenShake EventAction Data Model

**User Story:** As a game designer, I want to define a screen shake event with configurable intensity, duration, and mode (timed or continuous), so that I can create earthquake or impact effects as well as sustained shaking during dialog or cutscenes.

#### Acceptance Criteria

1. THE EventAction enum SHALL include a `ScreenShake` variant with an `intensity` field of type `f32`, a `duration` field of type `f32`, and a `mode` field of type `ScreenShakeMode`.
2. THE `ScreenShakeMode` enum SHALL have two variants: `Timed` and `Continuous`.
3. WHEN the `mode` field is `Timed`, THE `duration` field SHALL specify how long the shake effect lasts in seconds.
4. WHEN the `mode` field is `Continuous`, THE `duration` field SHALL be ignored and the shake effect SHALL persist until a `StopScreenShake` action is processed.
5. WHEN the `mode` field is not specified in JSON, THE EventAction parser SHALL default to `Timed`.
6. WHEN the `intensity` field is deserialized, THE EventAction parser SHALL accept values in the range 0.0 to 50.0 inclusive, representing maximum pixel displacement.
7. WHEN the `duration` field is deserialized, THE EventAction parser SHALL accept values in the range 0.0 to 10.0 inclusive, representing seconds.
8. THE EventAction `ScreenShake` variant SHALL serialize to and deserialize from JSON using the existing `#[serde(tag = "type")]` format.
9. FOR ALL valid ScreenShake EventAction values, serializing then deserializing SHALL produce an equivalent value (round-trip property).

### Requirement 2: ScreenShake Runtime Effect

**User Story:** As a player, I want to see the screen shake when an earthquake event triggers, so that I experience dramatic in-game moments, including sustained shaking during dialog sequences.

#### Acceptance Criteria

1. WHEN the ActionQueue advances to a `ScreenShake` action with mode `Timed`, THE Renderer SHALL insert a `ScreenShakeState` resource containing the configured intensity, duration, mode, and an elapsed timer starting at 0.0.
2. WHEN the ActionQueue advances to a `ScreenShake` action with mode `Continuous`, THE Renderer SHALL insert a `ScreenShakeState` resource containing the configured intensity and mode, with no duration limit.
3. WHILE a `ScreenShakeState` resource is present, THE Renderer SHALL apply a random translational offset to the `GameCamera` transform each frame, with magnitude not exceeding the configured intensity in pixels.
4. WHILE a `ScreenShakeState` resource is present with mode `Timed`, THE ActionQueue SHALL block advancement to the next action until the shake completes.
5. WHEN a `ScreenShake` action has mode `Continuous`, THE ActionQueue SHALL advance immediately to the next action without blocking (non-blocking action).
6. WHEN the elapsed timer in a `Timed` `ScreenShakeState` reaches or exceeds the configured duration, THE Renderer SHALL remove the `ScreenShakeState` resource and reset the camera offset to zero.
7. WHEN a `ScreenShake` action has mode `Timed` and a duration of 0.0, THE Renderer SHALL treat the action as instantly complete and advance the ActionQueue without applying any visual offset.
8. WHILE a `Continuous` `ScreenShakeState` resource is present, THE Renderer SHALL continue applying the shake effect until a `StopScreenShake` action is processed.

### Requirement 3: StopScreenShake EventAction Data Model

**User Story:** As a game designer, I want to stop a continuous screen shake at a specific point in my event sequence, so that I can control exactly when the shaking ends relative to dialog or other actions.

#### Acceptance Criteria

1. THE EventAction enum SHALL include a `StopScreenShake` variant with no additional fields.
2. THE EventAction `StopScreenShake` variant SHALL serialize to and deserialize from JSON using the existing `#[serde(tag = "type")]` format.
3. FOR ALL valid StopScreenShake EventAction values, serializing then deserializing SHALL produce an equivalent value (round-trip property).

### Requirement 4: StopScreenShake Runtime Effect

**User Story:** As a player, I want the screen to stop shaking at the appropriate narrative moment, so that the effect feels intentional and controlled.

#### Acceptance Criteria

1. WHEN the ActionQueue advances to a `StopScreenShake` action, THE Renderer SHALL remove the `ScreenShakeState` resource and reset the camera offset to zero.
2. WHEN a `StopScreenShake` action is processed, THE ActionQueue SHALL advance immediately to the next action without blocking (non-blocking action).
3. IF no `ScreenShakeState` resource is present when a `StopScreenShake` action is processed, THEN THE Renderer SHALL treat the action as a no-op and advance the ActionQueue to the next action.

### Requirement 5: FadeTransition EventAction Data Model

**User Story:** As a game designer, I want to define fade-in and fade-out transitions with configurable duration and color, so that I can create smooth scene transitions.

#### Acceptance Criteria

1. THE EventAction enum SHALL include a `FadeTransition` variant with a `fade_type` field (enum: `FadeIn`, `FadeOut`), a `duration` field of type `f32`, and a `color` field of type `[f32; 4]` representing RGBA.
2. WHEN `fade_type` is `FadeOut`, THE FadeTransition SHALL transition the screen from fully visible to fully covered by the specified color.
3. WHEN `fade_type` is `FadeIn`, THE FadeTransition SHALL transition the screen from fully covered by the specified color to fully visible.
4. WHEN the `duration` field is deserialized, THE EventAction parser SHALL accept values in the range 0.0 to 10.0 inclusive, representing seconds.
5. THE `color` field SHALL default to `[0.0, 0.0, 0.0, 1.0]` (opaque black) when not specified in JSON.
6. FOR ALL valid FadeTransition EventAction values, serializing then deserializing SHALL produce an equivalent value (round-trip property).

### Requirement 6: FadeTransition Runtime Effect

**User Story:** As a player, I want to see smooth fade transitions between scenes, so that the game feels polished and cinematic.

#### Acceptance Criteria

1. WHEN the ActionQueue advances to a `FadeTransition` action, THE Renderer SHALL insert a `FadeState` resource and spawn a full-screen UI overlay entity with the configured color.
2. WHILE a `FadeState` resource is present with `fade_type` `FadeOut`, THE Renderer SHALL interpolate the overlay opacity from 0.0 to 1.0 over the configured duration.
3. WHILE a `FadeState` resource is present with `fade_type` `FadeIn`, THE Renderer SHALL interpolate the overlay opacity from 1.0 to 0.0 over the configured duration.
4. WHILE a `FadeState` resource is present, THE ActionQueue SHALL block advancement to the next action until the fade completes.
5. WHEN the fade animation completes, THE Renderer SHALL remove the `FadeState` resource.
6. WHEN a `FadeOut` completes, THE Renderer SHALL leave the overlay entity visible at full opacity until a subsequent `FadeIn` action removes it.
7. WHEN a `FadeIn` completes, THE Renderer SHALL despawn the overlay entity.
8. WHEN a `FadeTransition` action has a duration of 0.0, THE Renderer SHALL apply the final state instantly (full overlay for FadeOut, no overlay for FadeIn) and advance the ActionQueue without animation.

### Requirement 7: SetState EventAction Data Model

**User Story:** As a game designer, I want to set persistent game state flags from event triggers, so that I can track story progression and use those flags as conditions.

#### Acceptance Criteria

1. THE EventAction enum SHALL include a `SetState` variant with a `key` field of type `String` and a `value` field of type `String`.
2. THE `key` field SHALL be a non-empty string identifier for the state variable.
3. THE `value` field SHALL be a string representing the value to assign to the state variable.
4. FOR ALL valid SetState EventAction values, serializing then deserializing SHALL produce an equivalent value (round-trip property).

### Requirement 8: SetState Runtime Effect

**User Story:** As a game designer, I want state changes to take effect immediately when triggered, so that subsequent triggers can check conditions reliably.

#### Acceptance Criteria

1. WHEN the ActionQueue advances to a `SetState` action, THE Renderer SHALL insert or update the entry in the `GameState` resource with the specified key and value.
2. WHEN a `SetState` action is processed, THE ActionQueue SHALL advance immediately to the next action without blocking (non-blocking action).
3. THE GameState resource SHALL be initialized as an empty `HashMap<String, String>` at renderer startup.
4. IF the `GameState` resource already contains an entry for the specified key, THEN THE Renderer SHALL overwrite the existing value with the new value.

### Requirement 9: SetPlayerAppearance EventAction Data Model

**User Story:** As a game designer, I want to change the player's visual appearance via event triggers — hiding the sprite, swapping to a different spritesheet for disguises or transformations, or restoring the default look — so that I can create cutscenes, disguise mechanics, and transformation sequences.

#### Acceptance Criteria

1. THE EventAction enum SHALL include a `SetPlayerAppearance` variant with an `appearance` field of type `PlayerAppearance`.
2. THE `PlayerAppearance` enum SHALL have three variants: `Hidden`, `Spritesheet` (with a `path` field of type `String`), and `Default`.
3. WHEN `appearance` is `Hidden`, THE action SHALL indicate the player sprite should be hidden entirely.
4. WHEN `appearance` is `Spritesheet`, THE action SHALL indicate the player sprite should be swapped to the spritesheet at the specified file path.
5. WHEN `appearance` is `Default`, THE action SHALL indicate the player sprite should be restored to its original appearance and visibility.
6. THE `PlayerAppearance` enum SHALL serialize using an internally-tagged representation (e.g., `{"variant": "Spritesheet", "path": "assets/disguise.png"}`) compatible with the existing `#[serde(tag = "type")]` format of EventAction.
7. FOR ALL valid SetPlayerAppearance EventAction values, serializing then deserializing SHALL produce an equivalent value (round-trip property).

### Requirement 10: SetPlayerAppearance Runtime Effect

**User Story:** As a player, I want my character's appearance to change dynamically during gameplay — disappearing in cutscenes, transforming into another character, or returning to normal — so that the visual narrative matches the story.

#### Acceptance Criteria

1. WHEN the ActionQueue advances to a `SetPlayerAppearance` action with `appearance` set to `Hidden`, THE Renderer SHALL set the `Visibility` component of the PlayerSprite entity to `Visibility::Hidden`.
2. WHEN the ActionQueue advances to a `SetPlayerAppearance` action with `appearance` set to `Spritesheet`, THE Renderer SHALL load the spritesheet at the specified path, rebuild the texture atlas, and replace the PlayerSprite entity's texture and atlas layout with the new spritesheet.
3. WHEN the ActionQueue advances to a `SetPlayerAppearance` action with `appearance` set to `Spritesheet`, THE Renderer SHALL set the `Visibility` component of the PlayerSprite entity to `Visibility::Inherited` if the sprite was previously hidden.
4. WHEN the ActionQueue advances to a `SetPlayerAppearance` action with `appearance` set to `Default`, THE Renderer SHALL restore the PlayerSprite entity's texture and atlas layout to the original spritesheet assigned at spawn time and set `Visibility` to `Visibility::Inherited`.
5. WHEN a `SetPlayerAppearance` action is processed, THE ActionQueue SHALL advance immediately to the next action without blocking (non-blocking action).
6. WHILE the PlayerSprite is hidden, THE Renderer SHALL continue to process player movement input and collision detection normally.
7. IF the spritesheet path specified in a `Spritesheet` appearance does not exist or fails to load, THEN THE Renderer SHALL log a warning and leave the player's current appearance unchanged.

### Requirement 11: Editor Support for New Event Actions

**User Story:** As a game designer, I want to configure the new event actions in the editor's Event Trigger Editor dialog, so that I can place and tune them without editing JSON by hand.

#### Acceptance Criteria

1. THE Editor Event Trigger Editor dialog SHALL include `ScreenShake`, `StopScreenShake`, `FadeTransition`, `SetState`, and `SetPlayerAppearance` as selectable action types alongside the existing `JumpTo` and `ShowDialog` options.
2. WHEN `ScreenShake` is selected, THE Editor SHALL display a selector for `mode` (Timed or Continuous, default Timed), an input field for `intensity` (numeric, default 5.0), and an input field for `duration` (numeric, default 0.5).
3. WHEN `ScreenShake` is selected and `mode` is set to `Continuous`, THE Editor SHALL disable or hide the `duration` input field since duration is not applicable in continuous mode.
4. WHEN `StopScreenShake` is selected, THE Editor SHALL display no additional configuration fields.
5. WHEN `FadeTransition` is selected, THE Editor SHALL display a selector for `fade_type` (FadeIn or FadeOut), an input field for `duration` (numeric, default 1.0), and a color picker for `color` (default black).
6. WHEN `SetState` is selected, THE Editor SHALL display input fields for `key` (text) and `value` (text).
7. WHEN `SetPlayerAppearance` is selected, THE Editor SHALL display a selector for `appearance` with options `Hidden`, `Spritesheet`, and `Default` (default `Hidden`).
8. WHEN `SetPlayerAppearance` is selected and `appearance` is set to `Spritesheet`, THE Editor SHALL display a file path picker for selecting the spritesheet asset path.
9. WHEN `SetPlayerAppearance` is selected and `appearance` is set to `Hidden` or `Default`, THE Editor SHALL display no additional configuration fields beyond the appearance selector.
10. THE Editor SHALL validate that `intensity` is between 0.0 and 50.0 and `duration` is between 0.0 and 10.0 before allowing the action to be saved.
11. THE Editor SHALL validate that the `key` field for `SetState` is non-empty before allowing the action to be saved.
12. THE Editor SHALL validate that the `path` field for `SetPlayerAppearance` with `Spritesheet` appearance is non-empty before allowing the action to be saved.

### Requirement 12: ActionQueue Integration

**User Story:** As a game designer, I want the new event actions to work seamlessly in sequences with existing JumpTo and ShowDialog actions, so that I can compose complex trigger chains including sustained shaking during dialog.

#### Acceptance Criteria

1. THE ActionQueue SHALL process `ScreenShake`, `StopScreenShake`, `FadeTransition`, `SetState`, and `SetPlayerAppearance` actions in sequence order alongside existing `JumpTo` and `ShowDialog` actions.
2. WHEN a blocking action (`ScreenShake` with mode `Timed`, or `FadeTransition`) is active, THE ActionQueue SHALL wait for completion before advancing to the next action.
3. WHEN a non-blocking action (`ScreenShake` with mode `Continuous`, `StopScreenShake`, `SetState`, or `SetPlayerAppearance`) is processed, THE ActionQueue SHALL advance to the next action in the same frame.
4. WHEN a `JumpTo` action is encountered in the queue, THE ActionQueue SHALL clear the remaining queue and execute the map transition, consistent with existing behavior.
5. WHEN a `JumpTo` action clears the queue while a `Continuous` ScreenShake is active, THE Renderer SHALL remove the `ScreenShakeState` resource and reset the camera offset to zero.

### Requirement 13: Serialization Compatibility

**User Story:** As a game designer, I want my existing project files to continue loading correctly after the new event actions are added, so that I do not lose any work.

#### Acceptance Criteria

1. WHEN a project file containing only `JumpTo` and `ShowDialog` actions is loaded, THE EventAction parser SHALL deserialize all actions correctly without errors.
2. WHEN a project file containing the new action types is loaded by an older version of the toolkit that does not recognize them, THE EventAction parser SHALL report a clear deserialization error identifying the unknown action type.
3. FOR ALL valid ProjectFile values containing any combination of EventAction variants, serializing then deserializing SHALL produce an equivalent ProjectFile (round-trip property).
