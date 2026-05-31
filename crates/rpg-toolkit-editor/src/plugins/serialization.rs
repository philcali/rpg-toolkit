use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass};
use std::collections::HashMap;
use std::io::Write;

use crate::data::tileset::TilesetEntry;
use crate::data::undo::UndoHistory;
use crate::data::{EditorState, Project, ProjectFile};
use crate::plugins::dialog_text_panel::{TextIdIndex, rebuild_text_id_index};

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

/// Detect project format from a path.
enum ProjectSource {
    Directory(std::path::PathBuf),
    Zip(std::path::PathBuf),
    LegacyJson(std::path::PathBuf),
}

fn detect_project_source(path: &std::path::Path) -> Option<ProjectSource> {
    if path.is_dir() {
        Some(ProjectSource::Directory(path.to_path_buf()))
    } else if path.extension().is_some_and(|e| e == "rpg") {
        Some(ProjectSource::Zip(path.to_path_buf()))
    } else if path.extension().is_some_and(|e| e == "json") {
        Some(ProjectSource::LegacyJson(path.to_path_buf()))
    } else {
        None
    }
}

/// When saving to a directory format, ensure asset files are in the right
/// subdirectory and update file_path references accordingly.
fn prepare_assets_for_save(project: &mut Project, project_dir: &std::path::Path) {
    let tilesets_dir = project_dir.join("tilesets");
    let data_dir = project_dir.join("data");
    std::fs::create_dir_all(&tilesets_dir).ok();
    std::fs::create_dir_all(&data_dir).ok();

    for entry in project.tilesets.values_mut() {
        let current_path = &entry.meta.file_path;
        if current_path.is_empty() {
            continue;
        }
        let name = std::path::Path::new(current_path)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown.png");
        let new_path = format!("tilesets/{}", name);
        let dest = tilesets_dir.join(name);
        if current_path != &new_path
            && !dest.exists()
            && let Ok(data) = std::fs::read(current_path)
        {
            std::fs::write(&dest, data).ok();
        }
        entry.meta.file_path = new_path;
    }

    for ss in project.spritesheets.values_mut() {
        let current_path = &ss.file_path;
        if current_path.is_empty() {
            continue;
        }
        let name = std::path::Path::new(current_path)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown.png");
        let new_path = format!("data/{}", name);
        let dest = data_dir.join(name);
        if current_path != &new_path
            && !dest.exists()
            && let Ok(data) = std::fs::read(current_path)
        {
            std::fs::write(&dest, data).ok();
        }
        ss.file_path = new_path;
    }
}

