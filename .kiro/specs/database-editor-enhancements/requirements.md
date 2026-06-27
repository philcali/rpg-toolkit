# Requirements Document

## Introduction

This feature enhances the RPG toolkit's database editors with nine improvements: adding a "Monster" ability category to separate monster-only abilities from player abilities, replacing free-text ability input in the enemy editor with dropdown selection from the abilities database, introducing a class/character ability learning system with level thresholds, adding enemy portrait support to the enemy editor, allowing equippable items to grant abilities when equipped, adding optional visual asset references (spritesheet, face portrait, status portrait) to characters, providing native file picker dialogs for character visual asset fields, providing a native file picker dialog for the enemy portrait field, and adding starting equipment configuration to characters.

## Glossary

- **Ability_Editor**: The editor panel responsible for creating and managing abilities in the abilities database.
- **Enemy_Editor**: The editor panel responsible for creating and managing enemy definitions in the enemies database.
- **Character_Editor**: The editor panel responsible for creating and managing playable character definitions.
- **Ability_Registry**: The data structure (`AbilityRegistry`) that stores all defined abilities keyed by their unique ID.
- **Enemy_Registry**: The data structure (`EnemyRegistry`) that stores all defined enemies keyed by their unique ID.
- **Character_Registry**: The data structure (`CharacterRegistry`) that stores all defined characters keyed by their unique ID.
- **Ability_Category**: An enum classifying abilities into types (currently Skill, Spell, SpecialAction).
- **Learnable_Ability**: A reference from a character to an ability in the Ability_Registry, paired with the level at which the character learns that ability.
- **Enemy_Portrait**: A file path reference to an image asset representing the visual appearance of an enemy in battle.
- **Dropdown_Selector**: A UI widget (egui ComboBox or searchable combobox) that presents a filtered list of entries from another database for selection.
- **Item_Editor**: The editor panel responsible for creating and managing items in the items database.
- **Item_Registry**: The data structure (`ItemRegistry`) that stores all defined items keyed by their unique ID.
- **Granted_Ability**: A reference from an equippable item to an ability in the Ability_Registry, indicating the ability becomes available while the item is equipped.
- **Character_Visual_Assets**: Optional file path references on a character for a spritesheet, face portrait, and status portrait, used by the built-in status page renderer or custom plugins.
- **File_Picker**: A native OS file dialog (powered by the `rfd` crate) that allows the user to browse and select image files from the filesystem.
- **Starting_Equipment**: A list of item IDs on a character referencing items in the Item_Registry, representing the equipment a character begins the game with.

## Requirements

### Requirement 1: Monster Ability Category

**User Story:** As a game designer, I want a "Monster" category in the ability editor, so that I can create abilities exclusive to monsters and keep them separate from player-usable abilities.

#### Acceptance Criteria

1. THE Ability_Editor SHALL include "Monster" as a selectable option in the category filter combo box alongside the existing "All", "Skill", "Spell", and "Special Action" filters.
2. THE Ability_Editor SHALL include "Monster" as a selectable option when creating a new ability in the create dialog's category selector.
3. THE Ability_Editor SHALL include "Monster" as a selectable option in the category edit combo box for existing abilities.
4. WHEN "Monster" is selected as the category filter, THE Ability_Editor SHALL display only abilities with the Monster category.
5. WHEN "All" is selected as the category filter, THE Ability_Editor SHALL display abilities of all categories including Monster.
6. THE Ability_Registry SHALL persist the Monster category value through JSON serialization and deserialization without data loss, round-tripping an identical enum variant.

### Requirement 2: Enemy Ability Dropdown Selection

**User Story:** As a game designer, I want to select enemy abilities from a dropdown populated by the abilities database, so that I avoid typos and can only assign abilities that actually exist.

#### Acceptance Criteria

