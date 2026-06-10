use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

use crate::data::project::Project;
use crate::data::{AnyDialogOpen, AppEditorMode, EditorState, EditorUiSet, TilesetMeta};
use crate::plugins::attribute::{
    ElevationDialog, ElevationTransitionDialog, EventTriggerDialog, NpcPlacementDialog,
    SpawnPointConfirmDialog,
};
use crate::plugins::serialization::{SerializationAction, SerializationRequest};
use crate::plugins::spritesheet::RemoveSpritesheetDialog;

/// Plugin that provides the application shell: menu bar, canvas area, and side panel.
pub struct AppShellPlugin;

impl Plugin for AppShellPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NewMapDialog>()
            .init_resource::<ErrorDialog>()
            .init_resource::<LoadTilesetDialog>()
            .init_resource::<UnsavedChangesDialog>()
            .init_resource::<EditorState>()
            .init_resource::<AnyDialogOpen>()
            .init_resource::<AppEditorMode>()
            .add_systems(
                EguiPrimaryContextPass,
                (app_shell_ui, update_any_dialog_open)
                    .chain()
                    .in_set(EditorUiSet::Shell),
            );
    }
}

/// State for the "New Map" dialog.
#[derive(Resource)]
pub struct NewMapDialog {
    pub open: bool,
    pub name: String,
    pub width: String,
    pub height: String,
    pub tile_width: u32,
    pub tile_height: u32,
}

impl Default for NewMapDialog {
    fn default() -> Self {
        Self {
            open: false,
            name: "Untitled".to_string(),
            width: "32".to_string(),
            height: "32".to_string(),
            tile_width: 16,
            tile_height: 16,
        }
    }
}

/// State for a simple error dialog.
#[derive(Resource, Default)]
pub struct ErrorDialog {
    pub open: bool,
    pub message: String,
}

/// The pending action that triggered the unsaved changes prompt.
#[derive(Clone, Debug)]
pub enum PendingAction {
    OpenProject,
    NewProject,
}

/// State for the unsaved changes confirmation dialog.
#[derive(Resource, Default)]
pub struct UnsavedChangesDialog {
    pub open: bool,
    pub pending_action: Option<PendingAction>,
}

/// Valid tile sizes for the tile size picker.
const TILE_SIZE_OPTIONS: [u32; 4] = [8, 16, 32, 64];

/// Action to perform on a map tab (deferred to avoid borrow conflicts).
enum MapTabAction {
    Select(usize),
    Close(usize),
}

/// State for the "Load Tileset" dialog (tile size picker).
#[derive(Resource)]
pub struct LoadTilesetDialog {
    pub open: bool,
    pub selected_tile_size: u32,
}

