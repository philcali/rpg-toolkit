use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

use crate::data::editor_state::{EditCommand, EditCommandKind};
use crate::data::map::{EventAction, MapId, SpawnPoint};
use crate::data::{AnyDialogOpen, AttributeTool, EditorMode, EditorState, Project};
use crate::systems::input::CursorWorldState;
use rpg_toolkit_common::{
    DialogConfigData, DialogPositionData, DialogTextData, FacingDirection, FadeType, NpcInstance,
    PatrolConfig, PatrolMode, PlayerAppearance, ScreenShakeMode, SpritesheetId, TriggerMode,
    validate_waypoint_bounds,
};

/// The type of action being added in the Event Trigger Editor.
#[derive(Default, PartialEq)]
pub enum ActionType {
    #[default]
    JumpTo,
    ShowDialog,
    ScreenShake,
    StopScreenShake,
    FadeTransition,
    SetState,
    SetPlayerAppearance,
}

/// The text source mode for a ShowDialog action.
#[derive(Default, PartialEq)]
pub enum DialogTextMode {
    #[default]
    Inline,
    TextId,
}

/// Resource for the spawn point confirmation dialog.
#[derive(Resource, Default)]
pub struct SpawnPointConfirmDialog {
    pub open: bool,
    pub new_map_id: Option<MapId>,
    pub new_x: u32,
    pub new_y: u32,
}

/// Resource for the event trigger editing dialog.
#[derive(Resource)]
pub struct EventTriggerDialog {
    pub open: bool,
    pub layer_index: usize,
    pub tile_x: u32,
    pub tile_y: u32,
    pub actions: Vec<EventAction>,
    pub original_actions: Vec<EventAction>,
    /// Pending new JumpTo fields
    pub new_target_map_id: String,
    pub new_target_x: String,
    pub new_target_y: String,
    /// Type of action being added: JumpTo or ShowDialog
    pub new_action_type: ActionType,
    /// ShowDialog fields
    pub new_dialog_text_mode: DialogTextMode,
    pub new_dialog_inline_text: String,
    pub new_dialog_text_id: String,
    pub new_dialog_text_speed: String,
    pub new_dialog_position: DialogPositionData,
    pub new_dialog_movement_block: bool,
    /// ScreenShake fields
    pub new_shake_mode: ScreenShakeMode,
    pub new_shake_intensity: String,
    pub new_shake_duration: String,
    /// FadeTransition fields
    pub new_fade_type: FadeType,
    pub new_fade_duration: String,
    pub new_fade_color: [f32; 4],
    /// SetState fields
    pub new_state_key: String,
    pub new_state_value: String,
    /// SetPlayerAppearance fields
    pub new_appearance: PlayerAppearance,
    pub new_appearance_path: String,
    /// Index of the action being edited (None = adding new)
    pub editing_index: Option<usize>,
}

impl Default for EventTriggerDialog {
    fn default() -> Self {
        Self {
            open: false,
            layer_index: 0,
            tile_x: 0,
            tile_y: 0,
            actions: Vec::new(),
            original_actions: Vec::new(),
            new_target_map_id: String::new(),
            new_target_x: String::new(),
            new_target_y: String::new(),
            new_action_type: ActionType::JumpTo,
            new_dialog_text_mode: DialogTextMode::Inline,
            new_dialog_inline_text: String::new(),
            new_dialog_text_id: String::new(),
            new_dialog_text_speed: "30".to_string(),
            new_dialog_position: DialogPositionData::Bottom,
            new_dialog_movement_block: true,
            new_shake_mode: ScreenShakeMode::Timed,
            new_shake_intensity: "5.0".to_string(),
            new_shake_duration: "0.5".to_string(),
            new_fade_type: FadeType::FadeOut,
            new_fade_duration: "1.0".to_string(),
            new_fade_color: [0.0, 0.0, 0.0, 1.0],
            new_state_key: String::new(),
            new_state_value: String::new(),
            new_appearance: PlayerAppearance::Hidden,
            new_appearance_path: String::new(),
            editing_index: None,
        }
    }
}

/// Resource for the NPC placement/editing dialog.
#[derive(Resource)]
pub struct NpcPlacementDialog {
    pub open: bool,
    pub tile_x: u32,
    pub tile_y: u32,
    pub selected_spritesheet_id: Option<SpritesheetId>,
    pub selected_facing: FacingDirection,
    pub editing_index: Option<usize>,
    pub original_npc: Option<NpcInstance>,
    // Patrol config fields
    pub patrol_waypoints: Vec<(u32, u32)>,
    pub patrol_mode: PatrolMode,
    pub patrol_speed: String,
    pub patrol_pause: String,
    pub adding_waypoints: bool,
    // Event trigger fields
    pub trigger_mode: TriggerMode,
    pub event_triggers: Vec<EventAction>,
    // Action editing fields (same as EventTriggerDialog)
    pub npc_new_action_type: ActionType,
    pub npc_new_target_map_id: String,
    pub npc_new_target_x: String,
    pub npc_new_target_y: String,
    pub npc_new_dialog_text_mode: DialogTextMode,
    pub npc_new_dialog_inline_text: String,
    pub npc_new_dialog_text_id: String,
    pub npc_new_dialog_text_speed: String,
    pub npc_new_dialog_position: DialogPositionData,
    pub npc_new_dialog_movement_block: bool,
    pub npc_editing_action_index: Option<usize>,
    // ScreenShake fields
    pub npc_new_shake_mode: ScreenShakeMode,
    pub npc_new_shake_intensity: String,
    pub npc_new_shake_duration: String,
    // FadeTransition fields
    pub npc_new_fade_type: FadeType,
    pub npc_new_fade_duration: String,
    pub npc_new_fade_color: [f32; 4],
    // SetState fields
    pub npc_new_state_key: String,
    pub npc_new_state_value: String,
    // SetPlayerAppearance fields
    pub npc_new_appearance: PlayerAppearance,
    pub npc_new_appearance_path: String,
}

impl Default for NpcPlacementDialog {
    fn default() -> Self {
        Self {
            open: false,
            tile_x: 0,
            tile_y: 0,
            selected_spritesheet_id: None,
            selected_facing: FacingDirection::Down,
            editing_index: None,
            original_npc: None,
            patrol_waypoints: Vec::new(),
            patrol_mode: PatrolMode::Loop,
            patrol_speed: "0.3".to_string(),
            patrol_pause: "0.5".to_string(),
            adding_waypoints: false,
            trigger_mode: TriggerMode::Interaction,
            event_triggers: Vec::new(),
            npc_new_action_type: ActionType::JumpTo,
            npc_new_target_map_id: String::new(),
            npc_new_target_x: "0".to_string(),
            npc_new_target_y: "0".to_string(),
            npc_new_dialog_text_mode: DialogTextMode::Inline,
            npc_new_dialog_inline_text: String::new(),
            npc_new_dialog_text_id: String::new(),
            npc_new_dialog_text_speed: "30".to_string(),
            npc_new_dialog_position: DialogPositionData::Bottom,
            npc_new_dialog_movement_block: true,
            npc_editing_action_index: None,
            npc_new_shake_mode: ScreenShakeMode::Timed,
            npc_new_shake_intensity: "5.0".to_string(),
            npc_new_shake_duration: "0.5".to_string(),
            npc_new_fade_type: FadeType::FadeOut,
            npc_new_fade_duration: "1.0".to_string(),
            npc_new_fade_color: [0.0, 0.0, 0.0, 1.0],
            npc_new_state_key: String::new(),
            npc_new_state_value: String::new(),
            npc_new_appearance: PlayerAppearance::Hidden,
            npc_new_appearance_path: String::new(),
        }
    }
}

pub struct AttributePlugin;

/// Truncates a string to at most `max_len` characters, appending "…" if truncated.
pub fn truncate_preview(s: &str, max_len: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() > max_len {
        let truncated: String = chars[..max_len].iter().collect();
        format!("{}…", truncated)
    } else {
        s.to_string()
    }
}

impl Plugin for AttributePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SpawnPointConfirmDialog>()
            .init_resource::<EventTriggerDialog>()
            .init_resource::<NpcPlacementDialog>()
            .add_systems(
                EguiPrimaryContextPass,
                (
                    event_trigger_panel_ui,
                    spawn_point_confirm_ui,
                    npc_placement_dialog_ui,
                ),
            )
            .add_systems(
                Update,
                (
                    attribute_overlay_system.after(crate::plugins::canvas::draw_grid),
                    attribute_click_system.after(crate::systems::input::update_cursor_state),
                ),
            );
    }
}

