use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::data::tileset::TilesetEntry;
use crate::data::undo::UndoHistory;
use crate::data::{EditorState, Project, ProjectFile};
use crate::plugins::dialog_text_panel::{TextIdIndex, rebuild_text_id_index};
use rpg_toolkit_common::asset::{AssetManager, ProjectSource};
use rpg_toolkit_common::{CharacterSpritesheet, SpritesheetId};

/// Plugin that handles project save/load via JSON serialization and native file dialogs.
pub struct SerializationPlugin;

impl Plugin for SerializationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SerializationAction>()
            .add_systems(EguiPrimaryContextPass, handle_serialization_actions);
    }
}

/// Resource used to communicate save/load requests between the menu UI and the serialization system.
#[derive(Resource, Default)]
pub struct SerializationAction {
    pub pending: Option<SerializationRequest>,
}

#[derive(Debug, Clone)]
pub enum SerializationRequest {
    Save,
    SaveAs,
    Open,
    NewProject,
}

/// Reconstruct tileset entries from a `ProjectFile` using the given asset server and atlas layouts.
fn reconstruct_tilesets(
    project_file: &ProjectFile,
    base_dir: &std::path::Path,
    asset_server: &Res<AssetServer>,
    atlas_layouts: &mut ResMut<Assets<TextureAtlasLayout>>,
) -> HashMap<String, TilesetEntry> {
    let mut tilesets = HashMap::new();
    for (id, meta) in &project_file.tilesets {
        let tileset_path = base_dir.join(&meta.file_path);
        if tileset_path.exists() {
            let layout = TextureAtlasLayout::from_grid(
                UVec2::new(meta.tile_width, meta.tile_height),
                meta.columns,
                meta.rows,
                None,
                None,
            );
            let atlas_handle = atlas_layouts.add(layout);
            let texture_handle: Handle<Image> =
                asset_server.load(tileset_path.to_string_lossy().to_string());

            tilesets.insert(
                id.clone(),
                TilesetEntry {
                    meta: meta.clone(),
                    texture: texture_handle,
                    atlas_layout: atlas_handle,
                },
            );
        } else {
            warn!(
                "Tileset file not found: {}. Tileset '{}' will not be loaded.",
                tileset_path.display(),
                id
            );
        }
    }
    tilesets
}

/// Load a project using `AssetManager`, handling both directory and ZIP sources.
///
/// For ZIP sources, the editor manages the temp directory lifetime so Bevy can
/// continue to access extracted assets.
fn load_project_unified(
    path: &std::path::Path,
    project: &mut ResMut<Project>,
    editor_state: &mut ResMut<EditorState>,
    asset_server: &Res<AssetServer>,
    atlas_layouts: &mut ResMut<Assets<TextureAtlasLayout>>,
    text_id_index: &mut ResMut<TextIdIndex>,
) {
    let source = match AssetManager::detect_source(path) {
        Ok(s) => s,
        Err(e) => {
            warn!("Unsupported project format: {}", e);
            return;
        }
    };

    match source {
        ProjectSource::Directory(dir) => {
            let (project_file, validation_errors) = match AssetManager::load_project(&dir) {
                Ok(result) => result,
                Err(e) => {
                    warn!("Failed to load project from directory: {}", e);
                    return;
                }
            };

            for err in &validation_errors {
                warn!(
                    "Missing asset '{}' ({}): {}",
                    err.asset_id,
                    err.category,
                    err.resolved_path.display()
                );
            }

            apply_loaded_project(
                &project_file,
                &dir,
                project,
                editor_state,
                asset_server,
                atlas_layouts,
                text_id_index,
            );

            editor_state.current_save_path = Some(dir);
            editor_state.original_zip_path = None;
            editor_state._temp_dir = None;

            info!("Project loaded successfully from directory");
        }
        ProjectSource::Zip(zip_path) => {
            // For ZIP sources, the editor must manage the temp directory lifetime
            // so that Bevy can access extracted asset files throughout the session.
            let zip_data = match std::fs::read(&zip_path) {
                Ok(data) => data,
                Err(e) => {
                    warn!("Failed to read ZIP file: {}", e);
                    return;
                }
            };

            let temp_dir = match tempfile::tempdir() {
                Ok(d) => d,
                Err(e) => {
                    warn!("Failed to create temp directory: {}", e);
                    return;
                }
            };

            let mut archive = match zip::ZipArchive::new(std::io::Cursor::new(&zip_data)) {
                Ok(a) => a,
                Err(e) => {
                    warn!("Failed to open ZIP archive: {}", e);
                    return;
                }
            };

            if let Err(e) = archive.extract(temp_dir.path()) {
                warn!("Failed to extract ZIP archive: {}", e);
                return;
            }

            // Load from the extracted directory using AssetManager
            let (project_file, validation_errors) =
                match AssetManager::load_project(temp_dir.path()) {
                    Ok(result) => result,
                    Err(e) => {
                        warn!("Failed to load project from ZIP: {}", e);
                        return;
                    }
                };

            for err in &validation_errors {
                warn!(
                    "Missing asset '{}' ({}): {}",
                    err.asset_id,
                    err.category,
                    err.resolved_path.display()
                );
            }

            let base_dir = temp_dir.path().to_path_buf();
            apply_loaded_project(
                &project_file,
                &base_dir,
                project,
                editor_state,
                asset_server,
                atlas_layouts,
                text_id_index,
            );

            editor_state.current_save_path = Some(base_dir);
            editor_state.original_zip_path = Some(zip_path);
            editor_state._temp_dir = Some(temp_dir);

            info!("Project loaded successfully from ZIP archive");
        }
    }

    editor_state.active_brush = None;
}

