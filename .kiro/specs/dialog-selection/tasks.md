# Implementation Plan: Dialog Selection

## Overview

This plan implements a JRPG-style dialog selection system across three crates: `rpg-toolkit-common` (data model + validation), `rpg-toolkit-renderer` (runtime UI, input, and action queue integration), and `rpg-toolkit-editor` (editor form for authoring selections). The implementation proceeds data-model-first, then runtime systems, then editor integration, with property tests validating correctness at each layer.

## Tasks

- [x] 1. Define the ShowSelection data model in rpg-toolkit-common
  - [x] 1.1 Add ChoiceData struct and ShowSelection variant to EventAction enum
    - Add `ChoiceData` struct with `label: DialogTextData` and `actions: Vec<EventAction>` fields to `crates/rpg-toolkit-common/src/map.rs`
    - Add `ShowSelection { prompt: DialogTextData, config: DialogConfigData, choices: Vec<ChoiceData> }` variant to the `EventAction` enum
    - Implement validation via `#[serde(try_from)]` on a helper struct: choices count must be 2–6, each choice's actions list must be ≤ 20, inline labels must be 1–80 characters
    - Derive `Clone, Debug, PartialEq, Serialize, Deserialize` on `ChoiceData`
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 1.7_

  - [x] 1.2 Write property test for serialization round-trip (Property 1)
    - **Property 1: Serialization Round-Trip**
    - Use `proptest` to generate valid `ShowSelection` actions with nested `EventAction` trees up to 3 levels deep
    - Verify `serde_json::to_string` → `serde_json::from_str` produces a `PartialEq`-equal value
    - **Validates: Requirements 1.6, 8.1, 8.2**

  - [ ]* 1.3 Write property test for type tag presence (Property 2)
    - **Property 2: Type Tag Presence**
    - Generate random valid `ShowSelection` instances and verify the serialized JSON contains `"type":"ShowSelection"` at the top level
    - **Validates: Requirements 1.2, 8.3**

  - [ ]* 1.4 Write property test for choice count validation (Property 3)
    - **Property 3: Choice Count Validation**
    - Generate choice arrays of length 0–10 and verify deserialization rejects counts outside 2–6
    - Verify missing `prompt` or `choices` fields also produce deserialization errors
    - **Validates: Requirements 1.4, 8.4**

  - [ ]* 1.5 Write property test for label length validation (Property 4)
    - **Property 4: Label Length Validation**
    - Generate inline label strings of length 0–200 and verify validation rejects empty strings and strings > 80 characters
    - **Validates: Requirements 1.7**

- [x] 2. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [x] 3. Implement runtime selection state and action queue integration
  - [x] 3.1 Add WaitingFor::Selection variant and SelectionState resource
    - Add `Selection` variant to the `WaitingFor` enum in `crates/rpg-toolkit-renderer/src/resources.rs`
    - Define `SelectionState` resource struct with `cursor_index: usize`, `choice_count: usize`, and `choices: Vec<ResolvedChoice>` in a new file `crates/rpg-toolkit-renderer/src/systems/selection.rs`
    - Define `ResolvedChoice` struct with `label: String` and `actions: Vec<EventAction>`
    - _Requirements: 5.1_

  - [x] 3.2 Extend advance_action_queue to handle ShowSelection and WaitingFor::Selection
    - In `crates/rpg-toolkit-renderer/src/systems/triggers.rs`, add a `WaitingFor::Selection` match arm that checks for `SelectionState` resource presence (mirrors `WaitingFor::Dialog`)
    - Add an `EventAction::ShowSelection` match arm that resolves prompt and label text from the `DialogTextRegistry`, handles missing-ID gracefully (warn + skip), spawns the selection UI, inserts `SelectionState`, and sets `waiting_for = WaitingFor::Selection`
    - _Requirements: 5.1, 5.2, 5.3, 6.1, 6.2, 6.3, 6.4, 6.5_

  - [ ]* 3.3 Write property test for queue blocking during selection (Property 7)
    - **Property 7: Queue Blocking During Selection**
    - Generate action queues with `WaitingFor::Selection` while `SelectionState` is present, verify `advance_action_queue` does not pop actions or change waiting state
    - **Validates: Requirements 5.2**

