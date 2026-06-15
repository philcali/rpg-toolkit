# Implementation Plan: Enemies Editor

## Overview

This plan implements the Enemies Editor feature in three phases: (1) shared Element enum and Enemy data model in `rpg-toolkit-common`, (2) project integration and serialization, and (3) the editor UI plugin in `rpg-toolkit-editor`. Property-based tests validate correctness properties from the design using `proptest`.

## Tasks

- [x] 1. Create shared Element enum and Enemy data model
  - [x] 1.1 Create `crates/rpg-toolkit-common/src/element.rs` with the `Element` enum
    - Define `Element` enum with variants: Fire, Ice, Lightning, Wind, Earth, Light, Dark
    - Derive Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize
    - Implement `Element::all()` returning a static slice of all variants
    - Add `pub mod element;` and `pub use element::Element;` to `crates/rpg-toolkit-common/src/lib.rs`
    - _Requirements: 1.8_

  - [x] 1.2 Create `crates/rpg-toolkit-common/src/enemy.rs` with data structures
    - Define `EnemyId` type alias (`String`)
    - Define `EnemyStat` struct with `name: String` and `base_value: u32`
    - Define `ItemDrop` struct with `item_id: ItemId` and `drop_chance: f64`
    - Define `DefeatReward` struct with `exp: u32`, `gold: u32`, `item_drops: Vec<ItemDrop>`
    - Define `CarriedItem` struct with `item_id: ItemId` and `obtain_chance: f64`
    - Define `ElementalModifier` struct with `element: Element` and `multiplier: f64`
    - Define `Enemy` struct with all fields: id, display_name, description, stats, defeat_rewards, carried_items, elemental_modifiers, abilities
    - Derive Clone, Debug, PartialEq, Serialize, Deserialize on all structs
    - Define `EnemyRegistry` struct with `enemies: HashMap<EnemyId, Enemy>`
    - Derive Clone, Debug, Default, PartialEq, Serialize, Deserialize on `EnemyRegistry`
    - Add `pub mod enemy;` and re-exports to `crates/rpg-toolkit-common/src/lib.rs`
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 1.7, 1.9, 1.10_

  - [x] 1.3 Add `EnemyValidationError` variant to `CommonError` in `crates/rpg-toolkit-common/src/error.rs`
    - Add `#[error("Enemy validation error: {0}")] EnemyValidationError(String)` variant
    - _Requirements: 1.11_

  - [x] 1.4 Implement `EnemyRegistry` CRUD methods in `crates/rpg-toolkit-common/src/enemy.rs`
    - Implement `create_enemy(&mut self, name: &str) -> Result<EnemyId, CommonError>` — trim, validate name (1–64 chars, ≥1 non-whitespace), generate UUID v4, insert with default stats (HP=10, Attack=5, Defense=5, Speed=5), empty rewards/modifiers/carried_items/abilities
    - Implement `delete_enemy(&mut self, id: &EnemyId) -> Result<(), CommonError>`
    - Implement `rename_enemy(&mut self, id: &EnemyId, new_name: &str) -> Result<(), CommonError>`
    - Implement `update_description(&mut self, id: &EnemyId, desc: &str) -> Result<(), CommonError>` — truncate to 256 Unicode codepoints
    - _Requirements: 2.1, 2.2, 2.3, 2.4, 3.1, 3.2, 4.1, 4.2, 4.3, 4.4_

  - [x] 1.5 Implement stat management methods in `crates/rpg-toolkit-common/src/enemy.rs`
    - Implement `add_stat(&mut self, id: &EnemyId, stat_name: &str) -> Result<(), CommonError>` — trim name, validate 1–32 chars, unique, max 20 stats, append with base_value 0
    - Implement `remove_stat(&mut self, id: &EnemyId, stat_name: &str) -> Result<(), CommonError>` — reject removal of "HP"
    - Implement `update_stat(&mut self, id: &EnemyId, stat_name: &str, base_value: u32) -> Result<(), CommonError>`
    - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5, 5.6, 5.7, 5.8, 5.9_

  - [x] 1.6 Implement defeat rewards management methods in `crates/rpg-toolkit-common/src/enemy.rs`
    - Implement `update_exp(&mut self, id: &EnemyId, exp: u32) -> Result<(), CommonError>`
    - Implement `update_gold(&mut self, id: &EnemyId, gold: u32) -> Result<(), CommonError>`
    - Implement `add_item_drop(&mut self, id: &EnemyId, item_id: &str, drop_chance: f64) -> Result<(), CommonError>` — validate non-empty item_id, 0.0–1.0 range, max 10
    - Implement `remove_item_drop(&mut self, id: &EnemyId, index: usize) -> Result<(), CommonError>`
    - _Requirements: 6.1, 6.2, 6.3, 6.4, 6.5, 6.6, 6.7, 6.8, 6.9_

  - [x] 1.7 Implement carried items management methods in `crates/rpg-toolkit-common/src/enemy.rs`
    - Implement `add_carried_item(&mut self, id: &EnemyId, item_id: &str, obtain_chance: f64) -> Result<(), CommonError>` — validate non-empty item_id, 0.0–1.0 range, max 8
    - Implement `remove_carried_item(&mut self, id: &EnemyId, index: usize) -> Result<(), CommonError>`
    - _Requirements: 7.1, 7.2, 7.3, 7.4, 7.5, 7.6, 7.7_

  - [x] 1.8 Implement elemental modifier management methods in `crates/rpg-toolkit-common/src/enemy.rs`
    - Implement `add_elemental_modifier(&mut self, id: &EnemyId, element: Element, multiplier: f64) -> Result<(), CommonError>` — validate multiplier ≥ 0.0, no duplicate element
    - Implement `update_elemental_modifier(&mut self, id: &EnemyId, element: Element, multiplier: f64) -> Result<(), CommonError>`
    - Implement `remove_elemental_modifier(&mut self, id: &EnemyId, element: Element) -> Result<(), CommonError>`
    - _Requirements: 8.1, 8.2, 8.3, 8.4, 8.5, 8.6, 8.7, 8.8_

  - [x] 1.9 Implement ability management methods in `crates/rpg-toolkit-common/src/enemy.rs`
    - Implement `add_ability(&mut self, id: &EnemyId, ability_id: &str) -> Result<(), CommonError>` — validate non-empty, no duplicates, max 10
    - Implement `remove_ability(&mut self, id: &EnemyId, ability_id: &str) -> Result<(), CommonError>`
    - _Requirements: 1.2 (abilities field)_

  - [x] 1.10 Implement listing and search methods in `crates/rpg-toolkit-common/src/enemy.rs`
    - Implement `sorted_enemies(&self) -> Vec<&Enemy>` — case-insensitive sort by display_name, ties broken by byte-order
    - Implement `search_enemies(&self, query: &str) -> Vec<&Enemy>` — case-insensitive substring match, sorted
    - _Requirements: 9.1, 9.2, 9.3, 9.4_

