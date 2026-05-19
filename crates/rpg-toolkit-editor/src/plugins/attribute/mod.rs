//! Attribute editing plugin — coordinates overlay rendering, click handling,
//! and modal dialogs for opacity, event triggers, spawn points, and NPCs.

mod action_editor;
mod action_editor_forms;
mod action_editor_ui;
mod click;
mod event_trigger_dialog;
mod npc_dialog;
mod overlay;
mod spawn_point_dialog;

#[allow(unused_imports)]
pub use action_editor::{ActionEditorState, truncate_preview};
pub use event_trigger_dialog::EventTriggerDialog;
pub use npc_dialog::NpcPlacementDialog;
pub use spawn_point_dialog::SpawnPointConfirmDialog;

use bevy::prelude::*;
use bevy_egui::EguiPrimaryContextPass;

pub struct AttributePlugin;

impl Plugin for AttributePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SpawnPointConfirmDialog>()
            .init_resource::<EventTriggerDialog>()
            .init_resource::<NpcPlacementDialog>()
            .add_systems(
                EguiPrimaryContextPass,
                (
                    event_trigger_dialog::event_trigger_panel_ui,
                    spawn_point_dialog::spawn_point_confirm_ui,
                    npc_dialog::npc_placement_dialog_ui,
                ),
            )
            .add_systems(
                Update,
                (
                    overlay::attribute_overlay_system.after(crate::plugins::canvas::draw_grid),
                    click::attribute_click_system.after(crate::systems::input::update_cursor_state),
                ),
            );
    }
}
