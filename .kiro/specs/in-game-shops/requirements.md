# Requirements Document

## Introduction

This feature adds an in-game shop system to the RPG toolkit. Shops allow game designers to define purchasable inventories of items in the editor, and players to browse and buy/sell items during gameplay via a dedicated Shop scene. The data model lives in `rpg-toolkit-common`, authoring UI lives in `rpg-toolkit-editor`, the Shop scene plugin lives in `rpg-toolkit-scenes` (alongside `TitleScreenPlugin`), and the `EventAction::OpenShop` trigger is processed in `rpg-toolkit-renderer`. The existing `AppPhase::Shop` state drives the scene lifecycle.

## Glossary

- **Shop_Registry**: A project-level collection of all shop definitions, stored in `ProjectFile` alongside other registries (items, abilities, enemies).
- **Shop_Definition**: A named shop entity with a unique ID, display name, and a list of shop inventory entries.
- **Shop_Entry**: A single entry in a shop's inventory, referencing an item by ID with a price override, optional stock limit, and optional availability condition.
- **Shop_Panel**: The editor UI panel for authoring and managing shop definitions, activated via a dedicated `AppEditorMode::Shop` mode (alongside Map, Character, Item, Ability, Enemy modes).
- **Shop_Scene**: The scene plugin in `rpg-toolkit-scenes` (parallel to `TitleScreenPlugin`) that displays a shop's inventory to the player and handles buy/sell interactions. Activated via `AppPhase::Shop`.
- **Currency_State**: The runtime resource tracking the player's current currency balance (already exists as `CurrencyState`).
- **Inventory_State**: The runtime resource tracking the player's held items and quantities (already exists as `InventoryState`).
- **Item_Registry**: The project-level collection of item definitions (already exists as `ItemRegistry`).
- **Condition_Check**: An existing mechanism for evaluating game state flags with operators (Equals, NotEquals, Exists, NotExists).
- **OpenShop_Action**: A new `EventAction` variant that transitions the game to `AppPhase::Shop` with a reference to the target shop.

## Requirements

### Requirement 1: Shop Data Model

**User Story:** As a game designer, I want to define shops with inventories of items, so that I can create varied merchants for different areas of the game.

#### Acceptance Criteria

1. THE Shop_Registry SHALL store Shop_Definition entries in a HashMap keyed by a unique shop ID (UUID v4 string).
2. WHEN a Shop_Definition is created, THE Shop_Registry SHALL assign a UUID v4 string as the shop ID.
3. THE Shop_Definition SHALL contain a display name (1–64 non-whitespace characters after trimming), a list of Shop_Entry items (maximum 256 entries), and a unique shop ID.
4. THE Shop_Entry SHALL reference an item by ItemId, specify a buy price as an unsigned 32-bit integer, specify an optional sell price as an unsigned 32-bit integer, specify an optional stock limit as an unsigned 32-bit integer in range 1–9999, and specify an optional availability condition as a BranchCondition.
5. WHEN a Shop_Entry has no explicit sell price, THE Shop_Scene SHALL calculate the sell price as half the item's base value (rounded down via integer division).
6. WHEN a Shop_Entry has no stock limit, THE Shop_Scene SHALL treat the item as having unlimited stock.
7. THE Shop_Registry SHALL enforce that each Shop_Definition contains no duplicate ItemId references within its entry list; attempting to insert a duplicate SHALL return an error without modifying the existing entries.
8. IF a Shop_Entry references an ItemId that does not exist in the Item_Registry at runtime, THEN THE Shop_Scene SHALL skip that entry and log a warning identifying the missing ItemId.

### Requirement 2: Shop Editor Mode

**User Story:** As a game designer, I want a dedicated editor mode for shops (like the Item, Ability, and Enemy modes), so that I can author any kind of transactional encounter—merchants, armories, scroll vendors, mercenary recruiters, or barter exchanges.

#### Acceptance Criteria

