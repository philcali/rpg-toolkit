use bevy::prelude::*;

pub mod components;
pub mod dialog;
pub mod events;
pub mod input;
pub mod resources;
pub mod systems;

pub use components::{
    GameCamera, MoveAnimation, NpcSprite, PlayerCharacter, PlayerSpriteState, RendererTileSprite,
};
pub use events::{MapChanged, PlayerMoved, ShowDialog};
pub use input::{Direction, MovementIntent, read_input};
pub use resources::{
    AnimationConfig, MovementConfig, PixelScaleConfig, PixelScaleMode, PlayerVisual,
    RendererProjectData, RendererState,
};
pub use systems::camera::{apply_pixel_scale, compute_zoom_to_fit, spawn_camera, update_camera};
pub use systems::collision::is_tile_blocked;
pub use systems::map_render::{spawn_npc_sprites, sync_map_sprites};
pub use systems::player::{
    animate_player, animate_player_sprite, grid_to_world, player_movement, spawn_player,
};
pub use systems::spritesheet::{build_spritesheet_atlas, load_spritesheet_assets};
pub use systems::triggers::{check_triggers, handle_map_change};

pub use dialog::{
    DialogBox, DialogConfig, DialogPosition, DialogState, DialogText, DialogTextNode,
    DialogTextRegistry, compute_visible_chars,
};
pub use systems::dialog::{handle_dialog_event, handle_dialog_input, update_dialog_typewriter};

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
            .init_resource::<AnimationConfig>()
            .init_resource::<PixelScaleConfig>()
            .init_resource::<DialogTextRegistry>()
            // Events
            .add_message::<MapChanged>()
            .add_message::<PlayerMoved>()
            .add_message::<ShowDialog>()
            // Startup systems
            .add_systems(
                Startup,
                (
                    load_spritesheet_assets,
                    spawn_player,
                    spawn_camera,
                    fire_initial_map_changed,
                )
                    .chain(),
            )
            // Update systems with explicit ordering
            .add_systems(
                Update,
                (
                    read_input,
                    player_movement.after(read_input),
                    animate_player.after(player_movement),
                    animate_player_sprite.after(animate_player),
                    check_triggers.after(animate_player),
                    handle_map_change.after(check_triggers),
                    sync_map_sprites.after(handle_map_change),
                    spawn_npc_sprites.after(sync_map_sprites),
                    apply_pixel_scale.after(spawn_npc_sprites),
                    update_camera.after(apply_pixel_scale),
                    // Dialog systems
                    handle_dialog_event,
                    update_dialog_typewriter.after(handle_dialog_event),
                    handle_dialog_input.after(update_dialog_typewriter),
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
