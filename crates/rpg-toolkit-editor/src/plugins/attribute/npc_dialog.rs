//! Modal dialog for placing and editing NPCs on the map.

use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

use super::action_editor::ActionEditorState;
use super::action_editor_ui::render_action_editor;
use crate::data::Project;
use crate::data::commands::{EditCommand, EditCommandKind};
use crate::data::map::EventAction;
use rpg_toolkit_common::{
    FacingDirection, NpcInstance, PatrolConfig, PatrolMode, SpritesheetId, TriggerMode,
};

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
    // Shared action editor state
    pub action_editor: ActionEditorState,
    // Conditional visibility
    pub required_state_key: String,
    pub required_state_value: String,
    pub has_required_state: bool,
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
            action_editor: ActionEditorState::default(),
            required_state_key: String::new(),
            required_state_value: String::new(),
            has_required_state: false,
        }
    }
}

impl NpcPlacementDialog {
    /// Resets all fields to defaults for a fresh dialog.
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Opens the dialog for new NPC placement at the given tile.
    pub fn open_new(
        &mut self,
        tile_x: u32,
        tile_y: u32,
        default_spritesheet: Option<SpritesheetId>,
    ) {
        self.reset();
        self.open = true;
        self.tile_x = tile_x;
        self.tile_y = tile_y;
        self.selected_spritesheet_id = default_spritesheet;
    }

    /// Opens the dialog pre-populated from an existing NPC for editing.
    pub fn open_edit(&mut self, index: usize, npc: &NpcInstance) {
        self.reset();
        self.open = true;
        self.tile_x = npc.x;
        self.tile_y = npc.y;
        self.selected_spritesheet_id = Some(npc.spritesheet_id.clone());
        self.selected_facing = npc.facing;
        self.editing_index = Some(index);
        self.original_npc = Some(npc.clone());
        // Pre-populate patrol config
        if let Some(ref config) = npc.patrol_config {
            self.patrol_waypoints = config.waypoints.clone();
            self.patrol_mode = config.mode;
            self.patrol_speed = config.speed.to_string();
            self.patrol_pause = config.pause.to_string();
        }
        // Pre-populate event trigger config
        self.trigger_mode = npc.trigger_mode;
        self.event_triggers = npc.event_triggers.clone();
        // Pre-populate required_state
        if let Some(ref rs) = npc.required_state {
            self.has_required_state = true;
            self.required_state_key = rs.0.clone();
            self.required_state_value = rs.1.clone();
        } else {
            self.has_required_state = false;
        }
    }
}

/// Egui dialog for placing or editing an NPC on a tile.
pub fn npc_placement_dialog_ui(
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

            // Conditional visibility
            ui.separator();
            ui.label(egui::RichText::new("Conditional Visibility").strong());
            ui.horizontal(|ui| {
                ui.checkbox(&mut dialog.has_required_state, "Required state");
            });
            if dialog.has_required_state {
                ui.horizontal(|ui| {
                    ui.label("Key:");
                    ui.add(
                        egui::TextEdit::singleline(&mut dialog.required_state_key)
                            .desired_width(80.0),
                    );
                });
                ui.horizontal(|ui| {
                    ui.label("Value:");
                    ui.add(
                        egui::TextEdit::singleline(&mut dialog.required_state_value)
                            .desired_width(80.0),
                    );
                });
                ui.label("NPC is hidden unless state key matches value.");
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

            let dialog = &mut *dialog;
            render_action_editor(
                ui,
                &mut dialog.event_triggers,
                &mut dialog.action_editor,
                "npc_event_trigger",
                &map_entries,
                &project.face_portraits,
            );

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
                elevation: 0,
                required_state: if dialog.has_required_state {
                    Some((
                        dialog.required_state_key.clone(),
                        dialog.required_state_value.clone(),
                    ))
                } else {
                    None
                },
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