/// Draws gizmo overlays on tiles with attributes when in attribute mode.
fn attribute_overlay_system(
    editor_state: Res<EditorState>,
    project: Res<Project>,
    npc_dialog: Res<NpcPlacementDialog>,
    mut gizmos: Gizmos,
) {
    if editor_state.editor_mode != EditorMode::Attribute {
        return;
    }

    let Some(map) = project.active_map() else {
        return;
    };

    let tile = map.tile_width as f32;

    // Draw opacity overlays (red semi-transparent) for the active layer
    if let Some(layer) = map.layers.get(map.active_layer_index) {
        for (y, row) in layer.attributes.cells.iter().enumerate() {
            for (x, attrs) in row.iter().enumerate() {
                if attrs.opacity {
                    let px = x as f32 * tile + tile / 2.0;
                    let py = -(y as f32 * tile + tile / 2.0);
                    gizmos.rect_2d(
                        Isometry2d::from_translation(Vec2::new(px, py)),
                        Vec2::splat(tile),
                        Color::srgba(1.0, 0.0, 0.0, 0.35),
                    );
                }

                if !attrs.event_trigger.is_empty() {
                    let px = x as f32 * tile + tile / 2.0;
                    let py = -(y as f32 * tile + tile / 2.0);
                    gizmos.rect_2d(
                        Isometry2d::from_translation(Vec2::new(px, py)),
                        Vec2::splat(tile * 0.8),
                        Color::srgba(0.0, 0.4, 1.0, 0.45),
                    );
                }
            }
        }
    }

    // Draw spawn point marker if on the current map
    if let Some(ref sp) = project.spawn_point
        && let Some(active_map_id) = project.active_map_id()
        && sp.map_id == *active_map_id
    {
        let px = sp.x as f32 * tile + tile / 2.0;
        let py = -(sp.y as f32 * tile + tile / 2.0);
        gizmos.rect_2d(
            Isometry2d::from_translation(Vec2::new(px, py)),
            Vec2::splat(tile * 0.9),
            Color::srgba(0.0, 1.0, 0.0, 0.5),
        );
        // Draw a cross inside the spawn point marker
        let half = tile * 0.35;
        let center = Vec2::new(px, py);
        gizmos.line_2d(
            center + Vec2::new(-half, -half),
            center + Vec2::new(half, half),
            Color::srgba(0.0, 1.0, 0.0, 0.8),
        );
        gizmos.line_2d(
            center + Vec2::new(-half, half),
            center + Vec2::new(half, -half),
            Color::srgba(0.0, 1.0, 0.0, 0.8),
        );
    }

    // Draw NPC overlays (purple/magenta) when in NPC placement mode
    if editor_state.attribute_tool == AttributeTool::NpcPlacement {
        for npc in &map.npcs {
            let px = npc.x as f32 * tile + tile / 2.0;
            let py = -(npc.y as f32 * tile + tile / 2.0);
            gizmos.rect_2d(
                Isometry2d::from_translation(Vec2::new(px, py)),
                Vec2::splat(tile * 0.85),
                Color::srgba(0.8, 0.2, 0.8, 0.45),
            );
        }
    }

    // Draw patrol path overlay when NPC placement dialog is open
    if editor_state.attribute_tool == AttributeTool::NpcPlacement && npc_dialog.open {
        let waypoints = &npc_dialog.patrol_waypoints;
        let color = Color::srgba(1.0, 0.8, 0.0, 0.8); // Yellow/orange for patrol paths
        let marker_color = Color::srgba(1.0, 0.6, 0.0, 0.9);

        // Draw connected line segments between waypoints
        for i in 0..waypoints.len() {
            let (wx, wy) = waypoints[i];
            let px = wx as f32 * tile + tile / 2.0;
            let py = -(wy as f32 * tile + tile / 2.0);

            // Draw line to next waypoint
            if i + 1 < waypoints.len() {
                let (nx, ny) = waypoints[i + 1];
                let npx = nx as f32 * tile + tile / 2.0;
                let npy = -(ny as f32 * tile + tile / 2.0);
                gizmos.line_2d(Vec2::new(px, py), Vec2::new(npx, npy), color);
            }

            // Draw numbered marker at each waypoint (small rect)
            gizmos.rect_2d(
                Isometry2d::from_translation(Vec2::new(px, py)),
                Vec2::splat(tile * 0.4),
                marker_color,
            );
        }
    }

    // Also draw patrol paths for existing NPCs that have patrol configs
    if editor_state.attribute_tool == AttributeTool::NpcPlacement {
        let path_color = Color::srgba(0.8, 0.6, 0.0, 0.5); // Dimmer for non-selected NPCs
        for npc in &map.npcs {
            if let Some(ref config) = npc.patrol_config {
                for i in 0..config.waypoints.len() {
                    let (wx, wy) = config.waypoints[i];
                    let px = wx as f32 * tile + tile / 2.0;
                    let py = -(wy as f32 * tile + tile / 2.0);

                    if i + 1 < config.waypoints.len() {
                        let (nx, ny) = config.waypoints[i + 1];
                        let npx = nx as f32 * tile + tile / 2.0;
                        let npy = -(ny as f32 * tile + tile / 2.0);
                        gizmos.line_2d(Vec2::new(px, py), Vec2::new(npx, npy), path_color);
                    }

                    gizmos.rect_2d(
                        Isometry2d::from_translation(Vec2::new(px, py)),
                        Vec2::splat(tile * 0.3),
                        path_color,
                    );
                }
            }
        }
    }
}

/// Handles left-click in attribute mode for opacity toggle, event trigger selection,
/// and spawn point placement.
#[allow(clippy::too_many_arguments)]
fn attribute_click_system(
    mouse: Res<ButtonInput<MouseButton>>,
    editor_state: Res<EditorState>,
    cursor_state: Res<CursorWorldState>,
    mut project: ResMut<Project>,
    mut edit_events: MessageWriter<EditCommand>,
    mut event_trigger_dialog: ResMut<EventTriggerDialog>,
    mut spawn_confirm_dialog: ResMut<SpawnPointConfirmDialog>,
    mut npc_placement_dialog: ResMut<NpcPlacementDialog>,
    any_dialog_open: Res<AnyDialogOpen>,
) {
    // Block all attribute clicks when any modal dialog is open,
    // EXCEPT when the NPC dialog is open and we're adding waypoints
    if any_dialog_open.0 {
        // Allow waypoint clicks through when adding waypoints
        if !(npc_placement_dialog.open
            && npc_placement_dialog.adding_waypoints
            && editor_state.attribute_tool == AttributeTool::NpcPlacement)
        {
            return;
        }
    }

    if editor_state.editor_mode != EditorMode::Attribute {
        return;
    }

    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    let Some((col, row)) = cursor_state.tile_pos else {
        return;
    };

    match editor_state.attribute_tool {
        AttributeTool::Opacity => {
            let Some(map) = project.active_map_mut() else {
                return;
            };
            let layer_index = map.active_layer_index;

            let old_value = map
                .layers
                .get(layer_index)
                .and_then(|l| l.attributes.cells.get(row as usize))
                .and_then(|r| r.get(col as usize))
                .map(|a| a.opacity)
                .unwrap_or(false);

            let new_value = !old_value;

            // Apply the change
            if let Some(layer) = map.layers.get_mut(layer_index)
                && let Some(attr_row) = layer.attributes.cells.get_mut(row as usize)
                && let Some(cell) = attr_row.get_mut(col as usize)
            {
                cell.opacity = new_value;
            }

            edit_events.write(EditCommand {
                kind: EditCommandKind::SetOpacity {
                    layer_index,
                    x: col,
                    y: row,
                    old_value,
                    new_value,
                },
            });
        }

        AttributeTool::EventTrigger => {
            let Some(map) = project.active_map() else {
                return;
            };
            let layer_index = map.active_layer_index;

            let existing = map
                .layers
                .get(layer_index)
                .and_then(|l| l.attributes.cells.get(row as usize))
                .and_then(|r| r.get(col as usize))
                .map(|a| a.event_trigger.clone())
                .unwrap_or_default();

            // Open the event trigger dialog populated with existing data
            event_trigger_dialog.open = true;
            event_trigger_dialog.layer_index = layer_index;
            event_trigger_dialog.tile_x = col;
            event_trigger_dialog.tile_y = row;
            event_trigger_dialog.actions = existing.clone();
            event_trigger_dialog.original_actions = existing;
            event_trigger_dialog.new_target_map_id = String::new();
            event_trigger_dialog.new_target_x = "0".to_string();
            event_trigger_dialog.new_target_y = "0".to_string();
            event_trigger_dialog.new_action_type = ActionType::JumpTo;
            event_trigger_dialog.new_dialog_text_mode = DialogTextMode::Inline;
            event_trigger_dialog.new_dialog_inline_text = String::new();
            event_trigger_dialog.new_dialog_text_id = String::new();
            event_trigger_dialog.new_dialog_text_speed = "30".to_string();
            event_trigger_dialog.new_dialog_position = DialogPositionData::Bottom;
            event_trigger_dialog.new_dialog_movement_block = true;
            event_trigger_dialog.editing_index = None;
        }

        AttributeTool::SpawnPoint => {
            let Some(active_map_id) = project.active_map_id().cloned() else {
                return;
            };

            // Check map bounds
            let Some(map) = project.active_map() else {
                return;
            };
            if col >= map.width || row >= map.height {
                return;
            }

            if project.spawn_point.is_some() {
                // Open confirmation dialog
                spawn_confirm_dialog.open = true;
                spawn_confirm_dialog.new_map_id = Some(active_map_id);
                spawn_confirm_dialog.new_x = col;
                spawn_confirm_dialog.new_y = row;
            } else {
                // No existing spawn point — set directly
                let new_spawn = Some(SpawnPoint {
                    map_id: active_map_id,
                    x: col,
                    y: row,
                });

                edit_events.write(EditCommand {
                    kind: EditCommandKind::SetSpawnPoint {
                        old_spawn: None,
                        new_spawn: new_spawn.clone(),
                    },
                });
            }
        }

        AttributeTool::NpcPlacement => {
            // If dialog is open and adding waypoints, append clicked tile
            if npc_placement_dialog.open && npc_placement_dialog.adding_waypoints {
                let Some(map) = project.active_map() else {
                    return;
                };
                if validate_waypoint_bounds((col, row), map.width, map.height) {
                    npc_placement_dialog.patrol_waypoints.push((col, row));
                }
                return;
            }

            let Some(map) = project.active_map() else {
                return;
            };

            // Check if an NPC already exists at this tile
            let existing = map
                .npcs
                .iter()
                .enumerate()
                .find(|(_, npc)| npc.x == col && npc.y == row);

            if let Some((idx, npc)) = existing {
                // Open dialog pre-populated with existing NPC data for editing
                npc_placement_dialog.open = true;
                npc_placement_dialog.tile_x = col;
                npc_placement_dialog.tile_y = row;
                npc_placement_dialog.selected_spritesheet_id = Some(npc.spritesheet_id.clone());
                npc_placement_dialog.selected_facing = npc.facing;
                npc_placement_dialog.editing_index = Some(idx);
                npc_placement_dialog.original_npc = Some(npc.clone());
                // Pre-populate patrol config
                if let Some(ref config) = npc.patrol_config {
                    npc_placement_dialog.patrol_waypoints = config.waypoints.clone();
                    npc_placement_dialog.patrol_mode = config.mode;
                    npc_placement_dialog.patrol_speed = config.speed.to_string();
                    npc_placement_dialog.patrol_pause = config.pause.to_string();
                } else {
                    npc_placement_dialog.patrol_waypoints = Vec::new();
                    npc_placement_dialog.patrol_mode = PatrolMode::Loop;
                    npc_placement_dialog.patrol_speed = "0.3".to_string();
                    npc_placement_dialog.patrol_pause = "0.5".to_string();
                }
                npc_placement_dialog.adding_waypoints = false;
                // Pre-populate event trigger config
                npc_placement_dialog.trigger_mode = npc.trigger_mode;
                npc_placement_dialog.event_triggers = npc.event_triggers.clone();
                npc_placement_dialog.npc_new_action_type = ActionType::JumpTo;
                npc_placement_dialog.npc_new_target_map_id = String::new();
                npc_placement_dialog.npc_new_target_x = "0".to_string();
                npc_placement_dialog.npc_new_target_y = "0".to_string();
                npc_placement_dialog.npc_new_dialog_text_mode = DialogTextMode::Inline;
                npc_placement_dialog.npc_new_dialog_inline_text = String::new();
                npc_placement_dialog.npc_new_dialog_text_id = String::new();
                npc_placement_dialog.npc_new_dialog_text_speed = "30".to_string();
                npc_placement_dialog.npc_new_dialog_position = DialogPositionData::Bottom;
                npc_placement_dialog.npc_new_dialog_movement_block = true;
                npc_placement_dialog.npc_editing_action_index = None;
                npc_placement_dialog.npc_new_shake_mode = ScreenShakeMode::Timed;
                npc_placement_dialog.npc_new_shake_intensity = "5.0".to_string();
                npc_placement_dialog.npc_new_shake_duration = "0.5".to_string();
                npc_placement_dialog.npc_new_fade_type = FadeType::FadeOut;
                npc_placement_dialog.npc_new_fade_duration = "1.0".to_string();
                npc_placement_dialog.npc_new_fade_color = [0.0, 0.0, 0.0, 1.0];
                npc_placement_dialog.npc_new_state_key = String::new();
                npc_placement_dialog.npc_new_state_value = String::new();
                npc_placement_dialog.npc_new_appearance = PlayerAppearance::Hidden;
                npc_placement_dialog.npc_new_appearance_path = String::new();
            } else {
                // Open empty dialog for new placement
                let first_spritesheet = project.spritesheets.keys().next().cloned();
                npc_placement_dialog.open = true;
                npc_placement_dialog.tile_x = col;
                npc_placement_dialog.tile_y = row;
                npc_placement_dialog.selected_spritesheet_id = first_spritesheet;
                npc_placement_dialog.selected_facing = FacingDirection::Down;
                npc_placement_dialog.editing_index = None;
                npc_placement_dialog.original_npc = None;
                npc_placement_dialog.patrol_waypoints = Vec::new();
                npc_placement_dialog.patrol_mode = PatrolMode::Loop;
                npc_placement_dialog.patrol_speed = "0.3".to_string();
                npc_placement_dialog.patrol_pause = "0.5".to_string();
                npc_placement_dialog.adding_waypoints = false;
                npc_placement_dialog.trigger_mode = TriggerMode::Interaction;
                npc_placement_dialog.event_triggers = Vec::new();
                npc_placement_dialog.npc_new_action_type = ActionType::JumpTo;
                npc_placement_dialog.npc_new_target_map_id = String::new();
                npc_placement_dialog.npc_new_target_x = "0".to_string();
                npc_placement_dialog.npc_new_target_y = "0".to_string();
                npc_placement_dialog.npc_new_dialog_text_mode = DialogTextMode::Inline;
                npc_placement_dialog.npc_new_dialog_inline_text = String::new();
                npc_placement_dialog.npc_new_dialog_text_id = String::new();
                npc_placement_dialog.npc_new_dialog_text_speed = "30".to_string();
                npc_placement_dialog.npc_new_dialog_position = DialogPositionData::Bottom;
                npc_placement_dialog.npc_new_dialog_movement_block = true;
                npc_placement_dialog.npc_editing_action_index = None;
                npc_placement_dialog.npc_new_shake_mode = ScreenShakeMode::Timed;
                npc_placement_dialog.npc_new_shake_intensity = "5.0".to_string();
                npc_placement_dialog.npc_new_shake_duration = "0.5".to_string();
                npc_placement_dialog.npc_new_fade_type = FadeType::FadeOut;
                npc_placement_dialog.npc_new_fade_duration = "1.0".to_string();
                npc_placement_dialog.npc_new_fade_color = [0.0, 0.0, 0.0, 1.0];
                npc_placement_dialog.npc_new_state_key = String::new();
                npc_placement_dialog.npc_new_state_value = String::new();
                npc_placement_dialog.npc_new_appearance = PlayerAppearance::Hidden;
                npc_placement_dialog.npc_new_appearance_path = String::new();
            }
        }
    }
}

