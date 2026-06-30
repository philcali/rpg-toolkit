# Implementation Plan: Event Rewards

## Overview

This plan implements five new reward-oriented `EventAction` variants (GiveCurrency, GiveExperience, GiveItem, LearnAbility, AddPartyMember) with `TransferDirection` support, new ECS resources for tracking player state, persistence layer expansion, and editor UI extensions. Implementation proceeds bottom-up: data model first, then runtime systems, persistence, and finally editor UI.

## Tasks

- [x] 1. Data model: TransferDirection and reward EventAction variants
  - [x] 1.1 Add TransferDirection enum and reward variants to EventAction in rpg-toolkit-common
    - Add `TransferDirection` enum (Give/Take with `#[default] Give`) to `map.rs`
    - Add type aliases `ItemId`, `AbilityId`, `CharacterId` (all `String`) if not already present
    - Add `default_quantity` helper function returning `1u32`
    - Add five new variants to `EventAction`: `GiveCurrency`, `GiveExperience`, `GiveItem`, `LearnAbility`, `AddPartyMember` with all fields (`amount`/`item_id`/`ability_id`/`character_id`, `direction`, `on_success`, `on_failure`, `target`, `quantity`) using `#[serde(default)]` annotations
    - Update `pub use` exports in `lib.rs` to include `TransferDirection`
    - _Requirements: 1.1, 1.4, 1.6, 1.7, 1.8, 1.9, 3.1, 3.4, 3.7, 3.9, 3.10, 3.11, 5.1, 5.6, 5.7, 5.9, 5.10, 5.11, 7.1, 7.6, 7.8, 7.9, 7.10, 9.1, 9.4, 9.6, 9.7, 9.8_

  - [x] 1.2 Add custom deserialization validation for reward action fields
    - Implement `#[serde(try_from)]` or custom `Deserialize` with validation using intermediate raw structs (following the `ChoiceData` pattern)
    - GiveCurrency: validate `amount` in `[1, 9_999_999]`
    - GiveExperience: validate `amount` in `[1, 9_999_999]`; `target` if present must be non-empty
    - GiveItem: validate `item_id` non-empty; `quantity` in `[1, 999]`
    - LearnAbility: validate `ability_id` non-empty; `target` non-empty
    - AddPartyMember: validate `character_id` length 1–64
    - Return descriptive deserialization errors for invalid inputs
    - _Requirements: 1.2, 1.3, 1.10, 3.2, 3.3, 3.5, 3.6, 5.2, 5.3, 5.4, 5.5, 7.2, 7.3, 7.4, 7.5, 9.2, 9.3_

  - [x] 1.3 Write property test for reward action serialization round-trip
    - **Property 1: Reward action serialization round-trip**
    - Create `crates/rpg-toolkit-common/tests/properties/event_reward_actions.rs`
    - Generate arbitrary valid reward EventAction values (all 5 types, both directions, nested actions up to depth 2)
    - Assert serialize → deserialize produces equal value
    - Use `ProptestConfig { cases: 100, .. }`
    - **Validates: Requirements 1.5, 3.8, 5.8, 7.7, 9.5, 13.3, 13.6**

  - [ ]* 1.4 Write property test for deserialization rejection of invalid parameters
    - **Property 5: Deserialization rejects invalid reward action parameters**
    - Generate amounts outside `[1, 9_999_999]`, quantities outside `[1, 999]`, empty strings for required fields
    - Verify deserialization produces errors in all invalid cases
    - **Validates: Requirements 1.2, 1.3, 3.2, 3.3, 3.6, 5.3, 5.5, 7.3, 7.5, 9.3**

  - [x] 1.5 Write unit tests for backward compatibility and serialization format
    - Verify pre-existing EventAction variants still deserialize correctly (no regression)
    - Verify each reward variant serializes with correct `"type"` tag
    - Verify `direction` defaults to `Give` when absent from JSON
    - Verify `on_success`/`on_failure` default to empty when absent from JSON
    - Verify invalid `direction` string produces descriptive error
    - _Requirements: 13.1, 13.2, 13.4, 13.5_

