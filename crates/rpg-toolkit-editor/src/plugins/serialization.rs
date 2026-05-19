use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass};
use std::collections::HashMap;
use std::path::PathBuf;

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
    let _ = ctx; // We only needed to ensure egui context is available

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
            load_project_with_dialog(
                &mut project,
                &mut editor_state,
                &asset_server,
                &mut atlas_layouts,
                &mut text_id_index,
            );
        }
        SerializationRequest::NewProject => {
            *project = Project::default();
            editor_state.current_save_path = None;
            editor_state.active_brush = None;
            editor_state.active_tileset_tab = None;
            info!("New project created");
        }
    }

    Ok(())
}

/// Opens a file dialog and saves the project to the chosen path.
fn save_project_with_dialog(project: &mut ResMut<Project>, editor_state: &mut ResMut<EditorState>) {
    let file = rfd::FileDialog::new()
        .add_filter("RPG Project", &["json"])
        .set_file_name("project.json")
        .save_file();

    if let Some(path) = file {
        save_project_to_path(&path, project, editor_state);
    }
}

/// Saves the project to a specific path.
fn save_project_to_path(
    path: &PathBuf,
    project: &mut ResMut<Project>,
    editor_state: &mut ResMut<EditorState>,
) {
    // Extract maps (clone the HashMap)
    let maps = project.maps.clone();

    // Extract tileset metas only (runtime handles are not serialized)
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
    );

    match project_file.serialize() {
        Ok(json) => match std::fs::write(path, &json) {
            Ok(()) => {
                // Mark all maps as having no unsaved changes
                for (_id, has_changes) in project.has_unsaved_changes.iter_mut() {
                    *has_changes = false;
                }
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
    project: &mut ResMut<Project>,
    editor_state: &mut ResMut<EditorState>,
    asset_server: &Res<AssetServer>,
    atlas_layouts: &mut ResMut<Assets<TextureAtlasLayout>>,
    text_id_index: &mut ResMut<TextIdIndex>,
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

    let project_file = match ProjectFile::deserialize(&json) {
        Ok(p) => p,
        Err(e) => {
            warn!("Failed to deserialize project: {}", e);
            return;
        }
    };

    // Reconstruct tilesets with runtime handles from the serialized metas
    let mut tilesets = HashMap::new();
    for (id, meta) in &project_file.tilesets {
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
                tileset_path, id
            );
        }
    }

    // Initialize empty undo histories and unsaved-changes flags for each map
    let mut undo_histories = HashMap::new();
    let mut has_unsaved_changes = HashMap::new();
    for map_id in project_file.maps.keys() {
        undo_histories.insert(map_id.clone(), UndoHistory::default());
        has_unsaved_changes.insert(map_id.clone(), false);
    }

    // Build open_tabs from all map IDs and set active_tab
    let open_tabs: Vec<_> = project_file.maps.keys().cloned().collect();
    let active_tab = if open_tabs.is_empty() { None } else { Some(0) };

    // Replace the current project resource with the loaded one
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
    };

    // Reset editor state
    editor_state.current_save_path = Some(path);
    editor_state.active_brush = None;

    // Rebuild the TextIdIndex from the loaded maps
    **text_id_index = rebuild_text_id_index(&project.maps);

    info!("Project loaded successfully");
}