1. THE AppEditorMode enum SHALL include a Shop variant, selectable from the Mode menu alongside Map, Character, Item, Ability, and Enemy.
2. WHEN the Shop mode is active, THE Shop_Panel SHALL display a scrollable list of all Shop_Definition entries sorted case-insensitively by display name.
3. WHEN the game designer clicks "Create Shop", THE Shop_Panel SHALL create a new Shop_Definition with the display name "New Shop" and an empty entry list, and select it for editing.
4. WHEN a Shop_Definition is selected, THE Shop_Panel SHALL display an editable display name field, and its list of Shop_Entry items showing item name, buy price, sell price (or "Auto" when no explicit sell price is set), and stock limit (or "Unlimited" when no stock limit is set).
5. WHEN the game designer adds a Shop_Entry, THE Shop_Panel SHALL present a searchable list of items from the Item_Registry for selection.
6. WHEN the game designer sets a buy price on a Shop_Entry, THE Shop_Panel SHALL validate that the value is an unsigned 32-bit integer (0 to 4,294,967,295) and display a validation error indicating the allowed range if the input is invalid.
7. WHEN the game designer sets a stock limit on a Shop_Entry, THE Shop_Panel SHALL validate that the value is in range 1–9999 or cleared for unlimited, and display a validation error indicating the allowed range if the input is invalid.
8. WHEN the game designer deletes a Shop_Definition, THE Shop_Panel SHALL prompt for confirmation before removal.
9. WHEN the game designer adds a Shop_Entry referencing an ItemId already present in the shop, THE Shop_Panel SHALL display a validation error and reject the duplicate.
10. WHEN the game designer edits the display name of a Shop_Definition, THE Shop_Panel SHALL validate that the trimmed name is between 1 and 64 characters and display a validation error if it is outside that range.
11. WHEN the game designer removes a Shop_Entry from a Shop_Definition, THE Shop_Panel SHALL remove the entry from the shop's entry list immediately without confirmation.

### Requirement 3: Shop Trigger via EventAction

**User Story:** As a game designer, I want to trigger a shop from NPC interactions or tile events, so that players can access shops through natural in-game encounters.

#### Acceptance Criteria

1. THE EventAction enum SHALL include an OpenShop variant containing a shop ID string field that must be a non-empty string.
2. WHEN an OpenShop action is executed, THE Renderer SHALL transition the AppPhase to Shop and insert a resource containing the referenced shop ID for the Shop_Scene plugin to consume.
3. IF an OpenShop action references a shop ID not present in the Shop_Registry, THEN THE Renderer SHALL log a warning and skip the action without crashing or transitioning AppPhase.
4. THE Editor action editor UI SHALL include OpenShop as a selectable action type with a searchable shop selector populated from the Shop_Registry.
5. WHEN no Shop_Definitions exist in the Shop_Registry, THE Editor action editor UI SHALL disable the OpenShop action type and display a tooltip indicating that at least one shop must be created first.

### Requirement 4: Shop Scene Plugin

**User Story:** As a player, I want to browse a shop's inventory and see item details, so that I can make informed purchasing decisions.

#### Acceptance Criteria

1. THE Shop_Scene SHALL be implemented as a Bevy Plugin in the `rpg-toolkit-scenes` crate, following the same pattern as TitleScreenPlugin.
2. THE Shop_Scene SHALL register systems on `OnEnter(AppPhase::Shop)` for spawning UI, `OnExit(AppPhase::Shop)` for despawning UI, and `Update` with `run_if(in_state(AppPhase::Shop))` for input handling.
3. WHEN AppPhase transitions to Shop, THE Shop_Scene SHALL display the shop's display name, the player's current currency balance, and the list of available items with their names, buy prices, and remaining stock (or "∞" for unlimited stock).
4. WHEN the player selects an item in the Shop_Scene, THE Shop_Scene SHALL display the item's description, category, rarity, and stat modifiers from the Item_Registry.
5. WHILE the Shop_Scene is active, THE Shop_Scene SHALL update the displayed currency balance within the same frame as the transaction that modified it.
6. WHEN a Shop_Entry has a stock limit and the remaining stock reaches zero, THE Shop_Scene SHALL display the item as "Sold Out" with a visual dimming indicator and prevent further purchases of that item.
7. WHEN the player presses the cancel/back input, THE Shop_Scene SHALL transition the AppPhase back to InGame and despawn all Shop_Scene UI entities.
8. IF the Shop_Scene is opened with a shop ID that has no visible items (all entries filtered by conditions or missing from Item_Registry), THEN THE Shop_Scene SHALL display "No items available" and allow the player to exit.

### Requirement 5: Buy Transaction

**User Story:** As a player, I want to buy items from a shop, so that I can acquire equipment and consumables for my adventure.

#### Acceptance Criteria

1. WHEN the player confirms a purchase, THE Shop_Scene SHALL deduct the total cost (buy price multiplied by the selected quantity) from the Currency_State and add the item and quantity to the Inventory_State.
2. IF the player's currency balance is less than the total cost (buy price multiplied by the selected quantity), THEN THE Shop_Scene SHALL display an insufficient funds message and reject the transaction.
3. IF the item is stackable and the player's current stack plus purchase quantity would exceed the item's stack limit, THEN THE Shop_Scene SHALL display an inventory full message and reject the transaction.
4. IF the item is non-stackable and the player already holds the item in the Inventory_State, THEN THE Shop_Scene SHALL display an inventory full message and reject the transaction.
5. WHEN a purchase succeeds and the Shop_Entry has a stock limit, THE Shop_Scene SHALL decrement the remaining stock by the purchased quantity.
6. WHILE an item is selected in the buy view, THE Shop_Scene SHALL allow the player to select a purchase quantity between 1 and the maximum affordable amount, where maximum affordable amount is the floor of currency balance divided by buy price, capped by remaining stock if the Shop_Entry has a stock limit, and capped by the item's available stack space (stack limit minus current held quantity) if the item is stackable, or fixed at 1 if the item is non-stackable.

