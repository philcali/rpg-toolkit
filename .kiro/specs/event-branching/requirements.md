# Requirements Document

## Introduction

This feature extends the existing `StateCheck` event action with a richer editor UX and more expressive condition model for authoring branching event sequences in the RPG toolkit. Currently, `StateCheck` supports a single key/value check with inline `on_true`/`on_false` branches, but the editor only offers a minimal text-field interface with no way to visually manage the nested action sequences. This feature adds:

1. **A dedicated branching editor UX** — collapsible, nested action editors for the true/false branches of a `StateCheck`, so designers can visually compose full sequences inside each branch without editing JSON.
2. **Compound conditions** — a new `BranchCondition` model supporting multiple checks combined with AND/OR logic, and comparison operators beyond simple equality (not-equals, key-exists, key-not-exists).
3. **Sequence-level conditional triggers** — a new `ConditionalTrigger` wrapper on tiles and NPCs that gates an entire event trigger list behind a condition, providing a higher-level alternative to inline `StateCheck` for cases where "this tile's entire behavior changes based on game state."

These changes build on the existing `StateCheck` runtime (which already works: pops itself, pushes the matching branch to the front of the `ActionQueue`) and extend the data model and editor to make complex narrative branching accessible to non-technical designers.

## Glossary

- **ActionQueue**: The Bevy ECS resource that holds a `VecDeque<EventAction>` and processes actions sequentially, waiting for blocking actions to complete before advancing.
- **EventAction**: The `#[serde(tag = "type")]` enum in `rpg-toolkit-common` representing a single step in a trigger sequence.
- **StateCheck**: The existing `EventAction` variant that evaluates a single game state key and dispatches to `on_true` or `on_false` action branches.
- **BranchCondition**: A new condition model that supports compound expressions (multiple checks joined by AND/OR), comparison operators (Equals, NotEquals, Exists, NotExists), and evaluation against the runtime `GameState`.
- **ConditionCheck**: A single atomic comparison within a `BranchCondition` — one key, one operator, and an optional expected value.
- **ConditionOperator**: The enum of comparison operators: `Equals`, `NotEquals`, `Exists`, `NotExists`.
- **ConditionLogic**: The enum specifying how multiple `ConditionCheck` items are combined: `All` (AND) or `Any` (OR).
- **ConditionalTrigger**: A new data structure on `TileAttributes` and `NpcInstance` that pairs a `BranchCondition` with an alternative event trigger list, enabling sequence-level branching.
- **GameState**: The Bevy ECS resource holding a `HashMap<String, String>` of runtime game flags.
- **Editor**: The `rpg-toolkit-editor` crate providing the map editing UI.
- **ActionEditorState**: The editor state struct managing form fields for adding/editing `EventAction` entries.
- **Branch_Editor**: A nested instance of the action editor UI rendered inside a collapsible section for editing `on_true` or `on_false` sequences of a `StateCheck`.
- **Renderer**: The `rpg-toolkit-renderer` crate responsible for running the game world and processing triggers.

## Requirements

### Requirement 1: BranchCondition Data Model

**User Story:** As a game designer, I want to express complex conditions combining multiple state checks with AND/OR logic, so that I can create branching narratives that depend on multiple game flags.

#### Acceptance Criteria

1. THE `rpg-toolkit-common` crate SHALL define a `ConditionOperator` enum with variants `Equals`, `NotEquals`, `Exists`, and `NotExists`.
2. THE `rpg-toolkit-common` crate SHALL define a `ConditionCheck` struct with a `key` field of type `String`, an `operator` field of type `ConditionOperator`, and a `value` field of type `Option<String>`.
3. THE `rpg-toolkit-common` crate SHALL define a `ConditionLogic` enum with variants `All` and `Any`.
4. THE `rpg-toolkit-common` crate SHALL define a `BranchCondition` struct with a `logic` field of type `ConditionLogic` and a `checks` field of type `Vec<ConditionCheck>`.
5. WHEN the `operator` is `Exists` or `NotExists`, THE `value` field of `ConditionCheck` SHALL be ignored during evaluation.
6. WHEN the `logic` is `All`, THE `BranchCondition` SHALL evaluate to true only when every `ConditionCheck` in `checks` evaluates to true.
7. WHEN the `logic` is `Any`, THE `BranchCondition` SHALL evaluate to true when at least one `ConditionCheck` in `checks` evaluates to true.
8. WHEN the `checks` list is empty, THE `BranchCondition` SHALL evaluate to true regardless of `logic`.
9. THE `BranchCondition`, `ConditionCheck`, `ConditionOperator`, and `ConditionLogic` types SHALL derive `Serialize` and `Deserialize` using serde.
10. FOR ALL valid `BranchCondition` values, serializing then deserializing SHALL produce an equivalent value (round-trip property).

### Requirement 2: BranchCondition Runtime Evaluation

**User Story:** As a player, I want branching conditions to evaluate correctly against the current game state, so that the story responds accurately to my progress.

#### Acceptance Criteria

