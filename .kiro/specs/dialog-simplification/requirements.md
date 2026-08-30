# Requirements Document

## Introduction

This feature simplifies the RPG toolkit's dialog system by removing the legacy `DialogTextData::Id` variant, the standalone `dialog_texts` and `face_portraits` registries, and the associated editor UI (Dialog Text Panel, TextIdIndex). Face portraits are consolidated into the existing `Character` struct via its `VisualAssets`. The cleanup reduces code complexity while preserving backward compatibility for legacy project files.

## Glossary

- **Project_File**: The top-level serialized project structure (`ProjectFile`) containing maps, registries, and configuration.
- **Dialog_Text_Data**: The enum representing dialog text content in event actions (`DialogTextData`).
- **Character_Registry**: The collection of playable characters defined in a project (`CharacterRegistry`).
- **Character**: A single playable character entry with stats, abilities, visual assets, and equipment.
- **Visual_Assets**: The optional asset paths on a Character (spritesheet, face_portrait, status_portrait).
- **Renderer**: The game runtime system that processes event actions and renders dialogs (`rpg-toolkit-renderer`).
- **Action_Editor**: The editor UI form for configuring event actions such as ShowDialog and ShowSelection.
- **Dialog_Text_Panel**: The legacy floating window plugin for managing the `dialog_texts` registry.
- **Text_Id_Index**: The legacy reverse-lookup system tracking which map tiles reference which text IDs.
- **Show_Dialog_Action**: The `EventAction::ShowDialog` variant that displays a dialog box at runtime.
- **Show_Selection_Action**: The `EventAction::ShowSelection` variant that displays a choice prompt at runtime.

## Requirements

### Requirement 1: Remove Dialog Text Id Variant from Data Model

**User Story:** As a developer, I want the `DialogTextData` enum to only support inline text, so that the data model is simpler and the unused registry-lookup path is eliminated.

#### Acceptance Criteria

1. THE Dialog_Text_Data enum SHALL contain only the `Inline(String)` variant in new code paths
2. WHEN a project file containing a `DialogTextData::Id` value is deserialized, THE Dialog_Text_Data SHALL parse successfully without returning an error
3. WHEN a `DialogTextData::Id` value is deserialized from a legacy file, THE Dialog_Text_Data SHALL convert the Id variant to an `Inline("")` value (empty string)
4. WHEN the Project_File is serialized, THE Dialog_Text_Data SHALL write only the `Inline` variant

### Requirement 2: Remove dialog_texts Field from Project File

**User Story:** As a developer, I want the standalone `dialog_texts` registry removed from the project structure, so that unused data is no longer persisted.

#### Acceptance Criteria

1. WHEN a legacy project file containing a `dialog_texts` field is loaded, THE Project_File SHALL parse successfully without returning an error
2. WHEN the Project_File is serialized for saving, THE Project_File SHALL omit the `dialog_texts` field from the output
3. THE Project_File SHALL NOT expose a `dialog_texts` field in its public API

### Requirement 3: Remove face_portraits Field from Project File

**User Story:** As a developer, I want the standalone `face_portraits` registry removed from the project structure, so that portrait data lives solely on Character entries.

#### Acceptance Criteria

1. WHEN a legacy project file containing a `face_portraits` field is loaded, THE Project_File SHALL parse successfully without returning an error
2. WHEN the Project_File is serialized for saving, THE Project_File SHALL omit the `face_portraits` field from the output
3. THE Project_File SHALL NOT expose a `face_portraits` field in its public API

### Requirement 4: Character Face Portrait via Visual Assets

**User Story:** As a game designer, I want each character to have an optional face portrait path in their visual assets, so that dialog face portraits are derived from the character database.

#### Acceptance Criteria

1. THE Character struct SHALL include a `face_portrait` field within Visual_Assets as an optional asset path
2. WHEN the `face_portrait` field is absent in a serialized character, THE Character SHALL deserialize with `face_portrait` set to None
3. WHEN the `face_portrait` field contains a non-empty string, THE Character SHALL store the trimmed path (up to 260 characters)
4. FOR ALL valid Character structs, serializing then deserializing SHALL produce an equivalent Character (round-trip property)

### Requirement 5: Renderer Graceful Degradation for Legacy Id References

**User Story:** As a game designer, I want legacy dialog Id references to degrade gracefully at runtime, so that old projects do not crash when run with the updated renderer.

#### Acceptance Criteria

