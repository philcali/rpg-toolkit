# Implementation Plan: Event Branching

## Overview

This plan implements compound branching conditions, a new `Branch` action variant, and sequence-level conditional triggers on tiles and NPCs. The implementation builds incrementally: data model types first, then runtime evaluation, then renderer integration, then editor UI, and finally property-based tests.

## Tasks

- [x] 1. Define condition data model types in `rpg-toolkit-common`
  - [x] 1.1 Create `crates/rpg-toolkit-common/src/condition.rs` with `ConditionOperator`, `ConditionCheck`, `ConditionLogic`, and `BranchCondition` structs
    - Define `ConditionOperator` enum with `Equals`, `NotEquals`, `Exists`, `NotExists` variants, deriving `Clone`, `Copy`, `Debug`, `PartialEq`, `Eq`, `Serialize`, `Deserialize`
    - Define `ConditionCheck` struct with `key: String`, `operator: ConditionOperator`, `value: Option<String>` (with `#[serde(default)]` on value)
    - Define `ConditionLogic` enum with `All` (default) and `Any` variants
    - Define `BranchCondition` struct with `logic: ConditionLogic` (`#[serde(default)]`) and `checks: Vec<ConditionCheck>` (`#[serde(default)]`)
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.9_

  - [x] 1.2 Implement `evaluate` methods on `ConditionCheck` and `BranchCondition`
    - `ConditionCheck::evaluate(&self, flags: &HashMap<String, String>) -> bool` implementing operator semantics: Equals checks `flags[key] == value`, NotEquals checks key absent or value differs, Exists checks key present, NotExists checks key absent
    - Handle edge case: `Equals` with `value: None` returns false, `NotEquals` with `value: None` returns true
    - `BranchCondition::evaluate(&self, flags: &HashMap<String, String>) -> bool` implementing: `All` → all checks true (empty = true), `Any` → any check true (empty = true)
    - _Requirements: 1.5, 1.6, 1.7, 1.8, 2.1, 2.2, 2.3, 2.4, 2.5_

  - [x] 1.3 Define `ConditionalTrigger` struct in `condition.rs`
    - Define struct with `condition: BranchCondition` and `actions: Vec<crate::map::EventAction>`
    - Derive `Clone`, `Debug`, `PartialEq`, `Serialize`, `Deserialize`
    - _Requirements: 4.2_

  - [x] 1.4 Add `Branch` variant to `EventAction` enum in `map.rs`
    - Add `Branch { condition: BranchCondition, on_true: Vec<EventAction>, on_false: Vec<EventAction> }` to the existing `EventAction` enum
    - _Requirements: 3.1, 3.5_

  - [x] 1.5 Add `conditional_triggers` field to `TileAttributes` in `map.rs`
    - Add `#[serde(default)] pub conditional_triggers: Vec<ConditionalTrigger>` to `TileAttributes`
    - _Requirements: 4.1, 4.7_

  - [x] 1.6 Add `conditional_triggers` field to `NpcInstance` in `spritesheet.rs`
    - Add `#[serde(default)] pub conditional_triggers: Vec<ConditionalTrigger>` to `NpcInstance`
    - _Requirements: 5.1, 5.6_

  - [x] 1.7 Register `condition` module in `crates/rpg-toolkit-common/src/lib.rs`
    - Add `pub mod condition;` and re-export key types (`BranchCondition`, `ConditionCheck`, `ConditionOperator`, `ConditionLogic`, `ConditionalTrigger`)
    - _Requirements: 1.1, 1.2, 1.3, 1.4_

- [ ] 2. Checkpoint — Ensure the project compiles
  - Ensure `cargo build` succeeds for `rpg-toolkit-common` crate, ask the user if questions arise.

