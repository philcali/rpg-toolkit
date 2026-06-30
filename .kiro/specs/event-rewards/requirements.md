# Requirements Document

## Introduction

This feature extends the existing `EventAction` enum in `rpg-toolkit-common` with five new reward-oriented variants: **GiveCurrency**, **GiveExperience**, **GiveItem**, **LearnAbility**, and **AddPartyMember**. These actions allow game designers to grant tangible rewards during gameplay events — treasure chests, quest completions, boss defeats, and story milestones. Each reward action supports a **TransferDirection** modifier that enables both granting and taking resources. When direction is `Take`, the action checks resource sufficiency and follows conditional branching (`on_success` / `on_failure`), enabling shop and trade mechanics where progression is gated on spending resources. The new actions integrate into the existing `ActionQueue` sequential processing pipeline and are configurable through the editor's Event Trigger Editor dialog.

## Glossary

- **ActionQueue**: The Bevy ECS resource that holds a `VecDeque<EventAction>` and processes actions sequentially, waiting for blocking actions to complete before advancing.
- **EventAction**: The `#[serde(tag = "type")]` enum in `rpg-toolkit-common` representing a single step in a trigger sequence. Currently has `JumpTo`, `ShowDialog`, `ScreenShake`, `StopScreenShake`, `FadeTransition`, `SetState`, `SetPlayerAppearance`, `StateCheck`, `Branch`, and `ShowSelection` variants.
- **Renderer**: The `rpg-toolkit-renderer` crate responsible for running the game world, processing triggers, and rendering visual effects.
- **Editor**: The `rpg-toolkit-editor` crate providing the map editing UI including the Event Trigger Editor dialog.
- **PartyState**: A new Bevy ECS resource that holds the current party composition, including active `CharacterId` entries representing playable characters available to the player.
- **InventoryState**: A new Bevy ECS resource that holds the player's current inventory as a mapping of `ItemId` to quantity.
- **CurrencyState**: A new Bevy ECS resource that holds the player's current currency balance as a `u64` value.
- **CharacterProgressState**: A new Bevy ECS resource that holds per-character experience and learned abilities, keyed by `CharacterId`.
- **ItemId**: A `String` type alias representing a unique identifier for an item in the `ItemRegistry`.
- **AbilityId**: A `String` type alias representing a unique identifier for an ability in the `AbilityRegistry`.
- **CharacterId**: A `String` type alias representing a unique identifier for a character in the `CharacterRegistry`.
- **TransferDirection**: An enum with two variants — `Give` (default) and `Take` — that determines whether a reward action grants a resource to the player or attempts to remove it. When direction is `Take`, the action performs a sufficiency check and follows conditional branching based on the result.

## Requirements

### Requirement 1: GiveCurrency EventAction Data Model

**User Story:** As a game designer, I want to define an event action that awards or deducts currency from the player, so that treasure chests can grant monetary value and shops can charge for purchases.

#### Acceptance Criteria

1. THE EventAction enum SHALL include a `GiveCurrency` variant with an `amount` field of type `u64`.
2. WHEN the `amount` field is deserialized, THE EventAction parser SHALL accept values in the range 1 to 9_999_999 inclusive.
3. IF the `amount` field is absent from the JSON or contains a value less than 1 or greater than 9_999_999, THEN THE EventAction parser SHALL return a deserialization error indicating the invalid amount.
4. THE EventAction `GiveCurrency` variant SHALL serialize to and deserialize from JSON using the existing `#[serde(tag = "type")]` format, producing a JSON object containing a `"type"` field with value `"GiveCurrency"` and an `"amount"` field with the numeric value.
5. FOR ALL valid GiveCurrency EventAction values, serializing then deserializing SHALL produce an equivalent value (round-trip property).
6. THE `GiveCurrency` variant SHALL include a `direction` field of type `TransferDirection` that defaults to `Give` when absent from JSON.
7. THE `GiveCurrency` variant SHALL include an `on_success` field of type `Vec<EventAction>` that defaults to an empty list when absent from JSON.
8. THE `GiveCurrency` variant SHALL include an `on_failure` field of type `Vec<EventAction>` that defaults to an empty list when absent from JSON.
9. WHEN the `direction` field is deserialized, THE EventAction parser SHALL accept the string values `"Give"` and `"Take"` (case-sensitive).
10. IF the `direction` field contains a value other than `"Give"` or `"Take"`, THEN THE EventAction parser SHALL return a deserialization error indicating an invalid direction.

