# Implementation Plan: Editor Dialog Integration

## Overview

Integrate the runtime dialog system into the RPG toolkit editor. Implementation proceeds bottom-up: shared data types in `rpg-toolkit-common` first, then renderer changes (conversion functions, ActionQueue, sequential trigger execution), then editor changes (ProjectFile/Project persistence, EditCommandKind variants, Event Trigger Editor UI, Dialog Text Panel, TextIdIndex, serialization, undo/redo). Property-based tests validate round-trip serialization, text truncation, and reverse index consistency.

## Tasks

- [x] 1. Add dialog data types to rpg-toolkit-common
  - [x] 1.1 Define DialogTextData, DialogConfigData, DialogPositionData in common/map.rs
    - Add `DialogTextData` enum with `Inline(String)` and `Id(String)` variants, `#[serde(tag = "type", content = "value")]`, deriving `Clone, Debug, PartialEq, Eq, Serialize, Deserialize`
    - Add `DialogPositionData` enum with `Top`, `Center`, `Bottom` variants, `#[default] Bottom`, deriving `Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize`
    - Add `DialogConfigData` struct with `text_speed: f32` (serde default 30.0), `position: DialogPositionData` (serde default), `movement_block: bool` (serde default true), deriving `Clone, Debug, PartialEq, Serialize, Deserialize`, with `Default` impl
    - Add `default_text_speed` and `default_movement_block` helper functions for serde defaults
    - _Requirements: 1.1, 1.2, 1.3_

  - [x] 1.2 Add ShowDialog variant to EventAction enum in common/map.rs
    - Add `ShowDialog { text: DialogTextData, config: DialogConfigData }` variant to `EventAction`
    - Change `EventAction` derives from `PartialEq, Eq` to `PartialEq` only (because `DialogConfigData` contains `f32`)
    - Update any code that depends on `EventAction: Eq` (check `TileAttributes` — it may need `Eq` removed too)
    - Update `pub use` exports in `common/src/lib.rs` to include `DialogTextData`, `DialogConfigData`, `DialogPositionData`
    - _Requirements: 1.1, 1.2, 8.1, 8.2, 8.3_

  - [x] 1.3 Add dialog_texts field to ProjectFile in common/project.rs
    - Add `#[serde(default)] pub dialog_texts: HashMap<String, String>` field to `ProjectFile`
    - Update `ProjectFile::new` constructor to accept a `dialog_texts: HashMap<String, String>` parameter
    - Add validation warning for ShowDialog actions referencing non-existent maps (similar to existing JumpTo warning)
    - _Requirements: 7.1, 7.4, 8.3_

- [x] 2. Checkpoint — Ensure common crate compiles
  - Ensure `cargo check -p rpg-toolkit-common` passes. Fix any compile errors from the `Eq` removal or new types. Ask the user if questions arise.

- [x] 3. Add conversion functions and ActionQueue to the renderer
  - [x] 3.1 Add conversion functions in renderer for common ↔ renderer dialog types
    - Create `dialog_text_from_data(data: &DialogTextData) -> DialogText` and `dialog_config_from_data(data: &DialogConfigData) -> DialogConfig` functions in `crates/rpg-toolkit-renderer/src/dialog.rs`
    - Map `DialogTextData::Inline`/`Id` to `DialogText::Inline`/`Id`, and `DialogPositionData` variants to `DialogPosition` variants
    - _Requirements: 1.4_

  - [x] 3.2 Define ActionQueue resource in renderer/resources.rs
    - Add `ActionQueue` resource with `actions: VecDeque<EventAction>` and `waiting_for_dialog: bool` fields
    - Import `VecDeque` from `std::collections` and `EventAction` from `rpg_toolkit_common`
    - _Requirements: 4.8_

  - [x] 3.3 Implement advance_action_queue system in renderer/systems/triggers.rs
    - Add `advance_action_queue` system that peeks the next action in `ActionQueue`
    - For `ShowDialog`: convert `DialogTextData`/`DialogConfigData` to renderer types using conversion functions, fire `ShowDialog` event, set `waiting_for_dialog = true`; if `DialogState` still exists while waiting, return early; when `DialogState` is removed, pop the completed action
    - For `JumpTo`: set `pending_map_change` on `RendererState`, clear the queue, remove `ActionQueue` resource
    - If queue is empty after processing, remove `ActionQueue` resource via `commands.remove_resource::<ActionQueue>()`
    - Handle missing `DialogTextRegistry` for `Id` lookups: log warning, skip action
    - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5, 4.6, 4.9_

  - [x] 3.4 Modify check_triggers to populate ActionQueue instead of executing first JumpTo
    - Refactor `check_triggers` to collect all `EventAction` entries from all layers at the destination tile into a `VecDeque`
    - If the queue is non-empty and no `ActionQueue` resource currently exists, insert the `ActionQueue` resource
    - If an `ActionQueue` already exists, ignore the new trigger (sequence in progress)
    - Remove the existing `return` after first JumpTo — all actions are now queued
    - _Requirements: 4.1, 4.7, 4.8, 4.9_

  - [x] 3.5 Register ActionQueue and advance_action_queue in ProjectRendererPlugin
    - Add `advance_action_queue` system to the `Update` schedule, ordered after `check_triggers` and before `handle_dialog_event`
    - Update system ordering: `check_triggers` → `advance_action_queue` → `handle_map_change`
    - Update `pub use` exports in `renderer/src/lib.rs` for new public types and functions
    - _Requirements: 4.1, 4.8_

