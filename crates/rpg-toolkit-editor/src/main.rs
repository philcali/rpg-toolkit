mod algorithms;
mod data;
mod plugins;
mod systems;

use bevy::asset::UnapprovedPathMode;
use bevy::prelude::*;
use bevy_egui::{EguiPlugin, EguiPrimaryContextPass};
use plugins::{
    AbilityPanelPlugin, AppShellPlugin, AttributePlugin, CanvasPlugin, CharacterPanelPlugin,
    DialogTextPanelPlugin, EnemyPanelPlugin, ItemPanelPlugin, LayerPanelPlugin, PaintingPlugin,
    SerializationPlugin, ShopPanelPlugin, SpritesheetPlugin, TextIdIndex, TilePalettePlugin,
    ToolbarPlugin, UndoRedoPlugin,
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
        // Configure UI system set ordering: Shell → Panels → Overlay
        .configure_sets(
            EguiPrimaryContextPass,
            (
                data::EditorUiSet::Shell,
                data::EditorUiSet::Panels,
                data::EditorUiSet::Overlay,
            )
                .chain(),
        )
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
        .add_plugins(DialogTextPanelPlugin)
        .add_plugins(CharacterPanelPlugin)
        .add_plugins(ItemPanelPlugin)
        .add_plugins(AbilityPanelPlugin)
        .add_plugins(EnemyPanelPlugin)
        .add_plugins(ShopPanelPlugin)
        .init_resource::<data::Project>()
        .init_resource::<TextIdIndex>()
        .init_resource::<systems::input::CursorWorldState>()
        .init_resource::<systems::render::EditorAnimationTick>()
        .add_systems(
            Update,
            (
                systems::input::update_cursor_state,
                systems::render::tick_editor_animation,
                systems::render::sync_tile_sprites,
                systems::render::animate_editor_tiles,
            )
                .chain(),
        )
        .run();
}