1. WHEN the user adds an ability to an enemy, THE Enemy_Editor SHALL present a Dropdown_Selector populated with abilities from the Ability_Registry.
2. THE Dropdown_Selector SHALL display each ability as its display name followed by its category in brackets (e.g., "Fireball [Spell]"), sorted case-insensitively by display name.
3. WHEN the user selects an ability from the Dropdown_Selector, THE Enemy_Editor SHALL store the corresponding ability ID on the enemy, up to a maximum of 10 abilities per enemy.
4. THE Dropdown_Selector SHALL support text filtering using case-insensitive substring matching against the ability display name to narrow the list of abilities shown.
5. THE Enemy_Editor SHALL prevent assigning an ability that is already assigned to the same enemy.
6. WHEN no abilities exist in the Ability_Registry, THE Enemy_Editor SHALL display a message indicating no abilities are available.
7. IF the user attempts to add an ability when the enemy already has 10 abilities assigned, THEN THE Enemy_Editor SHALL display an error message indicating the maximum ability limit has been reached and SHALL NOT add the ability.

### Requirement 3: Character Ability Learning System

**User Story:** As a game designer, I want to define which abilities a character class can learn and at which level they learn them, so that I can design class-specific progression paths.

#### Acceptance Criteria

1. THE Character_Registry SHALL store a list of Learnable_Ability entries for each character, where each entry contains an ability ID referencing the Ability_Registry and a required level integer between 1 and 99 inclusive.
2. THE Character_Editor SHALL display a "Learnable Abilities" section showing all assigned learnable abilities with their ability display name and required level, sorted ascending by required level.
3. WHEN the user adds a learnable ability and the Ability_Registry contains at least one ability, THE Character_Editor SHALL present a Dropdown_Selector populated with abilities from the Ability_Registry sorted alphabetically by display name.
4. WHEN the user adds a learnable ability, THE Character_Editor SHALL require a level value between 1 and 99 inclusive, rejecting values outside this range by clamping the input to the nearest valid bound.
5. IF the user attempts to assign an ability ID that already exists in the character's learnable ability list, THEN THE Character_Editor SHALL reject the addition and display an error message indicating the ability is already assigned.
6. THE Character_Editor SHALL allow the user to remove a learnable ability from the list.
7. THE Character_Editor SHALL allow the user to modify the required level of an existing learnable ability, constrained to the range 1 to 99 inclusive.
8. THE Character_Registry SHALL persist learnable ability data through serialization and deserialization without data loss.
9. IF the Ability_Registry contains no abilities, THEN THE Character_Editor SHALL disable the add-ability control and display a message indicating no abilities are available.

### Requirement 4: Enemy Portrait

**User Story:** As a game designer, I want to define an enemy portrait image for each enemy, so that I can visually represent enemies during battle encounters.

#### Acceptance Criteria

1. THE Enemy_Registry SHALL store an optional portrait file path (maximum 260 characters) for each enemy, defaulting to None for newly created enemies.
2. THE Enemy_Editor SHALL display a "Portrait" section in the enemy detail view containing the portrait path input field and a "Clear" control.
3. WHEN no portrait is set, THE Enemy_Editor SHALL display a text label indicating no portrait is assigned in the "Portrait" section.
4. THE Enemy_Editor SHALL allow the user to set or change the portrait by entering a file path in a single-line text input, truncating input to 260 characters.
5. WHEN the user activates the clear control, THE Enemy_Editor SHALL reset the portrait field to None.
6. THE Enemy_Registry SHALL persist the portrait field through serialization and deserialization such that a round-trip produces an identical value (None remains None, a set path remains the same string).
7. IF the user enters a portrait path that is empty or contains only whitespace after trimming, THEN THE Enemy_Editor SHALL display a validation error message and SHALL NOT store the value in the Enemy_Registry.

### Requirement 5: Equipment-Granted Abilities

**User Story:** As a game designer, I want to define abilities that are granted to a character when they equip certain items (weapons, armor, accessories), so that I can create interesting equipment with unique skill effects.

#### Acceptance Criteria