1. WHEN a `ConditionCheck` has operator `Equals`, THE Renderer SHALL evaluate it as true if the `GameState` contains the specified key with a value equal to the expected `value`.
2. WHEN a `ConditionCheck` has operator `NotEquals`, THE Renderer SHALL evaluate it as true if the `GameState` does not contain the specified key, or contains the key with a value different from the expected `value`.
3. WHEN a `ConditionCheck` has operator `Exists`, THE Renderer SHALL evaluate it as true if the `GameState` contains the specified key (regardless of value).
4. WHEN a `ConditionCheck` has operator `NotExists`, THE Renderer SHALL evaluate it as true if the `GameState` does not contain the specified key.
5. WHEN a `ConditionCheck` has operator `Equals` or `NotEquals` and `value` is `None`, THE Renderer SHALL treat the check as always false for `Equals` and always true for `NotEquals`.

### Requirement 3: Enhanced StateCheck with BranchCondition

**User Story:** As a game designer, I want to use compound conditions in my inline StateCheck actions, so that I can branch on multiple flags without nesting multiple StateChecks.

#### Acceptance Criteria

1. THE EventAction enum SHALL include a new `Branch` variant with a `condition` field of type `BranchCondition`, an `on_true` field of type `Vec<EventAction>`, and an `on_false` field of type `Vec<EventAction>`.
2. WHEN the ActionQueue advances to a `Branch` action, THE Renderer SHALL evaluate the `BranchCondition` against the current `GameState`.
3. WHEN the `BranchCondition` evaluates to true, THE Renderer SHALL pop the `Branch` action and push all actions from `on_true` to the front of the ActionQueue.
4. WHEN the `BranchCondition` evaluates to false, THE Renderer SHALL pop the `Branch` action and push all actions from `on_false` to the front of the ActionQueue.
5. THE existing `StateCheck` variant SHALL remain unchanged for backward compatibility with existing project files.
6. FOR ALL valid `Branch` EventAction values, serializing then deserializing SHALL produce an equivalent value (round-trip property).

### Requirement 4: ConditionalTrigger on Tiles

**User Story:** As a game designer, I want to attach conditions to entire tile event trigger sequences, so that a tile's behavior changes completely based on game state without needing inline StateCheck actions.

#### Acceptance Criteria

1. THE `TileAttributes` struct SHALL include a new `conditional_triggers` field of type `Vec<ConditionalTrigger>`.
2. THE `ConditionalTrigger` struct SHALL contain a `condition` field of type `BranchCondition` and an `actions` field of type `Vec<EventAction>`.
3. WHEN the player steps on a tile with `conditional_triggers`, THE Renderer SHALL evaluate each `ConditionalTrigger` condition in order against the current `GameState`.
4. WHEN a `ConditionalTrigger` condition evaluates to true, THE Renderer SHALL use that trigger's `actions` list instead of the tile's default `event_trigger` list.
5. WHEN no `ConditionalTrigger` condition evaluates to true, THE Renderer SHALL fall through to the tile's default `event_trigger` list.
6. WHEN multiple `ConditionalTrigger` conditions evaluate to true, THE Renderer SHALL use only the first matching trigger's `actions` (priority order).
7. THE `conditional_triggers` field SHALL default to an empty vector when not present in serialized data.
8. FOR ALL valid `TileAttributes` values containing `conditional_triggers`, serializing then deserializing SHALL produce an equivalent value (round-trip property).

### Requirement 5: ConditionalTrigger on NPCs

**User Story:** As a game designer, I want to attach conditions to NPC event trigger sequences, so that NPC dialog and behavior changes based on story progression.

#### Acceptance Criteria

1. THE `NpcInstance` struct SHALL include a new `conditional_triggers` field of type `Vec<ConditionalTrigger>`.
2. WHEN the player interacts with or collides with an NPC that has `conditional_triggers`, THE Renderer SHALL evaluate each `ConditionalTrigger` condition in order against the current `GameState`.
3. WHEN a `ConditionalTrigger` condition evaluates to true, THE Renderer SHALL use that trigger's `actions` list instead of the NPC's default `event_triggers` list.
4. WHEN no `ConditionalTrigger` condition evaluates to true, THE Renderer SHALL fall through to the NPC's default `event_triggers` list.
5. WHEN multiple `ConditionalTrigger` conditions evaluate to true, THE Renderer SHALL use only the first matching trigger's `actions` (priority order).
6. THE `conditional_triggers` field SHALL default to an empty vector when not present in serialized data.
7. FOR ALL valid `NpcInstance` values containing `conditional_triggers`, serializing then deserializing SHALL produce an equivalent value (round-trip property).

### Requirement 6: Branch Editor UX — Nested Action Editing

**User Story:** As a game designer, I want to visually author actions inside each branch of a StateCheck or Branch action, so that I can compose complex branching sequences without editing JSON.

#### Acceptance Criteria