- [ ] 2. Checkpoint - Verify data model compiles
  - Ensure all tests pass, ask the user if questions arise.

- [x] 3. Project integration and serialization
  - [x] 3.1 Add `enemies: EnemyRegistry` and `has_unsaved_enemy_changes: bool` to the editor `Project` resource in `crates/rpg-toolkit-editor/src/data/project.rs`
    - Add `pub enemies: EnemyRegistry` field (import from common)
    - Add `pub has_unsaved_enemy_changes: bool` field initialized to false in Default impl
    - _Requirements: 10.1, 10.2_

  - [x] 3.2 Add `enemies: EnemyRegistry` field to `ProjectFile` in `crates/rpg-toolkit-common/src/project.rs`
    - Add `#[serde(default)] pub enemies: EnemyRegistry` field
    - Update `ProjectFile::new()` to accept and store the enemies parameter
    - Add validation in `ProjectFile::deserialize()` to check enemy registry keys match IDs
    - _Requirements: 10.3, 10.5, 10.6_

  - [x] 3.3 Add `enemies: EnemyRegistry` field to `ProjectManifest` in `crates/rpg-toolkit-common/src/manifest.rs`
    - Add `#[serde(default)] pub enemies: EnemyRegistry` field
    - Update `into_project_file()` to pass enemies through
    - Update `to_manifest()` in `ProjectFile` to include enemies
    - _Requirements: 10.4, 10.5_

  - [x] 3.4 Update serialization plugin in `crates/rpg-toolkit-editor/src/plugins/serialization.rs`
    - Include `project.enemies` when building `ProjectFile` for save
    - Populate `project.enemies` when loading from `ProjectFile`
    - Reset `project.has_unsaved_enemy_changes` to false after save and on load
    - _Requirements: 10.5_

