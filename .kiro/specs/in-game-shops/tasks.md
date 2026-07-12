# Implementation Plan: In-Game Shops

## Overview

This implementation adds a complete in-game shop system across four crates: data model in `rpg-toolkit-common`, editor UI in `rpg-toolkit-editor`, runtime scene in `rpg-toolkit-scenes`, and event integration in `rpg-toolkit-renderer`. Tasks are ordered so that foundational data models come first, followed by transaction logic, editor UI, scene plugin, and finally integration wiring.

## Tasks

- [ ] 1. Implement shop data model and error types
  - [x] 1.1 Add ShopValidationError variant to CommonError and create shop module with core types
    - Add `ShopValidationError(String)` variant to `CommonError` in `crates/rpg-toolkit-common/src/error.rs`
    - Create `crates/rpg-toolkit-common/src/shop.rs` with `ShopId`, `ShopEntry`, `ShopDefinition`, `ShopRegistry` structs
    - Add `pub mod shop;` to `crates/rpg-toolkit-common/src/lib.rs` and re-export types
    - _Requirements: 1.1, 1.3, 1.4_

  - [x] 1.2 Implement ShopRegistry CRUD methods
    - Implement `create_shop(name: &str) -> Result<ShopId, CommonError>` with UUID generation and name validation (1–64 trimmed chars)
    - Implement `delete_shop(id: &ShopId) -> Result<(), CommonError>`
    - Implement `rename_shop(id: &ShopId, name: &str) -> Result<(), CommonError>` with validation
    - Implement `add_entry(shop_id: &ShopId, entry: ShopEntry) -> Result<(), CommonError>` rejecting duplicates and enforcing max 256 entries
    - Implement `remove_entry(shop_id: &ShopId, item_id: &ItemId) -> Result<(), CommonError>`
    - Implement `update_entry(shop_id: &ShopId, item_id: &ItemId, ...) -> Result<(), CommonError>`
    - Implement `sorted_shops() -> Vec<&ShopDefinition>` with case-insensitive sort
    - Implement `search_shops(query: &str) -> Vec<&ShopDefinition>` with substring filter
    - _Requirements: 1.1, 1.2, 1.3, 1.7, 2.2_

  - [x] 1.3 Write property tests for shop validation and sorting (Properties 1, 2, 4)
    - Create `tests/properties/shop_invariants.rs`
    - **Property 1: Shop name validation** — For any string input, name validator accepts iff trimmed length 1–64
    - **Validates: Requirements 1.3, 2.10**
    - **Property 2: No duplicate items per shop** — add_entry sequence never produces duplicate ItemIds
    - **Validates: Requirements 1.7, 2.9**
    - **Property 4: Shop list case-insensitive sorting** — sorted output is non-decreasing case-insensitively
    - **Validates: Requirements 2.2**

  - [x] 1.4 Integrate ShopRegistry into ProjectFile and SaveFile
    - Add `#[serde(default)] pub shops: ShopRegistry` field to `ProjectFile` in `crates/rpg-toolkit-common/src/project.rs`
    - Add shop ID validation to `ProjectFile::deserialize()` (return `ProjectValidationError` on mismatch, warn on missing item references)
    - Add `#[serde(default)] pub shop_stock: BTreeMap<String, BTreeMap<String, u32>>` field to `SaveFile` in `crates/rpg-toolkit-common/src/save.rs`
    - Update `ProjectManifest` to include `shops: ShopRegistry` field
    - _Requirements: 8.1, 8.2, 8.3, 8.4, 8.5, 9.1, 9.5_

  - [x] 1.5 Write property tests for serialization round-trips (Properties 5, 13, 14, 15)
    - Create `tests/properties/shop_round_trip.rs`
    - **Property 5: OpenShop action serialization round-trip** — non-empty shop ID survives JSON round-trip, empty rejected
    - **Validates: Requirements 3.1**
    - **Property 13: Shop registry serialization round-trip** — valid ShopRegistry in ProjectFile survives round-trip
    - **Validates: Requirements 8.1, 8.3**
    - **Property 14: Shop ID mismatch validation** — mismatched ID/key returns ProjectValidationError
    - **Validates: Requirements 8.2**
    - **Property 15: Shop stock persistence round-trip** — shop_stock map in SaveFile survives round-trip
    - **Validates: Requirements 9.1, 9.2, 9.5**

