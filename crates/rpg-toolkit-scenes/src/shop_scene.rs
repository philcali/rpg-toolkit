use std::collections::HashMap;

use bevy::prelude::*;
use rpg_toolkit_common::AppPhase;
use rpg_toolkit_common::item::{Item, ItemCategory, ItemId, ItemRegistry};
use rpg_toolkit_common::save::SaveFile;
use rpg_toolkit_common::shop::{ShopEntry, ShopRegistry};

use crate::title_screen::{CurrencyState, GameState, InventoryState, TitleScreenConfig};

/// Result of a successful buy transaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuyResult {
    pub new_balance: u64,
    pub new_inventory_qty: u32,
    pub new_stock: Option<u32>,
}

/// Result of a successful sell transaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SellResult {
    pub new_balance: u64,
    pub new_inventory_qty: u32,
}

/// Errors that can occur during shop transactions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShopError {
    InsufficientFunds,
    InventoryFull,
    InsufficientStock,
    InsufficientInventory,
}

/// Computes the effective sell price for an item.
///
/// Returns `entry.sell_price` if set, otherwise `item.value / 2` (integer division).
pub fn compute_sell_price(entry: &ShopEntry, item: &Item) -> u32 {
    entry.sell_price.unwrap_or(item.value / 2)
}

/// Computes the maximum purchasable quantity given constraints.
///
/// The result is the minimum of:
/// - Affordability: `floor(balance / buy_price)` (or `u32::MAX` if `buy_price` is 0)
/// - Stock: `remaining_stock` (or `u32::MAX` if unlimited)
/// - Stack space: `(stack_limit - currently_held)` if stackable,
///   or `(1 - min(currently_held, 1))` if non-stackable
pub fn max_buy_quantity(
    balance: u64,
    buy_price: u32,
    remaining_stock: Option<u32>,
    is_stackable: bool,
    stack_limit: u32,
    currently_held: u32,
) -> u32 {
    let affordable = if buy_price == 0 {
        u32::MAX
    } else {
        let max_afford = balance / buy_price as u64;
        if max_afford > u32::MAX as u64 {
            u32::MAX
        } else {
            max_afford as u32
        }
    };

    let stock_available = remaining_stock.unwrap_or(u32::MAX);

    let stack_space = if is_stackable {
        stack_limit.saturating_sub(currently_held)
    } else {
        1u32.saturating_sub(currently_held.min(1))
    };

    affordable.min(stock_available).min(stack_space)
}

/// Validates and executes a buy transaction, returning the new state or an error.
///
/// Rejects if:
/// - `balance < buy_price * quantity` (insufficient funds)
/// - stackable and `inventory_qty + quantity > stack_limit` (inventory full)
/// - non-stackable and `inventory_qty > 0` (inventory full)
/// - `remaining_stock` is Some and `remaining_stock < quantity` (insufficient stock)
pub fn execute_buy(
    balance: u64,
    inventory_qty: u32,
    buy_price: u32,
    quantity: u32,
    remaining_stock: Option<u32>,
    is_stackable: bool,
    stack_limit: u32,
) -> Result<BuyResult, ShopError> {
    let total_cost = buy_price as u64 * quantity as u64;

    if balance < total_cost {
        return Err(ShopError::InsufficientFunds);
    }

    if is_stackable {
        if inventory_qty + quantity > stack_limit {
            return Err(ShopError::InventoryFull);
        }
    } else if inventory_qty > 0 {
        return Err(ShopError::InventoryFull);
    }

    if let Some(stock) = remaining_stock
        && stock < quantity
    {
        return Err(ShopError::InsufficientStock);
    }

    Ok(BuyResult {
        new_balance: balance - total_cost,
        new_inventory_qty: inventory_qty + quantity,
        new_stock: remaining_stock.map(|s| s - quantity),
    })
}

/// Validates and executes a sell transaction, returning the new state or an error.
///
/// Rejects if `inventory_qty < quantity` (insufficient inventory).
/// On success, adds `sell_price * quantity` to balance using saturating addition.
pub fn execute_sell(
    balance: u64,
    inventory_qty: u32,
    sell_price: u32,
    quantity: u32,
) -> Result<SellResult, ShopError> {
    if inventory_qty < quantity {
        return Err(ShopError::InsufficientInventory);
    }

    let revenue = sell_price as u64 * quantity as u64;
    let new_balance = balance.saturating_add(revenue);

    Ok(SellResult {
        new_balance,
        new_inventory_qty: inventory_qty - quantity,
    })
}

/// Filters shop entries by condition evaluation against game state flags.
///
/// An entry is visible if:
/// - It has no condition, OR
/// - Its condition has an empty checks list, OR
/// - Its condition evaluates to true against the provided flags
///
/// Additionally, entries whose `item_id` is not in the `item_registry` are skipped.
pub fn visible_entries<'a>(
    entries: &'a [ShopEntry],
    flags: &HashMap<String, String>,
    item_registry: &ItemRegistry,
) -> Vec<&'a ShopEntry> {
    entries
        .iter()
        .filter(|entry| {
            // Skip entries whose item_id is not in the registry
            if !item_registry.items.contains_key(&entry.item_id) {
                return false;
            }

            // Check condition
            match &entry.condition {
                None => true,
                Some(cond) => {
                    if cond.checks.is_empty() {
                        true
                    } else {
                        cond.evaluate(flags)
                    }
                }
            }
        })
        .collect()
}

