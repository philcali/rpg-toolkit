use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};

use crate::data::{EditorState, MapData};

/// Plugin that provides the application shell: menu bar, canvas area, and side panel.
pub struct AppShellPlugin;

impl Plugin for AppShellPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NewMapDialog>()
            .init_resource::<ErrorDialog>()
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

fn app_shell_ui(
    mut contexts: EguiContexts,
    mut new_map_dialog: ResMut<NewMapDialog>,
    mut error_dialog: ResMut<ErrorDialog>,
    mut commands: Commands,
    existing_map: Option<Res<MapData>>,
) -> Result {
    let ctx = contexts.ctx_mut()?;

    // Menu bar
    egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("New Map").clicked() {
                    new_map_dialog.open = true;
                    new_map_dialog.name = "Untitled".to_string();
                    new_map_dialog.width = "32".to_string();
                    new_map_dialog.height = "32".to_string();
                    ui.close();
                }
                if ui.button("Load Tileset").clicked() {
                    ui.close();
                }
                ui.separator();
                if ui.button("Save Project").clicked() {
                    ui.close();
                }
                if ui.button("Open Project").clicked() {
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

    // Side panel (tile palette placeholder)
    egui::SidePanel::right("tile_palette")
        .default_width(200.0)
        .show(ctx, |ui| {
            ui.heading("Tile Palette");
            ui.separator();
            ui.label("No tileset loaded.");
        });

    // Central panel (canvas placeholder)
    egui::CentralPanel::default().show(ctx, |ui| {
        if existing_map.is_some() {
            // Canvas will be rendered by CanvasPlugin; leave this empty
        } else {
            ui.heading("Canvas");
            ui.separator();
            ui.label("No map loaded. Use File > New Map to create one.");
        }
    });

    Ok(())
}
