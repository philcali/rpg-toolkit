use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

use crate::data::{EditorState, TilesetData};

/// Plugin that renders the tile palette side panel.
pub struct TilePalettePlugin;

impl Plugin for TilePalettePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(EguiPrimaryContextPass, tile_palette_ui);
    }
}

fn tile_palette_ui(
    mut contexts: EguiContexts,
    tileset: Option<Res<TilesetData>>,
    mut editor_state: ResMut<EditorState>,
) -> Result {
    let Some(tileset) = tileset else {
        // No tileset loaded — show placeholder
        let ctx = contexts.ctx_mut()?;
        egui::SidePanel::right("tile_palette")
            .default_width(200.0)
            .show(ctx, |ui| {
                ui.heading("Tile Palette");
                ui.separator();
                ui.label("No tileset loaded.");
            });
        return Ok(());
    };

    let meta = &tileset.meta;
    let tile_w = meta.tile_width;
    let tile_h = meta.tile_height;
    let columns = meta.columns;
    let rows = meta.rows;

    // Register the tileset texture with egui so we can render it
    let texture_handle = tileset.texture.clone();
    let egui_texture_id = contexts.add_image(bevy_egui::EguiTextureHandle::Strong(texture_handle));

    let ctx = contexts.ctx_mut()?;

    egui::SidePanel::right("tile_palette")
        .default_width(200.0)
        .show(ctx, |ui| {
            ui.heading("Tile Palette");
            ui.separator();

            // Tile size info
            ui.label(format!("Tile size: {}x{}", tile_w, tile_h));
            ui.label(format!(
                "Grid: {} × {} ({} tiles)",
                columns,
                rows,
                columns * rows
            ));
            ui.separator();

            // Active brush indicator
            if let Some(ref brush) = editor_state.active_brush {
                ui.label(format!("Selected: ({}, {})", brush.col, brush.row));
            } else {
                ui.label("No tile selected");
            }
            ui.separator();

            // Scrollable tile grid
            let img_w = (columns * tile_w) as f32;
            let img_h = (rows * tile_h) as f32;

            // Compute display tile size to fit the panel
            let available_width = ui.available_width();
            let display_tile_size = (available_width / columns as f32).floor().max(8.0);

            egui::ScrollArea::vertical().show(ui, |ui| {
                let grid = egui::Grid::new("tile_grid").spacing([1.0, 1.0]);

                grid.show(ui, |ui| {
                    for row in 0..rows {
                        for col in 0..columns {
                            // Compute UV coordinates for this tile
                            let uv_min = egui::pos2(
                                (col * tile_w) as f32 / img_w,
                                (row * tile_h) as f32 / img_h,
                            );
                            let uv_max = egui::pos2(
                                ((col + 1) * tile_w) as f32 / img_w,
                                ((row + 1) * tile_h) as f32 / img_h,
                            );
                            let uv = egui::Rect::from_min_max(uv_min, uv_max);

                            let tile_image = egui::Image::new(egui::load::SizedTexture::new(
                                egui_texture_id,
                                [display_tile_size, display_tile_size],
                            ))
                            .uv(uv)
                            .sense(egui::Sense::click());

                            // Check if this tile is the active brush
                            let is_selected = editor_state
                                .active_brush
                                .is_some_and(|b| b.col == col && b.row == row);

                            let response = ui
                                .add(tile_image)
                                .on_hover_text(format!("Tile ({}, {})", col, row));

                            // Draw selection highlight
                            if is_selected {
                                let rect = response.rect;
                                ui.painter().rect_stroke(
                                    rect,
                                    0.0,
                                    egui::Stroke::new(2.0, egui::Color32::YELLOW),
                                    egui::StrokeKind::Outside,
                                );
                            }

                            if response.clicked() {
                                editor_state.active_brush =
                                    Some(crate::data::map::TileIndex { col, row });
                            }
                        }
                        ui.end_row();
                    }
                });
            });
        });

    Ok(())
}