### Requirement 2: GiveCurrency Runtime Effect

**User Story:** As a player, I want to receive or spend currency when triggered by game events, so that I can accumulate wealth from treasure and purchase items from shops.

#### Acceptance Criteria

1. WHEN the ActionQueue advances to a `GiveCurrency` action with direction `Give`, THE Renderer SHALL add the specified `amount` to the `CurrencyState` resource balance using saturating addition (capping at `u64::MAX`).
2. WHEN a `GiveCurrency` action with direction `Give` is processed, THE ActionQueue SHALL advance immediately to the next action without blocking (non-blocking action).
3. WHEN the ActionQueue advances to a `GiveCurrency` action with direction `Give`, THE Renderer SHALL ignore the `on_success` and `on_failure` fields.
4. THE CurrencyState resource SHALL be initialized with a balance of 0 at renderer startup.
5. WHEN the ActionQueue advances to a `GiveCurrency` action with direction `Take` and the `CurrencyState` balance is greater than or equal to the specified `amount`, THE Renderer SHALL subtract the `amount` from the `CurrencyState` balance.
6. WHEN a `GiveCurrency` action with direction `Take` succeeds, THE ActionQueue SHALL pop the current action and push the `on_success` actions to the front of the queue for immediate processing.
7. WHEN the ActionQueue advances to a `GiveCurrency` action with direction `Take` and the `CurrencyState` balance is less than the specified `amount`, THE Renderer SHALL not modify the `CurrencyState` balance.
8. WHEN a `GiveCurrency` action with direction `Take` fails due to insufficient currency, THE ActionQueue SHALL pop the current action and push the `on_failure` actions to the front of the queue for immediate processing.

### Requirement 3: GiveExperience EventAction Data Model

**User Story:** As a game designer, I want to define an event action that awards or deducts experience points, so that story milestones can grant XP and special mechanics can cost XP.

#### Acceptance Criteria

1. THE EventAction enum SHALL include a `GiveExperience` variant with an `amount` field of type `u64` and an optional `target` field of type `Option<CharacterId>`.
2. WHEN the `amount` field is deserialized, THE EventAction parser SHALL accept values in the range 1 to 9_999_999 inclusive.
3. IF the `amount` field is absent, zero, or greater than 9_999_999 in the JSON input, THEN THE EventAction parser SHALL return a deserialization error indicating the value is out of the accepted range.
4. WHEN the `target` field is absent from JSON, THE EventAction parser SHALL default the field to `None`.
5. WHEN the `target` field is present in JSON, THE EventAction parser SHALL accept a non-empty string value as a valid `CharacterId`.
6. IF the `target` field is present but contains an empty string, THEN THE EventAction parser SHALL return a deserialization error indicating the target is invalid.
7. THE EventAction `GiveExperience` variant SHALL serialize to and deserialize from JSON using the existing `#[serde(tag = "type")]` format.
8. FOR ALL valid GiveExperience EventAction values, serializing then deserializing SHALL produce an equivalent value (round-trip property).
9. THE `GiveExperience` variant SHALL include a `direction` field of type `TransferDirection` that defaults to `Give` when absent from JSON.
10. THE `GiveExperience` variant SHALL include an `on_success` field of type `Vec<EventAction>` that defaults to an empty list when absent from JSON.
11. THE `GiveExperience` variant SHALL include an `on_failure` field of type `Vec<EventAction>` that defaults to an empty list when absent from JSON.

### Requirement 4: GiveExperience Runtime Effect

**User Story:** As a player, I want party members to gain or lose experience from story events, so that key narrative moments contribute to or cost character progression.

