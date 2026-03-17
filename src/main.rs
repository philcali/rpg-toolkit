mod plugins;

use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use plugins::AppShellPlugin;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "RPG Map Editor".into(),
                    resize_constraints: WindowResizeConstraints {
                        min_width: 800.0,
                        min_height: 600.0,
                        ..default()
                    },
                    ..default()
                }),
                ..default()
            }),
        )
        .add_plugins(EguiPlugin::default())
        .add_plugins(AppShellPlugin)
        .add_systems(Startup, setup_camera)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}