/// Returns a list of sellable items from the player's inventory.
///
/// An item is sellable if:
/// - It exists in the inventory with quantity > 0
/// - It exists in the item_registry
/// - It is NOT a KeyItem
/// - Its computed sell price > 0
///
/// For sell price computation, if the item has a matching shop entry, use that entry's
/// sell_price override. Otherwise use `item.value / 2`.
///
/// Returns a Vec of `(item_id, quantity_held, sell_price)`.
pub fn sellable_items(
    inventory: &HashMap<String, u32>,
    item_registry: &ItemRegistry,
    shop_entries: &[ShopEntry],
) -> Vec<(ItemId, u32, u32)> {
    inventory
        .iter()
        .filter_map(|(item_id, &qty)| {
            if qty == 0 {
                return None;
            }

            let item = item_registry.items.get(item_id)?;

            // Exclude KeyItems
            if item.category() == ItemCategory::KeyItem {
                return None;
            }

            // Compute sell price: check if there's a shop entry for this item
            let sell_price = shop_entries
                .iter()
                .find(|e| e.item_id == *item_id)
                .map(|entry| compute_sell_price(entry, item))
                .unwrap_or(item.value / 2);

            if sell_price == 0 {
                return None;
            }

            Some((item_id.clone(), qty, sell_price))
        })
        .collect()
}

// ─── Shop Scene Plugin ────────────────────────────────────────────────────────

/// Wrapper resource for `ShopRegistry` (from rpg-toolkit-common) so it can be
/// used as a Bevy Resource in the scenes crate.
#[derive(Resource, Clone, Debug, Default)]
pub struct ShopRegistryRes {
    pub registry: ShopRegistry,
}

/// Wrapper resource for `ItemRegistry` (from rpg-toolkit-common) so it can be
/// used as a Bevy Resource in the scenes crate.
#[derive(Resource, Clone, Debug, Default)]
pub struct ItemRegistryRes {
    pub registry: ItemRegistry,
}

// Re-export ActiveShopId from rpg-toolkit-common for use by dependents.
pub use rpg_toolkit_common::shop::ActiveShopId;

/// Runtime stock tracking for the current shop session.
/// Only entries with stock limits are tracked here.
#[derive(Resource, Default, Debug, Clone, PartialEq, Eq)]
pub struct ShopStockState {
    pub remaining: HashMap<ItemId, u32>,
}

/// Marker component for all shop scene UI entities (used for despawn on exit).
#[derive(Component)]
struct ShopSceneEntity;

/// The current mode of the shop UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ShopMode {
    Buy,
    Sell,
}

/// Internal state resource tracking UI interactions within the shop.
#[derive(Resource, Debug, Clone)]
#[allow(dead_code)]
struct ShopUiState {
    mode: ShopMode,
    selected_index: usize,
    quantity: u32,
    /// Cached list of visible buy entries (item_id, buy_price, stock info).
    buy_items: Vec<ShopBuyItem>,
    /// Cached list of sellable items (item_id, qty_held, sell_price).
    sell_items: Vec<(ItemId, u32, u32)>,
    /// Error message to display (clears on next input).
    error_message: Option<String>,
    /// The shop display name.
    shop_name: String,
}

/// A single item available for purchase in the shop.
#[derive(Debug, Clone)]
struct ShopBuyItem {
    item_id: ItemId,
    display_name: String,
    buy_price: u32,
    stock_limit: Option<u32>,
    is_stackable: bool,
    stack_limit: u32,
}

// ─── Plugin ───────────────────────────────────────────────────────────────────

pub struct ShopScenePlugin;

impl Plugin for ShopScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppPhase::Shop), spawn_shop_ui)
            .add_systems(OnExit(AppPhase::Shop), despawn_shop_ui)
            .add_systems(Update, shop_input.run_if(in_state(AppPhase::Shop)));
    }
}

// ─── Colors ───────────────────────────────────────────────────────────────────

const SHOP_COLOR_SELECTED: Color = Color::srgb(1.0, 1.0, 0.0);
const SHOP_COLOR_NORMAL: Color = Color::srgb(1.0, 1.0, 1.0);
const SHOP_COLOR_SOLD_OUT: Color = Color::srgb(0.5, 0.5, 0.5);
const SHOP_COLOR_ERROR: Color = Color::srgb(1.0, 0.3, 0.3);
const SHOP_COLOR_HEADER: Color = Color::srgb(0.7, 0.9, 1.0);

// ─── Marker Components ────────────────────────────────────────────────────────

#[derive(Component)]
struct ShopItemList;

#[derive(Component)]
struct ShopCurrencyText;

#[derive(Component)]
struct ShopModeText;

#[derive(Component)]
struct ShopErrorText;

#[derive(Component)]
struct ShopQuantityText;

