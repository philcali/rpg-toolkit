use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

use crate::data::editor_state::{EditCommand, EditCommandKind};
use crate::data::map::{EventAction, MapId, SpawnPoint};
use crate::data::{AnyDialogOpen, AttributeTool, EditorMode, EditorState, Project};
use crate::systems::input::CursorWorldState;
use rpg_toolkit_common::{
    DialogConfigData, DialogPositionData, DialogTextData, FacingDirection, NpcInstance,
    SpritesheetId,
};

/// The type of action being added in the Event Trigger Editor.
#[derive(Default, PartialEq)]
pub enum ActionType {
    #[default]
    JumpTo,
    ShowDialog,
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
    // Block all attribute clicks when any modal dialog is open
    if any_dialog_open.0 {
        return;
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
                    match action {
                        EventAction::JumpTo {
                            target_map_id,
                            target_x,
                            target_y,
                        } => {
                            let label = format!(
                                "{}. JumpTo → map: {}, ({}, {})",
                                i + 1,
                                target_map_id,
                                target_x,
                                target_y
                            );
                            if is_being_edited {
                                ui.label(
                                    egui::RichText::new(label)
                                        .strong()
                                        .color(egui::Color32::from_rgb(100, 180, 255)),
                                );
                            } else {
                                ui.label(label);
                            }
                        }
                        EventAction::ShowDialog { text, .. } => {
                            let preview = match text {
                                DialogTextData::Inline(s) => truncate_preview(s, 40),
                                DialogTextData::Id(id) => {
                                    format!("ID: {}", id)
                                }
                            };
                            let label = format!("{}. ShowDialog — {}", i + 1, preview);
                            if is_being_edited {
                                ui.label(
                                    egui::RichText::new(label)
                                        .strong()
                                        .color(egui::Color32::from_rgb(100, 180, 255)),
                                );
                            } else {
                                ui.label(label);
                            }
                        }
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
                ui.radio_value(&mut dialog.new_action_type, ActionType::JumpTo, "JumpTo");
                ui.radio_value(
                    &mut dialog.new_action_type,
                    ActionType::ShowDialog,
                    "ShowDialog",
                );
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
                            // Replace existing action
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
            } else {
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
                            // Replace existing action
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

    let is_editing = dialog.editing_index.is_some();
    let title = if is_editing { "Edit NPC" } else { "Place NPC" };

    egui::Window::new(title)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
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
            let npc = NpcInstance {
                spritesheet_id: spritesheet_id.clone(),
                x: dialog.tile_x,
                y: dialog.tile_y,
                facing: dialog.selected_facing,
                event_triggers: Vec::new(),
                patrol_path: Vec::new(),
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
