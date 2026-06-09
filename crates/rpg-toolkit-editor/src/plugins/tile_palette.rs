use std::collections::HashMap;

use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

use crate::data::map::TileRef;
use crate::data::project::Project;
use crate::data::{
    AnimationEditorState, AppEditorMode, EditorState, EditorUiSet, StampBrushSelection,
    clamp_palette_scale,
};
use crate::plugins::app_shell::ErrorDialog;
use crate::plugins::searchable_combobox::searchable_combobox;
use crate::plugins::spritesheet::{
    RemoveSpritesheetDialog, process_spritesheet_remove, spritesheet_section_ui,
};
use rpg_toolkit_common::{
    AnimationFrame, SpritesheetId, TileAnimation, compute_animation_frame_index,
    validate_tile_animation,
};

/// Tracks the drag start tile position in egui memory for stamp brush selection.
#[derive(Clone, Default)]
struct PaletteDragState {
    /// The (col, row) where the drag started, if a drag is in progress.
    start: Option<(u32, u32)>,
}

/// Bevy texture handles for loaded spritesheet images, keyed by spritesheet ID.
/// Populated when spritesheets are imported or when a project is loaded.
#[derive(Resource, Default)]
pub struct SpritesheetTextures {
    pub handles: HashMap<SpritesheetId, Handle<Image>>,
}

/// Plugin that renders the tile palette side panel.
pub struct TilePalettePlugin;

impl Plugin for TilePalettePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SpritesheetTextures>()
            .init_resource::<AnimationEditorState>()
            .add_systems(
                EguiPrimaryContextPass,
                (sync_spritesheet_textures, tile_palette_ui)
                    .chain()
                    .in_set(EditorUiSet::Panels)
                    .run_if(resource_equals(AppEditorMode::MapEditor)),
            );
    }
}

/// Ensures every spritesheet in the project has a corresponding Bevy texture handle,
/// and removes handles for spritesheets that no longer exist.
fn sync_spritesheet_textures(
    project: Res<Project>,
    asset_server: Res<AssetServer>,
    mut ss_textures: ResMut<SpritesheetTextures>,
) {
    // Load textures for new spritesheets
    for (id, ss) in &project.spritesheets {
        if !ss_textures.handles.contains_key(id) && !ss.file_path.is_empty() {
            let handle: Handle<Image> = asset_server.load(ss.file_path.clone());
            ss_textures.handles.insert(id.clone(), handle);
        }
    }

    // Remove handles for deleted spritesheets
    ss_textures
        .handles
        .retain(|id, _| project.spritesheets.contains_key(id));
}