#### Acceptance Criteria

1. WHEN the ActionQueue advances to a `GiveExperience` action with direction `Give` and `target` set to `None`, THE Renderer SHALL add the specified `amount` to the experience total in the `CharacterProgressState` resource for each character listed in the `PartyState` resource using saturating addition (capping at `u64::MAX`).
2. WHEN the ActionQueue advances to a `GiveExperience` action with direction `Give` and `target` set to a specific `CharacterId`, THE Renderer SHALL add the specified `amount` only to that character's experience total in the `CharacterProgressState` resource using saturating addition (capping at `u64::MAX`).
3. IF the `target` CharacterId does not exist in the CharacterProgressState, THEN THE Renderer SHALL log a warning and advance the ActionQueue without modifying any state.
4. IF `target` is `None` and a party member's CharacterId does not exist in the CharacterProgressState, THEN THE Renderer SHALL log a warning for that member and skip it, continuing to process remaining party members.
5. WHEN a `GiveExperience` action with direction `Give` is processed, THE ActionQueue SHALL advance immediately to the next action without blocking (non-blocking action).
6. WHEN the ActionQueue advances to a `GiveExperience` action with direction `Give`, THE Renderer SHALL ignore the `on_success` and `on_failure` fields.
7. THE CharacterProgressState resource SHALL be initialized with an empty mapping at renderer startup.
8. WHEN the ActionQueue advances to a `GiveExperience` action with direction `Take` and `target` set to a specific `CharacterId`, THE Renderer SHALL check whether that character's experience total is greater than or equal to `amount`.
9. WHEN a `GiveExperience` action with direction `Take` targets a specific character and the character has sufficient experience, THE Renderer SHALL subtract `amount` from that character's experience total.
10. WHEN a `GiveExperience` action with direction `Take` targets a specific character and the character has insufficient experience, THE Renderer SHALL not modify the experience total.
11. WHEN the ActionQueue advances to a `GiveExperience` action with direction `Take` and `target` set to `None`, THE Renderer SHALL check whether every character in the `PartyState` has experience greater than or equal to `amount`.
12. WHEN a `GiveExperience` action with direction `Take` targets all party members and all members have sufficient experience, THE Renderer SHALL subtract `amount` from each party member's experience total.
13. WHEN a `GiveExperience` action with direction `Take` targets all party members and any member has insufficient experience, THE Renderer SHALL not modify any character's experience total (atomic check).
14. WHEN a `GiveExperience` action with direction `Take` succeeds, THE ActionQueue SHALL pop the current action and push the `on_success` actions to the front of the queue for immediate processing.
15. WHEN a `GiveExperience` action with direction `Take` fails due to insufficient experience, THE ActionQueue SHALL pop the current action and push the `on_failure` actions to the front of the queue for immediate processing.

### Requirement 5: GiveItem EventAction Data Model

**User Story:** As a game designer, I want to define an event action that adds or removes an item from the player's inventory, so that treasure chests can grant loot and shops can take payment in items.

#### Acceptance Criteria

1. THE EventAction enum SHALL include a `GiveItem` variant with an `item_id` field of type `ItemId` and a `quantity` field of type `u32`.
2. WHEN the `item_id` field is deserialized, THE EventAction parser SHALL accept non-empty string values.
3. IF the `item_id` field is an empty string during deserialization, THEN THE EventAction parser SHALL return a deserialization error indicating that item_id must not be empty.
4. WHEN the `quantity` field is deserialized, THE EventAction parser SHALL accept values in the range 1 to 999 inclusive.
5. IF the `quantity` field value is less than 1 or greater than 999 during deserialization, THEN THE EventAction parser SHALL return a deserialization error indicating the valid range.
6. WHEN the `quantity` field is not specified in JSON, THE EventAction parser SHALL default to 1.
7. THE EventAction `GiveItem` variant SHALL serialize to and deserialize from JSON using the existing `#[serde(tag = "type")]` format.
8. FOR ALL valid GiveItem EventAction values, serializing then deserializing SHALL produce an equivalent value (round-trip property).
9. THE `GiveItem` variant SHALL include a `direction` field of type `TransferDirection` that defaults to `Give` when absent from JSON.
10. THE `GiveItem` variant SHALL include an `on_success` field of type `Vec<EventAction>` that defaults to an empty list when absent from JSON.
11. THE `GiveItem` variant SHALL include an `on_failure` field of type `Vec<EventAction>` that defaults to an empty list when absent from JSON.

