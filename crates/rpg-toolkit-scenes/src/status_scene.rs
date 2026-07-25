use std::collections::HashMap;

use bevy::prelude::*;
use rpg_toolkit_common::AppPhase;
use rpg_toolkit_common::ability::AbilityRegistry;
use rpg_toolkit_common::character::CharacterRegistry;
use rpg_toolkit_common::item::{ItemCategory, ItemRegistry};

use crate::shop_scene::ItemRegistryRes;
use crate::title_screen::{CharacterProgress, CharacterProgressState, InventoryState, PartyState};

/// Maximum number of party members displayed in the party list view.
/// The active party is capped at this size for display purposes.
pub const MAX_PARTY_DISPLAY: usize = 4;

// ─── Enums ────────────────────────────────────────────────────────────────────

/// The active top-level sub-page.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusMode {
    PartyList,
    Inventory,
}

/// Whether a detail view is active.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetailView {
    None,
    CharacterDetail,
}

/// Category tabs for the inventory browser.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InventoryTab {
    Weapon,
    Armor,
    Accessory,
    Consumable,
    KeyItem,
}

// ─── State Resource ───────────────────────────────────────────────────────────

/// The full internal UI state for the status scene.
#[derive(Resource, Debug, Clone)]
pub struct StatusUiState {
    /// Current top-level sub-page.
    pub mode: StatusMode,
    /// Active detail view (None means we're at top level).
    pub detail_view: DetailView,
    /// Selection index for the party list.
    pub party_selection: usize,
    /// Selection index for the inventory item list.
    pub inventory_selection: usize,
    /// Currently active inventory category tab.
    pub inventory_tab: InventoryTab,
    /// Cached resolved party data for display.
    pub party_data: Vec<PartyMemberDisplayData>,
    /// Cached inventory items per tab.
    pub inventory_data: Vec<InventoryItemDisplayData>,
}

// ─── Display Data Structs ─────────────────────────────────────────────────────

/// Resolved display data for a party member row.
#[derive(Debug, Clone)]
pub struct PartyMemberDisplayData {
    pub character_id: String,
    pub display_name: String,
    pub level: u32,
    pub effective_hp: u32,
    pub has_portrait: bool,
    pub portrait_path: Option<String>,
}

/// Resolved display data for an inventory item.
#[derive(Debug, Clone)]
pub struct InventoryItemDisplayData {
    pub item_id: String,
    pub display_name: String,
    pub quantity: u32,
    pub has_icon: bool,
    pub icon_path: Option<String>,
    pub description: String,
    pub stat_modifiers: Vec<(String, i32)>,
}

// ─── Registry Wrapper Resources ───────────────────────────────────────────────

/// Wrapper for CharacterRegistry as a Bevy Resource.
#[derive(Resource, Clone, Debug, Default)]
pub struct CharacterRegistryRes {
    pub registry: CharacterRegistry,
}

/// Wrapper for AbilityRegistry as a Bevy Resource.
#[derive(Resource, Clone, Debug, Default)]
pub struct AbilityRegistryRes {
    pub registry: AbilityRegistry,
}

// ─── Colors ────────────────────────────────────────────────────────────────────

const STATUS_COLOR_HEADER: Color = Color::srgb(0.7, 0.9, 1.0);
const STATUS_COLOR_NORMAL: Color = Color::srgb(1.0, 1.0, 1.0);
const STATUS_COLOR_DIMMED: Color = Color::srgb(0.5, 0.5, 0.5);
const STATUS_COLOR_SELECTED_BG: Color = Color::srgb(0.2, 0.3, 0.5);
const STATUS_COLOR_TAB_ACTIVE: Color = Color::srgb(1.0, 1.0, 0.0);
const STATUS_COLOR_POSITIVE_MOD: Color = Color::srgb(0.3, 0.9, 0.3);
const STATUS_COLOR_NEGATIVE_MOD: Color = Color::srgb(0.9, 0.3, 0.3);

// ─── Marker Components ────────────────────────────────────────────────────────

/// Top-level marker on every entity spawned by the status scene.
#[derive(Component)]
pub struct StatusSceneMarker;

/// Marker for the party list container node.
#[derive(Component)]
#[allow(dead_code)]
pub(crate) struct PartyListContainer;

/// Marker for individual party member row text nodes.
#[derive(Component)]
#[allow(dead_code)]
pub(crate) struct PartyMemberRow(pub(crate) usize);

/// Marker for the character detail panel root.
#[derive(Component)]
#[allow(dead_code)]
pub(crate) struct CharacterDetailPanel;

/// Marker for the inventory list container.
#[derive(Component)]
#[allow(dead_code)]
pub(crate) struct InventoryListContainer;

/// Marker for the inventory detail/description panel.
#[derive(Component)]
#[allow(dead_code)]
pub(crate) struct InventoryDetailPanel;

/// Marker for tab indicator text.
#[derive(Component)]
#[allow(dead_code)]
pub(crate) struct InventoryTabIndicator;

/// Marker for the sub-page tab indicator (Party | Inventory).
#[derive(Component)]
#[allow(dead_code)]
pub(crate) struct SubPageTabIndicator;

