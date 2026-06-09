# Implementation Plan: Character Editor

## Overview

This plan implements the Character Editor feature across `rpg-toolkit-common` (data model) and `rpg-toolkit-editor` (UI and integration). The approach is bottom-up: data model first, then serialization integration, then the editor mode infrastructure, and finally the character panel UI. Property-based tests validate core invariants throughout.

## Tasks

- [x] 1. Define character data model in rpg-toolkit-common
  - [x] 1.1 Create `character.rs` module with `Stat`, `Character`, `CharacterRegistry` structs and constants
    - Create `crates/rpg-toolkit-common/src/character.rs`
    - Define `CharacterId` type alias (`String`)
    - Define `Stat` struct with `name: String`, `base_value: u32`, `growth_value: u32` (derive Clone, Debug, PartialEq, Eq, Serialize, Deserialize)
    - Define `Character` struct with `id: CharacterId`, `display_name: String`, `stats: Vec<Stat>`
    - Define `CharacterRegistry` struct with `characters: HashMap<CharacterId, Character>` (derive Default, Serialize, Deserialize)
    - Define `OPTIONAL_STATS` and `REQUIRED_STATS` constants
    - Implement `create_character`, `delete_character`, `rename_character`, `add_stat`, `remove_stat`, `update_stat`, `compute_stat_value`, and `sorted_characters` methods
    - Add `CharacterValidationError(String)` variant to `CommonError`
    - Register module in `lib.rs` and add public exports
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 1.7, 1.8, 1.9, 1.10, 5.4, 5.5, 7.2, 7.5, 8.1_

  - [x] 1.2 Write property test: Required stats invariant (Property 2)
    - **Property 2: Required stats invariant**
    - Create `tests/properties/character_invariants.rs`
    - Generate a character via `create_character`, apply random sequences of `add_stat`/`remove_stat`/`update_stat`, assert HP and Level stats are always present
    - Register test in `tests/properties/Cargo.toml`
    - **Validates: Requirements 1.5, 1.6, 4.5, 5.4**

  - [ ]* 1.3 Write property test: Duplicate stat rejection (Property 3)
    - **Property 3: Duplicate stat rejection**
    - In `tests/properties/character_invariants.rs`
    - Generate a character with stats, attempt to add a stat with an already-existing name, assert error returned and stat list unchanged
    - **Validates: Requirements 1.10**

  - [ ]* 1.4 Write property test: Whitespace-only name rejection (Property 5)
    - **Property 5: Whitespace-only name rejection**
    - In `tests/properties/character_invariants.rs`
    - Generate whitespace-only strings, assert `create_character` and `rename_character` return errors and registry is unchanged
    - **Validates: Requirements 3.3, 4.3**

  - [ ]* 1.5 Write property test: Stat progression computation (Property 4)
    - **Property 4: Stat progression computation**
    - Create `tests/properties/character_progression.rs`
    - Generate arbitrary `base_value`, `growth_value`, and `level` in [1, 99], verify `compute_stat_value` returns `min(base + growth * (level - 1), u32::MAX)`
    - Register test in `tests/properties/Cargo.toml`
    - **Validates: Requirements 7.2, 7.5**

  - [ ]* 1.6 Write property test: Character list ordering (Property 6)
    - **Property 6: Character list ordering**
    - In `tests/properties/character_invariants.rs`
    - Generate a registry with multiple characters with random names, verify `sorted_characters` returns case-insensitive alphabetical order
    - **Validates: Requirements 8.1**

