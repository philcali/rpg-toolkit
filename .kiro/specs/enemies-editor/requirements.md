# Requirements Document

## Introduction

The Enemies Editor introduces a dedicated editor mode for creating and managing enemy definitions in the RPG Toolkit. Enemies share a stat system similar to characters (HP, attack, defense, etc.) but include additional combat-specific data: rewards granted upon defeat (EXP, gold, item drops with probabilities), a list of carried items that can be obtained through various means during combat (stealing, trading, bartering, etc., influenced by skills), and elemental strengths/weaknesses that modify incoming damage. This feature adds an EnemyRegistry to the common crate and a new editor panel plugin following the established 3-panel layout pattern used by the Character, Item, and Ability editors.

## Glossary

- **Enemy**: A combat opponent defined with stats, defeat rewards, carried items, and elemental modifiers.
- **Enemy_Registry**: The project-level collection that stores and manages all defined enemies, analogous to CharacterRegistry, ItemRegistry, and AbilityRegistry.
- **Enemy_Editor**: The egui-based editor panel providing CRUD operations and visual editing for enemies.
- **Enemy_Stat**: A named numeric attribute on an enemy (e.g., HP, Attack, Defense, Speed) with a base value.
- **Defeat_Reward**: The rewards granted to the player upon defeating an enemy, comprising EXP, gold, and item drops.
- **Item_Drop**: An entry in the defeat reward specifying an item and the probability (0.0–1.0) that the item is dropped.
- **Carried_Item**: An entry in the carried items list specifying an item and the probability (0.0–1.0) that the item can be obtained through various means (steal, trade, barter) during combat, where the method of obtaining is influenced by the player's skills.
- **Elemental_Modifier**: A multiplier applied to incoming damage of a specific element, representing a strength (multiplier less than 1.0) or weakness (multiplier greater than 1.0).
- **Element**: A damage type category used in the elemental modifier system (e.g., Fire, Ice, Lightning, Wind, Earth, Light, Dark).

## Requirements

### Requirement 1: Enemy Data Model

**User Story:** As a game designer, I want to define enemies with structured properties, so that the game engine can use enemy data consistently in combat encounters.

#### Acceptance Criteria

