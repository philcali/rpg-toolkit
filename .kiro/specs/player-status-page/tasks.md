# Implementation Plan: Player Status Page

## Overview

Implements the `StatusScenePlugin` in the `rpg-toolkit-scenes` crate following the same architectural pattern as `ShopScenePlugin`. The plan is structured so that pure helper functions and data models are built first (immediately testable), property-based tests validate them next, and ECS systems and UI hierarchy come last. Every task leaves the codebase in a compilable state.

## Tasks

- [x] 1. Define data models, constants, and pure helper functions
  - [x] 1.1 Create `status_scene.rs` with data models, enums, and constants
    - Add `status_scene` module to `crates/rpg-toolkit-scenes/src/lib.rs`
    - Define `StatusMode`, `DetailView`, `InventoryTab`, `StatusUiState` resource
    - Define `PartyMemberDisplayData` and `InventoryItemDisplayData` structs
    - Define `StatusSceneMarker` component and internal marker components
    - Define `MAX_PARTY_DISPLAY` constant (4)
    - Export `StatusScenePlugin` from `lib.rs` (as empty struct implementing Plugin with no systems yet)
    - Ensure the file compiles with `cargo check -p rpg-toolkit-scenes`
    - _Requirements: 1.1, 1.4, 2.9, 6.1_

  - [x] 1.2 Implement pure helper functions (`compute_effective_stat`, `clamp_selection`, `next_tab`, `prev_tab`, `tab_to_category`)
    - Implement `compute_effective_stat(base_value: u32, growth_value: u32, level: u32) -> u32` with saturating arithmetic
    - Implement `clamp_selection(index: usize, len: usize) -> usize` returning 0 for empty lists
    - Implement `next_tab` and `prev_tab` following fixed order Weapon→Armor→Accessory→Consumable→KeyItem with clamping at boundaries
    - Implement `tab_to_category` mapping `InventoryTab` to `ItemCategory`
    - Make all functions `pub` for testability
    - _Requirements: 2.2, 3.3, 3.4, 4.6, 5.6_

  - [x] 1.3 Implement `resolve_party_display_data` helper
    - Accept `party: &[String]`, `character_registry: &CharacterRegistry`, `progress: &HashMap<String, CharacterProgress>`
    - Skip members not found in registry, preserve input order
    - Compute `effective_hp` via `compute_effective_stat` using HP stat base/growth and Level stat base_value from progress (defaulting to character's Level stat base_value if no progress entry)
    - Set `has_portrait` based on `visual_assets.face_portrait.is_some()`
    - Truncate result to `MAX_PARTY_DISPLAY` entries
    - _Requirements: 2.2, 2.3, 2.4, 2.5, 2.6, 2.9_

  - [x] 1.4 Implement `resolve_inventory_tab_data` helper
    - Accept `inventory: &HashMap<String, u32>`, `item_registry: &ItemRegistry`, `tab: InventoryTab`
    - Filter items by tab category using `tab_to_category`, skip unresolvable item_ids
    - Sort results case-insensitively by `display_name`
    - Include `description`, `stat_modifiers`, icon info, and quantity from inventory map
    - _Requirements: 4.1, 4.2, 4.3, 4.8_

  - [x] 1.5 Implement `resolve_ordered_ids` generic helper for equipment and ability resolution
    - Accept an ordered slice of string IDs and a lookup closure or HashMap
    - Return only IDs present in the lookup, preserving input order
    - Used for both `starting_equipment` → `ItemRegistry` and `learned_abilities` → `AbilityRegistry`
    - _Requirements: 3.5, 3.6, 3.7, 3.8_

- [ ] 2. Property-based tests for pure helpers
  - [ ]* 2.1 Write property test for `compute_effective_stat`
    - **Property 1: Effective stat computation is correct**
    - **Validates: Requirements 2.2, 3.3, 3.4**
    - Generate random `(base_value: u32, growth_value: u32, level: u32)` with level ≥ 1
    - Assert result equals `base_value.saturating_add(growth_value.saturating_mul(level.saturating_sub(1)))`

  - [ ]* 2.2 Write property test for `clamp_selection`
    - **Property 6: Selection index clamping stays within valid bounds**
    - **Validates: Requirements 5.6, 5.8**
    - Generate random `(index: usize, len: usize)`
    - Assert: when len > 0, result ∈ [0, len-1]; when len == 0, result == 0

  - [ ]* 2.3 Write property test for `next_tab` / `prev_tab`
    - **Property 5: Tab navigation follows the fixed ordering**
    - **Validates: Requirements 4.6**
    - For all 5 InventoryTab variants: `prev_tab(next_tab(tab)) == tab` (except at boundaries)
    - Assert `next_tab(KeyItem) == KeyItem` and `prev_tab(Weapon) == Weapon`
    - Assert full chain: Weapon→Armor→Accessory→Consumable→KeyItem

  - [ ]* 2.4 Write property test for `resolve_party_display_data`
    - **Property 2: Party member resolution filters unresolvable IDs, truncates to cap, and computes correct display data**
    - **Validates: Requirements 2.2, 2.3, 2.4, 2.5, 2.6, 2.9**
    - Generate random party lists, character registries with varied stats/portraits, progress maps
    - Assert output length ≤ MAX_PARTY_DISPLAY, all entries correspond to registry entries, order preserved, effective_hp correct, has_portrait correct

  - [ ]* 2.5 Write property test for `resolve_ordered_ids`
    - **Property 3: Ordered ID resolution preserves order and filters missing entries**
    - **Validates: Requirements 3.5, 3.6, 3.7, 3.8**
    - Generate random ID lists and HashMaps with subset of IDs present
    - Assert output contains only present IDs, preserving input order

  - [ ]* 2.6 Write property test for `resolve_inventory_tab_data`
    - **Property 4: Inventory tab resolution filters unresolvable items and sorts case-insensitively**
    - **Validates: Requirements 4.1, 4.3, 4.8**
    - Generate random inventory maps and item registries with mixed categories
    - Assert all output items exist in registry and match tab category, output sorted case-insensitively, quantities match

  - [ ]* 2.7 Write property test for `StatusUiState` mode transitions
    - **Property 7: Sub-page selection indices are preserved independently**
    - **Validates: Requirements 5.7**
    - Generate random sequences of mode changes interleaved with selection index modifications
    - Assert changing `mode` never modifies `party_selection` or `inventory_selection`

- [x] 3. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [x] 4. Implement ECS systems (spawn, input, despawn)
  - [x] 4.1 Implement `spawn_status_ui` system
    - Register as `OnEnter(AppPhase::Status)` system
    - Check for required resources (`CharacterRegistryRes`, `ItemRegistryRes`, `AbilityRegistryRes`); log `warn!()` and return early if missing
    - Read `PartyState`, resolve party display data via `resolve_party_display_data`
    - Read `InventoryState`, resolve initial inventory tab (Weapon) via `resolve_inventory_tab_data`
    - Insert `StatusUiState` resource with defaults (mode=PartyList, party_selection=0, inventory_tab=Weapon)
    - Spawn UI root node with `StatusSceneMarker`
    - _Requirements: 1.1, 2.1, 2.6, 4.1, 6.2, 6.3, 6.4, 6.5_

  - [x] 4.2 Implement `despawn_status_ui` system
    - Register as `OnExit(AppPhase::Status)` system
    - Query all entities with `StatusSceneMarker` and despawn them
    - Remove `StatusUiState` resource
    - _Requirements: 1.3, 1.4, 6.6_

  - [x] 4.3 Implement `status_input` system — navigation and sub-page switching
    - Register as `Update` system with `run_if(in_state(AppPhase::Status))`
    - Early return if `StatusUiState` resource not present
    - Handle Escape/Backspace: if `detail_view == None`, transition to `AppPhase::InGame`; if in CharacterDetail, return to PartyList
    - Handle Up/Down (ArrowUp/W, ArrowDown/S): adjust selection index using `clamp_selection`
    - Handle Left/Right (ArrowLeft/A, ArrowRight/D): switch `StatusMode` between PartyList and Inventory (preserve each sub-page's selection index independently); when in Inventory mode, switch tabs via `next_tab`/`prev_tab` instead
    - Handle Enter/Space: from PartyList, set `detail_view = CharacterDetail`; from Inventory, no action (read-only)
    - Skip Up/Down/Enter when list length is 0
    - _Requirements: 1.2, 5.1, 5.2, 5.3, 5.4, 5.5, 5.6, 5.7, 5.8_

  - [x] 4.4 Add `CharacterRegistryRes` and `AbilityRegistryRes` resource wrappers
    - Define `CharacterRegistryRes` wrapping `CharacterRegistry` in `status_scene.rs`
    - Define `AbilityRegistryRes` wrapping `AbilityRegistry` in `status_scene.rs`
    - Export both from crate `lib.rs`
    - _Requirements: 6.4, 6.5_

- [x] 5. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [x] 6. Build UI hierarchy
  - [x] 6.1 Implement Party List sub-page UI hierarchy
    - Spawn header text, sub-page tab indicator, party list container with member rows
    - Each row: portrait placeholder/image area, display name text, level text, HP text
    - Highlight selected row with distinct background color
    - Handle empty party with "No party members" indicator
    - Attach `StatusSceneMarker` to all entities
    - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.6, 2.7_

  - [x] 6.2 Implement Character Detail view UI hierarchy
    - Left column: face portrait area (placeholder if none)
    - Right column: name + level, stats list (excluding Level row) with effective values, equipment list (resolved names), abilities list (resolved names)
    - Skips unresolvable equipment/ability IDs
    - Attach `StatusSceneMarker` to all entities
    - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5, 3.6, 3.7, 3.8, 3.9_

  - [x] 6.3 Implement Inventory sub-page UI hierarchy
    - Tab bar with 5 category tabs, highlight active tab
    - Item list container with rows: icon area, display name, quantity
    - Detail panel: item description + stat modifiers formatted with sign prefix
    - Empty category indicator when tab has no items
    - Attach `StatusSceneMarker` to all entities
    - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5, 4.6, 4.7_

  - [x] 6.4 Wire UI updates in `status_input` system
    - On selection change: update highlight colors on party rows / inventory rows
    - On sub-page switch: show/hide party list vs inventory containers
    - On tab switch: rebuild inventory list from `resolve_inventory_tab_data`, reset `inventory_selection` to 0 via `clamp_selection`
    - On detail view enter/exit: show/hide character detail panel vs party list
    - On item highlight change: update inventory detail panel text
    - _Requirements: 2.7, 2.8, 4.5, 4.6, 5.1, 5.2, 5.3, 5.4_

- [~] 7. Final checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

## Notes

- Tasks marked with `*` are optional and can be skipped for faster MVP
- Each task references specific requirements for traceability
- Checkpoints ensure incremental validation
- Property tests validate universal correctness properties from the design document
- Unit tests validate specific examples and edge cases
- The design uses Rust throughout; all code targets the `rpg-toolkit-scenes` crate
- Property tests go in `tests/properties/` following existing project conventions (one file per test binary)
- `CharacterRegistryRes` and `AbilityRegistryRes` are new wrapper resources analogous to `ItemRegistryRes` / `ShopRegistryRes`

## Task Dependency Graph

```json
{
  "waves": [
    { "id": 0, "tasks": ["1.1"] },
    { "id": 1, "tasks": ["1.2", "1.4", "1.5"] },
    { "id": 2, "tasks": ["1.3"] },
    { "id": 3, "tasks": ["2.1", "2.2", "2.3", "2.5", "2.6", "2.7"] },
    { "id": 4, "tasks": ["2.4", "4.4"] },
    { "id": 5, "tasks": ["4.1", "4.2"] },
    { "id": 6, "tasks": ["4.3"] },
    { "id": 7, "tasks": ["6.1", "6.2", "6.3"] },
    { "id": 8, "tasks": ["6.4"] }
  ]
}
```