- [x] 2. Integrate characters into project serialization
  - [x] 2.1 Add `characters` field to `ProjectFile` and `ProjectManifest`
    - Add `#[serde(default)] pub characters: CharacterRegistry` to `ProjectFile` in `crates/rpg-toolkit-common/src/project.rs`
    - Add `#[serde(default)] pub characters: CharacterRegistry` to `ProjectManifest` in `crates/rpg-toolkit-common/src/manifest.rs`
    - Update `ProjectFile::new()` constructor to accept characters parameter
    - Update `to_manifest()` to include characters
    - Update deserialization validation to check for duplicate character IDs
    - _Requirements: 2.1, 2.3, 2.4, 2.5_

  - [x] 2.2 Add `characters` field to editor `Project` resource and update serialization plugin
    - Add `pub characters: CharacterRegistry` to `Project` struct in `crates/rpg-toolkit-editor/src/data/project.rs`
    - Update `to_project_file()` in `serialization.rs` to include `project.characters`
    - Update `load_project_from_dir`, `load_project_from_zip`, `load_project_from_json` to populate `project.characters` from deserialized `ProjectFile`
    - Update `NewProject` action to reset characters to default
    - _Requirements: 2.1, 2.2, 2.5_

  - [x] 2.3 Write property test: Character serialization round-trip (Property 1)
    - **Property 1: Character serialization round-trip**
    - Create `tests/properties/character_round_trip.rs`
    - Generate arbitrary `CharacterRegistry` instances, wrap in `ProjectFile`, serialize to JSON, deserialize, assert equality
    - Register test in `tests/properties/Cargo.toml`
    - **Validates: Requirements 2.1, 2.2**

- [x] 3. Checkpoint
  - Ensure all tests pass, ask the user if questions arise.

- [x] 4. Introduce AppEditorMode and gate existing plugins
  - [x] 4.1 Add `AppEditorMode` resource to editor state
    - Define `AppEditorMode` enum (`MapEditor`, `CharacterEditor`) with `Default` → `MapEditor` in `crates/rpg-toolkit-editor/src/data/state.rs`
    - Derive `Clone, Copy, Debug, Default, PartialEq, Eq, Resource`
    - Export from `data/mod.rs`
    - Initialize the resource in `AppShellPlugin::build` (or `main.rs`)
    - _Requirements: 3.1, 8.2_

  - [x] 4.2 Add run conditions to existing map editor plugins
    - Add `.run_if(resource_equals(AppEditorMode::MapEditor))` to UI systems in: `TilePalettePlugin`, `CanvasPlugin`, `LayerPanelPlugin`, `ToolbarPlugin`, `PaintingPlugin`, `AttributePlugin`, `SpritesheetPlugin`, `DialogTextPanelPlugin`, `UndoRedoPlugin`
    - Import `AppEditorMode` in each plugin file
    - Ensure the map tab bar in `AppShellPlugin` only renders in MapEditor mode
    - _Requirements: 3.1, 4.1_

  - [x] 4.3 Add mode switcher to the app shell menu bar
    - Add a "Mode" menu button to the menu bar in `app_shell_ui`
    - Include selectable labels for "🗺 Map Editor" and "👤 Character Editor"
    - Switching updates the `AppEditorMode` resource
    - Add `ResMut<AppEditorMode>` parameter to `app_shell_ui` system
    - _Requirements: 3.1, 8.2_

