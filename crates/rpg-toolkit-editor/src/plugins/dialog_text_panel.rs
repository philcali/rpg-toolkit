use std::collections::HashMap;

use bevy::prelude::*;
use bevy_egui::egui;

use rpg_toolkit_common::{DialogTextData, EventAction, MapData};

use crate::data::editor_state::{EditCommand, EditCommandKind};
use crate::data::map::MapId;
use crate::data::project::Project;
use crate::plugins::attribute::truncate_preview;

/// Plugin for the Dialog Text Management panel.
pub struct DialogTextPanelPlugin;

impl Plugin for DialogTextPanelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DialogTextPanelState>();
    }
}

/// State for the Dialog Text Panel.
#[derive(Resource, Default)]
pub struct DialogTextPanelState {
    /// Currently selected entry for viewing/editing.
    pub selected_text_id: Option<String>,
    /// Pending navigation to a map (set by find-usages click, consumed by layer_panel_ui).
    pub pending_navigation: Option<MapId>,
    /// Modal dialog state for add/edit.
    pub modal_open: bool,
    /// The text ID field in the modal (read-only when editing).
    pub modal_text_id: String,
    /// The text content field in the modal.
    pub modal_text_content: String,
    /// If Some, we're editing an existing entry; if None, we're adding a new one.
    pub modal_editing_id: Option<String>,
}

/// A single usage of a Text_Id in the project.
#[derive(Clone, Debug, PartialEq)]
pub struct TextIdUsage {
    pub map_id: MapId,
    pub map_name: String,
    pub layer_index: usize,
    pub x: u32,
    pub y: u32,
}

/// Reverse index mapping Text_Id → list of tiles that reference it via ShowDialog actions.
/// Runtime-only (not persisted). Rebuilt on project load, incrementally updated on edits.
#[derive(Resource, Default, Clone, Debug)]
pub struct TextIdIndex {
    pub index: HashMap<String, Vec<TextIdUsage>>,
}

impl TextIdIndex {
    /// Returns the usages for a given Text_Id, or an empty slice if none.
    pub fn get(&self, text_id: &str) -> &[TextIdUsage] {
        self.index.get(text_id).map(|v| v.as_slice()).unwrap_or(&[])
    }
}

/// Builds the complete reverse index by scanning all maps.
/// Called once on project load and when a full rebuild is needed.
pub fn rebuild_text_id_index(maps: &HashMap<MapId, MapData>) -> TextIdIndex {
    let mut index: HashMap<String, Vec<TextIdUsage>> = HashMap::new();
    for (map_id, map) in maps {
        for (layer_index, layer) in map.layers.iter().enumerate() {
            for (y, row) in layer.attributes.cells.iter().enumerate() {
                for (x, attrs) in row.iter().enumerate() {
                    for action in &attrs.event_trigger {
                        if let EventAction::ShowDialog { text, .. } = action
                            && let DialogTextData::Id(id) = text
                        {
                            index.entry(id.clone()).or_default().push(TextIdUsage {
                                map_id: map_id.clone(),
                                map_name: map.name.clone(),
                                layer_index,
                                x: x as u32,
                                y: y as u32,
                            });
                        }
                    }
                }
            }
        }
    }
    TextIdIndex { index }
}

/// Incrementally updates the reverse index when a single tile's event triggers change.
/// Removes old entries for the tile, then adds new entries based on the new trigger list.
/// Called when a SetEventTrigger EditCommand is applied or undone.
#[allow(clippy::too_many_arguments)]
pub fn update_text_id_index_for_tile(
    index: &mut TextIdIndex,
    map_id: &MapId,
    map_name: &str,
    layer_index: usize,
    x: u32,
    y: u32,
    old_triggers: &[EventAction],
    new_triggers: &[EventAction],
) {
    // Remove old entries for this tile
    for action in old_triggers {
        if let EventAction::ShowDialog {
            text: DialogTextData::Id(id),
            ..
        } = action
            && let Some(usages) = index.index.get_mut(id)
        {
            usages.retain(|u| {
                !(u.map_id == *map_id && u.layer_index == layer_index && u.x == x && u.y == y)
            });
            if usages.is_empty() {
                index.index.remove(id);
            }
        }
    }
    // Add new entries for this tile
    for action in new_triggers {
        if let EventAction::ShowDialog {
            text: DialogTextData::Id(id),
            ..
        } = action
        {
            index
                .index
                .entry(id.clone())
                .or_default()
                .push(TextIdUsage {
                    map_id: map_id.clone(),
                    map_name: map_name.to_string(),
                    layer_index,
                    x,
                    y,
                });
        }
    }
}