1. THE Enemy_Registry SHALL store enemies in a HashMap keyed by EnemyId (UUID v4 string).
2. THE Enemy SHALL contain an id (EnemyId), display_name (String, 1–64 characters after trimming, containing at least one non-whitespace character), description (String, maximum 256 characters), stats (Vec of Enemy_Stat, maximum 20 entries), defeat_rewards (Defeat_Reward), carried_items (Vec of Carried_Item, maximum 8 entries), and elemental_modifiers (Vec of Elemental_Modifier, maximum one entry per Element variant).
3. THE Enemy_Stat SHALL contain a name (String, 1–32 characters, unique within the parent Enemy's stats Vec) and a base_value (u32).
4. THE Defeat_Reward SHALL contain exp (u32), gold (u32), and item_drops (Vec of Item_Drop, maximum 10 entries).
5. THE Item_Drop SHALL contain an item_id (ItemId) and a drop_chance (f64 in the range 0.0 to 1.0, inclusive).
6. THE Carried_Item SHALL contain an item_id (ItemId) and an obtain_chance (f64 in the range 0.0 to 1.0, inclusive).
7. THE Elemental_Modifier SHALL contain an element (Element enum variant) and a multiplier (f64 in the range 0.0 to 10.0, inclusive).
8. THE Element enum SHALL support the variants: Fire, Ice, Lightning, Wind, Earth, Light, and Dark.
9. THE Enemy SHALL derive Clone, Debug, PartialEq, Serialize, and Deserialize.
10. THE Enemy_Registry SHALL derive Clone, Debug, Default, PartialEq, Serialize, and Deserialize.
11. IF an Enemy field fails validation (display_name empty or exceeding 64 characters after trimming, stat name empty or exceeding 32 characters, duplicate stat name, elemental_modifiers containing duplicate Element variants, carried_items exceeding 8 entries, or item_drops exceeding 10 entries), THEN THE Enemy_Registry SHALL return an error indicating the specific validation failure without modifying the registry.

### Requirement 2: Enemy Creation

**User Story:** As a game designer, I want to create new enemies with a name, so that I can populate my game with combat encounters.

#### Acceptance Criteria

1. WHEN the user provides a display name, THE Enemy_Registry SHALL trim leading and trailing whitespace, validate the trimmed name is between 1 and 64 characters with at least one non-whitespace character, generate a UUID v4 identifier, create a new enemy with that identifier and the trimmed display_name, insert it into the enemies HashMap, and return the generated EnemyId.
2. IF the display name is empty or contains only whitespace after trimming, THEN THE Enemy_Registry SHALL return an EnemyValidationError without modifying the registry.
3. IF the display name exceeds 64 characters after trimming, THEN THE Enemy_Registry SHALL return an EnemyValidationError without modifying the registry.
4. WHEN an enemy is created, THE Enemy_Registry SHALL initialize stats with default required stats (HP with base_value 10, Attack with base_value 5, Defense with base_value 5, Speed with base_value 5), defeat_rewards with exp 0, gold 0, and an empty item_drops Vec, carried_items as an empty Vec, elemental_modifiers as an empty Vec, and description as an empty String.

### Requirement 3: Enemy Deletion

**User Story:** As a game designer, I want to remove enemies I no longer need, so that I can keep my enemy list organized.

#### Acceptance Criteria

1. WHEN the user requests deletion of an enemy by ID, THE Enemy_Registry SHALL remove the enemy from the registry and return a success result, such that subsequent lookups by that ID return a not-found error and subsequent listings no longer include the deleted enemy.
2. IF the specified enemy ID does not exist in the registry, THEN THE Enemy_Registry SHALL return an EnemyValidationError whose message includes the ID that was not found.

### Requirement 4: Enemy Field Updates

**User Story:** As a game designer, I want to edit enemy properties, so that I can fine-tune enemies for game balance.

#### Acceptance Criteria

1. WHEN the user updates a display name, THE Enemy_Registry SHALL trim leading and trailing whitespace from the provided name, validate it using the same rules as creation (1–64 characters after trimming, at least one non-whitespace character), and store the trimmed result as the enemy's display_name.
2. IF the updated display name is empty, contains only whitespace after trimming, or exceeds 64 characters after trimming, THEN THE Enemy_Registry SHALL return an EnemyValidationError without modifying the enemy.
3. WHEN the user updates a description, THE Enemy_Registry SHALL store the first 256 characters (by Unicode codepoint count) of the provided description on the enemy, accepting an empty string as a valid description.
4. IF the enemy ID does not exist in the registry during any update operation, THEN THE Enemy_Registry SHALL return an EnemyValidationError indicating the ID that was not found.

### Requirement 5: Enemy Stat Management

**User Story:** As a game designer, I want to add, remove, and modify enemy stats, so that I can customize each enemy's combat profile.

#### Acceptance Criteria

1. WHEN the user adds a stat to an enemy, THE Enemy_Registry SHALL trim the provided name, validate it, and append an Enemy_Stat with the trimmed name and a base_value of 0 to the enemy's stats list.
2. IF the user attempts to add a stat with a trimmed name shorter than 1 character or longer than 32 characters, THEN THE Enemy_Registry SHALL return an EnemyValidationError.
3. IF the user attempts to add a stat with a name that already exists on the enemy (case-sensitive comparison on the trimmed name), THEN THE Enemy_Registry SHALL return an EnemyValidationError.
4. IF the user attempts to add a stat that would cause the enemy to exceed 20 stats, THEN THE Enemy_Registry SHALL return an EnemyValidationError and the stat SHALL NOT be added.
5. WHEN the user updates a stat's base_value, THE Enemy_Registry SHALL accept an unsigned 32-bit integer (0 to 4,294,967,295) and store the new value on the matching stat.
6. IF the specified stat name does not exist on the enemy when updating or removing, THEN THE Enemy_Registry SHALL return an EnemyValidationError.
7. WHEN the user removes a stat from an enemy, THE Enemy_Registry SHALL remove the stat entry with the matching name (case-sensitive comparison on the trimmed name).
8. THE Enemy_Registry SHALL prevent removal of the required stat "HP" from any enemy.
9. IF the user attempts to remove a required stat, THEN THE Enemy_Registry SHALL return an EnemyValidationError indicating that the stat is required.

### Requirement 6: Defeat Rewards Management

**User Story:** As a game designer, I want to configure rewards for defeating an enemy, so that players receive appropriate compensation for combat.

#### Acceptance Criteria

1. WHEN the user updates the exp value on an enemy's defeat_rewards, THE Enemy_Registry SHALL store the new exp value on the enemy.
2. WHEN the user updates the gold value on an enemy's defeat_rewards, THE Enemy_Registry SHALL store the new gold value on the enemy.
3. WHEN the user adds an item drop to an enemy's defeat_rewards, THE Enemy_Registry SHALL validate that the item_id is not empty after trimming and that drop_chance is between 0.0 and 1.0 inclusive, then append the Item_Drop with the trimmed item_id and the provided drop_chance.
4. IF the drop_chance is outside the range 0.0 to 1.0, THEN THE Enemy_Registry SHALL return an EnemyValidationError without modifying the item_drops list.
5. IF the item_id is empty after trimming, THEN THE Enemy_Registry SHALL return an EnemyValidationError without modifying the item_drops list.
6. IF the user attempts to add an item drop when the enemy already has 10 item drops, THEN THE Enemy_Registry SHALL return an EnemyValidationError without modifying the item_drops list.
7. WHEN the user removes an item drop by index, THE Enemy_Registry SHALL remove the Item_Drop at that position from the item_drops list.
8. IF the item drop index is out of bounds, THEN THE Enemy_Registry SHALL return an EnemyValidationError.
9. IF the enemy ID does not exist in the registry during any defeat reward operation, THEN THE Enemy_Registry SHALL return an EnemyValidationError indicating the ID that was not found.

### Requirement 7: Carried Items Management

**User Story:** As a game designer, I want to define items that enemies carry, so that players can obtain them through various means (stealing, trading, bartering) influenced by their skills.

#### Acceptance Criteria

1. WHEN the user adds a carried item to an enemy, THE Enemy_Registry SHALL validate that the item_id is not empty after trimming and that obtain_chance is between 0.0 and 1.0 inclusive, then append the Carried_Item with the trimmed item_id and the provided obtain_chance.
2. IF the obtain_chance is outside the range 0.0 to 1.0, THEN THE Enemy_Registry SHALL return an EnemyValidationError without modifying the carried_items list.
3. IF the item_id is empty after trimming, THEN THE Enemy_Registry SHALL return an EnemyValidationError without modifying the carried_items list.
4. IF the user attempts to add a carried item when the enemy already has 8 entries, THEN THE Enemy_Registry SHALL return an EnemyValidationError without modifying the carried_items list.
5. WHEN the user removes a carried item by index, THE Enemy_Registry SHALL remove the Carried_Item at that position from the carried_items list.
6. IF the carried item index is out of bounds, THEN THE Enemy_Registry SHALL return an EnemyValidationError.
7. IF the enemy ID does not exist in the registry during any carried items operation, THEN THE Enemy_Registry SHALL return an EnemyValidationError indicating the ID that was not found.

### Requirement 8: Elemental Modifier Management

**User Story:** As a game designer, I want to assign elemental strengths and weaknesses to enemies, so that I can create strategic combat encounters.

#### Acceptance Criteria

1. WHEN the user adds an elemental modifier to an enemy, THE Enemy_Registry SHALL append the Elemental_Modifier with the provided element and multiplier.
2. IF the multiplier is negative, THEN THE Enemy_Registry SHALL return an EnemyValidationError without modifying the elemental_modifiers list.
3. IF the user attempts to add an elemental modifier for an element that already has a modifier on the enemy, THEN THE Enemy_Registry SHALL return an EnemyValidationError indicating the duplicate element.
4. WHEN the user updates an existing elemental modifier's multiplier, THE Enemy_Registry SHALL validate that the new multiplier is greater than or equal to 0.0, then store the new multiplier value for the matching element.
5. IF the new multiplier is negative during an update, THEN THE Enemy_Registry SHALL return an EnemyValidationError without modifying the existing modifier.
6. WHEN the user removes an elemental modifier by element, THE Enemy_Registry SHALL remove the Elemental_Modifier with the matching element variant.
7. IF the specified element does not have a modifier on the enemy during an update or remove operation, THEN THE Enemy_Registry SHALL return an EnemyValidationError.
8. IF the enemy ID does not exist in the registry during any elemental modifier operation, THEN THE Enemy_Registry SHALL return an EnemyValidationError indicating the ID that was not found.

### Requirement 9: Enemy Listing and Filtering

**User Story:** As a game designer, I want to browse enemies in an organized list, so that I can quickly find enemies to edit.

#### Acceptance Criteria

1. THE Enemy_Registry SHALL provide a sorted listing method that returns a Vec of references to Enemy, sorted case-insensitively by display_name, with ties broken by lexicographic (byte-order) comparison of the original display_name.
2. IF the Enemy_Registry contains no enemies, THEN the sorted listing method SHALL return an empty Vec.
3. WHEN the user provides a non-empty search string, THE Enemy_Registry SHALL return only enemies whose display_name contains the search string using case-insensitive substring matching.
4. IF the search string is empty or contains only whitespace, THEN THE Enemy_Registry SHALL return the full sorted listing without filtering.

### Requirement 10: Project Integration

**User Story:** As a game designer, I want enemies to persist with my project, so that my work is saved and loaded correctly.

#### Acceptance Criteria

1. THE Project resource SHALL contain an enemies field of type Enemy_Registry.
2. THE Project resource SHALL contain a has_unsaved_enemy_changes boolean flag, initialized to false.
3. THE ProjectFile SHALL include an enemies field annotated with the serde default attribute so that deserializing a project file that does not contain the enemies field succeeds with an empty Enemy_Registry.
4. THE ProjectManifest SHALL include an enemies field annotated with the serde default attribute so that deserializing a manifest that does not contain the enemies field succeeds with an empty Enemy_Registry.
5. WHEN the Project is converted to a ProjectFile or ProjectManifest for saving, THE system SHALL include the enemies field in the serialized output so that a subsequent load restores the same Enemy_Registry contents.
6. WHEN a ProjectFile containing enemies is deserialized, THE system SHALL validate that each enemy registry key matches the enemy's own id field and return a validation error if any key does not match.

### Requirement 11: Editor Mode Integration

**User Story:** As a game designer, I want to switch to the Enemies editor mode, so that I can manage enemies alongside other game data.

#### Acceptance Criteria

1. THE AppEditorMode enum SHALL include an Enemy variant.
2. WHEN the user selects the Enemy entry from the Mode menu in the app shell, THE system SHALL set the AppEditorMode resource to Enemy.
3. WHILE the AppEditorMode resource is set to Enemy, THE Enemy_Editor panel SHALL be displayed in the central area.
4. WHILE the AppEditorMode resource is not set to Enemy, THE Enemy_Editor panel SHALL not render.
5. IF the AppEditorMode resource is set to Enemy and no other Enemy-specific state exists, THEN THE Enemy_Editor panel SHALL display an empty state without errors.

### Requirement 12: Enemy Editor List Panel

**User Story:** As a game designer, I want a list of enemies on the left side, so that I can browse and select enemies to edit.

#### Acceptance Criteria

1. THE Enemy_Editor SHALL display a left side panel with a default width of 220 pixels containing the enemy list sorted case-insensitively by display_name within a vertical scroll area.
2. WHEN the user selects an enemy from the list, THE Enemy_Editor SHALL display that enemy's details in the central panel and sync the editing state buffers to the selected enemy's current field values.
3. THE Enemy_Editor SHALL display each list entry as a selectable label showing the enemy's display_name, with a delete button (🗑) per entry.
4. THE Enemy_Editor SHALL provide a "Create" button at the top of the list panel that opens a creation dialog containing a text field for the display_name (validated with the same trimming and 1–64 character rules as Enemy_Registry creation), a "Create" confirmation button, and a "Cancel" button.
5. WHEN the user confirms creation in the dialog with a valid name, THE Enemy_Editor SHALL create the enemy, close the dialog, auto-select the newly created enemy in the list, and set has_unsaved_enemy_changes to true.
6. IF the creation dialog name is invalid (empty after trimming or exceeds 64 characters), THEN THE Enemy_Editor SHALL display the validation error in the dialog using colored_label with Color32::RED and keep the dialog open.
7. WHEN the user clicks the delete button on a list entry, THE Enemy_Editor SHALL display a confirmation dialog showing the enemy's display_name with "Confirm" and "Cancel" buttons before removing the enemy.
8. WHEN the user confirms deletion and the deleted enemy was currently selected, THE Enemy_Editor SHALL clear the selection so that no enemy is selected.
9. WHEN an enemy is deleted, THE Enemy_Editor SHALL set has_unsaved_enemy_changes to true.
10. IF no enemies exist in the registry, THEN THE Enemy_Editor SHALL display the message "No enemies yet. Create one to get started." in the list area.

### Requirement 13: Enemy Editor Detail Panel

**User Story:** As a game designer, I want to edit all enemy fields in the central panel, so that I can fully configure each enemy.

#### Acceptance Criteria

1. WHEN an enemy is selected, THE Enemy_Editor SHALL display a text_edit_singleline for display_name, a multiline TextEdit for description that truncates input to 256 characters as the user types, a stats section with a grid showing stat name and base_value with DragValue widgets (range 0 to u32::MAX), a defeat rewards section with DragValue fields for exp (range 0 to u32::MAX) and gold (range 0 to u32::MAX) and a list of item drops where each entry shows item_id and drop_chance (DragValue, range 0.0 to 1.0), a carried items section listing carried item entries where each entry shows item_id and obtain_chance (DragValue, range 0.0 to 1.0), and an elemental modifiers section listing entries where each entry shows element name and multiplier (DragValue, range 0.0 to f64::MAX).
2. WHEN the display_name text_edit_singleline loses focus, THE Enemy_Editor SHALL validate the field by trimming whitespace and requiring 1–64 characters with at least one non-whitespace character, and display validation errors as red text below the field.
3. THE Enemy_Editor SHALL truncate the display_name input to 64 characters as the user types to prevent exceeding the maximum length.
4. THE Enemy_Editor SHALL provide buttons to add and remove stats, item drops, carried items, and elemental modifiers in their respective sections.
5. WHEN any field is modified, THE Enemy_Editor SHALL set has_unsaved_enemy_changes to true.
6. IF a validation error occurs during editing, THEN THE Enemy_Editor SHALL display the error message inline below the relevant field using colored_label with Color32::RED.
7. WHILE no enemy is selected, THE Enemy_Editor SHALL display the text "Select an enemy to edit, or create a new one." in the central panel.

### Requirement 14: Enemy Editor Preview Panel

**User Story:** As a game designer, I want a preview summary of the selected enemy on the right side, so that I can see an overview while editing.

#### Acceptance Criteria

1. WHEN an enemy is selected, THE Enemy_Editor SHALL display a right side panel with a 250-pixel default width showing a heading "Enemy Preview", followed by read-only labels for the enemy's display_name, a stat summary listing each stat name and base_value (or the text "No stats defined." if the stats list is empty), a defeat rewards summary showing exp, gold, and the count of item drops, a carried items summary showing the count of carried item entries, and an elemental modifiers summary listing each element and its multiplier (or the text "No elemental modifiers." if the list is empty).
2. WHILE no enemy is selected, THE Enemy_Editor SHALL display the text "Select an enemy to preview." in the preview panel.
3. WHEN enemy fields are modified in the central panel, THE Enemy_Editor SHALL reflect the updated values in the preview panel within the same render frame.
4. WHEN the currently selected enemy is deleted, THE Enemy_Editor SHALL clear the preview panel and display the text "Select an enemy to preview."

### Requirement 15: Enemy Serialization Round-Trip

**User Story:** As a game designer, I want enemies to serialize and deserialize without data loss, so that my project data remains intact.

#### Acceptance Criteria

1. THE Enemy_Registry SHALL produce a structurally equal instance (via PartialEq) when serialized to JSON with serde_json and then deserialized back, for any registry containing enemies that satisfy the validation rules defined in Requirements 1 through 8, including f64 fields (drop_chance, obtain_chance, multiplier) that are finite and within their specified valid ranges.
2. THE Enemy_Registry SHALL serialize using serde with the Element enum as unit variants (serialized as bare strings such as "Fire", "Ice"), consistent with the derive-based Serialize/Deserialize pattern used by ItemRegistry, CharacterRegistry, and AbilityRegistry.
3. IF the JSON input is malformed or contains values that violate the Enemy data model (unknown enum variants, missing required fields, or type mismatches), THEN the deserialization SHALL return a serde_json::Error rather than silently dropping data or producing a default Enemy_Registry.
4. THE Enemy_Registry round-trip property SHALL be verified using property-based testing that generates arbitrary valid registries containing between 0 and 50 enemies, where each enemy has between 1 and 20 stats, between 0 and 10 item drops with drop_chance values in 0.0 to 1.0, between 0 and 8 carried items with obtain_chance values in 0.0 to 1.0, and between 0 and 7 elemental modifiers with multiplier values of 0.0 or greater using finite f64 values only.