- [x] 4. Implement selection UI rendering
  - [x] 4.1 Spawn selection prompt UI entities
    - Define UI marker components: `SelectionBox`, `SelectionCursor`, `SelectionLabel { index: usize }` in `crates/rpg-toolkit-renderer/src/systems/selection.rs`
    - Implement `spawn_selection_ui` function that creates the selection prompt entity hierarchy: root container (position from `DialogConfigData`), semi-transparent panel (same styling as standard dialog: 80% width, border, overflow clip, auto-height), prompt text node, face portrait (if configured), vertical choice list with "▶" cursor at index 0
    - Reuse the same positioning and styling logic from `dialog.rs` for consistency
    - _Requirements: 2.1, 2.3, 2.4, 2.5, 2.6, 2.7_

  - [x] 4.2 Implement selection prompt idempotency (ignore if already active)
    - In the `ShowSelection` handling within `advance_action_queue` (or a dedicated `handle_selection_event` system), check if `SelectionState` already exists; if so, skip spawning new UI
    - _Requirements: 2.2_

  - [ ]* 4.3 Write property test for selection prompt idempotency (Property 6)
    - **Property 6: Selection Prompt Idempotency**
    - Generate two `ShowSelection` payloads; verify the second is ignored when `SelectionState` is already present
    - **Validates: Requirements 2.2**

- [x] 5. Implement selection navigation and confirmation input
  - [x] 5.1 Implement cursor navigation with wrapping
    - Create a `handle_selection_input` system in `crates/rpg-toolkit-renderer/src/systems/selection.rs`
    - Read Up/Down key presses (ArrowUp, KeyW, ArrowDown, KeyS) using `just_pressed` for discrete navigation
    - Update `SelectionState.cursor_index` using `(index + delta).rem_euclid(count)` for wrapping
    - Update the `SelectionCursor` entity position to reflect the new cursor index
    - Block player movement while `SelectionState` is present (consume direction inputs)
    - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5_

  - [x] 5.2 Implement selection confirmation (Space/Enter)
    - In `handle_selection_input`, detect Space/Enter `just_pressed`
    - On confirm: remove `SelectionState` resource, despawn all `SelectionBox` entities, pop the `ShowSelection` action from the front of the queue, insert the committed choice's `actions` at the front of the `ActionQueue`, and clear `waiting_for` to `Nothing`
    - _Requirements: 4.1, 4.2, 4.3, 4.4_

  - [ ]* 5.3 Write property test for cursor navigation wrapping (Property 5)
    - **Property 5: Cursor Navigation Wrapping**
    - Generate `(choice_count, cursor_index, direction)` tuples with `choice_count` in 2–6 and `cursor_index` in 0..choice_count
    - Verify Up → `(cursor_index - 1 + choice_count) % choice_count` and Down → `(cursor_index + 1) % choice_count`
    - **Validates: Requirements 3.1, 3.2, 3.3, 3.4**

  - [ ]* 5.4 Write property test for confirmation branch injection (Property 8)
    - **Property 8: Confirmation Injects Correct Branch**
    - Generate `SelectionState` with random `cursor_index` and choices, verify confirming results in `choices[cursor_index].actions` at the front of `ActionQueue` and `SelectionState` removed
    - **Validates: Requirements 4.1, 4.3**

- [x] 6. Implement movement and interaction blocking during selection
  - [x] 6.1 Block player movement and NPC patrols during active selection
    - Add `SelectionState` resource check to the player movement system (similar to existing `DialogState` check) to block movement input
    - Verify NPC patrol movement already blocks when `ActionQueue` is present (no additional change needed, but confirm behavior)
    - Suppress interaction input (ignore Space/Enter for NPC interaction) while `SelectionState` is active
    - _Requirements: 3.5, 5.4, 5.5_