// ─── Systems ──────────────────────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
fn spawn_shop_ui(
    mut commands: Commands,
    active_shop: Option<Res<ActiveShopId>>,
    shop_registry: Option<Res<ShopRegistryRes>>,
    item_registry: Option<Res<ItemRegistryRes>>,
    game_state: Res<GameState>,
    currency: Res<CurrencyState>,
    inventory: Res<InventoryState>,
    config: Res<TitleScreenConfig>,
) {
    let Some(active_shop) = active_shop else {
        warn!("ShopScenePlugin: No ActiveShopId resource found");
        return;
    };
    let Some(shop_registry) = shop_registry else {
        warn!("ShopScenePlugin: No ShopRegistryRes resource found");
        return;
    };
    let Some(item_registry) = item_registry else {
        warn!("ShopScenePlugin: No ItemRegistryRes resource found");
        return;
    };

    let Some(shop_def) = shop_registry.registry.shops.get(&active_shop.shop_id) else {
        warn!(
            "ShopScenePlugin: Shop ID '{}' not found in registry",
            active_shop.shop_id
        );
        return;
    };

    // Load saved stock from SaveFile
    let save_file = SaveFile::load(&config.save_path);

    // Filter visible entries
    let visible = visible_entries(
        &shop_def.entries,
        &game_state.flags,
        &item_registry.registry,
    );

    // Initialize ShopStockState from save, clamping per Property 16
    let mut stock_state = ShopStockState::default();
    let saved_shop_stock = save_file
        .shop_stock
        .get(&active_shop.shop_id)
        .cloned()
        .unwrap_or_default();

    for entry in &shop_def.entries {
        if let Some(configured_limit) = entry.stock_limit {
            let saved_value = saved_shop_stock.get(&entry.item_id).copied();
            let remaining = match saved_value {
                Some(saved) => saved.min(configured_limit), // clamp to limit
                None => configured_limit,                   // full restock
            };
            stock_state
                .remaining
                .insert(entry.item_id.clone(), remaining);
        }
    }

    commands.insert_resource(stock_state.clone());

    // Build buy items list from visible entries
    let buy_items: Vec<ShopBuyItem> = visible
        .iter()
        .filter_map(|entry| {
            let item = item_registry.registry.items.get(&entry.item_id)?;
            Some(ShopBuyItem {
                item_id: entry.item_id.clone(),
                display_name: item.display_name.clone(),
                buy_price: entry.buy_price,
                stock_limit: entry.stock_limit,
                is_stackable: item.stackable,
                stack_limit: item.stack_limit,
            })
        })
        .collect();

    // Build sell items list
    let sell_items = sellable_items(&inventory.items, &item_registry.registry, &shop_def.entries);

    let shop_name = shop_def.display_name.clone();

    commands.insert_resource(ShopUiState {
        mode: ShopMode::Buy,
        selected_index: 0,
        quantity: 1,
        buy_items: buy_items.clone(),
        sell_items,
        error_message: None,
        shop_name: shop_name.clone(),
    });

    // Spawn UI
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(20.0)),
                row_gap: Val::Px(10.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.05, 0.05, 0.15)),
            ShopSceneEntity,
        ))
        .with_children(|parent| {
            // Shop name header
            parent.spawn((
                Text::new(shop_name.clone()),
                TextFont {
                    font_size: 36.0,
                    ..default()
                },
                TextColor(SHOP_COLOR_HEADER),
            ));

            // Currency display
            parent.spawn((
                Text::new(format!("Gold: {}", currency.balance)),
                TextFont {
                    font_size: 22.0,
                    ..default()
                },
                TextColor(SHOP_COLOR_NORMAL),
                ShopCurrencyText,
            ));

            // Mode indicator (Buy / Sell)
            parent.spawn((
                Text::new("[Buy]  Sell  (Tab to switch)"),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(SHOP_COLOR_NORMAL),
                ShopModeText,
            ));

            // Item list container
            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::FlexStart,
                        row_gap: Val::Px(4.0),
                        ..default()
                    },
                    ShopItemList,
                ))
                .with_children(|list_parent| {
                    if buy_items.is_empty() {
                        list_parent.spawn((
                            Text::new("No items available"),
                            TextFont {
                                font_size: 20.0,
                                ..default()
                            },
                            TextColor(SHOP_COLOR_SOLD_OUT),
                        ));
                    } else {
                        for (i, buy_item) in buy_items.iter().enumerate() {
                            let stock_text = match buy_item.stock_limit {
                                Some(_) => {
                                    let remaining = stock_state
                                        .remaining
                                        .get(&buy_item.item_id)
                                        .copied()
                                        .unwrap_or(0);
                                    if remaining == 0 {
                                        "Sold Out".to_string()
                                    } else {
                                        format!("x{}", remaining)
                                    }
                                }
                                None => "∞".to_string(),
                            };

                            let prefix = if i == 0 { "> " } else { "  " };
                            let is_sold_out = buy_item.stock_limit.is_some()
                                && stock_state
                                    .remaining
                                    .get(&buy_item.item_id)
                                    .copied()
                                    .unwrap_or(0)
                                    == 0;

                            let line = format!(
                                "{}{} - {} gold [{}]",
                                prefix, buy_item.display_name, buy_item.buy_price, stock_text
                            );

                            let color = if is_sold_out {
                                SHOP_COLOR_SOLD_OUT
                            } else if i == 0 {
                                SHOP_COLOR_SELECTED
                            } else {
                                SHOP_COLOR_NORMAL
                            };

                            list_parent.spawn((
                                Text::new(line),
                                TextFont {
                                    font_size: 20.0,
                                    ..default()
                                },
                                TextColor(color),
                            ));
                        }
                    }
                });

            // Quantity display
            parent.spawn((
                Text::new("Qty: 1"),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(SHOP_COLOR_NORMAL),
                ShopQuantityText,
            ));

            // Error message (initially empty)
            parent.spawn((
                Text::new(""),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(SHOP_COLOR_ERROR),
                ShopErrorText,
            ));
        });
}