- [ ] 4. Checkpoint - Verify serialization integration compiles
  - Ensure all tests pass, ask the user if questions arise.

- [x] 5. Editor mode and app shell integration
  - [x] 5.1 Add `Enemy` variant to `AppEditorMode` in `crates/rpg-toolkit-editor/src/data/state.rs`
    - Add `Enemy` variant to the `AppEditorMode` enum
    - _Requirements: 11.1_

  - [x] 5.2 Add "👹 Enemy Editor" entry to the Mode menu in `crates/rpg-toolkit-editor/src/plugins/app_shell.rs`
    - Add a selectable_label for `AppEditorMode::Enemy` with label "👹 Enemy Editor"
    - Follow the same pattern as the existing Character, Item, and Ability entries
    - _Requirements: 11.2_

- [x] 6. Implement Enemy Editor panel plugin
  - [x] 6.1 Create `crates/rpg-toolkit-editor/src/plugins/enemy_panel.rs` with plugin structure and state resource
    - Define `EnemyPanelPlugin` struct implementing `Plugin`
    - Register `EnemyPanelState` resource and `enemy_panel_ui` system in `EditorUiSet::Panels` with `run_if(resource_equals(AppEditorMode::Enemy))`
    - Define `EnemyPanelState` resource with fields: selected_enemy, create_dialog_open, create_name_buffer, create_error, delete_confirm_target, name_edit_buffer, name_edit_error, description_buffer, search_buffer, ability_search_buffer
    - _Requirements: 11.3, 11.4, 11.5_

  - [x] 6.2 Implement the left list panel in `enemy_panel_ui`
    - Add `SidePanel::left("enemy_list")` with default_width 220.0
    - Display "Create" button at top that opens create dialog
    - Show search field using `search_buffer`
    - Display sorted/filtered enemy list in a ScrollArea with selectable labels
    - Show delete button (🗑) per entry
    - Show "No enemies yet. Create one to get started." when list is empty
    - _Requirements: 12.1, 12.2, 12.3, 12.4, 12.10_

  - [x] 6.3 Implement the create enemy dialog
    - Show `egui::Window` when `create_dialog_open` is true
    - Include text_edit_singleline for name with 64-char truncation
    - Validate on "Create" button: trim, check 1–64 chars and ≥1 non-whitespace
    - On success: create enemy, close dialog, auto-select, set has_unsaved_enemy_changes
    - On failure: show error with colored_label(Color32::RED)
    - Include "Cancel" button to close without action
    - _Requirements: 12.4, 12.5, 12.6_

  - [x] 6.4 Implement the delete confirmation dialog
    - Show `egui::Window` when `delete_confirm_target` is Some
    - Display enemy name and "Confirm"/"Cancel" buttons
    - On confirm: delete enemy, clear selection if deleted was selected, set has_unsaved_enemy_changes
    - _Requirements: 12.7, 12.8, 12.9_

  - [x] 6.5 Implement the central detail panel — display_name and description editing
    - Show "Select an enemy to edit, or create a new one." when no enemy selected
    - Show text_edit_singleline for display_name with 64-char truncation
    - Validate on lost_focus: trim, 1–64 chars, ≥1 non-whitespace; show red error if invalid
    - Show multiline TextEdit for description with 256-char truncation
    - Set has_unsaved_enemy_changes on any field change
    - _Requirements: 13.1, 13.2, 13.3, 13.5, 13.6, 13.7_

  - [x] 6.6 Implement the central detail panel — stats section
    - Display stats in a grid with stat name label and DragValue for base_value (range 0..=u32::MAX)
    - Show delete button per stat (disabled/hidden for "HP")
    - Provide "Add Stat" button with text field for stat name
    - Validate stat name (1–32 chars, unique, max 20 stats)
    - Set has_unsaved_enemy_changes on change
    - _Requirements: 13.1, 13.4, 13.5_

  - [x] 6.7 Implement the central detail panel — defeat rewards section
    - Show DragValue for exp (range 0..=u32::MAX) and gold (range 0..=u32::MAX)
    - Display item_drops list with item_id label and DragValue for drop_chance (0.0..=1.0)
    - Provide "Add Item Drop" button and delete button per entry
    - Set has_unsaved_enemy_changes on change
    - _Requirements: 13.1, 13.4, 13.5_

  - [x] 6.8 Implement the central detail panel — carried items section
    - Display carried_items list with item_id label and DragValue for obtain_chance (0.0..=1.0)
    - Provide "Add Carried Item" button and delete button per entry
    - Set has_unsaved_enemy_changes on change
    - _Requirements: 13.1, 13.4, 13.5_

  - [x] 6.9 Implement the central detail panel — elemental modifiers section
    - Display elemental_modifiers list with element name and DragValue for multiplier (0.0..=f64::MAX)
    - Provide "Add Modifier" with Element combo box selection
    - Provide delete button per entry
    - Set has_unsaved_enemy_changes on change
    - _Requirements: 13.1, 13.4, 13.5_

  - [x] 6.10 Implement the right preview panel
    - Add `SidePanel::right("enemy_preview")` with default_width 250.0
    - Show heading "Enemy Preview"
    - Display read-only labels: display_name, stat summary, defeat rewards summary (exp, gold, item drop count), carried items count, elemental modifiers list
    - Show "No stats defined." / "No elemental modifiers." when lists are empty
    - Show "Select an enemy to preview." when no enemy selected
    - Reflect changes in same render frame
    - _Requirements: 14.1, 14.2, 14.3, 14.4_

