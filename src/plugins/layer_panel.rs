use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

use crate::data::{EditCommand, MapData};

/// Plugin that renders the layer management panel.
///
/// TODO: Add inline layer renaming so users can double-click a layer name
/// to edit it and give layers meaningful labels (e.g. "Ground", "Objects", "Overlay").
pub struct LayerPanelPlugin;

impl Plugin for LayerPanelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LayerCounter>()
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

fn layer_panel_ui(
    mut contexts: EguiContexts,
    mut map: Option<ResMut<MapData>>,
    mut counter: ResMut<LayerCounter>,
    mut edit_events: MessageWriter<EditCommand>,
) -> Result {
    let ctx = contexts.ctx_mut()?;

    egui::SidePanel::left("layer_panel")
        .default_width(160.0)
        .resizable(true)
        .show(ctx, |ui| {
            ui.heading("Layers");
            ui.separator();

            let Some(ref mut map) = map else {
                ui.label("No map loaded.");
                return;
            };

            let layer_count = map.layers.len();

            // Deferred mutation flags — collected during UI, applied after rendering
            let mut should_add = false;
            let mut should_delete = false;
            let mut toggle_vis: Option<usize> = None;
            let mut select: Option<usize> = None;

            // Add / Delete buttons
            ui.horizontal(|ui| {
                if ui.button("+ Add").clicked() {
                    should_add = true;
                }

                let can_delete = layer_count > 1;
                if ui.add_enabled(can_delete, egui::Button::new("− Delete")).clicked() {
                    should_delete = true;
                }
            });

            ui.separator();

            // Layer list — top-to-bottom = highest to lowest in stacking order
            egui::ScrollArea::vertical().show(ui, |ui| {
                for i in (0..layer_count).rev() {
                    let layer = &map.layers[i];
                    let is_active = i == map.active_layer_index;

                    ui.horizontal(|ui| {
                        // Visibility toggle
                        let vis_label = if layer.visible { "👁" } else { "  " };
                        if ui.small_button(vis_label).clicked() {
                            toggle_vis = Some(i);
                        }

                        // Layer name — highlight active layer
                        let label = egui::RichText::new(&layer.name);
                        let label = if is_active {
                            label.strong()
                        } else {
                            label
                        };

                        let response = ui.selectable_label(is_active, label);
                        if response.clicked() {
                            select = Some(i);
                        }
                    });
                }
            });

            // Apply all deferred mutations after the UI has finished reading map.layers
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
        });

    Ok(())
}
