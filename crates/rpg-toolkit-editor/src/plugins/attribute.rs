use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

use crate::data::editor_state::{EditCommand, EditCommandKind};
use crate::data::map::{EventAction, MapId, SpawnPoint};
use crate::data::{AttributeTool, EditorMode, EditorState, Project};
use crate::systems::input::CursorWorldState;

/// Resource for the spawn point confirmation dialog.
#[derive(Resource, Default)]
pub struct SpawnPointConfirmDialog {
    pub open: bool,
    pub new_map_id: Option<MapId>,
    pub new_x: u32,
    pub new_y: u32,
}

/// Resource for the event trigger editing dialog.
#[derive(Resource, Default)]
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
}

pub struct AttributePlugin;

impl Plugin for AttributePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SpawnPointConfirmDialog>()
            .init_resource::<EventTriggerDialog>()
            .add_systems(
                EguiPrimaryContextPass,
                (event_trigger_panel_ui, spawn_point_confirm_ui),
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
    mut contexts: EguiContexts,
) {
    // Don't process clicks when an egui dialog is open and consuming pointer input
    if (event_trigger_dialog.open || spawn_confirm_dialog.open)
        && contexts
            .ctx_mut()
            .is_ok_and(|ctx| ctx.is_pointer_over_area())
    {
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

            // Display existing actions with remove/reorder controls
            let mut remove_idx: Option<usize> = None;
            let mut swap: Option<(usize, usize)> = None;
            let action_count = dialog.actions.len();

            for (i, action) in dialog.actions.iter().enumerate() {
                ui.horizontal(|ui| {
                    match action {
                        EventAction::JumpTo {
                            target_map_id,
                            target_x,
                            target_y,
                        } => {
                            ui.label(format!(
                                "{}. JumpTo → map: {}, ({}, {})",
                                i + 1,
                                target_map_id,
                                target_x,
                                target_y
                            ));
                        }
                    }

                    if i > 0 && ui.small_button("▲").clicked() {
                        swap = Some((i, i - 1));
                    }
                    if i + 1 < action_count && ui.small_button("▼").clicked() {
                        swap = Some((i, i + 1));
                    }
                    if ui.small_button("✕").clicked() {
                        remove_idx = Some(i);
                    }
                });
            }

            if let Some(idx) = remove_idx {
                dialog.actions.remove(idx);
            }
            if let Some((a, b)) = swap {
                dialog.actions.swap(a, b);
            }

            ui.separator();
            ui.label("Add JumpTo Action:");

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
                            ui.selectable_value(&mut dialog.new_target_map_id, id.clone(), name);
                        }
                    });
            });

            ui.horizontal(|ui| {
                ui.label("X:");
                ui.text_edit_singleline(&mut dialog.new_target_x);
                ui.label("Y:");
                ui.text_edit_singleline(&mut dialog.new_target_y);
            });

            if ui.button("Add JumpTo").clicked() {
                let has_target = !dialog.new_target_map_id.is_empty();
                if has_target {
                    let target_map = dialog.new_target_map_id.clone();
                    let x = dialog.new_target_x.trim().parse::<u32>().unwrap_or(0);
                    let y = dialog.new_target_y.trim().parse::<u32>().unwrap_or(0);
                    dialog.actions.push(EventAction::JumpTo {
                        target_map_id: target_map,
                        target_x: x,
                        target_y: y,
                    });
                    dialog.new_target_map_id = String::new();
                    dialog.new_target_x = "0".to_string();
                    dialog.new_target_y = "0".to_string();
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
