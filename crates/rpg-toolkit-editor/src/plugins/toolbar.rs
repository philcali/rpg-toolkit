use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

use crate::data::{AppEditorMode, AttributeTool, EditorMode, EditorState, EditorTool, EditorUiSet};

/// Resource storing the canvas area bounds for toolbar positioning and input gating.
/// Written by the app shell after panels are laid out.
#[derive(Resource, Default)]
pub struct CanvasRect {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

pub struct ToolbarPlugin;

impl Plugin for ToolbarPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EditorTool>()
            .init_resource::<CanvasRect>()
            .add_systems(
                EguiPrimaryContextPass,
                toolbar_ui
                    .in_set(EditorUiSet::Overlay)
                    .run_if(resource_equals(AppEditorMode::MapEditor)),
            )
            .add_systems(
                Update,
                tool_hotkeys.run_if(resource_equals(AppEditorMode::MapEditor)),
            );
    }
}

/// Tool definitions with icon and keyboard shortcut.
const TOOLS: &[(EditorTool, &str, &str, KeyCode)] = &[
    (EditorTool::Paint, "✏", "B", KeyCode::KeyB),
    (EditorTool::Erase, "🗑", "E", KeyCode::KeyE),
    (EditorTool::FloodFill, "🎨", "G", KeyCode::KeyG),
    (EditorTool::Pan, "✋", "H", KeyCode::KeyH),
    (EditorTool::StampBrush, "⊞", "S", KeyCode::KeyS),
];

fn toolbar_ui(
    mut contexts: EguiContexts,
    mut active_tool: ResMut<EditorTool>,
    mut editor_state: ResMut<EditorState>,
    mut canvas_rect: ResMut<CanvasRect>,
) -> Result {
    let ctx = contexts.ctx_mut()?;

    // Compute canvas rect now that all panels (top, left, right) have been laid out.
    let avail = ctx.available_rect();
    canvas_rect.left = avail.left();
    canvas_rect.top = avail.top();
    canvas_rect.right = avail.right();
    canvas_rect.bottom = avail.bottom();

    let offset = [canvas_rect.left, canvas_rect.top];

    egui::Window::new("toolbar")
        .title_bar(false)
        .resizable(false)
        .movable(false)
        .fixed_size(egui::vec2(32.0, 0.0))
        .anchor(egui::Align2::LEFT_TOP, offset)
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                // Mode toggle button
                let mode_label = match editor_state.editor_mode {
                    EditorMode::Paint => "🎨",
                    EditorMode::Attribute => "⚙",
                };
                let resp = ui
                    .button(mode_label)
                    .on_hover_text(match editor_state.editor_mode {
                        EditorMode::Paint => "Switch to Attribute mode",
                        EditorMode::Attribute => "Switch to Paint mode",
                    });
                if resp.clicked() {
                    match editor_state.editor_mode {
                        EditorMode::Paint => {
                            // Switch to Attribute mode: save current tool
                            editor_state.previous_tool = Some(*active_tool);
                            editor_state.editor_mode = EditorMode::Attribute;
                        }
                        EditorMode::Attribute => {
                            // Switch back to Paint mode: restore previous tool
                            if let Some(prev) = editor_state.previous_tool.take() {
                                *active_tool = prev;
                            }
                            editor_state.editor_mode = EditorMode::Paint;
                        }
                    }
                }

                ui.separator();

                match editor_state.editor_mode {
                    EditorMode::Paint => {
                        // Show paint tools
                        for &(tool, icon, key, _) in TOOLS {
                            let resp = ui.selectable_label(*active_tool == tool, icon);
                            if resp.clicked() {
                                *active_tool = tool;
                            }
                            resp.on_hover_text(format!("{tool:?} ({key})"));
                        }
                    }
                    EditorMode::Attribute => {
                        // Show attribute tools
                        let attr_tools: &[(AttributeTool, &str, &str)] = &[
                            (AttributeTool::Opacity, "🔒", "Opacity"),
                            (AttributeTool::EventTrigger, "⚡", "Event Trigger"),
                            (AttributeTool::SpawnPoint, "📍", "Spawn Point"),
                            (AttributeTool::NpcPlacement, "👤", "NPC Placement"),
                            (AttributeTool::Elevation, "⬆", "Elevation"),
                            (
                                AttributeTool::ElevationTransition,
                                "🔀",
                                "Elevation Transition",
                            ),
                        ];

                        for &(tool, icon, label) in attr_tools {
                            let resp =
                                ui.selectable_label(editor_state.attribute_tool == tool, icon);
                            if resp.clicked() {
                                editor_state.attribute_tool = tool;
                            }
                            resp.on_hover_text(label);
                        }
                    }
                }
            });
        });

    Ok(())
}

/// Switch the active tool via keyboard shortcuts (B, E, G, H, S).
/// Only fires when egui is not capturing keyboard input and in Paint mode.
fn tool_hotkeys(
    keys: Res<ButtonInput<KeyCode>>,
    mut active_tool: ResMut<EditorTool>,
    mut contexts: EguiContexts,
    editor_state: Res<EditorState>,
) {
    // Don't intercept keys when in attribute mode
    if editor_state.editor_mode == EditorMode::Attribute {
        return;
    }

    // Don't intercept keys when egui has a text field focused
    if let Ok(ctx) = contexts.ctx_mut()
        && ctx.wants_keyboard_input()
    {
        return;
    }

    for &(tool, _, _, key) in TOOLS {
        if keys.just_pressed(key) {
            *active_tool = tool;
            return;
        }
    }
}
