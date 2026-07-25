use bevy::asset::UnapprovedPathMode;
use bevy::prelude::*;
use rpg_toolkit_common::asset::{AssetManager, ProjectSource};
use rpg_toolkit_common::{AppPhase, ProjectFile};
use rpg_toolkit_renderer::{
    DialogTextRegistry, PixelScaleConfig, PixelScaleMode, ProjectRendererPlugin,
    RendererProjectData, SavePath,
};
use rpg_toolkit_scenes::{
    AbilityRegistryRes, CharacterProgressState, CharacterRegistryRes, CurrencyState, GameState,
    InventoryState, ItemRegistryRes, PartyState, RendererState, ShopRegistryRes, ShopScenePlugin,
    StatusScenePlugin, TitleScreenConfig, TitleScreenPlugin,
};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Temporary resource to pass data from main() into the Bevy startup system.
#[derive(Resource)]
struct PendingProjectLoad {
    project_file: ProjectFile,
    /// Maps tileset ID → resolved absolute path to the image file.
    tileset_paths: HashMap<String, std::path::PathBuf>,
    /// Guard to keep temp directory alive for the app lifetime.
    _temp_dir: Option<tempfile::TempDir>,
}

/// Resource that holds the temp directory alive for the entire app lifetime.
/// Without this, ZIP-extracted files would be deleted before Bevy's async
/// asset loader can read them.
#[derive(Resource)]
struct TempDirGuard {
    _dir: tempfile::TempDir,
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

fn parse_save_arg(args: &[String]) -> Option<String> {
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--save" {
            return args.get(i + 1).cloned();
        }
        i += 1;
    }
    None
}

/// Resolve tileset and spritesheet paths to absolute paths for the renderer.
fn resolve_asset_paths(
    project_file: &mut ProjectFile,
    project_root: &Path,
) -> HashMap<String, PathBuf> {
    let tileset_paths: HashMap<String, PathBuf> = project_file
        .tilesets
        .iter()
        .map(|(id, meta)| (id.clone(), project_root.join(&meta.file_path)))
        .collect();

    // Resolve spritesheet paths to absolute so the renderer can load them
    for ss in project_file.spritesheets.values_mut() {
        if !ss.file_path.is_empty() && !Path::new(&ss.file_path).is_absolute() {
            ss.file_path = project_root
                .join(&ss.file_path)
                .to_string_lossy()
                .to_string();
        }
    }

    tileset_paths
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
            if args[i] == "--save" {
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
            eprintln!("Usage: rpg-toolkit-launcher <path-to-project.rpg> [--scale <N|fit>] [--save <path>]");
            std::process::exit(1);
        })
    };

    let scale_mode = parse_scale_arg(&args);
    let save_path_arg = parse_save_arg(&args);
    let project_path = Path::new(&project_path_str);

    // Reject legacy JSON format before attempting detection
    if project_path.extension().is_some_and(|e| e == "json") {
        eprintln!(
            "Error: legacy JSON format is no longer supported. Please convert your project to directory or .rpg ZIP format."
        );
        std::process::exit(1);
    }

    let source = AssetManager::detect_source(project_path).unwrap_or_else(|e| {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    });

    let (project_file, tileset_paths, temp_dir, save_path) = match source {
        ProjectSource::Directory(ref dir) => {
            let (pf, validation_errors) = AssetManager::load_project(dir).unwrap_or_else(|e| {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            });

            for err in &validation_errors {
                eprintln!(
                    "Warning: missing asset '{}' ({}): {}",
                    err.asset_id,
                    err.category,
                    err.resolved_path.display()
                );
            }

            let project_dir = dir.canonicalize().unwrap_or_else(|_| dir.to_path_buf());
            let mut pf = pf;
            let tp = resolve_asset_paths(&mut pf, &project_dir);
            let save_path = save_path_arg
                .map(PathBuf::from)
                .unwrap_or_else(|| dir.join("save.json"));
            (pf, tp, None, save_path)
        }
        ProjectSource::Zip(ref zip_path) => {
            // For ZIP sources, the launcher must manage the temp directory lifetime
            // so that Bevy can access extracted asset files asynchronously.
            let zip_data = std::fs::read(zip_path).unwrap_or_else(|e| {
                eprintln!("Error: could not read zip file: {}", e);
                std::process::exit(1);
            });

            let temp_dir = tempfile::tempdir().unwrap_or_else(|e| {
                eprintln!("Error: could not create temp directory: {}", e);
                std::process::exit(1);
            });

            let mut archive =
                zip::ZipArchive::new(std::io::Cursor::new(&zip_data)).unwrap_or_else(|e| {
                    eprintln!("Error: failed to open zip archive: {}", e);
                    std::process::exit(1);
                });

            archive.extract(temp_dir.path()).unwrap_or_else(|e| {
                eprintln!("Error: failed to extract zip archive: {}", e);
                std::process::exit(1);
            });

            // Load from the extracted directory using AssetManager
            let (pf, validation_errors) = AssetManager::load_project(temp_dir.path())
                .unwrap_or_else(|e| {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                });

            for err in &validation_errors {
                eprintln!(
                    "Warning: missing asset '{}' ({}): {}",
                    err.asset_id,
                    err.category,
                    err.resolved_path.display()
                );
            }

            let mut pf = pf;
            let tp = resolve_asset_paths(&mut pf, temp_dir.path());
            let save_path = save_path_arg.map(PathBuf::from).unwrap_or_else(|| {
                zip_path
                    .parent()
                    .map(|p| p.join("save.json"))
                    .unwrap_or_else(|| PathBuf::from("save.json"))
            });
            (pf, tp, Some(temp_dir), save_path)
        }
    };

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

    // Initialize AppPhase state (starts at TitleScreen by default)
    app.init_state::<AppPhase>();

    // Insert game state resources with defaults (not from save file)
    app.init_resource::<GameState>();
    app.init_resource::<CurrencyState>();
    app.init_resource::<InventoryState>();
    app.init_resource::<PartyState>();
    app.init_resource::<CharacterProgressState>();
    app.init_resource::<RendererState>();

    // Keep temp directory alive for the entire app lifetime (ZIP extractions)
    if let Some(td) = temp_dir {
        app.insert_resource(TempDirGuard { _dir: td });
    }

    app.insert_resource(PendingProjectLoad {
        project_file: project_file.clone(),
        tileset_paths,
        _temp_dir: None,
    })
    .insert_resource(SavePath {
        path: save_path.clone(),
    })
    .insert_resource(TitleScreenConfig {
        save_path,
        spawn_point: project_file.spawn_point.clone(),
    })
    .add_systems(PreStartup, load_project_resources)
    .add_plugins(TitleScreenPlugin)
    .add_plugins(ShopScenePlugin)
    .add_plugins(StatusScenePlugin)
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

    commands.insert_resource(ShopRegistryRes {
        registry: pending.project_file.shops.clone(),
    });

    commands.insert_resource(ItemRegistryRes {
        registry: pending.project_file.items.clone(),
    });

    commands.insert_resource(CharacterRegistryRes {
        registry: pending.project_file.characters.clone(),
    });

    commands.insert_resource(AbilityRegistryRes {
        registry: pending.project_file.abilities.clone(),
    });

    commands.insert_resource(DialogTextRegistry::from_map(
        pending.project_file.dialog_texts.clone(),
    ));

    commands.remove_resource::<PendingProjectLoad>();
}
