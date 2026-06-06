# Requirements Document

## Introduction

This feature adds a JRPG-style dialog selection system to the RPG toolkit. During dialog sequences, the player can be presented with a list of choices (e.g., "Yes / No", "Go left / Go right / Stay here"). Each choice maps to a distinct branch of event actions, allowing NPCs to pose questions and the game flow to diverge based on player input. The feature integrates with the existing `EventAction` pipeline, `ActionQueue`, and editor tooling.

## Glossary

- **Dialog_Selection_System**: The runtime subsystem responsible for presenting choices to the player during a dialog sequence and routing execution to the selected branch.
- **Choice**: A single selectable option within a dialog selection prompt, consisting of display text and an associated list of event actions.
- **Selection_Prompt**: The UI element displayed to the player containing the prompt text and a vertical list of choices.
- **Cursor**: The visual indicator highlighting the currently focused choice in the selection prompt.
- **Action_Queue**: The existing Bevy resource (`ActionQueue`) that sequences `EventAction` processing; dialog selections block the queue until the player commits a choice.
- **Event_Action**: An enum variant in `rpg_toolkit_common::map::EventAction` representing a discrete game action (dialog, map transition, state change, etc.).
- **Dialog_Text_Registry**: The existing `DialogTextRegistry` resource that maps string IDs to localized dialog text strings.
- **Editor**: The `rpg-toolkit-editor` crate providing the egui-based map and NPC editing interface.

## Requirements

### Requirement 1: ShowSelection Event Action Data Model

**User Story:** As a game designer, I want to define a selection prompt with multiple choices in the event action data, so that the game can present branching dialog options to the player.

#### Acceptance Criteria

1. THE Event_Action enum SHALL include a `ShowSelection` variant containing a prompt text field of type `DialogTextData`, a dialog configuration field of type `DialogConfigData`, and an ordered list of choices.
2. WHEN a `ShowSelection` variant is serialized, THE Event_Action serializer SHALL produce a JSON object with an internally-tagged representation using `"type": "ShowSelection"` consistent with the existing `#[serde(tag = "type")]` convention on EventAction.
3. THE Choice data structure SHALL contain a `label` field of type String with a maximum length of 80 characters and an `actions` field containing an ordered list of at most 20 Event_Action values.
4. IF a `ShowSelection` variant is constructed or deserialized with fewer than 2 or more than 6 choices, THEN THE system SHALL reject the data with a validation error indicating the choices count is out of the allowed range of 2 to 6.
5. THE prompt text field in `ShowSelection` SHALL use the existing `DialogTextData` enum to support both inline text and registry ID references.
6. WHEN a `ShowSelection` variant is deserialized from JSON, THE Event_Action deserializer SHALL reconstruct the prompt, configuration, and choices such that re-serializing the result produces byte-identical JSON output (round-trip property).
7. IF a Choice `label` field is empty or exceeds 80 characters, THEN THE system SHALL reject the data with a validation error indicating the label length constraint.

### Requirement 2: Selection Prompt Rendering

**User Story:** As a player, I want to see a styled selection box with labeled choices when the game presents a decision point, so that I can clearly understand my options.

#### Acceptance Criteria

1. WHEN a `ShowSelection` action is processed by the Action_Queue, THE Dialog_Selection_System SHALL spawn a selection prompt UI containing the prompt text and all choice labels rendered as a vertical list in their defined order.
2. IF a Selection_Prompt is already active when a new `ShowSelection` action is processed, THEN THE Dialog_Selection_System SHALL ignore the new action until the current Selection_Prompt is dismissed.
3. THE Selection_Prompt SHALL use the `DialogConfigData` position field to determine vertical placement (Top, Center, Bottom) consistent with the existing dialog box positioning.
4. THE Selection_Prompt SHALL render with the same panel styling as the standard dialog box (semi-transparent background, light border, 80% width, overflow clip) and SHALL size its height to fit the prompt text and all choice labels without truncation.
5. WHILE the Selection_Prompt is active, THE Dialog_Selection_System SHALL display a Cursor to the left of the currently focused choice label.
6. THE Cursor SHALL initially focus the first choice in the list (index 0).
7. WHEN a face portrait is configured in the `DialogConfigData` (face_portrait field is present), THE Selection_Prompt SHALL display the portrait image to the left of the prompt text, consistent with existing dialog portrait rendering.

### Requirement 3: Selection Navigation Input

**User Story:** As a player, I want to navigate between choices using keyboard input, so that I can select the option I prefer.

#### Acceptance Criteria

1. WHILE the Selection_Prompt is active, WHEN the player presses the Up arrow key or the Up direction key (W or equivalent mapped direction input) as a discrete key press, THE Dialog_Selection_System SHALL move the Cursor to the previous choice in the list.
2. WHILE the Selection_Prompt is active, WHEN the player presses the Down arrow key or the Down direction key (S or equivalent mapped direction input) as a discrete key press, THE Dialog_Selection_System SHALL move the Cursor to the next choice in the list.
3. WHILE the Selection_Prompt is active AND the Cursor is on the first choice, WHEN the player presses any Up navigation key, THE Dialog_Selection_System SHALL wrap the Cursor to the last choice.
4. WHILE the Selection_Prompt is active AND the Cursor is on the last choice, WHEN the player presses any Down navigation key, THE Dialog_Selection_System SHALL wrap the Cursor to the first choice.
5. WHILE the Selection_Prompt is active, THE Dialog_Selection_System SHALL repurpose the direction keys for cursor navigation and block their normal player movement behavior, while allowing any in-progress movement to complete naturally.