/// Deferred action from the dialog text panel UI.
pub enum DialogTextAction {
    Insert {
        text_id: String,
        text: String,
    },
    Update {
        text_id: String,
        new_text: String,
    },
    Remove {
        text_id: String,
    },
    OpenAddModal,
    OpenEditModal {
        text_id: String,
        current_text: String,
    },
    CloseModal,
    Select(String),
    Deselect,
    NavigateToUsage {
        map_id: MapId,
    },
}

/// Renders the Dialog Text Panel section inside the left side panel.
/// Called from `layer_panel_ui` after the Map Browser section.
pub fn render_dialog_text_panel(
    ui: &mut egui::Ui,
    project: &Project,
    state: &mut DialogTextPanelState,
    edit_events: &mut MessageWriter<EditCommand>,
    text_id_index: &TextIdIndex,
) {
    let mut actions: Vec<DialogTextAction> = Vec::new();

    ui.heading("Dialog Texts");
    ui.separator();

    // Sorted entries for consistent display
    let mut entries: Vec<(String, String)> = project
        .dialog_texts
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    entries.sort_by(|a, b| a.0.cmp(&b.0));

    if entries.is_empty() {
        ui.label("No dialog texts.");
    } else {
        egui::ScrollArea::vertical()
            .id_salt("dialog_texts_scroll")
            .max_height(200.0)
            .show(ui, |ui| {
                for (text_id, text_content) in &entries {
                    ui.horizontal(|ui| {
                        let preview = truncate_preview(text_content, 40);
                        let label_text = format!("{} — {}", text_id, preview);

                        let is_selected = state.selected_text_id.as_ref() == Some(text_id);
                        let response = ui.selectable_label(is_selected, &label_text);
                        if response.clicked() {
                            if is_selected {
                                actions.push(DialogTextAction::Deselect);
                            } else {
                                actions.push(DialogTextAction::Select(text_id.clone()));
                            }
                        }

                        if ui.small_button("✏").clicked() {
                            actions.push(DialogTextAction::OpenEditModal {
                                text_id: text_id.clone(),
                                current_text: text_content.clone(),
                            });
                        }
                        if ui.small_button("✕").clicked() {
                            actions.push(DialogTextAction::Remove {
                                text_id: text_id.clone(),
                            });
                        }
                    });
                }
            });
    }

    ui.add_space(4.0);

    // "Add" button opens the modal
    if ui.button("Add…").clicked() {
        actions.push(DialogTextAction::OpenAddModal);
    }

    ui.separator();

    // Find-usages display for selected entry
    if let Some(ref selected_id) = state.selected_text_id {
        ui.label(
            egui::RichText::new(format!("Usages of \"{}\":", selected_id))
                .strong()
                .small(),
        );
        let usages = text_id_index.get(selected_id);
        if usages.is_empty() {
            ui.label(egui::RichText::new("No usages found.").weak().small());
        } else {
            egui::ScrollArea::vertical()
                .id_salt("text_id_usages_scroll")
                .max_height(100.0)
                .show(ui, |ui| {
                    for usage in usages {
                        let label = format!(
                            "{} — Layer {}, ({}, {})",
                            usage.map_name, usage.layer_index, usage.x, usage.y
                        );
                        if ui
                            .small_button(&label)
                            .on_hover_text("Click to navigate to this tile")
                            .clicked()
                        {
                            actions.push(DialogTextAction::NavigateToUsage {
                                map_id: usage.map_id.clone(),
                            });
                        }
                    }
                });
        }
        ui.separator();
    }

    // Apply deferred actions
    for action in actions {
        match action {
            DialogTextAction::Insert { text_id, text } => {
                edit_events.write(EditCommand {
                    kind: EditCommandKind::InsertDialogText {
                        text_id: text_id.clone(),
                        text,
                    },
                });
                state.modal_open = false;
                state.modal_text_id.clear();
                state.modal_text_content.clear();
                state.modal_editing_id = None;
            }
            DialogTextAction::Update { text_id, new_text } => {
                if let Some(old_text) = project.dialog_texts.get(&text_id) {
                    edit_events.write(EditCommand {
                        kind: EditCommandKind::UpdateDialogText {
                            text_id,
                            old_text: old_text.clone(),
                            new_text,
                        },
                    });
                }
                state.modal_open = false;
                state.modal_text_id.clear();
                state.modal_text_content.clear();
                state.modal_editing_id = None;
            }
            DialogTextAction::Remove { text_id } => {
                if let Some(old_text) = project.dialog_texts.get(&text_id) {
                    edit_events.write(EditCommand {
                        kind: EditCommandKind::RemoveDialogText {
                            text_id: text_id.clone(),
                            old_text: old_text.clone(),
                        },
                    });
                }
                // Clear selection if the removed entry was selected
                if state.selected_text_id.as_ref() == Some(&text_id) {
                    state.selected_text_id = None;
                }
            }
            DialogTextAction::OpenAddModal => {
                state.modal_open = true;
                state.modal_text_id.clear();
                state.modal_text_content.clear();
                state.modal_editing_id = None;
            }
            DialogTextAction::OpenEditModal {
                text_id,
                current_text,
            } => {
                state.modal_open = true;
                state.modal_text_id = text_id.clone();
                state.modal_text_content = current_text;
                state.modal_editing_id = Some(text_id);
            }
            DialogTextAction::CloseModal => {
                state.modal_open = false;
                state.modal_text_id.clear();
                state.modal_text_content.clear();
                state.modal_editing_id = None;
            }
            DialogTextAction::Select(text_id) => {
                state.selected_text_id = Some(text_id);
            }
            DialogTextAction::Deselect => {
                state.selected_text_id = None;
            }
            DialogTextAction::NavigateToUsage { map_id } => {
                state.pending_navigation = Some(map_id);
            }
        }
    }
}

