#  Implementation Plan: Dialog Rendering Polish

## Overview

This plan implements six dialog rendering enhancements: fixed height, border, overflow indicator, attribute dialog mode, inline text markup, and face portrait display. Tasks are ordered data-model-first (common crate changes), then renderer logic (markup parser, UI spawning, systems), then property-based tests, with checkpoints between major phases.

## Tasks

- [x] 1. Extend data models in rpg-toolkit-common
  - [x] 1.1 Add `attribute_dialog` and `face_portrait` fields to `DialogConfigData`
    - Add `#[serde(default)] pub attribute_dialog: bool` to `DialogConfigData` in `crates/rpg-toolkit-common/src/map.rs`
    - Add `#[serde(default)] pub face_portrait: Option<String>` to `DialogConfigData`
    - Update the `Default` impl to include the new fields
    - _Requirements: 4.4, 6.4_

- [x] 2. Extend renderer DialogConfig and conversion logic
  - [x] 2.1 Add new fields to renderer `DialogConfig`
    - Add `pub attribute_dialog: bool` and `pub face_portrait: Option<String>` to `DialogConfig` in `crates/rpg-toolkit-renderer/src/dialog.rs`
    - Update `Default` impl for `DialogConfig`
    - _Requirements: 4.4, 6.4_

  - [x] 2.2 Update `dialog_config_from_data` conversion function
    - Map the new `attribute_dialog` and `face_portrait` fields from `DialogConfigData` to `DialogConfig`
    - _Requirements: 4.4, 6.4_

  - [x] 2.3 Update `ShowDialog` event and public exports in `lib.rs`
    - Ensure `ShowDialog` event carries the updated `DialogConfig` with new fields
    - Export any new component markers (`DialogPanel`, `OverflowIndicator`, `FacePortrait`) from `lib.rs`
    - _Requirements: 4.4, 6.4_

- [x] 3. Implement inline text markup parser
  - [x] 3.1 Create `markup.rs` module with `TextStyle`, `TextSegment`, and `parse_markup`
    - Create `crates/rpg-toolkit-renderer/src/dialog/markup.rs` (or `crates/rpg-toolkit-renderer/src/markup.rs`)
    - Define `TextStyle` enum: `Plain`, `Bold`, `Italic`, `BoldItalic`
    - Define `TextSegment` struct with `text: String` and `style: TextStyle`
    - Implement `parse_markup(input: &str) -> Vec<TextSegment>` as a pure function
    - Parser scans left-to-right, greedily matches longest delimiter first (3 underscores, then 2, then 1)
    - Unclosed delimiters emit remaining text as `Plain`
    - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5, 5.6_

  - [ ]* 3.2 Write property test: markup parse preserves text content (Property 1)
    - **Property 1: Markup parse preserves text content**
    - For any input string, concatenating all segment `text` fields equals the input with valid delimiter underscores removed
    - Add test to `tests/properties/` and register in `tests/properties/Cargo.toml`
    - **Validates: Requirements 5.1, 5.2, 5.3, 5.4, 5.5**

  - [x] 3.3 Write property test: markup style classification correctness (Property 2)
    - **Property 2: Markup style classification correctness**
    - For generated inputs with properly fenced styled spans, `parse_markup` assigns correct styles (Italic for `_`, Bold for `__`, BoldItalic for `___`)
    - Add test to `tests/properties/` and register in `tests/properties/Cargo.toml`
    - **Validates: Requirements 5.1, 5.2, 5.3, 5.4, 5.5**

  - [ ]* 3.4 Write property test: unclosed delimiters produce plain text (Property 3)
    - **Property 3: Unclosed delimiters produce no error and yield plain text**
    - For any input with an opening delimiter not closed before end-of-string, `parse_markup` does not panic and includes unclosed delimiters as `Plain` text
    - Add test to `tests/properties/` and register in `tests/properties/Cargo.toml`
    - **Validates: Requirements 5.6**

- [x] 4. Checkpoint - Ensure data model and parser compile
  - Ensure all tests pass, ask the user if questions arise.

- [x] 5. Refactor `spawn_dialog_ui` for fixed height, border, and overflow
  - [x] 5.1 Add new component markers
    - Add `DialogPanel`, `OverflowIndicator`, and `FacePortrait` component markers to `crates/rpg-toolkit-renderer/src/dialog.rs`
    - _Requirements: 1.1, 2.1, 3.1, 6.1_

  - [x] 5.2 Apply fixed height and overflow clipping to the inner panel
    - Set `height: Val::Px(120.0)` on the inner dialog panel node
    - Set `overflow: Overflow::clip()` to clip text exceeding the panel height
    - Attach `DialogPanel` marker component to the inner panel entity
    - _Requirements: 1.1, 1.2, 1.3_

  - [x] 5.3 Apply visible border to the dialog panel
    - Set `border: UiRect::all(Val::Px(2.0))` on the inner panel node
    - Set `BorderColor` to a visually distinct color (e.g., light gray or white)
    - Ensure border renders on all four sides
    - _Requirements: 2.1, 2.2, 2.3_

  - [x] 5.4 Implement attribute dialog mode (borderless/backgroundless)
    - When `config.attribute_dialog` is true, set `BackgroundColor` to transparent
    - When `config.attribute_dialog` is true, set `border` to zero and `BorderColor` to transparent
    - Preserve text rendering, text speed, and position behavior
    - _Requirements: 4.1, 4.2, 4.3_

  - [x] 5.5 Implement face portrait spawning
    - When `config.face_portrait` is `Some(path)`, spawn an `ImageNode` with the portrait asset
    - Attach `FacePortrait` marker component
    - Use a horizontal flex layout: portrait on the left, text on the right
    - Set portrait to a fixed size that maintains aspect ratio (e.g., 64x64 or percentage-based)
    - When `face_portrait` is `None`, use text-only layout with no reserved portrait space
    - _Requirements: 6.1, 6.2, 6.3, 6.4, 6.5_

  - [x] 5.6 Integrate markup parser into text spawning
    - Call `parse_markup` on the resolved dialog text
    - Spawn `TextSpan` children for each `TextSegment` with appropriate `TextFont` styles (bold weight, italic style, or both)
    - Replace the single `Text::new(...)` with a parent `Text` entity and styled `TextSpan` children
    - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5_