/// Apply a loaded `ProjectFile` to the editor's `Project` resource.
///
/// This handles reconstructing tilesets (Bevy textures/atlases), resolving
/// spritesheet paths from relative to absolute, and setting up undo histories.
fn apply_loaded_project(
    project_file: &ProjectFile,
    base_dir: &std::path::Path,
    project: &mut ResMut<Project>,
    _editor_state: &mut ResMut<EditorState>,
    asset_server: &Res<AssetServer>,
    atlas_layouts: &mut ResMut<Assets<TextureAtlasLayout>>,
    text_id_index: &mut ResMut<TextIdIndex>,
) {
    let tilesets = reconstruct_tilesets(project_file, base_dir, asset_server, atlas_layouts);

    let mut undo_histories = HashMap::new();
    let mut has_unsaved_changes = HashMap::new();
    for map_id in project_file.maps.keys() {
        undo_histories.insert(map_id.clone(), UndoHistory::default());
        has_unsaved_changes.insert(map_id.clone(), false);
    }

    let open_tabs: Vec<_> = project_file.maps.keys().cloned().collect();
    let active_tab = if open_tabs.is_empty() { None } else { Some(0) };

    // Resolve spritesheet paths from relative to absolute for Bevy asset loading
    let spritesheets = resolve_spritesheet_paths_to_absolute(&project_file.spritesheets, base_dir);

    **project = Project {
        maps: project_file.maps.clone(),
        tilesets,
        open_tabs,
        active_tab,
        undo_histories,
        has_unsaved_changes,
        spawn_point: project_file.spawn_point.clone(),
        spritesheets,
        player_spritesheet: project_file.player_spritesheet.clone(),
        dialog_texts: project_file.dialog_texts.clone(),
        face_portraits: project_file.face_portraits.clone(),
        characters: project_file.characters.clone(),
        has_unsaved_character_changes: false,
        items: project_file.items.clone(),
        has_unsaved_item_changes: false,
        abilities: project_file.abilities.clone(),
        has_unsaved_ability_changes: false,
        enemies: project_file.enemies.clone(),
        has_unsaved_enemy_changes: false,
        shops: project_file.shops.clone(),
        has_unsaved_shop_changes: false,
        intro_events: project_file.intro_events.clone(),
        has_unsaved_intro_events_changes: false,
    };

    **text_id_index = rebuild_text_id_index(&project.maps);
}

/// Resolve spritesheet file paths from relative (as stored on disk) to absolute
/// paths that the Bevy asset server can load at runtime.
fn resolve_spritesheet_paths_to_absolute(
    spritesheets: &HashMap<SpritesheetId, CharacterSpritesheet>,
    base_dir: &std::path::Path,
) -> HashMap<SpritesheetId, CharacterSpritesheet> {
    spritesheets
        .iter()
        .map(|(id, ss)| {
            let mut ss_clone = ss.clone();
            if !ss_clone.file_path.is_empty() {
                let path = std::path::Path::new(&ss_clone.file_path);
                if !path.is_absolute() {
                    let resolved = base_dir.join(path);
                    ss_clone.file_path = resolved.to_string_lossy().to_string();
                }
            }
            (id.clone(), ss_clone)
        })
        .collect()
}

