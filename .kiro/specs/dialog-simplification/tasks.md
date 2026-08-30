# Implementation Plan: Dialog Simplification

## Overview

This plan removes the legacy `DialogTextData::Id` variant, the standalone `dialog_texts` and `face_portraits` registries, and the `DialogTextPanel`/`TextIdIndex` editor infrastructure. Face portraits are consolidated into `Character::VisualAssets`, the action editor is simplified to inline-only text entry with character-based portrait selection, and a categorized searchable dropdown replaces the flat action type selector. Implementation progresses from data model changes in `rpg-toolkit-common`, through renderer graceful degradation, to editor UI cleanup and new features.

## Tasks

- [x] 1. Simplify DialogTextData enum and ProjectFile in rpg-toolkit-common
  - [x] 1.1 Implement custom Deserialize for DialogTextData to convert Id → Inline("")
    - In `crates/rpg-toolkit-common/src/map.rs`, remove the `Id(String)` variant from the public `DialogTextData` enum
    - Remove the derived `Deserialize` and implement a custom `Deserialize` using a private `Raw` helper enum that accepts both `Inline` and `Id` tags
    - Map `Raw::Id(_)` to `DialogTextData::Inline(String::new())`
    - Keep `#[derive(Serialize)]` with `#[serde(tag = "type", content = "value")]` so only `Inline` is ever written
    - Add unit test: deserializing `{"type":"Id","value":"foo"}` produces `Inline("")`
    - Add unit test: deserializing `{"type":"Inline","value":"hello"}` produces `Inline("hello")`
    - _Requirements: 1.1, 1.2, 1.3, 1.4_

  - [x] 1.2 Remove dialog_texts and face_portraits from ProjectFile public API
    - In `crates/rpg-toolkit-common/src/project.rs`, remove the public `dialog_texts: HashMap<String, String>` field
    - Remove the public `face_portraits: HashMap<String, String>` field
    - Add private serde sink fields with `#[serde(default, skip_serializing, rename = "dialog_texts")]` and equivalent for `face_portraits`
    - Use `serde_json::Value` or `HashMap<String, String>` with `#[serde(deserialize_with = "...")]` to tolerate malformed JSON types (arrays, numbers, etc.) by defaulting to empty
    - Update the `ProjectFile` constructor and any `new()` / builder methods to remove these parameters
    - _Requirements: 2.1, 2.2, 2.3, 3.1, 3.2, 3.3, 9.4_

  - [x] 1.3 Ensure face_portrait field exists on Character VisualAssets
    - In `crates/rpg-toolkit-common/src/character.rs`, verify `VisualAssets` has `pub face_portrait: Option<String>` with `#[serde(default)]`
    - If `face_portrait` is already present (it appears to be per the design), confirm trim and truncation logic: add a `set_face_portrait` or equivalent method that trims whitespace and truncates to 260 characters
    - Add unit test: setting a whitespace-padded path stores the trimmed value
    - Add unit test: setting a path longer than 260 chars stores at most 260 chars
    - _Requirements: 4.1, 4.2, 4.3_

