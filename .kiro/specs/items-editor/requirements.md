# Requirements Document

## Introduction

The Items Editor feature extends the RPG Toolkit to support defining game items — weapons, armor, consumables, accessories, and key items — through a dedicated editor mode. Currently the toolkit has no item system. This feature introduces an item data model in `rpg-toolkit-common` with type-specific properties, stat modifiers, rarity tiers, and equipment slot assignments. It integrates item data into the project manifest and serialization pipeline, and provides an egui-based editor panel following the same left-list / center-editor / right-preview layout established by the Character Editor. Items are foundational for shops, inventories, loot tables, and equipment screens.

## Glossary

- **Editor**: The Bevy/egui-based RPG Toolkit Editor application (`rpg-toolkit-editor` crate)
- **Item**: A named game object that can be found, purchased, equipped, or consumed by characters
- **Item_Registry**: The project-level collection of all defined items, stored as a HashMap keyed by Item_Id
- **Item_Id**: A unique UUID v4 string identifier assigned to each item on creation
- **Item_Category**: The classification of an item that determines its type-specific properties (Weapon, Armor, Accessory, Consumable, Key_Item)
- **Stat_Modifier**: A named numeric modifier that an item applies to a character stat when equipped (e.g., +5 Strength)
- **Equipment_Slot**: A named slot on a character where an item can be equipped (Main_Hand, Off_Hand, Head, Body, Legs, Feet, Accessory_1, Accessory_2)
- **Rarity**: A tier classification for items that indicates relative power or scarcity (Common, Uncommon, Rare, Epic, Legendary)
- **Stackable**: A property indicating whether multiple instances of an item occupy a single inventory slot with a quantity counter
- **Stack_Limit**: The maximum number of a stackable item that can occupy a single inventory slot
- **Consumable_Effect**: A named effect that a consumable item applies when used (Restore_HP, Restore_MP, Cure_Status, Buff_Stat)
- **Item_Panel**: The egui editor UI panel for managing items, rendered when AppEditorMode is set to ItemEditor
- **Project_File**: The on-disk serialization format for the complete RPG project (`ProjectFile` struct)
- **Project_Manifest**: The lightweight directory-based manifest format (`ProjectManifest` struct)

## Requirements

### Requirement 1: Item Data Model

**User Story:** As a toolkit developer, I want a well-defined item data structure in the common crate, so that items can be shared between the editor and renderer.

#### Acceptance Criteria

1. THE Item_Registry SHALL store zero or more Item entries, each identified by a unique Item_Id of type UUID v4 string
2. THE Item SHALL contain a display name field of type string with a minimum length of 1 non-whitespace character and a maximum length of 64 characters after trimming leading and trailing whitespace
3. THE Item SHALL contain a description field of type string with a maximum length of 256 characters that may be empty
4. THE Item SHALL contain exactly one Item_Category value from the set: Weapon, Armor, Accessory, Consumable, Key_Item
5. THE Item SHALL contain a value field of type unsigned 32-bit integer representing the base price of the item
6. THE Item SHALL contain exactly one Rarity value from the set: Common, Uncommon, Rare, Epic, Legendary
7. THE Item SHALL contain a stackable field of type boolean indicating whether the item can stack in inventory
8. WHEN an Item has stackable set to true, THE Item SHALL contain a stack_limit field of type unsigned 32-bit integer with a minimum value of 2 and a maximum value of 999
9. WHEN an Item has stackable set to false, THE Item SHALL have a stack_limit of 1
10. THE Item SHALL contain a collection of zero or more Stat_Modifier entries up to a maximum of 20, each with a stat name of 1 to 32 characters and a modifier value of type signed 32-bit integer
11. IF a Stat_Modifier is added with a stat name that already exists on the Item, THEN THE Item_Registry SHALL reject the addition and return an error indicating a duplicate stat modifier
12. WHEN an Item is created, THE Item_Registry SHALL assign a unique UUID v4 identifier to the Item
13. IF an Item is created with a display name that is empty, whitespace-only, or exceeds 64 characters after trimming, THEN THE Item_Registry SHALL reject the creation and return an error indicating an invalid display name
14. IF an Item is created with a description that exceeds 256 characters, THEN THE Item_Registry SHALL reject the creation and return an error indicating an invalid description

### Requirement 2: Item Category Properties

**User Story:** As a game designer, I want items to have type-specific properties based on their category, so that weapons feel different from armor and consumables have distinct behavior.

