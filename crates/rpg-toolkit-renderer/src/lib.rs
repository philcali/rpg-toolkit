use bevy::prelude::*;

pub mod components;
pub mod events;
pub mod input;
pub mod resources;
pub mod systems;

pub use components::{GameCamera, MoveAnimation, PlayerCharacter, RendererTileSprite};
pub use events::{MapChanged, PlayerMoved};
pub use input::{Direction, MovementIntent, read_input};
pub use resources::{MovementConfig, PlayerVisual, RendererProjectData, RendererState};
pub use systems::camera::{spawn_camera, update_camera};
pub use systems::collision::is_tile_blocked;
pub use systems::map_render::sync_map_sprites;
pub use systems::player::{animate_player, grid_to_world, player_movement, spawn_player};
pub use systems::triggers::{check_triggers, handle_map_change};

/// The renderer plugin that renders a loaded project as a playable game world.
pub struct ProjectRendererPlugin;

impl Plugin for ProjectRendererPlugin {
    fn build(&self, app: &mut App) {
        app
            // Resources
            .init_resource::<RendererState>()
            .init_resource::<MovementConfig>()
            .init_resource::<PlayerVisual>()
            .init_resource::<MovementIntent>()
            // Events
            .add_message::<MapChanged>()
            .add_message::<PlayerMoved>()
            // Startup systems
            .add_systems(
                Startup,
                (spawn_player, spawn_camera, fire_initial_map_changed).chain(),
            )
            // Update systems with explicit ordering
            .add_systems(
                Update,
                (
                    read_input,
                    player_movement.after(read_input),
                    animate_player.after(player_movement),
                    check_triggers.after(animate_player),
                    handle_map_change.after(check_triggers),
                    sync_map_sprites.after(handle_map_change),
                    update_camera.after(sync_map_sprites),
                ),
            );
    }
}

/// Startup system that fires the initial `MapChanged` event so `sync_map_sprites`
/// renders the first map on the first frame.
fn fire_initial_map_changed(
    renderer_state: Res<RendererState>,
    mut map_changed: MessageWriter<MapChanged>,
) {
    if let Some(map_id) = &renderer_state.active_map_id {
        map_changed.write(MapChanged {
            previous_map_id: None,
            new_map_id: map_id.clone(),
        });
    }
}
