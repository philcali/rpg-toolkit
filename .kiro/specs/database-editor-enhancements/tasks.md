# Implementation Plan: Database Editor Enhancements

## Overview

This plan implements nine database editor enhancements for the RPG toolkit: Monster ability category, enemy ability dropdown selection, character ability learning system, enemy portrait, equipment-granted abilities, character visual assets, character visual asset file pickers, enemy portrait file picker, and character starting equipment. Implementation progresses from data model changes in `rpg-toolkit-common` to UI changes in `rpg-toolkit-editor`, wiring everything together at the end.

## Tasks

- [x] 1. Add Monster ability category and update ability panel
  - [x] 1.1 Add Monster variant to AbilityCategory enum
    - Add `Monster` variant to the `AbilityCategory` enum in `crates/rpg-toolkit-common/src/ability.rs`
    - Update `arb_category()` proptest strategy in the existing tests to include `Monster`
    - Verify existing serialization round-trip test passes with the new variant
    - _Requirements: 1.6_

  - [x] 1.2 Write property test for category filter with Monster variant
    - **Property 1: Category filter returns exactly matching abilities**
    - **Validates: Requirements 1.4, 1.5**
    - Create `crates/rpg-toolkit-common/tests/properties/ability_category_filter.rs`
    - Generate arbitrary registries containing Monster abilities
    - Assert `filtered_abilities(Some(Monster))` returns only Monster abilities
    - Assert `filtered_abilities(None)` returns all abilities including Monster

  - [ ]* 1.3 Write property test for ability registry serialization with Monster
    - **Property 2: Ability registry serialization round-trip**
    - **Validates: Requirements 1.6**
    - Add test in `crates/rpg-toolkit-common/tests/properties/ability_category_filter.rs`
    - Serialize registry with Monster abilities to JSON and deserialize back
    - Assert equality

  - [x] 1.4 Update ability panel UI to support Monster category
    - Add `"Monster"` to the category filter ComboBox in `crates/rpg-toolkit-editor/src/plugins/ability_panel.rs`
    - Add `"Monster"` to the create dialog category selector
    - Add `"Monster"` to the category edit ComboBox
    - Add match arm `AbilityCategory::Monster => "Monster"` in category display name logic
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5_

- [x] 2. Implement enemy ability dropdown selection
  - [x] 2.1 Replace free-text ability input with searchable dropdown in enemy panel
    - Modify `crates/rpg-toolkit-editor/src/plugins/enemy_panel.rs`
    - Replace the `add_ability_id_buffer` text input + "Add" button with the `searchable_combobox` widget
    - Build items list from `AbilityRegistry::filtered_abilities(None)` formatted as `"{display_name} [{category}]"`
    - On selection, call `EnemyRegistry::add_ability(enemy_id, selected_ability_id)`
    - Add `ability_search_buffer: String` to `EnemyPanelState`
    - Show "No abilities available" when registry is empty
    - Show error when max 10 abilities reached
    - Prevent duplicate assignment (handled by existing `add_ability` validation)
    - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 2.7_

  - [ ]* 2.2 Write property test for searchable filter correctness
    - **Property 3: Searchable filter correctness**
    - **Validates: Requirements 2.2, 2.4, 5.4**
    - Create `crates/rpg-toolkit-common/tests/properties/searchable_filter.rs`
    - Generate arbitrary lists of `(id, label)` pairs and query strings
    - Assert filter returns exactly matching items (case-insensitive substring) sorted by label

- [x] 3. Implement character ability learning system data model
  - [x] 3.1 Add LearnableAbility struct and learnable_abilities field to Character
    - Add `LearnableAbility { ability_id: AbilityId, required_level: u32 }` struct to `crates/rpg-toolkit-common/src/character.rs`
    - Add `learnable_abilities: Vec<LearnableAbility>` field to `Character` with `#[serde(default)]`
    - Initialize as empty `Vec` in `create_character`
    - Add `add_learnable_ability(id, ability_id, level)` method — validates level 1–99, rejects duplicates, max 20 entries
    - Add `remove_learnable_ability(id, ability_id)` method
    - Add `update_learnable_ability_level(id, ability_id, new_level)` method — clamps to 1–99
    - _Requirements: 3.1, 3.4, 3.5, 3.6, 3.7, 3.8_

  - [x] 3.2 Write property test for learnable ability level invariant
    - **Property 4: Learnable ability level invariant**
    - **Validates: Requirements 3.1, 3.4, 3.7**
    - Create `crates/rpg-toolkit-common/tests/properties/character_learnable.rs`
    - Generate arbitrary characters with learnable abilities
    - Assert all `required_level` values are in [1, 99]
    - Assert add/update with out-of-range levels results in clamped values

  - [ ]* 3.3 Write property test for character serialization round-trip
    - **Property 5: Character serialization round-trip with new fields**
    - **Validates: Requirements 3.8, 6.7**
    - Add test in `crates/rpg-toolkit-common/tests/properties/character_learnable.rs`
    - Serialize `CharacterRegistry` with learnable abilities and visual assets, deserialize back
    - Assert equality