- [~] 2. Checkpoint - Verify common crate compiles and tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [ ] 3. Property tests for data model changes
  - [ ]* 3.1 Write property test for legacy Id deserialization
    - **Property 1: Legacy Id deserialization produces Inline empty string**
    - **Validates: Requirements 1.2, 1.3**
    - In `crates/rpg-toolkit-common/tests/properties/`, create `dialog_text_id_deserialization.rs`
    - Use proptest to generate arbitrary strings for the Id value field
    - Serialize as `{"type":"Id","value":"<s>"}`, deserialize, assert result is `DialogTextData::Inline("")`

  - [ ]* 3.2 Write property test for Character serialization round-trip
    - **Property 2: Character serialization round-trip**
    - **Validates: Requirements 4.4**
    - In `crates/rpg-toolkit-common/tests/properties/`, create `character_face_portrait_round_trip.rs`
    - Use proptest to generate arbitrary valid `Character` structs with various `VisualAssets` (Some/None face_portrait)
    - Serialize to JSON and deserialize back, assert equivalence

  - [ ]* 3.3 Write property test for face_portrait path trimming and truncation
    - **Property 3: Character face_portrait path trimming and truncation**
    - **Validates: Requirements 4.3**
    - In `crates/rpg-toolkit-common/tests/properties/`, create `face_portrait_trim_truncation.rs`
    - Use proptest to generate strings with leading/trailing whitespace and lengths > 260
    - Assert stored value is trimmed and at most 260 characters

  - [ ]* 3.4 Write property test for project file migration round-trip
    - **Property 5: Project file migration round-trip**
    - **Validates: Requirements 1.4, 2.1, 2.2, 3.1, 3.2, 9.1, 9.2, 9.3**
    - In `crates/rpg-toolkit-common/tests/properties/`, create `project_migration_round_trip.rs`
    - Use proptest to generate legacy project JSON with `dialog_texts`, `face_portraits`, and `DialogTextData::Id` values
    - Load, save, reload and assert the second load produces an equivalent `ProjectFile`

  - [ ]* 3.5 Write property test for malformed legacy field tolerance
    - **Property 6: Malformed legacy field tolerance**
    - **Validates: Requirements 9.4**
    - In `crates/rpg-toolkit-common/tests/properties/`, create `malformed_legacy_fields.rs`
    - Use proptest to generate random JSON value types (array, number, boolean, nested object) for `dialog_texts` and `face_portraits` fields
    - Assert deserialization of `ProjectFile` succeeds with empty defaults

- [x] 4. Update renderer for graceful degradation
  - [~] 4.1 Simplify handle_dialog_event to use direct asset path for face_portrait
    - In `crates/rpg-toolkit-renderer/src/systems/dialog.rs`, remove the `face_portraits` registry lookup block in `handle_dialog_event`
    - The `resolved_config.face_portrait` already contains the direct asset path (set by editor) — use as-is
    - Remove the `DialogText::Id` match arm or update it to resolve to empty string with a `warn!()` log
    - If `DialogText` enum in the renderer differs from `DialogTextData` in common, update accordingly to handle the `Id` variant gracefully
    - Add unit test: renderer resolves `DialogText::Id("foo")` to empty string
    - Add unit test: renderer logs a warning when encountering `Id` variant
    - _Requirements: 5.1, 5.2, 5.3, 5.4_

  - [~] 4.2 Remove face_portraits field access from RendererProjectData
    - In `crates/rpg-toolkit-renderer/src/resources.rs` or wherever `RendererProjectData` is defined, remove references to `project_file.face_portraits`
    - Update any initialization code that copies `face_portraits` from `ProjectFile` to the renderer
    - _Requirements: 5.1_