/// Build a `ProjectFile` from the editor's `Project` with normalized relative paths
/// suitable for on-disk storage via `AssetManager`.
fn build_project_file_for_save(project: &Project) -> ProjectFile {
    let tilesets_meta: HashMap<_, _> = project
        .tilesets
        .iter()
        .map(|(id, entry)| {
            let name = std::path::Path::new(&entry.meta.file_path)
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown.png");
            let mut meta = entry.meta.clone();
            meta.file_path = format!("tilesets/{}", name);
            (id.clone(), meta)
        })
        .collect();

    let spritesheets: HashMap<_, _> = project
        .spritesheets
        .iter()
        .map(|(id, ss)| {
            let name = std::path::Path::new(&ss.file_path)
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown.png");
            let mut ss_clone = ss.clone();
            ss_clone.file_path = format!("data/{}", name);
            (id.clone(), ss_clone)
        })
        .collect();

    // Normalize item icon paths to relative
    let mut items = project.items.clone();
    for item in items.items.values_mut() {
        if let Some(ref icon_path) = item.graphics.icon {
            let name = std::path::Path::new(icon_path)
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown.png");
            item.graphics.icon = Some(format!("data/{}", name));
        }
    }

    // Normalize ability icon paths to relative
    let mut abilities = project.abilities.clone();
    for ability in abilities.abilities.values_mut() {
        if let Some(ref icon_path) = ability.graphics.icon {
            let name = std::path::Path::new(icon_path)
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown.png");
            ability.graphics.icon = Some(format!("data/{}", name));
        }
    }

    let mut project_file = ProjectFile::new(
        project.maps.clone(),
        tilesets_meta,
        project.spawn_point.clone(),
        spritesheets,
        project.player_spritesheet.clone(),
        project.dialog_texts.clone(),
        project.face_portraits.clone(),
        project.characters.clone(),
        items,
        abilities,
        project.enemies.clone(),
        project.shops.clone(),
    );
    project_file.intro_events = project.intro_events.clone();
    project_file
}

/// Copy asset files from their current (possibly absolute) paths to the proper
/// subdirectories under the target directory. This ensures the source_dir has
/// the expected structure for `AssetManager::save_project`.
fn copy_assets_to_source_dir(project: &Project, target_dir: &std::path::Path) {
    let tilesets_dir = target_dir.join("tilesets");
    let data_dir = target_dir.join("data");
    std::fs::create_dir_all(&tilesets_dir).ok();
    std::fs::create_dir_all(&data_dir).ok();

    for entry in project.tilesets.values() {
        let current_path = &entry.meta.file_path;
        if current_path.is_empty() {
            continue;
        }
        let name = std::path::Path::new(current_path)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown.png");
        let dest = tilesets_dir.join(name);
        if !dest.exists()
            && let Ok(data) = std::fs::read(current_path)
        {
            std::fs::write(&dest, data).ok();
        }
    }

    for ss in project.spritesheets.values() {
        let current_path = &ss.file_path;
        if current_path.is_empty() {
            continue;
        }
        let name = std::path::Path::new(current_path)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown.png");
        let dest = data_dir.join(name);
        if !dest.exists()
            && let Ok(data) = std::fs::read(current_path)
        {
            std::fs::write(&dest, data).ok();
        }
    }

    // Copy item icon files
    for item in project.items.items.values() {
        if let Some(ref icon_path) = item.graphics.icon {
            if icon_path.is_empty() {
                continue;
            }
            let name = std::path::Path::new(icon_path)
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown.png");
            let dest = data_dir.join(name);
            if !dest.exists()
                && let Ok(data) = std::fs::read(icon_path)
            {
                std::fs::write(&dest, data).ok();
            }
        }
    }

    // Copy ability icon files
    for ability in project.abilities.abilities.values() {
        if let Some(ref icon_path) = ability.graphics.icon {
            if icon_path.is_empty() {
                continue;
            }
            let name = std::path::Path::new(icon_path)
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown.png");
            let dest = data_dir.join(name);
            if !dest.exists()
                && let Ok(data) = std::fs::read(icon_path)
            {
                std::fs::write(&dest, data).ok();
            }
        }
    }
}

/// Save a project using `AssetManager`, handling both directory and ZIP targets.
fn save_project_unified(
    target: &std::path::Path,
    project: &Project,
    current_save_path: Option<&std::path::Path>,
) -> Result<(), String> {
    // Determine the source directory where asset files currently reside
    let source_dir = current_save_path
        .and_then(|p| {
            if p.is_dir() {
                Some(p.to_path_buf())
            } else {
                p.parent().map(|d| d.to_path_buf())
            }
        })
        .unwrap_or_else(|| {
            // For a new save with no existing path, use the target itself for directories
            if target.extension().is_none() || target.is_dir() {
                target.to_path_buf()
            } else {
                target
                    .parent()
                    .map(|d| d.to_path_buf())
                    .unwrap_or_else(|| PathBuf::from("."))
            }
        });

    // Copy assets from their current (possibly absolute) paths to the source_dir
    copy_assets_to_source_dir(project, &source_dir);

    // Build a ProjectFile with normalized relative paths
    let project_file = build_project_file_for_save(project);

    // Create an AssetManager and populate its registry from the project file
    let mut manager = AssetManager::new();
    let registry = AssetManager::registry_from_project_file(&project_file);
    manager.set_registry(registry);

    // Save using AssetManager
    let warnings = manager
        .save_project(&project_file, target, &source_dir)
        .map_err(|e| e.to_string())?;

    for warning in &warnings {
        warn!(
            "Save warning for '{}' ({}): {}",
            warning.asset_id, warning.category, warning.message
        );
    }

    Ok(())
}

