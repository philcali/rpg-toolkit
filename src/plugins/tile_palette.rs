use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

use crate::data::map::TileRef;
use crate::data::project::Project;
use crate::data::{EditorState, StampBrushSelection};

/// Tracks the drag start tile position in egui memory for stamp brush selection.
#[derive(Clone, Default)]
struct PaletteDragState {
    /// The (col, row) where the drag started, if a drag is in progress.
    start: Option<(u32, u32)>,
}

/// Plugin that renders the tile palette side panel.
pub struct TilePalettePlugin;

impl Plugin for TilePalettePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(EguiPrimaryContextPass, tile_palette_ui);
    }
}

fn tile_palette_ui(
    mut contexts: EguiContexts,
    project: Res<Project>,
    mut editor_state: ResMut<EditorState>,
) -> Result {
    // If project has no tilesets, show placeholder
    let has_tilesets = !project.tilesets.is_empty();

    if !has_tilesets {
        let ctx = contexts.ctx_mut()?;
        egui::SidePanel::right("tile_palette")
            .default_width(200.0)
            .show(ctx, |ui| {
                ui.heading("Tile Palette");
                ui.separator();
                ui.label("No tileset loaded.");
            });
        return Ok(());
    }

    // Collect and sort tileset entries by name (then ID for stability)
    let mut sorted_tilesets: Vec<_> = project.tilesets.iter().collect();
    sorted_tilesets.sort_by(|(id_a, entry_a), (id_b, entry_b)| {
        entry_a
            .meta
            .file_path
            .cmp(&entry_b.meta.file_path)
            .then_with(|| id_a.cmp(id_b))
    });

    // Auto-select first tileset if active_tileset_tab is None or invalid
    let active_tab_valid = editor_state
        .active_tileset_tab
        .as_ref()
        .is_some_and(|id| project.tilesets.contains_key(id));

    if !active_tab_valid {
        editor_state.active_tileset_tab = sorted_tilesets.first().map(|(id, _)| (*id).clone());
    }

    let active_tileset_id = editor_state.active_tileset_tab.clone();

    // Register textures with egui for each tileset and find the active one
    let mut tileset_textures: Vec<(String, egui::TextureId)> = Vec::new();
    for (id, entry) in &sorted_tilesets {
        let texture_handle = entry.texture.clone();
        let egui_tex_id = contexts.add_image(bevy_egui::EguiTextureHandle::Strong(texture_handle));
        tileset_textures.push(((*id).clone(), egui_tex_id));
    }

    let ctx = contexts.ctx_mut()?;

    egui::SidePanel::right("tile_palette")
        .default_width(200.0)
        .show(ctx, |ui| {
            ui.heading("Tile Palette");
            ui.separator();

            // Tileset Tab Bar
            ui.horizontal_wrapped(|ui| {
                for (id, entry) in &sorted_tilesets {
                    let label = tileset_tab_label(&entry.meta.file_path);
                    let is_active = active_tileset_id.as_deref() == Some(*id);
                    if ui.selectable_label(is_active, &label).clicked() {
                        editor_state.active_tileset_tab = Some((*id).clone());
                    }
                }
            });
            ui.separator();

            // Find the active tileset entry and its egui texture
            let Some(active_id) = &editor_state.active_tileset_tab else {
                ui.label("No tileset selected.");
                return;
            };
            let active_id = active_id.clone();

            let Some(entry) = project.tilesets.get(&active_id) else {
                ui.label("No tileset selected.");
                return;
            };

            let egui_texture_id = tileset_textures
                .iter()
                .find(|(id, _)| id == &active_id)
                .map(|(_, tex_id)| *tex_id);

            let Some(egui_texture_id) = egui_texture_id else {
                ui.label("Texture not available.");
                return;
            };

            let meta = &entry.meta;
            let tile_w = meta.tile_width;
            let tile_h = meta.tile_height;
            let columns = meta.columns;
            let rows = meta.rows;

            // Tile size info
            ui.label(format!("Tile size: {}x{}", tile_w, tile_h));
            ui.label(format!(
                "Grid: {} × {} ({} tiles)",
                columns,
                rows,
                columns * rows
            ));
            ui.separator();

            // Active brush / stamp indicator
            if let Some(ref stamp) = editor_state.stamp_brush {
                if stamp.tileset_id == active_id {
                    ui.label(format!(
                        "Stamp: ({},{}) {}×{}",
                        stamp.top_left_col, stamp.top_left_row, stamp.width, stamp.height
                    ));
                } else if let Some(ref brush) = editor_state.active_brush {
                    if brush.tileset_id == active_id {
                        ui.label(format!("Selected: ({}, {})", brush.col, brush.row));
                    } else {
                        ui.label("No tile selected");
                    }
                } else {
                    ui.label("No tile selected");
                }
            } else if let Some(ref brush) = editor_state.active_brush {
                if brush.tileset_id == active_id {
                    ui.label(format!("Selected: ({}, {})", brush.col, brush.row));
                } else {
                    ui.label("No tile selected");
                }
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

            // Read drag state from egui memory
            let drag_id = egui::Id::new("palette_drag_state");
            let mut drag_state = ui
                .memory(|mem| mem.data.get_temp::<PaletteDragState>(drag_id))
                .unwrap_or_default();

            egui::ScrollArea::vertical().show(ui, |ui| {
                // Collect tile responses so we can process drag logic after the grid
                let mut tile_responses: Vec<(u32, u32, egui::Response)> = Vec::new();

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
                            .sense(egui::Sense::click_and_drag());

                            let response = ui
                                .add(tile_image)
                                .on_hover_text(format!("Tile ({}, {})", col, row));

                            tile_responses.push((col, row, response));
                        }
                        ui.end_row();
                    }
                });

                // --- Process drag logic across all tile responses ---

                // Detect drag start: any tile that just got a primary press
                for (col, row, resp) in &tile_responses {
                    if resp.drag_started_by(egui::PointerButton::Primary) {
                        drag_state.start = Some((*col, *row));
                    }
                }

                // Determine the current hover tile (for live preview during drag)
                let current_hover: Option<(u32, u32)> = tile_responses
                    .iter()
                    .find(|(_, _, resp)| resp.hovered())
                    .map(|(col, row, _)| (*col, *row));

                // Compute the selection rectangle for highlighting during drag
                let selection_rect: Option<(u32, u32, u32, u32)> =
                    if let (Some((sc, sr)), Some((hc, hr))) = (drag_state.start, current_hover) {
                        let min_c = sc.min(hc);
                        let max_c = sc.max(hc);
                        let min_r = sr.min(hr);
                        let max_r = sr.max(hr);
                        Some((min_c, min_r, max_c, max_r))
                    } else {
                        None
                    };

                // Draw highlights
                for (col, row, resp) in &tile_responses {
                    let c = *col;
                    let r = *row;

                    // During active drag, highlight the selection rectangle
                    if let Some((min_c, min_r, max_c, max_r)) = selection_rect
                        && drag_state.start.is_some()
                            && c >= min_c
                            && c <= max_c
                            && r >= min_r
                            && r <= max_r
                        {
                            ui.painter().rect_stroke(
                                resp.rect,
                                0.0,
                                egui::Stroke::new(
                                    2.0,
                                    egui::Color32::from_rgba_unmultiplied(0, 200, 255, 200),
                                ),
                                egui::StrokeKind::Outside,
                            );
                            continue;
                        }

                    // Highlight stamp brush selection
                    if let Some(ref stamp) = editor_state.stamp_brush
                        && stamp.tileset_id == active_id
                            && c >= stamp.top_left_col
                            && c < stamp.top_left_col + stamp.width
                            && r >= stamp.top_left_row
                            && r < stamp.top_left_row + stamp.height
                        {
                            ui.painter().rect_stroke(
                                resp.rect,
                                0.0,
                                egui::Stroke::new(
                                    2.0,
                                    egui::Color32::from_rgba_unmultiplied(0, 255, 128, 200),
                                ),
                                egui::StrokeKind::Outside,
                            );
                            continue;
                        }

                    // Highlight single active brush
                    let is_selected = editor_state
                        .active_brush
                        .as_ref()
                        .is_some_and(|b| b.tileset_id == active_id && b.col == c && b.row == r);
                    if is_selected {
                        ui.painter().rect_stroke(
                            resp.rect,
                            0.0,
                            egui::Stroke::new(2.0, egui::Color32::YELLOW),
                            egui::StrokeKind::Outside,
                        );
                    }
                }

                // Detect drag release or simple click
                let drag_released = tile_responses
                    .iter()
                    .any(|(_, _, resp)| resp.drag_stopped_by(egui::PointerButton::Primary));

                // Also detect simple clicks (no drag movement)
                let clicked_tile: Option<(u32, u32)> = tile_responses
                    .iter()
                    .find(|(_, _, resp)| resp.clicked())
                    .map(|(col, row, _)| (*col, *row));

                if drag_released {
                    if let Some((start_col, start_row)) = drag_state.start {
                        // Find the end tile: use hover position, or fall back to start
                        let (end_col, end_row) = current_hover.unwrap_or((start_col, start_row));

                        let min_col = start_col.min(end_col);
                        let max_col = start_col.max(end_col);
                        let min_row = start_row.min(end_row);
                        let max_row = start_row.max(end_row);
                        let w = max_col - min_col + 1;
                        let h = max_row - min_row + 1;

                        if w == 1 && h == 1 {
                            // Single-click: set active_brush, clear stamp
                            editor_state.active_brush = Some(TileRef {
                                tileset_id: active_id.clone(),
                                col: min_col,
                                row: min_row,
                            });
                            editor_state.stamp_brush = None;
                        } else {
                            // Multi-tile drag: set stamp brush selection
                            editor_state.stamp_brush = Some(StampBrushSelection {
                                tileset_id: active_id.clone(),
                                top_left_col: min_col,
                                top_left_row: min_row,
                                width: w,
                                height: h,
                            });
                            // Also set active_brush to top-left tile for fallback
                            editor_state.active_brush = Some(TileRef {
                                tileset_id: active_id.clone(),
                                col: min_col,
                                row: min_row,
                            });
                        }
                    }
                    drag_state.start = None;
                } else if let Some((col, row)) = clicked_tile {
                    // Simple click with no drag — select single tile
                    editor_state.active_brush = Some(TileRef {
                        tileset_id: active_id.clone(),
                        col,
                        row,
                    });
                    editor_state.stamp_brush = None;
                    drag_state.start = None;
                }
            });

            // Persist drag state back to egui memory
            ui.memory_mut(|mem| mem.data.insert_temp(drag_id, drag_state));
        });

    Ok(())
}

/// Extracts a short label from a file path for the tileset tab.
fn tileset_tab_label(file_path: &str) -> String {
    std::path::Path::new(file_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Tileset")
        .to_string()
}