#### Acceptance Criteria

1. WHEN an Item has Item_Category of Weapon, THE Item SHALL contain an attack_power field of type unsigned 32-bit integer with a minimum value of 0 and a maximum value of 4,294,967,295
2. WHEN an Item has Item_Category of Weapon, THE Item SHALL contain an equipment_slot field restricted to the value Main_Hand or Off_Hand
3. WHEN an Item has Item_Category of Armor, THE Item SHALL contain a defense_power field of type unsigned 32-bit integer with a minimum value of 0 and a maximum value of 4,294,967,295
4. WHEN an Item has Item_Category of Armor, THE Item SHALL contain an equipment_slot field restricted to the values Head, Body, Legs, or Feet
5. WHEN an Item has Item_Category of Accessory, THE Item SHALL contain an equipment_slot field restricted to the values Accessory_1 or Accessory_2
6. WHEN an Item has Item_Category of Consumable, THE Item SHALL contain a collection of one or more Consumable_Effect entries with a maximum of 4 entries
7. WHEN an Item has Item_Category of Consumable, THE Item SHALL have stackable set to true
8. WHEN an Item has Item_Category of Key_Item, THE Item SHALL have stackable set to false
9. WHEN an Item has Item_Category of Key_Item, THE Item SHALL have a value of 0
10. THE Consumable_Effect SHALL contain an effect_type from the set: Restore_HP, Restore_MP, Cure_Status, Buff_Stat
11. THE Consumable_Effect SHALL contain a potency field of type unsigned 32-bit integer with a minimum value of 1 and a maximum value of 4,294,967,295 representing the magnitude of the effect
12. WHEN a Consumable_Effect has effect_type of Buff_Stat, THE Consumable_Effect SHALL contain a target_stat field with a value from the set: Strength, Stamina, Speed, Luck, Wisdom, Intelligence
13. WHEN a Consumable_Effect has effect_type of Buff_Stat, THE Consumable_Effect SHALL contain a duration field of type unsigned 32-bit integer representing the number of turns the buff remains active, with a minimum value of 1 and a maximum value of 99
14. WHEN a Consumable_Effect has effect_type of Cure_Status, THE Consumable_Effect SHALL contain a target_status field with a value from the set: Poison, Paralysis, Sleep, Confusion, Silence, All

### Requirement 3: Item Serialization

**User Story:** As a toolkit developer, I want items to serialize and deserialize as part of the project file, so that item data persists across editor sessions.

#### Acceptance Criteria

1. THE Project_File SHALL include the Item_Registry in the serialized JSON output under an "items" key
2. WHEN a Project_File is serialized and then deserialized, THE Item_Registry SHALL contain the same set of items with identical field values including Item_Id, display name, and category-specific properties (round-trip property)
3. WHEN a Project_File with no "items" key is deserialized, THE Project_File SHALL default to an empty Item_Registry using serde default attribute
4. WHEN an Item_Registry contains duplicate Item_Id values during deserialization, THE Project_File SHALL return a CommonError::ProjectValidationError indicating which Item_Id is duplicated
5. THE Project_File SHALL serialize the Item_Registry to the Project_Manifest for directory-based storage under an "items" key with the same structure as the single-file JSON format
6. THE Item_Registry SHALL serialize category-specific properties using serde tagged enum representation to preserve Item_Category type information across serialization cycles
7. WHEN an Item_Registry entry contains an Item_Id key that does not match the Item_Id stored within the item value during deserialization, THE Project_File SHALL return a CommonError::ProjectValidationError indicating the mismatched identifiers

### Requirement 4: Item Creation

**User Story:** As a game designer, I want to create new items in the editor, so that I can build my game's item database.

#### Acceptance Criteria