// ─── Pure Helper Functions ─────────────────────────────────────────────────────

/// Computes the effective stat value: base_value + growth_value * (level - 1).
/// Uses saturating arithmetic to prevent overflow.
pub fn compute_effective_stat(base_value: u32, growth_value: u32, level: u32) -> u32 {
    let level_factor = level.saturating_sub(1);
    base_value.saturating_add(growth_value.saturating_mul(level_factor))
}

/// Clamps a selection index to valid bounds [0, len-1].
/// Returns 0 if len is 0.
pub fn clamp_selection(index: usize, len: usize) -> usize {
    if len == 0 { 0 } else { index.min(len - 1) }
}

/// Returns the next tab in the fixed order, or stays if at the end.
/// Fixed order: Weapon → Armor → Accessory → Consumable → KeyItem
pub fn next_tab(tab: InventoryTab) -> InventoryTab {
    match tab {
        InventoryTab::Weapon => InventoryTab::Armor,
        InventoryTab::Armor => InventoryTab::Accessory,
        InventoryTab::Accessory => InventoryTab::Consumable,
        InventoryTab::Consumable => InventoryTab::KeyItem,
        InventoryTab::KeyItem => InventoryTab::KeyItem,
    }
}

/// Returns the previous tab in the fixed order, or stays if at the start.
/// Fixed order: Weapon → Armor → Accessory → Consumable → KeyItem
pub fn prev_tab(tab: InventoryTab) -> InventoryTab {
    match tab {
        InventoryTab::Weapon => InventoryTab::Weapon,
        InventoryTab::Armor => InventoryTab::Weapon,
        InventoryTab::Accessory => InventoryTab::Armor,
        InventoryTab::Consumable => InventoryTab::Accessory,
        InventoryTab::KeyItem => InventoryTab::Consumable,
    }
}

/// Maps InventoryTab enum to ItemCategory for filtering.
pub fn tab_to_category(tab: InventoryTab) -> ItemCategory {
    match tab {
        InventoryTab::Weapon => ItemCategory::Weapon,
        InventoryTab::Armor => ItemCategory::Armor,
        InventoryTab::Accessory => ItemCategory::Accessory,
        InventoryTab::Consumable => ItemCategory::Consumable,
        InventoryTab::KeyItem => ItemCategory::KeyItem,
    }
}

/// Returns the display label for an inventory tab.
pub fn tab_label(tab: InventoryTab) -> &'static str {
    match tab {
        InventoryTab::Weapon => "Weapon",
        InventoryTab::Armor => "Armor",
        InventoryTab::Accessory => "Accessory",
        InventoryTab::Consumable => "Consumable",
        InventoryTab::KeyItem => "KeyItem",
    }
}

/// Returns the list of all inventory tabs in fixed order.
pub fn all_tabs() -> [InventoryTab; 5] {
    [
        InventoryTab::Weapon,
        InventoryTab::Armor,
        InventoryTab::Accessory,
        InventoryTab::Consumable,
        InventoryTab::KeyItem,
    ]
}

/// Resolves an ordered list of IDs, returning only those present in the lookup.
/// Preserves input order. Used for equipment and ability resolution.
pub fn resolve_ordered_ids<F>(ids: &[String], exists: F) -> Vec<String>
where
    F: Fn(&str) -> bool,
{
    ids.iter()
        .filter(|id| exists(id.as_str()))
        .cloned()
        .collect()
}

/// Resolves party member display data from registries and progress state.
/// Skips members whose CharacterId cannot be found in the registry.
/// Truncates the result to at most MAX_PARTY_DISPLAY (4) entries.
pub fn resolve_party_display_data(
    party: &[String],
    character_registry: &CharacterRegistry,
    _progress: &HashMap<String, CharacterProgress>,
) -> Vec<PartyMemberDisplayData> {
    let mut resolved: Vec<PartyMemberDisplayData> = party
        .iter()
        .filter_map(|character_id| {
            let character = character_registry.characters.get(character_id)?;

            // Find the "Level" stat base_value (default to 1 if not found)
            let level = character
                .stats
                .iter()
                .find(|s| s.name == "Level")
                .map(|s| s.base_value)
                .unwrap_or(1);

            // Find the "HP" stat base_value and growth_value (default to 0 if not found)
            let (hp_base, hp_growth) = character
                .stats
                .iter()
                .find(|s| s.name == "HP")
                .map(|s| (s.base_value, s.growth_value))
                .unwrap_or((0, 0));

            let effective_hp = compute_effective_stat(hp_base, hp_growth, level);
            let has_portrait = character.visual_assets.face_portrait.is_some();
            let portrait_path = character.visual_assets.face_portrait.clone();

            Some(PartyMemberDisplayData {
                character_id: character_id.clone(),
                display_name: character.display_name.clone(),
                level,
                effective_hp,
                has_portrait,
                portrait_path,
            })
        })
        .collect();

    resolved.truncate(MAX_PARTY_DISPLAY);
    resolved
}

// ─── Plugin ───────────────────────────────────────────────────────────────────

pub struct StatusScenePlugin;

