use bevy::prelude::*;
use rpg_toolkit_common::FacingDirection;

use crate::components::{PlayerCharacter, PlayerSpriteState};
use crate::events::PlayerMoved;
use crate::resources::{
    ActionQueue, JumpAnimState, RendererProjectData, RendererState, WaitingFor,
};
use crate::systems::player::grid_to_world;

/// Computes the landing tile for a jump given the player's current position,
/// facing direction, and jump distance. Clamps to map bounds.
pub fn compute_landing(
    grid_x: u32,
    grid_y: u32,
    facing: FacingDirection,
    distance: u32,
    map_width: u32,
    map_height: u32,
) -> (u32, u32) {
    let (dx, dy): (i32, i32) = match facing {
        FacingDirection::Up => (0, -(distance as i32)),
        FacingDirection::Down => (0, distance as i32),
        FacingDirection::Left => (-(distance as i32), 0),
        FacingDirection::Right => (distance as i32, 0),
    };
    let new_x = (grid_x as i32 + dx).clamp(0, map_width as i32 - 1) as u32;
    let new_y = (grid_y as i32 + dy).clamp(0, map_height as i32 - 1) as u32;
    (new_x, new_y)
}

/// Computes the vertical arc offset for a jump animation at progress `t`.
///
/// Returns a positive offset (upward in screen space) that follows a parabolic
/// curve: 0 at t=0, peak at t=0.5, 0 at t=1.
/// For distance 0 (jump in place), uses a fixed small hop height.
pub fn jump_arc_offset(t: f32, distance: u32, tile_height: f32) -> f32 {
    let peak = if distance == 0 {
        tile_height * 0.5
    } else {
        tile_height * (distance as f32) * 0.5
    };
    4.0 * peak * t * (1.0 - t)
}

/// Bevy system that animates the player during a jump.
///
/// Each frame: advances elapsed time, computes progress `t`, interpolates position
/// from start to landing, and applies the parabolic vertical offset.
///
/// On completion: updates PlayerCharacter grid position to landing tile,
/// removes JumpAnimState resource, resets ActionQueue waiting state, pops the
/// action, and fires a PlayerMoved event to trigger landing tile events.
#[allow(clippy::too_many_arguments)]
pub fn jump_animation_system(
    mut commands: Commands,
    time: Res<Time>,
    jump_state: Option<ResMut<JumpAnimState>>,
    mut action_queue: Option<ResMut<ActionQueue>>,
    project_data: Option<Res<RendererProjectData>>,
    renderer_state: Res<RendererState>,
    mut query: Query<(
        &mut PlayerCharacter,
        &mut Transform,
        Option<&PlayerSpriteState>,
    )>,
    mut player_moved: MessageWriter<PlayerMoved>,
) {
    let Some(mut state) = jump_state else {
        return;
    };

    let Some(ref pd) = project_data else {
        return;
    };

    let Some(map_id) = &renderer_state.active_map_id else {
        return;
    };

    let Some(map) = pd.project_file.maps.get(map_id) else {
        return;
    };

    let tile_width = map.tile_width;
    let tile_height = map.tile_height;

    state.elapsed += time.delta_secs();
    // Guard against a zero/near-zero duration, which would produce a NaN or
    // infinite `t` and complete the jump in a single frame with no arc.
    let t = if state.duration > f32::EPSILON {
        (state.elapsed / state.duration).clamp(0.0, 1.0)
    } else {
        1.0
    };

    let start_world = grid_to_world(state.start_x, state.start_y, tile_width, tile_height);
    let landing_world = grid_to_world(state.landing_x, state.landing_y, tile_width, tile_height);

    // Interpolate horizontal/vertical position
    let pos = start_world.lerp(landing_world, t);

    // Compute parabolic arc offset (positive = upward in world space)
    let arc_offset = jump_arc_offset(t, state.distance, tile_height as f32);

    for (mut player, mut transform, sprite_state) in query.iter_mut() {
        let y_offset = sprite_state.map_or(0.0, |ss| ss.y_offset);

        transform.translation.x = pos.x;
        transform.translation.y = pos.y + y_offset + arc_offset;

        // Check for completion
        if state.elapsed >= state.duration {
            // Snap to final position
            transform.translation.x = landing_world.x;
            transform.translation.y = landing_world.y + y_offset;

            let from_grid = (player.grid_x, player.grid_y);
            let to_grid = (state.landing_x, state.landing_y);

            // Update player grid position
            player.grid_x = state.landing_x;
            player.grid_y = state.landing_y;

            // Remove JumpAnimState resource
            commands.remove_resource::<JumpAnimState>();

            // Unblock the action queue and pop the Jump action
            if let Some(ref mut queue) = action_queue {
                queue.waiting_for = WaitingFor::Nothing;
                queue.actions.pop_front();
            }

            // Fire PlayerMoved event to trigger landing tile events
            player_moved.write(PlayerMoved {
                from: from_grid,
                to: to_grid,
            });
        }
    }
}
