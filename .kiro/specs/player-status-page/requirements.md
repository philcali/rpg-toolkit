# Requirements Document

## Introduction

The Player Status Page is a read-only default scene (following the pattern established by `ShopScenePlugin` and `TitleScreenPlugin`) that allows players to inspect their party members, view character details, and browse their inventory during gameplay. It is triggered by transitioning to `AppPhase::Status` and provides a menu-driven UI rendered with Bevy's built-in UI system. This iteration is read-only; equipment management and consumable usage are deferred to a follow-up spec.

## Glossary

- **Status_Scene**: The Bevy plugin and associated systems that render the player status page when `AppPhase::Status` is active.
- **Party_Member**: A playable character in the active party, identified by a `CharacterId`.
- **Status_Mode**: The active sub-page within the Status Scene (PartyList or Inventory).
- **Character_Detail**: A drill-down view showing full information for a single party member.
- **Effective_Stat**: A character's stat value computed as `base_value + growth_value * (level - 1)`.
- **Category_Tab**: A filter for the inventory view that selects items by their `ItemCategory`.

## Requirements

### Requirement 1: Scene Lifecycle

**User Story:** As a player, I want to open and close the status page during gameplay, so that I can check my party's condition without losing game progress.

#### Acceptance Criteria

1. WHEN the game transitions to `AppPhase::Status`, THE Status_Scene SHALL spawn its UI hierarchy and internal state resources (including `StatusMode`, selection indices for each sub-page, and any local marker components).
2. WHEN the player presses the Escape key while on a top-level sub-page (Party List or Inventory with no detail view active), THE Status_Scene SHALL transition the game back to `AppPhase::InGame`.
3. WHEN the game transitions out of `AppPhase::Status`, THE Status_Scene SHALL despawn all entities marked with the `StatusSceneMarker` component and remove internal state resources, leaving zero orphaned entities.
4. THE Status_Scene SHALL attach a `StatusSceneMarker` component to every UI entity it spawns so that the exit system can query and despawn them in a single pass.

### Requirement 2: Party List Sub-Page

**User Story:** As a player, I want to see all my party members at a glance with their portraits, so that I can quickly assess my team.

#### Acceptance Criteria

1. WHEN the Status Scene spawns, THE Status_Scene SHALL display the Party List sub-page as the default view with the first party member row selected.
2. THE Status_Scene SHALL display each active party member from `PartyState` as a row in the order they appear in the `PartyState.members` list, containing the character's face portrait, display name, current level (the character's Level stat base_value from `CharacterProgressState`), and effective HP value computed using the formula `base_value + growth_value * (level - 1)`.
3. IF a party member has a `face_portrait` visual asset configured in the `CharacterRegistry`, THEN THE Status_Scene SHALL render that portrait graphic in the party member's row.
4. IF a party member has no `face_portrait` configured, THEN THE Status_Scene SHALL display a placeholder indicator in the portrait area.
5. IF a `CharacterId` in `PartyState` cannot be resolved via `CharacterRegistryRes`, THEN THE Status_Scene SHALL skip that member and not display a row for it.
6. IF `PartyState` contains zero resolvable party members, THEN THE Status_Scene SHALL display an empty party indicator instead of the member list.
7. THE Status_Scene SHALL highlight the currently selected party member row with a visually distinct background color differentiating it from unselected rows.
8. WHEN the player presses Enter or Space on a selected party member, THE Status_Scene SHALL navigate to the Character Detail view for that member.
9. IF `PartyState.members` contains more than 4 resolvable party members, THEN THE Status_Scene SHALL display only the first 4 members in list order, discarding the remainder from the party list view.

### Requirement 3: Character Detail View

**User Story:** As a player, I want to inspect a single character's full stats and abilities in a booklet layout, so that I can understand their strengths.

#### Acceptance Criteria

1. WHEN the player enters the Character Detail view, THE Status_Scene SHALL display the selected character's face portrait on the left side of the screen.
2. IF the selected character has no `face_portrait` visual asset configured, THEN THE Status_Scene SHALL display a placeholder indicator in the portrait area of the Character Detail view.
3. WHILE the Character Detail view is active, THE Status_Scene SHALL display the character's display name and current level on the right side, where level is read from the character's "Level" stat effective value in `CharacterProgressState`.
4. WHILE the Character Detail view is active, THE Status_Scene SHALL display all of the character's stats (excluding the "Level" stat row) with their effective values computed using the formula `base_value + growth_value * (level - 1)`, listed in the order they appear in the character's `stats` vector.
5. WHILE the Character Detail view is active, THE Status_Scene SHALL display the character's equipped items (from `starting_equipment`) listed by item display name resolved via `ItemRegistryRes`, in the order they appear in the `starting_equipment` vector.
6. IF an item ID in `starting_equipment` cannot be resolved via `ItemRegistryRes`, THEN THE Status_Scene SHALL omit that entry from the equipped items list.
7. WHILE the Character Detail view is active, THE Status_Scene SHALL display the character's learned abilities from `CharacterProgressState` resolved to display names via `AbilityRegistryRes`, listed in the order they appear in the learned abilities list.
8. IF an ability ID in the learned abilities list cannot be resolved via `AbilityRegistryRes`, THEN THE Status_Scene SHALL omit that entry from the displayed abilities list.
9. WHEN the player presses Escape in the Character Detail view, THE Status_Scene SHALL return to the Party List sub-page with the same member still selected.