- [x] 4. Implement character ability learning system UI
  - [x] 4.1 Add Learnable Abilities section to character panel
    - Modify `crates/rpg-toolkit-editor/src/plugins/character_panel.rs`
    - Add "Learnable Abilities" section after stats section
    - Display entries sorted by `required_level` ascending, showing ability display name and level
    - Add searchable dropdown (using `searchable_combobox`) to add new learnable abilities
    - Add `DragValue` for level input (range 1..=99)
    - Add remove button per entry
    - Show "No abilities available" when `AbilityRegistry` is empty
    - Add `add_learnable_search_buffer`, `add_learnable_level`, `add_learnable_error` to panel state
    - _Requirements: 3.2, 3.3, 3.4, 3.5, 3.6, 3.7, 3.9_

- [ ] 5. Checkpoint - Verify core data models and learning system
  - Ensure all tests pass, ask the user if questions arise.

- [x] 6. Implement enemy portrait
  - [x] 6.1 Add portrait field and methods to Enemy/EnemyRegistry
    - Add `portrait: Option<String>` field to `Enemy` struct in `crates/rpg-toolkit-common/src/enemy.rs` with `#[serde(default)]`
    - Initialize as `None` in `create_enemy`
    - Add `set_portrait(id, path)` method — trims, validates non-empty after trim, truncates to 260 chars
    - Add `clear_portrait(id)` method — sets to `None`
    - _Requirements: 4.1, 4.4, 4.5, 4.6, 4.7_

  - [ ]* 6.2 Write property test for enemy portrait invariant
    - **Property 6: Enemy portrait invariant**
    - **Validates: Requirements 4.1, 4.4, 4.7**
    - Create `crates/rpg-toolkit-common/tests/properties/enemy_portrait.rs`
    - Generate arbitrary enemies with portrait values
    - Assert portrait is either None or non-empty trimmed string ≤ 260 chars
    - Assert whitespace-only set attempts are rejected

  - [x] 6.3 Write property test for enemy serialization with portrait
    - **Property 7: Enemy serialization round-trip with portrait**
    - **Validates: Requirements 4.6**
    - Add test in `crates/rpg-toolkit-common/tests/properties/enemy_portrait.rs`
    - Serialize `EnemyRegistry` with portrait data, deserialize back, assert equality

  - [x] 6.4 Add Portrait section to enemy panel UI
    - Modify `crates/rpg-toolkit-editor/src/plugins/enemy_panel.rs`
    - Add "Portrait" section with single-line text input (truncate to 260 chars)
    - Show "No portrait assigned" label when `None`
    - Add "Clear" button to reset to `None`
    - Validate on lost focus: if trimmed empty → show error, don't store
    - Add `portrait_buffer`, `portrait_error` to `EnemyPanelState`
    - _Requirements: 4.2, 4.3, 4.4, 4.5, 4.7_