- [x] 3. Integrate Branch action into the renderer
  - [x] 3.1 Add `Branch` match arm to `advance_action_queue` in `crates/rpg-toolkit-renderer/src/systems/triggers.rs`
    - Evaluate `condition.evaluate(&game_state.flags)` (or empty map if GameState absent)
    - Pop the `Branch` action from the queue
    - Push `on_true` or `on_false` actions to the front of the queue (in reverse order, same pattern as existing `StateCheck`)
    - _Requirements: 3.2, 3.3, 3.4, 10.3, 10.4_

  - [x] 3.2 Modify `check_triggers` in `triggers.rs` to evaluate `conditional_triggers` on tiles
    - After existing `required_state` check (which gates visibility/opacity), before collecting `event_trigger` actions
    - For each layer at destination tile: iterate `conditional_triggers` in order, evaluate each condition against `game_state.flags`
    - If a condition matches, use that trigger's `actions` instead of `event_trigger` and break
    - Fall through to existing `event_trigger` behavior if no conditional trigger matches
    - Add `game_state: Res<GameState>` parameter to `check_triggers` system
    - _Requirements: 4.3, 4.4, 4.5, 4.6, 10.1_

  - [x] 3.3 Modify `npc_trigger_system` in `crates/rpg-toolkit-renderer/src/systems/npc.rs` to evaluate `conditional_triggers` on NPCs
    - After `required_state` passes and before using `event_triggers`, check `conditional_triggers` in order
    - For collision triggers: evaluate conditional triggers, use first match's actions or fall through to `event_triggers`
    - For interaction triggers: same pattern — evaluate conditional triggers first
    - _Requirements: 5.2, 5.3, 5.4, 5.5, 10.2_

- [ ] 4. Checkpoint — Ensure renderer builds and existing tests pass
  - Ensure `cargo build` succeeds for `rpg-toolkit-renderer` and existing property tests still pass, ask the user if questions arise.

- [x] 5. Implement Branch editor UI components
  - [x] 5.1 Add `Branch` to `ActionType` enum in `crates/rpg-toolkit-editor/src/plugins/attribute/action_editor.rs`
    - Add `Branch` variant to the `ActionType` enum
    - Add Branch-related fields to `ActionEditorState`: `branch_logic: ConditionLogic`, `branch_checks: Vec<ConditionCheck>`, `branch_on_true: Vec<EventAction>`, `branch_on_false: Vec<EventAction>`, `branch_on_true_editor: Box<ActionEditorState>`, `branch_on_false_editor: Box<ActionEditorState>`
    - Update `Default`, `load_from_action`, and `build_action` implementations for the new `Branch` variant
    - _Requirements: 6.1, 7.1_

  - [x] 5.2 Add depth parameter to `render_action_editor` in `action_editor_ui.rs`
    - Add `depth: usize` parameter (default 0 at top-level call sites)
    - When `depth >= 1`, exclude `Branch` and `StateCheck` from the action type ComboBox dropdown
    - Update all existing call sites (`event_trigger_dialog.rs`, `npc_dialog.rs`) to pass `depth: 0`
    - _Requirements: 6.4, 6.7_

  - [x] 5.3 Implement condition editor form in `action_editor_forms.rs`
    - Create `render_branch_form` function
    - Render logic selector ComboBox (All / Any)
    - Render dynamic list of condition check rows: key text field, operator ComboBox (Equals, Not Equals, Exists, Not Exists), value text field (disabled when operator is Exists/NotExists), remove button (✕)
    - Render "Add Condition" button to append a new check
    - Validate: disable save button if no checks or any check has empty key
    - _Requirements: 7.1, 7.2, 7.3, 7.4, 7.5, 7.6, 7.7_

  - [x] 5.4 Implement nested action editors for Branch on_true/on_false in `action_editor_ui.rs`
    - When displaying a `Branch` or `StateCheck` action in the list, render `CollapsingHeader` sections for on_true and on_false
    - Inside each collapsed section, call `render_action_editor` recursively with `depth + 1`
    - Nested editors operate on `&mut Vec<EventAction>` references to the branch's on_true/on_false fields
    - _Requirements: 6.1, 6.2, 6.3, 6.5, 6.6_

  - [x] 5.5 Add `Branch` to the action list label display in `action_editor_ui.rs`
    - Display Branch actions in the list with format: "N. Branch — {logic} [{check_count} checks] | true:{on_true_count} false:{on_false_count}"
    - _Requirements: 6.1_

- [x] 6. Implement ConditionalTrigger editor panels
  - [x] 6.1 Add ConditionalTrigger panel to `EventTriggerDialog` in `event_trigger_dialog.rs`
    - Add `conditional_triggers: Vec<ConditionalTrigger>` field to `EventTriggerDialog` resource
    - Render "Conditional Triggers" section above the default action list
    - Render "Add Conditional Trigger" button
    - Each trigger as a numbered `CollapsingHeader` ("Condition 1", "Condition 2", etc.)
    - When expanded: condition editor (logic + checks) and nested action editor (`depth: 1`) for that trigger's actions
    - Reorder buttons (▲▼) and remove button (✕)
    - Save logic: persist `conditional_triggers` to the tile's `TileAttributes`
    - _Requirements: 8.1, 8.2, 8.3, 8.4, 8.5, 8.6, 8.8_

  - [x] 6.2 Add ConditionalTrigger panel to NPC dialog in `npc_dialog.rs`
    - Add `conditional_triggers: Vec<ConditionalTrigger>` field to `NpcPlacementDialog` resource
    - Render the same "Conditional Triggers" section as in EventTriggerDialog
    - Pre-populate from NPC instance when editing
    - Save logic: persist `conditional_triggers` to the `NpcInstance`
    - _Requirements: 8.7_