### Requirement 4: Selection Confirmation

**User Story:** As a player, I want to confirm my choice by pressing an action key, so that the game continues with the branch I selected.

#### Acceptance Criteria

1. WHILE the Selection_Prompt is active, WHEN the player presses the Space key or Enter key, THE Dialog_Selection_System SHALL commit the currently focused choice.
2. WHEN a choice is committed, THE Dialog_Selection_System SHALL first remove the `SelectionState` resource and despawn the Selection_Prompt UI entities before inserting any branch actions.
3. WHEN a choice is committed, THE Dialog_Selection_System SHALL insert the committed choice's `actions` list at the front of the Action_Queue for sequential processing, after the consumed `ShowSelection` action has been popped from the queue.
4. WHEN a choice is committed and the `SelectionState` resource is removed, THE Action_Queue SHALL detect the absence of the resource and resume processing from the front of the queue on the next system tick.

### Requirement 5: Action Queue Integration

**User Story:** As a game designer, I want `ShowSelection` to block the action queue like `ShowDialog` does, so that subsequent actions only execute after the player makes a choice.

#### Acceptance Criteria

1. WHEN the Action_Queue processes a `ShowSelection` action, THE Action_Queue SHALL set its waiting state to `WaitingFor::Selection` (a new variant analogous to `WaitingFor::Dialog`).
2. WHILE the Action_Queue waiting state is `WaitingFor::Selection`, THE Action_Queue SHALL not advance to or process any subsequent action in the queue.
3. WHEN the `SelectionState` resource is removed (choice committed), THE Action_Queue SHALL clear the `WaitingFor::Selection` state and resume processing from the front of the queue, where the committed choice's branch actions have been inserted.
4. WHILE a Selection_Prompt is active, THE NPC patrol movement system SHALL freeze all NPC movement by skipping patrol tick updates.
5. WHILE a Selection_Prompt is active, THE interaction intent system SHALL suppress new interaction input by ignoring interaction key presses.

### Requirement 6: Dialog Text Registry Support

**User Story:** As a game designer, I want selection prompts and choice labels to support text registry IDs, so that dialog selections can be localized.

#### Acceptance Criteria

1. WHEN the prompt text uses a `DialogTextData::Id` reference, THE Dialog_Selection_System SHALL resolve the text from the Dialog_Text_Registry before rendering.
2. IF the prompt text references a registry ID that does not exist in the Dialog_Text_Registry, OR the Dialog_Text_Registry resource is absent, THEN THE Dialog_Selection_System SHALL log a warning and pop the `ShowSelection` action from the queue without spawning a Selection_Prompt.
3. THE Choice `label` field SHALL use the `DialogTextData` enum to support both inline text and registry ID references.
4. WHEN a choice label uses a `DialogTextData::Id` reference, THE Dialog_Selection_System SHALL resolve the label from the Dialog_Text_Registry before rendering.
5. IF a choice label references a registry ID that does not exist in the Dialog_Text_Registry, THEN THE Dialog_Selection_System SHALL log a warning and pop the `ShowSelection` action from the queue without spawning a Selection_Prompt, consistent with the prompt text error handling.

### Requirement 7: Editor Integration

**User Story:** As a game designer using the editor, I want to add and configure `ShowSelection` actions in the action editor UI, so that I can author branching dialog choices without editing JSON manually.

#### Acceptance Criteria

1. THE Editor action type selector SHALL include a "Show Selection" option in the action type combo box, positioned alphabetically among the existing action types.
2. WHEN the user selects the "Show Selection" action type, THE Editor SHALL display a form with fields for prompt text (text input or registry ID selector), dialog configuration (position combo box, face portrait selector), and a list of choices.
3. THE Editor choice list SHALL allow the user to add new choices (up to 6 maximum) via an "Add Choice" button, and remove existing choices via a per-choice "Remove" button, with the Remove button disabled when only 2 choices remain.
4. THE Editor SHALL provide a nested action editor for each choice's action list, reusing the same recursive action editor pattern used by the existing Branch and StateCheck editors.
5. WHEN the user saves a `ShowSelection` action in the editor, THE Editor SHALL validate that at least 2 choices are defined and each choice has a non-empty label, displaying an inline error message adjacent to any invalid field.

### Requirement 8: Serialization Round-Trip

**User Story:** As a developer, I want `ShowSelection` data to serialize and deserialize without data loss, so that saved maps maintain full fidelity.

#### Acceptance Criteria

1. WHEN a valid `ShowSelection` action (containing 2 to 6 choices, a prompt text, and a dialog configuration) is serialized to JSON and then deserialized back, THE serialization system SHALL produce a `ShowSelection` value that is structurally equal to the original (all fields, including prompt, config defaults, and choice order, compare equal via `PartialEq`).
2. WHEN a valid Choice value contains nested Event_Action lists (including recursive nesting up to at least 3 levels deep, such as a ShowSelection containing choices whose actions include StateCheck or Branch variants), serializing to JSON and deserializing back SHALL produce a value structurally equal to the original, preserving action order and nesting at every level.
3. THE `ShowSelection` JSON representation SHALL include a top-level `"type": "ShowSelection"` field produced by the `#[serde(tag = "type")]` attribute, matching the tagging convention used by all other EventAction variants.
4. IF the JSON input is missing required fields (prompt, choices) or contains fewer than 2 choices or more than 6 choices, THEN THE deserialization system SHALL return a deserialization error rather than producing a partial or default-filled value.
