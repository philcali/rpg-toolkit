use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

use crate::data::EditCommand;
use crate::data::map::MapId;
use crate::data::project::Project;

/// Plugin that renders the layer management panel and the map browser,
/// combined into a single left side panel.
pub struct LayerPanelPlugin;

impl Plugin for LayerPanelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LayerCounter>()
            .init_resource::<MapBrowserState>()
            .add_systems(EguiPrimaryContextPass, layer_panel_ui);
    }
}

/// Tracks how many layers have been created for default naming.
#[derive(Resource)]
struct LayerCounter {
    next_id: u32,
}

impl Default for LayerCounter {
    fn default() -> Self {
        Self { next_id: 1 }
    }
}

/// Persistent state for the Map Browser section.
#[derive(Resource, Default)]
struct MapBrowserState {
    renaming: Option<MapId>,
    rename_buffer: String,
    pending_delete: Option<MapId>,
}

/// Deferred action from the map browser UI.
enum BrowserAction {
    Open(MapId),
    StartRename(MapId, String),
    CommitRename(MapId, String),
    CancelRename,
    RequestDelete(MapId),
    ConfirmDelete(MapId),
    CancelDelete,
}

fn layer_panel_ui(
    mut contexts: EguiContexts,
    mut counter: ResMut<LayerCounter>,
    mut edit_events: MessageWriter<EditCommand>,
    mut project: ResMut<Project>,
    mut browser_state: ResMut<MapBrowserState>,
) -> Result {
    let ctx = contexts.ctx_mut()?;

    let mut browser_actions: Vec<BrowserAction> = Vec::new();

    egui::SidePanel::left("layer_panel")
        .default_width(160.0)
        .resizable(true)
        .show(ctx, |ui| {
            // ── Layers section ──
            ui.heading("Layers");
            ui.separator();

            let has_active_map = project.active_map().is_some();

            if !has_active_map {
                ui.label("No map loaded.");
                ui.add_space(8.0);
                render_map_browser(ui, &project, &mut browser_state, &mut browser_actions);
                return;
            }

            // Read layer info immutably first for rendering
            let (layer_count, active_layer_index, layer_info) = {
                let map = project.active_map().unwrap();
                let info: Vec<(String, bool)> = map
                    .layers
                    .iter()
                    .map(|l| (l.name.clone(), l.visible))
                    .collect();
                (map.layers.len(), map.active_layer_index, info)
            };

            let mut should_add = false;
            let mut should_delete = false;
            let mut toggle_vis: Option<usize> = None;
            let mut select: Option<usize> = None;

            ui.horizontal(|ui| {
                if ui.button("+ Add").clicked() {
                    should_add = true;
                }
                let can_delete = layer_count > 1;
                if ui
                    .add_enabled(can_delete, egui::Button::new("− Delete"))
                    .clicked()
                {
                    should_delete = true;
                }
            });

            ui.separator();

            egui::ScrollArea::vertical()
                .id_salt("layers_scroll")
                .max_height(ui.available_height() * 0.5)
                .show(ui, |ui| {
                    for i in (0..layer_count).rev() {
                        let (ref name, visible) = layer_info[i];
                        let is_active = i == active_layer_index;

                        ui.horizontal(|ui| {
                            let vis_label = if visible { "👁" } else { "  " };
                            if ui.small_button(vis_label).clicked() {
                                toggle_vis = Some(i);
                            }

                            let label = egui::RichText::new(name);
                            let label = if is_active { label.strong() } else { label };

                            let response = ui.selectable_label(is_active, label);
                            if response.clicked() {
                                select = Some(i);
                            }
                        });
                    }
                });

            // Apply deferred layer mutations via the active map in Project
            if (should_add || should_delete || toggle_vis.is_some() || select.is_some())
                && let Some(map) = project.active_map_mut()
            {
                if should_add {
                    let name = format!("Layer {}", counter.next_id);
                    counter.next_id += 1;
                    let cmd = map.add_layer(name);
                    edit_events.write(cmd);
                }
                if should_delete {
                    let idx = map.active_layer_index;
                    if let Ok(cmd) = map.delete_layer(idx) {
                        edit_events.write(cmd);
                    }
                }
                if let Some(idx) = toggle_vis {
                    map.toggle_layer_visibility(idx);
                }
                if let Some(idx) = select {
                    let _ = map.set_active_layer(idx);
                }
            }

            // ── Map Browser section (below layers) ──
            ui.add_space(8.0);
            render_map_browser(ui, &project, &mut browser_state, &mut browser_actions);
        });

    // Delete confirmation dialog (rendered outside the panel)
    if let Some(ref delete_id) = browser_state.pending_delete {
        let map_name = project
            .maps
            .get(delete_id)
            .map(|m| m.name.clone())
            .unwrap_or_else(|| "???".to_string());

        let mut still_open = true;
        egui::Window::new("Delete Map")
            .collapsible(false)
            .resizable(false)
            .open(&mut still_open)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.label(format!("Are you sure you want to delete \"{}\"?", map_name));
                ui.label("This action cannot be undone.");
                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("Delete").clicked() {
                        browser_actions.push(BrowserAction::ConfirmDelete(delete_id.clone()));
                    }
                    if ui.button("Cancel").clicked() {
                        browser_actions.push(BrowserAction::CancelDelete);
                    }
                });
            });

        if !still_open {
            browser_actions.push(BrowserAction::CancelDelete);
        }
    }

    // Apply deferred browser actions
    for action in browser_actions {
        match action {
            BrowserAction::Open(id) => {
                project.open_map_tab(id);
            }
            BrowserAction::StartRename(id, current_name) => {
                browser_state.renaming = Some(id);
                browser_state.rename_buffer = current_name;
            }
            BrowserAction::CommitRename(id, new_name) => {
                if let Some(map) = project.maps.get_mut(&id) {
                    map.name = new_name;
                }
                browser_state.renaming = None;
                browser_state.rename_buffer.clear();
            }
            BrowserAction::CancelRename => {
                browser_state.renaming = None;
                browser_state.rename_buffer.clear();
            }
            BrowserAction::RequestDelete(id) => {
                browser_state.pending_delete = Some(id);
            }
            BrowserAction::ConfirmDelete(id) => {
                if let Err(e) = project.remove_map(&id) {
                    warn!("Cannot delete map: {}", e);
                }
                browser_state.pending_delete = None;
            }
            BrowserAction::CancelDelete => {
                browser_state.pending_delete = None;
            }
        }
    }

    Ok(())
}

