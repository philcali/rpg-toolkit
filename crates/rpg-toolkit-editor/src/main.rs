mod algorithms;
mod data;
mod plugins;
mod systems;

use bevy::asset::UnapprovedPathMode;
use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use plugins::{
    AppShellPlugin, AttributePlugin, CanvasPlugin, LayerPanelPlugin, PaintingPlugin,
    SerializationPlugin, SpritesheetPlugin, TilePalettePlugin, ToolbarPlugin, UndoRedoPlugin,
};

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "RPG Toolkit".into(),
                        resize_constraints: WindowResizeConstraints {
                            min_width: 800.0,
                            min_height: 600.0,
                            ..default()
                        },
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    unapproved_path_mode: UnapprovedPathMode::Allow,
                    ..default()
                }),
        )
        .add_plugins(EguiPlugin::default())
        .add_plugins(AppShellPlugin)
        .add_plugins(CanvasPlugin)
        .add_plugins(TilePalettePlugin)
        .add_plugins(LayerPanelPlugin)
        .add_plugins(PaintingPlugin)
        .add_plugins(UndoRedoPlugin)
        .add_plugins(SerializationPlugin)
        .add_plugins(ToolbarPlugin)
        .add_plugins(AttributePlugin)
        .add_plugins(SpritesheetPlugin)
        .init_resource::<data::Project>()
        .init_resource::<systems::input::CursorWorldState>()
        .add_systems(
            Update,
            (
                systems::input::update_cursor_state,
                systems::render::sync_tile_sprites,
            ),
        )
        .run();
}