### Requirement 6: GiveItem Runtime Effect

**User Story:** As a player, I want to receive or surrender items when triggered by game events, so that treasure chests grant loot and I can trade items for services.

#### Acceptance Criteria

1. WHEN the ActionQueue advances to a `GiveItem` action with direction `Give` and the item does not already exist in the `InventoryState`, THE Renderer SHALL insert a new entry in the `InventoryState` resource with the specified `item_id` and `quantity`.
2. IF the item already exists in the InventoryState and the item is stackable and direction is `Give`, THEN THE Renderer SHALL increase the existing quantity by the specified amount, capping the result at the item's `stack_limit` and silently discarding any excess beyond that limit.
3. IF the item is not stackable and already exists in the InventoryState with quantity 1 and direction is `Give`, THEN THE Renderer SHALL log a warning and not add a duplicate.
4. IF the `item_id` does not exist in the project's `ItemRegistry`, THEN THE Renderer SHALL log a warning and advance the ActionQueue without modifying the inventory.
5. WHEN a `GiveItem` action with direction `Give` is processed, THE ActionQueue SHALL advance immediately to the next action without blocking (non-blocking action).
6. WHEN the ActionQueue advances to a `GiveItem` action with direction `Give`, THE Renderer SHALL ignore the `on_success` and `on_failure` fields.
7. WHEN the ActionQueue advances to a `GiveItem` action with direction `Take`, THE Renderer SHALL check whether the `InventoryState` contains the specified `item_id` with quantity greater than or equal to the specified `quantity`.
8. WHEN a `GiveItem` action with direction `Take` succeeds (sufficient quantity exists), THE Renderer SHALL subtract the specified `quantity` from the item's inventory entry, removing the entry entirely if the resulting quantity is zero.
9. WHEN a `GiveItem` action with direction `Take` fails because the item does not exist in the inventory or the existing quantity is less than the specified `quantity`, THE Renderer SHALL not modify the `InventoryState`.
10. WHEN a `GiveItem` action with direction `Take` succeeds, THE ActionQueue SHALL pop the current action and push the `on_success` actions to the front of the queue for immediate processing.
11. WHEN a `GiveItem` action with direction `Take` fails due to insufficient items, THE ActionQueue SHALL pop the current action and push the `on_failure` actions to the front of the queue for immediate processing.

### Requirement 7: LearnAbility EventAction Data Model

**User Story:** As a game designer, I want to define an event action that teaches or removes an ability from a character, so that story events can unlock unique skills and special mechanics can require forgetting abilities.

#### Acceptance Criteria

1. THE EventAction enum SHALL include a `LearnAbility` variant with an `ability_id` field of type `AbilityId` and a `target` field of type `CharacterId`.
2. WHEN the `ability_id` field is deserialized, THE EventAction parser SHALL accept non-empty string values.
3. IF the `ability_id` field is an empty string during deserialization, THEN THE EventAction parser SHALL return a deserialization error indicating that ability_id must not be empty.
4. WHEN the `target` field is deserialized, THE EventAction parser SHALL accept non-empty string values.
5. IF the `target` field is an empty string during deserialization, THEN THE EventAction parser SHALL return a deserialization error indicating that target must not be empty.
6. THE EventAction `LearnAbility` variant SHALL serialize to and deserialize from JSON using the existing `#[serde(tag = "type")]` format.
7. FOR ALL valid LearnAbility EventAction values, serializing then deserializing SHALL produce an equivalent value (round-trip property).
8. THE `LearnAbility` variant SHALL include a `direction` field of type `TransferDirection` that defaults to `Give` when absent from JSON.
9. THE `LearnAbility` variant SHALL include an `on_success` field of type `Vec<EventAction>` that defaults to an empty list when absent from JSON.
10. THE `LearnAbility` variant SHALL include an `on_failure` field of type `Vec<EventAction>` that defaults to an empty list when absent from JSON.

