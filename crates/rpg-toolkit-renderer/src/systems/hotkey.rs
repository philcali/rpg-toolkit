use bevy::prelude::*;
use std::collections::VecDeque;

use crate::dialog::DialogState;
use crate::resources::{ActionQueue, RendererProjectData};
use crate::systems::selection::SelectionState;

/// System that checks keyboard input against configured hotkey bindings and
/// fires the matching binding's event actions into a new `ActionQueue`.
///
/// Guard conditions (checked via system parameters):
/// - Only runs when `AppPhase` is `InGame` (enforced by `.run_if(in_state(AppPhase::InGame))` in plugin setup)
/// - Returns early if an `ActionQueue` resource is already present
/// - Returns early if a `DialogState` resource is present
/// - Returns early if a `SelectionState` resource is present
///
/// On match: pushes the first matching binding's `event_actions` into a new `ActionQueue`.
/// If `event_actions` is empty, treats as no-op.
pub fn hotkey_input_system(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    project_data: Res<RendererProjectData>,
    action_queue: Option<Res<ActionQueue>>,
    dialog_state: Option<Res<DialogState>>,
    selection_state: Option<Res<SelectionState>>,
) {
    // Guard: do nothing if any blocking state is active
    if action_queue.is_some() || dialog_state.is_some() || selection_state.is_some() {
        return;
    }

    let bindings = &project_data.project_file.hotkey_bindings;
    if bindings.is_empty() {
        return;
    }

    // Check each just-pressed key against configured bindings (first match wins)
    for key in keyboard.get_just_pressed() {
        let key_name = keycode_to_string(*key);

        for binding in bindings {
            if binding.key_code == key_name {
                // Found a match — if event_actions is non-empty, create an ActionQueue
                if !binding.event_actions.is_empty() {
                    commands.insert_resource(ActionQueue {
                        actions: VecDeque::from(binding.event_actions.clone()),
                        waiting_for: crate::resources::WaitingFor::Nothing,
                    });
                }
                // First match wins — return regardless of whether actions were empty
                return;
            }
        }
    }
}

/// Converts a Bevy `KeyCode` to its string representation matching
/// what's stored in `HotkeyBinding.key_code` (e.g., "ShiftLeft", "KeyZ", "Space").
///
/// Uses the Debug trait format which produces the enum variant name.
fn keycode_to_string(key: KeyCode) -> String {
    format!("{:?}", key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keycode_to_string_produces_variant_names() {
        assert_eq!(keycode_to_string(KeyCode::ShiftLeft), "ShiftLeft");
        assert_eq!(keycode_to_string(KeyCode::KeyZ), "KeyZ");
        assert_eq!(keycode_to_string(KeyCode::Space), "Space");
        assert_eq!(keycode_to_string(KeyCode::Escape), "Escape");
        assert_eq!(keycode_to_string(KeyCode::Enter), "Enter");
        assert_eq!(keycode_to_string(KeyCode::ArrowUp), "ArrowUp");
    }
}