- [x] 2. Checkpoint - Ensure all data model tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [x] 3. ECS resources and ActionQueue integration in rpg-toolkit-renderer
  - [x] 3.1 Add new ECS resource types to rpg-toolkit-renderer
    - Create or extend `resources.rs` with `CurrencyState`, `InventoryState`, `CharacterProgress`, `CharacterProgressState`, `PartyState` structs
    - Derive `Resource`, `Default`, `Debug`, `Clone`, `PartialEq`, `Eq` as appropriate
    - Register all four resources with `init_resource::<T>()` in `ProjectRendererPlugin::build()`
    - Update `pub use` exports in `lib.rs` for the new resources
    - _Requirements: 2.4, 4.7, 10.5_

  - [x] 3.2 Import new EventAction variants in triggers.rs and handle GiveCurrency
    - Import `TransferDirection` and new types in `systems/triggers.rs`
    - Add system parameters for `CurrencyState` in `advance_action_queue`
    - Handle `GiveCurrency` with direction `Give`: saturating add to balance, pop and continue
    - Handle `GiveCurrency` with direction `Take`: sufficiency check, subtract if sufficient, push `on_success`/`on_failure` branch to front
    - _Requirements: 2.1, 2.2, 2.3, 2.5, 2.6, 2.7, 2.8_

  - [x] 3.3 Handle GiveExperience action in ActionQueue
    - Add system parameter for `CharacterProgressState` and `PartyState`
    - Handle `GiveExperience` Give direction: add experience to target or all party members (saturating), log warnings for missing characters, non-blocking
    - Handle `GiveExperience` Take direction: atomic sufficiency check across target(s), subtract if sufficient, push branch to front
    - Handle `target: None` Take: all party members must have sufficient experience (atomic check)
    - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5, 4.6, 4.8, 4.9, 4.10, 4.11, 4.12, 4.13, 4.14, 4.15_

  - [x] 3.4 Handle GiveItem action in ActionQueue
    - Add system parameter for `InventoryState`
    - Access `ItemRegistry` from `RendererProjectData` (or project items) for stackability/stack_limit checks
    - Handle `GiveItem` Give direction: new item insert, stackable increment (cap at `stack_limit`), unstackable duplicate → `on_failure` branch, stack cap reached → `on_failure` branch
    - Handle `GiveItem` Take direction: sufficiency check on quantity, subtract/remove entry, push branch to front
    - Log warning and advance if `item_id` not found in registry
    - _Requirements: 6.1, 6.2, 6.3, 6.4, 6.5, 6.6, 6.7, 6.8, 6.9, 6.10, 6.11_

  - [x] 3.5 Handle LearnAbility action in ActionQueue
    - Handle `LearnAbility` Give direction: add ability to character's learned list, no-op if already known, log warnings for missing character/ability
    - Handle `LearnAbility` Take direction: check if character knows ability, remove if present and push `on_success`, leave unchanged and push `on_failure` if not known
    - _Requirements: 8.1, 8.2, 8.3, 8.4, 8.5, 8.6, 8.7, 8.8, 8.9, 8.10, 8.11_

  - [x] 3.6 Handle AddPartyMember action in ActionQueue
    - Handle `AddPartyMember` Give direction: append to party list, no-op if already present, log warning if character not in registry
    - Handle `AddPartyMember` Take direction: check membership, remove if present and push `on_success`, push `on_failure` if not in party
    - _Requirements: 10.1, 10.2, 10.3, 10.4, 10.6, 10.7, 10.8, 10.9, 10.10, 10.11_

  - [x] 3.7 Write unit tests for ActionQueue reward action processing
    - Test GiveCurrency Give/Take with sufficient and insufficient balance
    - Test GiveExperience Give/Take for single target and all-party (atomic check)
    - Test GiveItem Give: new item, stackable, unstackable duplicate, stack cap triggers on_failure
    - Test GiveItem Take: remove quantity, remove entry at zero, insufficient quantity
    - Test LearnAbility Give/Take: learn, idempotent, forget, not-known failure
    - Test AddPartyMember Give/Take: add, idempotent, remove, not-in-party failure
    - Test branch injection pushes on_success/on_failure correctly to queue front
    - Test non-blocking actions chain within single frame
    - _Requirements: 12.1, 12.2, 12.3, 12.4, 12.5, 12.6, 12.7, 12.8_

- [x] 4. Checkpoint - Ensure renderer tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [x] 5. Persistence layer changes
  - [x] 5.1 Expand SaveFile and add save_game function
    - Add `CharacterProgressData` struct to `save.rs`
    - Expand `SaveFile` with `currency: u64`, `inventory: BTreeMap<String, u32>`, `party: Vec<String>`, `character_progress: BTreeMap<String, CharacterProgressData>` — all with `#[serde(default)]`
    - Add public `save_game` function that accepts references to `GameState`, `CurrencyState`, `InventoryState`, `PartyState`, `CharacterProgressState`, `SavePath` and writes the save file
    - _Requirements: 2.4, 4.7, 10.5, 13.1_

  - [x] 5.2 Remove save_shutdown system and update plugin registration
    - Remove the `save_shutdown` function definition from `lib.rs`
    - Remove `.add_systems(Last, save_shutdown)` from `ProjectRendererPlugin::build()`
    - _Requirements: (design decision — persistence is on-demand only)_

  - [x] 5.3 Update launcher load path to populate new ECS resources from SaveFile
    - In `crates/rpg-toolkit-launcher/src/main.rs`, after loading `SaveFile`, insert `CurrencyState`, `InventoryState`, `PartyState`, `CharacterProgressState` resources from the save file fields
    - Import the new resource types from `rpg_toolkit_renderer`
    - Ensure existing `GameState` population remains unchanged
    - _Requirements: 2.4, 4.7, 10.5_

  - [ ]* 5.4 Write property test for SaveFile serialization round-trip
    - **Property 6: SaveFile serialization round-trip preserves all resource state**
    - Generate arbitrary valid `SaveFile` values with all fields populated
    - Assert serialize → deserialize produces equal value
    - **Validates: Requirements 2.4, 4.7, 10.5**

  - [x] 5.5 Write unit tests for persistence backward compatibility
    - Verify old save file (only `state` field) deserializes into new `SaveFile` with zeros/empty defaults for new fields
    - Verify `save_game` produces a `SaveFile` matching input resource state
    - Verify `CharacterProgressData` preserves experience and learned_abilities
    - Verify empty resources produce a valid minimal save file
    - _Requirements: 13.1_