### Requirement 4: Inventory Browser Sub-Page

**User Story:** As a player, I want to browse my full inventory organized by category, so that I can see what items I have.

#### Acceptance Criteria

1. WHEN the player navigates to the Inventory sub-page, THE Status_Scene SHALL display the player's inventory items from `InventoryState`, resolving each item's data via `ItemRegistryRes`.
2. THE Status_Scene SHALL organize items into category tabs in this fixed order: Weapon, Armor, Accessory, Consumable, KeyItem, with the Weapon tab selected by default on entry.
3. THE Status_Scene SHALL display each item's icon graphic (from `EntityGraphics`), display name, and quantity held, sorted case-insensitively by display name within each tab.
4. IF an item has no icon graphic configured, THEN THE Status_Scene SHALL display a placeholder indicator in the icon area.
5. WHEN the player highlights an inventory item, THE Status_Scene SHALL display that item's description and stat modifiers (each formatted with a sign prefix, e.g. "+5" or "-3") in a detail panel.
6. WHILE the Inventory sub-page is active, THE Status_Scene SHALL allow the player to switch between category tabs using Left/Right input (overriding the sub-page switching behavior defined in Requirement 5).
7. WHEN a category tab contains no items, THE Status_Scene SHALL display an empty category indicator and prevent vertical list navigation until the player switches to a tab that contains items.
8. IF an item_id present in `InventoryState` cannot be resolved via `ItemRegistryRes`, THEN THE Status_Scene SHALL omit that item from the displayed list.

### Requirement 5: Navigation and Input

**User Story:** As a player, I want consistent keyboard controls across all status page views, so that navigation feels natural and familiar from the shop screen.

#### Acceptance Criteria

1. THE Status_Scene SHALL support Up arrow and W keys for moving the selection index one position upward, and Down arrow and S keys for moving the selection index one position downward, within any visible list.
2. THE Status_Scene SHALL support Left arrow and A keys for switching to the previous top-level sub-page, and Right arrow and D keys for switching to the next top-level sub-page (Party List and Inventory).
3. WHEN the player presses Enter or Space while a list item is focused, THE Status_Scene SHALL activate that item's detail view (character detail from party list, or item detail from inventory).
4. WHEN the player presses Escape or Backspace while in a detail view, THE Status_Scene SHALL return to the parent list view with the previously focused item still selected.
5. WHEN the player presses Escape or Backspace while at the top-level list view (no detail view active), THE Status_Scene SHALL exit the status scene and return to the previous AppPhase.
6. WHEN the player navigates beyond the first or last item in a list, THE Status_Scene SHALL clamp the selection index to the nearest valid bound (0 for upward, list length minus 1 for downward) without wrapping.
7. WHEN the player switches sub-pages via Left/Right or A/D, THE Status_Scene SHALL preserve each sub-page's selection index independently so that returning to a previously visited sub-page restores the last focused item.
8. WHILE a list in the active sub-page contains zero items, THE Status_Scene SHALL accept sub-page switching and back navigation input but SHALL NOT process Up/Down or confirm input.

### Requirement 6: Plugin Architecture

**User Story:** As a game developer using this toolkit, I want the status page to follow the same plugin pattern as the shop scene, so that it integrates cleanly into the existing architecture.

#### Acceptance Criteria

1. THE Status_Scene SHALL be implemented as a Bevy `Plugin` struct named `StatusScenePlugin` in the `rpg-toolkit-scenes` crate and exported from the crate's `lib.rs`.
2. THE Status_Scene SHALL register systems using `OnEnter(AppPhase::Status)`, `OnExit(AppPhase::Status)`, and `Update` with `run_if(in_state(AppPhase::Status))`.
3. THE Status_Scene SHALL use the existing shared resources (`InventoryState`, `PartyState`, `CharacterProgressState`, `GameState`) without redefining those struct types.
4. THE Status_Scene SHALL require a `CharacterRegistryRes` resource (wrapping `CharacterRegistry`) and an `ItemRegistryRes` resource to resolve character and item data. IF either resource is not present when the scene enters, THEN THE Status_Scene SHALL log a warning and skip UI spawning without panicking.
5. THE Status_Scene SHALL require an `AbilityRegistryRes` resource (wrapping `AbilityRegistry`) to resolve ability display names. IF the resource is not present when the scene enters, THEN THE Status_Scene SHALL log a warning and skip UI spawning without panicking.
6. WHEN the state exits `AppPhase::Status`, THE Status_Scene SHALL despawn all UI entities it spawned by using a dedicated marker component, leaving no orphaned entities.