impl Plugin for StatusScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppPhase::Status), spawn_status_ui)
            .add_systems(OnExit(AppPhase::Status), despawn_status_ui)
            .add_systems(Update, status_input.run_if(in_state(AppPhase::Status)));
    }
}

// ─── Systems ──────────────────────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
fn spawn_status_ui(
    mut commands: Commands,
    character_registry: Option<Res<CharacterRegistryRes>>,
    item_registry: Option<Res<ItemRegistryRes>>,
    ability_registry: Option<Res<AbilityRegistryRes>>,
    party: Option<Res<PartyState>>,
    inventory: Option<Res<InventoryState>>,
    progress: Option<Res<CharacterProgressState>>,
) {
    let Some(character_registry) = character_registry else {
        warn!("StatusScenePlugin: No CharacterRegistryRes resource found");
        return;
    };
    let Some(item_registry) = item_registry else {
        warn!("StatusScenePlugin: No ItemRegistryRes resource found");
        return;
    };
    let Some(_ability_registry) = ability_registry else {
        warn!("StatusScenePlugin: No AbilityRegistryRes resource found");
        return;
    };

    // Read party state (default to empty if not present)
    let party_members = party.as_ref().map(|p| p.members.as_slice()).unwrap_or(&[]);

    // Read progress state (default to empty if not present)
    let progress_map = progress
        .as_ref()
        .map(|p| p.characters.clone())
        .unwrap_or_default();

    // Resolve party display data
    let party_data =
        resolve_party_display_data(party_members, &character_registry.registry, &progress_map);

    // Read inventory state (default to empty if not present)
    let inventory_items = inventory
        .as_ref()
        .map(|i| &i.items)
        .cloned()
        .unwrap_or_default();

    // Resolve initial inventory tab (Weapon)
    let inventory_data = resolve_inventory_tab_data(
        &inventory_items,
        &item_registry.registry,
        InventoryTab::Weapon,
    );

    // Insert StatusUiState with defaults
    commands.insert_resource(StatusUiState {
        mode: StatusMode::PartyList,
        detail_view: DetailView::None,
        party_selection: 0,
        inventory_selection: 0,
        inventory_tab: InventoryTab::Weapon,
        party_data: party_data.clone(),
        inventory_data: inventory_data.clone(),
    });

    // Spawn root UI node with StatusSceneMarker and full Party List hierarchy
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
            StatusSceneMarker,
        ))
        .with_children(|parent| {
            // Header text
            parent.spawn((
                Text::new("Status"),
                TextFont {
                    font_size: 36.0,
                    ..default()
                },
                TextColor(STATUS_COLOR_HEADER),
            ));

            // Sub-page tab indicator ("Party | Inventory")
            parent.spawn((
                Text::new("[Party]  Inventory"),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(STATUS_COLOR_TAB_ACTIVE),
                SubPageTabIndicator,
            ));

            // Party List Container
            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::FlexStart,
                        row_gap: Val::Px(6.0),
                        width: Val::Percent(80.0),
                        ..default()
                    },
                    PartyListContainer,
                ))
                .with_children(|list_parent| {
                    if party_data.is_empty() {
                        // Empty party indicator
                        list_parent.spawn((
                            Text::new("No party members"),
                            TextFont {
                                font_size: 20.0,
                                ..default()
                            },
                            TextColor(STATUS_COLOR_DIMMED),
                        ));
                    } else {
                        // Spawn a row for each party member
                        for (i, member) in party_data.iter().enumerate() {
                            let is_selected = i == 0;
                            let bg_color = if is_selected {
                                STATUS_COLOR_SELECTED_BG
                            } else {
                                Color::NONE
                            };

                            list_parent
                                .spawn((
                                    Node {
                                        flex_direction: FlexDirection::Row,
                                        align_items: AlignItems::Center,
                                        column_gap: Val::Px(12.0),
                                        padding: UiRect::all(Val::Px(6.0)),
                                        width: Val::Percent(100.0),
                                        ..default()
                                    },
                                    BackgroundColor(bg_color),
                                    PartyMemberRow(i),
                                ))
                                .with_children(|row| {
                                    // Portrait placeholder
                                    let portrait_text =
                                        if member.has_portrait { "[P]" } else { "[?]" };
                                    row.spawn((
                                        Text::new(portrait_text),
                                        TextFont {
                                            font_size: 20.0,
                                            ..default()
                                        },
                                        TextColor(STATUS_COLOR_DIMMED),
                                    ));

                                    // Display name
                                    row.spawn((
                                        Text::new(member.display_name.clone()),
                                        TextFont {
                                            font_size: 20.0,
                                            ..default()
                                        },
                                        TextColor(STATUS_COLOR_NORMAL),
                                    ));

                                    // Level text
                                    row.spawn((
                                        Text::new(format!("Lv {}", member.level)),
                                        TextFont {
                                            font_size: 20.0,
                                            ..default()
                                        },
                                        TextColor(STATUS_COLOR_NORMAL),
                                    ));

                                    // HP text
                                    row.spawn((
                                        Text::new(format!("HP: {}", member.effective_hp)),
                                        TextFont {
                                            font_size: 20.0,
                                            ..default()
                                        },
                                        TextColor(STATUS_COLOR_NORMAL),
                                    ));
                                });
                        }
                    }
                });

            // Character Detail Panel (hidden initially, populated by status_input when entering detail view)
            parent.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Row,
                    display: Display::None,
                    padding: UiRect::all(Val::Px(10.0)),
                    column_gap: Val::Px(20.0),
                    ..default()
                },
                CharacterDetailPanel,
            ));

            // Inventory Container (hidden by default since mode starts at PartyList)
            parent
                .spawn((
                    Node {
                        display: Display::None,
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::FlexStart,
                        row_gap: Val::Px(8.0),
                        width: Val::Percent(80.0),
                        ..default()
                    },
                    InventoryListContainer,
                ))
                .with_children(|inv_parent| {
                    // Tab Bar
                    inv_parent
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(16.0),
                            ..default()
                        })
                        .with_children(|tab_bar| {
                            for tab in all_tabs() {
                                let is_active = tab == InventoryTab::Weapon;
                                let color = if is_active {
                                    STATUS_COLOR_TAB_ACTIVE
                                } else {
                                    STATUS_COLOR_DIMMED
                                };
                                tab_bar.spawn((
                                    Text::new(tab_label(tab)),
                                    TextFont {
                                        font_size: 20.0,
                                        ..default()
                                    },
                                    TextColor(color),
                                    InventoryTabIndicator,
                                ));
                            }
                        });

                    // Item List
                    inv_parent
                        .spawn(Node {
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(4.0),
                            width: Val::Percent(100.0),
                            ..default()
                        })
                        .with_children(|item_list| {
                            if inventory_data.is_empty() {
                                item_list.spawn((
                                    Text::new("No items in this category"),
                                    TextFont {
                                        font_size: 20.0,
                                        ..default()
                                    },
                                    TextColor(STATUS_COLOR_DIMMED),
                                ));
                            } else {
                                for (i, item) in inventory_data.iter().enumerate() {
                                    let is_selected = i == 0;
                                    let bg_color = if is_selected {
                                        STATUS_COLOR_SELECTED_BG
                                    } else {
                                        Color::NONE
                                    };

                                    let icon_text = if item.has_icon { "[I]" } else { "[?]" };
                                    let row_text = format!(
                                        "{}  {}    x{}",
                                        icon_text, item.display_name, item.quantity
                                    );

                                    item_list.spawn((
                                        Text::new(row_text),
                                        TextFont {
                                            font_size: 20.0,
                                            ..default()
                                        },
                                        TextColor(STATUS_COLOR_NORMAL),
                                        BackgroundColor(bg_color),
                                    ));
                                }
                            }
                        });

                    // Detail Panel
                    inv_parent
                        .spawn((
                            Node {
                                flex_direction: FlexDirection::Column,
                                row_gap: Val::Px(4.0),
                                width: Val::Percent(100.0),
                                padding: UiRect::top(Val::Px(10.0)),
                                ..default()
                            },
                            InventoryDetailPanel,
                        ))
                        .with_children(|detail| {
                            if let Some(first_item) = inventory_data.first() {
                                // Item description
                                detail.spawn((
                                    Text::new(first_item.description.clone()),
                                    TextFont {
                                        font_size: 18.0,
                                        ..default()
                                    },
                                    TextColor(STATUS_COLOR_NORMAL),
                                ));

                                // Stat modifiers
                                for (stat_name, value) in &first_item.stat_modifiers {
                                    let (sign, color) = if *value >= 0 {
                                        ("+", STATUS_COLOR_POSITIVE_MOD)
                                    } else {
                                        ("", STATUS_COLOR_NEGATIVE_MOD)
                                    };
                                    let mod_text = format!("{}{} {}", sign, value, stat_name);
                                    detail.spawn((
                                        Text::new(mod_text),
                                        TextFont {
                                            font_size: 18.0,
                                            ..default()
                                        },
                                        TextColor(color),
                                    ));
                                }
                            }
                        });
                });
        });
}

