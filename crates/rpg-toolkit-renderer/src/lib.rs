use bevy::prelude::*;

pub mod components;
pub mod dialog;
pub mod effects;
pub mod events;
pub mod input;
pub mod resources;
pub mod systems;

pub use components::{
    FadeOverlay, GameCamera, MoveAnimation, NpcMoveAnimation, NpcPatrolState, NpcSprite,
    NpcSpriteState, PlayerCharacter, PlayerSpriteState, RendererTileSprite,
};
pub use events::{MapChanged, PlayerMoved, ShowDialog};
pub use input::{Direction, MovementIntent, read_input};
pub use resources::{
    ActionQueue, AnimationConfig, FadeState, GameState, InteractionIntent, MovementConfig,
    NpcCollisionEvent, NpcPositions, PixelScaleConfig, PixelScaleMode, PlayerAppearanceState,
    PlayerVisual, RendererProjectData, RendererState, ScreenShakeState, WaitingFor,
};
pub use systems::camera::{apply_pixel_scale, compute_zoom_to_fit, spawn_camera, update_camera};
pub use systems::collision::is_tile_blocked;
pub use systems::map_render::{
    compute_tile_z, init_npc_positions, resort_tile_z_on_elevation_change, spawn_npc_sprites,
    sync_map_sprites,
};
pub use systems::npc::{
    npc_patrol_animation, npc_patrol_movement, npc_trigger_system, read_interaction_input,
};
pub use systems::player::{
    animate_player, animate_player_sprite, grid_to_world, player_movement, spawn_player,
};
pub use systems::spritesheet::{build_spritesheet_atlas, load_spritesheet_assets};
pub use systems::triggers::{
    advance_action_queue, check_triggers, fade_system, handle_map_change, screen_shake_system,
};

pub use effects::{
    compute_fade_opacity, compute_shake_offset, is_blocking_action, is_fade_complete,
    is_shake_complete,
};

pub use dialog::{
    DialogBox, DialogConfig, DialogPosition, DialogState, DialogText, DialogTextNode,
    DialogTextRegistry, compute_visible_chars, dialog_config_from_data, dialog_text_from_data,
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
            .init_resource::<NpcPositions>()
            .init_resource::<InteractionIntent>()
            .init_resource::<NpcCollisionEvent>()
            .init_resource::<GameState>()
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
                    read_interaction_input.after(read_input),
                    player_movement.after(read_input),
                    npc_patrol_movement.after(player_movement),
                    animate_player.after(player_movement),
                    animate_player_sprite.after(animate_player),
                    npc_patrol_animation.after(animate_player_sprite),
                    check_triggers.after(animate_player),
                    npc_trigger_system.after(check_triggers),
                    advance_action_queue.after(npc_trigger_system),
                    handle_map_change.after(advance_action_queue),
                    sync_map_sprites.after(handle_map_change),
                    spawn_npc_sprites.after(sync_map_sprites),
                    init_npc_positions.after(spawn_npc_sprites),
                    resort_tile_z_on_elevation_change.after(init_npc_positions),
                    apply_pixel_scale.after(resort_tile_z_on_elevation_change),
                    update_camera.after(apply_pixel_scale),
                ),
            )
            // Effect and dialog systems (separate tuple to stay within Bevy's limit)
            .add_systems(
                Update,
                (
                    screen_shake_system.after(update_camera),
                    fade_system.after(advance_action_queue),
                    handle_dialog_event.after(advance_action_queue),
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
