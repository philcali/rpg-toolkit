use bevy::prelude::*;
use rpg_toolkit_common::{AppPhase, SaveFile, SpawnPoint};
use std::collections::HashMap;
use std::path::PathBuf;

// ─── Resources ────────────────────────────────────────────────────────────────
// These mirror the renderer's resource types. The launcher (task 7.1) will
// ensure that both the scenes crate and the renderer operate on the SAME Bevy
// resources by moving shared types to common or using type aliases. For now,
// we define them here so the TitleScreenPlugin can compile and function
// independently.

/// Configuration resource inserted by the launcher before adding this plugin.
/// Provides the save file path and project spawn point.
#[derive(Resource)]
pub struct TitleScreenConfig {
    pub save_path: PathBuf,
    pub spawn_point: Option<SpawnPoint>,
}

/// Persistent game state flags (key-value store).
#[derive(Resource, Default)]
pub struct GameState {
    pub flags: HashMap<String, String>,
}

/// Player's current currency balance.
#[derive(Resource, Default, Debug, Clone, PartialEq, Eq)]
pub struct CurrencyState {
    pub balance: u64,
}

/// Player's inventory: item_id → quantity held.
#[derive(Resource, Default, Debug, Clone, PartialEq, Eq)]
pub struct InventoryState {
    pub items: HashMap<String, u32>,
}

/// Active party members (ordered list).
#[derive(Resource, Default, Debug, Clone, PartialEq, Eq)]
pub struct PartyState {
    pub members: Vec<String>,
}

/// Per-character experience and learned abilities.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CharacterProgress {
    pub experience: u64,
    pub learned_abilities: Vec<String>,
}

/// Progress state for all characters.
#[derive(Resource, Default, Debug, Clone, PartialEq, Eq)]
pub struct CharacterProgressState {
    pub characters: HashMap<String, CharacterProgress>,
}

/// Runtime state for the renderer — active map and pending transitions.
#[derive(Resource, Default)]
pub struct RendererState {
    pub active_map_id: Option<String>,
    pub pending_map_change: Option<String>,
    pub pending_target_coords: Option<(u32, u32)>,
    pub pending_target_elevation: Option<u32>,
}

// ─── Title Screen Internal State ──────────────────────────────────────────────

/// Marker component for all title screen UI entities (used for despawn on exit).
#[derive(Component)]
struct TitleScreenEntity;

/// Marker for the "New Game" text node.
#[derive(Component)]
struct NewGameOption;

/// Marker for the "Continue" text node.
#[derive(Component)]
struct ContinueOption;

/// Marker for the error message text node.
#[derive(Component)]
struct ErrorMessage;

/// Local resource tracking the title screen's selection state.
#[derive(Resource)]
struct TitleScreenState {
    /// 0 = New Game, 1 = Continue
    selected: usize,
    /// Whether the Continue option is available.
    continue_available: bool,
}

// ─── Type aliases for complex query types ─────────────────────────────────────

type NewGameQuery<'w, 's> = Query<
    'w,
    's,
    (&'static mut Text, &'static mut TextColor),
    (
        With<NewGameOption>,
        Without<ContinueOption>,
        Without<ErrorMessage>,
    ),
>;

type ContinueQuery<'w, 's> = Query<
    'w,
    's,
    (&'static mut Text, &'static mut TextColor),
    (
        With<ContinueOption>,
        Without<NewGameOption>,
        Without<ErrorMessage>,
    ),
>;

type ErrorQuery<'w, 's> = Query<
    'w,
    's,
    (&'static mut Text, &'static mut TextColor),
    (
        With<ErrorMessage>,
        Without<NewGameOption>,
        Without<ContinueOption>,
    ),
>;

// ─── Plugin ───────────────────────────────────────────────────────────────────

pub struct TitleScreenPlugin;

impl Plugin for TitleScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppPhase::TitleScreen), spawn_title_screen)
            .add_systems(OnExit(AppPhase::TitleScreen), despawn_title_screen)
            .add_systems(
                Update,
                title_screen_input.run_if(in_state(AppPhase::TitleScreen)),
            );
    }
}

// ─── Colors ───────────────────────────────────────────────────────────────────