impl Default for LoadTilesetDialog {
    fn default() -> Self {
        Self {
            open: false,
            selected_tile_size: 16,
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn app_shell_ui(
    mut contexts: EguiContexts,
    mut new_map_dialog: ResMut<NewMapDialog>,
    mut error_dialog: ResMut<ErrorDialog>,
    mut load_tileset_dialog: ResMut<LoadTilesetDialog>,
    mut unsaved_dialog: ResMut<UnsavedChangesDialog>,
    mut serialization_action: ResMut<SerializationAction>,
    _commands: Commands,
    mut editor_state: ResMut<EditorState>,
    asset_server: Res<AssetServer>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut project: ResMut<Project>,
    mut app_editor_mode: ResMut<AppEditorMode>,
) -> Result {
    let ctx = contexts.ctx_mut()?;

    // Menu bar
    egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("New Project").clicked() {
                    let has_unsaved = project.has_unsaved_changes.values().any(|&v| v);
                    if has_unsaved {
                        unsaved_dialog.open = true;
                        unsaved_dialog.pending_action = Some(PendingAction::NewProject);
                    } else {
                        serialization_action.pending = Some(SerializationRequest::NewProject);
                    }
                    ui.close();
                }
                ui.separator();
                if ui.button("New Map").clicked() {
                    new_map_dialog.open = true;
                    new_map_dialog.name = "Untitled".to_string();
                    new_map_dialog.width = "32".to_string();
                    new_map_dialog.height = "32".to_string();
                    new_map_dialog.tile_width = 16;
                    new_map_dialog.tile_height = 16;
                    ui.close();
                }
                if ui.button("Load Tileset").clicked() {
                    load_tileset_dialog.open = true;
                    load_tileset_dialog.selected_tile_size = 16;
                    ui.close();
                }
                ui.separator();
                if ui.button("Save Project").clicked() {
                    serialization_action.pending = Some(SerializationRequest::Save);
                    ui.close();
                }
                if ui.button("Save As").clicked() {
                    serialization_action.pending = Some(SerializationRequest::SaveAs);
                    ui.close();
                }
                if ui.button("Open Project").clicked() {
                    let has_unsaved = project.has_unsaved_changes.values().any(|&v| v);
                    if has_unsaved {
                        unsaved_dialog.open = true;
                        unsaved_dialog.pending_action = Some(PendingAction::OpenProject);
                    } else {
                        serialization_action.pending = Some(SerializationRequest::Open);
                    }
                    ui.close();
                }
            });
            ui.menu_button("Edit", |ui| {
                if ui.button("Undo").clicked() {
                    ui.close();
                }
                if ui.button("Redo").clicked() {
                    ui.close();
                }
            });
            ui.menu_button("Mode", |ui| {
                if ui
                    .selectable_label(*app_editor_mode == AppEditorMode::Map, "🗺 Map Editor")
                    .clicked()
                {
                    *app_editor_mode = AppEditorMode::Map;
                    ui.close();
                }
                if ui
                    .selectable_label(
                        *app_editor_mode == AppEditorMode::Character,
                        "👤 Character Editor",
                    )
                    .clicked()
                {
                    *app_editor_mode = AppEditorMode::Character;
                    ui.close();
                }
                if ui
                    .selectable_label(*app_editor_mode == AppEditorMode::Item, "⚔ Item Editor")
                    .clicked()
                {
                    *app_editor_mode = AppEditorMode::Item;
                    ui.close();
                }
            });
        });
    });

    // New Map dialog
    if new_map_dialog.open {
        let mut still_open = true;
        let mut should_create = false;

        egui::Window::new("New Map")
            .collapsible(false)
            .resizable(false)
            .open(&mut still_open)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Name:");
                    ui.text_edit_singleline(&mut new_map_dialog.name);
                });
                ui.horizontal(|ui| {
                    ui.label("Width:");
                    ui.text_edit_singleline(&mut new_map_dialog.width);
                });
                ui.horizontal(|ui| {
                    ui.label("Height:");
                    ui.text_edit_singleline(&mut new_map_dialog.height);
                });
                ui.label("Dimensions must be between 1 and 256.");
                ui.separator();
                ui.horizontal(|ui| {
                    ui.label("Tile Width:");
                    egui::ComboBox::from_id_salt("new_map_tile_width")
                        .selected_text(format!("{}", new_map_dialog.tile_width))
                        .show_ui(ui, |ui| {
                            for &size in &TILE_SIZE_OPTIONS {
                                ui.selectable_value(
                                    &mut new_map_dialog.tile_width,
                                    size,
                                    format!("{}", size),
                                );
                            }
                        });
                });
                ui.horizontal(|ui| {
                    ui.label("Tile Height:");
                    egui::ComboBox::from_id_salt("new_map_tile_height")
                        .selected_text(format!("{}", new_map_dialog.tile_height))
                        .show_ui(ui, |ui| {
                            for &size in &TILE_SIZE_OPTIONS {
                                ui.selectable_value(
                                    &mut new_map_dialog.tile_height,
                                    size,
                                    format!("{}", size),
                                );
                            }
                        });
                });
                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("Create").clicked() {
                        should_create = true;
                    }
                    if ui.button("Cancel").clicked() {
                        new_map_dialog.open = false;
                    }
                });
            });

        if !still_open {
            new_map_dialog.open = false;
        }

        if should_create {
            let name = new_map_dialog.name.trim().to_string();
            let w = new_map_dialog.width.trim().parse::<u32>();
            let h = new_map_dialog.height.trim().parse::<u32>();
            let tile_w = new_map_dialog.tile_width;
            let tile_h = new_map_dialog.tile_height;

            match (w, h) {
                (Ok(w), Ok(h)) => match project.add_map(name, w, h, tile_w, tile_h) {
                    Ok(_map_id) => {
                        new_map_dialog.open = false;
                    }
                    Err(e) => {
                        error_dialog.open = true;
                        error_dialog.message = e.to_string();
                    }
                },
                _ => {
                    error_dialog.open = true;
                    error_dialog.message =
                        "Invalid input: width and height must be integers between 1 and 256."
                            .to_string();
                }
            }
        }
    }

    // Error dialog
    if error_dialog.open {
        let mut still_open = true;
        egui::Window::new("Error")
            .collapsible(false)
            .resizable(false)
            .open(&mut still_open)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.label(&error_dialog.message);
                ui.separator();
                if ui.button("OK").clicked() {
                    error_dialog.open = false;
                }
            });
        if !still_open {
            error_dialog.open = false;
        }
    }

    // Unsaved changes dialog
    if unsaved_dialog.open {
        let mut still_open = true;
        let mut choice: Option<&str> = None;

        egui::Window::new("Unsaved Changes")
            .collapsible(false)
            .resizable(false)
            .open(&mut still_open)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.label("You have unsaved changes. What would you like to do?");
                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("Save").clicked() {
                        choice = Some("save");
                    }
                    if ui.button("Discard").clicked() {
                        choice = Some("discard");
                    }
                    if ui.button("Cancel").clicked() {
                        choice = Some("cancel");
                    }
                });
            });

        if !still_open {
            unsaved_dialog.open = false;
            unsaved_dialog.pending_action = None;
        }

        match choice {
            Some("save") => {
                // Trigger save, then proceed with the pending action
                serialization_action.pending = Some(SerializationRequest::Save);
                let pending = unsaved_dialog.pending_action.take();
                unsaved_dialog.open = false;
                match pending {
                    Some(PendingAction::OpenProject) => {
                        serialization_action.pending = Some(SerializationRequest::Open);
                    }
                    Some(PendingAction::NewProject) => {
                        serialization_action.pending = Some(SerializationRequest::NewProject);
                    }
                    None => {}
                }
            }
            Some("discard") => {
                let pending = unsaved_dialog.pending_action.take();
                unsaved_dialog.open = false;
                match pending {
                    Some(PendingAction::OpenProject) => {
                        serialization_action.pending = Some(SerializationRequest::Open);
                    }
                    Some(PendingAction::NewProject) => {
                        serialization_action.pending = Some(SerializationRequest::NewProject);
                    }
                    None => {}
                }
            }
            Some("cancel") => {
                unsaved_dialog.open = false;
                unsaved_dialog.pending_action = None;
            }
            _ => {}
        }
    }

    // Load Tileset dialog (tile size picker → file dialog → load)
    if load_tileset_dialog.open {
        let mut still_open = true;
        let mut should_load = false;

        egui::Window::new("Load Tileset")
            .collapsible(false)
            .resizable(false)
            .open(&mut still_open)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.label("Select tile size before loading:");
                ui.horizontal(|ui| {
                    ui.label("Tile Size:");
                    egui::ComboBox::from_id_salt("tile_size_combo")
                        .selected_text(format!(
                            "{}x{}",
                            load_tileset_dialog.selected_tile_size,
                            load_tileset_dialog.selected_tile_size
                        ))
                        .show_ui(ui, |ui| {
                            for &size in &TILE_SIZE_OPTIONS {
                                ui.selectable_value(
                                    &mut load_tileset_dialog.selected_tile_size,
                                    size,
                                    format!("{}x{}", size, size),
                                );
                            }
                        });
                });
                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("Choose File…").clicked() {
                        should_load = true;
                    }
                    if ui.button("Cancel").clicked() {
                        load_tileset_dialog.open = false;
                    }
                });
            });

        if !still_open {
            load_tileset_dialog.open = false;
        }

        if should_load {
            load_tileset_dialog.open = false;
            let tile_size = load_tileset_dialog.selected_tile_size;

            // Open native file dialog (blocking — acceptable for desktop app)
            let file = rfd::FileDialog::new()
                .add_filter("Images", &["png", "jpg", "jpeg"])
                .pick_file();

            if let Some(path) = file {
                // Validate extension
                let ext = path
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("")
                    .to_lowercase();

                if !["png", "jpg", "jpeg"].contains(&ext.as_str()) {
                    error_dialog.open = true;
                    error_dialog.message =
                        "Unsupported image format. Supported: PNG, JPEG".to_string();
                } else {
                    // Load image with the `image` crate to get dimensions
                    match image::open(&path) {
                        Ok(img) => {
                            let (img_w, img_h) = (img.width(), img.height());

                            match TilesetMeta::from_image_dimensions(
                                img_w, img_h, tile_size, tile_size,
                            ) {
                                Ok(mut meta) => {
                                    meta.file_path = path.to_string_lossy().to_string();

                                    // Create TextureAtlasLayout
                                    let layout = TextureAtlasLayout::from_grid(
                                        UVec2::new(tile_size, tile_size),
                                        meta.columns,
                                        meta.rows,
                                        None,
                                        None,
                                    );
                                    let atlas_handle = atlas_layouts.add(layout);

                                    // Load texture via Bevy asset server
                                    let texture_handle: Handle<Image> =
                                        asset_server.load(path.to_string_lossy().to_string());

                                    // Add tileset to Project and auto-switch tab
                                    {
                                        let new_id =
                                            project.add_tileset(meta, texture_handle, atlas_handle);
                                        editor_state.active_tileset_tab = Some(new_id);
                                    }
                                }
                                Err(e) => {
                                    error_dialog.open = true;
                                    error_dialog.message = e.to_string();
                                }
                            }
                        }
                        Err(e) => {
                            error_dialog.open = true;
                            error_dialog.message = format!("Failed to read image: {}", e);
                        }
                    }
                }
            }
        }
    }

    // Map Tab Bar — horizontal tab strip above the canvas (only in Map mode)
    if *app_editor_mode == AppEditorMode::Map && !project.open_tabs.is_empty() {
        let mut tab_action: Option<MapTabAction> = None;

        egui::TopBottomPanel::top("map_tab_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                for (idx, map_id) in project.open_tabs.iter().enumerate() {
                    let is_active = project.active_tab == Some(idx);
                    let map_name = project
                        .maps
                        .get(map_id)
                        .map(|m| m.name.as_str())
                        .unwrap_or("???");
                    let has_unsaved = project
                        .has_unsaved_changes
                        .get(map_id)
                        .copied()
                        .unwrap_or(false);

                    let label = if has_unsaved {
                        format!("● {}", map_name)
                    } else {
                        map_name.to_string()
                    };

                    let tab = ui.selectable_label(is_active, &label);
                    if tab.clicked() {
                        tab_action = Some(MapTabAction::Select(idx));
                    }

                    // Close button
                    if ui.small_button("×").clicked() {
                        tab_action = Some(MapTabAction::Close(idx));
                    }

                    ui.separator();
                }
            });
        });

        if let Some(action) = tab_action {
            match action {
                MapTabAction::Select(idx) => project.set_active_tab(idx),
                MapTabAction::Close(idx) => project.close_map_tab(idx),
            }
        }
    }

    // Update canvas rect for toolbar positioning (available_rect reflects
    // the area remaining after side panels and top bars have been laid out).
    // NOTE: At this point only the top panel(s) have been rendered in this system.
    // Side panels render in separate systems. The canvas_rect is refined by
    // consumers (toolbar) who read available_rect after all panels are drawn.

    // Central panel — only rendered in Map mode.
    // In CharacterEditor mode, the CharacterPanelPlugin owns the central panel.
    if *app_editor_mode == AppEditorMode::Map {
        let has_active_map = project.active_map().is_some();
        if has_active_map {
            let frame = egui::Frame::new()
                .fill(egui::Color32::TRANSPARENT)
                .inner_margin(0.0);
            egui::CentralPanel::default().frame(frame).show(ctx, |_ui| {
                // Bevy renders the grid and tiles behind this transparent panel.
            });
        } else {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.centered_and_justified(|ui| {
                    ui.label("No map open");
                });
            });
        }
    }

    Ok(())
}

