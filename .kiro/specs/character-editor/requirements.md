# Requirements Document

## Introduction

The Character Editor feature extends the RPG toolkit to support defining multiple playable characters with configurable stat systems and level-based progression rules. Currently the toolkit assumes a single playable character. This feature introduces a character data model in `rpg-toolkit-common`, integrates character data into the project manifest and serialization pipeline, and provides an egui-based editor panel for creating, editing, and deleting characters. This is foundational work that will later support status screens, inventory, shops, and currency labels.

## Glossary

- **Editor**: The Bevy/egui-based RPG Toolkit Editor application (`rpg-toolkit-editor` crate)
- **Character**: A named playable character entity with a set of stats and progression rules
- **Character_Registry**: The project-level collection of all defined playable characters, stored in the project manifest
- **Stat**: A named numeric attribute of a character (e.g., HP, Level, Strength)
- **Required_Stat**: A stat that every character must have and cannot be removed (HP and Level)
- **Optional_Stat**: A stat that a user may add to or remove from a character (Strength, Stamina, Speed, Luck, MP, Wisdom, Intelligence)
- **Progression_Rule**: A formula or configuration that defines how a stat's value changes when a character's level increases
- **Base_Value**: The starting numeric value of a stat at Level 1
- **Growth_Value**: The numeric amount added to a stat per level gained
- **Character_Panel**: The egui editor UI panel for managing characters and their stats
- **Project_File**: The on-disk serialization format for the complete RPG project (`ProjectFile` struct)

## Requirements

### Requirement 1: Character Data Model

**User Story:** As a toolkit developer, I want a well-defined character data structure in the common crate, so that characters can be shared between the editor and renderer.

#### Acceptance Criteria

1. THE Character_Registry SHALL store zero or more Character entries, each identified by a unique string identifier
2. THE Character SHALL contain a display name field of type string with a minimum length of 1 character and a maximum length of 64 characters
3. THE Character SHALL contain a collection of Stat entries, each identified by a stat name of 1 to 32 characters in length
4. WHEN a Character is created, THE Character_Registry SHALL assign a unique UUID v4 identifier to the Character
5. THE Character SHALL always contain a Required_Stat named "HP" with a Base_Value of 10 and a Growth_Value of 5
6. THE Character SHALL always contain a Required_Stat named "Level" with a Base_Value of 1 and a Growth_Value of 0
7. WHERE an Optional_Stat is added, THE Character SHALL store the stat name, Base_Value, and Growth_Value for that stat
8. THE Stat SHALL store Base_Value as an unsigned 32-bit integer with a minimum value of 0 and a maximum value of 4,294,967,295
9. THE Stat SHALL store Growth_Value as an unsigned 32-bit integer with a minimum value of 0 and a maximum value of 4,294,967,295
10. IF an Optional_Stat is added with a stat name that already exists on the Character, THEN THE Character_Registry SHALL reject the addition and return an error indicating a duplicate stat name

### Requirement 2: Character Serialization

**User Story:** As a toolkit developer, I want characters to serialize and deserialize as part of the project file, so that character data persists across editor sessions.

#### Acceptance Criteria

1. THE Project_File SHALL include the Character_Registry in the serialized JSON output under a "characters" key
2. WHEN a Project_File is serialized and then deserialized, THE Character_Registry SHALL contain the same set of characters with identical field values (round-trip property)
3. WHEN a Project_File with no "characters" key is deserialized, THE Project_File SHALL default to an empty Character_Registry using serde's default attribute
4. WHEN a Character_Registry contains duplicate character identifiers, THE Project_File SHALL return a CommonError::ProjectValidationError during deserialization
5. THE Project_File SHALL serialize the Character_Registry to the project manifest for directory-based storage

### Requirement 3: Character Creation

**User Story:** As a game designer, I want to create new playable characters in the editor, so that I can define the cast of my RPG.

#### Acceptance Criteria

1. WHEN the user activates the create character action, THE Character_Panel SHALL display a form for entering a character display name with a maximum input length of 50 characters
2. WHEN the user confirms character creation with a display name containing at least 1 non-whitespace character, THE Editor SHALL add a new Character to the Character_Registry with the provided name (trimmed of leading and trailing whitespace), default HP stat (Base_Value: 10, Growth_Value: 5), and default Level stat (Base_Value: 1, Growth_Value: 0)
3. IF the user confirms character creation with an empty or whitespace-only display name, THEN THE Editor SHALL display a validation error indicating the name is required and prevent creation
4. WHEN a character is successfully created, THE Character_Panel SHALL select the newly created character for editing
5. WHEN the user dismisses the character creation form without confirming, THE Editor SHALL discard the form input and leave the Character_Registry unchanged