1. THE Item_Registry SHALL store a list of granted ability IDs (maximum 4 per item) for equippable items (Weapon, Armor, Accessory categories), where each entry references an ability in the Ability_Registry.
2. THE Item_Editor SHALL display a "Granted Abilities" section for equippable items (Weapon, Armor, Accessory) showing all assigned abilities by their display name and category in brackets (e.g., "Fireball [Spell]").
3. WHEN the user adds a granted ability, THE Item_Editor SHALL present a Dropdown_Selector populated with abilities from the Ability_Registry, sorted case-insensitively by display name.
4. THE Dropdown_Selector SHALL support text filtering using case-insensitive substring matching against the ability display name.
5. IF the user attempts to assign an ability that is already granted by the same item, THEN THE Item_Editor SHALL reject the addition and display an error message indicating the ability is already assigned.
6. IF the user attempts to add a granted ability when the item already has 4 granted abilities, THEN THE Item_Editor SHALL display an error message indicating the maximum limit has been reached and SHALL NOT add the ability.
7. THE Item_Editor SHALL allow the user to remove a granted ability from the list.
8. THE "Granted Abilities" section SHALL NOT appear for Consumable or KeyItem category items.
9. WHEN no abilities exist in the Ability_Registry, THE Item_Editor SHALL display a message indicating no abilities are available in the "Granted Abilities" section.
10. THE Item_Registry SHALL persist granted ability data through serialization and deserialization such that a round-trip produces an identical list of ability IDs.

### Requirement 6: Character Visual Assets

**User Story:** As a game designer, I want to assign optional visual assets to each character (a spritesheet, a face portrait, and a status portrait), so that the built-in status page renderer and custom plugins can reference these assets from the character database.

#### Acceptance Criteria

1. THE Character_Registry SHALL store three optional file path fields for each character: spritesheet path, face portrait path, and status portrait path, each defaulting to None for newly created characters.
2. THE Character_Editor SHALL display a "Visual Assets" section in the character detail view containing single-line text input fields for spritesheet, face portrait, and status portrait, each labeled with its asset type.
3. WHEN no asset path is set for a given field, THE Character_Editor SHALL display a placeholder label reading "No asset assigned" in place of a file path value.
4. THE Character_Editor SHALL truncate each asset path text input to 260 characters, preventing the user from entering more than 260 characters in any single asset path field.
5. WHEN the user finishes editing an asset path field (on lost focus), THE Character_Editor SHALL trim leading and trailing whitespace from the input and store the resulting value as the character's asset path.
6. THE Character_Editor SHALL allow the user to clear any assigned asset path, resetting the field to None.
7. THE Character_Registry SHALL persist all three visual asset fields through serialization and deserialization such that a round-trip produces identical values (None remains None, a set path remains the same string).
8. IF the user enters an asset path that is empty or contains only whitespace after trimming, THEN THE Character_Editor SHALL treat the field as cleared (set to None) and SHALL NOT store an empty string value.
9. IF the user modifies any visual asset field, THEN THE Character_Editor SHALL mark the project as having unsaved character changes.

### Requirement 7: Character Visual Asset File Picker

**User Story:** As a game designer, I want a "Browse..." button next to each character visual asset text input, so that I can select image files from the filesystem without manually typing paths.

#### Acceptance Criteria

1. THE Character_Editor SHALL display a "Browse..." button adjacent to each visual asset text input field (spritesheet, face portrait, status portrait).
2. WHEN the user activates the "Browse..." button, THE Character_Editor SHALL open a native file dialog (using the `rfd` crate) filtered to image file types (png, jpg, jpeg).
3. WHEN the user selects a file in the native file dialog, THE Character_Editor SHALL populate the corresponding text buffer with the selected file path and commit the value to the character model immediately.
4. WHEN the user cancels the native file dialog without selecting a file, THE Character_Editor SHALL leave the corresponding text buffer unchanged.
5. THE Character_Editor SHALL apply the same 260-character truncation rule to file paths obtained via the File_Picker as it does to manually entered paths.
6. IF the user selects a file via the File_Picker, THEN THE Character_Editor SHALL mark the project as having unsaved character changes.
7. THE Character_Editor SHALL retain the existing text input for each visual asset field, allowing manual path entry alongside the File_Picker option.

