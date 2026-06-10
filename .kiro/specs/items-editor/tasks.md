# Implementation Plan: Items Editor

## Overview

This plan implements the Items Editor feature across `rpg-toolkit-common` (item data model, validation, serialization) and `rpg-toolkit-editor` (UI mode integration and item panel). The approach follows the same bottom-up pattern as the Character Editor: data model first, then error variant, then serialization integration, then editor mode extension, and finally the item panel UI. Property-based tests validate core invariants throughout.

## Tasks

- [x] 1. Define item data model in rpg-toolkit-common
  - [x] 1.1 Create `item.rs` module with all item types, enums, and `ItemRegistry` struct
    - Create `crates/rpg-toolkit-common/src/item.rs`
    - Define `ItemId` type alias (`String`)
    - Define `Rarity` enum (Common, Uncommon, Rare, Epic, Legendary) with Serialize/Deserialize
    - Define `EquipmentSlot` enum (MainHand, OffHand, Head, Body, Legs, Feet, Accessory1, Accessory2)
    - Define `StatModifier` struct with `stat_name: String`, `value: i32`
    - Define `BuffTargetStat` enum (Strength, Stamina, Speed, Luck, Wisdom, Intelligence)
    - Define `CureTargetStatus` enum (Poison, Paralysis, Sleep, Confusion, Silence, All)
    - Define `ConsumableEffectType` enum with `#[serde(tag = "effect_type")]` (RestoreHP, RestoreMP, CureStatus, BuffStat)
    - Define `ConsumableEffect` struct with `effect: ConsumableEffectType`, `potency: u32`
    - Define `ItemCategoryData` enum with `#[serde(tag = "category")]` (Weapon, Armor, Accessory, Consumable, KeyItem)
    - Define `ItemCategory` enum for API/UI use (Weapon, Armor, Accessory, Consumable, KeyItem)
    - Define `Item` struct with all fields (id, display_name, description, category_data, value, rarity, stackable, stack_limit, stat_modifiers)
    - Define `ItemRegistry` struct with `items: HashMap<ItemId, Item>` (derive Default)
    - Implement `Item::category()` method returning the `ItemCategory` for the item
    - Add `ItemValidationError(String)` variant to `CommonError` in `error.rs`
    - Register module in `lib.rs` and add public exports
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 1.7, 1.8, 1.9, 1.10, 2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 2.10, 2.11, 2.12, 2.13, 2.14_

  - [x] 1.2 Implement `ItemRegistry` CRUD and validation methods
    - Implement `create_item(name: &str, category: ItemCategory) -> Result<ItemId, CommonError>` — validates name (trimmed, 1–64 chars, non-whitespace), generates UUID v4, initializes with category defaults per design
    - Implement `delete_item(id: &ItemId) -> Result<(), CommonError>` — removes item or returns error if not found
    - Implement `update_display_name(id: &ItemId, name: &str) -> Result<(), CommonError>` — validates and updates name
    - Implement `update_description(id: &ItemId, desc: &str) -> Result<(), CommonError>` — validates max 256 chars, truncates if needed
    - Implement `change_category(id: &ItemId, new_category: ItemCategory) -> Result<(), CommonError>` — replaces category_data with defaults, enforces category constraints (Consumable→stackable=true, KeyItem→stackable=false/value=0), preserves common properties
    - Implement `set_stackable(id: &ItemId, stackable: bool) -> Result<(), CommonError>` — toggles stackable, adjusts stack_limit (true→99 if was 1, false→1)
    - Implement `set_stack_limit(id: &ItemId, limit: u32) -> Result<(), CommonError>` — validates [2, 999] for stackable items
    - Implement `sorted_items(&self) -> Vec<&Item>` — case-insensitive alphabetical sort
    - Implement `filtered_items(&self, category: Option<ItemCategory>) -> Vec<&Item>` — sorted + filtered
    - Implement `format_modifier_value(value: i32) -> String` — returns "+N", "-N", or "+0"
    - _Requirements: 1.2, 1.8, 1.9, 1.12, 1.13, 1.14, 2.7, 2.8, 2.9, 4.2, 4.5, 4.6, 4.7, 4.8, 4.9, 5.2, 5.3, 5.4, 5.6, 5.10, 5.11, 6.8, 7.2, 8.1, 8.5_

  - [x] 1.3 Implement stat modifier management methods on `ItemRegistry`
    - Implement `add_stat_modifier(id: &ItemId, stat_name: &str, value: i32) -> Result<(), CommonError>` — validates stat_name (1–32 chars, non-whitespace), rejects duplicates, enforces max 20 modifiers
    - Implement `remove_stat_modifier(id: &ItemId, stat_name: &str) -> Result<(), CommonError>` — removes modifier by name
    - Implement `update_stat_modifier(id: &ItemId, stat_name: &str, value: i32) -> Result<(), CommonError>` — updates existing modifier value
    - _Requirements: 1.10, 1.11, 6.2, 6.3, 6.4, 6.6, 6.7_

  - [x] 1.4 Implement consumable effect management methods on `ItemRegistry`
    - Implement `add_consumable_effect(id: &ItemId, effect: ConsumableEffect) -> Result<(), CommonError>` — validates potency ≥ 1, enforces max 4 effects, rejects if not Consumable
    - Implement `remove_consumable_effect(id: &ItemId, index: usize) -> Result<(), CommonError>` — rejects removal of last effect
    - Implement `update_consumable_effect(id: &ItemId, index: usize, effect: ConsumableEffect) -> Result<(), CommonError>` — updates effect at index
    - _Requirements: 2.6, 2.11, 5.8, 5.9_