/// Egui panel for editing event triggers on a tile.
fn event_trigger_panel_ui(
    mut contexts: EguiContexts,
    mut dialog: ResMut<EventTriggerDialog>,
    mut project: ResMut<Project>,
    mut edit_events: MessageWriter<EditCommand>,
) -> Result {
    if !dialog.open {
        return Ok(());
    }

    let ctx = contexts.ctx_mut()?;

    let mut should_close = false;
    let mut should_save = false;

    egui::Window::new("Event Trigger Editor")
        .collapsible(false)
        .resizable(true)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.label(format!(
                "Tile ({}, {}) — Layer {}",
                dialog.tile_x, dialog.tile_y, dialog.layer_index
            ));
            ui.separator();

            // Display existing actions with remove/reorder/edit controls
            let mut remove_idx: Option<usize> = None;
            let mut swap: Option<(usize, usize)> = None;
            let mut edit_idx: Option<usize> = None;
            let action_count = dialog.actions.len();

            for (i, action) in dialog.actions.iter().enumerate() {
                let is_being_edited = dialog.editing_index == Some(i);
                ui.horizontal(|ui| {
                    let label = match action {
                        EventAction::JumpTo {
                            target_map_id,
                            target_x,
                            target_y,
                        } => {
                            format!(
                                "{}. JumpTo → map: {}, ({}, {})",
                                i + 1,
                                target_map_id,
                                target_x,
                                target_y
                            )
                        }
                        EventAction::ShowDialog { text, .. } => {
                            let preview = match text {
                                DialogTextData::Inline(s) => truncate_preview(s, 40),
                                DialogTextData::Id(id) => {
                                    format!("ID: {}", id)
                                }
                            };
                            format!("{}. ShowDialog — {}", i + 1, preview)
                        }
                        EventAction::ScreenShake {
                            intensity,
                            duration,
                            mode,
                        } => {
                            format!(
                                "{}. ScreenShake — intensity: {}, duration: {}, mode: {:?}",
                                i + 1,
                                intensity,
                                duration,
                                mode
                            )
                        }
                        EventAction::StopScreenShake => {
                            format!("{}. StopScreenShake", i + 1)
                        }
                        EventAction::FadeTransition {
                            fade_type,
                            duration,
                            ..
                        } => {
                            format!(
                                "{}. FadeTransition — {:?}, duration: {}",
                                i + 1,
                                fade_type,
                                duration
                            )
                        }
                        EventAction::SetState { key, value } => {
                            format!("{}. SetState — {}: {}", i + 1, key, value)
                        }
                        EventAction::SetPlayerAppearance { appearance } => {
                            format!("{}. SetPlayerAppearance — {:?}", i + 1, appearance)
                        }
                    };
                    if is_being_edited {
                        ui.label(
                            egui::RichText::new(label)
                                .strong()
                                .color(egui::Color32::from_rgb(100, 180, 255)),
                        );
                    } else {
                        ui.label(label);
                    }

                    if i > 0 && ui.small_button("▲").clicked() {
                        swap = Some((i, i - 1));
                    }
                    if i + 1 < action_count && ui.small_button("▼").clicked() {
                        swap = Some((i, i + 1));
                    }
                    if ui
                        .small_button("✏")
                        .on_hover_text("Edit this action")
                        .clicked()
                    {
                        edit_idx = Some(i);
                    }
                    if ui.small_button("✕").clicked() {
                        remove_idx = Some(i);
                    }
                });
            }

            if let Some(idx) = remove_idx {
                // If we were editing the removed action, clear editing state
                if dialog.editing_index == Some(idx) {
                    dialog.editing_index = None;
                } else if let Some(ei) = dialog.editing_index {
                    // Adjust editing index if an earlier action was removed
                    if idx < ei {
                        dialog.editing_index = Some(ei - 1);
                    }
                }
                dialog.actions.remove(idx);
            }
            if let Some((a, b)) = swap {
                dialog.actions.swap(a, b);
                // Track the editing index through swaps
                if dialog.editing_index == Some(a) {
                    dialog.editing_index = Some(b);
                } else if dialog.editing_index == Some(b) {
                    dialog.editing_index = Some(a);
                }
            }
            // Load action into form for editing
            if let Some(idx) = edit_idx
                && let Some(action) = dialog.actions.get(idx).cloned()
            {
                match action {
                    EventAction::JumpTo {
                        target_map_id,
                        target_x,
                        target_y,
                    } => {
                        dialog.new_action_type = ActionType::JumpTo;
                        dialog.new_target_map_id = target_map_id;
                        dialog.new_target_x = target_x.to_string();
                        dialog.new_target_y = target_y.to_string();
                    }
                    EventAction::ShowDialog { text, config } => {
                        dialog.new_action_type = ActionType::ShowDialog;
                        match text {
                            DialogTextData::Inline(s) => {
                                dialog.new_dialog_text_mode = DialogTextMode::Inline;
                                dialog.new_dialog_inline_text = s;
                                dialog.new_dialog_text_id.clear();
                            }
                            DialogTextData::Id(id) => {
                                dialog.new_dialog_text_mode = DialogTextMode::TextId;
                                dialog.new_dialog_text_id = id;
                                dialog.new_dialog_inline_text.clear();
                            }
                        }
                        dialog.new_dialog_text_speed = config.text_speed.to_string();
                        dialog.new_dialog_position = config.position;
                        dialog.new_dialog_movement_block = config.movement_block;
                    }
                    EventAction::ScreenShake {
                        intensity,
                        duration,
                        mode,
                    } => {
                        dialog.new_action_type = ActionType::ScreenShake;
                        dialog.new_shake_intensity = intensity.to_string();
                        dialog.new_shake_duration = duration.to_string();
                        dialog.new_shake_mode = mode;
                    }
                    EventAction::StopScreenShake => {
                        dialog.new_action_type = ActionType::StopScreenShake;
                    }
                    EventAction::FadeTransition {
                        fade_type,
                        duration,
                        color,
                    } => {
                        dialog.new_action_type = ActionType::FadeTransition;
                        dialog.new_fade_type = fade_type;
                        dialog.new_fade_duration = duration.to_string();
                        dialog.new_fade_color = color;
                    }
                    EventAction::SetState { key, value } => {
                        dialog.new_action_type = ActionType::SetState;
                        dialog.new_state_key = key;
                        dialog.new_state_value = value;
                    }
                    EventAction::SetPlayerAppearance { appearance } => {
                        dialog.new_action_type = ActionType::SetPlayerAppearance;
                        if let PlayerAppearance::Spritesheet { ref path } = appearance {
                            dialog.new_appearance_path = path.clone();
                        } else {
                            dialog.new_appearance_path = String::new();
                        }
                        dialog.new_appearance = appearance;
                    }
                }
                dialog.editing_index = Some(idx);
            }

            ui.separator();

            // Action type selector
            let is_editing_action = dialog.editing_index.is_some();
            let form_label = if is_editing_action {
                "Edit Action:"
            } else {
                "Add Action:"
            };
            ui.label(egui::RichText::new(form_label).strong());

            ui.horizontal(|ui| {
                ui.label("Action Type:");
                let action_type_text = match dialog.new_action_type {
                    ActionType::JumpTo => "JumpTo",
                    ActionType::ShowDialog => "ShowDialog",
                    ActionType::ScreenShake => "ScreenShake",
                    ActionType::StopScreenShake => "StopScreenShake",
                    ActionType::FadeTransition => "FadeTransition",
                    ActionType::SetState => "SetState",
                    ActionType::SetPlayerAppearance => "SetPlayerAppearance",
                };
                egui::ComboBox::from_id_salt("event_trigger_action_type")
                    .selected_text(action_type_text)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut dialog.new_action_type,
                            ActionType::JumpTo,
                            "JumpTo",
                        );
                        ui.selectable_value(
                            &mut dialog.new_action_type,
                            ActionType::ShowDialog,
                            "ShowDialog",
                        );
                        ui.selectable_value(
                            &mut dialog.new_action_type,
                            ActionType::ScreenShake,
                            "ScreenShake",
                        );
                        ui.selectable_value(
                            &mut dialog.new_action_type,
                            ActionType::StopScreenShake,
                            "StopScreenShake",
                        );
                        ui.selectable_value(
                            &mut dialog.new_action_type,
                            ActionType::FadeTransition,
                            "FadeTransition",
                        );
                        ui.selectable_value(
                            &mut dialog.new_action_type,
                            ActionType::SetState,
                            "SetState",
                        );
                        ui.selectable_value(
                            &mut dialog.new_action_type,
                            ActionType::SetPlayerAppearance,
                            "SetPlayerAppearance",
                        );
                    });
            });

            ui.separator();

            if dialog.new_action_type == ActionType::JumpTo {
                let jumpto_form_label = if dialog.editing_index.is_some() {
                    "Edit JumpTo Action:"
                } else {
                    "Add JumpTo Action:"
                };
                ui.label(jumpto_form_label);

                // Map selector dropdown
                let map_ids: Vec<(String, String)> = project
                    .maps
                    .iter()
                    .map(|(id, m)| (id.clone(), m.name.clone()))
                    .collect();

                ui.horizontal(|ui| {
                    ui.label("Target Map:");
                    let selected_text = if dialog.new_target_map_id.is_empty() {
                        "Select map...".to_string()
                    } else {
                        map_ids
                            .iter()
                            .find(|(id, _)| *id == dialog.new_target_map_id)
                            .map(|(_, name)| name.clone())
                            .unwrap_or_else(|| dialog.new_target_map_id.clone())
                    };
                    egui::ComboBox::from_id_salt("event_trigger_map_select")
                        .selected_text(selected_text)
                        .show_ui(ui, |ui| {
                            for (id, name) in &map_ids {
                                ui.selectable_value(
                                    &mut dialog.new_target_map_id,
                                    id.clone(),
                                    name,
                                );
                            }
                        });
                });

                ui.horizontal(|ui| {
                    ui.label("X:");
                    ui.text_edit_singleline(&mut dialog.new_target_x);
                    ui.label("Y:");
                    ui.text_edit_singleline(&mut dialog.new_target_y);
                });

                let jumpto_button_label = if dialog.editing_index.is_some() {
                    "Update JumpTo"
                } else {
                    "Add JumpTo"
                };
                if ui.button(jumpto_button_label).clicked() {
                    let has_target = !dialog.new_target_map_id.is_empty();
                    if has_target {
                        let target_map = dialog.new_target_map_id.clone();
                        let x = dialog.new_target_x.trim().parse::<u32>().unwrap_or(0);
                        let y = dialog.new_target_y.trim().parse::<u32>().unwrap_or(0);
                        let new_action = EventAction::JumpTo {
                            target_map_id: target_map,
                            target_x: x,
                            target_y: y,
                        };
                        if let Some(idx) = dialog.editing_index {
                            if idx < dialog.actions.len() {
                                dialog.actions[idx] = new_action;
                            }
                            dialog.editing_index = None;
                        } else {
                            dialog.actions.push(new_action);
                        }
                        dialog.new_target_map_id = String::new();
                        dialog.new_target_x = "0".to_string();
                        dialog.new_target_y = "0".to_string();
                    }
                }
                if dialog.editing_index.is_some() && ui.button("Cancel Edit").clicked() {
                    dialog.editing_index = None;
                    dialog.new_target_map_id = String::new();
                    dialog.new_target_x = "0".to_string();
                    dialog.new_target_y = "0".to_string();
                }
            } else if dialog.new_action_type == ActionType::ShowDialog {
                let show_dialog_form_label = if dialog.editing_index.is_some() {
                    "Edit ShowDialog Action:"
                } else {
                    "Add ShowDialog Action:"
                };
                ui.label(show_dialog_form_label);

                // Text source toggle
                ui.horizontal(|ui| {
                    ui.label("Text Source:");
                    ui.radio_value(
                        &mut dialog.new_dialog_text_mode,
                        DialogTextMode::Inline,
                        "Inline",
                    );
                    ui.radio_value(
                        &mut dialog.new_dialog_text_mode,
                        DialogTextMode::TextId,
                        "Text ID",
                    );
                });

                match dialog.new_dialog_text_mode {
                    DialogTextMode::Inline => {
                        ui.label("Dialog Text:");
                        ui.text_edit_multiline(&mut dialog.new_dialog_inline_text);
                    }
                    DialogTextMode::TextId => {
                        ui.horizontal(|ui| {
                            ui.label("Text ID:");
                            ui.text_edit_singleline(&mut dialog.new_dialog_text_id);
                        });
                    }
                }

                ui.horizontal(|ui| {
                    ui.label("Text Speed:");
                    ui.add(
                        egui::TextEdit::singleline(&mut dialog.new_dialog_text_speed)
                            .desired_width(60.0),
                    );
                });

                ui.horizontal(|ui| {
                    ui.label("Position:");
                    egui::ComboBox::from_id_salt("dialog_position_select")
                        .selected_text(match dialog.new_dialog_position {
                            DialogPositionData::Top => "Top",
                            DialogPositionData::Center => "Center",
                            DialogPositionData::Bottom => "Bottom",
                        })
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut dialog.new_dialog_position,
                                DialogPositionData::Top,
                                "Top",
                            );
                            ui.selectable_value(
                                &mut dialog.new_dialog_position,
                                DialogPositionData::Center,
                                "Center",
                            );
                            ui.selectable_value(
                                &mut dialog.new_dialog_position,
                                DialogPositionData::Bottom,
                                "Bottom",
                            );
                        });
                });

                ui.checkbox(&mut dialog.new_dialog_movement_block, "Movement Block");

                let show_dialog_button_label = if dialog.editing_index.is_some() {
                    "Update ShowDialog"
                } else {
                    "Add ShowDialog"
                };
                if ui.button(show_dialog_button_label).clicked() {
                    let text = match dialog.new_dialog_text_mode {
                        DialogTextMode::Inline => {
                            if dialog.new_dialog_inline_text.is_empty() {
                                None
                            } else {
                                Some(DialogTextData::Inline(
                                    dialog.new_dialog_inline_text.clone(),
                                ))
                            }
                        }
                        DialogTextMode::TextId => {
                            if dialog.new_dialog_text_id.is_empty() {
                                None
                            } else {
                                Some(DialogTextData::Id(dialog.new_dialog_text_id.clone()))
                            }
                        }
                    };

                    if let Some(text) = text {
                        let text_speed = dialog
                            .new_dialog_text_speed
                            .trim()
                            .parse::<f32>()
                            .unwrap_or(30.0);
                        let config = DialogConfigData {
                            text_speed,
                            position: dialog.new_dialog_position.clone(),
                            movement_block: dialog.new_dialog_movement_block,
                        };
                        let new_action = EventAction::ShowDialog { text, config };
                        if let Some(idx) = dialog.editing_index {
                            if idx < dialog.actions.len() {
                                dialog.actions[idx] = new_action;
                            }
                            dialog.editing_index = None;
                        } else {
                            dialog.actions.push(new_action);
                        }
                        // Reset ShowDialog fields
                        dialog.new_dialog_inline_text = String::new();
                        dialog.new_dialog_text_id = String::new();
                        dialog.new_dialog_text_speed = "30".to_string();
                        dialog.new_dialog_position = DialogPositionData::Bottom;
                        dialog.new_dialog_movement_block = true;
                    }
                }
                if dialog.editing_index.is_some() && ui.button("Cancel Edit").clicked() {
                    dialog.editing_index = None;
                    dialog.new_dialog_inline_text = String::new();
                    dialog.new_dialog_text_id = String::new();
                    dialog.new_dialog_text_speed = "30".to_string();
                    dialog.new_dialog_position = DialogPositionData::Bottom;
                    dialog.new_dialog_movement_block = true;
                }
            } else if dialog.new_action_type == ActionType::ScreenShake {
                // ScreenShake form
                ui.horizontal(|ui| {
                    ui.label("Mode:");
                    ui.radio_value(&mut dialog.new_shake_mode, ScreenShakeMode::Timed, "Timed");
                    ui.radio_value(
                        &mut dialog.new_shake_mode,
                        ScreenShakeMode::Continuous,
                        "Continuous",
                    );
                });

                ui.horizontal(|ui| {
                    ui.label("Intensity:");
                    ui.add(
                        egui::TextEdit::singleline(&mut dialog.new_shake_intensity)
                            .desired_width(60.0),
                    );
                    ui.label("(0.0 – 50.0)");
                });

                if dialog.new_shake_mode == ScreenShakeMode::Timed {
                    ui.horizontal(|ui| {
                        ui.label("Duration:");
                        ui.add(
                            egui::TextEdit::singleline(&mut dialog.new_shake_duration)
                                .desired_width(60.0),
                        );
                        ui.label("(0.0 – 10.0)");
                    });
                }

                let btn_label = if dialog.editing_index.is_some() {
                    "Update ScreenShake"
                } else {
                    "Add ScreenShake"
                };
                if ui.button(btn_label).clicked() {
                    let intensity = dialog
                        .new_shake_intensity
                        .trim()
                        .parse::<f32>()
                        .unwrap_or(5.0)
                        .clamp(0.0, 50.0);
                    let duration = dialog
                        .new_shake_duration
                        .trim()
                        .parse::<f32>()
                        .unwrap_or(0.5)
                        .clamp(0.0, 10.0);
                    let new_action = EventAction::ScreenShake {
                        intensity,
                        duration,
                        mode: dialog.new_shake_mode,
                    };
                    if let Some(idx) = dialog.editing_index {
                        if idx < dialog.actions.len() {
                            dialog.actions[idx] = new_action;
                        }
                        dialog.editing_index = None;
                    } else {
                        dialog.actions.push(new_action);
                    }
                    dialog.new_shake_intensity = "5.0".to_string();
                    dialog.new_shake_duration = "0.5".to_string();
                    dialog.new_shake_mode = ScreenShakeMode::Timed;
                }
                if dialog.editing_index.is_some() && ui.button("Cancel Edit").clicked() {
                    dialog.editing_index = None;
                    dialog.new_shake_intensity = "5.0".to_string();
                    dialog.new_shake_duration = "0.5".to_string();
                    dialog.new_shake_mode = ScreenShakeMode::Timed;
                }
            } else if dialog.new_action_type == ActionType::StopScreenShake {
                // StopScreenShake — no configuration fields
                ui.label("No additional configuration needed.");

                let btn_label = if dialog.editing_index.is_some() {
                    "Update StopScreenShake"
                } else {
                    "Add StopScreenShake"
                };
                if ui.button(btn_label).clicked() {
                    let new_action = EventAction::StopScreenShake;
                    if let Some(idx) = dialog.editing_index {
                        if idx < dialog.actions.len() {
                            dialog.actions[idx] = new_action;
                        }
                        dialog.editing_index = None;
                    } else {
                        dialog.actions.push(new_action);
                    }
                }
                if dialog.editing_index.is_some() && ui.button("Cancel Edit").clicked() {
                    dialog.editing_index = None;
                }
            } else if dialog.new_action_type == ActionType::FadeTransition {
                // FadeTransition form
                ui.horizontal(|ui| {
                    ui.label("Fade Type:");
                    ui.radio_value(&mut dialog.new_fade_type, FadeType::FadeIn, "FadeIn");
                    ui.radio_value(&mut dialog.new_fade_type, FadeType::FadeOut, "FadeOut");
                });

                ui.horizontal(|ui| {
                    ui.label("Duration:");
                    ui.add(
                        egui::TextEdit::singleline(&mut dialog.new_fade_duration)
                            .desired_width(60.0),
                    );
                    ui.label("(0.0 – 10.0)");
                });

                ui.horizontal(|ui| {
                    ui.label("Color (RGBA):");
                    let mut color3 = [
                        dialog.new_fade_color[0],
                        dialog.new_fade_color[1],
                        dialog.new_fade_color[2],
                    ];
                    if ui.color_edit_button_rgb(&mut color3).changed() {
                        dialog.new_fade_color[0] = color3[0];
                        dialog.new_fade_color[1] = color3[1];
                        dialog.new_fade_color[2] = color3[2];
                    }
                });

                let btn_label = if dialog.editing_index.is_some() {
                    "Update FadeTransition"
                } else {
                    "Add FadeTransition"
                };
                if ui.button(btn_label).clicked() {
                    let duration = dialog
                        .new_fade_duration
                        .trim()
                        .parse::<f32>()
                        .unwrap_or(1.0)
                        .clamp(0.0, 10.0);
                    let new_action = EventAction::FadeTransition {
                        fade_type: dialog.new_fade_type,
                        duration,
                        color: dialog.new_fade_color,
                    };
                    if let Some(idx) = dialog.editing_index {
                        if idx < dialog.actions.len() {
                            dialog.actions[idx] = new_action;
                        }
                        dialog.editing_index = None;
                    } else {
                        dialog.actions.push(new_action);
                    }
                    dialog.new_fade_type = FadeType::FadeOut;
                    dialog.new_fade_duration = "1.0".to_string();
                    dialog.new_fade_color = [0.0, 0.0, 0.0, 1.0];
                }
                if dialog.editing_index.is_some() && ui.button("Cancel Edit").clicked() {
                    dialog.editing_index = None;
                    dialog.new_fade_type = FadeType::FadeOut;
                    dialog.new_fade_duration = "1.0".to_string();
                    dialog.new_fade_color = [0.0, 0.0, 0.0, 1.0];
                }
            } else if dialog.new_action_type == ActionType::SetState {
                // SetState form
                ui.horizontal(|ui| {
                    ui.label("Key:");
                    ui.text_edit_singleline(&mut dialog.new_state_key);
                });
                ui.horizontal(|ui| {
                    ui.label("Value:");
                    ui.text_edit_singleline(&mut dialog.new_state_value);
                });

                let btn_label = if dialog.editing_index.is_some() {
                    "Update SetState"
                } else {
                    "Add SetState"
                };
                if ui.button(btn_label).clicked() && !dialog.new_state_key.is_empty() {
                    let new_action = EventAction::SetState {
                        key: dialog.new_state_key.clone(),
                        value: dialog.new_state_value.clone(),
                    };
                    if let Some(idx) = dialog.editing_index {
                        if idx < dialog.actions.len() {
                            dialog.actions[idx] = new_action;
                        }
                        dialog.editing_index = None;
                    } else {
                        dialog.actions.push(new_action);
                    }
                    dialog.new_state_key = String::new();
                    dialog.new_state_value = String::new();
                }
                if dialog.editing_index.is_some() && ui.button("Cancel Edit").clicked() {
                    dialog.editing_index = None;
                    dialog.new_state_key = String::new();
                    dialog.new_state_value = String::new();
                }
            } else if dialog.new_action_type == ActionType::SetPlayerAppearance {
                // SetPlayerAppearance form
                ui.horizontal(|ui| {
                    ui.label("Appearance:");
                    let appearance_text = match &dialog.new_appearance {
                        PlayerAppearance::Hidden => "Hidden",
                        PlayerAppearance::Spritesheet { .. } => "Spritesheet",
                        PlayerAppearance::Default => "Default",
                    };
                    egui::ComboBox::from_id_salt("event_trigger_appearance_select")
                        .selected_text(appearance_text)
                        .show_ui(ui, |ui| {
                            if ui
                                .selectable_label(
                                    matches!(dialog.new_appearance, PlayerAppearance::Hidden),
                                    "Hidden",
                                )
                                .clicked()
                            {
                                dialog.new_appearance = PlayerAppearance::Hidden;
                            }
                            if ui
                                .selectable_label(
                                    matches!(
                                        dialog.new_appearance,
                                        PlayerAppearance::Spritesheet { .. }
                                    ),
                                    "Spritesheet",
                                )
                                .clicked()
                            {
                                dialog.new_appearance = PlayerAppearance::Spritesheet {
                                    path: dialog.new_appearance_path.clone(),
                                };
                            }
                            if ui
                                .selectable_label(
                                    matches!(dialog.new_appearance, PlayerAppearance::Default),
                                    "Default",
                                )
                                .clicked()
                            {
                                dialog.new_appearance = PlayerAppearance::Default;
                            }
                        });
                });

                if matches!(dialog.new_appearance, PlayerAppearance::Spritesheet { .. }) {
                    ui.horizontal(|ui| {
                        ui.label("Path:");
                        ui.text_edit_singleline(&mut dialog.new_appearance_path);
                    });
                }

                let btn_label = if dialog.editing_index.is_some() {
                    "Update SetPlayerAppearance"
                } else {
                    "Add SetPlayerAppearance"
                };
                if ui.button(btn_label).clicked() {
                    let appearance = match &dialog.new_appearance {
                        PlayerAppearance::Spritesheet { .. } => {
                            if dialog.new_appearance_path.is_empty() {
                                None
                            } else {
                                Some(PlayerAppearance::Spritesheet {
                                    path: dialog.new_appearance_path.clone(),
                                })
                            }
                        }
                        other => Some(other.clone()),
                    };
                    if let Some(appearance) = appearance {
                        let new_action = EventAction::SetPlayerAppearance { appearance };
                        if let Some(idx) = dialog.editing_index {
                            if idx < dialog.actions.len() {
                                dialog.actions[idx] = new_action;
                            }
                            dialog.editing_index = None;
                        } else {
                            dialog.actions.push(new_action);
                        }
                        dialog.new_appearance = PlayerAppearance::Hidden;
                        dialog.new_appearance_path = String::new();
                    }
                }
                if dialog.editing_index.is_some() && ui.button("Cancel Edit").clicked() {
                    dialog.editing_index = None;
                    dialog.new_appearance = PlayerAppearance::Hidden;
                    dialog.new_appearance_path = String::new();
                }
            }

            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("Save").clicked() {
                    should_save = true;
                }
                if ui.button("Cancel").clicked() {
                    should_close = true;
                }
            });
        });

    if should_save {
        let layer_index = dialog.layer_index;
        let x = dialog.tile_x;
        let y = dialog.tile_y;
        let old_trigger = dialog.original_actions.clone();
        let new_trigger = dialog.actions.clone();

        // Apply the change to the map
        if let Some(map) = project.active_map_mut()
            && let Some(layer) = map.layers.get_mut(layer_index)
            && let Some(attr_row) = layer.attributes.cells.get_mut(y as usize)
            && let Some(cell) = attr_row.get_mut(x as usize)
        {
            cell.event_trigger = new_trigger.clone();
        }

        edit_events.write(EditCommand {
            kind: EditCommandKind::SetEventTrigger {
                layer_index,
                x,
                y,
                old_trigger,
                new_trigger,
            },
        });

        dialog.open = false;
    }

    if should_close {
        dialog.open = false;
    }

    Ok(())
}