1. WHEN the user activates the create item action, THE Item_Panel SHALL display a form for entering an item display name and selecting an Item_Category, with no Item_Category pre-selected
2. WHEN the user confirms item creation with a valid display name (at least 1 non-whitespace character, maximum 64 characters, trimmed of leading and trailing whitespace) and a selected Item_Category, THE Editor SHALL add a new Item to the Item_Registry with an empty description, an empty collection of Stat_Modifiers, and default values appropriate to the selected category
3. IF the user confirms item creation with an empty or whitespace-only display name, THEN THE Editor SHALL display a validation error indicating the name is required and prevent creation
4. IF the user confirms item creation without selecting an Item_Category, THEN THE Editor SHALL display a validation error indicating the category is required and prevent creation
5. WHEN a Weapon item is created, THE Editor SHALL initialize attack_power to 0, equipment_slot to Main_Hand, Rarity to Common, stackable to false, and value to 0
6. WHEN an Armor item is created, THE Editor SHALL initialize defense_power to 0, equipment_slot to Body, Rarity to Common, stackable to false, and value to 0
7. WHEN an Accessory item is created, THE Editor SHALL initialize equipment_slot to Accessory_1, Rarity to Common, stackable to false, and value to 0
8. WHEN a Consumable item is created, THE Editor SHALL initialize stackable to true, stack_limit to 99, Rarity to Common, value to 0, and one default Consumable_Effect of type Restore_HP with potency 10
9. WHEN a Key_Item is created, THE Editor SHALL initialize stackable to false, Rarity to Common, and value to 0
10. WHEN an item is successfully created, THE Item_Panel SHALL select the newly created item for editing
11. WHEN the user dismisses the item creation form without confirming, THE Editor SHALL discard the form input and leave the Item_Registry unchanged

### Requirement 5: Item Editing

**User Story:** As a game designer, I want to edit existing items, so that I can refine their properties and balance the game.

#### Acceptance Criteria

1. WHEN an item is selected in the Item_Panel, THE Editor SHALL display all item fields in an editable form appropriate to the item's Item_Category
2. WHEN the user modifies an item's display name to a valid non-empty string (at least 1 non-whitespace character, maximum 64 characters), THE Editor SHALL update the Item in the Item_Registry without requiring a separate save action
3. IF the user modifies an item's display name to an empty or whitespace-only string, THEN THE Editor SHALL display a validation error and retain the previous valid display name
4. WHEN the user modifies an item's description, THE Editor SHALL update the description field, truncating input at 256 characters
5. WHEN the user modifies an item's value, Rarity, attack_power, defense_power, equipment_slot, or stack_limit fields, THE Editor SHALL validate that numeric values are within their defined bounds (value: 0 to 4,294,967,295; attack_power: 0 to 4,294,967,295; defense_power: 0 to 4,294,967,295; stack_limit: 2 to 999) and update the corresponding field in the Item without requiring a separate save action
6. WHEN the user changes an item's Item_Category, THE Editor SHALL replace the category-specific properties with defaults appropriate to the new category (as defined in Requirement 4 criteria 5-9), enforce category constraints (Consumable: stackable set to true with one default Consumable_Effect of type Restore_HP with potency 10; Key_Item: stackable set to false and value set to 0), and retain common properties (name, description, value, Rarity) except where overridden by category constraints
7. IF the user enters a non-numeric value into a numeric field, THEN THE Editor SHALL reject the input and retain the previous valid numeric value
8. WHEN the user modifies a Consumable item's effects, THE Editor SHALL allow adding, removing, and editing Consumable_Effect entries, provided that at least one Consumable_Effect remains after any removal
9. IF the user attempts to remove the last Consumable_Effect from a Consumable item, THEN THE Editor SHALL reject the removal and retain the existing Consumable_Effect entry
10. WHEN the user changes an item's stackable field to true, THE Editor SHALL set the stack_limit to 99 if the current stack_limit is 1
11. WHEN the user changes an item's stackable field to false, THE Editor SHALL set the stack_limit to 1

### Requirement 6: Stat Modifier Management

**User Story:** As a game designer, I want to add and remove stat modifiers on items, so that I can define how equipment affects character stats.

#### Acceptance Criteria

1. WHEN the user activates the add stat modifier action on an Item, THE Item_Panel SHALL display a form for entering a stat name and modifier value
2. WHEN the user confirms adding a stat modifier with a valid stat name (1 to 32 characters, containing at least 1 non-whitespace character) and a signed 32-bit integer value (range -2,147,483,648 to 2,147,483,647), THE Editor SHALL add the Stat_Modifier to the Item
3. IF the user confirms adding a stat modifier with an empty, whitespace-only, or over-32-character stat name, THEN THE Editor SHALL display a validation error indicating an invalid stat name and prevent the addition
4. IF the user adds a stat modifier with a stat name that already exists on the Item, THEN THE Editor SHALL display a validation error indicating a duplicate stat modifier and prevent the addition
5. IF the user enters a non-numeric or out-of-range value into the modifier value field, THEN THE Editor SHALL reject the input and retain the previous valid numeric value
6. WHEN the user removes a Stat_Modifier from an Item, THE Editor SHALL immediately remove the modifier entry without a confirmation prompt
7. WHEN the user modifies an existing Stat_Modifier's value, THE Editor SHALL update the modifier value in the Item without requiring a separate save action
8. THE Item_Panel SHALL display Stat_Modifiers with positive values prefixed by "+", negative values prefixed by "-", and a value of zero displayed as "+0"