- [x] 2. Integrate items into project serialization
  - [x] 2.1 Add `items` field to `ProjectFile` and `ProjectManifest`
    - Add `#[serde(default)] pub items: ItemRegistry` to `ProjectFile` in `crates/rpg-toolkit-common/src/project.rs`
    - Add `#[serde(default)] pub items: ItemRegistry` to `ProjectManifest` in `crates/rpg-toolkit-common/src/manifest.rs`
    - Update `ProjectFile::new()` constructor to accept `items: ItemRegistry` parameter
    - Update `to_manifest()` to include items
    - Update `ProjectFile::deserialize()` to validate Item_Id consistency (key matches value.id)
    - Update `into_project_file` in manifest to pass items through
    - _Requirements: 3.1, 3.3, 3.4, 3.5, 3.6, 3.7_

  - [x] 2.2 Add `items` field to editor `Project` resource and update serialization plugin
    - Add `pub items: ItemRegistry` and `pub has_unsaved_item_changes: bool` to `Project` struct in `crates/rpg-toolkit-editor/src/data/project.rs`
    - Update `to_project_file()` in `serialization.rs` to include `project.items`
    - Update `load_project_from_dir`, `load_project_from_zip`, `load_project_from_json` to populate `project.items` from deserialized `ProjectFile`
    - Update `NewProject` action to reset items to default
    - _Requirements: 3.1, 3.3, 3.5_

- [ ] 3. Checkpoint
  - Ensure all tests pass, ask the user if questions arise.

- [x] 4. Add ItemEditor mode and menu entry
  - [x] 4.1 Add `ItemEditor` variant to `AppEditorMode` enum
    - Add `ItemEditor` variant to `AppEditorMode` in `crates/rpg-toolkit-editor/src/data/state.rs`
    - No changes to `Default` derivation (remains `MapEditor`)
    - _Requirements: 10.1_

  - [x] 4.2 Add "⚔ Item Editor" entry to the Mode menu in `AppShellPlugin`
    - Add a third selectable label in the Mode menu in `app_shell.rs`
    - Label: "⚔ Item Editor", sets `AppEditorMode::ItemEditor`
    - _Requirements: 10.2, 10.4_