### Requirement 6: Sell Transaction

**User Story:** As a player, I want to sell items from my inventory to a shop, so that I can earn currency for items I no longer need.

#### Acceptance Criteria

1. WHEN the player confirms a sale, THE Shop_Scene SHALL add the sell price multiplied by quantity to the Currency_State balance using saturating addition (capping at u64::MAX) and remove the item quantity from the Inventory_State.
2. IF the player does not hold the item or holds fewer than the requested sell quantity, THEN THE Shop_Scene SHALL display a message indicating insufficient inventory and reject the transaction without modifying Currency_State or Inventory_State.
3. THE Shop_Scene SHALL calculate the sell price for each item as the Shop_Entry sell price override if defined, or half the item's base value (from Item.value) rounded down via integer division otherwise.
4. THE Shop_Scene SHALL allow the player to select a sell quantity between 1 and the number of that item currently held in inventory.
5. THE Shop_Scene SHALL display a "Sell" tab or mode that lists items from the player's Inventory_State with their computed sell prices, excluding items whose category is KeyItem and excluding items whose computed sell price is 0.

### Requirement 7: Conditional Shop Entry Availability

**User Story:** As a game designer, I want to gate specific shop items behind game state conditions, so that I can unlock powerful items as the story progresses.

#### Acceptance Criteria

1. WHEN the Shop_Scene is opened or the player returns to the shop item list, THE Shop_Scene SHALL evaluate each Shop_Entry's BranchCondition against the current GameState flags (the saved state key-value pairs).
2. WHEN a Shop_Entry's availability condition evaluates to false, THE Shop_Scene SHALL omit the item entirely from the displayed inventory list so that the player cannot see or interact with it.
3. WHEN a Shop_Entry has no availability condition, THE Shop_Scene SHALL display the item unconditionally.
4. WHEN a Shop_Entry has an availability condition with an empty checks list, THE Shop_Scene SHALL treat the condition as true and display the item unconditionally.
5. THE Shop_Panel SHALL provide a condition editor for each Shop_Entry allowing the game designer to add (up to 16 entries), remove, and configure ConditionCheck entries specifying key, operator (Equals, NotEquals, Exists, NotExists), and optional comparison value, with a selectable logic mode (All/Any).

### Requirement 8: Shop Data Serialization

**User Story:** As a game designer, I want shop definitions to persist with my project, so that I can save and reload my work reliably.

#### Acceptance Criteria

1. THE ProjectFile SHALL include a Shop_Registry field that serializes to and deserializes from JSON, defaulting to an empty Shop_Registry when the field is absent in the input.
2. WHEN a ProjectFile is deserialized and a Shop_Definition's ID does not match its registry key, THEN THE deserializer SHALL return a ProjectValidationError indicating the mismatched key and ID values.
3. THE ProjectFile SHALL serialize and deserialize a Shop_Registry such that the deserialized value is structurally equal (all fields identical) to the original value.
4. WHEN a ProjectFile is deserialized and a Shop_Entry references an ItemId not present in the Item_Registry, THEN THE deserializer SHALL log a warning identifying the missing ItemId and the containing shop ID, and continue deserialization successfully.
5. IF the Shop_Registry JSON contains duplicate shop ID keys, THEN THE deserializer SHALL apply last-wins semantics without returning an error.

### Requirement 9: Shop Stock Persistence

**User Story:** As a player, I want shop stock levels to persist across save/load, so that limited items remain sold out if I already purchased them.

#### Acceptance Criteria

1. WHEN the player saves the game, THE SaveFile SHALL include a shop_stock map keyed by shop_id, where each value is a map of item_id to remaining stock (u32), including only entries that have a configured stock limit.
2. WHEN a save file is loaded, THE Shop_Scene SHALL restore remaining stock values from the save data by looking up the current shop_id and each item_id in the shop_stock map.
3. WHEN a save file does not contain stock data for a shop entry (shop_id or item_id not present in shop_stock), THE Shop_Scene SHALL treat the stock as its original configured limit (full restock).
4. IF the save file contains a remaining stock value that exceeds the configured stock limit for a Shop_Entry, THEN THE Shop_Scene SHALL clamp the restored value to the configured stock limit.
5. THE SaveFile SHALL serialize and deserialize the shop_stock map such that the deserialized value is structurally equal to the original value (round-trip property).