- [x] 7. Implement equipment-granted abilities
  - [x] 7.1 Add granted_abilities field and methods to Item/ItemRegistry
    - Add `granted_abilities: Vec<AbilityId>` field to `Item` struct in `crates/rpg-toolkit-common/src/item.rs` with `#[serde(default)]`
    - Initialize as empty `Vec` in `create_item`
    - Add `add_granted_ability(id, ability_id)` method — validates equippable category, non-empty ID, rejects duplicates, max 4
    - Add `remove_granted_ability(id, ability_id)` method
    - Update `change_category` — clear `granted_abilities` when switching to Consumable or KeyItem
    - _Requirements: 5.1, 5.5, 5.6, 5.7, 5.8, 5.10_

  - [ ]* 7.2 Write property test for granted abilities category and count constraint
    - **Property 8: Granted abilities category and count constraint**
    - **Validates: Requirements 5.1, 5.8**
    - Create `crates/rpg-toolkit-common/tests/properties/item_granted_abilities.rs`
    - Generate arbitrary items with granted abilities
    - Assert Consumable/KeyItem always have empty granted_abilities
    - Assert equippable items have at most 4 granted abilities

  - [x] 7.3 Write property test for item serialization with granted abilities
    - **Property 9: Item serialization round-trip with granted abilities**
    - **Validates: Requirements 5.10**
    - Add test in `crates/rpg-toolkit-common/tests/properties/item_granted_abilities.rs`
    - Serialize `ItemRegistry` with granted abilities, deserialize back, assert equality

  - [x] 7.4 Add Granted Abilities section to item panel UI
    - Modify `crates/rpg-toolkit-editor/src/plugins/item_panel.rs`
    - Add "Granted Abilities" section visible only for Weapon/Armor/Accessory items
    - Display each granted ability by display name + category bracket
    - Add searchable dropdown to add abilities
    - Add remove button per entry
    - Show "No abilities available" when registry is empty
    - Show error when max 4 reached or duplicate
    - Add `granted_ability_search_buffer`, `granted_ability_error` to `ItemPanelState`
    - _Requirements: 5.2, 5.3, 5.4, 5.5, 5.6, 5.7, 5.8, 5.9_

- [x] 8. Implement character visual assets
  - [x] 8.1 Add VisualAssets struct and fields to Character
    - Add `VisualAssets { spritesheet: Option<String>, face_portrait: Option<String>, status_portrait: Option<String> }` struct to `crates/rpg-toolkit-common/src/character.rs`
    - Derive `Default` (all `None`)
    - Add `visual_assets: VisualAssets` field to `Character` with `#[serde(default)]`
    - Initialize as `VisualAssets::default()` in `create_character`
    - Add `set_visual_asset(id, asset_type, path)` method — trims, if empty → None, else truncate to 260 and store
    - Add `clear_visual_asset(id, asset_type)` method — sets field to None
    - Define `VisualAssetType` enum: `Spritesheet`, `FacePortrait`, `StatusPortrait`
    - _Requirements: 6.1, 6.4, 6.5, 6.6, 6.7, 6.8_

  - [ ]* 8.2 Write property test for visual asset path invariant
    - **Property 10: Visual asset path invariant**
    - **Validates: Requirements 6.1, 6.4, 6.5, 6.8**
    - Create `crates/rpg-toolkit-common/tests/properties/character_visual_assets.rs`
    - Generate arbitrary characters with visual assets
    - Assert each field is either None or trimmed non-empty string ≤ 260 chars
    - Assert setting whitespace-only path results in None

  - [x] 8.3 Add Visual Assets section to character panel UI
    - Modify `crates/rpg-toolkit-editor/src/plugins/character_panel.rs`
    - Add "Visual Assets" section with three labeled single-line text inputs
    - Show placeholder "No asset assigned" when None
    - Truncate to 260 chars
    - On lost focus: trim, if empty → set to None, otherwise store
    - Add clear button per field
    - Mark `has_unsaved_character_changes` on any modification
    - Add `spritesheet_buffer`, `face_portrait_buffer`, `status_portrait_buffer` to panel state
    - _Requirements: 6.2, 6.3, 6.4, 6.5, 6.6, 6.8, 6.9_

- [x] 9. Final checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [x] 10. Add file picker to character visual assets
  - [x] 10.1 Add "Browse..." buttons to character visual asset fields
    - Modify `crates/rpg-toolkit-editor/src/plugins/character_panel.rs`
    - Add a "Browse..." button next to each visual asset text input (spritesheet, face portrait, status portrait)
    - On click, open `rfd::FileDialog::new()` with filter for image files (png, jpg, jpeg)
    - On file selection, populate the corresponding text buffer with the file path
    - Truncate to 260 characters, then call `CharacterRegistry::set_visual_asset()`
    - Mark `has_unsaved_character_changes` on successful selection
    - If dialog is cancelled, leave buffer unchanged
    - Retain existing text input for manual entry
    - _Requirements: 7.1, 7.2, 7.3, 7.4, 7.5, 7.6, 7.7_