const COLOR_SELECTED: Color = Color::srgb(1.0, 1.0, 0.0);
const COLOR_NORMAL: Color = Color::srgb(1.0, 1.0, 1.0);
const COLOR_DISABLED: Color = Color::srgb(0.4, 0.4, 0.4);
const COLOR_ERROR: Color = Color::srgb(1.0, 0.3, 0.3);

// ─── Systems ──────────────────────────────────────────────────────────────────

fn spawn_title_screen(mut commands: Commands, config: Res<TitleScreenConfig>) {
    // Determine if Continue is available by attempting to load the save file.
    let continue_available = is_save_available(&config.save_path);

    commands.insert_resource(TitleScreenState {
        selected: 0,
        continue_available,
    });

    // Root full-screen container
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(20.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.05, 0.05, 0.1)),
            TitleScreenEntity,
        ))
        .with_children(|parent| {
            // Title text
            parent.spawn((
                Text::new("RPG Toolkit"),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            // Spacer
            parent.spawn(Node {
                height: Val::Px(40.0),
                ..default()
            });

            // "New Game" option
            parent.spawn((
                Text::new("> New Game"),
                TextFont {
                    font_size: 28.0,
                    ..default()
                },
                TextColor(COLOR_SELECTED),
                NewGameOption,
            ));

            // "Continue" option
            let continue_color = if continue_available {
                COLOR_NORMAL
            } else {
                COLOR_DISABLED
            };
            parent.spawn((
                Text::new("  Continue"),
                TextFont {
                    font_size: 28.0,
                    ..default()
                },
                TextColor(continue_color),
                ContinueOption,
            ));

            // Error message (initially hidden/empty)
            parent.spawn((
                Text::new(""),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(COLOR_ERROR),
                ErrorMessage,
            ));
        });
}

fn despawn_title_screen(mut commands: Commands, query: Query<Entity, With<TitleScreenEntity>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
    commands.remove_resource::<TitleScreenState>();
}

#[allow(clippy::too_many_arguments)]
fn title_screen_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<TitleScreenState>,
    config: Res<TitleScreenConfig>,
    mut next_phase: ResMut<NextState<AppPhase>>,
    mut game_state: ResMut<GameState>,
    mut currency: ResMut<CurrencyState>,
    mut inventory: ResMut<InventoryState>,
    mut party: ResMut<PartyState>,
    mut progress: ResMut<CharacterProgressState>,
    mut renderer_state: ResMut<RendererState>,
    mut new_game_query: NewGameQuery,
    mut continue_query: ContinueQuery,
    mut error_query: ErrorQuery,
) {
    // Navigation: Up/Down or W/S
    if (keyboard.just_pressed(KeyCode::ArrowUp) || keyboard.just_pressed(KeyCode::KeyW))
        && state.selected > 0
    {
        state.selected -= 1;
        update_selection_visuals(&state, &mut new_game_query, &mut continue_query);
    }
    if (keyboard.just_pressed(KeyCode::ArrowDown) || keyboard.just_pressed(KeyCode::KeyS))
        && state.selected < 1
    {
        state.selected += 1;
        update_selection_visuals(&state, &mut new_game_query, &mut continue_query);
    }

    // Confirm: Enter or Space
    if keyboard.just_pressed(KeyCode::Enter) || keyboard.just_pressed(KeyCode::Space) {
        match state.selected {
            0 => {
                // New Game
                handle_new_game(
                    &config,
                    &mut next_phase,
                    &mut game_state,
                    &mut currency,
                    &mut inventory,
                    &mut party,
                    &mut progress,
                    &mut renderer_state,
                    &mut error_query,
                );
            }
            1 if state.continue_available => {
                // Continue
                handle_continue(
                    &config,
                    &mut next_phase,
                    &mut game_state,
                    &mut currency,
                    &mut inventory,
                    &mut party,
                    &mut progress,
                    &mut renderer_state,
                    &mut error_query,
                );
            }
            _ => {}
        }
    }
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

/// Check if a valid save file exists at the given path.
fn is_save_available(path: &PathBuf) -> bool {
    match std::fs::read_to_string(path) {
        Ok(contents) => serde_json::from_str::<SaveFile>(&contents).is_ok(),
        Err(_) => false,
    }
}

fn update_selection_visuals(
    state: &TitleScreenState,
    new_game_query: &mut NewGameQuery,
    continue_query: &mut ContinueQuery,
) {
    if let Ok((mut text, mut color)) = new_game_query.single_mut() {
        if state.selected == 0 {
            **text = "> New Game".to_string();
            *color = TextColor(COLOR_SELECTED);
        } else {
            **text = "  New Game".to_string();
            *color = TextColor(COLOR_NORMAL);
        }
    }
    if let Ok((mut text, mut color)) = continue_query.single_mut() {
        if state.selected == 1 {
            if state.continue_available {
                **text = "> Continue".to_string();
                *color = TextColor(COLOR_SELECTED);
            } else {
                **text = "  Continue".to_string();
                *color = TextColor(COLOR_DISABLED);
            }
        } else if state.continue_available {
            **text = "  Continue".to_string();
            *color = TextColor(COLOR_NORMAL);
        } else {
            **text = "  Continue".to_string();
            *color = TextColor(COLOR_DISABLED);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn handle_new_game(
    config: &TitleScreenConfig,
    next_phase: &mut ResMut<NextState<AppPhase>>,
    game_state: &mut ResMut<GameState>,
    currency: &mut ResMut<CurrencyState>,
    inventory: &mut ResMut<InventoryState>,
    party: &mut ResMut<PartyState>,
    progress: &mut ResMut<CharacterProgressState>,
    renderer_state: &mut ResMut<RendererState>,
    error_query: &mut ErrorQuery,
) {
    let Some(spawn_point) = &config.spawn_point else {
        show_error(error_query, "Error: No spawn point configured in project");
        return;
    };

    // Reset all game state to defaults
    game_state.flags.clear();
    currency.balance = 0;
    inventory.items.clear();
    party.members.clear();
    progress.characters.clear();

    // Set renderer to spawn point
    renderer_state.active_map_id = Some(spawn_point.map_id.clone());
    renderer_state.pending_map_change = None;
    renderer_state.pending_target_coords = Some((spawn_point.x, spawn_point.y));
    renderer_state.pending_target_elevation = Some(0);

    next_phase.set(AppPhase::InGame);
}

#[allow(clippy::too_many_arguments)]
fn handle_continue(
    config: &TitleScreenConfig,
    next_phase: &mut ResMut<NextState<AppPhase>>,
    game_state: &mut ResMut<GameState>,
    currency: &mut ResMut<CurrencyState>,
    inventory: &mut ResMut<InventoryState>,
    party: &mut ResMut<PartyState>,
    progress: &mut ResMut<CharacterProgressState>,
    renderer_state: &mut ResMut<RendererState>,
    error_query: &mut ErrorQuery,
) {
    let save_file = SaveFile::load(&config.save_path);

    // Populate game state from save
    game_state.flags = save_file.state.into_iter().collect();
    currency.balance = save_file.currency;
    inventory.items = save_file.inventory.into_iter().collect();
    party.members = save_file.party;
    progress.characters = save_file
        .character_progress
        .into_iter()
        .map(|(id, data)| {
            (
                id,
                CharacterProgress {
                    experience: data.experience,
                    learned_abilities: data.learned_abilities,
                },
            )
        })
        .collect();

    // Determine map/position from save data, falling back to spawn point
    let (map_id, x, y, elevation) = match (save_file.map_id, save_file.position) {
        (Some(mid), Some((sx, sy))) => (mid, sx, sy, save_file.elevation),
        _ => {
            // Fall back to spawn point
            let Some(spawn_point) = &config.spawn_point else {
                show_error(error_query, "Error: No spawn point configured in project");
                return;
            };
            (
                spawn_point.map_id.clone(),
                spawn_point.x,
                spawn_point.y,
                None,
            )
        }
    };

    renderer_state.active_map_id = Some(map_id);
    renderer_state.pending_map_change = None;
    renderer_state.pending_target_coords = Some((x, y));
    renderer_state.pending_target_elevation = elevation;

    next_phase.set(AppPhase::InGame);
}

fn show_error(error_query: &mut ErrorQuery, message: &str) {
    if let Ok((mut text, mut color)) = error_query.single_mut() {
        **text = message.to_string();
        *color = TextColor(COLOR_ERROR);
    }
}