/// Renders the "Maps" browser section inside the side panel.
fn render_map_browser(
    ui: &mut egui::Ui,
    project: &ResMut<Project>,
    browser_state: &mut MapBrowserState,
    actions: &mut Vec<BrowserAction>,
) {
    ui.heading("Maps");
    ui.separator();

    if project.maps.is_empty() {
        ui.label("No maps in project.");
        return;
    }

    let mut sorted_maps: Vec<_> = project
        .maps
        .iter()
        .map(|(id, map)| (id.clone(), map.name.clone()))
        .collect();
    sorted_maps.sort_by(|a, b| a.1.cmp(&b.1).then(a.0.cmp(&b.0)));

    let active_map_id = project.active_map_id().cloned();

    egui::ScrollArea::vertical()
        .id_salt("maps_scroll")
        .show(ui, |ui| {
            for (map_id, map_name) in &sorted_maps {
                let is_active = active_map_id.as_ref() == Some(map_id);
                let is_renaming = browser_state.renaming.as_ref() == Some(map_id);

                if is_renaming {
                    let response = ui.text_edit_singleline(&mut browser_state.rename_buffer);

                    if response.gained_focus() || !response.has_focus() {
                        response.request_focus();
                    }

                    if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        let new_name = browser_state.rename_buffer.trim().to_string();
                        if !new_name.is_empty() {
                            actions.push(BrowserAction::CommitRename(map_id.clone(), new_name));
                        } else {
                            actions.push(BrowserAction::CancelRename);
                        }
                    } else if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                        actions.push(BrowserAction::CancelRename);
                    }
                } else {
                    let response = ui.selectable_label(is_active, map_name);

                    if response.double_clicked() {
                        actions.push(BrowserAction::Open(map_id.clone()));
                    }

                    response.context_menu(|ui| {
                        if ui.button("Open").clicked() {
                            actions.push(BrowserAction::Open(map_id.clone()));
                            ui.close();
                        }
                        if ui.button("Rename").clicked() {
                            actions
                                .push(BrowserAction::StartRename(map_id.clone(), map_name.clone()));
                            ui.close();
                        }
                        if ui.button("Delete").clicked() {
                            actions.push(BrowserAction::RequestDelete(map_id.clone()));
                            ui.close();
                        }
                    });
                }
            }
        });
}