- [x] 11. Add file picker to enemy portrait
  - [x] 11.1 Add "Browse..." button to enemy portrait field
    - Modify `crates/rpg-toolkit-editor/src/plugins/enemy_panel.rs`
    - Add a "Browse..." button on the same row as the portrait text input
    - On click, open `rfd::FileDialog::new()` with filter for image files (png, jpg, jpeg)
    - On file selection, populate `portrait_buffer` with the file path, truncate to 260 chars
    - Call `EnemyRegistry::set_portrait()` to commit — if validation fails (empty after trim), show error in `portrait_error`
    - Mark `has_unsaved_enemy_changes` on successful selection
    - If dialog is cancelled, leave buffer and registry unchanged
    - _Requirements: 8.1, 8.2, 8.3, 8.4, 8.5, 8.6_

- [x] 12. Implement character starting equipment data model
  - [x] 12.1 Add starting_equipment field and methods to Character/CharacterRegistry
    - Add `starting_equipment: Vec<ItemId>` field to `Character` struct in `crates/rpg-toolkit-common/src/character.rs` with `#[serde(default)]`
    - Add `use crate::item::ItemId;` import
    - Initialize as empty `Vec` in `create_character`
    - Add `add_starting_equipment(id, item_id)` method — trims, validates non-empty, rejects duplicates, max 20
    - Add `remove_starting_equipment(id, item_id)` method — returns error if not found
    - _Requirements: 9.1, 9.4, 9.5, 9.6, 9.7, 9.10_

  - [ ]* 12.2 Write property test for starting equipment invariant
    - **Property 11: Starting equipment count and uniqueness invariant**
    - **Validates: Requirements 9.1, 9.5, 9.6**
    - Create `crates/rpg-toolkit-common/tests/properties/character_starting_equipment.rs`
    - Generate arbitrary characters with starting equipment lists
    - Assert starting_equipment.len() <= 20
    - Assert no duplicate item IDs in starting_equipment
    - Assert all item IDs are non-empty trimmed strings

- [x] 13. Implement character starting equipment UI
  - [x] 13.1 Add Starting Equipment section to character panel
    - Modify `crates/rpg-toolkit-editor/src/plugins/character_panel.rs`
    - Add "Starting Equipment" section below "Learnable Abilities"
    - Display entries sorted by display name (case-insensitive), showing item name + category bracket
    - Show raw item ID as fallback if item not found in ItemRegistry
    - Add searchable dropdown (using `searchable_combobox`) populated from `ItemRegistry` formatted as `"{display_name} [{category}]"`
    - Add remove button per entry
    - Show "No items available" when ItemRegistry is empty
    - Show error when max 20 reached or duplicate
    - Add `starting_equipment_search_buffer`, `starting_equipment_error` to `CharacterPanelState`
    - Mark `has_unsaved_character_changes` on any modification
    - _Requirements: 9.2, 9.3, 9.4, 9.5, 9.6, 9.7, 9.8, 9.9, 9.11_

- [ ] 14. Final checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

## Notes

- Tasks marked with `*` are optional and can be skipped for faster MVP
- Each task references specific requirements for traceability
- Checkpoints ensure incremental validation
- Property tests validate universal correctness properties from the design document
- Unit tests validate specific examples and edge cases
- All new data model fields use `#[serde(default)]` for backward compatibility with existing project files
- The existing `searchable_combobox.rs` module is reused for dropdown selectors across Requirements 2, 3, 5, and 9
- The `rfd` crate (already a workspace dependency) provides native file dialogs for Requirements 7 and 8

## Task Dependency Graph

```json
{
  "waves": [
    { "id": 0, "tasks": ["1.1", "3.1", "6.1", "7.1", "8.1"] },
    { "id": 1, "tasks": ["1.2", "1.3", "1.4", "3.2", "3.3", "6.2", "6.3", "7.2", "7.3", "8.2"] },
    { "id": 2, "tasks": ["2.1", "2.2", "4.1", "6.4", "7.4", "8.3"] },
    { "id": 3, "tasks": ["10.1", "11.1", "12.1"] },
    { "id": 4, "tasks": ["12.2", "13.1"] }
  ]
}
```
