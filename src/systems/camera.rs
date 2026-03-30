use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::data::EditorState;
use crate::plugins::canvas::EditorCamera;

/// Zoom speed factor applied to each scroll tick.
const ZOOM_SPEED: f32 = 0.1;

/// System that handles mouse-wheel zoom centered on the cursor position.
///
/// When the user scrolls, the zoom level changes and the camera offset is
/// adjusted so the world point under the cursor stays fixed on screen.
pub fn zoom_system(
    mut scroll_events: MessageReader<MouseWheel>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut editor: ResMut<EditorState>,
) {
    let scroll_y: f32 = scroll_events.read().map(|e| e.y).sum();
    if scroll_y == 0.0 {
        return;
    }

    let Ok(window) = windows.single() else { return };
    let Some(cursor_screen) = window.cursor_position() else {
        return;
    };

    let old_zoom = editor.zoom_level;
    let new_zoom = (old_zoom * (1.0 + scroll_y * ZOOM_SPEED)).clamp(0.25, 8.0);
    editor.set_zoom(new_zoom);

    // The viewport center in screen coords
    let vp_center = Vec2::new(window.width() / 2.0, window.height() / 2.0);

    // Screen offset from center (Bevy's camera is centered on the viewport)
    // Bevy Y is flipped: screen Y goes down, world Y goes up
    let screen_offset = cursor_screen - vp_center;
    let world_offset = Vec2::new(screen_offset.x, -screen_offset.y);

    // Compute world position under cursor using our own offset/zoom state
    // (not the camera's GlobalTransform, which may be stale this frame).
    // world = camera_offset + screen_offset_from_center / zoom
    let world_before = editor.camera_offset + world_offset / old_zoom;
    let world_after = editor.camera_offset + world_offset / new_zoom;

    // Shift offset so the world point under the cursor stays fixed
    editor.camera_offset += world_before - world_after;
}

/// Resource tracking middle-mouse pan state.
#[derive(Resource, Default)]
pub struct PanState {
    /// Whether the middle mouse button is currently held.
    pub is_panning: bool,
    /// Last known cursor screen position during a pan drag.
    pub last_cursor_pos: Option<Vec2>,
}

/// System that handles middle-mouse-button drag for panning.
pub fn pan_system(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut pan_state: ResMut<PanState>,
    mut editor: ResMut<EditorState>,
) {
    if mouse.just_pressed(MouseButton::Middle) {
        pan_state.is_panning = true;
        let cursor = windows.single().ok().and_then(|w| w.cursor_position());
        pan_state.last_cursor_pos = cursor;
        return;
    }

    if mouse.just_released(MouseButton::Middle) {
        pan_state.is_panning = false;
        pan_state.last_cursor_pos = None;
        return;
    }

    if !pan_state.is_panning {
        return;
    }

    let Ok(window) = windows.single() else { return };
    let Some(current_pos) = window.cursor_position() else {
        return;
    };
    let Some(last_pos) = pan_state.last_cursor_pos else {
        pan_state.last_cursor_pos = Some(current_pos);
        return;
    };

    let screen_delta = current_pos - last_pos;
    if screen_delta == Vec2::ZERO {
        return;
    }

    let zoom = editor.zoom_level;
    // Convert screen delta to world delta.
    // Screen X right = world X right, screen Y down = world Y up (negate Y).
    // Panning: dragging right should move the view right, meaning the camera
    // moves LEFT (offset decreases in X). So we subtract.
    let world_delta = Vec2::new(-screen_delta.x, screen_delta.y) / zoom;
    editor.camera_offset += world_delta;

    pan_state.last_cursor_pos = Some(current_pos);
}

/// System that applies the camera transform from `EditorState`.
///
/// This is the single authoritative system that writes to the camera's
/// `Transform` based on `EditorState.zoom_level` and `EditorState.camera_offset`.
pub fn apply_camera_transform(
    editor: Res<EditorState>,
    mut camera_q: Query<&mut Transform, With<EditorCamera>>,
) {
    if let Ok(mut transform) = camera_q.single_mut() {
        transform.scale = Vec3::splat(1.0 / editor.zoom_level);
        transform.translation = Vec3::new(editor.camera_offset.x, editor.camera_offset.y, 0.0);
    }
}