fn despawn_status_ui(mut commands: Commands, query: Query<Entity, With<StatusSceneMarker>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
    commands.remove_resource::<StatusUiState>();
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn status_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut ui_state: Option<ResMut<StatusUiState>>,
    mut next_phase: ResMut<NextState<AppPhase>>,
    inventory: Option<Res<InventoryState>>,
    item_registry: Option<Res<ItemRegistryRes>>,
    mut party_rows: Query<(&PartyMemberRow, &mut BackgroundColor)>,
    mut party_container: Query<&mut Node, With<PartyListContainer>>,
    mut inventory_container: Query<
        (Entity, &mut Node),
        (
            With<InventoryListContainer>,
            Without<PartyListContainer>,
            Without<CharacterDetailPanel>,
        ),
    >,
    mut detail_panel: Query<
        (Entity, &mut Node),
        (
            With<CharacterDetailPanel>,
            Without<PartyListContainer>,
            Without<InventoryListContainer>,
        ),
    >,
    mut sub_page_text: Query<&mut Text, With<SubPageTabIndicator>>,
    mut tab_indicators: Query<(&mut TextColor, &InventoryTabIndicator)>,
    character_registry: Option<Res<CharacterRegistryRes>>,
    ability_registry: Option<Res<AbilityRegistryRes>>,
    progress: Option<Res<CharacterProgressState>>,
    children_query: Query<&Children>,
    mut commands: Commands,
) {
    let Some(ref mut ui_state) = ui_state else {
        return;
    };

    // Escape / Backspace handling
    if keyboard.just_pressed(KeyCode::Escape) || keyboard.just_pressed(KeyCode::Backspace) {
        match ui_state.detail_view {
            DetailView::None => {
                // At top level — exit to InGame
                next_phase.set(AppPhase::InGame);
                return;
            }
            DetailView::CharacterDetail => {
                // In character detail — return to PartyList
                ui_state.detail_view = DetailView::None;
                // Show party list, hide detail panel
                if let Ok(mut node) = party_container.single_mut() {
                    node.display = Display::Flex;
                }
                if let Ok((_, mut node)) = detail_panel.single_mut() {
                    node.display = Display::None;
                }
                return;
            }
        }
    }

    // Determine list length based on current mode
    let list_len = match ui_state.mode {
        StatusMode::PartyList => ui_state.party_data.len(),
        StatusMode::Inventory => ui_state.inventory_data.len(),
    };

    // Up / Down navigation (skip if list is empty)
    if list_len > 0 {
        if keyboard.just_pressed(KeyCode::ArrowUp) || keyboard.just_pressed(KeyCode::KeyW) {
            match ui_state.mode {
                StatusMode::PartyList => {
                    let idx = ui_state.party_selection.saturating_sub(1);
                    ui_state.party_selection = clamp_selection(idx, list_len);
                    update_party_row_highlights(ui_state, &mut party_rows);
                }
                StatusMode::Inventory => {
                    let idx = ui_state.inventory_selection.saturating_sub(1);
                    ui_state.inventory_selection = clamp_selection(idx, list_len);
                    update_inventory_detail_panel(
                        ui_state,
                        &mut commands,
                        &inventory_container,
                        &children_query,
                    );
                }
            }
            return;
        }

        if keyboard.just_pressed(KeyCode::ArrowDown) || keyboard.just_pressed(KeyCode::KeyS) {
            match ui_state.mode {
                StatusMode::PartyList => {
                    let idx = ui_state.party_selection + 1;
                    ui_state.party_selection = clamp_selection(idx, list_len);
                    update_party_row_highlights(ui_state, &mut party_rows);
                }
                StatusMode::Inventory => {
                    let idx = ui_state.inventory_selection + 1;
                    ui_state.inventory_selection = clamp_selection(idx, list_len);
                    update_inventory_detail_panel(
                        ui_state,
                        &mut commands,
                        &inventory_container,
                        &children_query,
                    );
                }
            }
            return;
        }
    }

    // Left / Right navigation
    if keyboard.just_pressed(KeyCode::ArrowLeft) || keyboard.just_pressed(KeyCode::KeyA) {
        match ui_state.mode {
            StatusMode::PartyList => {
                // Already at leftmost sub-page, do nothing
            }
            StatusMode::Inventory => {
                if ui_state.detail_view == DetailView::None {
                    // Switch tab or switch sub-page
                    let new_tab = prev_tab(ui_state.inventory_tab);
                    if new_tab == ui_state.inventory_tab {
                        // Already at first tab — switch to PartyList mode
                        ui_state.mode = StatusMode::PartyList;
                        // Hide inventory, show party list
                        if let Ok((_, mut node)) = inventory_container.single_mut() {
                            node.display = Display::None;
                        }
                        if let Ok(mut node) = party_container.single_mut() {
                            node.display = Display::Flex;
                        }
                        update_sub_page_text(ui_state, &mut sub_page_text);
                        update_party_row_highlights(ui_state, &mut party_rows);
                    } else {
                        ui_state.inventory_tab = new_tab;
                        // Re-resolve inventory data for new tab
                        if let Some(ref item_reg) = item_registry {
                            let inv_items = inventory
                                .as_ref()
                                .map(|i| &i.items)
                                .cloned()
                                .unwrap_or_default();
                            ui_state.inventory_data = resolve_inventory_tab_data(
                                &inv_items,
                                &item_reg.registry,
                                ui_state.inventory_tab,
                            );
                            ui_state.inventory_selection =
                                clamp_selection(0, ui_state.inventory_data.len());
                        }
                        update_tab_highlights(ui_state, &mut tab_indicators);
                        rebuild_inventory_items(
                            ui_state,
                            &mut commands,
                            &inventory_container,
                            &children_query,
                        );
                    }
                }
            }
        }
        return;
    }

    if keyboard.just_pressed(KeyCode::ArrowRight) || keyboard.just_pressed(KeyCode::KeyD) {
        match ui_state.mode {
            StatusMode::PartyList => {
                if ui_state.detail_view == DetailView::None {
                    // Switch to Inventory mode
                    ui_state.mode = StatusMode::Inventory;
                    // Hide party list, show inventory
                    if let Ok(mut node) = party_container.single_mut() {
                        node.display = Display::None;
                    }
                    if let Ok((_, mut node)) = inventory_container.single_mut() {
                        node.display = Display::Flex;
                    }
                    update_sub_page_text(ui_state, &mut sub_page_text);
                    update_tab_highlights(ui_state, &mut tab_indicators);
                }
            }
            StatusMode::Inventory => {
                if ui_state.detail_view == DetailView::None {
                    // Switch to next tab
                    let new_tab = next_tab(ui_state.inventory_tab);
                    if new_tab != ui_state.inventory_tab {
                        ui_state.inventory_tab = new_tab;
                        // Re-resolve inventory data for new tab
                        if let Some(ref item_reg) = item_registry {
                            let inv_items = inventory
                                .as_ref()
                                .map(|i| &i.items)
                                .cloned()
                                .unwrap_or_default();
                            ui_state.inventory_data = resolve_inventory_tab_data(
                                &inv_items,
                                &item_reg.registry,
                                ui_state.inventory_tab,
                            );
                            ui_state.inventory_selection =
                                clamp_selection(0, ui_state.inventory_data.len());
                        }
                        update_tab_highlights(ui_state, &mut tab_indicators);
                        rebuild_inventory_items(
                            ui_state,
                            &mut commands,
                            &inventory_container,
                            &children_query,
                        );
                    }
                    // Already at last tab — do nothing
                }
            }
        }
        return;
    }

    // Enter / Space — confirm action (skip if list is empty)
    if list_len > 0
        && (keyboard.just_pressed(KeyCode::Enter) || keyboard.just_pressed(KeyCode::Space))
    {
        match ui_state.mode {
            StatusMode::PartyList => {
                if ui_state.detail_view == DetailView::None {
                    ui_state.detail_view = DetailView::CharacterDetail;
                    // Hide party list, show character detail panel
                    if let Ok(mut node) = party_container.single_mut() {
                        node.display = Display::None;
                    }
                    populate_character_detail(
                        ui_state,
                        &mut commands,
                        &mut detail_panel,
                        &character_registry,
                        &item_registry,
                        &ability_registry,
                        &progress,
                        &children_query,
                    );
                }
            }
            StatusMode::Inventory => {
                // Read-only — no action on confirm
            }
        }
    }
}

