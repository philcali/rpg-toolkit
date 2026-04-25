# Requirements Document

## Introduction

This specification defines the integration of the dialog system into the RPG toolkit editor. The dialog foundations spec established the runtime dialog system in the renderer (dialog box rendering, typewriter effect, movement blocking, DialogTextRegistry, DialogConfig, DialogText types). This spec bridges that runtime system into the editor so that map creators can configure dialog triggers on tiles and manage dialog text content.

Three capabilities are introduced:

1. A `ShowDialog` variant on the `EventAction` enum, configurable through the existing Event Trigger Editor dialog. This allows map creators to place dialog triggers on tiles — when the player steps on the tile at runtime, the renderer fires a `ShowDialog` event with the configured text and settings.

2. Sequential trigger execution in the renderer. Currently the trigger system processes only the first EventAction and returns. This spec changes the renderer to process all EventAction entries on a tile sequentially: ShowDialog actions display a dialog and wait for the player to dismiss it before advancing to the next action, while JumpTo actions execute immediately and terminate the sequence (since the player changes maps). This enables dialog exchanges (multiple ShowDialog actions in sequence) and dialog-to-JumpTo sequences (show dialog(s), then teleport the player).

3. A Dialog Text Management panel in the editor's left side panel (below the Map Browser section, always visible regardless of editor mode) where users can perform CRUD operations on entries in the `DialogTextRegistry`, see which tiles/maps reference a given Text_Id, and navigate to those locations. This registry maps string IDs to dialog text strings and is persisted as part of the project file.

The workflow for creating a dialog exchange is:
1. Write each line of the exchange in the DialogTextRegistry with its own Text_Id.
2. Add multiple ShowDialog EventActions to the tile's trigger list in the Event Trigger Editor, each referencing the appropriate Text_Id.
3. Optionally end with a JumpTo action to teleport the player after the dialog sequence.

## Glossary

- **Editor**: The rpg-toolkit-editor crate that provides the map editing UI built with Bevy and egui.
- **Renderer**: The rpg-toolkit-renderer crate that renders the project as a playable game world.
- **EventAction**: An enum in rpg-toolkit-common representing a single action within a tile's event trigger sequence. Currently contains `JumpTo`. Serialized with `#[serde(tag = "type")]`.
- **Event_Trigger_Editor**: The existing egui modal dialog in the editor that opens when a user clicks a tile in EventTrigger attribute mode. Allows adding, removing, and reordering EventAction entries on a tile.
- **ShowDialog_Action**: A new variant of EventAction that carries dialog text content and dialog configuration, used to trigger a dialog box when the player steps on the tile at runtime.
- **DialogText**: An enum with two variants — `Inline(String)` for literal text or `Id(String)` for a reference to a DialogTextRegistry entry. Defined in rpg-toolkit-renderer.
- **DialogConfig**: A configuration struct specifying text speed, dialog position, and movement blocking behavior. Defined in rpg-toolkit-renderer.
- **DialogPosition**: An enum with variants Top, Center, and Bottom controlling vertical placement of the dialog box on screen.
- **DialogTextRegistry**: A Bevy resource containing a HashMap from string IDs to dialog text strings. Supports insert, get, remove, and JSON serialization.
- **Dialog_Text_Panel**: A new egui section in the editor's left side panel (below the Map Browser), always visible regardless of editor mode, for managing DialogTextRegistry entries (create, read, update, delete) and viewing where each entry is referenced.
- **ProjectFile**: The on-disk JSON format for the project, containing maps, tilesets, spawn point, spritesheets, and (newly) dialog text entries.
- **EditCommand**: The undo/redo command type used by the editor. Each reversible change emits an EditCommand with an EditCommandKind variant.
- **Text_Id**: A string identifier used as a key in the DialogTextRegistry.
- **Movement_Block**: A boolean flag in DialogConfig that controls whether player movement is suppressed while the dialog is active.
- **Text_Speed**: The number of characters revealed per second during the typewriter effect. A value of 0 means instant reveal.
- **Trigger_Sequence**: The ordered list of EventAction entries on a tile, processed sequentially by the renderer when the player steps on the tile.
- **Action_Queue**: A runtime queue in the renderer that holds the remaining EventAction entries from a Trigger_Sequence, advancing to the next action after the current one completes.

## Requirements

### Requirement 1: ShowDialog EventAction Variant

**User Story:** As a map creator, I want a ShowDialog action type available in the event trigger system, so that I can make tiles trigger dialog boxes when the player steps on them at runtime.

#### Acceptance Criteria

1. THE EventAction enum SHALL include a `ShowDialog` variant that carries a DialogText value and a DialogConfig value.
2. THE `ShowDialog` variant SHALL be serialized using the existing `#[serde(tag = "type")]` convention, producing JSON with a `"type": "ShowDialog"` discriminator.
3. FOR ALL valid ShowDialog EventAction values, serializing to JSON and then deserializing SHALL produce an equivalent EventAction (round-trip property).
4. WHEN the Renderer encounters a `ShowDialog` EventAction during trigger processing, THE Renderer SHALL fire a ShowDialog event with the carried DialogText and DialogConfig.
5. IF the Renderer encounters a `ShowDialog` EventAction while a dialog box is already active, THEN THE Renderer SHALL skip the action without error.