- [x] 7. Register plugin and wire everything together
  - [x] 7.1 Add `pub mod enemy_panel;` and `pub use enemy_panel::EnemyPanelPlugin;` to `crates/rpg-toolkit-editor/src/plugins/mod.rs`
    - _Requirements: 11.3_

  - [x] 7.2 Register `EnemyPanelPlugin` in `crates/rpg-toolkit-editor/src/main.rs`
    - Add `.add_plugins(EnemyPanelPlugin)` after `AbilityPanelPlugin`
    - _Requirements: 11.3_

- [ ] 8. Checkpoint - Verify full build compiles
  - Ensure all tests pass, ask the user if questions arise.

- [x] 9. Property-based tests
  - [x] 9.1 Write property test for serialization round-trip in `tests/properties/enemy_round_trip.rs`
    - **Property 1: Serialization round-trip preserves registry equality**
    - Generate arbitrary valid EnemyRegistry (0–50 enemies, 1–20 stats, 0–10 item drops, 0–8 carried items, 0–7 elemental modifiers, 0–10 abilities, finite f64 values)
    - Serialize to JSON with serde_json, deserialize back, assert PartialEq
    - Minimum 100 iterations
    - **Validates: Requirements 15.1, 15.4, 1.2, 1.9, 1.10**

  - [ ]* 9.2 Write property test for creation initialization in `tests/properties/enemy_invariants.rs`
    - **Property 2: Creation produces correctly initialized enemy**
    - For any valid display name (1–64 chars after trim), verify create_enemy inserts enemy with trimmed name, UUID key matching id, default stats (HP=10, Attack=5, Defense=5, Speed=5), empty items/modifiers/abilities, exp=0, gold=0
    - **Validates: Requirements 2.1, 2.4**

  - [ ]* 9.3 Write property test for invalid name rejection in `tests/properties/enemy_invariants.rs`
    - **Property 3: Invalid display name is rejected without modification**
    - For any string that is empty/whitespace-only after trim or >64 chars after trim, verify create_enemy and rename_enemy return EnemyValidationError and registry is unchanged
    - **Validates: Requirements 2.2, 2.3, 4.1, 4.2**

  - [x] 9.4 Write property test for non-existent ID operations in `tests/properties/enemy_invariants.rs`
    - **Property 4: Operations on non-existent enemy ID return error without modification**
    - For any EnemyId not in the registry, verify all mutation methods return error containing the ID and registry remains unchanged
    - **Validates: Requirements 3.2, 4.4, 5.6, 6.9, 7.7, 8.8**

  - [ ]* 9.5 Write property test for sorted listing in `tests/properties/enemy_invariants.rs`
    - **Property 5: Sorted listing is correctly ordered**
    - For any registry with ≥1 enemy, verify sorted_enemies returns all enemies in case-insensitive order with byte-order tiebreak
    - **Validates: Requirements 9.1, 9.2**

  - [ ]* 9.6 Write property test for search filtering in `tests/properties/enemy_invariants.rs`
    - **Property 6: Search filter returns only matching entries**
    - For any registry and non-empty/non-whitespace query, verify search_enemies returns only matching entries in sorted order
    - **Validates: Requirements 9.3, 9.4**

  - [ ]* 9.7 Write property test for description truncation in `tests/properties/enemy_invariants.rs`
    - **Property 7: Description truncation**
    - For any string, verify update_description stores at most first 256 Unicode codepoints
    - **Validates: Requirements 4.3**

  - [x] 9.8 Write property test for validation failure preservation in `tests/properties/enemy_invariants.rs`
    - **Property 8: Validation failure preserves registry state**
    - For any operation violating validation rules (duplicate stat, out-of-range probability, capacity overflow, removing HP), verify error returned and registry unchanged
    - **Validates: Requirements 1.11, 5.3, 5.4, 5.8, 5.9, 6.4, 6.5, 6.6, 7.2, 7.3, 7.4, 8.2, 8.3**