- [x] 7. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [x] 8. Implement editor integration for ShowSelection
  - [x] 8.1 Add ShowSelection variant to ActionType enum and ActionEditorState
    - Add `ShowSelection` to the `ActionType` enum in `crates/rpg-toolkit-editor/src/plugins/attribute/action_editor.rs`
    - Add selection-specific fields to `ActionEditorState`: `selection_prompt_mode: DialogTextMode`, `selection_prompt_text: String`, `selection_prompt_id: String`, `selection_position: DialogPositionData`, `selection_face_portrait: Option<String>`, `selection_choices: Vec<EditorChoice>`
    - Define `EditorChoice` struct with `label_mode: DialogTextMode`, `label_text: String`, `label_id: String`, `actions: Vec<EventAction>`
    - Implement `load_from_action` for the `ShowSelection` variant
    - Implement `build_action` for the `ShowSelection` variant with validation: at least 2 choices, each with non-empty label
    - _Requirements: 7.1, 7.5_

  - [x] 8.2 Implement render_show_selection_form in action_editor_forms.rs
    - Add `render_show_selection_form` function in `crates/rpg-toolkit-editor/src/plugins/attribute/action_editor_forms.rs`
    - Render prompt text input (inline text or registry ID toggle)
    - Render position combo box and face portrait selector
    - Render choice list with Add Choice (max 6) and Remove (disabled at 2) buttons
    - Render per-choice: label input + nested action editor (reuse recursive pattern from Branch)
    - Display inline validation errors for empty labels and insufficient choices
    - _Requirements: 7.2, 7.3, 7.4, 7.5_

  - [x] 8.3 Wire ShowSelection form into action_editor_ui dispatch
    - Add "Show Selection" to the action type combo box in `action_editor_ui.rs` (alphabetically positioned)
    - Add dispatch to `render_show_selection_form` when `ActionType::ShowSelection` is selected
    - _Requirements: 7.1_

  - [ ]* 8.4 Write property test for editor build_action validation (Property 9)
    - **Property 9: Editor Build Action Validation**
    - Generate `ActionEditorState` instances configured for `ShowSelection` with varying choice counts (0–8) and label states (empty, valid, too long)
    - Verify `build_action()` returns `None` for fewer than 2 choices or any empty inline label
    - **Validates: Requirements 7.5**

- [x] 9. Final checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

## Notes

- Tasks marked with `*` are optional and can be skipped for faster MVP
- Each task references specific requirements for traceability
- Checkpoints ensure incremental validation
- Property tests validate universal correctness properties from the design document using the `proptest` crate
- Unit tests validate specific examples and edge cases
- The implementation follows the existing patterns in the codebase (e.g., `ShowDialog`, `Branch`, `StateCheck`) for consistency
- `SelectionState` mirrors the `DialogState` resource lifecycle pattern
- `WaitingFor::Selection` mirrors `WaitingFor::Dialog` in the action queue

## Task Dependency Graph

```json
{
  "waves": [
    { "id": 0, "tasks": ["1.1"] },
    { "id": 1, "tasks": ["1.2", "1.3", "1.4", "1.5", "3.1"] },
    { "id": 2, "tasks": ["3.2"] },
    { "id": 3, "tasks": ["3.3", "4.1"] },
    { "id": 4, "tasks": ["4.2", "5.1"] },
    { "id": 5, "tasks": ["4.3", "5.2", "6.1"] },
    { "id": 6, "tasks": ["5.3", "5.4"] },
    { "id": 7, "tasks": ["8.1"] },
    { "id": 8, "tasks": ["8.2", "8.3"] },
    { "id": 9, "tasks": ["8.4"] }
  ]
}
```