### Requirement 2: Event Trigger Editor ShowDialog Configuration

**User Story:** As a map creator, I want to add and configure ShowDialog actions in the Event Trigger Editor dialog, so that I can set up dialog triggers on tiles without editing JSON by hand.

#### Acceptance Criteria

1. THE Event_Trigger_Editor SHALL provide an option to add a ShowDialog action in addition to the existing JumpTo action.
2. WHEN the user adds a ShowDialog action, THE Event_Trigger_Editor SHALL present fields for selecting the dialog text source: either inline text entry or a Text_Id reference to the DialogTextRegistry.
3. WHEN the user selects inline text mode, THE Event_Trigger_Editor SHALL display a multi-line text input field for entering the dialog text content.
4. WHEN the user selects Text_Id reference mode, THE Event_Trigger_Editor SHALL display a text input field for entering the Text_Id string.
5. THE Event_Trigger_Editor SHALL present configuration fields for Text_Speed (numeric input, default 30), DialogPosition (dropdown with Top, Center, Bottom options, default Bottom), and Movement_Block (checkbox, default true).
6. THE Event_Trigger_Editor SHALL display existing ShowDialog actions in the action list with a summary showing the text source type and a preview of the text content or Text_Id.
7. WHEN the user saves the Event Trigger Editor, THE Editor SHALL emit a SetEventTrigger EditCommand containing the updated action list, enabling undo and redo of ShowDialog action changes.
8. THE Event_Trigger_Editor SHALL allow stacking multiple ShowDialog actions on a single tile to create dialog exchanges, where each action represents one line of the exchange.

### Requirement 3: Event Trigger Editor ShowDialog Display

**User Story:** As a map creator, I want to see ShowDialog actions clearly listed alongside JumpTo actions in the Event Trigger Editor, so that I can review and manage all trigger actions on a tile.

#### Acceptance Criteria

1. THE Event_Trigger_Editor SHALL display each ShowDialog action with a label indicating the action type as "ShowDialog".
2. WHEN a ShowDialog action uses inline text, THE Event_Trigger_Editor SHALL display a truncated preview of the text content (first 40 characters followed by ellipsis if longer).
3. WHEN a ShowDialog action uses a Text_Id reference, THE Event_Trigger_Editor SHALL display the Text_Id string prefixed with "ID:".
4. THE Event_Trigger_Editor SHALL allow removing ShowDialog actions using the same remove control used for JumpTo actions.
5. THE Event_Trigger_Editor SHALL allow reordering ShowDialog actions relative to other actions using the same up/down controls used for JumpTo actions.

### Requirement 4: Renderer Sequential Trigger Execution

**User Story:** As a map creator, I want the renderer to process all EventAction entries on a tile sequentially, so that I can create dialog exchanges (multiple ShowDialog actions in a row) and dialog-to-JumpTo sequences (show dialog(s), then teleport the player).

#### Acceptance Criteria

1. WHEN the player moves to a tile that has a Trigger_Sequence with one or more EventAction entries, THE Renderer trigger system SHALL begin processing the actions in order, starting with the first action.
2. WHEN the Renderer processes a ShowDialog EventAction in a Trigger_Sequence, THE Renderer SHALL fire a ShowDialog event and wait for the player to dismiss the dialog before advancing to the next action in the sequence.
3. WHEN the Renderer processes a JumpTo EventAction in a Trigger_Sequence, THE Renderer SHALL execute the map transition immediately and terminate the sequence (remaining actions are not processed, since the player has changed maps).
4. WHEN a Trigger_Sequence contains multiple ShowDialog actions, THE Renderer SHALL display each dialog in order, waiting for the player to dismiss each one before showing the next.
5. WHEN a Trigger_Sequence contains ShowDialog actions followed by a JumpTo action, THE Renderer SHALL display all dialogs in order, then execute the JumpTo after the last dialog is dismissed.
6. IF a ShowDialog EventAction in a Trigger_Sequence references a Text_Id that does not exist in the DialogTextRegistry, THEN THE Renderer SHALL log a warning, skip that action, and continue processing the remaining actions in the sequence.
7. WHILE a Trigger_Sequence is being processed, THE Renderer SHALL not start processing a new Trigger_Sequence from another PlayerMoved event.
8. THE Renderer SHALL maintain an Action_Queue resource to track the remaining actions in the current Trigger_Sequence.
9. WHEN the last action in a Trigger_Sequence completes, THE Renderer SHALL remove the Action_Queue resource, indicating that no sequence is in progress.

### Requirement 5: Dialog Text Management Panel

**User Story:** As a map creator, I want a panel in the editor where I can create, view, edit, and delete dialog text entries, so that I can manage all dialog strings in one place and reference them by ID from tile triggers.

#### Acceptance Criteria