- [x] 4. Checkpoint — Ensure renderer crate compiles and existing tests pass
  - Ensure `cargo check -p rpg-toolkit-renderer` passes and `cargo test -p rpg-toolkit-properties` passes. Ask the user if questions arise.

- [x] 5. Add dialog_texts to editor Project and EditCommandKind variants
  - [x] 5.1 Add dialog_texts field to editor Project resource
    - Add `pub dialog_texts: HashMap<String, String>` field to `Project` struct in `crates/rpg-toolkit-editor/src/data/project.rs`
    - _Requirements: 7.2, 7.3_

  - [x] 5.2 Add InsertDialogText, UpdateDialogText, RemoveDialogText variants to EditCommandKind
    - Add `InsertDialogText { text_id: String, text: String }` variant
    - Add `UpdateDialogText { text_id: String, old_text: String, new_text: String }` variant
    - Add `RemoveDialogText { text_id: String, old_text: String }` variant
    - These are no-ops on `MapData` in `apply`/`apply_inverse` (handled at Project level like `SetSpawnPoint`)
    - _Requirements: 5.9_

  - [x] 5.3 Handle dialog text EditCommand variants in undo_redo plugin
    - In `consume_edit_commands` in `undo_redo.rs`, handle `InsertDialogText`, `UpdateDialogText`, `RemoveDialogText` at the Project level (similar to `SetSpawnPoint`)
    - `InsertDialogText`: insert into `project.dialog_texts`
    - `UpdateDialogText`: update the value in `project.dialog_texts`
    - `RemoveDialogText`: remove from `project.dialog_texts`
    - In `undo_redo_keyboard`, handle undo/redo for these variants by applying the inverse at the Project level
    - _Requirements: 5.9_

- [x] 6. Update serialization for dialog_texts persistence
  - [x] 6.1 Update save_project_to_path to include dialog_texts
    - Pass `project.dialog_texts.clone()` when constructing `ProjectFile` in `save_project_to_path`
    - _Requirements: 7.2_

  - [x] 6.2 Update load_project_with_dialog to populate dialog_texts
    - Set `dialog_texts: project_file.dialog_texts` when constructing the `Project` resource in `load_project_with_dialog`
    - _Requirements: 7.3, 7.4_

- [x] 7. Checkpoint — Ensure editor crate compiles
  - Ensure `cargo check -p rpg-toolkit-editor` passes. Ask the user if questions arise.

