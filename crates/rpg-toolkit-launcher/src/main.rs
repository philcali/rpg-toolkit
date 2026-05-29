use bevy::asset::UnapprovedPathMode;
use bevy::prelude::*;
use rpg_toolkit_common::ProjectFile;
use rpg_toolkit_renderer::{
    DialogTextRegistry, PixelScaleConfig, PixelScaleMode, ProjectRendererPlugin,
    RendererProjectData,
};
use std::collections::HashMap;
use std::path::Path;

/// Temporary resource to pass data from main() into the Bevy startup system.
#[derive(Resource)]
struct PendingProjectLoad {
    project_file: ProjectFile,
    /// Maps tileset ID → resolved absolute path to the image file.
    tileset_paths: HashMap<String, std::path::PathBuf>,
    /// Guard to keep temp directory alive for the app lifetime.
    _temp_dir: Option<tempfile::TempDir>,
}

fn parse_scale_arg(args: &[String]) -> Option<PixelScaleMode> {
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--scale" {
            let value = args.get(i + 1).unwrap_or_else(|| {
                eprintln!("Error: --scale requires a value (integer or 'fit')");
                std::process::exit(1);
            });
            return Some(match value.as_str() {
                "fit" | "auto" => PixelScaleMode::ZoomToFit,
                other => {
                    let n: u32 = other.parse().unwrap_or_else(|_| {
                        eprintln!(
                            "Error: --scale value must be a positive integer or 'fit', got '{}'",
                            other
                        );
                        std::process::exit(1);
                    });
                    if n == 0 {
                        eprintln!("Error: --scale value must be at least 1");
                        std::process::exit(1);
                    }
                    PixelScaleMode::Fixed(n)
                }
            });
        }
        i += 1;
    }
    None
}

/// Detect project format from a path.
enum ProjectSource {
    Directory(std::path::PathBuf),
    Zip(std::path::PathBuf),
    LegacyJson(std::path::PathBuf),
}

fn detect_project_source(path: &Path) -> Result<ProjectSource, String> {
    if path.is_dir() {
        if !path.join("manifest.json").exists() {
            return Err(format!(
                "directory '{}' does not contain manifest.json",
                path.display()
            ));
        }
        Ok(ProjectSource::Directory(path.to_path_buf()))
    } else if path.extension().is_some_and(|e| e == "rpg") {
        Ok(ProjectSource::Zip(path.to_path_buf()))
    } else if path.extension().is_some_and(|e| e == "json") {
        Ok(ProjectSource::LegacyJson(path.to_path_buf()))
    } else {
        Err(format!(
            "unsupported project format: {}. Expected .rpg or .json",
            path.display()
        ))
    }
}

/// Load a project from a directory-based format.
fn load_from_dir(
    path: &Path,
) -> Result<(ProjectFile, HashMap<String, std::path::PathBuf>), String> {
    let project_file = ProjectFile::deserialize_from_dir(path)
        .map_err(|e| format!("failed to load project: {}", e))?;
    let project_dir = path
        .canonicalize()
        .map_err(|e| format!("could not canonicalize path: {}", e))?;
    let tileset_paths: HashMap<String, std::path::PathBuf> = project_file
        .tilesets
        .iter()
        .map(|(id, meta)| (id.clone(), project_dir.join(&meta.file_path)))
        .collect();
    Ok((project_file, tileset_paths))
}

/// Load a project from a ZIP archive, extracting to a temp directory.
fn load_from_zip(
    path: &Path,
) -> Result<
    (
        ProjectFile,
        HashMap<String, std::path::PathBuf>,
        tempfile::TempDir,
    ),
    String,
> {
    let zip_data = std::fs::read(path).map_err(|e| format!("could not read zip file: {}", e))?;
    let temp_dir =
        tempfile::tempdir().map_err(|e| format!("could not create temp directory: {}", e))?;
    let project_file = ProjectFile::deserialize_from_zip(&zip_data, temp_dir.path())
        .map_err(|e| format!("failed to load project from zip: {}", e))?;
    let tileset_paths: HashMap<String, std::path::PathBuf> = project_file
        .tilesets
        .iter()
        .map(|(id, meta)| (id.clone(), temp_dir.path().join(&meta.file_path)))
        .collect();
    Ok((project_file, tileset_paths, temp_dir))
}

/// Load a project from a legacy single-file JSON format.
fn load_from_legacy_json(
    path: &Path,
) -> Result<(ProjectFile, HashMap<String, std::path::PathBuf>), String> {
    let contents =
        std::fs::read_to_string(path).map_err(|e| format!("could not read file: {}", e))?;
    let project_file = ProjectFile::deserialize(&contents)
        .map_err(|e| format!("failed to deserialize project: {}", e))?;
    let project_dir = path
        .parent()
        .map(|p| p.canonicalize().unwrap_or_else(|_| p.to_path_buf()))
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    let tileset_paths: HashMap<String, std::path::PathBuf> = project_file
        .tilesets
        .iter()
        .map(|(id, meta)| (id.clone(), project_dir.join(&meta.file_path)))
        .collect();
    Ok((project_file, tileset_paths))
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let project_path_str = {
        let mut path = None;
        let mut i = 1;
        while i < args.len() {
            if args[i] == "--scale" {
                i += 2;
                continue;
            }
            if args[i].starts_with("--") {
                i += 1;
                continue;
            }
            path = Some(args[i].clone());
            break;
        }
        path.unwrap_or_else(|| {
            eprintln!("Usage: rpg-toolkit-launcher <path-to-project.rpg> [--scale <N|fit>]");
            std::process::exit(1);
        })
    };

    let scale_mode = parse_scale_arg(&args);
    let project_path = Path::new(&project_path_str);

    let source = detect_project_source(project_path).unwrap_or_else(|e| {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    });

    let (project_file, tileset_paths, temp_dir) = match source {
        ProjectSource::Directory(dir) => {
            let (pf, tp) = load_from_dir(&dir).unwrap_or_else(|e| {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            });
            (pf, tp, None)
        }
        ProjectSource::Zip(zip_path) => {
            let (pf, tp, td) = load_from_zip(&zip_path).unwrap_or_else(|e| {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            });
            (pf, tp, Some(td))
        }
        ProjectSource::LegacyJson(json_path) => {
            let (pf, tp) = load_from_legacy_json(&json_path).unwrap_or_else(|e| {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            });
            (pf, tp, None)
        }
    };

    if project_file.spawn_point.is_none() {
        eprintln!("Error: project has no spawn point defined");
        std::process::exit(1);
    }

    let mut app = App::new();
    app.add_plugins(
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
            })
            .set(ImagePlugin::default_nearest()),
    );

    if let Some(mode) = scale_mode {
        let effective = match &mode {
            PixelScaleMode::Fixed(n) => *n,
            PixelScaleMode::ZoomToFit => 1,
        };
        app.insert_resource(PixelScaleConfig {
            mode,
            effective_scale: effective,
        });
    }

    app.insert_resource(PendingProjectLoad {
        project_file,
        tileset_paths,
        _temp_dir: temp_dir,
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

        let texture: Handle<Image> = asset_server.load(image_path.to_string_lossy().to_string());
        tileset_textures.insert(tileset_id.clone(), texture);

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
        spritesheet_textures: HashMap::new(),
        spritesheet_atlas_layouts: HashMap::new(),
    });

    commands.insert_resource(DialogTextRegistry::from_map(
        pending.project_file.dialog_texts.clone(),
    ));

    commands.remove_resource::<PendingProjectLoad>();
}