/// Egui dialog for confirming spawn point relocation.
fn spawn_point_confirm_ui(
    mut contexts: EguiContexts,
    mut dialog: ResMut<SpawnPointConfirmDialog>,
    project: Res<Project>,
    mut edit_events: MessageWriter<EditCommand>,
) -> Result {
    if !dialog.open {
        return Ok(());
    }

    let ctx = contexts.ctx_mut()?;

    let mut should_confirm = false;
    let mut should_cancel = false;

    // Build info about the existing spawn point
    let existing_info = if let Some(ref sp) = project.spawn_point {
        let map_name = project
            .maps
            .get(&sp.map_id)
            .map(|m| m.name.as_str())
            .unwrap_or("Unknown");
        format!("Current spawn point: {} ({}, {})", map_name, sp.x, sp.y)
    } else {
        "No existing spawn point.".to_string()
    };

    let new_info = if let Some(ref new_map_id) = dialog.new_map_id {
        let map_name = project
            .maps
            .get(new_map_id)
            .map(|m| m.name.as_str())
            .unwrap_or("Unknown");
        format!(
            "New location: {} ({}, {})",
            map_name, dialog.new_x, dialog.new_y
        )
    } else {
        String::new()
    };

    egui::Window::new("Move Spawn Point?")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.label("A spawn point already exists.");
            ui.label(&existing_info);
            ui.separator();
            ui.label("Do you want to move it?");
            ui.label(&new_info);
            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("Move").clicked() {
                    should_confirm = true;
                }
                if ui.button("Cancel").clicked() {
                    should_cancel = true;
                }
            });
        });

    if should_confirm {
        if let Some(new_map_id) = dialog.new_map_id.take() {
            let old_spawn = project.spawn_point.clone();
            let new_spawn = Some(SpawnPoint {
                map_id: new_map_id,
                x: dialog.new_x,
                y: dialog.new_y,
            });

            edit_events.write(EditCommand {
                kind: EditCommandKind::SetSpawnPoint {
                    old_spawn,
                    new_spawn: new_spawn.clone(),
                },
            });
        }
        dialog.open = false;
    }

    if should_cancel {
        dialog.open = false;
        dialog.new_map_id = None;
    }

    Ok(())
}