- [x] 8. Update Event Trigger Editor UI for ShowDialog configuration
  - [x] 8.1 Add ActionType and DialogTextMode enums and ShowDialog fields to EventTriggerDialog
    - Define `ActionType` enum (`JumpTo`, `ShowDialog`) with `Default` (JumpTo) and `PartialEq`
    - Define `DialogTextMode` enum (`Inline`, `TextId`) with `Default` (Inline) and `PartialEq`
    - Add fields to `EventTriggerDialog`: `new_action_type`, `new_dialog_text_mode`, `new_dialog_inline_text`, `new_dialog_text_id`, `new_dialog_text_speed` (String, default "30"), `new_dialog_position` (DialogPositionData), `new_dialog_movement_block` (bool, default true)
    - _Requirements: 2.1, 2.2, 2.5_

  - [x] 8.2 Add truncate_preview pure function
    - Implement `truncate_preview(s: &str, max_len: usize) -> String` that truncates to `max_len` characters and appends "…" if truncated, otherwise returns the original string
    - Place in `attribute.rs` or a shared utility location
    - _Requirements: 3.2_

  - [x] 8.3 Update event_trigger_panel_ui to display ShowDialog actions and add ShowDialog creation form
    - Display existing ShowDialog actions in the action list with type label "ShowDialog", showing inline text preview (truncated to 40 chars) or "ID: {text_id}"
    - Add action type selector (dropdown or radio: "JumpTo" / "ShowDialog") above the existing "Add JumpTo" form
    - When "ShowDialog" is selected, show: text source toggle (Inline/Text ID), inline multi-line text input or Text ID single-line input, Text Speed numeric input (default 30), Position dropdown (Top/Center/Bottom, default Bottom), Movement Block checkbox (default true)
    - "Add ShowDialog" button creates a `ShowDialog` EventAction and appends to `dialog.actions`
    - Existing JumpTo form remains when "JumpTo" action type is selected
    - ShowDialog actions use the same remove (✕) and reorder (▲/▼) controls as JumpTo
    - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 2.7, 2.8, 3.1, 3.2, 3.3, 3.4, 3.5_

- [x] 9. Implement Dialog Text Panel plugin
  - [x] 9.1 Create dialog_text_panel.rs with DialogTextPanelPlugin, DialogTextPanelState, and TextIdUsage types
    - Create `crates/rpg-toolkit-editor/src/plugins/dialog_text_panel.rs`
    - Define `DialogTextPanelPlugin` struct implementing `Plugin`
    - Define `DialogTextPanelState` resource with fields: `new_text_id`, `new_text_content`, `selected_text_id`, `editing_text_id`, `edit_buffer`
    - Define `TextIdUsage` struct with `map_id`, `map_name`, `layer_index`, `x`, `y` fields
    - Register the plugin module in `plugins/mod.rs`
    - _Requirements: 5.1, 5.2_

  - [x] 9.2 Implement dialog_text_panel_ui system with CRUD operations
    - Render the Dialog Text Panel inside the existing left `SidePanel` (add to `layer_panel_ui` or as a separate system that renders in the same panel), below the Map Browser section
    - Display a collapsible "Dialog Texts" heading with a scrollable list of all entries showing Text_Id and truncated preview
    - Provide a creation form with Text_Id (single-line) and text content (multi-line) inputs, and an "Add" button
    - Disable "Add" button when Text_Id or text content is empty
    - Show warning when Text_Id already exists (do not overwrite)
    - Provide edit action (pencil icon or "Edit" button) that switches entry to edit mode with a text area
    - Provide delete action (✕ button) that removes the entry
    - Emit `InsertDialogText`, `UpdateDialogText`, `RemoveDialogText` EditCommands for each operation
    - Panel is always visible regardless of editor mode
    - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5, 5.6, 5.7, 5.8, 5.9_

  - [x] 9.3 Register DialogTextPanelPlugin in editor main.rs
    - Add `.add_plugins(DialogTextPanelPlugin)` to the editor app builder in `main.rs`
    - _Requirements: 5.1_

- [x] 10. Implement TextIdIndex reverse index
  - [x] 10.1 Define TextIdIndex resource and rebuild_text_id_index function
    - Add `TextIdIndex` resource (with `HashMap<String, Vec<TextIdUsage>>`) and `get(&self, text_id: &str) -> &[TextIdUsage]` method to `dialog_text_panel.rs`
    - Implement `rebuild_text_id_index(maps: &HashMap<MapId, MapData>) -> TextIdIndex` pure function that scans all maps/layers/tiles for `ShowDialog` actions with `DialogTextData::Id` references
    - _Requirements: 6.1, 6.4_

  - [x] 10.2 Implement update_text_id_index_for_tile incremental update function
    - Implement `update_text_id_index_for_tile(index, map_id, map_name, layer_index, x, y, old_triggers, new_triggers)` that removes old entries for the tile and adds new entries based on new triggers
    - _Requirements: 6.1, 6.4_

  - [x] 10.3 Integrate TextIdIndex into project load and edit command flow
    - Call `rebuild_text_id_index` in `load_project_with_dialog` after loading the project, insert as a resource
    - Call `update_text_id_index_for_tile` in `consume_edit_commands` (undo_redo.rs) when a `SetEventTrigger` command is processed (both apply and undo paths)
    - Initialize `TextIdIndex` as a default resource in the editor app
    - _Requirements: 6.1, 6.4_

  - [x] 10.4 Add find-usages display to Dialog Text Panel
    - When a dialog text entry is selected in the panel, display the list of usages from `TextIdIndex::get(selected_text_id)`
    - Show each usage with map name, layer index, and tile coordinates
    - When the user clicks a usage, navigate to that map (open tab if needed) and select the tile
    - Display "No usages found" when the list is empty
    - _Requirements: 6.1, 6.2, 6.3, 6.4, 6.5_

