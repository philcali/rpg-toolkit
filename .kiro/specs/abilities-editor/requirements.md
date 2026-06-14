# Requirements Document

## Introduction

The Abilities Editor introduces a unified concept of "abilities" to the RPG Toolkit. Abilities encompass skills, magic spells, and special actions that characters can acquire through multiple sources: learning from items, gaining from equipped weapons or armor, receiving from accessories, or unlocking through level progression. This feature adds an AbilityRegistry to the common crate and a dedicated editor panel following the established 3-panel layout pattern (list panel, detail editor, preview panel).

## Glossary

- **Ability**: A unified game mechanic representing a skill, spell, or special action that a character can use during gameplay.
- **Ability_Registry**: The project-level collection that stores and manages all defined abilities, analogous to ItemRegistry and CharacterRegistry.
- **Ability_Editor**: The egui-based editor panel providing CRUD operations and visual editing for abilities.
- **Ability_Source**: The mechanism through which a character acquires an ability (level-up, item learning, equipment grant, or accessory grant).
- **Ability_Category**: A classification grouping abilities by type: Skill, Spell, or Special_Action.
- **Target_Type**: Defines who or what an ability affects when used: single ally, all allies, single enemy, all enemies, or self.
- **Cost_Type**: The resource consumed when an ability is used, such as MP or HP.
- **Editor_Panel**: The 3-panel UI layout consisting of a left list panel, central detail editor, and right preview panel.

## Requirements

### Requirement 1: Ability Data Model

**User Story:** As a game designer, I want to define abilities with structured properties, so that the game engine can process ability effects consistently.

#### Acceptance Criteria

1. THE Ability_Registry SHALL store abilities in a HashMap keyed by AbilityId (UUID v4 string).
2. THE Ability SHALL contain an id (AbilityId), display_name (String, 1–64 non-whitespace characters after trimming), description (String, maximum 256 characters), category (Ability_Category), cost_type (Cost_Type), cost_value (u32), power (u32), target_type (Target_Type), and sources (Vec of Ability_Source, maximum 10 entries).
3. THE Ability_Category SHALL support the variants: Skill, Spell, and Special_Action.
4. THE Target_Type SHALL support the variants: SingleAlly, AllAllies, SingleEnemy, AllEnemies, and Self_Target.
5. THE Cost_Type SHALL support the variants: MP and HP.
6. THE Ability_Source SHALL support the variants: LevelUp with a required_level (u32, minimum value of 1) field, LearnedFromItem with an item_id (ItemId) field, EquipmentGrant with an item_id (ItemId) field, and AccessoryGrant with an item_id (ItemId) field.
7. THE Ability SHALL derive Clone, Debug, PartialEq, Eq, Serialize, and Deserialize.
8. THE Ability_Registry SHALL derive Clone, Debug, Default, PartialEq, Eq, Serialize, and Deserialize.

### Requirement 2: Ability Creation

**User Story:** As a game designer, I want to create new abilities with a name and category, so that I can build out my game's ability system.

#### Acceptance Criteria

1. WHEN the user provides a display name and category, THE Ability_Registry SHALL create a new ability with a generated UUID v4 identifier, insert it into the abilities HashMap, and return the generated AbilityId.
2. THE Ability_Registry SHALL validate display names by trimming leading and trailing whitespace and requiring the trimmed result to be between 1 and 64 characters in length.
3. IF the display name is empty or contains only whitespace after trimming, THEN THE Ability_Registry SHALL return an AbilityValidationError.
4. IF the display name exceeds 64 characters after trimming, THEN THE Ability_Registry SHALL return an AbilityValidationError.
5. WHEN an ability is created, THE Ability_Registry SHALL store the user-provided Ability_Category on the ability and initialize cost_value to 0, power to 0, target_type to SingleEnemy, cost_type to MP, sources to an empty Vec, and description to an empty String.

### Requirement 3: Ability Deletion

**User Story:** As a game designer, I want to remove abilities I no longer need, so that I can keep my ability list clean.

#### Acceptance Criteria

1. WHEN the user requests deletion of an ability by ID, THE Ability_Registry SHALL remove the ability from the registry such that subsequent lookups or listings no longer include the deleted ability.
2. IF the specified ability ID does not exist in the registry, THEN THE Ability_Registry SHALL return an AbilityValidationError indicating the ID that was not found.

### Requirement 4: Ability Field Updates

**User Story:** As a game designer, I want to edit ability properties, so that I can fine-tune abilities for balance.