/// System that processes pending serialization actions (save/load).
#[allow(clippy::too_many_arguments)]
fn handle_serialization_actions(
    mut action: ResMut<SerializationAction>,
    mut project: ResMut<Project>,
    mut editor_state: ResMut<EditorState>,
    asset_server: Res<AssetServer>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut contexts: EguiContexts,
    mut text_id_index: ResMut<TextIdIndex>,
) -> Result {
    let Some(request) = action.pending.take() else {
        return Ok(());
    };

    let ctx = contexts.ctx_mut()?;
    let _ = ctx;

    match request {
        SerializationRequest::Save => {
            if let Some(ref zip_path) = editor_state.original_zip_path.clone() {
                // Re-save back to the original ZIP archive
                save_project_to_path(zip_path, &mut project, &mut editor_state);
            } else if let Some(ref path) = editor_state.current_save_path.clone() {
                save_project_to_path(path, &mut project, &mut editor_state);
            } else {
                save_project_with_dialog(&mut project, &mut editor_state);
            }
        }
        SerializationRequest::SaveAs => {
            save_project_with_dialog(&mut project, &mut editor_state);
        }
        SerializationRequest::Open => {
            let file = rfd::FileDialog::new()
                .add_filter("RPG Project", &["rpg"])
                .pick_file();

            let Some(path) = file else { return Ok(()) };

            // Use AssetManager::detect_source for directory and .rpg formats
            match AssetManager::detect_source(&path) {
                Ok(_source) => {
                    load_project_unified(
                        &path,
                        &mut project,
                        &mut editor_state,
                        &asset_server,
                        &mut atlas_layouts,
                        &mut text_id_index,
                    );
                }
                Err(_) => {
                    if path.extension().is_some_and(|e| e == "json") {
                        warn!(
                            "Legacy JSON format is no longer supported. Please convert your project to directory or .rpg ZIP format."
                        );
                    } else {
                        warn!("Unsupported project format: {}", path.display());
                    }
                }
            }
        }
        SerializationRequest::NewProject => {
            *project = Project::default();
            editor_state.current_save_path = None;
            editor_state.active_brush = None;
            editor_state.original_zip_path = None;
            editor_state._temp_dir = None;
            info!("New project created");
        }
    }

    Ok(())
}

/// Opens a file dialog and saves the project to the chosen path.
fn save_project_with_dialog(project: &mut ResMut<Project>, editor_state: &mut ResMut<EditorState>) {
    let file = rfd::FileDialog::new()
        .add_filter("RPG Project (directory)", &["rpg"])
        .add_filter("RPG Project (ZIP)", &["rpg"])
        .set_file_name("project.rpg")
        .save_file();

    if let Some(path) = file {
        save_project_to_path(&path, project, editor_state);
    }
}

/// Saves the project to a specific path, auto-detecting format from extension.
fn save_project_to_path(
    path: &std::path::Path,
    project: &mut ResMut<Project>,
    editor_state: &mut ResMut<EditorState>,
) {
    if path.is_dir() || path.extension().is_some_and(|e| e == "rpg") {
        // Use AssetManager for directory and ZIP saves
        let result = save_project_unified(path, project, editor_state.current_save_path.as_deref());

        match result {
            Ok(()) => {
                for has_changes in project.has_unsaved_changes.values_mut() {
                    *has_changes = false;
                }
                project.has_unsaved_character_changes = false;
                project.has_unsaved_item_changes = false;
                project.has_unsaved_ability_changes = false;
                project.has_unsaved_enemy_changes = false;
                project.has_unsaved_shop_changes = false;

                if path.is_dir() {
                    editor_state.current_save_path = Some(path.to_path_buf());
                    editor_state.original_zip_path = None;
                    editor_state._temp_dir = None;
                    info!("Project saved to directory {}", path.display());
                } else {
                    editor_state.current_save_path = Some(path.to_path_buf());
                    editor_state.original_zip_path = Some(path.to_path_buf());
                    info!("Project saved as ZIP to {}", path.display());
                }
            }
            Err(e) => {
                warn!("Failed to save project: {}", e);
            }
        }
    } else {
        warn!(
            "Unsupported save format: {}. Use directory or .rpg format.",
            path.display()
        );
    }
}
