//! Project Settings panel — includes the "Game Start Events" section
//! for editing `intro_events` and the "Hotkey Bindings" section on the project manifest.

use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

use crate::data::project::Project;
use crate::data::{AppEditorMode, EditorUiSet};
use crate::plugins::attribute::action_editor::ActionEditorState;
use crate::plugins::attribute::action_editor_ui;
use crate::plugins::hotkey_panel::{HotkeyPanelState, render_hotkey_panel};

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
    mut hotkey_state: ResMut<HotkeyPanelState>,
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

                // Build portrait entries from characters with face_portrait set
                let portrait_entries: Vec<(String, String)> = project
                    .characters
                    .characters
                    .values()
                    .filter_map(|c| {
                        c.visual_assets
                            .face_portrait
                            .as_ref()
                            .filter(|p| !p.is_empty())
                            .map(|p| (p.clone(), c.display_name.clone()))
                    })
                    .collect();

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
                    &portrait_entries,
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

        ui.add_space(16.0);

        // Hotkey Bindings section
        egui::CollapsingHeader::new("⌨ Hotkey Bindings")
            .default_open(true)
            .show(ui, |ui| {
                render_hotkey_panel(ui, &mut project, &mut hotkey_state);
            });
    });

    Ok(())
}