### Requirement 7: Item Deletion

**User Story:** As a game designer, I want to delete items I no longer need, so that I can keep my item database organized.

#### Acceptance Criteria

1. WHEN the user activates the delete action on an Item, THE Editor SHALL display a confirmation prompt that includes the item's display name and requires explicit confirm or cancel action
2. WHEN the user confirms item deletion, THE Editor SHALL remove the Item from the Item_Registry
3. WHEN the user cancels item deletion, THE Editor SHALL retain the Item in the Item_Registry without modification
4. WHEN an item is deleted and the Item_Registry becomes empty, THE Item_Panel SHALL display an empty state with a prompt to create a new item
5. WHEN an item is deleted and other items remain in the Item_Registry, THE Item_Panel SHALL select the first item in the case-insensitive alphabetically sorted list

### Requirement 8: Item List Navigation and Filtering

**User Story:** As a game designer, I want to browse and filter items in my database, so that I can quickly find and navigate to specific items.

#### Acceptance Criteria

1. THE Item_Panel SHALL display a scrollable list of all items in the Item_Registry, showing each item's display name and Item_Category, ordered alphabetically by display name using case-insensitive comparison
2. WHEN the user selects an item from the list, THE Item_Panel SHALL visually highlight the selected entry and load that item's details into the editing form
3. WHILE the Item_Registry contains no items, THE Item_Panel SHALL display an empty state message indicating no items are defined
4. THE Item_Panel SHALL provide a category filter that defaults to showing all items and allows the user to select a single Item_Category or all items
5. WHEN a category filter is active, THE Item_Panel SHALL display only items matching the selected Item_Category in the list, maintaining alphabetical order by display name
6. IF the currently selected item does not match the active category filter, THEN THE Item_Panel SHALL clear the selection and select the first visible item in the filtered list; IF no items match the filter, THEN THE Item_Panel SHALL display an empty state message indicating no items match the selected category
7. THE Item_Panel SHALL display the Rarity of each item in the list using a visually distinct color-coded indicator, with each of the five Rarity tiers (Common, Uncommon, Rare, Epic, Legendary) rendered in a unique color

### Requirement 9: Item Preview

**User Story:** As a game designer, I want to preview an item's full stat impact and properties at a glance, so that I can verify the item is configured correctly.

#### Acceptance Criteria

1. WHEN an item is selected, THE Item_Panel SHALL display a preview section showing all Stat_Modifiers as a list with each entry displaying the stat name and modifier value prefixed by "+" for positive values and "-" for negative values
2. WHEN an item with an equipment_slot is selected, THE Item_Panel SHALL display which Equipment_Slot the item occupies in the preview
3. WHEN a Consumable item is selected, THE Item_Panel SHALL display all Consumable_Effect entries with their effect_type and potency values in the preview
4. WHEN an item is selected, THE Item_Panel SHALL display the item's Rarity in the preview section using a distinct color per tier: Common displayed in white, Uncommon in green, Rare in blue, Epic in purple, and Legendary in gold
5. WHEN an item has stackable set to true, THE Item_Panel SHALL display the stack_limit in the preview section
6. WHEN an item with zero Stat_Modifiers is selected, THE Item_Panel SHALL display an indication of "No stat modifiers" in the stat modifier area of the preview section
7. WHEN the user modifies any item property while the preview section is displayed, THE Item_Panel SHALL immediately update the preview section to reflect the current item values without requiring manual refresh

### Requirement 10: Editor Mode Integration

**User Story:** As a game designer, I want to access the item editor from the mode menu, so that I can switch between editing maps, characters, and items seamlessly.

#### Acceptance Criteria

1. THE Editor SHALL add an ItemEditor variant to the AppEditorMode enum
2. WHEN the user selects the ItemEditor mode from the mode menu, THE Editor SHALL display the Item_Panel and hide all map editor and character editor panels
3. WHEN the AppEditorMode is not set to ItemEditor, THE Item_Panel SHALL not render
4. THE mode menu SHALL display the ItemEditor option with a distinct icon or label (e.g., "⚔ Item Editor")