- [~] 5. Checkpoint - Verify renderer crate compiles and tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [x] 6. Remove DialogTextPanel plugin and TextIdIndex from editor
  - [x] 6.1 Remove DialogTextPanelPlugin registration and TextIdIndex resource
    - In `crates/rpg-toolkit-editor/src/main.rs`, remove `DialogTextPanelPlugin` from plugin registration (`.add_plugins(DialogTextPanelPlugin)`)
    - Remove `.init_resource::<TextIdIndex>()` from app setup
    - Remove the imports of `DialogTextPanelPlugin` and `TextIdIndex` from the use statement
    - _Requirements: 7.1, 8.1_

  - [x] 6.2 Remove TextIdIndex usage from serialization.rs
    - In `crates/rpg-toolkit-editor/src/plugins/serialization.rs`, remove the `use crate::plugins::dialog_text_panel::{TextIdIndex, rebuild_text_id_index}` import
    - Remove `text_id_index: &mut ResMut<TextIdIndex>` parameters from `load_project_from_path` and related functions
    - Remove calls to `rebuild_text_id_index` and any `*text_id_index = ...` assignments
    - Remove the `dialog_texts` and `face_portraits` fields from the `Project` reconstruction in `load_project_from_path`
    - Update the `save_project` function to stop passing `dialog_texts` and `face_portraits` to `ProjectFile` serialization
    - _Requirements: 7.2, 8.2_

  - [x] 6.3 Remove TextIdIndex usage from undo_redo.rs
    - In `crates/rpg-toolkit-editor/src/plugins/undo_redo.rs`, remove the `use crate::plugins::dialog_text_panel::{TextIdIndex, update_text_id_index_for_tile}` import
    - Remove `mut text_id_index: ResMut<TextIdIndex>` parameters from the undo/redo systems
    - Remove all `EditCommandKind::InsertFacePortrait`, `UpdateFacePortrait`, `RemoveFacePortrait` match arms
    - Remove all `update_text_id_index_for_tile` calls and associated TextIdIndex update logic
    - _Requirements: 8.2, 8.3, 8.4_

  - [x] 6.4 Remove dialog_text_panel.rs module and exports
    - In `crates/rpg-toolkit-editor/src/plugins/mod.rs`, remove `pub mod dialog_text_panel;` and `pub use dialog_text_panel::{DialogTextPanelPlugin, TextIdIndex};`
    - Delete the file `crates/rpg-toolkit-editor/src/plugins/dialog_text_panel.rs`
    - _Requirements: 7.2, 7.3, 8.3_

  - [x] 6.5 Remove face_portraits and dialog_texts from editor Project struct
    - In `crates/rpg-toolkit-editor/src/data/project.rs` (or wherever the editor `Project` resource is defined), remove `pub dialog_texts: HashMap<String, String>` and `pub face_portraits: HashMap<String, String>` fields
    - Update all references in `serialization.rs`, `undo_redo.rs`, `project_settings_panel.rs`, `hotkey_panel.rs`, `npc_dialog.rs`, and `event_trigger_dialog.rs`
    - Replace `face_portraits` HashMap parameter in `action_editor_ui` functions with a character-based portrait list derived from `CharacterRegistry`
    - _Requirements: 2.3, 3.3_

- [~] 7. Checkpoint - Verify editor compiles after panel removal
  - Ensure all tests pass, ask the user if questions arise.

- [x] 8. Simplify action editor forms (remove TextId mode, character-based portrait)
  - [x] 8.1 Remove DialogTextMode::TextId and associated fields from ActionEditorState
    - In `crates/rpg-toolkit-editor/src/plugins/attribute/action_editor.rs`, remove `DialogTextMode::TextId` variant from the `DialogTextMode` enum (or remove the enum entirely if only `Inline` remains)
    - Remove `dialog_text_mode`, `dialog_text_id`, `selection_prompt_mode`, `selection_prompt_id` fields from `ActionEditorState`
    - Remove `label_mode` and `label_id` fields from `EditorChoice`
    - Update `Default` impls, `reset()`, `new_nested()`, `load_from_action()`, and `build_action()` methods to remove all TextId logic
    - _Requirements: 6.1, 6.5_

  - [x] 8.2 Update action editor forms to use character-based portrait dropdown
    - In `crates/rpg-toolkit-editor/src/plugins/attribute/action_editor_forms.rs`, update ShowDialog and ShowSelection form rendering
    - Replace the `face_portraits: &HashMap<String, String>` parameter with a character registry query
    - Build portrait dropdown entries from characters in `CharacterRegistry` that have `visual_assets.face_portrait` set to `Some(non-empty)`
    - Display as `(character_name, portrait_path)` pairs using the existing `searchable_combobox` utility
    - Remove any TextId mode toggle UI elements from dialog/selection forms
    - _Requirements: 6.2, 6.3, 6.4_

  - [x] 8.3 Update action_editor_ui.rs to remove face_portraits parameter threading
    - In `crates/rpg-toolkit-editor/src/plugins/attribute/action_editor_ui.rs`, replace all `face_portraits: &HashMap<String, String>` parameters with character registry access
    - Update `render_action_editor_list`, `render_nested_branch_editors`, and all nested action editor calls
    - Thread `CharacterRegistry` resource (or a pre-built portrait list) through the UI functions
    - Update callers in `event_trigger_dialog.rs`, `npc_dialog.rs`, `project_settings_panel.rs`, and `hotkey_panel.rs`
    - _Requirements: 6.3, 6.4_