fn despawn_shop_ui(
    mut commands: Commands,
    query: Query<Entity, With<ShopSceneEntity>>,
    config: Res<TitleScreenConfig>,
    stock_state: Option<Res<ShopStockState>>,
    active_shop: Option<Res<ActiveShopId>>,
) {
    // Persist stock state to SaveFile before despawning
    if let (Some(stock_state), Some(active_shop)) = (stock_state, active_shop) {
        let mut save_file = SaveFile::load(&config.save_path);

        // Build the stock map for this shop (only items with remaining stock tracked)
        let shop_stock: std::collections::BTreeMap<String, u32> = stock_state
            .remaining
            .iter()
            .map(|(item_id, &qty)| (item_id.clone(), qty))
            .collect();

        save_file
            .shop_stock
            .insert(active_shop.shop_id.clone(), shop_stock);

        if let Err(e) = save_file.save(&config.save_path) {
            warn!("Failed to persist shop stock: {}", e);
        }
    }

    // Despawn all shop UI entities
    for entity in &query {
        commands.entity(entity).despawn();
    }

    // Remove internal state resources
    commands.remove_resource::<ShopUiState>();
    commands.remove_resource::<ShopStockState>();
    commands.remove_resource::<ActiveShopId>();
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn shop_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    ui_state: Option<ResMut<ShopUiState>>,
    mut currency: ResMut<CurrencyState>,
    mut inventory: ResMut<InventoryState>,
    stock_state: Option<ResMut<ShopStockState>>,
    mut next_phase: ResMut<NextState<AppPhase>>,
    item_registry: Option<Res<ItemRegistryRes>>,
    shop_registry: Option<Res<ShopRegistryRes>>,
    active_shop: Option<Res<ActiveShopId>>,
    mut currency_text: Query<
        &mut Text,
        (
            With<ShopCurrencyText>,
            Without<ShopModeText>,
            Without<ShopErrorText>,
            Without<ShopQuantityText>,
        ),
    >,
    mut mode_text: Query<
        &mut Text,
        (
            With<ShopModeText>,
            Without<ShopCurrencyText>,
            Without<ShopErrorText>,
            Without<ShopQuantityText>,
        ),
    >,
    mut error_text: Query<
        &mut Text,
        (
            With<ShopErrorText>,
            Without<ShopCurrencyText>,
            Without<ShopModeText>,
            Without<ShopQuantityText>,
        ),
    >,
    mut qty_text: Query<
        &mut Text,
        (
            With<ShopQuantityText>,
            Without<ShopCurrencyText>,
            Without<ShopModeText>,
            Without<ShopErrorText>,
        ),
    >,
) {
    // Early return if shop UI state hasn't been initialized yet
    let Some(mut ui_state) = ui_state else { return };
    let Some(mut stock_state) = stock_state else {
        return;
    };

    // Clear error on any input
    if keyboard.get_just_pressed().len() > 0 {
        ui_state.error_message = None;
        if let Ok(mut text) = error_text.single_mut() {
            **text = String::new();
        }
    }

    // Exit shop (Escape or Backspace)
    if keyboard.just_pressed(KeyCode::Escape) || keyboard.just_pressed(KeyCode::Backspace) {
        next_phase.set(AppPhase::InGame);
        return;
    }

    // Switch mode (Tab)
    if keyboard.just_pressed(KeyCode::Tab) {
        ui_state.mode = match ui_state.mode {
            ShopMode::Buy => ShopMode::Sell,
            ShopMode::Sell => ShopMode::Buy,
        };
        ui_state.selected_index = 0;
        ui_state.quantity = 1;

        // Refresh sell items list
        if ui_state.mode == ShopMode::Sell
            && let (Some(item_reg), Some(shop_reg), Some(active)) =
                (&item_registry, &shop_registry, &active_shop)
            && let Some(shop_def) = shop_reg.registry.shops.get(&active.shop_id)
        {
            ui_state.sell_items =
                sellable_items(&inventory.items, &item_reg.registry, &shop_def.entries);
        }

        update_mode_text(&ui_state, &mut mode_text);
    }

    let item_count = match ui_state.mode {
        ShopMode::Buy => ui_state.buy_items.len(),
        ShopMode::Sell => ui_state.sell_items.len(),
    };

    if item_count == 0 {
        return;
    }

    // Navigation (Up/Down)
    if (keyboard.just_pressed(KeyCode::ArrowUp) || keyboard.just_pressed(KeyCode::KeyW))
        && ui_state.selected_index > 0
    {
        ui_state.selected_index -= 1;
        ui_state.quantity = 1;
    }
    if (keyboard.just_pressed(KeyCode::ArrowDown) || keyboard.just_pressed(KeyCode::KeyS))
        && ui_state.selected_index < item_count - 1
    {
        ui_state.selected_index += 1;
        ui_state.quantity = 1;
    }

    // Quantity adjustment (Left/Right)
    if keyboard.just_pressed(KeyCode::ArrowRight) || keyboard.just_pressed(KeyCode::Equal) {
        let max_qty = compute_max_quantity(&ui_state, &currency, &inventory, &stock_state);
        if ui_state.quantity < max_qty {
            ui_state.quantity += 1;
        }
    }
    if (keyboard.just_pressed(KeyCode::ArrowLeft) || keyboard.just_pressed(KeyCode::Minus))
        && ui_state.quantity > 1
    {
        ui_state.quantity -= 1;
    }

    // Update quantity text
    if let Ok(mut text) = qty_text.single_mut() {
        **text = format!("Qty: {}", ui_state.quantity);
    }

    // Confirm transaction (Enter/Space)
    if keyboard.just_pressed(KeyCode::Enter) || keyboard.just_pressed(KeyCode::Space) {
        match ui_state.mode {
            ShopMode::Buy => {
                handle_buy_confirm(
                    &mut ui_state,
                    &mut currency,
                    &mut inventory,
                    &mut stock_state,
                    &mut error_text,
                );
            }
            ShopMode::Sell => {
                handle_sell_confirm(
                    &mut ui_state,
                    &mut currency,
                    &mut inventory,
                    &mut error_text,
                    &item_registry,
                    &shop_registry,
                    &active_shop,
                );
            }
        }

        // Update currency display
        if let Ok(mut text) = currency_text.single_mut() {
            **text = format!("Gold: {}", currency.balance);
        }
    }
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

#[allow(clippy::type_complexity)]
fn update_mode_text(
    state: &ShopUiState,
    mode_text: &mut Query<
        &mut Text,
        (
            With<ShopModeText>,
            Without<ShopCurrencyText>,
            Without<ShopErrorText>,
            Without<ShopQuantityText>,
        ),
    >,
) {
    if let Ok(mut text) = mode_text.single_mut() {
        **text = match state.mode {
            ShopMode::Buy => "[Buy]  Sell  (Tab to switch)".to_string(),
            ShopMode::Sell => " Buy  [Sell] (Tab to switch)".to_string(),
        };
    }
}

fn compute_max_quantity(
    state: &ShopUiState,
    currency: &CurrencyState,
    inventory: &InventoryState,
    stock_state: &ShopStockState,
) -> u32 {
    match state.mode {
        ShopMode::Buy => {
            if state.selected_index >= state.buy_items.len() {
                return 0;
            }
            let item = &state.buy_items[state.selected_index];
            let remaining_stock = item.stock_limit.map(|_| {
                stock_state
                    .remaining
                    .get(&item.item_id)
                    .copied()
                    .unwrap_or(0)
            });
            let currently_held = inventory.items.get(&item.item_id).copied().unwrap_or(0);
            max_buy_quantity(
                currency.balance,
                item.buy_price,
                remaining_stock,
                item.is_stackable,
                item.stack_limit,
                currently_held,
            )
        }
        ShopMode::Sell => {
            if state.selected_index >= state.sell_items.len() {
                return 0;
            }
            let (ref item_id, _, _) = state.sell_items[state.selected_index];
            inventory.items.get(item_id).copied().unwrap_or(0)
        }
    }
}

#[allow(clippy::type_complexity)]
fn handle_buy_confirm(
    ui_state: &mut ResMut<ShopUiState>,
    currency: &mut ResMut<CurrencyState>,
    inventory: &mut ResMut<InventoryState>,
    stock_state: &mut ResMut<ShopStockState>,
    error_text: &mut Query<
        &mut Text,
        (
            With<ShopErrorText>,
            Without<ShopCurrencyText>,
            Without<ShopModeText>,
            Without<ShopQuantityText>,
        ),
    >,
) {
    if ui_state.selected_index >= ui_state.buy_items.len() {
        return;
    }

    let item = ui_state.buy_items[ui_state.selected_index].clone();

    // Check if sold out
    if item.stock_limit.is_some() {
        let remaining = stock_state
            .remaining
            .get(&item.item_id)
            .copied()
            .unwrap_or(0);
        if remaining == 0 {
            show_shop_error(error_text, ui_state, "Sold Out!");
            return;
        }
    }

    let remaining_stock = item.stock_limit.map(|_| {
        stock_state
            .remaining
            .get(&item.item_id)
            .copied()
            .unwrap_or(0)
    });
    let inventory_qty = inventory.items.get(&item.item_id).copied().unwrap_or(0);

    let result = execute_buy(
        currency.balance,
        inventory_qty,
        item.buy_price,
        ui_state.quantity,
        remaining_stock,
        item.is_stackable,
        item.stack_limit,
    );

    match result {
        Ok(buy_result) => {
            currency.balance = buy_result.new_balance;
            inventory
                .items
                .insert(item.item_id.clone(), buy_result.new_inventory_qty);
            if let Some(new_stock) = buy_result.new_stock {
                stock_state
                    .remaining
                    .insert(item.item_id.clone(), new_stock);
            }
            ui_state.quantity = 1;
        }
        Err(ShopError::InsufficientFunds) => {
            show_shop_error(error_text, ui_state, "Insufficient funds!");
        }
        Err(ShopError::InventoryFull) => {
            show_shop_error(error_text, ui_state, "Inventory full!");
        }
        Err(ShopError::InsufficientStock) => {
            show_shop_error(error_text, ui_state, "Insufficient stock!");
        }
        Err(_) => {
            show_shop_error(error_text, ui_state, "Transaction failed!");
        }
    }
}

#[allow(clippy::type_complexity)]
fn handle_sell_confirm(
    ui_state: &mut ResMut<ShopUiState>,
    currency: &mut ResMut<CurrencyState>,
    inventory: &mut ResMut<InventoryState>,
    error_text: &mut Query<
        &mut Text,
        (
            With<ShopErrorText>,
            Without<ShopCurrencyText>,
            Without<ShopModeText>,
            Without<ShopQuantityText>,
        ),
    >,
    item_registry: &Option<Res<ItemRegistryRes>>,
    shop_registry: &Option<Res<ShopRegistryRes>>,
    active_shop: &Option<Res<ActiveShopId>>,
) {
    if ui_state.selected_index >= ui_state.sell_items.len() {
        return;
    }

    let (item_id, _, sell_price) = ui_state.sell_items[ui_state.selected_index].clone();
    let inventory_qty = inventory.items.get(&item_id).copied().unwrap_or(0);

    let result = execute_sell(
        currency.balance,
        inventory_qty,
        sell_price,
        ui_state.quantity,
    );

    match result {
        Ok(sell_result) => {
            currency.balance = sell_result.new_balance;
            if sell_result.new_inventory_qty == 0 {
                inventory.items.remove(&item_id);
            } else {
                inventory
                    .items
                    .insert(item_id.clone(), sell_result.new_inventory_qty);
            }
            ui_state.quantity = 1;

            // Refresh sell items list
            if let (Some(item_reg), Some(shop_reg), Some(active)) =
                (item_registry, shop_registry, active_shop)
                && let Some(shop_def) = shop_reg.registry.shops.get(&active.shop_id)
            {
                ui_state.sell_items =
                    sellable_items(&inventory.items, &item_reg.registry, &shop_def.entries);
            }

            // Adjust selected index if it's now out of bounds
            if ui_state.selected_index >= ui_state.sell_items.len()
                && !ui_state.sell_items.is_empty()
            {
                ui_state.selected_index = ui_state.sell_items.len() - 1;
            }
        }
        Err(ShopError::InsufficientInventory) => {
            show_shop_error(error_text, ui_state, "Insufficient inventory!");
        }
        Err(_) => {
            show_shop_error(error_text, ui_state, "Transaction failed!");
        }
    }
}

#[allow(clippy::type_complexity)]
fn show_shop_error(
    error_text: &mut Query<
        &mut Text,
        (
            With<ShopErrorText>,
            Without<ShopCurrencyText>,
            Without<ShopModeText>,
            Without<ShopQuantityText>,
        ),
    >,
    ui_state: &mut ResMut<ShopUiState>,
    message: &str,
) {
    ui_state.error_message = Some(message.to_string());
    if let Ok(mut text) = error_text.single_mut() {
        **text = message.to_string();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rpg_toolkit_common::condition::{
        BranchCondition, ConditionCheck, ConditionLogic, ConditionOperator,
    };
    use rpg_toolkit_common::item::{Item, ItemCategoryData, ItemRegistry, Rarity};

    fn make_item(
        id: &str,
        value: u32,
        stackable: bool,
        stack_limit: u32,
        category_data: ItemCategoryData,
    ) -> Item {
        Item {
            id: id.to_string(),
            display_name: id.to_string(),
            description: String::new(),
            category_data,
            value,
            rarity: Rarity::Common,
            stackable,
            stack_limit,
            stat_modifiers: vec![],
            granted_abilities: vec![],
            graphics: Default::default(),
        }
    }

    fn make_consumable(id: &str, value: u32) -> Item {
        make_item(
            id,
            value,
            true,
            99,
            ItemCategoryData::Consumable { effects: vec![] },
        )
    }

    fn make_key_item(id: &str) -> Item {
        make_item(id, 0, false, 1, ItemCategoryData::KeyItem)
    }

    fn make_entry(
        item_id: &str,
        buy_price: u32,
        sell_price: Option<u32>,
        stock_limit: Option<u32>,
    ) -> ShopEntry {
        ShopEntry {
            item_id: item_id.to_string(),
            buy_price,
            sell_price,
            stock_limit,
            condition: None,
        }
    }

    // ── compute_sell_price ────────────────────────────────────────────────

    #[test]
    fn test_compute_sell_price_with_override() {
        let entry = make_entry("potion", 100, Some(30), None);
        let item = make_consumable("potion", 100);
        assert_eq!(compute_sell_price(&entry, &item), 30);
    }

    #[test]
    fn test_compute_sell_price_without_override() {
        let entry = make_entry("potion", 100, None, None);
        let item = make_consumable("potion", 100);
        assert_eq!(compute_sell_price(&entry, &item), 50);
    }

    #[test]
    fn test_compute_sell_price_odd_value() {
        let entry = make_entry("potion", 100, None, None);
        let item = make_consumable("potion", 7);
        assert_eq!(compute_sell_price(&entry, &item), 3); // 7/2 = 3 (floor)
    }

    // ── max_buy_quantity ──────────────────────────────────────────────────

    #[test]
    fn test_max_buy_quantity_basic() {
        // 1000 gold, price 100, unlimited stock, stackable limit 99, holding 0
        assert_eq!(max_buy_quantity(1000, 100, None, true, 99, 0), 10);
    }

    #[test]
    fn test_max_buy_quantity_limited_by_stock() {
        // Can afford 10 but only 3 in stock
        assert_eq!(max_buy_quantity(1000, 100, Some(3), true, 99, 0), 3);
    }

    #[test]
    fn test_max_buy_quantity_limited_by_stack() {
        // Can afford 10, unlimited stock, but only 5 stack space left
        assert_eq!(max_buy_quantity(1000, 100, None, true, 99, 94), 5);
    }

    #[test]
    fn test_max_buy_quantity_non_stackable_empty() {
        // Non-stackable, not holding any
        assert_eq!(max_buy_quantity(1000, 100, None, false, 1, 0), 1);
    }

    #[test]
    fn test_max_buy_quantity_non_stackable_already_held() {
        // Non-stackable, already holding one
        assert_eq!(max_buy_quantity(1000, 100, None, false, 1, 1), 0);
    }

    #[test]
    fn test_max_buy_quantity_zero_price() {
        // Free item, limited by stock
        assert_eq!(max_buy_quantity(0, 0, Some(5), true, 99, 0), 5);
    }

    // ── execute_buy ──────────────────────────────────────────────────────

    #[test]
    fn test_execute_buy_success() {
        let result = execute_buy(1000, 0, 100, 3, Some(10), true, 99).unwrap();
        assert_eq!(result.new_balance, 700);
        assert_eq!(result.new_inventory_qty, 3);
        assert_eq!(result.new_stock, Some(7));
    }

    #[test]
    fn test_execute_buy_insufficient_funds() {
        let result = execute_buy(200, 0, 100, 3, None, true, 99);
        assert_eq!(result, Err(ShopError::InsufficientFunds));
    }

    #[test]
    fn test_execute_buy_stack_overflow() {
        let result = execute_buy(10000, 95, 100, 10, None, true, 99);
        assert_eq!(result, Err(ShopError::InventoryFull));
    }

    #[test]
    fn test_execute_buy_non_stackable_already_held() {
        let result = execute_buy(10000, 1, 100, 1, None, false, 1);
        assert_eq!(result, Err(ShopError::InventoryFull));
    }

    #[test]
    fn test_execute_buy_insufficient_stock() {
        let result = execute_buy(10000, 0, 100, 5, Some(3), true, 99);
        assert_eq!(result, Err(ShopError::InsufficientStock));
    }

    #[test]
    fn test_execute_buy_unlimited_stock() {
        let result = execute_buy(1000, 0, 100, 5, None, true, 99).unwrap();
        assert_eq!(result.new_balance, 500);
        assert_eq!(result.new_inventory_qty, 5);
        assert_eq!(result.new_stock, None);
    }

    // ── execute_sell ─────────────────────────────────────────────────────

    #[test]
    fn test_execute_sell_success() {
        let result = execute_sell(100, 5, 30, 3).unwrap();
        assert_eq!(result.new_balance, 190);
        assert_eq!(result.new_inventory_qty, 2);
    }

    #[test]
    fn test_execute_sell_insufficient_inventory() {
        let result = execute_sell(100, 2, 30, 3);
        assert_eq!(result, Err(ShopError::InsufficientInventory));
    }

    #[test]
    fn test_execute_sell_saturating_add() {
        let result = execute_sell(u64::MAX - 10, 5, 100, 1).unwrap();
        assert_eq!(result.new_balance, u64::MAX);
    }

    // ── visible_entries ──────────────────────────────────────────────────

    #[test]
    fn test_visible_entries_no_condition() {
        let mut registry = ItemRegistry::default();
        let item = make_consumable("potion", 100);
        registry.items.insert("potion".to_string(), item);

        let entries = vec![make_entry("potion", 100, None, None)];
        let flags = HashMap::new();

        let visible = visible_entries(&entries, &flags, &registry);
        assert_eq!(visible.len(), 1);
    }

    #[test]
    fn test_visible_entries_missing_item() {
        let registry = ItemRegistry::default(); // empty
        let entries = vec![make_entry("missing", 100, None, None)];
        let flags = HashMap::new();

        let visible = visible_entries(&entries, &flags, &registry);
        assert_eq!(visible.len(), 0);
    }

    #[test]
    fn test_visible_entries_condition_false() {
        let mut registry = ItemRegistry::default();
        let item = make_consumable("potion", 100);
        registry.items.insert("potion".to_string(), item);

        let entry = ShopEntry {
            item_id: "potion".to_string(),
            buy_price: 100,
            sell_price: None,
            stock_limit: None,
            condition: Some(BranchCondition {
                logic: ConditionLogic::All,
                checks: vec![ConditionCheck {
                    key: "boss_defeated".to_string(),
                    operator: ConditionOperator::Equals,
                    value: Some("true".to_string()),
                }],
            }),
        };
        let entries = vec![entry];
        let flags = HashMap::new(); // flag not set

        let visible = visible_entries(&entries, &flags, &registry);
        assert_eq!(visible.len(), 0);
    }

    #[test]
    fn test_visible_entries_condition_true() {
        let mut registry = ItemRegistry::default();
        let item = make_consumable("potion", 100);
        registry.items.insert("potion".to_string(), item);

        let entry = ShopEntry {
            item_id: "potion".to_string(),
            buy_price: 100,
            sell_price: None,
            stock_limit: None,
            condition: Some(BranchCondition {
                logic: ConditionLogic::All,
                checks: vec![ConditionCheck {
                    key: "boss_defeated".to_string(),
                    operator: ConditionOperator::Equals,
                    value: Some("true".to_string()),
                }],
            }),
        };
        let entries = vec![entry];
        let mut flags = HashMap::new();
        flags.insert("boss_defeated".to_string(), "true".to_string());

        let visible = visible_entries(&entries, &flags, &registry);
        assert_eq!(visible.len(), 1);
    }

    #[test]
    fn test_visible_entries_empty_checks() {
        let mut registry = ItemRegistry::default();
        let item = make_consumable("potion", 100);
        registry.items.insert("potion".to_string(), item);

        let entry = ShopEntry {
            item_id: "potion".to_string(),
            buy_price: 100,
            sell_price: None,
            stock_limit: None,
            condition: Some(BranchCondition {
                logic: ConditionLogic::All,
                checks: vec![],
            }),
        };
        let entries = vec![entry];
        let flags = HashMap::new();

        let visible = visible_entries(&entries, &flags, &registry);
        assert_eq!(visible.len(), 1);
    }

    // ── sellable_items ───────────────────────────────────────────────────

    #[test]
    fn test_sellable_items_basic() {
        let mut registry = ItemRegistry::default();
        let item = make_consumable("potion", 100);
        registry.items.insert("potion".to_string(), item);

        let mut inventory = HashMap::new();
        inventory.insert("potion".to_string(), 5u32);

        let entries = vec![make_entry("potion", 100, Some(40), None)];

        let sellable = sellable_items(&inventory, &registry, &entries);
        assert_eq!(sellable.len(), 1);
        assert_eq!(sellable[0], ("potion".to_string(), 5, 40));
    }

    #[test]
    fn test_sellable_items_excludes_key_items() {
        let mut registry = ItemRegistry::default();
        let item = make_key_item("quest_item");
        registry.items.insert("quest_item".to_string(), item);

        let mut inventory = HashMap::new();
        inventory.insert("quest_item".to_string(), 1u32);

        let entries = vec![make_entry("quest_item", 0, Some(100), None)];

        let sellable = sellable_items(&inventory, &registry, &entries);
        assert_eq!(sellable.len(), 0);
    }

    #[test]
    fn test_sellable_items_excludes_zero_sell_price() {
        let mut registry = ItemRegistry::default();
        let item = make_consumable("junk", 0); // value 0, so sell price = 0
        registry.items.insert("junk".to_string(), item);

        let mut inventory = HashMap::new();
        inventory.insert("junk".to_string(), 3u32);

        let entries: Vec<ShopEntry> = vec![]; // no entry override

        let sellable = sellable_items(&inventory, &registry, &entries);
        assert_eq!(sellable.len(), 0);
    }

    #[test]
    fn test_sellable_items_uses_default_sell_price() {
        let mut registry = ItemRegistry::default();
        let item = make_consumable("potion", 80);
        registry.items.insert("potion".to_string(), item);

        let mut inventory = HashMap::new();
        inventory.insert("potion".to_string(), 2u32);

        let entries: Vec<ShopEntry> = vec![]; // no entry override

        let sellable = sellable_items(&inventory, &registry, &entries);
        assert_eq!(sellable.len(), 1);
        assert_eq!(sellable[0], ("potion".to_string(), 2, 40)); // 80/2 = 40
    }

    #[test]
    fn test_sellable_items_excludes_missing_from_registry() {
        let registry = ItemRegistry::default(); // empty

        let mut inventory = HashMap::new();
        inventory.insert("nonexistent".to_string(), 3u32);

        let entries: Vec<ShopEntry> = vec![];

        let sellable = sellable_items(&inventory, &registry, &entries);
        assert_eq!(sellable.len(), 0);
    }

    #[test]
    fn test_sellable_items_excludes_zero_quantity() {
        let mut registry = ItemRegistry::default();
        let item = make_consumable("potion", 100);
        registry.items.insert("potion".to_string(), item);

        let mut inventory = HashMap::new();
        inventory.insert("potion".to_string(), 0u32);

        let entries: Vec<ShopEntry> = vec![];

        let sellable = sellable_items(&inventory, &registry, &entries);
        assert_eq!(sellable.len(), 0);
    }
}
