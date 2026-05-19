//! Click handling for attribute mode — dispatches to opacity toggle,
//! event trigger dialog, spawn point placement, or NPC placement
//! based on the active AttributeTool.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::event_trigger_dialog::EventTriggerDialog;
use super::npc_dialog::NpcPlacementDialog;
use super::spawn_point_dialog::SpawnPointConfirmDialog;
use crate::data::commands::{EditCommand, EditCommandKind};
use crate::data::map::SpawnPoint;
use crate::data::{AnyDialogOpen, AttributeTool, EditorMode, EditorState, Project};
use crate::systems::input::CursorWorldState;
use rpg_toolkit_common::validate_waypoint_bounds;

/// Bundled parameters for the attribute click system.
#[derive(SystemParam)]
pub struct AttributeClickParams<'w> {
    mouse: Res<'w, ButtonInput<MouseButton>>,
    editor_state: Res<'w, EditorState>,
    cursor_state: Res<'w, CursorWorldState>,
    project: ResMut<'w, Project>,
    edit_events: MessageWriter<'w, EditCommand>,
    event_trigger_dialog: ResMut<'w, EventTriggerDialog>,
    spawn_confirm_dialog: ResMut<'w, SpawnPointConfirmDialog>,
    npc_placement_dialog: ResMut<'w, NpcPlacementDialog>,
    any_dialog_open: Res<'w, AnyDialogOpen>,
}

/// Handles left-click in attribute mode for opacity toggle, event trigger selection,
/// and spawn point placement.
pub fn attribute_click_system(mut params: AttributeClickParams) {
    // Block all attribute clicks when any modal dialog is open,
    // EXCEPT when the NPC dialog is open and we're adding waypoints
    if params.any_dialog_open.0 {
        // Allow waypoint clicks through when adding waypoints
        if !(params.npc_placement_dialog.open
            && params.npc_placement_dialog.adding_waypoints
            && params.editor_state.attribute_tool == AttributeTool::NpcPlacement)
        {
            return;
        }
    }

    if params.editor_state.editor_mode != EditorMode::Attribute {
        return;
    }

    if !params.mouse.just_pressed(MouseButton::Left) {
        return;
    }

    let Some((col, row)) = params.cursor_state.tile_pos else {
        return;
    };

    match params.editor_state.attribute_tool {
        AttributeTool::Opacity => {
            let Some(map) = params.project.active_map_mut() else {
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

            params.edit_events.write(EditCommand {
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
            let Some(map) = params.project.active_map() else {
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
            let dialog = &mut *params.event_trigger_dialog;
            dialog.open = true;
            dialog.layer_index = layer_index;
            dialog.tile_x = col;
            dialog.tile_y = row;
            dialog.actions = existing.clone();
            dialog.original_actions = existing;
            dialog.action_editor.reset();
        }

        AttributeTool::SpawnPoint => {
            let Some(active_map_id) = params.project.active_map_id().cloned() else {
                return;
            };

            // Check map bounds
            let Some(map) = params.project.active_map() else {
                return;
            };
            if col >= map.width || row >= map.height {
                return;
            }

            if params.project.spawn_point.is_some() {
                // Open confirmation dialog
                params.spawn_confirm_dialog.open = true;
                params.spawn_confirm_dialog.new_map_id = Some(active_map_id);
                params.spawn_confirm_dialog.new_x = col;
                params.spawn_confirm_dialog.new_y = row;
            } else {
                // No existing spawn point — set directly
                let new_spawn = Some(SpawnPoint {
                    map_id: active_map_id,
                    x: col,
                    y: row,
                });

                params.edit_events.write(EditCommand {
                    kind: EditCommandKind::SetSpawnPoint {
                        old_spawn: None,
                        new_spawn: new_spawn.clone(),
                    },
                });
            }
        }

        AttributeTool::NpcPlacement => {
            // If dialog is open and adding waypoints, append clicked tile
            if params.npc_placement_dialog.open && params.npc_placement_dialog.adding_waypoints {
                let Some(map) = params.project.active_map() else {
                    return;
                };
                if validate_waypoint_bounds((col, row), map.width, map.height) {
                    params
                        .npc_placement_dialog
                        .patrol_waypoints
                        .push((col, row));
                }
                return;
            }

            let Some(map) = params.project.active_map() else {
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
                params.npc_placement_dialog.open_edit(idx, npc);
            } else {
                // Open empty dialog for new placement
                let first_spritesheet = params.project.spritesheets.keys().next().cloned();
                params
                    .npc_placement_dialog
                    .open_new(col, row, first_spritesheet);
            }
        }
    }
}