- [ ] 7. Checkpoint — Ensure full project compiles
  - Ensure `cargo build` succeeds for all crates, ask the user if questions arise.

- [ ]* 8. Write property tests for condition evaluation
  - [x] 8.1 Write property test for BranchCondition evaluation semantics
    - **Property 1: BranchCondition Evaluation Semantics**
    - Generate arbitrary `BranchCondition` (random logic, random checks) and arbitrary `HashMap<String, String>` flags
    - Verify: `All` logic returns true iff every check passes (empty = true); `Any` logic returns true iff at least one check passes (empty = true)
    - **Validates: Requirements 1.6, 1.7, 1.8**

  - [x] 8.2 Write property test for ConditionCheck operator semantics
    - **Property 2: ConditionCheck Operator Semantics**
    - Generate arbitrary `ConditionCheck` and arbitrary `HashMap<String, String>` flags
    - Verify each operator: `Equals` true iff value matches, `NotEquals` true iff key absent or value differs, `Exists` true iff key present, `NotExists` true iff key absent
    - Handle edge case: `Equals` with `None` value → false, `NotEquals` with `None` value → true
    - **Validates: Requirements 1.5, 2.1, 2.2, 2.3, 2.4, 2.5**

  - [ ]* 8.3 Write property test for Branch action dispatch
    - **Property 3: Branch Action Dispatches Correct Branch**
    - Generate arbitrary `BranchCondition`, `on_true` actions, `on_false` actions, and `HashMap<String, String>` flags
    - Simulate: evaluate condition, then verify the queue front matches `on_true` when true, `on_false` when false
    - **Validates: Requirements 3.2, 3.3, 3.4**

  - [ ]* 8.4 Write property test for ConditionalTrigger first-match-wins
    - **Property 4: ConditionalTrigger First-Match-Wins Selection**
    - Generate arbitrary `Vec<ConditionalTrigger>`, default actions, and `HashMap<String, String>` flags
    - Verify: selected actions are from the first trigger whose condition evaluates true, or default if none match
    - **Validates: Requirements 4.3, 4.4, 4.5, 4.6, 5.2, 5.3, 5.4, 5.5**

  - [ ]* 8.5 Write property test for required_state precedence
    - **Property 5: required_state Precedence Over ConditionalTriggers**
    - Generate tile/NPC with non-matching `required_state` and arbitrary `conditional_triggers`
    - Verify: no actions produced regardless of conditional trigger conditions
    - **Validates: Requirements 10.1, 10.2**

  - [ ]* 8.6 Write property test for serialization round-trip
    - **Property 6: Serialization Round-Trip**
    - Generate arbitrary valid instances of `BranchCondition`, `ConditionCheck`, `ConditionalTrigger`, `EventAction::Branch`, `TileAttributes` with conditional triggers, `NpcInstance` with conditional triggers
    - Serialize to JSON, deserialize, assert equality
    - **Validates: Requirements 1.10, 3.6, 4.8, 5.7, 9.5**

- [x] 9. Register property test in test configuration
  - Add `[[test]] name = "event_branching" path = "event_branching.rs"` entry to `tests/properties/Cargo.toml`
  - _Requirements: 1.10, 3.6, 9.5_

- [x] 10. Final checkpoint — Ensure all tests pass
  - Ensure `cargo test` succeeds for all crates and property tests, ask the user if questions arise.

## Notes

- Tasks marked with `*` are optional and can be skipped for faster MVP
- Each task references specific requirements for traceability
- Checkpoints ensure incremental validation
- Property tests validate universal correctness properties from the design document
- The existing `StateCheck` variant remains completely unchanged — no modifications to its runtime behavior or serialization format
- All new fields use `#[serde(default)]` for backward compatibility with existing project files
- Editor nesting is limited to depth 1 (no Branch/StateCheck inside nested editors) to keep the UI simple