#### Acceptance Criteria

1. WHEN the user updates a display name, THE Ability_Registry SHALL validate the new name using the same rules as creation (trimmed, 1-64 non-whitespace characters after trimming).
2. IF the updated display name is empty or exceeds 64 characters after trimming, THEN THE Ability_Registry SHALL return an AbilityValidationError.
3. WHEN the user updates a description, THE Ability_Registry SHALL store the first 256 characters (by Unicode codepoint count) of the provided description on the ability.
4. WHEN the user updates category, cost_type, target_type, power, or cost_value, THE Ability_Registry SHALL store the new value on the ability without additional validation beyond type constraints.
5. IF the ability ID does not exist in the registry during any update operation, THEN THE Ability_Registry SHALL return an AbilityValidationError.

### Requirement 5: Ability Source Management

**User Story:** As a game designer, I want to define how characters acquire abilities, so that I can create meaningful progression paths.

#### Acceptance Criteria

1. WHEN the user adds a source to an ability, THE Ability_Registry SHALL append the Ability_Source to the ability's sources list.
2. IF the ability ID does not exist in the registry during a source add or remove operation, THEN THE Ability_Registry SHALL return an AbilityValidationError.
3. THE Ability_Registry SHALL enforce a maximum of 10 sources per ability.
4. IF the user attempts to add a source when the ability already has 10 sources, THEN THE Ability_Registry SHALL return an AbilityValidationError.
5. WHEN the user removes a source by index, THE Ability_Registry SHALL remove the source at that position.
6. IF the source index is out of bounds, THEN THE Ability_Registry SHALL return an AbilityValidationError.
7. WHEN a LevelUp source is added, THE Ability_Registry SHALL validate that required_level is at least 1.
8. IF a LevelUp source has required_level of 0, THEN THE Ability_Registry SHALL return an AbilityValidationError.
9. WHEN a LearnedFromItem, EquipmentGrant, or AccessoryGrant source is added, THE Ability_Registry SHALL validate that item_id is not empty after trimming.

### Requirement 6: Ability Listing and Filtering

**User Story:** As a game designer, I want to browse and filter abilities by category, so that I can quickly find what I need.

#### Acceptance Criteria

1. THE Ability_Registry SHALL provide a filtered listing method that accepts an optional Ability_Category parameter and returns a Vec of references to Ability.
2. WHEN a category filter is provided, THE Ability_Registry SHALL return only abilities whose category matches the specified Ability_Category, sorted case-insensitively by display name.
3. WHEN no category filter is provided, THE Ability_Registry SHALL return all abilities sorted case-insensitively by display name.
4. IF the Ability_Registry contains no abilities matching the filter criteria, THEN THE Ability_Registry SHALL return an empty Vec.

### Requirement 7: Project Integration

**User Story:** As a game designer, I want abilities to persist with my project, so that my work is saved and loaded correctly.

#### Acceptance Criteria

1. THE Project resource SHALL contain an abilities field of type Ability_Registry.
2. THE Project resource SHALL contain a has_unsaved_ability_changes boolean flag, initialized to false.
3. THE ProjectFile SHALL include an abilities field annotated with the serde default attribute so that deserializing a project file that does not contain the abilities field succeeds with an empty Ability_Registry.
4. WHEN a project is deserialized, THE ProjectFile SHALL validate that each ability registry key matches the corresponding ability's id field.
5. IF an ability registry key does not match the ability id, THEN THE ProjectFile SHALL return a ProjectValidationError with a message indicating the mismatched registry key and the ability id.

### Requirement 8: Editor Mode Integration

**User Story:** As a game designer, I want to switch to the Abilities editor mode, so that I can manage abilities alongside other game data.

#### Acceptance Criteria

1. THE AppEditorMode enum SHALL include an Ability variant.
2. WHEN the user selects the Ability entry from the Mode menu in the app shell, THE system SHALL set the AppEditorMode resource to Ability.
3. WHILE the AppEditorMode resource is set to Ability, THE Ability_Editor panel SHALL be displayed in the central area.
4. WHILE the AppEditorMode resource is not set to Ability, THE Ability_Editor panel SHALL not render.
5. IF the AppEditorMode resource is set to Ability and no other Ability-specific state exists, THEN THE Ability_Editor panel SHALL display an empty state without errors.

### Requirement 9: Ability Editor List Panel

