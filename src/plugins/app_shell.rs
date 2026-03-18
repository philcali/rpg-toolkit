use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};

use crate::data::{EditorState, MapData, TilesetData, TilesetMeta};
use crate::plugins::serialization::{SerializationAction, SerializationRequest};

/// Plugin that provides the application shell: menu bar, canvas area, and side panel.
pub struct AppShellPlugin;

impl Plugin for AppShellPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NewMapDialog>()
            .init_resource::<ErrorDialog>()
            .init_resource::<LoadTilesetDialog>()
            .init_resource::<UnsavedChangesDialog>()
            .init_resource::<EditorState>()
            .add_systems(EguiPrimaryContextPass, app_shell_ui);
    }
}

/// State for the "New Map" dialog.
#[derive(Resource)]
pub struct NewMapDialog {
    pub open: bool,
    pub name: String,
    pub width: String,
    pub height: String,
}

impl Default for NewMapDialog {
    fn default() -> Self {
        Self {
            open: false,
            name: "Untitled".to_string(),
            width: "32".to_string(),
            height: "32".to_string(),
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
    NewMap,
}

/// State for the unsaved changes confirmation dialog.
#[derive(Resource, Default)]
pub struct UnsavedChangesDialog {
    pub open: bool,
    pub pending_action: Option<PendingAction>,
}

/// Valid tile sizes for the tile size picker.
const TILE_SIZE_OPTIONS: [u32; 4] = [8, 16, 32, 64];

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

fn app_shell_ui(
    mut contexts: EguiContexts,
    mut new_map_dialog: ResMut<NewMapDialog>,
    mut error_dialog: ResMut<ErrorDialog>,
    mut load_tileset_dialog: ResMut<LoadTilesetDialog>,
    mut unsaved_dialog: ResMut<UnsavedChangesDialog>,
    mut serialization_action: ResMut<SerializationAction>,
    mut commands: Commands,
    existing_map: Option<Res<MapData>>,
    editor_state: Res<EditorState>,
    asset_server: Res<AssetServer>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) -> Result {
    let ctx = contexts.ctx_mut()?;

    // Menu bar
    egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("New Map").clicked() {
                    if editor_state.has_unsaved_changes {
                        unsaved_dialog.open = true;
                        unsaved_dialog.pending_action = Some(PendingAction::NewMap);
                    } else {
                        new_map_dialog.open = true;
                        new_map_dialog.name = "Untitled".to_string();
                        new_map_dialog.width = "32".to_string();
                        new_map_dialog.height = "32".to_string();
                    }
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
                if ui.button("Open Project").clicked() {
                    if editor_state.has_unsaved_changes {
                        // For now, just open directly — could add unsaved prompt for Open too
                    }
                    serialization_action.pending = Some(SerializationRequest::Open);
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

            match (w, h) {
                (Ok(w), Ok(h)) => match MapData::new(name, w, h) {
                    Ok(map) => {
                        commands.insert_resource(map);
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
                if let Some(PendingAction::NewMap) = pending {
                    new_map_dialog.open = true;
                    new_map_dialog.name = "Untitled".to_string();
                    new_map_dialog.width = "32".to_string();
                    new_map_dialog.height = "32".to_string();
                }
            }
            Some("discard") => {
                let pending = unsaved_dialog.pending_action.take();
                unsaved_dialog.open = false;
                if let Some(PendingAction::NewMap) = pending {
                    new_map_dialog.open = true;
                    new_map_dialog.name = "Untitled".to_string();
                    new_map_dialog.width = "32".to_string();
                    new_map_dialog.height = "32".to_string();
                }
            }
            Some("cancel") | _ if choice.is_some() => {
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

                                    let tileset_data = TilesetData {
                                        meta,
                                        texture: texture_handle,
                                        atlas_layout: atlas_handle,
                                    };
                                    commands.insert_resource(tileset_data);
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

    // Central panel — transparent when a map is loaded so Bevy's 2D
    // camera (gizmo grid, sprites) shows through.
    if existing_map.is_some() {
        let frame = egui::Frame::new()
            .fill(egui::Color32::TRANSPARENT)
            .inner_margin(0.0);
        egui::CentralPanel::default().frame(frame).show(ctx, |_ui| {
            // Bevy renders the grid and tiles behind this transparent panel.
        });
    } else {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Canvas");
            ui.separator();
            ui.label("No map loaded. Use File > New Map to create one.");
        });
    }

    Ok(())
}