### Requirement 8: LearnAbility Runtime Effect

**User Story:** As a player, I want characters to learn or forget abilities from story events, so that key moments unlock unique powers and special mechanics can require ability sacrifices.

#### Acceptance Criteria

1. WHEN the ActionQueue advances to a `LearnAbility` action with direction `Give`, THE Renderer SHALL add the specified `ability_id` to the target character's learned abilities list in the `CharacterProgressState` resource.
2. IF the target character already knows the specified ability and direction is `Give`, THEN THE Renderer SHALL treat the action as a no-op and advance the ActionQueue.
3. IF the `target` CharacterId does not exist in the CharacterProgressState, THEN THE Renderer SHALL log a warning and advance the ActionQueue without modifying any state.
4. IF the `ability_id` does not exist in the project's `AbilityRegistry`, THEN THE Renderer SHALL log a warning and advance the ActionQueue without modifying any state.
5. WHEN a `LearnAbility` action with direction `Give` is processed, THE ActionQueue SHALL advance immediately to the next action without blocking (non-blocking action).
6. WHEN the ActionQueue advances to a `LearnAbility` action with direction `Give`, THE Renderer SHALL ignore the `on_success` and `on_failure` fields.
7. WHEN the ActionQueue advances to a `LearnAbility` action with direction `Take`, THE Renderer SHALL check whether the target character's learned abilities list in the `CharacterProgressState` contains the specified `ability_id`.
8. WHEN a `LearnAbility` action with direction `Take` succeeds (the character knows the ability), THE Renderer SHALL remove the specified `ability_id` from the target character's learned abilities list.
9. WHEN a `LearnAbility` action with direction `Take` fails because the character does not know the specified ability, THE Renderer SHALL not modify the `CharacterProgressState`.
10. WHEN a `LearnAbility` action with direction `Take` succeeds, THE ActionQueue SHALL pop the current action and push the `on_success` actions to the front of the queue for immediate processing.
11. WHEN a `LearnAbility` action with direction `Take` fails, THE ActionQueue SHALL pop the current action and push the `on_failure` actions to the front of the queue for immediate processing.

### Requirement 9: AddPartyMember EventAction Data Model

**User Story:** As a game designer, I want to define an event action that adds or removes a character from the player's active party, so that story events can introduce new playable characters and mechanics can require dismissing party members.

#### Acceptance Criteria

1. THE EventAction enum SHALL include an `AddPartyMember` variant with a `character_id` field of type `CharacterId`.
2. WHEN the `character_id` field is deserialized, THE EventAction parser SHALL accept string values containing 1 to 64 characters inclusive.
3. IF the `character_id` field is empty or exceeds 64 characters during deserialization, THEN THE EventAction parser SHALL reject the input with a deserialization error indicating the character_id length is invalid.
4. THE EventAction `AddPartyMember` variant SHALL serialize to and deserialize from JSON using the existing `#[serde(tag = "type")]` format, producing a JSON object with a `"type"` field set to `"AddPartyMember"` and a `"character_id"` field containing the string value.
5. FOR ALL valid AddPartyMember EventAction values, serializing then deserializing SHALL produce an equivalent value (round-trip property).
6. THE `AddPartyMember` variant SHALL include a `direction` field of type `TransferDirection` that defaults to `Give` when absent from JSON.
7. THE `AddPartyMember` variant SHALL include an `on_success` field of type `Vec<EventAction>` that defaults to an empty list when absent from JSON.
8. THE `AddPartyMember` variant SHALL include an `on_failure` field of type `Vec<EventAction>` that defaults to an empty list when absent from JSON.

### Requirement 10: AddPartyMember Runtime Effect

**User Story:** As a player, I want new characters to join or leave my party during story events, so that narrative moments feel impactful and mechanics can require party member sacrifice.