// ─── UI Update Helpers ────────────────────────────────────────────────────────

/// Updates background color on party member rows to highlight the selected row.
fn update_party_row_highlights(
    ui_state: &StatusUiState,
    party_rows: &mut Query<(&PartyMemberRow, &mut BackgroundColor)>,
) {
    for (row, mut bg) in party_rows.iter_mut() {
        if row.0 == ui_state.party_selection {
            *bg = BackgroundColor(STATUS_COLOR_SELECTED_BG);
        } else {
            *bg = BackgroundColor(Color::NONE);
        }
    }
}

/// Updates the sub-page tab indicator text to reflect current mode.
fn update_sub_page_text(
    ui_state: &StatusUiState,
    sub_page_text: &mut Query<&mut Text, With<SubPageTabIndicator>>,
) {
    let label = match ui_state.mode {
        StatusMode::PartyList => "[Party]  Inventory",
        StatusMode::Inventory => "Party  [Inventory]",
    };
    if let Ok(mut text) = sub_page_text.single_mut() {
        **text = label.to_string();
    }
}

/// Updates the inventory tab indicator colors to highlight the active tab.
fn update_tab_highlights(
    ui_state: &StatusUiState,
    tab_indicators: &mut Query<(&mut TextColor, &InventoryTabIndicator)>,
) {
    let active_label = tab_label(ui_state.inventory_tab);
    for (mut color, _indicator) in tab_indicators.iter_mut() {
        // We match by checking if the tab label corresponds to the active tab.
        // Since InventoryTabIndicator is a unit struct marker on each tab text,
        // we identify which tab each indicator is for by iterating in spawn order.
        // However, we don't have the tab identity stored on the indicator component.
        // We'll use a simpler approach: reset all to dimmed, then set active to highlighted.
        *color = TextColor(STATUS_COLOR_DIMMED);
    }
    // We need to identify the active tab by index. Use all_tabs() order.
    let active_index = all_tabs()
        .iter()
        .position(|&t| tab_label(t) == active_label)
        .unwrap_or(0);
    for (i, (mut color, _)) in tab_indicators.iter_mut().enumerate() {
        if i == active_index {
            *color = TextColor(STATUS_COLOR_TAB_ACTIVE);
        }
    }
}