/// Build a `ProjectFile` from the editor's `Project` resource.
fn to_project_file(project: &Project) -> ProjectFile {
    let tilesets_meta: HashMap<_, _> = project
        .tilesets
        .iter()
        .map(|(id, entry)| (id.clone(), entry.meta.clone()))
        .collect();

    ProjectFile::new(
        project.maps.clone(),
        tilesets_meta,
        project.spawn_point.clone(),
        project.spritesheets.clone(),
        project.player_spritesheet.clone(),
        project.dialog_texts.clone(),
        project.face_portraits.clone(),
    )
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

/// Load a project from a directory-based format.
fn load_project_from_dir(
    dir: &std::path::Path,
    project: &mut ResMut<Project>,
    editor_state: &mut ResMut<EditorState>,
    asset_server: &Res<AssetServer>,
    atlas_layouts: &mut ResMut<Assets<TextureAtlasLayout>>,
    text_id_index: &mut ResMut<TextIdIndex>,
) {
    let project_file = match ProjectFile::deserialize_from_dir(dir) {
        Ok(p) => p,
        Err(e) => {
            warn!("Failed to deserialize project: {}", e);
            return;
        }
    };

    let tilesets = reconstruct_tilesets(&project_file, dir, asset_server, atlas_layouts);

    let mut undo_histories = HashMap::new();
    let mut has_unsaved_changes = HashMap::new();
    for map_id in project_file.maps.keys() {
        undo_histories.insert(map_id.clone(), UndoHistory::default());
        has_unsaved_changes.insert(map_id.clone(), false);
    }

    let open_tabs: Vec<_> = project_file.maps.keys().cloned().collect();
    let active_tab = if open_tabs.is_empty() { None } else { Some(0) };

    **project = Project {
        maps: project_file.maps,
        tilesets,
        open_tabs,
        active_tab,
        undo_histories,
        has_unsaved_changes,
        spawn_point: project_file.spawn_point,
        spritesheets: project_file.spritesheets,
        player_spritesheet: project_file.player_spritesheet,
        dialog_texts: project_file.dialog_texts,
        face_portraits: project_file.face_portraits,
    };

    editor_state.current_save_path = Some(dir.to_path_buf());
    editor_state.active_brush = None;
    editor_state.original_zip_path = None;

    **text_id_index = rebuild_text_id_index(&project.maps);

    info!("Project loaded successfully from directory");
}

/// Load a project from a ZIP archive, extracting to a temp directory.
fn load_project_from_zip(
    zip_path: &std::path::Path,
    project: &mut ResMut<Project>,
    editor_state: &mut ResMut<EditorState>,
    asset_server: &Res<AssetServer>,
    atlas_layouts: &mut ResMut<Assets<TextureAtlasLayout>>,
    text_id_index: &mut ResMut<TextIdIndex>,
) {
    let zip_data = match std::fs::read(zip_path) {
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

    let project_file = match ProjectFile::deserialize_from_dir(temp_dir.path()) {
        Ok(p) => p,
        Err(e) => {
            warn!("Failed to deserialize project from ZIP: {}", e);
            return;
        }
    };

    let tilesets =
        reconstruct_tilesets(&project_file, temp_dir.path(), asset_server, atlas_layouts);

    let mut undo_histories = HashMap::new();
    let mut has_unsaved_changes = HashMap::new();
    for map_id in project_file.maps.keys() {
        undo_histories.insert(map_id.clone(), UndoHistory::default());
        has_unsaved_changes.insert(map_id.clone(), false);
    }

    let open_tabs: Vec<_> = project_file.maps.keys().cloned().collect();
    let active_tab = if open_tabs.is_empty() { None } else { Some(0) };

    **project = Project {
        maps: project_file.maps,
        tilesets,
        open_tabs,
        active_tab,
        undo_histories,
        has_unsaved_changes,
        spawn_point: project_file.spawn_point,
        spritesheets: project_file.spritesheets,
        player_spritesheet: project_file.player_spritesheet,
        dialog_texts: project_file.dialog_texts,
        face_portraits: project_file.face_portraits,
    };

    editor_state.current_save_path = Some(temp_dir.path().to_path_buf());
    editor_state.active_brush = None;
    editor_state.original_zip_path = Some(zip_path.to_path_buf());

    **text_id_index = rebuild_text_id_index(&project.maps);

    info!("Project loaded successfully from ZIP archive");
}

/// Load a legacy single-file JSON project.
fn load_project_from_json(
    json_path: &std::path::Path,
    project: &mut ResMut<Project>,
    editor_state: &mut ResMut<EditorState>,
    asset_server: &Res<AssetServer>,
    atlas_layouts: &mut ResMut<Assets<TextureAtlasLayout>>,
    text_id_index: &mut ResMut<TextIdIndex>,
) {
    let json = match std::fs::read_to_string(json_path) {
        Ok(s) => s,
        Err(e) => {
            warn!("Failed to read project file: {}", e);
            return;
        }
    };

    let project_file = match ProjectFile::deserialize(&json) {
        Ok(p) => p,
        Err(e) => {
            warn!("Failed to deserialize project: {}", e);
            return;
        }
    };

    let project_dir = json_path
        .parent()
        .map(|p| p.canonicalize().unwrap_or_else(|_| p.to_path_buf()))
        .unwrap_or_else(|| std::path::PathBuf::from("."));

    let tilesets = reconstruct_tilesets(&project_file, &project_dir, asset_server, atlas_layouts);

    let mut undo_histories = HashMap::new();
    let mut has_unsaved_changes = HashMap::new();
    for map_id in project_file.maps.keys() {
        undo_histories.insert(map_id.clone(), UndoHistory::default());
        has_unsaved_changes.insert(map_id.clone(), false);
    }

    let open_tabs: Vec<_> = project_file.maps.keys().cloned().collect();
    let active_tab = if open_tabs.is_empty() { None } else { Some(0) };

    **project = Project {
        maps: project_file.maps,
        tilesets,
        open_tabs,
        active_tab,
        undo_histories,
        has_unsaved_changes,
        spawn_point: project_file.spawn_point,
        spritesheets: project_file.spritesheets,
        player_spritesheet: project_file.player_spritesheet,
        dialog_texts: project_file.dialog_texts,
        face_portraits: project_file.face_portraits,
    };

    editor_state.current_save_path = Some(json_path.to_path_buf());
    editor_state.active_brush = None;
    editor_state.original_zip_path = None;

    **text_id_index = rebuild_text_id_index(&project.maps);

    info!("Project loaded successfully from legacy JSON");
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
            if let Some(ref path) = editor_state.current_save_path.clone() {
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
                .add_filter("RPG Project", &["rpg", "json"])
                .pick_file();

            let Some(path) = file else { return Ok(()) };

            if let Some(source) = detect_project_source(&path) {
                match source {
                    ProjectSource::Directory(dir) => {
                        load_project_from_dir(
                            &dir,
                            &mut project,
                            &mut editor_state,
                            &asset_server,
                            &mut atlas_layouts,
                            &mut text_id_index,
                        );
                    }
                    ProjectSource::Zip(zip_path) => {
                        load_project_from_zip(
                            &zip_path,
                            &mut project,
                            &mut editor_state,
                            &asset_server,
                            &mut atlas_layouts,
                            &mut text_id_index,
                        );
                    }
                    ProjectSource::LegacyJson(json_path) => {
                        load_project_from_json(
                            &json_path,
                            &mut project,
                            &mut editor_state,
                            &asset_server,
                            &mut atlas_layouts,
                            &mut text_id_index,
                        );
                    }
                }
            } else {
                warn!("Unsupported project format: {}", path.display());
            }
        }
        SerializationRequest::NewProject => {
            *project = Project::default();
            editor_state.current_save_path = None;
            editor_state.active_brush = None;
            editor_state.original_zip_path = None;
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
        .add_filter("RPG Project (legacy)", &["json"])
        .set_file_name("project.json")
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
    if path.is_dir() {
        if let Err(e) = save_to_directory(path, project) {
            warn!("Failed to save project to directory: {}", e);
        } else {
            for (_id, has_changes) in project.has_unsaved_changes.iter_mut() {
                *has_changes = false;
            }
            editor_state.current_save_path = Some(path.to_path_buf());
            editor_state.original_zip_path = None;
            info!("Project saved to directory {}", path.display());
        }
    } else if path.extension().is_some_and(|e| e == "rpg") {
        if let Err(e) = save_to_zip(path, project, editor_state.current_save_path.as_deref()) {
            warn!("Failed to save project as ZIP: {}", e);
        } else {
            for (_id, has_changes) in project.has_unsaved_changes.iter_mut() {
                *has_changes = false;
            }
            editor_state.current_save_path = Some(path.to_path_buf());
            editor_state.original_zip_path = Some(path.to_path_buf());
            info!("Project saved as ZIP to {}", path.display());
        }
    } else {
        save_to_json(path, project, editor_state);
    }
}

/// Save project to a directory-based format.
fn save_to_directory(path: &std::path::Path, project: &mut ResMut<Project>) -> Result<(), String> {
    prepare_assets_for_save(project, path);
    let project_file = to_project_file(project);
    project_file
        .serialize_to_dir(path)
        .map_err(|e| e.to_string())
}

/// Save project as a ZIP archive.
fn save_to_zip(
    zip_path: &std::path::Path,
    project: &mut Project,
    current_save_path: Option<&std::path::Path>,
) -> Result<(), String> {
    let project_dir = current_save_path
        .and_then(|p| {
            if p.is_dir() {
                Some(p.to_path_buf())
            } else {
                p.parent().map(|d| d.to_path_buf())
            }
        })
        .unwrap_or_else(|| std::path::PathBuf::from("."));

    prepare_assets_for_save(project, &project_dir);

    let project_file = to_project_file(project);
    let manifest = project_file.to_manifest();

    let mut file =
        std::fs::File::create(zip_path).map_err(|e| format!("could not create ZIP file: {}", e))?;
    let mut zip = zip::ZipWriter::new(&mut file);
    let options =
        zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    let manifest_json = serde_json::to_string_pretty(&manifest)
        .map_err(|e| format!("could not serialize manifest: {}", e))?;
    zip.start_file("manifest.json", options)
        .map_err(|e| e.to_string())?;
    zip.write_all(manifest_json.as_bytes())
        .map_err(|e| e.to_string())?;

    for (map_id, map) in &project_file.maps {
        let map_json = serde_json::to_string_pretty(map)
            .map_err(|e| format!("could not serialize map '{}': {}", map_id, e))?;
        zip.start_file(format!("maps/{}.json", map_id), options)
            .map_err(|e| e.to_string())?;
        zip.write_all(map_json.as_bytes())
            .map_err(|e| e.to_string())?;
    }

    for (id, meta) in &project_file.tilesets {
        let src = project_dir.join(&meta.file_path);
        if !src.exists() {
            warn!(
                "Tileset file not found: {} (referenced by {})",
                src.display(),
                id
            );
            continue;
        }
        let dest = format!("tilesets/{}", meta.file_path);
        zip.start_file(&dest, options).map_err(|e| e.to_string())?;
        let data = std::fs::read(&src)
            .map_err(|e| format!("could not read tileset {}: {}", src.display(), e))?;
        zip.write_all(&data).map_err(|e| e.to_string())?;
    }

    for (id, ss) in &project_file.spritesheets {
        let src = project_dir.join(&ss.file_path);
        if !src.exists() {
            warn!(
                "Spritesheet file not found: {} (referenced by {})",
                src.display(),
                id
            );
            continue;
        }
        let dest = format!("data/{}", ss.file_path);
        zip.start_file(&dest, options).map_err(|e| e.to_string())?;
        let data = std::fs::read(&src)
            .map_err(|e| format!("could not read spritesheet {}: {}", src.display(), e))?;
        zip.write_all(&data).map_err(|e| e.to_string())?;
    }

    zip.finish()
        .map_err(|e| format!("could not finish ZIP: {}", e))?;
    Ok(())
}

/// Save project to legacy JSON format.
fn save_to_json(
    path: &std::path::Path,
    project: &mut ResMut<Project>,
    editor_state: &mut ResMut<EditorState>,
) {
    let maps = project.maps.clone();
    let tilesets_meta: HashMap<_, _> = project
        .tilesets
        .iter()
        .map(|(id, entry)| (id.clone(), entry.meta.clone()))
        .collect();

    let project_file = ProjectFile::new(
        maps,
        tilesets_meta,
        project.spawn_point.clone(),
        project.spritesheets.clone(),
        project.player_spritesheet.clone(),
        project.dialog_texts.clone(),
        project.face_portraits.clone(),
    );

    match project_file.serialize() {
        Ok(json) => match std::fs::write(path, &json) {
            Ok(()) => {
                for (_id, has_changes) in project.has_unsaved_changes.iter_mut() {
                    *has_changes = false;
                }
                editor_state.current_save_path = Some(path.to_path_buf());
                info!("Project saved to {}", path.display());
            }
            Err(e) => warn!("Failed to write project file: {}", e),
        },
        Err(e) => warn!("Failed to serialize project: {}", e),
    }
}
