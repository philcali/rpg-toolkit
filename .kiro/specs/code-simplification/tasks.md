Implementation Plan: Code Simplification

## Overview

This plan decomposes the `attribute.rs` monolith (2546 lines) into focused sub-modules, splits `editor_state.rs` by concern, introduces `SystemParam` bundles, and consolidates duplicated dialog initialization code. Each task is ordered to minimize compilation breakage — we start with additive changes (new files, new types) before modifying existing code to reference them.

## Tasks

- [x] 1. Split editor_state.rs into state.rs, commands.rs, and undo.rs
  - [x] 1.1 Create `data/state.rs` with EditorState, EditorTool, EditorMode, AttributeTool, StampBrushSelection, LineDragState, EditorError, and AnyDialogOpen
    - Move all state-related types and their impls from `editor_state.rs`
    - Add a module-level `//!` doc comment explaining this module holds editor state resources and enums
    - _Requirements: 4.1, 8.1, 8.2_
  - [x] 1.2 Create `data/commands.rs` with EditCommand, EditCommandKind, and their apply/apply_inverse implementations
    - Move command types and impl blocks from `editor_state.rs`
    - Add a module-level `//!` doc comment explaining this module defines reversible edit commands
    - _Requirements: 4.2, 8.1, 8.2_
  - [x] 1.3 Create `data/undo.rs` with UndoHistory and its implementation
    - Move UndoHistory struct and impl from `editor_state.rs`
    - Add a module-level `//!` doc comment explaining this module manages undo/redo history
    - _Requirements: 4.3, 8.1, 8.2_
  - [x] 1.4 Update `data/mod.rs` to declare new sub-modules and maintain identical public re-exports
    - Replace `pub mod editor_state` with `pub mod state`, `pub mod commands`, `pub mod undo`
    - Update all `pub use` statements to point to new module paths
    - Ensure the public API surface is unchanged (same types exported from `crate::data`)
    - _Requirements: 4.4_
  - [x] 1.5 Remove the old `data/editor_state.rs` file
    - Delete the file after confirming all contents have been moved
    - _Requirements: 4.1, 4.2, 4.3_

- [x] 2. Checkpoint — Verify editor_state split compiles
  - Run `cargo build --workspace` and ensure no errors or new warnings
  - Ensure all tests pass, ask the user if questions arise.

- [x] 3. Create the attribute plugin directory structure and shared action editor
  - [x] 3.1 Create `plugins/attribute/` directory and `plugins/attribute/action_editor.rs`
    - Define the `ActionEditorState` struct with all shared action-editing fields (action_type, editing_index, target coords, dialog fields, shake fields, fade fields, state fields, appearance fields)
    - Implement `ActionEditorState::reset()`, `load_from_action()`, and `build_action()` methods
    - Define the `render_action_editor()` function that renders the shared action editing UI
    - Move `ActionType`, `DialogTextMode`, and `truncate_preview` into this module
    - Add a module-level `//!` doc comment
    - _Requirements: 2.1, 2.3, 3.1, 3.2, 8.1, 8.2_
  - [x] 3.2 Create `plugins/attribute/overlay.rs`
    - Move `attribute_overlay_system` from `attribute.rs`
    - Add a module-level `//!` doc comment explaining gizmo overlay rendering for attribute mode
    - _Requirements: 1.2, 1.3, 8.1, 8.2_
  - [x] 3.3 Create `plugins/attribute/click.rs` with AttributeClickParams SystemParam
    - Move `attribute_click_system` from `attribute.rs`
    - Define `AttributeClickParams` SystemParam struct bundling all system parameters (mouse, editor_state, cursor_state, project, edit_events, event_trigger_dialog, spawn_confirm_dialog, npc_placement_dialog, any_dialog_open)
    - Refactor `attribute_click_system` to accept `AttributeClickParams` instead of individual params
    - Add a module-level `//!` doc comment explaining click dispatch for attribute mode
    - _Requirements: 1.2, 5.1, 5.2, 5.3, 8.1, 8.2_
  - [x] 3.4 Create `plugins/attribute/event_trigger_dialog.rs`
    - Move `EventTriggerDialog` resource and `event_trigger_panel_ui` system from `attribute.rs`
    - Replace the 15+ individual action-editing fields with an embedded `ActionEditorState`
    - Update `event_trigger_panel_ui` to call `render_action_editor()` for the action editing UI
    - Add a module-level `//!` doc comment
    - _Requirements: 1.2, 2.3, 3.1, 3.3, 8.1, 8.2_
  - [x] 3.5 Create `plugins/attribute/spawn_point_dialog.rs`
    - Move `SpawnPointConfirmDialog` resource and `spawn_point_confirm_ui` system from `attribute.rs`
    - Add a module-level `//!` doc comment
    - _Requirements: 1.2, 8.1, 8.2_
  - [x] 3.6 Create `plugins/attribute/npc_dialog.rs` with reset/open_new/open_edit methods
    - Move `NpcPlacementDialog` resource and `npc_placement_dialog_ui` system from `attribute.rs`
    - Replace the 15+ individual action-editing fields with an embedded `ActionEditorState`
    - Implement `NpcPlacementDialog::reset()` method that resets all fields to defaults
    - Implement `NpcPlacementDialog::open_new(tile_x, tile_y, default_spritesheet)` method
    - Implement `NpcPlacementDialog::open_edit(index, npc)` method that populates from existing NPC
    - Update `attribute_click_system` in `click.rs` to use `open_new`/`open_edit` instead of field-by-field initialization
    - Update `npc_placement_dialog_ui` to call `render_action_editor()` for action editing UI
    - Add a module-level `//!` doc comment
    - _Requirements: 1.2, 2.3, 3.1, 3.3, 6.1, 6.2, 6.3, 6.4, 8.1, 8.2_
  - [x] 3.7 Create `plugins/attribute/mod.rs` as the plugin entry point
    - Declare all sub-modules (action_editor, click, event_trigger_dialog, npc_dialog, overlay, spawn_point_dialog)
    - Define `AttributePlugin` struct and implement `Plugin` with identical resource/system registration
    - Re-export public types (AttributePlugin, EventTriggerDialog, NpcPlacementDialog, SpawnPointConfirmDialog, ActionEditorState)
    - Add a module-level `//!` doc comment
    - _Requirements: 1.1, 1.2, 1.4, 8.1, 8.2_