/// Updates the inventory display when selection changes.
/// Rebuilds the inventory container to reflect the new highlight and detail.
#[allow(clippy::type_complexity)]
fn update_inventory_detail_panel(
    ui_state: &StatusUiState,
    commands: &mut Commands,
    inventory_container: &Query<
        (Entity, &mut Node),
        (
            With<InventoryListContainer>,
            Without<PartyListContainer>,
            Without<CharacterDetailPanel>,
        ),
    >,
    children_query: &Query<&Children>,
) {
    rebuild_inventory_items(ui_state, commands, inventory_container, children_query);
}

/// Rebuilds the inventory items and detail panel inside InventoryListContainer.
/// Despawns children of the inventory container (except the tab bar) and respawns items + detail.
#[allow(clippy::type_complexity)]
fn rebuild_inventory_items(
    ui_state: &StatusUiState,
    commands: &mut Commands,
    inventory_container: &Query<
        (Entity, &mut Node),
        (
            With<InventoryListContainer>,
            Without<PartyListContainer>,
            Without<CharacterDetailPanel>,
        ),
    >,
    children_query: &Query<&Children>,
) {
    let Ok((inv_entity, _)) = inventory_container.single() else {
        return;
    };

    // Despawn all children of InventoryListContainer
    if let Ok(children) = children_query.get(inv_entity) {
        for child in children.iter() {
            commands.entity(child).despawn();
        }
    }

    // Rebuild the inventory container content: tab bar + item list + detail panel
    commands.entity(inv_entity).with_children(|inv_parent| {
        // Tab Bar
        inv_parent
            .spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(16.0),
                ..default()
            })
            .with_children(|tab_bar| {
                for tab in all_tabs() {
                    let is_active = tab == ui_state.inventory_tab;
                    let color = if is_active {
                        STATUS_COLOR_TAB_ACTIVE
                    } else {
                        STATUS_COLOR_DIMMED
                    };
                    tab_bar.spawn((
                        Text::new(tab_label(tab)),
                        TextFont {
                            font_size: 20.0,
                            ..default()
                        },
                        TextColor(color),
                        InventoryTabIndicator,
                    ));
                }
            });

        // Item List
        inv_parent
            .spawn(Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(4.0),
                width: Val::Percent(100.0),
                ..default()
            })
            .with_children(|item_list| {
                if ui_state.inventory_data.is_empty() {
                    item_list.spawn((
                        Text::new("No items in this category"),
                        TextFont {
                            font_size: 20.0,
                            ..default()
                        },
                        TextColor(STATUS_COLOR_DIMMED),
                    ));
                } else {
                    for (i, item) in ui_state.inventory_data.iter().enumerate() {
                        let is_selected = i == ui_state.inventory_selection;
                        let bg_color = if is_selected {
                            STATUS_COLOR_SELECTED_BG
                        } else {
                            Color::NONE
                        };

                        let icon_text = if item.has_icon { "[I]" } else { "[?]" };
                        let row_text =
                            format!("{}  {}    x{}", icon_text, item.display_name, item.quantity);

                        item_list.spawn((
                            Text::new(row_text),
                            TextFont {
                                font_size: 20.0,
                                ..default()
                            },
                            TextColor(STATUS_COLOR_NORMAL),
                            BackgroundColor(bg_color),
                        ));
                    }
                }
            });

        // Detail Panel
        inv_parent
            .spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(4.0),
                    width: Val::Percent(100.0),
                    padding: UiRect::top(Val::Px(10.0)),
                    ..default()
                },
                InventoryDetailPanel,
            ))
            .with_children(|detail| {
                if let Some(selected_item) =
                    ui_state.inventory_data.get(ui_state.inventory_selection)
                {
                    // Item description
                    detail.spawn((
                        Text::new(selected_item.description.clone()),
                        TextFont {
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(STATUS_COLOR_NORMAL),
                    ));

                    // Stat modifiers
                    for (stat_name, value) in &selected_item.stat_modifiers {
                        let (sign, color) = if *value >= 0 {
                            ("+", STATUS_COLOR_POSITIVE_MOD)
                        } else {
                            ("", STATUS_COLOR_NEGATIVE_MOD)
                        };
                        let mod_text = format!("{}{} {}", sign, value, stat_name);
                        detail.spawn((
                            Text::new(mod_text),
                            TextFont {
                                font_size: 18.0,
                                ..default()
                            },
                            TextColor(color),
                        ));
                    }
                }
            });
    });
}