### Requirement 4: Character Editing

**User Story:** As a game designer, I want to edit existing characters, so that I can refine their names and stat configurations.

#### Acceptance Criteria

1. WHEN a character is selected in the Character_Panel, THE Editor SHALL display the character's display name and all assigned stats in an editable form
2. WHEN the user modifies a character's display name to a valid non-empty string (at least 1 non-whitespace character, maximum 50 characters), THE Editor SHALL update the Character in the Character_Registry without requiring a separate save action
3. IF the user modifies a character's display name to an empty or whitespace-only string, THEN THE Editor SHALL display a validation error and retain the previous valid display name
4. WHEN the user modifies a stat's Base_Value or Growth_Value, THE Editor SHALL update the stat in the Character without requiring a separate save action
5. THE Character_Panel SHALL display Required_Stats as non-removable entries in the stat list (no delete button shown)
6. THE Character_Panel SHALL display Optional_Stats as removable entries in the stat list (delete button shown)
7. IF the user enters a non-numeric value into a stat's Base_Value or Growth_Value field, THEN THE Editor SHALL reject the input and retain the previous valid numeric value

### Requirement 5: Optional Stat Management

**User Story:** As a game designer, I want to add and remove optional stats on characters, so that I can customize each character's stat profile.

#### Acceptance Criteria

1. WHEN the user activates the add stat action on a Character, THE Character_Panel SHALL display a selection of available Optional_Stats that are not already assigned to the Character; IF all Optional_Stats are already assigned, THEN THE add stat action SHALL be disabled
2. WHEN the user selects an Optional_Stat to add, THE Editor SHALL add the stat to the Character with a default Base_Value of 0 and Growth_Value of 0
3. WHEN the user removes an Optional_Stat from a Character, THE Editor SHALL immediately remove the stat entry from the Character without a confirmation prompt
4. THE Editor SHALL prevent removal of Required_Stats (HP and Level) from a Character
5. THE Character_Panel SHALL offer these Optional_Stats for selection: Strength, Stamina, Speed, Luck, MP, Wisdom, Intelligence

### Requirement 6: Character Deletion

**User Story:** As a game designer, I want to delete characters I no longer need, so that I can keep my project organized.

#### Acceptance Criteria

1. WHEN the user activates the delete action on a Character, THE Editor SHALL display a confirmation prompt that includes the character's display name before proceeding
2. WHEN the user confirms character deletion, THE Editor SHALL remove the Character from the Character_Registry
3. WHEN the user cancels character deletion, THE Editor SHALL retain the Character in the Character_Registry without modification
4. WHEN a character is deleted and the Character_Registry becomes empty, THE Character_Panel SHALL display an empty state with a prompt to create a new character
5. WHEN a character is deleted and other characters remain in the Character_Registry, THE Character_Panel SHALL select the first character in the list

### Requirement 7: Stat Progression Display

**User Story:** As a game designer, I want to preview how stats grow with levels, so that I can verify my progression rules produce balanced characters.

#### Acceptance Criteria

1. WHEN a character is selected, THE Character_Panel SHALL display a computed preview of all assigned stat values at a default preview level of 1
2. WHEN the user changes the preview level, THE Character_Panel SHALL recalculate all displayed stat values using the formula: Base_Value + (Growth_Value × (preview_level - 1))
3. THE Character_Panel SHALL constrain the preview level input to a minimum of 1 and a maximum of 99
4. WHEN the user modifies a stat's Base_Value or Growth_Value while a preview level is displayed, THE Character_Panel SHALL recalculate the preview for the affected stat using the current preview level
5. IF the computed stat value exceeds the maximum unsigned 32-bit integer value (4,294,967,295), THEN THE Character_Panel SHALL display the maximum unsigned 32-bit integer value (4,294,967,295) for that stat

### Requirement 8: Character List Navigation

**User Story:** As a game designer, I want to see all my defined characters in a list, so that I can quickly navigate between them.

#### Acceptance Criteria

1. THE Character_Panel SHALL display a scrollable list of all characters in the Character_Registry, showing each character's display name, ordered alphabetically by display name
2. WHEN the user selects a character from the list, THE Character_Panel SHALL visually highlight the selected entry and load that character's details into the editing form
3. WHEN the Character_Registry is empty, THE Character_Panel SHALL display an empty state message indicating no characters are defined