- [x] 2. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [x] 3. Implement EventAction::OpenShop variant
  - [x] 3.1 Add OpenShop variant to EventAction enum
    - Add `OpenShop { #[serde(deserialize_with = "deserialize_non_empty_string")] shop_id: ShopId }` variant to `EventAction` enum in `crates/rpg-toolkit-common/src/map.rs`
    - Ensure the `deserialize_non_empty_string` helper rejects empty strings during deserialization
    - _Requirements: 3.1_

  - [x] 3.2 Handle OpenShop action in the renderer event processing
    - In the renderer's event action handler (in `crates/rpg-toolkit-renderer`), add a match arm for `EventAction::OpenShop { shop_id }`
    - Validate shop_id exists in `ShopRegistry`; if not, log warning and skip
    - If valid, insert `ActiveShopId` resource and transition `AppPhase` to `Shop`
    - _Requirements: 3.2, 3.3_

- [ ] 4. Implement shop transaction logic as pure functions
  - [x] 4.1 Create shop_scene module with transaction types and pure functions
    - Create `crates/rpg-toolkit-scenes/src/shop_scene.rs`
    - Define `BuyResult`, `SellResult`, `ShopError` types
    - Implement `compute_sell_price(entry, item) -> u32` (sell_price or item.value / 2)
    - Implement `max_buy_quantity(balance, buy_price, remaining_stock, is_stackable, stack_limit, currently_held) -> u32`
    - Implement `execute_buy(balance, inventory_qty, buy_price, quantity, remaining_stock, is_stackable, stack_limit) -> Result<BuyResult, ShopError>`
    - Implement `execute_sell(balance, inventory_qty, sell_price, quantity) -> Result<SellResult, ShopError>`
    - Implement `visible_entries(entries, flags, item_registry) -> Vec<&ShopEntry>`
    - Implement `sellable_items(inventory, item_registry, shop_entries) -> Vec<(ItemId, u32, u32)>`
    - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5, 5.6, 6.1, 6.2, 6.3, 6.4, 6.5, 7.1, 7.2, 7.3, 7.4_

  - [ ]* 4.2 Write property tests for buy transaction correctness (Property 6)
    - Create `tests/properties/shop_transactions.rs`
    - **Property 6: Buy transaction correctness** — new_balance = old - (price × qty), new_inventory = old + qty, new_stock = old - qty
    - **Validates: Requirements 5.1, 5.5**

  - [x] 4.3 Write property test for purchase rejection preserving state (Property 7)
    - **Property 7: Purchase rejection preserves state** — violated guards produce error with unchanged state
    - **Validates: Requirements 5.2, 5.3, 5.4, 4.6**

  - [ ]* 4.4 Write property test for max buy quantity computation (Property 8)
    - **Property 8: Maximum buy quantity computation** — result equals min(floor(balance/price), stock_or_max, stack_space)
    - **Validates: Requirements 5.6**

  - [ ]* 4.5 Write property test for sell transaction correctness (Property 9)
    - **Property 9: Sell transaction correctness** — new_balance = old + (sell_price × qty) saturating, new_inventory = old - qty
    - **Validates: Requirements 6.1**

  - [ ]* 4.6 Write property test for sell rejection preserving state (Property 10)
    - **Property 10: Sell rejection preserves state** — insufficient inventory produces error with unchanged state
    - **Validates: Requirements 6.2**

  - [x] 4.7 Write property tests for filtering logic (Properties 3, 11, 12, 16)
    - **Property 3: Default sell price calculation** — no explicit price → item.value / 2
    - **Validates: Requirements 1.5, 6.3**
    - **Property 11: Sell list filtering** — only items with qty > 0, not KeyItem, sell_price > 0
    - **Validates: Requirements 6.5**
    - **Property 12: Condition-based item visibility** — entries visible iff condition is None, empty checks, or evaluates true
    - **Validates: Requirements 7.1, 7.2, 7.3, 7.4**
    - **Property 16: Stock value clamping on load** — saved value exceeding limit is clamped; values within limit preserved
    - **Validates: Requirements 9.4**

- [x] 5. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [x] 6. Implement ShopScenePlugin with Bevy ECS
  - [x] 6.1 Implement ShopScenePlugin struct and lifecycle systems
    - Define `ActiveShopId` and `ShopStockState` resources
    - Implement `ShopScenePlugin` with `OnEnter(AppPhase::Shop)` / `OnExit(AppPhase::Shop)` / `Update` systems
    - Implement `spawn_shop_ui` system: load shop definition, evaluate conditions for visible entries, initialize stock state from SaveFile (clamping per Property 16), spawn UI entities
    - Implement `despawn_shop_ui` system: despawn all shop UI entities, persist stock to SaveFile
    - Implement `shop_input` system: handle buy/sell selection, quantity adjustment, cancel/back input to transition back to `AppPhase::InGame`
    - Display "No items available" when no visible items; display "Sold Out" for zero-stock items
    - Register the plugin in `crates/rpg-toolkit-scenes/src/lib.rs`
    - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5, 4.6, 4.7, 4.8, 5.1, 5.2, 5.3, 5.4, 5.5, 5.6, 6.1, 6.2, 6.3, 6.4, 6.5, 7.1, 7.2, 9.2, 9.3, 9.4_

  - [ ]* 6.2 Write unit tests for ShopScenePlugin stock initialization and clamping
    - Test stock restored from save data
    - Test stock clamped when saved value exceeds configured limit
    - Test stock defaults to full when no save data exists
    - _Requirements: 9.2, 9.3, 9.4_

