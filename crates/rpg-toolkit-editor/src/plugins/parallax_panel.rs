//! Parallax layer editing panel — allows adding, editing, and removing
//! parallax background layers on the active map.

use bevy::prelude::*;
use bevy_egui::egui;

use crate::data::project::Project;

use rpg_toolkit_common::ParallaxLayer;

/// Maximum number of parallax layers allowed per map.
const MAX_PARALLAX_LAYERS: usize = 16;

/// Plugin that provides the parallax layer editing panel UI.
/// The panel is rendered inline in the left side panel by layer_panel.rs.
pub struct ParallaxPanelPlugin;

impl Plugin for ParallaxPanelPlugin {
    fn build(&self, _app: &mut App) {
        // No standalone system — rendered inline by layer_panel_ui
    }
}

/// Renders the parallax layer editing UI inline within a parent panel.
/// Called from `layer_panel_ui` when a map is active.
pub fn render_parallax_panel(ui: &mut egui::Ui, project: &mut ResMut<Project>) {
    ui.heading("Parallax Layers");
    ui.separator();

    let has_map = project.active_map().is_some();
    if !has_map {
        ui.label("No map loaded.");
        return;
    }

    let layer_count = project
        .active_map()
        .map(|m| m.parallax_layers.len())
        .unwrap_or(0);

    // "Add Layer" button — disabled at max
    let can_add = layer_count < MAX_PARALLAX_LAYERS;
    ui.horizontal(|ui| {
        if ui
            .add_enabled(can_add, egui::Button::new("+ Add Layer"))
            .clicked()
            && let Some(map) = project.active_map_mut()
        {
            map.parallax_layers.push(ParallaxLayer {
                image_path: String::new(),
                scroll_factor: 0.5,
                z_order: 0,
            });
        }
        ui.label(format!("{} / {}", layer_count, MAX_PARALLAX_LAYERS));
    });

    ui.separator();

    if layer_count == 0 {
        ui.label("No parallax layers. Click \"+ Add Layer\" to add one.");
        return;
    }

    // Render each layer row
    let mut remove_index: Option<usize> = None;

    egui::ScrollArea::vertical()
        .id_salt("parallax_layers_scroll")
        .max_height(300.0)
        .show(ui, |ui| {
            let layer_count = project.active_map().unwrap().parallax_layers.len();

            for i in 0..layer_count {
                let id_salt = format!("parallax_layer_{}", i);
                ui.push_id(&id_salt, |ui| {
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.strong(format!("Layer {}", i));
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if ui.button("✕ Remove").clicked() {
                                        remove_index = Some(i);
                                    }
                                },
                            );
                        });

                        // Image path text input
                        let map = project.active_map_mut().unwrap();
                        let layer = &mut map.parallax_layers[i];

                        ui.horizontal(|ui| {
                            ui.label("Image:");
                            ui.text_edit_singleline(&mut layer.image_path);
                        });

                        // Validation warning for empty image_path
                        if layer.image_path.is_empty() {
                            ui.colored_label(
                                egui::Color32::from_rgb(255, 180, 0),
                                "⚠ Image path is empty",
                            );
                        }

                        // Scroll factor slider
                        ui.horizontal(|ui| {
                            ui.label("Scroll Factor:");
                            ui.add(
                                egui::Slider::new(&mut layer.scroll_factor, 0.0..=1.0)
                                    .step_by(0.05)
                                    .fixed_decimals(2),
                            );
                        });

                        // Z-order drag value
                        ui.horizontal(|ui| {
                            ui.label("Z-Order:");
                            ui.add(egui::DragValue::new(&mut layer.z_order).range(-999..=999));
                        });
                    });

                    ui.add_space(4.0);
                });
            }
        });

    // Apply deferred removal
    if let Some(idx) = remove_index
        && let Some(map) = project.active_map_mut()
    {
        map.parallax_layers.remove(idx);
    }
}