- [x] 6. Implement overflow detection system
  - [x] 6.1 Create `detect_overflow` system
    - Add a new system that runs after `handle_dialog_event`
    - Use a character-count heuristic (estimated chars per line × estimated visible lines based on panel height and font size) to determine if text overflows
    - When overflow is detected, spawn an `OverflowIndicator` entity (e.g., a "▼" text node or similar visual cue) positioned at the bottom-right of the `DialogPanel`
    - When text fits, despawn any existing `OverflowIndicator` entity
    - _Requirements: 3.1, 3.2, 3.3, 3.4_

  - [x] 6.2 Register `detect_overflow` system in the plugin
    - Add `detect_overflow` to the Update schedule in `lib.rs`, ordered after `handle_dialog_event`
    - Export the system from `lib.rs`
    - _Requirements: 3.1_

- [x] 7. Update typewriter system for multi-span text
  - [x] 7.1 Modify `update_dialog_typewriter` to reveal across spans
    - Update the typewriter logic to distribute `chars_revealed` across multiple `TextSpan` children sequentially
    - Each span shows characters up to its length before the next span begins revealing
    - Ensure `handle_dialog_input` instant-reveal also works with multi-span layout
    - _Requirements: 5.1, 5.2, 5.3, 4.3_

- [ ] 8. Checkpoint - Ensure full renderer compiles and existing tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [x] 9. Write unit tests for dialog UI structure
  - [x] 9.1 Test fixed height and border on standard dialog
    - Verify spawned `DialogPanel` node has correct height, border, and overflow values
    - _Requirements: 1.1, 1.2, 1.3, 2.1, 2.2, 2.3_

  - [x] 9.2 Test attribute dialog mode
    - Verify `BackgroundColor` is transparent and border is zero when `attribute_dialog=true`
    - _Requirements: 4.1, 4.2, 4.3_

  - [x] 9.3 Test face portrait spawning
    - Verify `FacePortrait` entity is spawned when `face_portrait` is `Some`
    - Verify no portrait entity when `face_portrait` is `None`
    - _Requirements: 6.1, 6.2, 6.3_

  - [x] 9.4 Test overflow indicator logic
    - Verify indicator appears for text exceeding estimated capacity
    - Verify indicator absent for short text
    - _Requirements: 3.1, 3.2, 3.3, 3.4_

- [x] 10. Add face portrait registry and editor UI integration
  - [x] 10.1 Add `face_portraits` registry to `ProjectFile` in rpg-toolkit-common
    - Add `#[serde(default)] pub face_portraits: HashMap<String, String>` to `ProjectFile` (maps portrait ID → asset path)
    - Update `ProjectFile::new()` to accept the new field
    - _Requirements: 6.4_

  - [x] 10.2 Add `face_portraits` to editor `Project` state
    - Mirror the `face_portraits` field in `crates/rpg-toolkit-editor/src/data/project.rs`
    - Ensure it is loaded/saved during serialization round-trips
    - _Requirements: 6.4_

  - [x] 10.3 Add face portrait management UI to the Dialog Text Panel modal
    - Add a "Face Portraits" section in `render_dialog_text_panel` (similar to the text entries list)
    - Show a list of registered portrait IDs with their asset paths
    - Allow adding new portrait entries (ID + asset path) and removing existing ones
    - Show a small image preview of each portrait using the asset path
    - _Requirements: 6.1, 6.4_

  - [x] 10.4 Add `face_portrait` field to `ActionEditorState` and ShowDialog form
    - Add `pub dialog_face_portrait: Option<String>` to `ActionEditorState`
    - In `render_show_dialog_form`, add a combo box / dropdown that lists portrait IDs from the project's `face_portraits` registry (plus a "None" option)
    - Wire the selected portrait ID into `build_action()` so `DialogConfigData.face_portrait` is set
    - Reset the field on form submission
    - _Requirements: 6.1, 6.4_

  - [x] 10.5 Add EditCommand variants for face portrait CRUD
    - Add `InsertFacePortrait { id, path }`, `UpdateFacePortrait { id, old_path, new_path }`, `RemoveFacePortrait { id, path }` to `EditCommandKind`
    - Implement apply/undo logic in the command handler
    - _Requirements: 6.4_

- [ ] 11. Final checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

## Notes

- Tasks marked with `*` are optional and can be skipped for faster MVP
- Each task references specific requirements for traceability
- The markup parser (task 3.1) is a pure function, making it ideal for property-based testing
- Property tests use `proptest` (already a workspace dependency)
- Checkpoints ensure incremental validation between major phases
- The data model changes (tasks 1-2) are backward-compatible via `#[serde(default)]`