- [x] 7. Implement ShopPanelPlugin editor UI
  - [x] 7.1 Create ShopPanelPlugin with shop list and CRUD operations
    - Create `crates/rpg-toolkit-editor/src/plugins/shop_panel.rs`
    - Add `Shop` variant to `AppEditorMode` enum
    - Implement left panel: scrollable shop list sorted case-insensitively, search field, create/delete buttons
    - Implement create shop action with default "New Shop" name
    - Implement delete with confirmation dialog
    - Implement shop selection and name editing with validation (1–64 chars)
    - Register plugin in `crates/rpg-toolkit-editor/src/main.rs` (or plugin registration file)
    - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.8, 2.10, 2.11_

  - [x] 7.2 Implement shop entry editing in the central panel
    - Display entry list with item name, buy price, sell price (or "Auto"), stock limit (or "Unlimited")
    - Implement add entry with searchable item selector from ItemRegistry
    - Validate buy price as u32 (0–4,294,967,295), stock limit as 1–9999 or cleared
    - Display validation errors for invalid input
    - Reject duplicate item entries with error message
    - Implement remove entry (immediate, no confirmation)
    - _Requirements: 2.4, 2.5, 2.6, 2.7, 2.9, 2.11_

  - [x] 7.3 Implement condition editor for shop entries
    - Add condition editor per ShopEntry allowing add/remove/configure ConditionCheck entries
    - Support key, operator (Equals, NotEquals, Exists, NotExists), optional value fields
    - Support logic mode selector (All/Any)
    - Enforce max 16 condition checks per entry
    - _Requirements: 7.5_

  - [x] 7.4 Add OpenShop to the action editor UI
    - Add OpenShop as a selectable action type in the attribute/action_editor UI
    - Implement searchable shop selector populated from ShopRegistry
    - Disable OpenShop option with tooltip when no shops exist
    - _Requirements: 3.4, 3.5_

- [x] 8. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [x] 9. Integration wiring and final validation
  - [x] 9.1 Wire ShopScenePlugin into the game application
    - Add `ShopScenePlugin` to the app's plugin group in the game launcher/renderer
    - Ensure `AppPhase::Shop` state exists in the `AppPhase` enum (or add it if missing)
    - Wire `ActiveShopId` resource insertion in the renderer's OpenShop handler (connects task 3.2 to task 6.1)
    - Verify save/load flow persists and restores shop_stock correctly
    - _Requirements: 3.2, 4.1, 4.2, 9.1, 9.2, 9.3_

  - [ ]* 9.2 Write integration tests for end-to-end shop flows
    - Test OpenShop event triggers AppPhase::Shop transition
    - Test buy transaction updates currency and inventory resources
    - Test sell transaction updates currency and inventory resources
    - Test save/load round-trip with shop stock data
    - _Requirements: 3.2, 5.1, 6.1, 9.1_

- [x] 10. Final checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

## Notes

- Tasks marked with `*` are optional and can be skipped for faster MVP
- Each task references specific requirements for traceability
- Checkpoints ensure incremental validation
- Property tests validate universal correctness properties from the design document
- Unit tests validate specific examples and edge cases
- The project uses `proptest` as a workspace dependency; property tests live in `tests/properties/`
- All transaction logic is implemented as pure functions for testability before being wired into Bevy ECS systems

## Task Dependency Graph

```json
{
  "waves": [
    { "id": 0, "tasks": ["1.1"] },
    { "id": 1, "tasks": ["1.2"] },
    { "id": 2, "tasks": ["1.3", "1.4"] },
    { "id": 3, "tasks": ["1.5", "3.1"] },
    { "id": 4, "tasks": ["3.2", "4.1"] },
    { "id": 5, "tasks": ["4.2", "4.3", "4.4", "4.5", "4.6", "4.7"] },
    { "id": 6, "tasks": ["6.1", "7.1"] },
    { "id": 7, "tasks": ["6.2", "7.2"] },
    { "id": 8, "tasks": ["7.3", "7.4"] },
    { "id": 9, "tasks": ["9.1"] },
    { "id": 10, "tasks": ["9.2"] }
  ]
}
```
