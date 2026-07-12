# Implementation Plan: Abilities Editor

## Overview

This plan implements the Abilities Editor feature across two crates: `rpg-toolkit-common` (data model, validation, serialization) and `rpg-toolkit-editor` (UI panel, editor integration). Tasks are ordered so that foundational data types come first, followed by registry logic, project integration, and finally the editor UI.

## Tasks

- [x] 1. Define ability data model and registry in the common crate
  - [x] 1.1 Create `crates/rpg-toolkit-common/src/ability.rs` with data types
    - Define `AbilityId` type alias (`String`)
    - Define `AbilityCategory` enum (`Skill`, `Spell`, `SpecialAction`) with `Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize`
    - Define `TargetType` enum (`SingleAlly`, `AllAllies`, `SingleEnemy`, `AllEnemies`, `SelfTarget`) with same derives
    - Define `CostType` enum (`MP`, `HP`) with same derives
    - Define `AbilitySource` enum with `#[serde(tag = "source_type")]`: `LevelUp { required_level: u32 }`, `LearnedFromItem { item_id: ItemId }`, `EquipmentGrant { item_id: ItemId }`, `AccessoryGrant { item_id: ItemId }` with `Clone, Debug, PartialEq, Eq, Serialize, Deserialize`
    - Define `Ability` struct with all fields per Requirement 1.2, deriving `Clone, Debug, PartialEq, Eq, Serialize, Deserialize`
    - Define `AbilityRegistry` struct wrapping `HashMap<AbilityId, Ability>`, deriving `Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize`
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 1.7, 1.8_

  - [x] 1.2 Add `AbilityValidationError` variant to `CommonError` in `error.rs`
    - Add `#[error("Ability validation error: {0}")] AbilityValidationError(String)` variant
    - _Requirements: 2.3, 2.4, 3.2, 4.2, 4.5, 5.2, 5.4, 5.6, 5.8_

  - [x] 1.3 Update `lib.rs` to export ability module and types
    - Add `pub mod ability;`
    - Add `pub use ability::{Ability, AbilityCategory, AbilityId, AbilityRegistry, AbilitySource, CostType, TargetType};`
    - _Requirements: 1.1_

- [x] 2. Implement AbilityRegistry CRUD methods
  - [x] 2.1 Implement `create_ability` method
    - Validate display name (trim, 1–64 chars, at least one non-whitespace)
    - Generate UUID v4 for the ID
    - Initialize defaults: `cost_value = 0`, `power = 0`, `target_type = SingleEnemy`, `cost_type = MP`, `sources = vec![]`, `description = ""`
    - Insert into the registry HashMap and return the AbilityId
    - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5_

  - [x] 2.2 Implement `delete_ability` method
    - Remove ability by ID from the HashMap
    - Return `AbilityValidationError` if ID not found
    - _Requirements: 3.1, 3.2_

  - [x] 2.3 Implement field update methods
    - `update_display_name`: validate same rules as creation, return error if ability not found
    - `update_description`: truncate to first 256 Unicode codepoints, store on ability
    - `update_category`, `update_cost_type`, `update_target_type`, `update_power`, `update_cost_value`: store new value, return error if ability not found
    - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5_

  - [x] 2.4 Implement source management methods
    - `add_source`: validate source (LevelUp `required_level >= 1`, item sources non-empty `item_id`), enforce max 10 sources, append to list
    - `remove_source`: validate index bounds, remove element at index
    - Return `AbilityValidationError` for all failure cases (ability not found, capacity exceeded, invalid index, invalid source data)
    - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5, 5.6, 5.7, 5.8, 5.9_

  - [x] 2.5 Implement `filtered_abilities` method
    - Accept `Option<AbilityCategory>` parameter
    - Return `Vec<&Ability>` filtered by category (or all if None)
    - Sort results case-insensitively by `display_name`
    - _Requirements: 6.1, 6.2, 6.3, 6.4_

  - [ ]* 2.6 Write property tests for creation and name validation
    - **Property 1: Creation produces a valid ability with correct defaults**
    - **Property 2: Name validation rejects all invalid names**
    - **Validates: Requirements 2.1, 2.2, 2.3, 2.4, 2.5, 4.1, 4.2**

  - [ ]* 2.7 Write property tests for deletion and field updates
    - **Property 3: Deletion removes and only removes the target ability**
    - **Property 4: Description truncation preserves the first 256 codepoints**
    - **Property 5: Field updates are stored correctly**
    - **Validates: Requirements 3.1, 4.3, 4.4**

  - [ ]* 2.8 Write property tests for source management
    - **Property 6: Source addition appends to the sources list**
    - **Property 7: Source removal removes exactly the element at the given index**
    - **Validates: Requirements 5.1, 5.3, 5.5**

  - [ ]* 2.9 Write property test for filtered listing
    - **Property 8: Filtered listing returns correctly filtered and sorted results**
    - **Validates: Requirements 6.1, 6.2, 6.3, 6.4**