**User Story:** As a game designer, I want a filterable list of abilities on the left side, so that I can browse and select abilities to edit.

#### Acceptance Criteria

1. THE Ability_Editor SHALL display a left side panel with a default width of 220 pixels containing the ability list sorted case-insensitively by display name.
2. THE Ability_Editor SHALL provide a ComboBox category filter with an "All" option (default) and one option per Ability_Category variant to filter the displayed abilities.
3. WHEN the category filter changes and the currently selected ability is not in the filtered results, THE Ability_Editor SHALL auto-select the first visible ability in the filtered list.
4. WHEN the user selects an ability from the list, THE Ability_Editor SHALL display that ability's details in the central panel.
5. THE Ability_Editor SHALL display each list entry as a selectable label showing the ability's display_name and category, with a delete button (🗑) per entry.
6. THE Ability_Editor SHALL provide a "Create" button at the top of the list panel that opens a creation dialog.
7. WHEN the user clicks the delete button on a list entry, THE Ability_Editor SHALL display a confirmation prompt before removing the ability.
8. WHEN an ability is created or deleted, THE Ability_Editor SHALL set has_unsaved_ability_changes to true.
9. IF no abilities exist in the registry, THEN THE Ability_Editor SHALL display the message "No abilities yet. Create one to get started." in the list area.

### Requirement 10: Ability Editor Detail Panel

**User Story:** As a game designer, I want to edit all ability fields in the central panel, so that I can configure abilities fully.

#### Acceptance Criteria

1. WHEN an ability is selected, THE Ability_Editor SHALL display a text_edit_singleline for display_name, a multiline TextEdit for description (max 256 characters), ComboBox widgets for category, cost_type, and target_type, and DragValue widgets for cost_value and power in the central panel.
2. THE Ability_Editor SHALL validate the display_name field on lost focus, applying the same trimming and 1-64 character rules as the registry, and display validation errors as red text below the field.
3. THE Ability_Editor SHALL display a sources section allowing the user to add a new source (via a dialog or inline form), view all existing sources with their details, and remove a source by clicking a delete button per entry.
4. WHEN any field is modified, THE Ability_Editor SHALL set has_unsaved_ability_changes to true.
5. IF a validation error occurs during editing, THEN THE Ability_Editor SHALL display the error message inline near the relevant field using colored_label with Color32::RED.
6. THE Ability_Editor SHALL truncate the display_name input to 64 characters as the user types to prevent exceeding the maximum length.

### Requirement 11: Ability Editor Preview Panel

**User Story:** As a game designer, I want a preview of the selected ability on the right side, so that I can see a summary while editing.

#### Acceptance Criteria

1. WHEN an ability is selected, THE Ability_Editor SHALL display a right side panel with a 250-pixel default width showing read-only labels for the ability's display_name, category, cost (formatted as cost_value followed by cost_type, e.g. "10 MP"), power, and target_type.
2. WHEN an ability is selected, THE Ability_Editor SHALL display a sources section in the preview panel listing each Ability_Source entry with its variant name and relevant detail (required_level for LevelUp, item_id for LearnedFromItem, EquipmentGrant, and AccessoryGrant).
3. WHILE no ability is selected, THE Ability_Editor SHALL display the text "Select an ability to preview." in the preview panel.
4. WHEN ability fields are modified in the central panel, THE Ability_Editor SHALL immediately reflect the updated values in the preview panel.

### Requirement 12: Ability Serialization Round-Trip

**User Story:** As a game designer, I want abilities to serialize and deserialize without data loss, so that my project data remains intact.

#### Acceptance Criteria

1. THE Ability_Registry SHALL produce a structurally equal instance (via PartialEq) when serialized to JSON with serde_json and then deserialized back, for any registry containing abilities that satisfy the validation rules defined in Requirements 1 through 5.
2. THE Ability_Registry SHALL serialize using serde with internally-tagged enum representation: Ability_Source using `#[serde(tag = "source_type")]` and Ability_Category, Target_Type, and Cost_Type as unit-variant enums, consistent with the conventions used by ItemRegistry and CharacterRegistry.
3. IF the JSON input is malformed or contains values that violate the Ability data model (unknown enum variants, missing required fields, or type mismatches), THEN the deserialization SHALL return an error rather than silently dropping data or producing a default Ability_Registry.
4. THE Ability_Registry round-trip property SHALL be verified using property-based testing that generates arbitrary valid registries containing between 0 and 50 abilities, each with between 0 and 10 sources.