1. THE Editor SHALL provide a Dialog_Text_Panel in the left side panel, below the Map Browser section, for managing DialogTextRegistry entries.
2. THE Dialog_Text_Panel SHALL be visible at all times regardless of the current editor mode (not limited to Attribute mode).
3. THE Dialog_Text_Panel SHALL display a scrollable list of all existing dialog text entries, showing each entry's Text_Id and a truncated preview of the text content.
4. THE Dialog_Text_Panel SHALL provide a form for creating a new dialog text entry with fields for Text_Id (single-line text input) and text content (multi-line text input).
5. WHEN the user submits a new entry with a non-empty Text_Id and non-empty text content, THE Dialog_Text_Panel SHALL insert the entry into the DialogTextRegistry.
6. IF the user submits a new entry with a Text_Id that already exists in the DialogTextRegistry, THEN THE Dialog_Text_Panel SHALL display a warning and not overwrite the existing entry.
7. THE Dialog_Text_Panel SHALL provide an edit action for each existing entry that allows modifying the text content.
8. THE Dialog_Text_Panel SHALL provide a delete action for each existing entry that removes the entry from the DialogTextRegistry.
9. WHEN the user creates, edits, or deletes a dialog text entry, THE Editor SHALL emit an EditCommand enabling undo and redo of the change.

### Requirement 6: Dialog Text Find Usages

**User Story:** As a map creator, I want to see which tiles and maps reference a given Text_Id in the Dialog Text Panel, so that I can navigate a project with many dialog entries and understand where each text is used.

#### Acceptance Criteria

1. WHEN the user selects a dialog text entry in the Dialog_Text_Panel, THE Dialog_Text_Panel SHALL display a list of all tiles and maps that reference the selected Text_Id in their ShowDialog EventAction entries.
2. THE find usages list SHALL display each reference with the map name, tile coordinates, and layer index.
3. WHEN the user clicks a reference in the find usages list, THE Editor SHALL navigate to the referenced map (opening it if not already open) and select the referenced tile.
4. THE find usages list SHALL scan all maps and all layers in the project for ShowDialog EventActions that use a DialogText::Id matching the selected Text_Id.
5. WHEN no tiles reference the selected Text_Id, THE Dialog_Text_Panel SHALL display a message indicating that the entry is not used.

### Requirement 7: Dialog Text Registry Persistence

**User Story:** As a map creator, I want dialog text entries to be saved and loaded as part of the project file, so that my dialog content persists across editor sessions.

#### Acceptance Criteria

1. THE ProjectFile SHALL include a field for dialog text entries, serialized as a flat JSON object mapping Text_Id strings to text string values.
2. WHEN the project is saved, THE Editor SHALL serialize all DialogTextRegistry entries into the ProjectFile.
3. WHEN a project is loaded, THE Editor SHALL deserialize the dialog text entries from the ProjectFile and populate the DialogTextRegistry.
4. WHEN a project is loaded and the dialog text field is absent from the JSON, THE Editor SHALL initialize an empty DialogTextRegistry (backward compatibility with existing project files).
5. FOR ALL valid dialog text registry contents, saving the project and then loading it SHALL produce an equivalent DialogTextRegistry (round-trip property).

### Requirement 8: ShowDialog Action Serialization in Project Files

**User Story:** As a map creator, I want ShowDialog actions on tiles to be saved and loaded correctly as part of the project file, so that my dialog triggers persist across editor sessions.

#### Acceptance Criteria

1. WHEN a tile has ShowDialog EventAction entries, THE serializer SHALL include the ShowDialog variant data in the tile's event_trigger array in the project JSON.
2. WHEN a project file containing ShowDialog EventAction entries is loaded, THE deserializer SHALL reconstruct the ShowDialog variants with the correct DialogText and DialogConfig values.
3. WHEN a project file created before this feature (containing only JumpTo actions) is loaded, THE deserializer SHALL load successfully without errors (backward compatibility).
4. FOR ALL valid EventAction lists containing ShowDialog entries, serializing to JSON and then deserializing SHALL produce equivalent EventAction lists (round-trip property).

### Requirement 9: Attribute Overlay for ShowDialog Triggers

**User Story:** As a map creator, I want to see a visual indicator on tiles that have ShowDialog triggers, so that I can identify which tiles will show dialog at runtime while editing the map.

#### Acceptance Criteria

1. WHILE the editor is in Attribute mode, THE Editor SHALL display the existing event trigger overlay on tiles that have ShowDialog EventAction entries, using the same visual indicator used for tiles with JumpTo actions.
2. THE attribute overlay system SHALL treat ShowDialog actions identically to JumpTo actions for the purpose of determining whether a tile has event triggers.

## Out of Scope

The following items are explicitly out of scope for this specification but are documented here for future reference:

- **Confirmation dialog (yes/no branching)**: A dialog that presents the player with a yes/no choice, where each option leads to a different sequence of actions. This requires branching logic in the trigger system and a new EventAction variant or dialog mode. Planned for a future spec.
- **Conversation trees**: Multi-path dialog with branching based on player choices beyond simple yes/no. Requires a dialog graph data structure and a more complex trigger execution model. Planned for a future spec.
- **NPC portraits**: Displaying character portraits alongside dialog text. Requires portrait asset management, a portrait field in DialogConfig or ShowDialog, and UI layout changes to the dialog box. Planned for a future spec.
