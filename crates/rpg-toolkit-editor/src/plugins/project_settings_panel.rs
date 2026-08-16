//! Project Settings panel — includes the "Game Start Events" section
//! for editing `intro_events` on the project manifest.

use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

use crate::data::project::Project;
use crate::data::{AppEditorMode, EditorUiSet};
use crate::plugins::attribute::action_editor::ActionEditorState;
use crate::plugins::attribute::action_editor_ui;

/// Plugin that provides the project settings panel UI.
pub struct ProjectSettingsPanelPlugin;

impl Plugin for ProjectSettingsPanelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ProjectSettingsState>().add_systems(
            EguiPrimaryContextPass,
            project_settings_panel_ui
                .in_set(EditorUiSet::Panels)
                .run_if(resource_equals(AppEditorMode::ProjectSettings)),
        );
    }
}

/// Editor state for the project settings panel.
#[derive(Resource, Default)]
pub struct ProjectSettingsState {
    /// Action editor state for the intro events list.
    pub intro_events_editor: ActionEditorState,
}

fn project_settings_panel_ui(
    mut contexts: EguiContexts,
    mut project: ResMut<Project>,
    mut settings_state: ResMut<ProjectSettingsState>,
) -> Result {
    let ctx = contexts.ctx_mut()?;

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("Project Settings");
        ui.separator();

        // Game Start Events section
        egui::CollapsingHeader::new("🎬 Game Start Events")
            .default_open(true)
            .show(ui, |ui| {
                ui.label("Event actions to execute when a new game starts (after player spawns).");
                ui.separator();

                // Clone face_portraits before taking mutable borrow on intro_events
                let face_portraits = project.face_portraits.clone();

                // Get or initialize the intro_events list
                let intro_events = project.intro_events.get_or_insert_with(Vec::new);
                let previous_len = intro_events.len();

                // Render the action editor for intro events
                action_editor_ui::render_action_editor(
                    ui,
                    intro_events,
                    &mut settings_state.intro_events_editor,
                    "project_intro_events",
                    &[], // map_entries (not needed for cinematic actions)
                    &face_portraits,
                    0,    // depth
                    None, // reward_ctx
                    &[],  // shops
                );

                // Detect changes
                let current_len = project.intro_events.as_ref().map_or(0, |v| v.len());
                if current_len != previous_len {
                    project.has_unsaved_intro_events_changes = true;
                }
            });
    });

    Ok(())
}
