# Requirements Document

## Introduction

This specification defines the foundational dialog system for the RPG toolkit's game renderer. Currently the renderer supports player movement, map rendering, sprite animation, and map transitions, but has no mechanism for displaying in-game text or dialog boxes. This feature introduces an event-driven dialog box that can be triggered programmatically, renders text with a configurable typewriter effect, supports positional placement on screen, and optionally blocks player movement while active. Dialog text can be supplied inline or referenced by ID from a separate text registry, enabling future localization and out-of-band text supply without rebuilding the game. The scope is limited to single-message dialog display — conversation trees, branching choices, and NPC portrait integration are out of scope for this foundation layer.

## Glossary

- **Renderer**: The rpg-toolkit-renderer crate that renders the project as a playable game world using Bevy ECS.
- **Dialog_Box**: A UI overlay entity rendered on top of the game world that displays text content to the player. Composed of a background panel and a text region.
- **Dialog_Event**: A Bevy event (Message) that triggers the display of a Dialog_Box, carrying either an inline text string or a Dialog_Text_Id reference, along with configuration for how the dialog should be presented.
- **Dialog_Config**: A configuration struct specifying how a Dialog_Box behaves, including text speed, screen position, and whether player movement is blocked.
- **Dialog_Text_Registry**: A Bevy resource containing a HashMap from Dialog_Text_Id keys to dialog text string values. Enables text to be loaded separately from game logic for localization and out-of-band text supply.
- **Dialog_Text_Id**: A string identifier used to look up dialog text content from the Dialog_Text_Registry.
- **Dialog_State**: A Bevy resource tracking whether a Dialog_Box is currently active, and the progress of the typewriter animation.
- **Typewriter_Effect**: A text reveal animation where characters appear one at a time at a configurable speed, simulating the look of text being typed.
- **Text_Speed**: The number of characters revealed per second during the Typewriter_Effect. A value of 0 means all text is revealed instantly.
- **Dialog_Position**: The vertical placement of the Dialog_Box on screen: top, center, or bottom.
- **Movement_Block**: A state in which player movement input is suppressed, preventing the player character from moving while a Dialog_Box is active.
- **Advance_Input**: The player input (confirm key) used to either complete the Typewriter_Effect instantly or dismiss a fully-revealed Dialog_Box.
- **Player_Character**: The player-controlled entity with grid-based movement, defined by the PlayerCharacter component.

## Requirements

### Requirement 1: Dialog Event Trigger

**User Story:** As a game creator, I want to fire an event that launches a dialog box with specified text and configuration, so that I can trigger dialog display from game logic such as tile triggers or NPC interactions.

#### Acceptance Criteria

1. THE Renderer SHALL provide a Dialog_Event message type that carries a Dialog_Config and either an inline text string or a Dialog_Text_Id referencing an entry in the Dialog_Text_Registry.
2. WHEN a Dialog_Event containing an inline text string is received and no Dialog_Box is currently active, THE Renderer SHALL spawn a Dialog_Box displaying the inline text.
3. WHEN a Dialog_Event containing a Dialog_Text_Id is received and no Dialog_Box is currently active, THE Renderer SHALL look up the text from the Dialog_Text_Registry and spawn a Dialog_Box displaying the resolved text.
4. IF a Dialog_Event contains a Dialog_Text_Id that does not exist in the Dialog_Text_Registry, THEN THE Renderer SHALL log a warning and ignore the Dialog_Event.
5. IF a Dialog_Event is received while a Dialog_Box is already active, THEN THE Renderer SHALL ignore the new Dialog_Event.
6. THE Dialog_Config SHALL contain fields for Text_Speed, Dialog_Position, and Movement_Block.
7. THE Dialog_Config SHALL provide default values of: Text_Speed of 30 characters per second, Dialog_Position of bottom, and Movement_Block of true.

### Requirement 2: Dialog Box Rendering

**User Story:** As a game creator, I want the dialog box to appear as a styled overlay on top of the game world, so that text is clearly readable regardless of the map content behind it.

#### Acceptance Criteria

1. WHEN a Dialog_Box is spawned, THE Renderer SHALL render a semi-transparent background panel sized to occupy approximately 80 percent of the screen width and enough height to contain the text region.
2. WHEN a Dialog_Box is spawned, THE Renderer SHALL render the Dialog_Box at a fixed screen position determined by the Dialog_Position value (top, center, or bottom of the Viewport).
3. THE Dialog_Box SHALL be rendered in screen space as a UI overlay, independent of the game camera position and Pixel_Scale.
4. THE Dialog_Box text SHALL use a legible font color that contrasts with the background panel.
5. WHILE a Dialog_Box is active, THE Renderer SHALL render the Dialog_Box above all game world entities including the Player_Character and NPC sprites.

### Requirement 3: Typewriter Text Reveal

**User Story:** As a game creator, I want dialog text to appear character by character at a configurable speed, so that the dialog feels dynamic and engaging like classic RPG text boxes.

#### Acceptance Criteria

1. WHEN a Dialog_Box is spawned with a Text_Speed greater than zero, THE Renderer SHALL reveal characters one at a time at the rate specified by Text_Speed (characters per second).
2. WHEN a Dialog_Box is spawned with a Text_Speed of zero, THE Renderer SHALL display all text immediately without a Typewriter_Effect.
3. WHILE the Typewriter_Effect is animating, THE Dialog_State SHALL track the number of characters currently revealed.
4. THE Typewriter_Effect SHALL compute the number of visible characters as the elapsed time multiplied by the Text_Speed, clamped to the total character count of the text.
5. FOR ALL non-negative elapsed times and non-negative Text_Speed values, THE visible character count SHALL be between zero and the total text length inclusive.

