use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

use crate::data::EditorTool;

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
            .add_systems(EguiPrimaryContextPass, toolbar_ui)
            .add_systems(Update, tool_hotkeys);
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
    canvas_rect: Res<CanvasRect>,
) -> Result {
    let ctx = contexts.ctx_mut()?;

    let offset = [canvas_rect.left, canvas_rect.top];

    egui::Window::new("toolbar")
        .title_bar(false)
        .resizable(false)
        .movable(false)
        .anchor(egui::Align2::LEFT_TOP, offset)
        .show(ctx, |ui| {
            ui.vertical(|ui| {
                for &(tool, icon, key, _) in TOOLS {
                    let resp = ui.selectable_label(*active_tool == tool, icon);
                    if resp.clicked() {
                        *active_tool = tool;
                    }
                    resp.on_hover_text(format!("{tool:?} ({key})"));
                }
            });
        });

    Ok(())
}

/// Switch the active tool via keyboard shortcuts (B, E, G, H, S).
/// Only fires when egui is not capturing keyboard input.
fn tool_hotkeys(
    keys: Res<ButtonInput<KeyCode>>,
    mut active_tool: ResMut<EditorTool>,
    mut contexts: EguiContexts,
) {
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