- [x] 9. Implement categorized searchable action type dropdown
  - [x] 9.1 Define ACTION_CATEGORIES constant with category groupings
    - Create a new file or add to `crates/rpg-toolkit-editor/src/plugins/attribute/action_editor.rs` a constant `ACTION_CATEGORIES: &[ActionCategory]`
    - Define `ActionCategory` struct with `name: &'static str` and `actions: &'static [(ActionType, &'static str)]`
    - Assign all `ActionType` variants to categories per Requirement 10.8: Dialog (ShowDialog, ShowSelection), Movement (JumpTo, Jump, SetSpeed, MoveEntity), Camera (CameraFollow, CameraPan), Rewards (GiveCurrency, GiveExperience, GiveItem, LearnAbility, AddPartyMember), State (SetState, StateCheck, Branch, SaveGame, ChangePhase), Visual Effects (ScreenShake, StopScreenShake, FadeTransition, SetPlayerAppearance), System (Wait, OpenShop)
    - _Requirements: 10.1, 10.7, 10.8_

  - [x] 9.2 Implement categorized dropdown UI with search filter
    - In `crates/rpg-toolkit-editor/src/plugins/attribute/action_editor_ui.rs`, replace the flat action type `ComboBox` with a grouped dropdown
    - Add a `TextEdit::singleline` search filter input at the top of the dropdown
    - Use `egui::CollapsingHeader` for each category, listing matching `SelectableLabel` items beneath
    - When filter is active: hide empty categories, show only actions whose display name contains the filter (case-insensitive)
    - When filter is cleared: show all categories expanded with all actions
    - Add an `action_type_search: String` field to `ActionEditorState` for the filter buffer
    - _Requirements: 10.2, 10.3, 10.4, 10.5, 10.6_

- [ ] 10. Property tests for editor logic
  - [ ]* 10.1 Write property test for action type category filter
    - **Property 7: Action type category filter returns correct matches**
    - **Validates: Requirements 10.4, 10.5, 10.6**
    - In `crates/rpg-toolkit-editor` tests, use proptest to generate arbitrary filter strings
    - Assert the filtered action list equals exactly those actions whose display name contains the filter as a case-insensitive substring

  - [ ]* 10.2 Write property test for action type categories forming a partition
    - **Property 8: Action type categories form a partition**
    - **Validates: Requirements 10.1, 10.7**
    - Assert each `ActionType` variant appears in exactly one category
    - Assert the union of all category actions equals the full set of `ActionType` variants

  - [ ]* 10.3 Write property test for portrait dropdown population
    - **Property 4: Portrait dropdown population matches CharacterRegistry**
    - **Validates: Requirements 6.3, 6.4**
    - Use proptest to generate random `CharacterRegistry` entries with varying `face_portrait` values (Some/None/empty)
    - Assert the portrait dropdown set equals exactly characters with `Some(non-empty)` face_portrait

- [~] 11. Final checkpoint - Full workspace compilation and test suite
  - Ensure all tests pass, ask the user if questions arise.

## Notes

- Tasks marked with `*` are optional and can be skipped for faster MVP
- Each task references specific requirements for traceability
- Checkpoints ensure incremental validation after each crate's changes
- Property tests validate universal correctness properties from the design document
- Unit tests validate specific examples and edge cases
- The order ensures no orphaned code: common crate first, then renderer, then editor cleanup, then new UI features
- The `searchable_combobox.rs` utility already exists and should be reused for the portrait dropdown
- Face portrait commands (`InsertFacePortrait`, `UpdateFacePortrait`, `RemoveFacePortrait`) in `EditCommandKind` should also be removed as part of task 6.3

## Task Dependency Graph

```json
{
  "waves": [
    { "id": 0, "tasks": ["1.1", "1.2", "1.3"] },
    { "id": 1, "tasks": ["3.1", "3.2", "3.3", "3.4", "3.5"] },
    { "id": 2, "tasks": ["4.1", "4.2"] },
    { "id": 3, "tasks": ["6.1", "6.4"] },
    { "id": 4, "tasks": ["6.2", "6.3", "6.5"] },
    { "id": 5, "tasks": ["8.1"] },
    { "id": 6, "tasks": ["8.2", "8.3", "9.1"] },
    { "id": 7, "tasks": ["9.2"] },
    { "id": 8, "tasks": ["10.1", "10.2", "10.3"] }
  ]
}
```