- [x] 11. Ensure attribute overlay covers ShowDialog triggers
  - [x] 11.1 Verify attribute_overlay_system handles ShowDialog actions
    - The existing `attribute_overlay_system` checks `!attrs.event_trigger.is_empty()` which already covers ShowDialog actions — verify this works correctly and no changes are needed
    - If the overlay logic filters by action type, update it to include ShowDialog
    - _Requirements: 9.1, 9.2_

- [x] 12. Checkpoint — Ensure full project compiles and all existing tests pass
  - Ensure `cargo check --workspace` and `cargo test -p rpg-toolkit-properties` pass. Ask the user if questions arise.

- [ ] 13. Property-based tests for correctness properties
  - [ ]* 13.1 Write property test for EventAction list round-trip (Property 1)
    - **Property 1: EventAction list round-trip**
    - Generate `Vec<EventAction>` with 0–10 actions mixing `JumpTo` (random map IDs, coords) and `ShowDialog` (random Inline/Id text, random config with text_speed 0.0–500.0, 3 position variants, bool movement_block)
    - Serialize to JSON with `serde_json::to_string`, deserialize with `serde_json::from_str`, assert equality
    - Add test file `tests/properties/event_action_round_trip.rs` and register in `tests/properties/Cargo.toml`
    - **Validates: Requirements 1.3, 8.4**

  - [ ]* 13.2 Write property test for text preview truncation (Property 2)
    - **Property 2: Text preview truncation**
    - Generate random strings of length 0–200 (including Unicode)
    - Assert: if original length > 40, result is first 40 chars + "…" and result length ≤ 41; otherwise result equals original
    - Add test to `tests/properties/event_action_round_trip.rs` or a dedicated file
    - **Validates: Requirements 3.2**

  - [ ]* 13.3 Write property test for reverse index consistency (Property 3)
    - **Property 3: Reverse index consistency with rebuild**
    - Generate 1–5 maps with 1–4 layers, 1×1 to 4×4 tiles, random event_trigger lists with ShowDialog(Id), ShowDialog(Inline), and JumpTo actions
    - Verify `rebuild_text_id_index` produces correct index (no false positives/negatives)
    - Verify single-tile `update_text_id_index_for_tile` on a correct index produces the same result as `rebuild_text_id_index` on the modified project
    - Add test file `tests/properties/text_id_index.rs` and register in `tests/properties/Cargo.toml`
    - **Validates: Requirements 6.1, 6.4**

  - [ ]* 13.4 Write property test for ProjectFile dialog text round-trip (Property 4)
    - **Property 4: ProjectFile dialog text round-trip**
    - Generate `ProjectFile` with 0–3 maps, 0–10 dialog_texts entries (keys: `[a-z_]{1,20}`, values: `[a-zA-Z0-9 ]{1,100}`)
    - Serialize with `ProjectFile::serialize`, deserialize with `ProjectFile::deserialize`, assert `dialog_texts` field is identical
    - Add test to `tests/properties/project_round_trip.rs` or a dedicated file
    - **Validates: Requirements 7.5**

- [ ] 14. Final checkpoint — Ensure all tests pass
  - Ensure `cargo test --workspace` passes. Ask the user if questions arise.

## Notes

- Tasks marked with `*` are optional and can be skipped for faster MVP
- Each task references specific requirements for traceability
- Checkpoints ensure incremental validation
- Property tests validate universal correctness properties from the design document (Properties 1–4)
- The `Eq` removal from `EventAction` (due to `f32` in `DialogConfigData`) may cascade to `TileAttributes` and other types — task 1.2 handles this
- `DialogTextPanelPlugin` is a new plugin registered alongside existing plugins; it renders in the same left `SidePanel` as the layer panel
- `TextIdIndex` is runtime-only derived data — not persisted, rebuilt on load, incrementally updated on edits
- All dialog text EditCommand variants operate at the Project level (like `SetSpawnPoint`), not on `MapData`
- The `tests/properties/` crate needs `rpg-toolkit-common` as a dev-dependency (already present) for property tests on common types
