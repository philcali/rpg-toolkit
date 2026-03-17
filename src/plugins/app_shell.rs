use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};

/// Plugin that provides the application shell: menu bar, canvas area, and side panel.
pub struct AppShellPlugin;

impl Plugin for AppShellPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(EguiPrimaryContextPass, app_shell_ui);
    }
}

fn app_shell_ui(mut contexts: EguiContexts) -> Result {
    let ctx = contexts.ctx_mut()?;

    // Menu bar
    egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("New Map").clicked() {
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
        ui.heading("Canvas");
        ui.separator();
        ui.label("No map loaded. Use File > New Map to create one.");
    });

    Ok(())
}