- [ ] 10. Final checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

## Notes

- Tasks marked with `*` are optional and can be skipped for faster MVP
- Each task references specific requirements for traceability
- Checkpoints ensure incremental validation
- Property tests validate universal correctness properties from the design document
- The `Element` enum is in its own module (`element.rs`) for cross-cutting reuse by items and abilities in future specs
- The editor plugin follows the exact same pattern as `CharacterPanelPlugin` and `AbilityPanelPlugin`
- All f64 fields in property tests must use finite values only to ensure round-trip correctness

## Task Dependency Graph

```json
{
  "waves": [
    { "id": 0, "tasks": ["1.1", "1.3"] },
    { "id": 1, "tasks": ["1.2"] },
    { "id": 2, "tasks": ["1.4", "1.5", "1.6", "1.7", "1.8", "1.9", "1.10"] },
    { "id": 3, "tasks": ["3.1", "3.2", "3.3", "5.1"] },
    { "id": 4, "tasks": ["3.4", "5.2", "6.1"] },
    { "id": 5, "tasks": ["6.2", "6.3", "6.4", "6.5", "6.6", "6.7", "6.8", "6.9", "6.10"] },
    { "id": 6, "tasks": ["7.1", "7.2"] },
    { "id": 7, "tasks": ["9.1", "9.2", "9.3", "9.4", "9.5", "9.6", "9.7", "9.8"] }
  ]
}
```
