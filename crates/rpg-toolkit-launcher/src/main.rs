use bevy::asset::UnapprovedPathMode;
use bevy::prelude::*;
use rpg_toolkit_common::ProjectFile;
use rpg_toolkit_renderer::{ProjectRendererPlugin, RendererProjectData};
use std::collections::HashMap;
use std::path::Path;

/// Temporary resource to pass data from main() into the Bevy startup system.
#[derive(Resource)]
struct PendingProjectLoad {
    project_file: ProjectFile,
    /// Maps tileset ID → resolved absolute path to the image file.
    tileset_paths: HashMap<String, std::path::PathBuf>,
}

fn main() {
    // Parse CLI argument
    let args: Vec<String> = std::env::args().collect();
    let project_path_str = args.get(1).unwrap_or_else(|| {
        eprintln!("Usage: rpg-toolkit-launcher <path-to-project.rpg>");
        std::process::exit(1);
    });

    // Read file contents
    let project_path = Path::new(project_path_str);
    let contents = std::fs::read_to_string(project_path).unwrap_or_else(|e| {
        eprintln!("Error: could not read '{}': {}", project_path.display(), e);
        std::process::exit(1);
    });

    // Deserialize and validate
    let project_file = ProjectFile::deserialize(&contents).unwrap_or_else(|e| {
        eprintln!("Error: failed to load project: {}", e);
        std::process::exit(1);
    });

    // Validate spawn point exists
    if project_file.spawn_point.is_none() {
        eprintln!("Error: project has no spawn point defined");
        std::process::exit(1);
    }

    // Resolve tileset image paths relative to the project file directory
    let project_dir = project_path
        .parent()
        .unwrap_or(Path::new("."))
        .canonicalize()
        .unwrap_or_else(|_| {
            project_path
                .parent()
                .unwrap_or(Path::new("."))
                .to_path_buf()
        });

    let tileset_paths: HashMap<String, std::path::PathBuf> = project_file
        .tilesets
        .iter()
        .map(|(id, meta)| (id.clone(), project_dir.join(&meta.file_path)))
        .collect();

    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "RPG Toolkit".into(),
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    unapproved_path_mode: UnapprovedPathMode::Allow,
                    ..default()
                }),
        )
        .insert_resource(PendingProjectLoad {
            project_file,
            tileset_paths,
        })
        .add_systems(PreStartup, load_project_resources)
        .add_plugins(ProjectRendererPlugin)
        .run();
}

/// Startup system that loads tileset textures via the Bevy asset server
/// and inserts the `RendererProjectData` resource.
fn load_project_resources(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    pending: Res<PendingProjectLoad>,
) {
    let mut tileset_textures: HashMap<String, Handle<Image>> = HashMap::new();
    let mut tileset_atlas_layouts: HashMap<String, Handle<TextureAtlasLayout>> = HashMap::new();

    for (tileset_id, meta) in &pending.project_file.tilesets {
        let image_path = pending
            .tileset_paths
            .get(tileset_id)
            .expect("tileset path should exist");

        // Load the texture via asset server using the absolute path
        let texture: Handle<Image> = asset_server.load(image_path.to_string_lossy().to_string());
        tileset_textures.insert(tileset_id.clone(), texture);

        // Build the texture atlas layout from tileset metadata
        let layout = TextureAtlasLayout::from_grid(
            UVec2::new(meta.tile_width, meta.tile_height),
            meta.columns,
            meta.rows,
            None,
            None,
        );
        let layout_handle = texture_atlas_layouts.add(layout);
        tileset_atlas_layouts.insert(tileset_id.clone(), layout_handle);
    }

    commands.insert_resource(RendererProjectData {
        project_file: pending.project_file.clone(),
        tileset_textures,
        tileset_atlas_layouts,
    });

    commands.remove_resource::<PendingProjectLoad>();
}