/// Renders the Dialog Text add/edit modal window.
/// Called from the egui system that has access to the egui context.
pub fn render_dialog_text_modal(
    ctx: &egui::Context,
    project: &Project,
    state: &mut DialogTextPanelState,
    edit_events: &mut MessageWriter<EditCommand>,
) {
    if !state.modal_open {
        return;
    }

    let is_editing = state.modal_editing_id.is_some();
    let title = if is_editing {
        "Edit Dialog Text"
    } else {
        "Add Dialog Text"
    };

    let mut action: Option<DialogTextAction> = None;

    egui::Window::new(title)
        .collapsible(false)
        .resizable(true)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Text ID:");
                if is_editing {
                    // Read-only when editing
                    ui.label(egui::RichText::new(&state.modal_text_id).strong());
                } else {
                    ui.text_edit_singleline(&mut state.modal_text_id);
                }
            });

            ui.label("Text:");
            ui.text_edit_multiline(&mut state.modal_text_content);

            // Validation
            if !is_editing {
                let id_exists = !state.modal_text_id.is_empty()
                    && project.dialog_texts.contains_key(&state.modal_text_id);
                if id_exists {
                    ui.colored_label(
                        egui::Color32::from_rgb(255, 180, 0),
                        "⚠ Text ID already exists",
                    );
                }
            }

            ui.separator();
            ui.horizontal(|ui| {
                if is_editing {
                    let can_save = !state.modal_text_content.is_empty();
                    if ui
                        .add_enabled(can_save, egui::Button::new("Save"))
                        .clicked()
                    {
                        action = Some(DialogTextAction::Update {
                            text_id: state.modal_text_id.clone(),
                            new_text: state.modal_text_content.clone(),
                        });
                    }
                } else {
                    let id_exists = !state.modal_text_id.is_empty()
                        && project.dialog_texts.contains_key(&state.modal_text_id);
                    let can_add = !state.modal_text_id.is_empty()
                        && !state.modal_text_content.is_empty()
                        && !id_exists;
                    if ui.add_enabled(can_add, egui::Button::new("Add")).clicked() {
                        action = Some(DialogTextAction::Insert {
                            text_id: state.modal_text_id.clone(),
                            text: state.modal_text_content.clone(),
                        });
                    }
                }
                if ui.button("Cancel").clicked() {
                    action = Some(DialogTextAction::CloseModal);
                }
            });
        });

    // Apply action outside the closure
    if let Some(action) = action {
        match action {
            DialogTextAction::Insert { text_id, text } => {
                edit_events.write(EditCommand {
                    kind: EditCommandKind::InsertDialogText {
                        text_id: text_id.clone(),
                        text,
                    },
                });
                state.modal_open = false;
                state.modal_text_id.clear();
                state.modal_text_content.clear();
                state.modal_editing_id = None;
            }
            DialogTextAction::Update { text_id, new_text } => {
                if let Some(old_text) = project.dialog_texts.get(&text_id) {
                    edit_events.write(EditCommand {
                        kind: EditCommandKind::UpdateDialogText {
                            text_id,
                            old_text: old_text.clone(),
                            new_text,
                        },
                    });
                }
                state.modal_open = false;
                state.modal_text_id.clear();
                state.modal_text_content.clear();
                state.modal_editing_id = None;
            }
            DialogTextAction::CloseModal => {
                state.modal_open = false;
                state.modal_text_id.clear();
                state.modal_text_content.clear();
                state.modal_editing_id = None;
            }
            _ => {}
        }
    }
}