/// Egui dialog for placing or editing an NPC on a tile.
fn npc_placement_dialog_ui(
    mut contexts: EguiContexts,
    mut dialog: ResMut<NpcPlacementDialog>,
    mut project: ResMut<Project>,
    mut edit_events: MessageWriter<EditCommand>,
) -> Result {
    if !dialog.open {
        return Ok(());
    }

    let ctx = contexts.ctx_mut()?;

    let mut should_close = false;
    let mut should_place = false;
    let mut should_remove = false;

    // Collect spritesheet data for the combo box (avoid borrow issues)
    let spritesheet_entries: Vec<(SpritesheetId, String)> = project
        .spritesheets
        .iter()
        .map(|(id, ss)| (id.clone(), ss.file_path.clone()))
        .collect();

    let map_entries: Vec<(String, String)> = project
        .maps
        .iter()
        .map(|(id, m)| (id.clone(), m.name.clone()))
        .collect();

    let is_editing = dialog.editing_index.is_some();
    let title = if is_editing { "Edit NPC" } else { "Place NPC" };

    egui::Window::new(title)
        .collapsible(false)
        .resizable(true)
        .default_pos([10.0, 100.0])
        .show(ctx, |ui| {
            ui.label(format!("Tile ({}, {})", dialog.tile_x, dialog.tile_y));
            ui.separator();

            // Spritesheet selection combo box
            ui.horizontal(|ui| {
                ui.label("Spritesheet:");
                let selected_text = match &dialog.selected_spritesheet_id {
                    Some(id) => spritesheet_entries
                        .iter()
                        .find(|(eid, _)| eid == id)
                        .map(|(_, path)| path.clone())
                        .unwrap_or_else(|| "Invalid".to_string()),
                    None => "None".to_string(),
                };

                egui::ComboBox::from_id_salt("npc_spritesheet_combo")
                    .selected_text(&selected_text)
                    .show_ui(ui, |ui| {
                        for (id, path) in &spritesheet_entries {
                            let is_selected = dialog.selected_spritesheet_id.as_ref() == Some(id);
                            if ui.selectable_label(is_selected, path).clicked() {
                                dialog.selected_spritesheet_id = Some(id.clone());
                            }
                        }
                    });
            });

            // Facing direction selection
            ui.horizontal(|ui| {
                ui.label("Facing:");
                ui.radio_value(&mut dialog.selected_facing, FacingDirection::Down, "Down");
                ui.radio_value(&mut dialog.selected_facing, FacingDirection::Left, "Left");
                ui.radio_value(&mut dialog.selected_facing, FacingDirection::Right, "Right");
                ui.radio_value(&mut dialog.selected_facing, FacingDirection::Up, "Up");
            });

            // Patrol Path configuration
            ui.separator();
            ui.label(egui::RichText::new("Patrol Path").strong());

            // Waypoint list with remove buttons
            let mut remove_wp_idx: Option<usize> = None;
            for (i, wp) in dialog.patrol_waypoints.iter().enumerate() {
                ui.horizontal(|ui| {
                    ui.label(format!("  {}. ({}, {})", i + 1, wp.0, wp.1));
                    if ui.small_button("✕").clicked() {
                        remove_wp_idx = Some(i);
                    }
                });
            }
            if let Some(idx) = remove_wp_idx {
                dialog.patrol_waypoints.remove(idx);
            }

            // PatrolMode radio buttons
            ui.horizontal(|ui| {
                ui.label("Mode:");
                ui.radio_value(&mut dialog.patrol_mode, PatrolMode::Loop, "Loop");
                ui.radio_value(&mut dialog.patrol_mode, PatrolMode::Random, "Random");
            });

            // Speed and pause text fields
            ui.horizontal(|ui| {
                ui.label("Speed (s/tile):");
                ui.add(egui::TextEdit::singleline(&mut dialog.patrol_speed).desired_width(50.0));
            });
            ui.horizontal(|ui| {
                ui.label("Pause (s):");
                ui.add(egui::TextEdit::singleline(&mut dialog.patrol_pause).desired_width(50.0));
            });

            // "Add Waypoints" toggle button
            let wp_label = if dialog.adding_waypoints {
                "Stop Adding Waypoints"
            } else {
                "Add Waypoints"
            };
            if ui.button(wp_label).clicked() {
                dialog.adding_waypoints = !dialog.adding_waypoints;
            }
            if dialog.adding_waypoints {
                ui.label("Click map tiles to add waypoints.");
            }

            // Event Trigger configuration
            ui.separator();
            ui.label(egui::RichText::new("Event Triggers").strong());

            // TriggerMode radio buttons
            ui.horizontal(|ui| {
                ui.label("Trigger Mode:");
                ui.radio_value(
                    &mut dialog.trigger_mode,
                    TriggerMode::Interaction,
                    "Interaction",
                );
                ui.radio_value(
                    &mut dialog.trigger_mode,
                    TriggerMode::Collision,
                    "Collision",
                );
            });

            // Action list with remove/reorder/edit controls
            let mut remove_action_idx: Option<usize> = None;
            let mut swap_actions: Option<(usize, usize)> = None;
            let mut edit_action_idx: Option<usize> = None;
            let action_count = dialog.event_triggers.len();

            for (i, action) in dialog.event_triggers.iter().enumerate() {
                let is_being_edited = dialog.npc_editing_action_index == Some(i);
                ui.horizontal(|ui| {
                    let label = match action {
                        EventAction::JumpTo {
                            target_map_id,
                            target_x,
                            target_y,
                        } => {
                            format!(
                                "{}. JumpTo → map: {}, ({}, {})",
                                i + 1,
                                target_map_id,
                                target_x,
                                target_y
                            )
                        }
                        EventAction::ShowDialog { text, .. } => {
                            let preview = match text {
                                DialogTextData::Inline(s) => truncate_preview(s, 30),
                                DialogTextData::Id(id) => format!("ID: {}", id),
                            };
                            format!("{}. ShowDialog — {}", i + 1, preview)
                        }
                        EventAction::ScreenShake {
                            intensity,
                            duration,
                            mode,
                        } => {
                            format!(
                                "{}. ScreenShake — intensity: {}, duration: {}, mode: {:?}",
                                i + 1,
                                intensity,
                                duration,
                                mode
                            )
                        }
                        EventAction::StopScreenShake => {
                            format!("{}. StopScreenShake", i + 1)
                        }
                        EventAction::FadeTransition {
                            fade_type,
                            duration,
                            ..
                        } => {
                            format!(
                                "{}. FadeTransition — {:?}, duration: {}",
                                i + 1,
                                fade_type,
                                duration
                            )
                        }
                        EventAction::SetState { key, value } => {
                            format!("{}. SetState — {}: {}", i + 1, key, value)
                        }
                        EventAction::SetPlayerAppearance { appearance } => {
                            format!("{}. SetPlayerAppearance — {:?}", i + 1, appearance)
                        }
                    };
                    if is_being_edited {
                        ui.label(
                            egui::RichText::new(label)
                                .strong()
                                .color(egui::Color32::from_rgb(100, 180, 255)),
                        );
                    } else {
                        ui.label(label);
                    }

                    if i > 0 && ui.small_button("▲").clicked() {
                        swap_actions = Some((i, i - 1));
                    }
                    if i + 1 < action_count && ui.small_button("▼").clicked() {
                        swap_actions = Some((i, i + 1));
                    }
                    if ui
                        .small_button("✏")
                        .on_hover_text("Edit this action")
                        .clicked()
                    {
                        edit_action_idx = Some(i);
                    }
                    if ui.small_button("✕").clicked() {
                        remove_action_idx = Some(i);
                    }
                });
            }

            if let Some(idx) = remove_action_idx {
                if dialog.npc_editing_action_index == Some(idx) {
                    dialog.npc_editing_action_index = None;
                } else if let Some(ei) = dialog.npc_editing_action_index
                    && idx < ei
                {
                    dialog.npc_editing_action_index = Some(ei - 1);
                }
                dialog.event_triggers.remove(idx);
            }
            if let Some((a, b)) = swap_actions {
                dialog.event_triggers.swap(a, b);
                if dialog.npc_editing_action_index == Some(a) {
                    dialog.npc_editing_action_index = Some(b);
                } else if dialog.npc_editing_action_index == Some(b) {
                    dialog.npc_editing_action_index = Some(a);
                }
            }
            if let Some(idx) = edit_action_idx
                && let Some(action) = dialog.event_triggers.get(idx).cloned()
            {
                match action {
                    EventAction::JumpTo {
                        target_map_id,
                        target_x,
                        target_y,
                    } => {
                        dialog.npc_new_action_type = ActionType::JumpTo;
                        dialog.npc_new_target_map_id = target_map_id;
                        dialog.npc_new_target_x = target_x.to_string();
                        dialog.npc_new_target_y = target_y.to_string();
                    }
                    EventAction::ShowDialog { text, config } => {
                        dialog.npc_new_action_type = ActionType::ShowDialog;
                        match text {
                            DialogTextData::Inline(s) => {
                                dialog.npc_new_dialog_text_mode = DialogTextMode::Inline;
                                dialog.npc_new_dialog_inline_text = s;
                                dialog.npc_new_dialog_text_id.clear();
                            }
                            DialogTextData::Id(id) => {
                                dialog.npc_new_dialog_text_mode = DialogTextMode::TextId;
                                dialog.npc_new_dialog_text_id = id;
                                dialog.npc_new_dialog_inline_text.clear();
                            }
                        }
                        dialog.npc_new_dialog_text_speed = config.text_speed.to_string();
                        dialog.npc_new_dialog_position = config.position;
                        dialog.npc_new_dialog_movement_block = config.movement_block;
                    }
                    EventAction::ScreenShake {
                        intensity,
                        duration,
                        mode,
                    } => {
                        dialog.npc_new_action_type = ActionType::ScreenShake;
                        dialog.npc_new_shake_intensity = intensity.to_string();
                        dialog.npc_new_shake_duration = duration.to_string();
                        dialog.npc_new_shake_mode = mode;
                    }
                    EventAction::StopScreenShake => {
                        dialog.npc_new_action_type = ActionType::StopScreenShake;
                    }
                    EventAction::FadeTransition {
                        fade_type,
                        duration,
                        color,
                    } => {
                        dialog.npc_new_action_type = ActionType::FadeTransition;
                        dialog.npc_new_fade_type = fade_type;
                        dialog.npc_new_fade_duration = duration.to_string();
                        dialog.npc_new_fade_color = color;
                    }
                    EventAction::SetState { key, value } => {
                        dialog.npc_new_action_type = ActionType::SetState;
                        dialog.npc_new_state_key = key;
                        dialog.npc_new_state_value = value;
                    }
                    EventAction::SetPlayerAppearance { appearance } => {
                        dialog.npc_new_action_type = ActionType::SetPlayerAppearance;
                        if let PlayerAppearance::Spritesheet { ref path } = appearance {
                            dialog.npc_new_appearance_path = path.clone();
                        } else {
                            dialog.npc_new_appearance_path = String::new();
                        }
                        dialog.npc_new_appearance = appearance;
                    }
                }
                dialog.npc_editing_action_index = Some(idx);
            }

            // Add/Edit action form
            let is_editing_action = dialog.npc_editing_action_index.is_some();
            let form_label = if is_editing_action {
                "Edit Action:"
            } else {
                "Add Action:"
            };
            ui.label(egui::RichText::new(form_label).strong());

            ui.horizontal(|ui| {
                ui.label("Type:");
                let npc_action_type_text = match dialog.npc_new_action_type {
                    ActionType::JumpTo => "JumpTo",
                    ActionType::ShowDialog => "ShowDialog",
                    ActionType::ScreenShake => "ScreenShake",
                    ActionType::StopScreenShake => "StopScreenShake",
                    ActionType::FadeTransition => "FadeTransition",
                    ActionType::SetState => "SetState",
                    ActionType::SetPlayerAppearance => "SetPlayerAppearance",
                };
                egui::ComboBox::from_id_salt("npc_event_trigger_action_type")
                    .selected_text(npc_action_type_text)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut dialog.npc_new_action_type,
                            ActionType::JumpTo,
                            "JumpTo",
                        );
                        ui.selectable_value(
                            &mut dialog.npc_new_action_type,
                            ActionType::ShowDialog,
                            "ShowDialog",
                        );
                        ui.selectable_value(
                            &mut dialog.npc_new_action_type,
                            ActionType::ScreenShake,
                            "ScreenShake",
                        );
                        ui.selectable_value(
                            &mut dialog.npc_new_action_type,
                            ActionType::StopScreenShake,
                            "StopScreenShake",
                        );
                        ui.selectable_value(
                            &mut dialog.npc_new_action_type,
                            ActionType::FadeTransition,
                            "FadeTransition",
                        );
                        ui.selectable_value(
                            &mut dialog.npc_new_action_type,
                            ActionType::SetState,
                            "SetState",
                        );
                        ui.selectable_value(
                            &mut dialog.npc_new_action_type,
                            ActionType::SetPlayerAppearance,
                            "SetPlayerAppearance",
                        );
                    });
            });

            if dialog.npc_new_action_type == ActionType::JumpTo {
                ui.horizontal(|ui| {
                    ui.label("Target Map:");
                    let selected_text = if dialog.npc_new_target_map_id.is_empty() {
                        "Select map...".to_string()
                    } else {
                        map_entries
                            .iter()
                            .find(|(id, _)| *id == dialog.npc_new_target_map_id)
                            .map(|(_, name)| name.clone())
                            .unwrap_or_else(|| dialog.npc_new_target_map_id.clone())
                    };
                    egui::ComboBox::from_id_salt("npc_event_trigger_map_select")
                        .selected_text(selected_text)
                        .show_ui(ui, |ui| {
                            for (id, name) in &map_entries {
                                ui.selectable_value(
                                    &mut dialog.npc_new_target_map_id,
                                    id.clone(),
                                    name,
                                );
                            }
                        });
                });

                ui.horizontal(|ui| {
                    ui.label("X:");
                    ui.add(
                        egui::TextEdit::singleline(&mut dialog.npc_new_target_x)
                            .desired_width(40.0),
                    );
                    ui.label("Y:");
                    ui.add(
                        egui::TextEdit::singleline(&mut dialog.npc_new_target_y)
                            .desired_width(40.0),
                    );
                });

                let jumpto_btn_label = if dialog.npc_editing_action_index.is_some() {
                    "Update JumpTo"
                } else {
                    "Add JumpTo"
                };
                if ui.button(jumpto_btn_label).clicked() {
                    let has_target = !dialog.npc_new_target_map_id.is_empty();
                    if has_target {
                        let target_map = dialog.npc_new_target_map_id.clone();
                        let x = dialog.npc_new_target_x.trim().parse::<u32>().unwrap_or(0);
                        let y = dialog.npc_new_target_y.trim().parse::<u32>().unwrap_or(0);
                        let new_action = EventAction::JumpTo {
                            target_map_id: target_map,
                            target_x: x,
                            target_y: y,
                        };
                        if let Some(idx) = dialog.npc_editing_action_index {
                            if idx < dialog.event_triggers.len() {
                                dialog.event_triggers[idx] = new_action;
                            }
                            dialog.npc_editing_action_index = None;
                        } else {
                            dialog.event_triggers.push(new_action);
                        }
                        dialog.npc_new_target_map_id = String::new();
                        dialog.npc_new_target_x = "0".to_string();
                        dialog.npc_new_target_y = "0".to_string();
                    }
                }
                if dialog.npc_editing_action_index.is_some() && ui.button("Cancel Edit").clicked() {
                    dialog.npc_editing_action_index = None;
                    dialog.npc_new_target_map_id = String::new();
                    dialog.npc_new_target_x = "0".to_string();
                    dialog.npc_new_target_y = "0".to_string();
                }
            } else if dialog.npc_new_action_type == ActionType::ShowDialog {
                // ShowDialog form
                ui.horizontal(|ui| {
                    ui.label("Text Source:");
                    ui.radio_value(
                        &mut dialog.npc_new_dialog_text_mode,
                        DialogTextMode::Inline,
                        "Inline",
                    );
                    ui.radio_value(
                        &mut dialog.npc_new_dialog_text_mode,
                        DialogTextMode::TextId,
                        "Text ID",
                    );
                });

                match dialog.npc_new_dialog_text_mode {
                    DialogTextMode::Inline => {
                        ui.label("Dialog Text:");
                        ui.text_edit_multiline(&mut dialog.npc_new_dialog_inline_text);
                    }
                    DialogTextMode::TextId => {
                        ui.horizontal(|ui| {
                            ui.label("Text ID:");
                            ui.text_edit_singleline(&mut dialog.npc_new_dialog_text_id);
                        });
                    }
                }

                ui.horizontal(|ui| {
                    ui.label("Text Speed:");
                    ui.add(
                        egui::TextEdit::singleline(&mut dialog.npc_new_dialog_text_speed)
                            .desired_width(60.0),
                    );
                });

                ui.horizontal(|ui| {
                    ui.label("Position:");
                    egui::ComboBox::from_id_salt("npc_dialog_position_select")
                        .selected_text(match dialog.npc_new_dialog_position {
                            DialogPositionData::Top => "Top",
                            DialogPositionData::Center => "Center",
                            DialogPositionData::Bottom => "Bottom",
                        })
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut dialog.npc_new_dialog_position,
                                DialogPositionData::Top,
                                "Top",
                            );
                            ui.selectable_value(
                                &mut dialog.npc_new_dialog_position,
                                DialogPositionData::Center,
                                "Center",
                            );
                            ui.selectable_value(
                                &mut dialog.npc_new_dialog_position,
                                DialogPositionData::Bottom,
                                "Bottom",
                            );
                        });
                });

                ui.checkbox(&mut dialog.npc_new_dialog_movement_block, "Movement Block");

                let show_dialog_btn_label = if dialog.npc_editing_action_index.is_some() {
                    "Update ShowDialog"
                } else {
                    "Add ShowDialog"
                };
                if ui.button(show_dialog_btn_label).clicked() {
                    let text = match dialog.npc_new_dialog_text_mode {
                        DialogTextMode::Inline => {
                            if dialog.npc_new_dialog_inline_text.is_empty() {
                                None
                            } else {
                                Some(DialogTextData::Inline(
                                    dialog.npc_new_dialog_inline_text.clone(),
                                ))
                            }
                        }
                        DialogTextMode::TextId => {
                            if dialog.npc_new_dialog_text_id.is_empty() {
                                None
                            } else {
                                Some(DialogTextData::Id(dialog.npc_new_dialog_text_id.clone()))
                            }
                        }
                    };

                    if let Some(text) = text {
                        let text_speed = dialog
                            .npc_new_dialog_text_speed
                            .trim()
                            .parse::<f32>()
                            .unwrap_or(30.0);
                        let config = DialogConfigData {
                            text_speed,
                            position: dialog.npc_new_dialog_position.clone(),
                            movement_block: dialog.npc_new_dialog_movement_block,
                        };
                        let new_action = EventAction::ShowDialog { text, config };
                        if let Some(idx) = dialog.npc_editing_action_index {
                            if idx < dialog.event_triggers.len() {
                                dialog.event_triggers[idx] = new_action;
                            }
                            dialog.npc_editing_action_index = None;
                        } else {
                            dialog.event_triggers.push(new_action);
                        }
                        dialog.npc_new_dialog_inline_text = String::new();
                        dialog.npc_new_dialog_text_id = String::new();
                        dialog.npc_new_dialog_text_speed = "30".to_string();
                        dialog.npc_new_dialog_position = DialogPositionData::Bottom;
                        dialog.npc_new_dialog_movement_block = true;
                    }
                }
                if dialog.npc_editing_action_index.is_some() && ui.button("Cancel Edit").clicked() {
                    dialog.npc_editing_action_index = None;
                    dialog.npc_new_dialog_inline_text = String::new();
                    dialog.npc_new_dialog_text_id = String::new();
                    dialog.npc_new_dialog_text_speed = "30".to_string();
                    dialog.npc_new_dialog_position = DialogPositionData::Bottom;
                    dialog.npc_new_dialog_movement_block = true;
                }
            } else if dialog.npc_new_action_type == ActionType::ScreenShake {
                // ScreenShake form
                ui.horizontal(|ui| {
                    ui.label("Mode:");
                    ui.radio_value(
                        &mut dialog.npc_new_shake_mode,
                        ScreenShakeMode::Timed,
                        "Timed",
                    );
                    ui.radio_value(
                        &mut dialog.npc_new_shake_mode,
                        ScreenShakeMode::Continuous,
                        "Continuous",
                    );
                });

                ui.horizontal(|ui| {
                    ui.label("Intensity:");
                    ui.add(
                        egui::TextEdit::singleline(&mut dialog.npc_new_shake_intensity)
                            .desired_width(60.0),
                    );
                    ui.label("(0.0 – 50.0)");
                });

                if dialog.npc_new_shake_mode == ScreenShakeMode::Timed {
                    ui.horizontal(|ui| {
                        ui.label("Duration:");
                        ui.add(
                            egui::TextEdit::singleline(&mut dialog.npc_new_shake_duration)
                                .desired_width(60.0),
                        );
                        ui.label("(0.0 – 10.0)");
                    });
                }

                let btn_label = if dialog.npc_editing_action_index.is_some() {
                    "Update ScreenShake"
                } else {
                    "Add ScreenShake"
                };
                if ui.button(btn_label).clicked() {
                    let intensity = dialog
                        .npc_new_shake_intensity
                        .trim()
                        .parse::<f32>()
                        .unwrap_or(5.0)
                        .clamp(0.0, 50.0);
                    let duration = dialog
                        .npc_new_shake_duration
                        .trim()
                        .parse::<f32>()
                        .unwrap_or(0.5)
                        .clamp(0.0, 10.0);
                    let new_action = EventAction::ScreenShake {
                        intensity,
                        duration,
                        mode: dialog.npc_new_shake_mode,
                    };
                    if let Some(idx) = dialog.npc_editing_action_index {
                        if idx < dialog.event_triggers.len() {
                            dialog.event_triggers[idx] = new_action;
                        }
                        dialog.npc_editing_action_index = None;
                    } else {
                        dialog.event_triggers.push(new_action);
                    }
                    dialog.npc_new_shake_intensity = "5.0".to_string();
                    dialog.npc_new_shake_duration = "0.5".to_string();
                    dialog.npc_new_shake_mode = ScreenShakeMode::Timed;
                }
                if dialog.npc_editing_action_index.is_some() && ui.button("Cancel Edit").clicked() {
                    dialog.npc_editing_action_index = None;
                    dialog.npc_new_shake_intensity = "5.0".to_string();
                    dialog.npc_new_shake_duration = "0.5".to_string();
                    dialog.npc_new_shake_mode = ScreenShakeMode::Timed;
                }
            } else if dialog.npc_new_action_type == ActionType::StopScreenShake {
                // StopScreenShake — no configuration fields
                ui.label("No additional configuration needed.");

                let btn_label = if dialog.npc_editing_action_index.is_some() {
                    "Update StopScreenShake"
                } else {
                    "Add StopScreenShake"
                };
                if ui.button(btn_label).clicked() {
                    let new_action = EventAction::StopScreenShake;
                    if let Some(idx) = dialog.npc_editing_action_index {
                        if idx < dialog.event_triggers.len() {
                            dialog.event_triggers[idx] = new_action;
                        }
                        dialog.npc_editing_action_index = None;
                    } else {
                        dialog.event_triggers.push(new_action);
                    }
                }
                if dialog.npc_editing_action_index.is_some() && ui.button("Cancel Edit").clicked() {
                    dialog.npc_editing_action_index = None;
                }
            } else if dialog.npc_new_action_type == ActionType::FadeTransition {
                // FadeTransition form
                ui.horizontal(|ui| {
                    ui.label("Fade Type:");
                    ui.radio_value(&mut dialog.npc_new_fade_type, FadeType::FadeIn, "FadeIn");
                    ui.radio_value(&mut dialog.npc_new_fade_type, FadeType::FadeOut, "FadeOut");
                });

                ui.horizontal(|ui| {
                    ui.label("Duration:");
                    ui.add(
                        egui::TextEdit::singleline(&mut dialog.npc_new_fade_duration)
                            .desired_width(60.0),
                    );
                    ui.label("(0.0 – 10.0)");
                });

                ui.horizontal(|ui| {
                    ui.label("Color (RGBA):");
                    let mut color3 = [
                        dialog.npc_new_fade_color[0],
                        dialog.npc_new_fade_color[1],
                        dialog.npc_new_fade_color[2],
                    ];
                    if ui.color_edit_button_rgb(&mut color3).changed() {
                        dialog.npc_new_fade_color[0] = color3[0];
                        dialog.npc_new_fade_color[1] = color3[1];
                        dialog.npc_new_fade_color[2] = color3[2];
                    }
                });

                let btn_label = if dialog.npc_editing_action_index.is_some() {
                    "Update FadeTransition"
                } else {
                    "Add FadeTransition"
                };
                if ui.button(btn_label).clicked() {
                    let duration = dialog
                        .npc_new_fade_duration
                        .trim()
                        .parse::<f32>()
                        .unwrap_or(1.0)
                        .clamp(0.0, 10.0);
                    let new_action = EventAction::FadeTransition {
                        fade_type: dialog.npc_new_fade_type,
                        duration,
                        color: dialog.npc_new_fade_color,
                    };
                    if let Some(idx) = dialog.npc_editing_action_index {
                        if idx < dialog.event_triggers.len() {
                            dialog.event_triggers[idx] = new_action;
                        }
                        dialog.npc_editing_action_index = None;
                    } else {
                        dialog.event_triggers.push(new_action);
                    }
                    dialog.npc_new_fade_type = FadeType::FadeOut;
                    dialog.npc_new_fade_duration = "1.0".to_string();
                    dialog.npc_new_fade_color = [0.0, 0.0, 0.0, 1.0];
                }
                if dialog.npc_editing_action_index.is_some() && ui.button("Cancel Edit").clicked() {
                    dialog.npc_editing_action_index = None;
                    dialog.npc_new_fade_type = FadeType::FadeOut;
                    dialog.npc_new_fade_duration = "1.0".to_string();
                    dialog.npc_new_fade_color = [0.0, 0.0, 0.0, 1.0];
                }
            } else if dialog.npc_new_action_type == ActionType::SetState {
                // SetState form
                ui.horizontal(|ui| {
                    ui.label("Key:");
                    ui.text_edit_singleline(&mut dialog.npc_new_state_key);
                });
                ui.horizontal(|ui| {
                    ui.label("Value:");
                    ui.text_edit_singleline(&mut dialog.npc_new_state_value);
                });

                let btn_label = if dialog.npc_editing_action_index.is_some() {
                    "Update SetState"
                } else {
                    "Add SetState"
                };
                if ui.button(btn_label).clicked() && !dialog.npc_new_state_key.is_empty() {
                    let new_action = EventAction::SetState {
                        key: dialog.npc_new_state_key.clone(),
                        value: dialog.npc_new_state_value.clone(),
                    };
                    if let Some(idx) = dialog.npc_editing_action_index {
                        if idx < dialog.event_triggers.len() {
                            dialog.event_triggers[idx] = new_action;
                        }
                        dialog.npc_editing_action_index = None;
                    } else {
                        dialog.event_triggers.push(new_action);
                    }
                    dialog.npc_new_state_key = String::new();
                    dialog.npc_new_state_value = String::new();
                }
                if dialog.npc_editing_action_index.is_some() && ui.button("Cancel Edit").clicked() {
                    dialog.npc_editing_action_index = None;
                    dialog.npc_new_state_key = String::new();
                    dialog.npc_new_state_value = String::new();
                }
            } else if dialog.npc_new_action_type == ActionType::SetPlayerAppearance {
                // SetPlayerAppearance form
                ui.horizontal(|ui| {
                    ui.label("Appearance:");
                    let appearance_text = match &dialog.npc_new_appearance {
                        PlayerAppearance::Hidden => "Hidden",
                        PlayerAppearance::Spritesheet { .. } => "Spritesheet",
                        PlayerAppearance::Default => "Default",
                    };
                    egui::ComboBox::from_id_salt("npc_event_trigger_appearance_select")
                        .selected_text(appearance_text)
                        .show_ui(ui, |ui| {
                            if ui
                                .selectable_label(
                                    matches!(dialog.npc_new_appearance, PlayerAppearance::Hidden),
                                    "Hidden",
                                )
                                .clicked()
                            {
                                dialog.npc_new_appearance = PlayerAppearance::Hidden;
                            }
                            if ui
                                .selectable_label(
                                    matches!(
                                        dialog.npc_new_appearance,
                                        PlayerAppearance::Spritesheet { .. }
                                    ),
                                    "Spritesheet",
                                )
                                .clicked()
                            {
                                dialog.npc_new_appearance = PlayerAppearance::Spritesheet {
                                    path: dialog.npc_new_appearance_path.clone(),
                                };
                            }
                            if ui
                                .selectable_label(
                                    matches!(dialog.npc_new_appearance, PlayerAppearance::Default),
                                    "Default",
                                )
                                .clicked()
                            {
                                dialog.npc_new_appearance = PlayerAppearance::Default;
                            }
                        });
                });

                if matches!(
                    dialog.npc_new_appearance,
                    PlayerAppearance::Spritesheet { .. }
                ) {
                    ui.horizontal(|ui| {
                        ui.label("Path:");
                        ui.text_edit_singleline(&mut dialog.npc_new_appearance_path);
                    });
                }

                let btn_label = if dialog.npc_editing_action_index.is_some() {
                    "Update SetPlayerAppearance"
                } else {
                    "Add SetPlayerAppearance"
                };
                if ui.button(btn_label).clicked() {
                    let appearance = match &dialog.npc_new_appearance {
                        PlayerAppearance::Spritesheet { .. } => {
                            if dialog.npc_new_appearance_path.is_empty() {
                                None
                            } else {
                                Some(PlayerAppearance::Spritesheet {
                                    path: dialog.npc_new_appearance_path.clone(),
                                })
                            }
                        }
                        other => Some(other.clone()),
                    };
                    if let Some(appearance) = appearance {
                        let new_action = EventAction::SetPlayerAppearance { appearance };
                        if let Some(idx) = dialog.npc_editing_action_index {
                            if idx < dialog.event_triggers.len() {
                                dialog.event_triggers[idx] = new_action;
                            }
                            dialog.npc_editing_action_index = None;
                        } else {
                            dialog.event_triggers.push(new_action);
                        }
                        dialog.npc_new_appearance = PlayerAppearance::Hidden;
                        dialog.npc_new_appearance_path = String::new();
                    }
                }
                if dialog.npc_editing_action_index.is_some() && ui.button("Cancel Edit").clicked() {
                    dialog.npc_editing_action_index = None;
                    dialog.npc_new_appearance = PlayerAppearance::Hidden;
                    dialog.npc_new_appearance_path = String::new();
                }
            }

            ui.separator();
            ui.horizontal(|ui| {
                let place_label = if is_editing { "Save" } else { "Place" };
                let can_place = dialog.selected_spritesheet_id.is_some();
                if ui
                    .add_enabled(can_place, egui::Button::new(place_label))
                    .clicked()
                {
                    should_place = true;
                }

                if is_editing && ui.button("Remove").clicked() {
                    should_remove = true;
                }

                if ui.button("Cancel").clicked() {
                    should_close = true;
                }
            });
        });

    if should_place {
        if let Some(ref spritesheet_id) = dialog.selected_spritesheet_id {
            let patrol_config = if dialog.patrol_mode == PatrolMode::Random
                || !dialog.patrol_waypoints.is_empty()
            {
                Some(PatrolConfig {
                    waypoints: dialog.patrol_waypoints.clone(),
                    mode: dialog.patrol_mode,
                    speed: dialog.patrol_speed.trim().parse::<f32>().unwrap_or(0.3),
                    pause: dialog.patrol_pause.trim().parse::<f32>().unwrap_or(0.5),
                })
            } else {
                None
            };

            let npc = NpcInstance {
                spritesheet_id: spritesheet_id.clone(),
                x: dialog.tile_x,
                y: dialog.tile_y,
                facing: dialog.selected_facing,
                event_triggers: dialog.event_triggers.clone(),
                patrol_config,
                trigger_mode: dialog.trigger_mode,
            };

            if let Some(map) = project.active_map_mut() {
                if let Some(idx) = dialog.editing_index {
                    // Replace existing NPC
                    if idx < map.npcs.len() {
                        map.npcs[idx] = npc.clone();
                    }
                } else {
                    // Add new NPC
                    map.npcs.push(npc.clone());
                }
            }

            edit_events.write(EditCommand {
                kind: EditCommandKind::PlaceNpc {
                    npc_index: dialog.editing_index,
                    old_npc: dialog.original_npc.clone(),
                    new_npc: npc,
                },
            });
        }
        dialog.open = false;
    }

    if should_remove {
        if let Some(idx) = dialog.editing_index {
            if let Some(map) = project.active_map_mut()
                && idx < map.npcs.len()
            {
                map.npcs.remove(idx);
            }

            if let Some(original) = dialog.original_npc.clone() {
                edit_events.write(EditCommand {
                    kind: EditCommandKind::RemoveNpc {
                        npc_index: idx,
                        removed_npc: original,
                    },
                });
            }
        }
        dialog.open = false;
    }

    if should_close {
        dialog.open = false;
    }

    Ok(())
}
