//! Hotkey Bindings panel — allows adding, editing, and removing
//! hotkey bindings in project settings.

use bevy::prelude::*;
use bevy_egui::egui;

use crate::data::project::Project;
use crate::plugins::attribute::action_editor::ActionEditorState;
use crate::plugins::attribute::action_editor_ui;

use rpg_toolkit_common::HotkeyBinding;

/// Plugin that provides the hotkey bindings panel UI in project settings.
pub struct HotkeyPanelPlugin;

impl Plugin for HotkeyPanelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HotkeyPanelState>();
        // No standalone system — rendered inline by project_settings_panel_ui
    }
}

/// Editor state for the hotkey bindings panel.
#[derive(Default, Resource)]
pub struct HotkeyPanelState {
    /// Per-binding action editor states, indexed by binding position.
    pub action_editors: Vec<ActionEditorState>,
    /// Whether key capture mode is active for a specific binding index.
    pub capturing_key_for: Option<usize>,
}

/// Renders the hotkey bindings editor UI inline within a parent panel.
/// Called from `project_settings_panel_ui`.
pub fn render_hotkey_panel(
    ui: &mut egui::Ui,
    project: &mut ResMut<Project>,
    panel_state: &mut ResMut<HotkeyPanelState>,
) {
    ui.heading("Hotkey Bindings");
    ui.label("Configure keyboard shortcuts that fire event actions during gameplay.");
    ui.separator();

    // Ensure action_editors vec is in sync with bindings count
    while panel_state.action_editors.len() < project.hotkey_bindings.len() {
        panel_state
            .action_editors
            .push(ActionEditorState::default());
    }
    if panel_state.action_editors.len() > project.hotkey_bindings.len() {
        panel_state
            .action_editors
            .truncate(project.hotkey_bindings.len());
    }

    // "Add Binding" button
    ui.horizontal(|ui| {
        if ui.button("+ Add Binding").clicked() {
            project.hotkey_bindings.push(HotkeyBinding {
                key_code: String::new(),
                name: String::new(),
                event_actions: Vec::new(),
            });
            panel_state
                .action_editors
                .push(ActionEditorState::default());
            project.has_unsaved_hotkey_changes = true;
        }
        ui.label(format!("{} binding(s)", project.hotkey_bindings.len()));
    });

    ui.separator();

    // Validation: check if any binding has empty key_code or name
    let has_validation_errors = project
        .hotkey_bindings
        .iter()
        .any(|b| b.key_code.is_empty() || b.name.is_empty());

    if has_validation_errors {
        ui.colored_label(
            egui::Color32::from_rgb(255, 180, 0),
            "⚠ Some bindings have empty key_code or name. Fill in all fields before saving.",
        );
        ui.separator();
    }

    if project.hotkey_bindings.is_empty() {
        ui.label("No hotkey bindings. Click \"+ Add Binding\" to add one.");
        return;
    }

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

    // Track deferred mutations
    let mut remove_index: Option<usize> = None;
    let mut swap: Option<(usize, usize)> = None;

    let binding_count = project.hotkey_bindings.len();

    egui::ScrollArea::vertical()
        .id_salt("hotkey_bindings_scroll")
        .show(ui, |ui| {
            for i in 0..binding_count {
                let id_salt = format!("hotkey_binding_{}", i);
                ui.push_id(&id_salt, |ui| {
                    ui.group(|ui| {
                        // Header row with reorder and remove buttons
                        ui.horizontal(|ui| {
                            ui.strong(format!("Binding {}", i + 1));

                            // Reorder arrows
                            if i > 0 && ui.small_button("▲").clicked() {
                                swap = Some((i, i - 1));
                            }
                            if i + 1 < binding_count && ui.small_button("▼").clicked() {
                                swap = Some((i, i + 1));
                            }

                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if ui.button("✕ Remove").clicked() {
                                        remove_index = Some(i);
                                    }
                                },
                            );
                        });

                        // Key code input with capture
                        let is_capturing = panel_state.capturing_key_for == Some(i);
                        let key_code_empty = project.hotkey_bindings[i].key_code.is_empty();

                        ui.horizontal(|ui| {
                            ui.label("Key Code:");
                            if is_capturing {
                                ui.colored_label(
                                    egui::Color32::from_rgb(100, 180, 255),
                                    "Press a key...",
                                );
                                // Check for key input via egui events
                                let mut captured_key: Option<String> = None;
                                ui.input(|input| {
                                    for event in &input.events {
                                        if let egui::Event::Key {
                                            key, pressed: true, ..
                                        } = event
                                        {
                                            captured_key = Some(format!("{:?}", key));
                                        }
                                    }
                                });
                                if let Some(key) = captured_key {
                                    project.hotkey_bindings[i].key_code = key;
                                    panel_state.capturing_key_for = None;
                                    project.has_unsaved_hotkey_changes = true;
                                }
                                if ui.button("Cancel").clicked() {
                                    panel_state.capturing_key_for = None;
                                }
                            } else {
                                let display = if key_code_empty {
                                    "(none)".to_string()
                                } else {
                                    project.hotkey_bindings[i].key_code.clone()
                                };
                                ui.label(&display);
                                if ui.button("Capture Key").clicked() {
                                    panel_state.capturing_key_for = Some(i);
                                }
                            }
                        });

                        // Validation warning for empty key_code
                        if project.hotkey_bindings[i].key_code.is_empty() {
                            ui.colored_label(
                                egui::Color32::from_rgb(255, 180, 0),
                                "⚠ Key code is empty",
                            );
                        }

                        // Name input
                        ui.horizontal(|ui| {
                            ui.label("Name:");
                            let response =
                                ui.text_edit_singleline(&mut project.hotkey_bindings[i].name);
                            if response.changed() {
                                // Enforce 64-char limit
                                if project.hotkey_bindings[i].name.len() > 64 {
                                    project.hotkey_bindings[i].name.truncate(64);
                                }
                                project.has_unsaved_hotkey_changes = true;
                            }
                        });

                        // Validation warning for empty name
                        if project.hotkey_bindings[i].name.is_empty() {
                            ui.colored_label(
                                egui::Color32::from_rgb(255, 180, 0),
                                "⚠ Name is empty",
                            );
                        }

                        // Event actions editor
                        ui.separator();
                        ui.label("Event Actions:");

                        let editor_state = &mut panel_state.action_editors[i];
                        let event_actions = &mut project.hotkey_bindings[i].event_actions;

                        action_editor_ui::render_action_editor(
                            ui,
                            event_actions,
                            editor_state,
                            &format!("hotkey_binding_{}_events", i),
                            &[], // map_entries
                            &portrait_entries,
                            0,    // depth
                            None, // reward_ctx
                            &[],  // shops
                        );
                    });

                    ui.add_space(8.0);
                });
            }
        });

    // Apply deferred removal
    if let Some(idx) = remove_index {
        project.hotkey_bindings.remove(idx);
        panel_state.action_editors.remove(idx);
        if panel_state.capturing_key_for == Some(idx) {
            panel_state.capturing_key_for = None;
        } else if let Some(cap) = panel_state.capturing_key_for
            && cap > idx
        {
            panel_state.capturing_key_for = Some(cap - 1);
        }
        project.has_unsaved_hotkey_changes = true;
    }

    // Apply deferred swap
    if let Some((a, b)) = swap {
        project.hotkey_bindings.swap(a, b);
        panel_state.action_editors.swap(a, b);
        if panel_state.capturing_key_for == Some(a) {
            panel_state.capturing_key_for = Some(b);
        } else if panel_state.capturing_key_for == Some(b) {
            panel_state.capturing_key_for = Some(a);
        }
        project.has_unsaved_hotkey_changes = true;
    }
}