- [ ] 5. Implement ItemPanelPlugin UI
  - [x] 5.1 Create `item_panel.rs` plugin with panel state and registration
    - Create `crates/rpg-toolkit-editor/src/plugins/item_panel.rs`
    - Define `ItemPanelPlugin` struct implementing `Plugin`
    - Define `ItemPanelState` resource (selected_item, category_filter, create_dialog_open, create_name_buffer, create_category, create_error, delete_confirm_target, add_stat_dialog_open, add_stat_name_buffer, add_stat_value_buffer, add_stat_error, name_edit_buffer, name_edit_error)
    - Register plugin in `plugins/mod.rs` and `main.rs`
    - Add a main UI system gated on `AppEditorMode::ItemEditor` in `EditorUiSet::Panels`
    - _Requirements: 10.2, 10.3_

  - [x] 5.2 Implement item list panel (left side)
    - Render a left `SidePanel` with "New Item" button at top
    - Add category filter combo box (All / Weapon / Armor / Accessory / Consumable / Key_Item)
    - Render scrollable, alphabetically sorted list of items showing display name, category, and rarity color indicator
    - Highlight selected item, show delete button per item
    - Show empty state message when no items or no items match filter
    - Handle filter changes: clear selection if selected item doesn't match filter, select first visible
    - _Requirements: 8.1, 8.2, 8.3, 8.4, 8.5, 8.6, 8.7_

  - [x] 5.3 Implement item creation dialog
    - Render creation form (name input + category combo box with no pre-selection) when create_dialog_open is true
    - Validate name is non-empty/non-whitespace and category is selected on confirm
    - Call `ItemRegistry::create_item`, handle errors inline (red text)
    - Auto-select newly created item
    - Cancel discards input
    - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.10, 4.11_

  - [x] 5.4 Implement item detail editor (center panel)
    - Render editable display name field with inline validation
    - Render description multiline field (truncated at 256 chars)
    - Render category combo box that triggers `change_category` on change
    - Render rarity combo box, value drag input
    - Render stackable toggle + stack_limit input (visible when stackable)
    - Render category-specific fields: attack_power/equipment_slot for Weapon, defense_power/equipment_slot for Armor, equipment_slot for Accessory, effects list for Consumable
    - Render stat modifier section with add/remove/edit capability
    - Non-numeric input in numeric fields is rejected, retaining previous value
    - Mark project as having unsaved item changes on any edit
    - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5, 5.6, 5.7, 5.8, 5.9, 5.10, 5.11, 6.1, 6.2, 6.3, 6.4, 6.5, 6.6, 6.7_

  - [x] 5.5 Implement item preview panel (right side)
    - Render a right `SidePanel` with rarity badge (color-coded per design: Common=white, Uncommon=green, Rare=blue, Epic=purple, Legendary=gold)
    - Display equipment slot if applicable
    - Display stat modifiers list with +/- formatting (or "No stat modifiers" if empty)
    - Display consumable effects with type and potency for Consumable items
    - Display stack limit if stackable
    - Update preview immediately on any property change
    - _Requirements: 9.1, 9.2, 9.3, 9.4, 9.5, 9.6, 9.7_

  - [x] 5.6 Implement item deletion with confirmation
    - Render delete confirmation dialog with item display name
    - On confirm: remove item, select first remaining (alphabetical) or show empty state
    - On cancel: retain item unchanged
    - _Requirements: 7.1, 7.2, 7.3, 7.4, 7.5_

- [ ] 6. Checkpoint
  - Ensure all tests pass, ask the user if questions arise.

- [x] 7. Property-based tests
  - [x] 7.1 Write property test: Item serialization round-trip (Property 1)
    - **Property 1: Item serialization round-trip**
    - Create property test in `crates/rpg-toolkit-common/tests/properties/item_round_trip.rs`
    - Generate arbitrary `ItemRegistry` with random items across all categories, wrap in `ProjectFile`, serialize to JSON, deserialize, assert equality
    - Minimum 100 iterations
    - **Validates: Requirements 3.2, 3.6**

  - [ ]* 7.2 Write property test: Stack limit invariant (Property 2)
    - **Property 2: Stack limit invariant**
    - Create property test in `crates/rpg-toolkit-common/tests/properties/item_invariants.rs`
    - Generate random items, apply random stackable toggle operations via `set_stackable`, assert: stackable=true → stack_limit in [2, 999], stackable=false → stack_limit == 1
    - Minimum 100 iterations
    - **Validates: Requirements 1.8, 1.9**

  - [x] 7.3 Write property test: Category-specific invariants (Property 3)
    - **Property 3: Category-specific invariants**
    - In `item_invariants.rs`
    - Generate random items across all categories, apply random operations including `change_category`, assert: Consumable → stackable=true, KeyItem → stackable=false and value=0
    - Minimum 100 iterations
    - **Validates: Requirements 2.7, 2.8, 2.9**

  - [ ]* 7.4 Write property test: Equipment slot validity per category (Property 4)
    - **Property 4: Equipment slot validity per category**
    - In `item_invariants.rs`
    - Generate random equippable items (Weapon, Armor, Accessory), verify slot is in the valid set for the item's category
    - Minimum 100 iterations
    - **Validates: Requirements 2.2, 2.4, 2.5**

  - [ ]* 7.5 Write property test: Display name validation (Property 5)
    - **Property 5: Display name validation**
    - In `item_invariants.rs`
    - Generate random strings (valid and invalid: empty, whitespace-only, >64 chars), attempt `create_item` and `update_display_name`, verify accept/reject matches criteria and registry unchanged on reject
    - Minimum 100 iterations
    - **Validates: Requirements 1.2, 1.13, 4.3, 5.3**

  - [ ]* 7.6 Write property test: Duplicate stat modifier rejection (Property 6)
    - **Property 6: Duplicate stat modifier rejection**
    - In `item_invariants.rs`
    - Generate items with random stat modifiers, attempt adding a duplicate stat name, verify error returned and stat modifiers unchanged
    - Minimum 100 iterations
    - **Validates: Requirements 1.11, 6.4**

  - [ ]* 7.7 Write property test: Consumable effects bounded (Property 7)
    - **Property 7: Consumable effects bounded**
    - In `item_invariants.rs`
    - Generate consumable items, apply random add/remove sequences, verify effects count in [1, 4] and potency ≥ 1 for all effects
    - Minimum 100 iterations
    - **Validates: Requirements 2.6, 2.11, 5.8, 5.9**

  - [ ]* 7.8 Write property test: Stat modifier display formatting (Property 8)
    - **Property 8: Stat modifier display formatting**
    - In `item_invariants.rs`
    - Generate random i32 values, verify `format_modifier_value` output: positive → "+N", negative → "-N", zero → "+0"
    - Minimum 100 iterations
    - **Validates: Requirements 6.8**

  - [ ]* 7.9 Write property test: Item list ordering (Property 9)
    - **Property 9: Item list ordering**
    - In `item_invariants.rs`
    - Generate registry with random names, verify `sorted_items` produces case-insensitive alphabetical order (each consecutive pair satisfies a.to_lowercase() <= b.to_lowercase())
    - Minimum 100 iterations
    - **Validates: Requirements 8.1**

  - [ ]* 7.10 Write property test: Category filter correctness (Property 10)
    - **Property 10: Category filter correctness**
    - In `item_invariants.rs`
    - Generate registry with mixed categories, apply filter, verify only matching items returned in case-insensitive alphabetical order
    - Minimum 100 iterations
    - **Validates: Requirements 8.5**

  - [ ]* 7.11 Write property test: Category change preserves common properties (Property 11)
    - **Property 11: Category change preserves common properties**
    - In `item_invariants.rs`
    - Generate random items, change category, verify display_name, description, stat_modifiers, and rarity are preserved; value preserved except KeyItem (forced to 0); stackable/stack_limit set per category constraints
    - Minimum 100 iterations
    - **Validates: Requirements 5.6**

