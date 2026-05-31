//! Editor panel for viewing and editing game state flags, and inspecting
//! which NPCs / tiles have conditional visibility (`required_state`).
//!
//! This panel is a testing aid — the flags it manages are local to the
//! editor and do not persist in the project file. They simulate the
//! runtime `GameState` resource so the editor can evaluate
//! `required_state` conditions during map editing.

use std::collections::HashMap;

use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

use crate::data::project::Project;

// ── Resources ──────────────────────────────────────────────────────────────

/// Editor-local game state flags for testing conditional visibility.
#[derive(Resource, Default)]
pub struct GameState {
    pub flags: HashMap<String, String>,
}

/// Persistent state for the game state panel UI.
#[derive(Resource, Default)]
pub struct GameStatePanelState {
    /// Index of the flag being edited (None = add mode).
    pub editing_flag: Option<String>,
    /// Buffer for the key being added/edited.
    pub key_buffer: String,
    /// Buffer for the value being added/edited.
    pub value_buffer: String,
}

// ── Plugin ─────────────────────────────────────────────────────────────────

pub struct GameStatePlugin;

impl Plugin for GameStatePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameState>()
            .init_resource::<GameStatePanelState>()
            .add_systems(EguiPrimaryContextPass, game_state_panel_ui);
    }
}

// ── UI ─────────────────────────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
fn game_state_panel_ui(
    mut contexts: EguiContexts,
    mut game_state: ResMut<GameState>,
    mut panel_state: ResMut<GameStatePanelState>,
    project: Res<Project>,
) -> Result {
    let ctx = contexts.ctx_mut()?;

    egui::SidePanel::right("game_state_panel")
        .default_width(320.0)
        .resizable(true)
        .show(ctx, |ui| {
            ui.heading("Game State");
            ui.separator();

            render_flags_section(ui, &mut game_state, &mut panel_state);

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(4.0);

            render_npc_conditions(ui, &project);

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(4.0);

            render_tile_conditions(ui, &project);
        });

    Ok(())
}

// ── Flags section ──────────────────────────────────────────────────────────

fn render_flags_section(
    ui: &mut egui::Ui,
    game_state: &mut ResMut<GameState>,
    panel_state: &mut ResMut<GameStatePanelState>,
) {
    ui.weak_label("Flags");

    // Collect flag data before entering the closure to avoid borrow conflicts
    let mut sorted_flags: Vec<(String, String)> = game_state
        .flags
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    sorted_flags.sort_by(|a, b| a.0.cmp(&b.0));

    let mut delete_keys: Vec<String> = Vec::new();

    // List of existing flags
    egui::ScrollArea::vertical()
        .id_salt("gs_flags_scroll")
        .max_height(120.0)
        .show(ui, |ui| {
            for (key, value) in &sorted_flags {
                let label = format!("{} = {}", key, value);

                ui.horizontal(|ui| {
                    if ui.button("✏").clicked() {
                        panel_state.editing_flag = Some(key.clone());
                        panel_state.key_buffer = key.clone();
                        panel_state.value_buffer = value.clone();
                    }
                    if ui.button("✕").clicked() {
                        delete_keys.push(key.clone());
                    }
                    ui.label(label);
                });
            }
        });

    // Apply deletions after the closure
    for key in &delete_keys {
        game_state.flags.remove(key.as_str());
    }
    if panel_state.editing_flag.is_some()
        && delete_keys.contains(panel_state.editing_flag.as_ref().unwrap())
    {
        panel_state.editing_flag = None;
        panel_state.key_buffer.clear();
        panel_state.value_buffer.clear();
    }

    // Add / edit form
    let is_editing = panel_state.editing_flag.is_some();
    let form_label = if is_editing { "Edit Flag" } else { "Add Flag" };
    ui.collapsing(form_label, |ui| {
        ui.horizontal(|ui| {
            ui.label("Key:");
            ui.text_edit_singleline(&mut panel_state.key_buffer);
        });
        ui.horizontal(|ui| {
            ui.label("Value:");
            ui.text_edit_singleline(&mut panel_state.value_buffer);
        });
        ui.horizontal(|ui| {
            if ui
                .button(if is_editing { "Update" } else { "Add" })
                .clicked()
            {
                let key = panel_state.key_buffer.trim().to_string();
                let value = panel_state.value_buffer.clone();
                if !key.is_empty() {
                    game_state.flags.insert(key.clone(), value);
                    panel_state.editing_flag = Some(key);
                }
            }
            if is_editing && ui.button("Cancel").clicked() {
                panel_state.editing_flag = None;
                panel_state.key_buffer.clear();
                panel_state.value_buffer.clear();
            }
        });
    });
}

// ── NPC conditions section ─────────────────────────────────────────────────

fn render_npc_conditions(ui: &mut egui::Ui, project: &Res<Project>) {
    ui.weak_label("NPC Conditions");

    let Some(map) = project.active_map() else {
        ui.weak("No map loaded.");
        return;
    };

    let conditioned: Vec<(usize, String, &str, &str)> = map
        .npcs
        .iter()
        .enumerate()
        .filter_map(|(i, npc)| {
            npc.required_state.as_ref().map(|(key, value)| {
                let name = &npc.spritesheet_id;
                (i, name.clone(), key.as_str(), value.as_str())
            })
        })
        .collect();

    if conditioned.is_empty() {
        ui.weak("No NPCs with required_state on this map.");
        return;
    }

    egui::ScrollArea::vertical()
        .id_salt("gs_npc_conditions_scroll")
        .max_height(160.0)
        .show(ui, |ui| {
            ui.weak("Index · Spritesheet · Key = Value");
            ui.separator();
            for (i, spritesheet, key, value) in &conditioned {
                let label = format!("{} · {} · {} = {}", i, spritesheet, key, value);
                ui.weak(label);
            }
        });
}

// ── Tile conditions section ────────────────────────────────────────────────

fn render_tile_conditions(ui: &mut egui::Ui, project: &Res<Project>) {
    ui.weak_label("Tile Conditions");

    let Some(map) = project.active_map() else {
        ui.weak("No map loaded.");
        return;
    };

    let Some(layer) = map.layers.get(map.active_layer_index) else {
        ui.weak("No active layer.");
        return;
    };

    let conditioned: Vec<(usize, usize, &str, &str)> = layer
        .attributes
        .cells
        .iter()
        .enumerate()
        .flat_map(|(y, row)| {
            row.iter().enumerate().filter_map(move |(x, attrs)| {
                attrs
                    .required_state
                    .as_ref()
                    .map(|(key, value)| (x, y, key.as_str(), value.as_str()))
            })
        })
        .collect();

    if conditioned.is_empty() {
        ui.weak(format!(
            "No tiles with required_state on \"{}\".",
            layer.name
        ));
        return;
    }

    egui::ScrollArea::vertical()
        .id_salt("gs_tile_conditions_scroll")
        .max_height(160.0)
        .show(ui, |ui| {
            ui.weak("X, Y · Key = Value");
            ui.separator();
            for (x, y, key, value) in &conditioned {
                let label = format!("{}, {} · {} = {}", x, y, key, value);
                ui.weak(label);
            }
        });
}

// ── Helpers ────────────────────────────────────────────────────────────────

trait EguiExt {
    fn weak_label(&mut self, text: &str);
}

impl EguiExt for egui::Ui {
    fn weak_label(&mut self, text: &str) {
        self.label(egui::RichText::new(text).weak());
    }
}