- [x] 4. Remove old attribute.rs and update plugins/mod.rs
  - [x] 4.1 Delete `plugins/attribute.rs`
    - Remove the monolithic file after all contents have been moved to sub-modules
    - _Requirements: 1.2_
  - [x] 4.2 Update `plugins/mod.rs` to reference the new attribute directory module
    - The `pub mod attribute;` declaration should now resolve to `attribute/mod.rs` automatically
    - Verify all re-exports from `plugins/mod.rs` still work (AttributePlugin)
    - _Requirements: 1.1, 1.4_

- [x] 5. Checkpoint — Verify attribute decomposition compiles
  - Run `cargo build --workspace` and ensure no errors or new warnings
  - Verify no individual attribute sub-module exceeds 500 lines
  - Ensure all tests pass, ask the user if questions arise.

- [x] 6. Introduce PaintingParams SystemParam bundle in painting.rs
  - [x] 6.1 Define `PaintingParams` SystemParam struct in `plugins/painting.rs`
    - Bundle mouse, keys, cursor_state, project, editor_state, tool, edit_events, and any_dialog_open into a single `PaintingParams` struct using `#[derive(SystemParam)]`
    - Name it descriptively per requirement 5.3
    - _Requirements: 5.1, 5.2, 5.3_
  - [x] 6.2 Refactor `painting_system` to accept `PaintingParams` instead of individual parameters
    - Update all field accesses to go through the params struct
    - Remove the `#[allow(clippy::too_many_arguments)]` annotation
    - _Requirements: 5.1, 5.2_

- [x] 7. Checkpoint — Verify SystemParam refactoring compiles
  - Run `cargo build --workspace` and ensure no errors or new warnings
  - Ensure all tests pass, ask the user if questions arise.

- [x] 8. Final verification
  - [x] 8.1 Run full workspace build
    - Execute `cargo build --workspace` and confirm zero errors and no new warnings
    - _Requirements: 7.1_
  - [x] 8.2 Run all existing tests
    - Execute `cargo test --workspace` to verify all property-based tests and unit tests pass
    - _Requirements: 7.2, 7.3_
  - [x] 8.3 Verify module size constraints
    - Confirm no attribute sub-module exceeds 500 lines (requirement 1.3)
    - _Requirements: 1.3_

## Notes

- Tasks are ordered to minimize compilation breakage: additive changes first (new files), then wiring, then removal of old files
- The `data/` split (task 1) is independent of the `attribute/` split (task 3) and is done first because it's simpler
- Each checkpoint verifies the codebase compiles before proceeding to the next phase
- The `ActionEditorState` struct and `render_action_editor` function are created before the dialog modules that depend on them
- The `NpcPlacementDialog::open_new`/`open_edit` methods eliminate the two ~30-line duplicated initialization blocks in `attribute_click_system`
- All re-exports are maintained so that downstream code (`use crate::data::EditorState`) continues to work unchanged