- [x] 8. Final integration and wiring
  - [x] 8.1 Wire item data into project save/load cycle end-to-end
    - Verify `prepare_assets_for_save` does not interfere with item data
    - Ensure the `to_manifest` path includes items for directory-based saves
    - Ensure ZIP save includes items in the manifest
    - Mark project as having unsaved changes when item mutations occur
    - _Requirements: 3.1, 3.5_

  - [ ]* 8.2 Write unit tests for UI state transitions and edge cases
    - Test: creating an item auto-selects it
    - Test: deleting selected item selects first remaining (alphabetical)
    - Test: deleting last item shows empty state
    - Test: category change from Consumable to KeyItem enforces constraints
    - Test: adding 20 stat modifiers then attempting 21st returns error
    - Test: stack_limit boundary values (2, 999)
    - Test: description truncation at exactly 256 characters
    - Test: backward-compatible deserialization with no "items" key
    - Test: deserialization with mismatched Item_Id key/value returns error
    - Test: deserialization with duplicate Item_Ids
    - Test: mode switching activates item panel
    - _Requirements: 1.8, 1.9, 1.10, 1.14, 2.7, 2.8, 2.9, 3.3, 3.4, 3.7, 4.10, 5.6, 7.4, 7.5, 10.1_

- [ ] 9. Final checkpoint
  - Ensure all tests pass, ask the user if questions arise.

## Notes

- Tasks marked with `*` are optional and can be skipped for faster MVP
- Each task references specific requirements for traceability
- Checkpoints ensure incremental validation
- Property tests validate universal correctness properties from the design document
- Unit tests validate specific examples and edge cases
- The `ItemEditor` mode variant is additive — existing map/character editor plugins already have mode-gating and require no changes
- The `uuid` crate is already available in the workspace for item ID generation
- The `proptest` crate is already in workspace dependencies for property-based tests
- The item panel follows the same three-panel layout (left list, center editor, right preview) as the character panel

## Task Dependency Graph

```json
{
  "waves": [
    { "id": 0, "tasks": ["1.1"] },
    { "id": 1, "tasks": ["1.2", "1.3", "1.4"] },
    { "id": 2, "tasks": ["2.1"] },
    { "id": 3, "tasks": ["2.2"] },
    { "id": 4, "tasks": ["4.1"] },
    { "id": 5, "tasks": ["4.2", "5.1"] },
    { "id": 6, "tasks": ["5.2", "5.3", "5.5", "5.6"] },
    { "id": 7, "tasks": ["5.4"] },
    { "id": 8, "tasks": ["7.1", "7.2", "7.3", "7.4", "7.5", "7.6", "7.7", "7.8", "7.9", "7.10", "7.11"] },
    { "id": 9, "tasks": ["8.1", "8.2"] }
  ]
}
```