1. WHEN the Renderer encounters a `DialogTextData::Id` value during Show_Dialog_Action processing, THE Renderer SHALL display an empty string instead of crashing
2. WHEN the Renderer encounters a `DialogTextData::Id` value during Show_Selection_Action prompt resolution, THE Renderer SHALL use an empty string as the prompt text
3. WHEN the Renderer encounters a `DialogTextData::Id` value as a choice label in Show_Selection_Action, THE Renderer SHALL use an empty string as the label text
4. THE Renderer SHALL log a warning when a `DialogTextData::Id` value is encountered at runtime

### Requirement 6: Update ShowDialog Form in Action Editor

**User Story:** As a game designer, I want the ShowDialog editor form to only offer inline text entry and character-based portrait selection, so that the UI reflects the simplified data model.

#### Acceptance Criteria

1. THE Action_Editor SHALL NOT display a "Text ID" input option for Show_Dialog_Action configuration
2. THE Action_Editor SHALL display an inline text entry field for Show_Dialog_Action text content
3. THE Action_Editor SHALL populate the face portrait dropdown from characters in the Character_Registry using their `face_portrait` visual asset path
4. WHEN a character has no `face_portrait` set, THE Action_Editor SHALL exclude that character from the portrait dropdown
5. THE Action_Editor SHALL NOT display a "Text ID" input option for Show_Selection_Action prompt configuration

### Requirement 7: Remove Dialog Text Panel Plugin

**User Story:** As a developer, I want the Dialog Text Panel floating window removed from the editor, so that dead UI code is eliminated.

#### Acceptance Criteria

1. THE editor application SHALL NOT register the Dialog_Text_Panel plugin
2. THE editor application SHALL NOT include the Dialog_Text_Panel module in compilation
3. THE editor application SHALL compile and run without the Dialog_Text_Panel source file

### Requirement 8: Remove TextIdIndex Reverse-Lookup System

**User Story:** As a developer, I want the TextIdIndex infrastructure removed, so that unused indexing code is eliminated.

#### Acceptance Criteria

1. THE editor application SHALL NOT register the Text_Id_Index resource
2. THE editor application SHALL NOT rebuild or update the Text_Id_Index on project load or tile edits
3. THE editor application SHALL compile and run without the Text_Id_Index type or associated functions
4. WHEN undo/redo operations modify event triggers, THE editor application SHALL NOT invoke Text_Id_Index update logic

### Requirement 9: Backward-Compatible Project File Round Trip

**User Story:** As a game designer, I want legacy project files to load without error and save in the new simplified format, so that existing projects are seamlessly migrated.

#### Acceptance Criteria

1. WHEN a legacy project file containing `dialog_texts`, `face_portraits`, and `DialogTextData::Id` values is loaded, THE Project_File SHALL parse all fields without error
2. WHEN the loaded Project_File is saved, THE Project_File SHALL produce valid output that omits `dialog_texts` and `face_portraits` fields
3. WHEN the saved output is loaded again, THE Project_File SHALL parse successfully and produce an equivalent structure (round-trip property for migrated data)
4. IF a legacy project file contains malformed `dialog_texts` or `face_portraits` fields (wrong JSON type), THEN THE Project_File SHALL use default empty values and continue loading

### Requirement 10: Categorized and Searchable Action Type Dropdown

**User Story:** As a game designer, I want the event action type dropdown to be organized by category with a search filter, so that I can quickly find the action I need without scrolling through a long flat list.

#### Acceptance Criteria

1. THE Action_Editor action type dropdown SHALL group action types into named categories (e.g., "Dialog", "Movement", "Camera", "Rewards", "State", "Visual Effects", "System")
2. EACH category SHALL display as a collapsible header within the dropdown, with the individual action types listed beneath it
3. THE Action_Editor SHALL display a text filter input at the top of the action type dropdown
4. WHEN the user types into the filter input, THE Action_Editor SHALL show only action types whose display name contains the filter text (case-insensitive match)
5. WHEN the filter matches actions across multiple categories, THE Action_Editor SHALL display all matching categories with only their matching actions visible
6. WHEN the filter text is cleared, THE Action_Editor SHALL display all categories and action types in their default expanded state
7. EACH action type SHALL belong to exactly one category
8. THE category assignments SHALL be: Dialog (ShowDialog, ShowSelection), Movement (JumpTo, Jump, SetSpeed, MoveEntity), Camera (CameraFollow, CameraPan), Rewards (GiveCurrency, GiveExperience, GiveItem, LearnAbility, AddPartyMember), State (SetState, StateCheck, Branch, SaveGame, ChangePhase), Visual Effects (ScreenShake, StopScreenShake, FadeTransition, SetPlayerAppearance), System (Wait, OpenShop)
