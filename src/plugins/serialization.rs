use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass};
use std::path::PathBuf;

use crate::data::{EditorState, MapData, ProjectFile, TilesetData, UndoHistory};

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
    Open,
}

/// System that processes pending serialization actions (save/load).
#[allow(clippy::too_many_arguments)]
fn handle_serialization_actions(
    mut action: ResMut<SerializationAction>,
    mut commands: Commands,
    map: Option<Res<MapData>>,
    tileset: Option<Res<TilesetData>>,
    mut editor_state: ResMut<EditorState>,
    mut undo_history: ResMut<UndoHistory>,
    asset_server: Res<AssetServer>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut contexts: EguiContexts,
) -> Result {
    let Some(request) = action.pending.take() else {
        return Ok(());
    };

    let ctx = contexts.ctx_mut()?;
    let _ = ctx; // We only needed to ensure egui context is available

    match request {
        SerializationRequest::Save => {
            if let Some(ref path) = editor_state.current_save_path.clone() {
                save_project_to_path(path, &map, &tileset, &mut editor_state);
            } else {
                // No path yet — fall through to Save As
                save_project_with_dialog(&map, &tileset, &mut editor_state);
            }
        }
        SerializationRequest::Open => {
            load_project_with_dialog(
                &mut commands,
                &mut editor_state,
                &mut undo_history,
                &asset_server,
                &mut atlas_layouts,
            );
        }
    }

    Ok(())
}

/// Opens a file dialog and saves the project to the chosen path.
fn save_project_with_dialog(
    map: &Option<Res<MapData>>,
    tileset: &Option<Res<TilesetData>>,
    editor_state: &mut ResMut<EditorState>,
) {
    let file = rfd::FileDialog::new()
        .add_filter("RPG Project", &["json"])
        .set_file_name("project.json")
        .save_file();

    if let Some(path) = file {
        save_project_to_path(&path, map, tileset, editor_state);
    }
}

/// Saves the project to a specific path.
fn save_project_to_path(
    path: &PathBuf,
    map: &Option<Res<MapData>>,
    tileset: &Option<Res<TilesetData>>,
    editor_state: &mut ResMut<EditorState>,
) {
    let Some(map) = map else {
        warn!("Cannot save: no map loaded");
        return;
    };

    let tileset_meta = tileset.as_ref().map(|ts| ts.meta.clone());
    let project = ProjectFile::new((**map).clone(), tileset_meta);

    match project.serialize() {
        Ok(json) => match std::fs::write(path, &json) {
            Ok(()) => {
                editor_state.has_unsaved_changes = false;
                editor_state.current_save_path = Some(path.clone());
                info!("Project saved to {}", path.display());
            }
            Err(e) => {
                warn!("Failed to write project file: {}", e);
            }
        },
        Err(e) => {
            warn!("Failed to serialize project: {}", e);
        }
    }
}

/// Opens a file dialog and loads a project from the chosen file.
fn load_project_with_dialog(
    commands: &mut Commands,
    editor_state: &mut ResMut<EditorState>,
    undo_history: &mut ResMut<UndoHistory>,
    asset_server: &Res<AssetServer>,
    atlas_layouts: &mut ResMut<Assets<TextureAtlasLayout>>,
) {
    let file = rfd::FileDialog::new()
        .add_filter("RPG Project", &["json"])
        .pick_file();

    let Some(path) = file else { return };

    let json = match std::fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) => {
            warn!("Failed to read project file: {}", e);
            return;
        }
    };

    let project = match ProjectFile::deserialize(&json) {
        Ok(p) => p,
        Err(e) => {
            warn!("Failed to deserialize project: {}", e);
            return;
        }
    };

    // Insert the loaded map data
    commands.insert_resource(project.map);

    // Clear undo history for the new project
    undo_history.undo_stack.clear();
    undo_history.redo_stack.clear();

    // Update editor state
    editor_state.has_unsaved_changes = false;
    editor_state.current_save_path = Some(path);
    editor_state.active_brush = None;

    // Reload tileset if the project references one
    if let Some(meta) = project.tileset {
        let tileset_path = &meta.file_path;
        if !tileset_path.is_empty() && std::path::Path::new(tileset_path).exists() {
            let layout = TextureAtlasLayout::from_grid(
                UVec2::new(meta.tile_width, meta.tile_height),
                meta.columns,
                meta.rows,
                None,
                None,
            );
            let atlas_handle = atlas_layouts.add(layout);
            let texture_handle: Handle<Image> = asset_server.load(tileset_path.to_string());

            let tileset_data = TilesetData {
                meta,
                texture: texture_handle,
                atlas_layout: atlas_handle,
            };
            commands.insert_resource(tileset_data);
        } else {
            warn!(
                "Tileset file not found: {}. Tileset will not be loaded.",
                tileset_path
            );
        }
    }

    info!("Project loaded successfully");
}