1. WHEN a `StateCheck` or `Branch` action is displayed in the action list, THE Editor SHALL show a collapsible section for the `on_true` actions and a collapsible section for the `on_false` actions.
2. WHEN the `on_true` section is expanded, THE Editor SHALL render a nested action editor allowing add, remove, reorder, and edit operations on the `on_true` action list.
3. WHEN the `on_false` section is expanded, THE Editor SHALL render a nested action editor allowing add, remove, reorder, and edit operations on the `on_false` action list.
4. THE nested action editors SHALL support all the same action types as the top-level action editor, **except** `StateCheck` and `Branch`, preventing further nesting. Complex multi-tier branching is handled by stacking multiple `ConditionalTrigger` entries with priority ordering.
5. WHEN the user modifies actions inside a branch, THE Editor SHALL update the parent `StateCheck` or `Branch` action's branch list in real time.
6. THE Editor SHALL visually indent nested branch content to indicate hierarchy depth.
7. THE Editor SHALL enforce a maximum nesting depth of 1 (no Branch or StateCheck inside a branch), keeping the UI simple while the data model and runtime support deeper nesting for future expansion.

### Requirement 7: Branch Condition Editor Form

**User Story:** As a game designer, I want a dedicated form for configuring compound branch conditions, so that I can set up multi-check conditions with the appropriate logic without confusion.

#### Acceptance Criteria

1. WHEN `Branch` is selected as the action type, THE Editor SHALL display a condition editor section with a logic selector (All/Any) and a list of condition checks.
2. THE condition editor SHALL allow adding new `ConditionCheck` entries with fields for key, operator, and value.
3. THE condition editor SHALL allow removing individual `ConditionCheck` entries from the list.
4. WHEN the operator is `Exists` or `NotExists`, THE Editor SHALL disable or hide the value field since it is not applicable.
5. THE Editor SHALL display the operator selector as a dropdown with options: Equals, Not Equals, Exists, Not Exists.
6. THE Editor SHALL validate that at least one `ConditionCheck` is present before allowing the Branch action to be saved.
7. THE Editor SHALL validate that the `key` field of each `ConditionCheck` is non-empty before allowing the Branch action to be saved.

### Requirement 8: ConditionalTrigger Editor UX

**User Story:** As a game designer, I want to manage conditional trigger overrides on tiles and NPCs through the editor, so that I can configure sequence-level branching visually.

#### Acceptance Criteria

1. THE Event Trigger Editor dialog SHALL include a "Conditional Triggers" section above the default action list.
2. THE Conditional Triggers section SHALL display each `ConditionalTrigger` as a collapsible panel showing a summary of its condition.
3. WHEN a conditional trigger panel is expanded, THE Editor SHALL display the condition editor (logic selector + condition checks) and a nested action editor for that trigger's actions.
4. THE Editor SHALL provide an "Add Conditional Trigger" button that creates a new `ConditionalTrigger` with an empty condition and empty actions list.
5. THE Editor SHALL provide a remove button for each conditional trigger entry.
6. THE Editor SHALL allow reordering conditional triggers to change their priority (first match wins).
7. THE NPC dialog SHALL include the same Conditional Triggers section for editing NPC conditional triggers.
8. THE Editor SHALL visually indicate the evaluation order with numbered labels (e.g., "Condition 1", "Condition 2") to communicate that the first matching condition takes priority.

### Requirement 9: Serialization Compatibility

**User Story:** As a game designer, I want my existing project files to continue loading correctly after the branching features are added, so that I do not lose any work.

#### Acceptance Criteria

1. WHEN a project file containing only existing `StateCheck` actions is loaded, THE EventAction parser SHALL deserialize all actions correctly without errors.
2. WHEN a project file does not contain `conditional_triggers` fields on tiles or NPCs, THE parser SHALL default those fields to empty vectors.
3. WHEN a project file contains the new `Branch` action type is loaded by an older version of the toolkit, THE EventAction parser SHALL report a clear deserialization error identifying the unknown action type.
4. THE existing `StateCheck` variant SHALL continue to function identically at runtime after the new `Branch` variant is added.
5. FOR ALL valid ProjectFile values containing any combination of `StateCheck`, `Branch`, and `ConditionalTrigger` data, serializing then deserializing SHALL produce an equivalent ProjectFile (round-trip property).

### Requirement 10: Integration with Existing Conditional Systems

**User Story:** As a game designer, I want branching event sequences to work alongside existing `required_state` visibility conditions on NPCs and tiles, so that I can combine visibility gating with behavioral branching.

#### Acceptance Criteria

1. WHEN a tile has both `required_state` and `conditional_triggers`, THE Renderer SHALL first check `required_state` for visibility and only evaluate `conditional_triggers` if the tile is visible.
2. WHEN an NPC has both `required_state` and `conditional_triggers`, THE Renderer SHALL first check `required_state` for visibility and only evaluate `conditional_triggers` if the NPC is active.
3. WHEN a `ConditionalTrigger` action list contains `SetState` actions, THE Renderer SHALL apply state changes immediately so subsequent actions in the same sequence can observe the updated state.
4. WHEN a `ConditionalTrigger` action list contains a `Branch` or `StateCheck` action, THE Renderer SHALL evaluate that nested condition against the current `GameState` at the time of evaluation (including any state changes made earlier in the same sequence).