### Requirement 8: Enemy Portrait File Picker

**User Story:** As a game designer, I want a "Browse..." button next to the enemy portrait text input, so that I can select an image file from the filesystem without manually typing the path.

#### Acceptance Criteria

1. THE Enemy_Editor SHALL display a "Browse..." button on the same horizontal row as the portrait text input field.
2. WHEN the user activates the "Browse..." button, THE Enemy_Editor SHALL open a native file dialog (using the `rfd` crate) filtered to image file types (png, jpg, jpeg).
3. WHEN the user selects a file in the native file dialog, THE Enemy_Editor SHALL populate the portrait text buffer with the absolute file path returned by the dialog, truncating to 260 characters, and SHALL commit the value to the Enemy_Registry via `set_portrait`.
4. WHEN the user cancels the native file dialog without selecting a file, THE Enemy_Editor SHALL leave the portrait text buffer and Enemy_Registry unchanged.
5. IF the file path returned by the dialog is empty after trimming, THEN THE Enemy_Editor SHALL display a validation error message indicating the path is invalid and SHALL NOT store the value in the Enemy_Registry.
6. IF the user selects a file via the File_Picker and the value is successfully stored, THEN THE Enemy_Editor SHALL mark the project as having unsaved enemy changes.

### Requirement 9: Character Starting Equipment

**User Story:** As a game designer, I want to define starting equipment for each character, so that characters begin the game with specific items already equipped or in their inventory.

#### Acceptance Criteria

1. THE Character_Registry SHALL store a list of Starting_Equipment item IDs for each character (maximum 20 entries), where each entry references an item in the Item_Registry, defaulting to an empty list for newly created characters.
2. THE Character_Editor SHALL display a "Starting Equipment" section below the "Learnable Abilities" section showing all assigned starting equipment items, each displayed as its display name followed by its category in brackets (e.g., "Iron Sword [Weapon]"), sorted case-insensitively by display name, with a remove button per entry.
3. WHEN the user adds a starting equipment item and the Item_Registry contains at least one item, THE Character_Editor SHALL present a Dropdown_Selector populated with items from the Item_Registry, displaying each item as its display name followed by its category in brackets (e.g., "Iron Sword [Weapon]"), sorted case-insensitively by display name.
4. IF the user attempts to assign an item ID that is empty or contains only whitespace after trimming, THEN THE Character_Editor SHALL reject the addition and display a validation error indicating the item ID is invalid.
5. IF the user attempts to assign an item ID that already exists in the character's starting equipment list, THEN THE Character_Editor SHALL reject the addition and display an error message indicating the item is already assigned.
6. IF the user attempts to add a starting equipment item when the character already has 20 starting equipment entries, THEN THE Character_Editor SHALL display an error message indicating the maximum limit has been reached and SHALL NOT add the item.
7. WHEN the user activates the remove button on a starting equipment entry, THE Character_Editor SHALL remove that item from the character's starting equipment list.
8. IF the Item_Registry contains no items, THEN THE Character_Editor SHALL display a message indicating no items are available in the "Starting Equipment" section instead of the Dropdown_Selector.
9. IF an item ID in the starting equipment list does not correspond to an existing item in the Item_Registry, THEN THE Character_Editor SHALL display the raw item ID as a fallback label for that entry.
10. THE Character_Registry SHALL persist starting equipment data through serialization and deserialization without data loss, using `#[serde(default)]` for backward compatibility with existing character files.
11. IF the user modifies the starting equipment list (addition or removal), THEN THE Character_Editor SHALL mark the project as having unsaved character changes.