- [x] 6. Checkpoint - Ensure persistence tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [x] 7. Editor UI for reward actions
  - [x] 7.1 Extend ActionType enum and ActionEditorState with reward fields
    - Add `GiveCurrency`, `GiveExperience`, `GiveItem`, `LearnAbility`, `AddPartyMember` to `ActionType` enum in `action_editor.rs`
    - Add reward-specific fields to `ActionEditorState`: `reward_direction`, `reward_on_success`, `reward_on_failure`, `reward_on_success_editor`, `reward_on_failure_editor`, `currency_amount`, `experience_amount`, `experience_target`, `give_item_id`, `give_item_quantity`, `learn_ability_id`, `learn_ability_target`, `add_party_character_id`
    - Update `Default` impl and `reset()` to initialize new fields
    - Update `new_nested()` to handle new fields without recursion
    - _Requirements: 11.1_

  - [x] 7.2 Implement load_from_action and build_action for reward variants
    - Extend `load_from_action` to populate reward fields from existing reward EventAction values
    - Extend `build_action` to construct reward EventAction variants from editor state fields
    - Include direction, on_success, on_failure in built actions
    - Apply validation: return `None` if required fields empty/invalid
    - _Requirements: 11.8_

  - [x] 7.3 Add reward action form UI rendering in action_editor_forms.rs
    - Add GiveCurrency form: numeric input for amount (default 100, range 1–9,999,999)
    - Add GiveExperience form: numeric input for amount + optional character selector for target (default "All Party Members")
    - Add GiveItem form: searchable item selector from ItemRegistry + numeric input for quantity (default 1, range 1–999)
    - Add LearnAbility form: searchable ability selector + searchable character selector for target
    - Add AddPartyMember form: searchable character selector from CharacterRegistry
    - Add TransferDirection toggle ("Give" / "Take") displayed for all reward action types
    - _Requirements: 11.2, 11.3, 11.4, 11.5, 11.6, 11.10_

  - [x] 7.4 Add nested action editors for on_success/on_failure branches
    - When direction is "Take", show expandable `on_success` action list editor (optional)
    - When direction is "Take", show expandable `on_failure` action list editor (required)
    - Hide both editors when direction is "Give"
    - Nested editors reuse `ActionEditorState::new_nested()` pattern (same as EditorChoice)
    - Support recursive action type selection in nested editors
    - _Requirements: 11.11, 11.12, 11.13_

  - [x] 7.5 Add validation logic for reward action forms
    - Disable Add/Update button if required fields are empty or outside valid range
    - Clamp amount to 1–9,999,999 and quantity to 1–999 on save
    - When direction is "Take" and `on_failure` list is empty, disable Add/Update button with validation message
    - _Requirements: 11.7, 11.9, 11.14_

- [x] 8. Final checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

## Notes

- Tasks marked with `*` are optional and can be skipped for faster MVP
- Each task references specific requirements for traceability
- Checkpoints ensure incremental validation
- Property tests validate universal correctness properties from the design document
- Unit tests validate specific examples and edge cases
- The project uses `proptest` as an existing dev-dependency in rpg-toolkit-common
- Existing property test files live in `crates/rpg-toolkit-common/tests/properties/`

## Task Dependency Graph

```json
{
  "waves": [
    { "id": 0, "tasks": ["1.1"] },
    { "id": 1, "tasks": ["1.2"] },
    { "id": 2, "tasks": ["1.3", "1.4", "1.5", "3.1"] },
    { "id": 3, "tasks": ["3.2", "3.3", "3.4", "3.5", "3.6"] },
    { "id": 4, "tasks": ["3.7", "5.1"] },
    { "id": 5, "tasks": ["5.2", "5.3"] },
    { "id": 6, "tasks": ["5.4", "5.5", "7.1"] },
    { "id": 7, "tasks": ["7.2"] },
    { "id": 8, "tasks": ["7.3"] },
    { "id": 9, "tasks": ["7.4", "7.5"] }
  ]
}
```