#[allow(clippy::too_many_arguments)]
fn tile_palette_ui(
    mut contexts: EguiContexts,
    mut project: ResMut<Project>,
    mut editor_state: ResMut<EditorState>,
    mut error_dialog: ResMut<ErrorDialog>,
    mut remove_dialog: ResMut<RemoveSpritesheetDialog>,
    ss_textures: Res<SpritesheetTextures>,
    mut anim_state: ResMut<AnimationEditorState>,
) -> Result {
    // If project has no tilesets, show placeholder + spritesheet section
    let has_tilesets = !project.tilesets.is_empty();

    if !has_tilesets {
        // Register spritesheet textures with egui before getting ctx
        let mut ss_egui_textures: HashMap<SpritesheetId, egui::TextureId> = HashMap::new();
        for (id, handle) in &ss_textures.handles {
            let egui_tex = contexts.add_image(bevy_egui::EguiTextureHandle::Strong(handle.clone()));
            ss_egui_textures.insert(id.clone(), egui_tex);
        }

        let ctx = contexts.ctx_mut()?;
        egui::SidePanel::right("tile_palette")
            .default_width(280.0)
            .resizable(true)
            .show(ctx, |ui| {
                ui.heading("Tile Palette");
                ui.separator();
                ui.label("No tileset loaded.");
                ui.add_space(8.0);
                spritesheet_section_ui(ui, &mut project, &mut error_dialog, &ss_egui_textures);
            });

        process_spritesheet_remove(ctx, &mut project, &mut remove_dialog);
        return Ok(());
    }

    // Collect and sort tileset entries by name (then ID for stability).
    // We collect the IDs and file_path labels into owned data so we don't
    // hold an immutable borrow on `project` across the mutable closure.
    let mut sorted_ids: Vec<(String, String)> = project
        .tilesets
        .iter()
        .map(|(id, entry)| (id.clone(), entry.meta.file_path.clone()))
        .collect();
    sorted_ids
        .sort_by(|(id_a, path_a), (id_b, path_b)| path_a.cmp(path_b).then_with(|| id_a.cmp(id_b)));

    // Auto-select first tileset if active_tileset_tab is None or invalid
    let active_tab_valid = editor_state
        .active_tileset_tab
        .as_ref()
        .is_some_and(|id| project.tilesets.contains_key(id));

    if !active_tab_valid {
        editor_state.active_tileset_tab = sorted_ids.first().map(|(id, _)| id.clone());
        // Set default palette scale based on the tileset's tile width
        if let Some((id, _)) = sorted_ids.first()
            && let Some(entry) = project.tilesets.get(id)
        {
            let default_scale = (entry.meta.tile_width as f32).max(24.0);
            editor_state.palette_tile_scale = clamp_palette_scale(default_scale);
        }
    }

    let active_tileset_id = editor_state.active_tileset_tab.clone();

    // Register textures with egui for each tileset
    let mut tileset_textures: Vec<(String, egui::TextureId)> = Vec::new();
    for (id, _) in &sorted_ids {
        if let Some(entry) = project.tilesets.get(id) {
            let texture_handle = entry.texture.clone();
            let egui_tex_id =
                contexts.add_image(bevy_egui::EguiTextureHandle::Strong(texture_handle));
            tileset_textures.push((id.clone(), egui_tex_id));
        }
    }

    // Register spritesheet textures with egui
    let mut ss_egui_textures: HashMap<SpritesheetId, egui::TextureId> = HashMap::new();
    for (id, handle) in &ss_textures.handles {
        let egui_tex = contexts.add_image(bevy_egui::EguiTextureHandle::Strong(handle.clone()));
        ss_egui_textures.insert(id.clone(), egui_tex);
    }

    let ctx = contexts.ctx_mut()?;

    egui::SidePanel::right("tile_palette")
        .default_width(280.0)
        .resizable(true)
        .show(ctx, |ui| {
            ui.heading("Tile Palette");
            ui.separator();

            // Tileset Selector (searchable combobox)
            let current_label = active_tileset_id
                .as_ref()
                .and_then(|id| sorted_ids.iter().find(|(sid, _)| sid == id))
                .map(|(_, path)| tileset_tab_label(path))
                .unwrap_or_else(|| "Select tileset…".to_string());

            if let Some(selected_id) = searchable_combobox(
                ui,
                "tileset_selector",
                &current_label,
                &sorted_ids,
                &mut editor_state.tileset_search_buffer,
            ) {
                editor_state.active_tileset_tab = Some(selected_id);
            }
            ui.separator();

            // Find the active tileset entry and its egui texture
            let Some(active_id) = &editor_state.active_tileset_tab else {
                ui.label("No tileset selected.");
                ui.add_space(8.0);
                spritesheet_section_ui(ui, &mut project, &mut error_dialog, &ss_egui_textures);
                return;
            };
            let active_id = active_id.clone();

            let Some(entry) = project.tilesets.get(&active_id) else {
                ui.label("No tileset selected.");
                ui.add_space(8.0);
                spritesheet_section_ui(ui, &mut project, &mut error_dialog, &ss_egui_textures);
                return;
            };

            let egui_texture_id = tileset_textures
                .iter()
                .find(|(id, _)| id == &active_id)
                .map(|(_, tex_id)| *tex_id);

            let Some(egui_texture_id) = egui_texture_id else {
                ui.label("Texture not available.");
                ui.add_space(8.0);
                spritesheet_section_ui(ui, &mut project, &mut error_dialog, &ss_egui_textures);
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

            // Zoom slider for palette tile scale
            ui.horizontal(|ui| {
                ui.label("Zoom:");
                let slider = egui::Slider::new(&mut editor_state.palette_tile_scale, 16.0..=128.0)
                    .clamping(egui::SliderClamping::Always);
                ui.add(slider);
            });
            editor_state.palette_tile_scale = clamp_palette_scale(editor_state.palette_tile_scale);

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

            // Use the user-controlled palette tile scale as display tile size
            let display_tile_size = editor_state.palette_tile_scale;

            // Read drag state from egui memory
            let drag_id = egui::Id::new("palette_drag_state");
            let mut drag_state = ui
                .memory(|mem| mem.data.get_temp::<PaletteDragState>(drag_id))
                .unwrap_or_default();

            egui::ScrollArea::both()
                .id_salt("tile_grid_scroll")
                .max_height(300.0) // Fixed height so the grid doesn't push animation editor out of view
                .show(ui, |ui| {
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
                        if let (Some((sc, sr)), Some((hc, hr))) = (drag_state.start, current_hover)
                        {
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
                            let (end_col, end_row) =
                                current_hover.unwrap_or((start_col, start_row));

                            let min_col = start_col.min(end_col);
                            let max_col = start_col.max(end_col);
                            let min_row = start_row.min(end_row);
                            let max_row = start_row.max(end_row);
                            let w = max_col - min_col + 1;
                            let h = max_row - min_row + 1;

                            if anim_state.active {
                                // In animation editor mode, add clicked tile as a frame
                                if w == 1 && h == 1 {
                                    anim_state.frames.push(AnimationFrame {
                                        col: min_col,
                                        row: min_row,
                                    });
                                    anim_state.error_message = None;
                                }
                            } else if w == 1 && h == 1 {
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
                        if anim_state.active {
                            // In animation editor mode, add clicked tile as a frame
                            anim_state.frames.push(AnimationFrame { col, row });
                            anim_state.error_message = None;
                        } else {
                            // Simple click with no drag — select single tile
                            editor_state.active_brush = Some(TileRef {
                                tileset_id: active_id.clone(),
                                col,
                                row,
                            });
                            editor_state.stamp_brush = None;
                        }
                        drag_state.start = None;
                    }
                });

            // Persist drag state back to egui memory
            ui.memory_mut(|mem| mem.data.insert_temp(drag_id, drag_state));

            // ── Animation Editor section ──
            ui.add_space(8.0);
            ui.separator();

            let toggle_label = if anim_state.active {
                "⏹ Animation Editor (Active)"
            } else {
                "▶ Animation Editor"
            };
            if ui.button(toggle_label).clicked() {
                anim_state.active = !anim_state.active;
                if !anim_state.active {
                    // Reset state when toggling off
                    anim_state.frames.clear();
                    anim_state.frame_duration_ms = 200;
                    anim_state.error_message = None;
                }
            }

            if anim_state.active {
                ui.add_space(4.0);
                ui.label("Click tiles above to add frames.");
                ui.add_space(4.0);

                // Frame duration input
                ui.horizontal(|ui| {
                    ui.label("Frame duration (ms):");
                    let mut duration = anim_state.frame_duration_ms as f64;
                    let drag_value = egui::DragValue::new(&mut duration)
                        .range(1.0..=10000.0)
                        .speed(1.0);
                    if ui.add(drag_value).changed() {
                        anim_state.frame_duration_ms = duration as u32;
                        anim_state.error_message = None;
                    }
                });

                ui.add_space(4.0);
                ui.label(format!("Frames: {}", anim_state.frames.len()));

                // Animation sequence list
                let mut remove_idx: Option<usize> = None;
                let mut swap_pair: Option<(usize, usize)> = None;

                egui::ScrollArea::vertical()
                    .id_salt("anim_frames_scroll")
                    .max_height(150.0)
                    .show(ui, |ui| {
                        for (i, frame) in anim_state.frames.iter().enumerate() {
                            ui.horizontal(|ui| {
                                // Frame number and coordinates
                                ui.label(format!("{}. ({}, {})", i + 1, frame.col, frame.row));

                                // Small tile preview
                                let uv_min = egui::pos2(
                                    (frame.col * tile_w) as f32 / img_w,
                                    (frame.row * tile_h) as f32 / img_h,
                                );
                                let uv_max = egui::pos2(
                                    ((frame.col + 1) * tile_w) as f32 / img_w,
                                    ((frame.row + 1) * tile_h) as f32 / img_h,
                                );
                                let uv = egui::Rect::from_min_max(uv_min, uv_max);
                                let preview = egui::Image::new(egui::load::SizedTexture::new(
                                    egui_texture_id,
                                    [20.0, 20.0],
                                ))
                                .uv(uv);
                                ui.add(preview);

                                // Move up button
                                if ui
                                    .add_enabled(i > 0, egui::Button::new("↑").small())
                                    .clicked()
                                {
                                    swap_pair = Some((i, i - 1));
                                }

                                // Move down button
                                let last_idx = anim_state.frames.len().saturating_sub(1);
                                if ui
                                    .add_enabled(i < last_idx, egui::Button::new("↓").small())
                                    .clicked()
                                {
                                    swap_pair = Some((i, i + 1));
                                }

                                // Remove button
                                if ui.button("×").clicked() {
                                    remove_idx = Some(i);
                                }
                            });
                        }
                    });

                // Apply reorder/remove after iteration
                if let Some((a, b)) = swap_pair {
                    anim_state.frames.swap(a, b);
                    anim_state.error_message = None;
                }
                if let Some(idx) = remove_idx {
                    anim_state.frames.remove(idx);
                    anim_state.error_message = None;
                }

                // Live animation preview
                if anim_state.frames.len() >= 2 {
                    ui.add_space(4.0);
                    ui.separator();
                    ui.label("Preview:");

                    let elapsed_secs = ui.ctx().input(|i| i.time);
                    let elapsed_ms = (elapsed_secs * 1000.0) as u64;
                    let frame_idx = compute_animation_frame_index(
                        elapsed_ms,
                        anim_state.frame_duration_ms,
                        anim_state.frames.len(),
                    );
                    let preview_frame = &anim_state.frames[frame_idx];

                    let uv_min = egui::pos2(
                        (preview_frame.col * tile_w) as f32 / img_w,
                        (preview_frame.row * tile_h) as f32 / img_h,
                    );
                    let uv_max = egui::pos2(
                        ((preview_frame.col + 1) * tile_w) as f32 / img_w,
                        ((preview_frame.row + 1) * tile_h) as f32 / img_h,
                    );
                    let uv = egui::Rect::from_min_max(uv_min, uv_max);
                    let preview_image = egui::Image::new(egui::load::SizedTexture::new(
                        egui_texture_id,
                        [48.0, 48.0],
                    ))
                    .uv(uv);
                    ui.add(preview_image);

                    // Request repaint so the animation keeps updating
                    ui.ctx().request_repaint();
                }

                ui.add_space(4.0);

                // Confirm and Cancel buttons
                ui.horizontal(|ui| {
                    if ui.button("Confirm").clicked() {
                        let animation = TileAnimation {
                            frames: anim_state.frames.clone(),
                            frame_duration_ms: anim_state.frame_duration_ms,
                        };
                        match validate_tile_animation(&animation, columns, rows) {
                            Ok(()) => {
                                if let Some(tileset_entry) = project.tilesets.get_mut(&active_id) {
                                    tileset_entry.meta.animations.push(animation);
                                }
                                // Reset animation editor state
                                anim_state.frames.clear();
                                anim_state.frame_duration_ms = 200;
                                anim_state.active = false;
                                anim_state.error_message = None;
                            }
                            Err(e) => {
                                anim_state.error_message = Some(e.to_string());
                            }
                        }
                    }
                    if ui.button("Cancel").clicked() {
                        anim_state.frames.clear();
                        anim_state.frame_duration_ms = 200;
                        anim_state.active = false;
                        anim_state.error_message = None;
                    }
                });

                // Show inline error message if validation failed
                if let Some(ref err_msg) = anim_state.error_message {
                    ui.colored_label(egui::Color32::RED, err_msg);
                }
            }

            // ── Spritesheet section (below tile grid) ──
            ui.add_space(8.0);
            spritesheet_section_ui(ui, &mut project, &mut error_dialog, &ss_egui_textures);
        });

    // Process spritesheet remove requests and confirmation dialog (outside the panel)
    process_spritesheet_remove(ctx, &mut project, &mut remove_dialog);

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