- [x] 3. Checkpoint - Ensure common crate compiles and tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [x] 4. Integrate abilities into project serialization
  - [x] 4.1 Update `ProjectFile` in `project.rs` to include abilities field
    - Add `#[serde(default)] pub abilities: AbilityRegistry` field
    - Update `ProjectFile::new()` to accept and store `AbilityRegistry`
    - Add ability registry key/id validation in `ProjectFile::deserialize()` matching the character/item pattern
    - _Requirements: 7.1, 7.3, 7.4, 7.5_

  - [x] 4.2 Update `ProjectManifest` in `manifest.rs` to include abilities field
    - Add `#[serde(default)] pub abilities: AbilityRegistry` field
    - Update `into_project_file` to pass abilities through
    - Update `ProjectFile::to_manifest()` to include abilities
    - _Requirements: 7.1, 7.3_

  - [x] 4.3 Write property test for serialization round-trip
    - **Property 9: Serialization round-trip preserves registry equality**
    - Generate arbitrary valid registries with 0–50 abilities, each with 0–10 sources
    - Use `proptest` with custom strategies for all ability types
    - **Validates: Requirements 12.1, 12.4**

- [x] 5. Checkpoint - Ensure project serialization works end-to-end
  - Ensure all tests pass, ask the user if questions arise.

- [x] 6. Integrate ability editor mode into the editor crate
  - [x] 6.1 Add `Ability` variant to `AppEditorMode` in `data/state.rs`
    - Add `Ability` variant to the `AppEditorMode` enum
    - _Requirements: 8.1_

  - [x] 6.2 Add abilities fields to `Project` resource in `data/project.rs`
    - Add `pub abilities: AbilityRegistry` field
    - Add `pub has_unsaved_ability_changes: bool` field (initialized to `false`)
    - _Requirements: 7.1, 7.2_

  - [x] 6.3 Update `plugins/app_shell.rs` to add Ability Editor menu entry
    - Add selectable label "✨ Ability Editor" to the Mode menu
    - Set `AppEditorMode::Ability` when clicked
    - _Requirements: 8.2_

- [x] 7. Implement the ability editor panel plugin
  - [x] 7.1 Create `plugins/ability_panel.rs` with plugin structure and state
    - Create `AbilityPanelPlugin` struct implementing `Plugin`
    - Register `AbilityPanelState` resource and `ability_panel_ui` system with `run_if(resource_equals(AppEditorMode::Ability))`
    - Define `AbilityPanelState` with all fields: `selected_ability`, `category_filter`, `create_dialog_open`, buffers, error states, source-add dialog state
    - _Requirements: 8.3, 8.4, 8.5_

  - [x] 7.2 Implement the left list panel (220px)
    - Render category filter ComboBox with "All" default and per-category options
    - Render "Create" button that opens the creation dialog
    - List abilities sorted case-insensitively by display_name, filtered by selected category
    - Each entry shows display_name, category, and a delete button (🗑)
    - Auto-select first visible ability when filter changes and current selection is hidden
    - Show "No abilities yet. Create one to get started." when list is empty
    - _Requirements: 9.1, 9.2, 9.3, 9.4, 9.5, 9.6, 9.7, 9.8, 9.9_

  - [x] 7.3 Implement the central detail panel
    - Render `text_edit_singleline` for display_name (truncate to 64 chars as user types)
    - Render multiline `TextEdit` for description (max 256 chars)
    - Render ComboBox widgets for category, cost_type, and target_type
    - Render `DragValue` widgets for cost_value and power
    - Validate display_name on lost focus, show red error text below field
    - Render sources section with add/remove controls
    - Set `has_unsaved_ability_changes = true` on any field modification
    - _Requirements: 10.1, 10.2, 10.3, 10.4, 10.5, 10.6_

  - [x] 7.4 Implement the right preview panel (250px)
    - Display read-only labels for display_name, category, cost (formatted as "N TYPE"), power, and target_type
    - Display sources list with variant name and detail (required_level or item_id)
    - Show "Select an ability to preview." when no ability is selected
    - Immediately reflect changes from the detail panel
    - _Requirements: 11.1, 11.2, 11.3, 11.4_

  - [x] 7.5 Implement create and delete dialogs
    - Create dialog: name input + category selection + Create/Cancel buttons
    - Delete confirmation dialog before removing an ability
    - Wire both to registry methods, handle errors inline
    - _Requirements: 9.6, 9.7, 9.8_

  - [x] 7.6 Update `plugins/mod.rs` to export `AbilityPanelPlugin`
    - Add `pub mod ability_panel;` and `pub use ability_panel::AbilityPanelPlugin;`
    - Register `AbilityPanelPlugin` in the editor's plugin group
    - _Requirements: 8.3_

- [ ] 8. Final checkpoint - Ensure all tests pass and project compiles
  - Ensure all tests pass, ask the user if questions arise.

## Notes

- Tasks marked with `*` are optional and can be skipped for faster MVP
- Each task references specific requirements for traceability
- Checkpoints ensure incremental validation
- Property tests validate universal correctness properties from the design document
- The `proptest` dev-dependency already exists in `rpg-toolkit-common/Cargo.toml`
- The implementation follows the same patterns as the existing `ItemRegistry`/`ItemPanelPlugin`

## Task Dependency Graph

```json
{
  "waves": [
    { "id": 0, "tasks": ["1.1", "1.2"] },
    { "id": 1, "tasks": ["1.3"] },
    { "id": 2, "tasks": ["2.1", "2.2", "2.3", "2.4", "2.5"] },
    { "id": 3, "tasks": ["2.6", "2.7", "2.8", "2.9"] },
    { "id": 4, "tasks": ["4.1", "4.2"] },
    { "id": 5, "tasks": ["4.3"] },
    { "id": 6, "tasks": ["6.1", "6.2", "6.3"] },
    { "id": 7, "tasks": ["7.1"] },
    { "id": 8, "tasks": ["7.2", "7.3", "7.4", "7.5"] },
    { "id": 9, "tasks": ["7.6"] }
  ]
}
```