/// Aggregates all dialog-open flags into a single `AnyDialogOpen` resource.
///
/// Canvas interaction systems check this instead of `ctx.wants_pointer_input()`
/// so that side panels and toolbar remain clickable while dialogs still block
/// canvas input.
#[allow(clippy::too_many_arguments)]
fn update_any_dialog_open(
    new_map: Res<NewMapDialog>,
    error: Res<ErrorDialog>,
    load_tileset: Res<LoadTilesetDialog>,
    unsaved: Res<UnsavedChangesDialog>,
    event_trigger: Res<EventTriggerDialog>,
    spawn_confirm: Res<SpawnPointConfirmDialog>,
    npc_placement: Res<NpcPlacementDialog>,
    elevation: Res<ElevationDialog>,
    elevation_transition: Res<ElevationTransitionDialog>,
    remove_spritesheet: Res<RemoveSpritesheetDialog>,
    map_delete: Res<crate::plugins::layer_panel::MapDeleteDialogOpen>,
    mut any_open: ResMut<AnyDialogOpen>,
) {
    any_open.0 = new_map.open
        || error.open
        || load_tileset.open
        || unsaved.open
        || event_trigger.open
        || spawn_confirm.open
        || npc_placement.open
        || elevation.open
        || elevation_transition.open
        || remove_spritesheet.open
        || map_delete.0;
}