#### Acceptance Criteria

1. WHEN the ActionQueue advances to an `AddPartyMember` action with direction `Give`, THE Renderer SHALL append the specified `character_id` to the end of the `PartyState` resource's active members list.
2. IF the character is already in the active party and direction is `Give`, THEN THE Renderer SHALL treat the action as a no-op and advance the ActionQueue.
3. IF the `character_id` does not exist in the project's `CharacterRegistry`, THEN THE Renderer SHALL log a warning and advance the ActionQueue without modifying the party.
4. WHEN an `AddPartyMember` action with direction `Give` is processed, THE ActionQueue SHALL advance immediately to the next action without blocking (non-blocking action).
5. THE PartyState resource SHALL be initialized with an empty active members list at renderer startup.
6. WHEN the ActionQueue advances to an `AddPartyMember` action with direction `Give`, THE Renderer SHALL ignore the `on_success` and `on_failure` fields.
7. WHEN the ActionQueue advances to an `AddPartyMember` action with direction `Take`, THE Renderer SHALL check whether the `PartyState` active members list contains the specified `character_id`.
8. WHEN an `AddPartyMember` action with direction `Take` succeeds (the character is in the party), THE Renderer SHALL remove the specified `character_id` from the `PartyState` active members list.
9. WHEN an `AddPartyMember` action with direction `Take` fails because the character is not in the active party, THE Renderer SHALL not modify the `PartyState`.
10. WHEN an `AddPartyMember` action with direction `Take` succeeds, THE ActionQueue SHALL pop the current action and push the `on_success` actions to the front of the queue for immediate processing.
11. WHEN an `AddPartyMember` action with direction `Take` fails, THE ActionQueue SHALL pop the current action and push the `on_failure` actions to the front of the queue for immediate processing.

### Requirement 11: Editor Support for Reward Event Actions

**User Story:** As a game designer, I want to configure the reward event actions in the editor's Event Trigger Editor dialog, so that I can place rewards on map tiles without editing JSON by hand.

#### Acceptance Criteria

1. THE Editor Event Trigger Editor dialog SHALL include `GiveCurrency`, `GiveExperience`, `GiveItem`, `LearnAbility`, and `AddPartyMember` as selectable action types alongside existing action options.
2. WHEN `GiveCurrency` is selected, THE Editor SHALL display a numeric input field for `amount` (default 100, range 1 to 9,999,999).
3. WHEN `GiveExperience` is selected, THE Editor SHALL display a numeric input field for `amount` (default 100, range 1 to 9,999,999) and an optional character selector for `target` where "All Party Members" is the default selection and individual characters from the project's `CharacterRegistry` are available as alternatives.
4. WHEN `GiveItem` is selected, THE Editor SHALL display a searchable item selector populated from the project's `ItemRegistry` and a numeric input field for `quantity` (default 1, range 1 to 999).
5. WHEN `LearnAbility` is selected, THE Editor SHALL display a searchable ability selector populated from the project's `AbilityRegistry` and a searchable character selector populated from the project's `CharacterRegistry` for the `target` field.
6. WHEN `AddPartyMember` is selected, THE Editor SHALL display a searchable character selector populated from the project's `CharacterRegistry`.
7. IF any required field (`amount`, `item_id`, `ability_id`, `character_id`, or `quantity`) is empty or outside its valid range, THEN THE Editor SHALL disable the Add/Update button for that action until all fields pass validation.
8. WHEN the user selects an existing reward action for editing, THE Editor SHALL populate the form fields with that action's current values and display an "Update" button in place of the "Add" button.
9. IF the `amount` field value is outside the range 1 to 9,999,999 or the `quantity` field value is outside the range 1 to 999, THEN THE Editor SHALL clamp the value to the nearest valid bound when the action is saved.
10. WHEN any reward action type is selected, THE Editor SHALL display a `TransferDirection` toggle with options "Give" (default) and "Take".
11. WHEN the direction toggle is set to "Take", THE Editor SHALL display an expandable `on_failure` action list editor that allows the designer to add nested EventAction sequences (using the same action type selector recursively).
12. WHEN the direction toggle is set to "Take", THE Editor SHALL display an optional expandable `on_success` action list editor that allows the designer to add nested EventAction sequences.
13. WHEN the direction toggle is set to "Give", THE Editor SHALL hide the `on_success` and `on_failure` action list editors.
14. IF direction is "Take" and the `on_failure` action list is empty, THEN THE Editor SHALL disable the Add/Update button and display a validation message indicating that at least one on_failure action is required.