### Requirement 4: Dialog Advance and Dismissal

**User Story:** As a game creator, I want the player to be able to advance through dialog text and dismiss the dialog box, so that the player controls the pacing of reading.

#### Acceptance Criteria

1. WHEN the player presses the Advance_Input while the Typewriter_Effect is still animating, THE Renderer SHALL immediately reveal all remaining text in the Dialog_Box.
2. WHEN the player presses the Advance_Input while all text is fully revealed, THE Renderer SHALL dismiss the Dialog_Box and remove the Dialog_State.
3. WHEN the Dialog_Box is dismissed, THE Renderer SHALL despawn the Dialog_Box entity and all associated UI entities.
4. THE Advance_Input SHALL be mapped to the Space key and the Enter key.
5. WHEN the Dialog_Box is dismissed and Movement_Block was active, THE Renderer SHALL restore normal player movement input processing.

### Requirement 5: Player Movement Blocking

**User Story:** As a game creator, I want to optionally block player movement while a dialog is displayed, so that the player focuses on reading the text without accidentally walking away.

#### Acceptance Criteria

1. WHEN a Dialog_Box is spawned with Movement_Block set to true, THE Renderer SHALL suppress all player movement input until the Dialog_Box is dismissed.
2. WHEN a Dialog_Box is spawned with Movement_Block set to false, THE Renderer SHALL continue processing player movement input normally while the Dialog_Box is active.
3. WHILE Movement_Block is active, THE Renderer SHALL prevent new tile-to-tile movement from being initiated, but SHALL allow any in-progress movement animation to complete.
4. WHEN the Dialog_Box is dismissed, THE Renderer SHALL resume processing player movement input within the same frame.
5. WHILE Movement_Block is active, THE Renderer SHALL continue to process non-movement input (Advance_Input for dialog interaction).

### Requirement 6: Dialog Position Configuration

**User Story:** As a game creator, I want to choose where the dialog box appears on screen (top, center, or bottom), so that I can position it to avoid obscuring important game content.

#### Acceptance Criteria

1. WHEN Dialog_Position is set to bottom, THE Renderer SHALL position the Dialog_Box anchored to the bottom of the screen with a margin from the screen edge.
2. WHEN Dialog_Position is set to top, THE Renderer SHALL position the Dialog_Box anchored to the top of the screen with a margin from the screen edge.
3. WHEN Dialog_Position is set to center, THE Renderer SHALL position the Dialog_Box vertically centered on the screen.
4. THE Dialog_Position SHALL default to bottom when not explicitly specified in the Dialog_Config.

### Requirement 7: Dialog State Tracking

**User Story:** As a game creator, I want the renderer to track whether a dialog is currently active, so that other game systems can query dialog state and respond accordingly.

#### Acceptance Criteria

1. WHEN a Dialog_Box is spawned, THE Renderer SHALL insert a Dialog_State resource indicating that a dialog is active.
2. WHEN the Dialog_Box is dismissed, THE Renderer SHALL remove the Dialog_State resource.
3. THE Dialog_State SHALL contain the current Typewriter_Effect progress (characters revealed out of total characters).
4. THE Dialog_State SHALL contain a flag indicating whether all text has been fully revealed.
5. WHILE no Dialog_Box is active, THE Dialog_State resource SHALL not be present in the Bevy world.

### Requirement 8: Dialog Config Serialization

**User Story:** As a game creator, I want dialog configuration to be serializable, so that dialog triggers can be saved as part of map event data and loaded from project files.

#### Acceptance Criteria

1. THE Dialog_Config SHALL implement Serialize and Deserialize using serde.
2. FOR ALL valid Dialog_Config values, serializing to JSON and then deserializing SHALL produce an equivalent Dialog_Config (round-trip property).
3. WHEN a Dialog_Config field is absent from the JSON input, THE deserializer SHALL use the default value for that field.
4. THE Dialog_Position enum SHALL implement Serialize and Deserialize using serde.
5. FOR ALL valid Dialog_Position variants, serializing to JSON and then deserializing SHALL produce an equivalent Dialog_Position (round-trip property).
6. THE Dialog_Text_Registry SHALL implement Serialize and Deserialize using serde.
7. FOR ALL valid Dialog_Text_Registry values, serializing to JSON and then deserializing SHALL produce an equivalent Dialog_Text_Registry (round-trip property).
8. THE Dialog_Event text content (inline text string or Dialog_Text_Id) SHALL implement Serialize and Deserialize using serde.

### Requirement 9: Dialog Text Registry

**User Story:** As a game creator, I want dialog text stored in a separate registry keyed by string IDs, so that text content can be loaded independently from game logic and eventually replaced with translations without rebuilding the game.

#### Acceptance Criteria

1. THE Renderer SHALL provide a Dialog_Text_Registry as a Bevy resource containing a mapping from Dialog_Text_Id keys to dialog text string values.
2. THE Dialog_Text_Registry SHALL allow inserting, retrieving, and removing text entries by Dialog_Text_Id.
3. WHEN a Dialog_Text_Id is looked up in the Dialog_Text_Registry and the key exists, THE Dialog_Text_Registry SHALL return the associated text string.
4. WHEN a Dialog_Text_Id is looked up in the Dialog_Text_Registry and the key does not exist, THE Dialog_Text_Registry SHALL return an indication that the key was not found.
5. THE Dialog_Text_Registry SHALL be loadable from a JSON file containing a flat object mapping Dialog_Text_Id strings to text string values.
6. THE Dialog_Text_Registry SHALL support being replaced at runtime, enabling text content to be swapped for localization without restarting the game.