/// Populates the character detail panel with data from the selected party member.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn populate_character_detail(
    ui_state: &StatusUiState,
    commands: &mut Commands,
    detail_panel: &mut Query<
        (Entity, &mut Node),
        (
            With<CharacterDetailPanel>,
            Without<PartyListContainer>,
            Without<InventoryListContainer>,
        ),
    >,
    character_registry: &Option<Res<CharacterRegistryRes>>,
    item_registry: &Option<Res<ItemRegistryRes>>,
    ability_registry: &Option<Res<AbilityRegistryRes>>,
    progress: &Option<Res<CharacterProgressState>>,
    children_query: &Query<&Children>,
) {
    let Ok((detail_entity, mut node)) = detail_panel.single_mut() else {
        return;
    };

    // Show the detail panel
    node.display = Display::Flex;

    // Despawn existing children of the detail panel
    if let Ok(children) = children_query.get(detail_entity) {
        for child in children.iter() {
            commands.entity(child).despawn();
        }
    }

    // Get selected party member data
    if ui_state.party_selection >= ui_state.party_data.len() {
        return;
    }
    let member = &ui_state.party_data[ui_state.party_selection];

    let Some(char_reg) = character_registry else {
        return;
    };
    let Some(character) = char_reg.registry.characters.get(&member.character_id) else {
        return;
    };

    // Resolve level
    let level = character
        .stats
        .iter()
        .find(|s| s.name == "Level")
        .map(|s| s.base_value)
        .unwrap_or(1);

    // Resolve equipment names
    let equipment_names: Vec<String> = if let Some(item_reg) = item_registry {
        resolve_ordered_ids(&character.starting_equipment, |id| {
            item_reg.registry.items.contains_key(id)
        })
        .iter()
        .filter_map(|id| {
            item_reg
                .registry
                .items
                .get(id)
                .map(|item| item.display_name.clone())
        })
        .collect()
    } else {
        vec![]
    };

    // Resolve ability names
    let ability_names: Vec<String> =
        if let (Some(prog), Some(abil_reg)) = (progress, ability_registry) {
            let learned = prog
                .characters
                .get(&member.character_id)
                .map(|p| p.learned_abilities.clone())
                .unwrap_or_default();
            resolve_ordered_ids(&learned, |id| abil_reg.registry.abilities.contains_key(id))
                .iter()
                .filter_map(|id| {
                    abil_reg
                        .registry
                        .abilities
                        .get(id)
                        .map(|a| a.display_name.clone())
                })
                .collect()
        } else {
            vec![]
        };

    // Spawn children for detail panel
    commands.entity(detail_entity).with_children(|parent| {
        // Left column: portrait placeholder
        parent
            .spawn(Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::FlexStart,
                width: Val::Px(120.0),
                ..default()
            })
            .with_children(|col| {
                let portrait_text = if character.visual_assets.face_portrait.is_some() {
                    "[Portrait]"
                } else {
                    "[No Portrait]"
                };
                col.spawn((
                    Text::new(portrait_text),
                    TextFont {
                        font_size: 18.0,
                        ..default()
                    },
                    TextColor(STATUS_COLOR_DIMMED),
                ));
            });

        // Right column: name+level, stats, equipment, abilities
        parent
            .spawn(Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(6.0),
                flex_grow: 1.0,
                ..default()
            })
            .with_children(|col| {
                // Name + Level
                col.spawn((
                    Text::new(format!("{}  Lv {}", character.display_name, level)),
                    TextFont {
                        font_size: 22.0,
                        ..default()
                    },
                    TextColor(STATUS_COLOR_HEADER),
                ));

                // Stats (excluding "Level")
                for stat in &character.stats {
                    if stat.name == "Level" {
                        continue;
                    }
                    let effective =
                        compute_effective_stat(stat.base_value, stat.growth_value, level);
                    col.spawn((
                        Text::new(format!("{}: {}", stat.name, effective)),
                        TextFont {
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(STATUS_COLOR_NORMAL),
                    ));
                }

                // Equipment header + names
                col.spawn((
                    Text::new("Equipment:"),
                    TextFont {
                        font_size: 18.0,
                        ..default()
                    },
                    TextColor(STATUS_COLOR_HEADER),
                ));
                if equipment_names.is_empty() {
                    col.spawn((
                        Text::new("  (none)"),
                        TextFont {
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(STATUS_COLOR_DIMMED),
                    ));
                } else {
                    for name in &equipment_names {
                        col.spawn((
                            Text::new(format!("  {}", name)),
                            TextFont {
                                font_size: 18.0,
                                ..default()
                            },
                            TextColor(STATUS_COLOR_NORMAL),
                        ));
                    }
                }

                // Abilities header + names
                col.spawn((
                    Text::new("Abilities:"),
                    TextFont {
                        font_size: 18.0,
                        ..default()
                    },
                    TextColor(STATUS_COLOR_HEADER),
                ));
                if ability_names.is_empty() {
                    col.spawn((
                        Text::new("  (none)"),
                        TextFont {
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(STATUS_COLOR_DIMMED),
                    ));
                } else {
                    for name in &ability_names {
                        col.spawn((
                            Text::new(format!("  {}", name)),
                            TextFont {
                                font_size: 18.0,
                                ..default()
                            },
                            TextColor(STATUS_COLOR_NORMAL),
                        ));
                    }
                }
            });
    });
}

/// Resolves inventory items for a given category tab.
/// Skips items not found in the registry. Sorts case-insensitively by display name.
pub fn resolve_inventory_tab_data(
    inventory: &HashMap<String, u32>,
    item_registry: &ItemRegistry,
    tab: InventoryTab,
) -> Vec<InventoryItemDisplayData> {
    let target_category = tab_to_category(tab);

    let mut results: Vec<InventoryItemDisplayData> = inventory
        .iter()
        .filter_map(|(item_id, &quantity)| {
            let item = item_registry.items.get(item_id)?;
            if item.category() != target_category {
                return None;
            }
            Some(InventoryItemDisplayData {
                item_id: item_id.clone(),
                display_name: item.display_name.clone(),
                quantity,
                has_icon: item.graphics.icon.is_some(),
                icon_path: item.graphics.icon.clone(),
                description: item.description.clone(),
                stat_modifiers: item
                    .stat_modifiers
                    .iter()
                    .map(|m| (m.stat_name.clone(), m.value))
                    .collect(),
            })
        })
        .collect();

    results.sort_by(|a, b| {
        a.display_name
            .to_lowercase()
            .cmp(&b.display_name.to_lowercase())
    });

    results
}