### Requirement 12: ActionQueue Integration for Reward Actions

**User Story:** As a game designer, I want reward actions to work seamlessly in sequences with existing event actions, so that I can compose complex trigger chains combining dialog, effects, rewards, and conditional spending.

#### Acceptance Criteria

1. THE ActionQueue SHALL process `GiveCurrency`, `GiveExperience`, `GiveItem`, `LearnAbility`, and `AddPartyMember` actions in sequence order alongside all existing EventAction variants.
2. WHEN any reward action with direction `Give` is processed, THE ActionQueue SHALL advance to the next action in the same frame (non-blocking).
3. WHEN multiple consecutive non-blocking actions (including reward actions with direction `Give`) are present in the queue, THE ActionQueue SHALL process all of them within the same frame until a blocking action or the end of the queue is reached.
4. WHEN a `JumpTo` action is encountered in the queue after a reward action, THE ActionQueue SHALL clear the remaining queue and execute the map transition, consistent with existing behavior.
5. WHEN any reward action with direction `Take` is processed, THE ActionQueue SHALL pop the current action, evaluate the sufficiency check, apply the resource change if sufficient, and push the appropriate branch (`on_success` or `on_failure`) to the front of the queue.
6. WHEN a `Take` action's `on_success` or `on_failure` branch is pushed to the front of the queue, THE ActionQueue SHALL process those actions in the same sequential manner as any other queued actions (including support for nested blocking actions, branching, and further reward actions).
7. WHEN a `Take` action's `on_success` field is an empty list and the take succeeds, THE ActionQueue SHALL simply advance to the next action in the queue (equivalent to no branch).
8. WHEN a `Take` action's `on_failure` field is an empty list and the take fails, THE ActionQueue SHALL simply advance to the next action in the queue (equivalent to no branch).

### Requirement 13: Serialization Compatibility for Reward Actions

**User Story:** As a game designer, I want my existing project files to continue loading correctly after the reward actions are added, so that I do not lose any work.

#### Acceptance Criteria

1. WHEN a project file containing only pre-existing action types is loaded, THE EventAction parser SHALL deserialize all actions and produce a ProjectFile that is structurally equal to the original data without errors.
2. WHEN a project file containing the new reward action types is loaded by an older version of the toolkit that does not recognize them, THE EventAction parser SHALL return a deserialization error whose message includes the unrecognized type tag value.
3. FOR ALL valid ProjectFile values (as accepted by `ProjectFile::deserialize`) containing any combination of EventAction variants (including the new reward variants), serializing then deserializing SHALL produce a ProjectFile that is structurally equal (via `PartialEq`) to the original value (round-trip property).
4. THE serialized JSON format of all pre-existing EventAction variants (JumpTo, ShowDialog, ScreenShake, StopScreenShake, FadeTransition, SetState, SetPlayerAppearance, StateCheck, Branch, ShowSelection) SHALL remain unchanged after the new reward variants are added.
5. WHEN a reward action with direction `Give` is serialized without explicit `on_success` or `on_failure` fields, THE EventAction serializer SHALL produce JSON that omits the `on_success` and `on_failure` keys (or serializes them as empty arrays), and deserializing that JSON SHALL produce a value with empty `on_success` and `on_failure` lists.
6. WHEN a reward action with direction `Take` and populated `on_success` and `on_failure` fields is serialized, THE EventAction serializer SHALL include the nested action arrays in the JSON output, and deserializing that JSON SHALL produce a structurally equal value including all nested actions (round-trip property for nested branching).