- [x] 5. Implement CharacterPanelPlugin UI
  - [x] 5.1 Create `character_panel.rs` plugin with panel state and registration
    - Create `crates/rpg-toolkit-editor/src/plugins/character_panel.rs`
    - Define `CharacterPanelPlugin` struct implementing `Plugin`
    - Define `CharacterPanelState` resource (selected_character, create_dialog_open, create_name_buffer, create_error, delete_confirm_target, preview_level, name_edit_buffer, name_edit_error)
    - Register plugin in `plugins/mod.rs` and `main.rs`
    - Add a main UI system that gates on `AppEditorMode::CharacterEditor`
    - _Requirements: 3.1, 3.4, 8.2, 8.3_

  - [x] 5.2 Implement character list panel (left side)
    - Render a left `SidePanel` with a scrollable, alphabetically sorted list of character names
    - Highlight the selected character
    - Show an "empty state" message when no characters exist
    - Include a "New Character" button at the top of the list
    - _Requirements: 8.1, 8.2, 8.3, 6.4_

  - [x] 5.3 Implement character creation dialog
    - Render creation form (name input, max 50 chars) when create_dialog_open is true
    - Validate name is non-empty/non-whitespace on confirm
    - Call `CharacterRegistry::create_character`, handle errors inline
    - Auto-select newly created character
    - Cancel discards input
    - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5_

  - [x] 5.4 Implement character detail editor (center panel)
    - Render editable display name field with inline validation
    - Render stat table: name, base_value input, growth_value input for each stat
    - Required stats (HP, Level) show no delete button
    - Optional stats show a delete button
    - Non-numeric input in stat fields is rejected, retaining previous value
    - Add an "Add Stat" button/dropdown showing only unassigned optional stats; disabled when all assigned
    - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5, 4.6, 4.7, 5.1, 5.2, 5.3, 5.4, 5.5_

  - [x] 5.5 Implement stat progression preview (right panel)
    - Render a right `SidePanel` with a level input (clamped 1–99, default 1)
    - Display computed stat values for all stats at the chosen preview level
    - Recalculate on stat value changes or level changes
    - Cap displayed values at u32::MAX
    - _Requirements: 7.1, 7.2, 7.3, 7.4, 7.5_

  - [x] 5.6 Implement character deletion with confirmation
    - Render delete button per character (in list or detail)
    - Show confirmation dialog with character name before deletion
    - On confirm: remove character, select first remaining or show empty state
    - On cancel: retain character unchanged
    - _Requirements: 6.1, 6.2, 6.3, 6.4, 6.5_

- [~] 6. Checkpoint
  - Ensure all tests pass, ask the user if questions arise.

- [x] 7. Final integration and wiring
  - [x] 7.1 Wire character data into project save/load cycle end-to-end
    - Verify `prepare_assets_for_save` does not interfere with character data
    - Ensure the `to_manifest` path includes characters for directory-based saves
    - Ensure ZIP save includes characters in the manifest
    - Mark project as having unsaved changes when character mutations occur
    - _Requirements: 2.1, 2.2, 2.5_

  - [ ]* 7.2 Write unit tests for UI state transitions and edge cases
    - Test: creating a character auto-selects it
    - Test: deleting selected character selects first remaining
    - Test: deleting last character shows empty state
    - Test: preview level clamping at 1 and 99
    - Test: adding all 7 optional stats disables add action
    - Test: backward-compatible deserialization with no "characters" key
    - _Requirements: 3.4, 6.4, 6.5, 7.3, 5.1, 2.3_

- [~] 8. Final checkpoint
  - Ensure all tests pass, ask the user if questions arise.

## Notes

- Tasks marked with `*` are optional and can be skipped for faster MVP
- Each task references specific requirements for traceability
- Checkpoints ensure incremental validation
- Property tests validate universal correctness properties from the design document
- Unit tests validate specific examples and edge cases
- The `AppEditorMode` concept is additive — it does not modify any internal logic of existing plugins, only gates their rendering
- The `uuid` crate is already available in the workspace for character ID generation

## Task Dependency Graph

```json
{
  "waves": [
    { "id": 0, "tasks": ["1.1"] },
    { "id": 1, "tasks": ["1.2", "1.3", "1.4", "1.5", "1.6", "2.1"] },
    { "id": 2, "tasks": ["2.2", "2.3"] },
    { "id": 3, "tasks": ["4.1"] },
    { "id": 4, "tasks": ["4.2", "4.3"] },
    { "id": 5, "tasks": ["5.1"] },
    { "id": 6, "tasks": ["5.2", "5.3", "5.5", "5.6"] },
    { "id": 7, "tasks": ["5.4"] },
    { "id": 8, "tasks": ["7.1", "7.2"] }
  ]
}
```
