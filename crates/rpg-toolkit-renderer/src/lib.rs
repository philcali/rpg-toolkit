use bevy::prelude::*;
use rpg_toolkit_common::AppPhase;
pub use rpg_toolkit_common::NewGameFlag;

pub mod components;
pub mod dialog;
pub mod effects;
pub mod events;
pub mod input;
pub mod markup;
pub mod resources;
pub mod save;
pub mod systems;

pub use components::{
    FadeOverlay, GameCamera, MoveAnimation, NpcMoveAnimation, NpcPatrolState, NpcSprite,
    NpcSpriteState, ParallaxSprite, PlayerCharacter, PlayerSpriteState, RendererTileSprite,
};
pub use events::{MapChanged, PlayerMoved, ShowDialog};
pub use input::{Direction, MovementIntent, handle_intro_skip, open_status_on_escape, read_input};
pub use resources::{
    ActionQueue, ActiveShopId, AnimationConfig, CameraFollowTarget, CameraPanState,
    CharacterProgress, CharacterProgressState, CurrencyState, EntityMoveState, FadeState,
    GameState, InteractionIntent, IntroEventsActive, InventoryState, JumpAnimState, MovementConfig,
    NpcCollisionEvent, NpcPositions, PartyState, PixelScaleConfig, PixelScaleMode,
    PlayerAppearanceState, PlayerVisual, PreviousCameraPosition, RendererProjectData,
    RendererState, ScreenShakeState, SpeedMultiplier, WaitState, WaitingFor,
};
pub use systems::camera::{apply_pixel_scale, compute_zoom_to_fit, spawn_camera, update_camera};
pub use systems::collision::is_tile_blocked;
pub use systems::entity_move::entity_move_system;
pub use systems::hotkey::hotkey_input_system;
pub use systems::jump::{compute_landing, jump_animation_system, jump_arc_offset};
pub use systems::map_render::{
    RendererAnimatedTile, RendererAnimationTick, animate_renderer_tiles, compute_tile_z,
    init_npc_positions, resort_tile_z_on_elevation_change, spawn_npc_sprites, sync_map_sprites,
    tick_renderer_animation, update_character_depth_sort,
};
pub use systems::npc::{
    npc_patrol_animation, npc_patrol_movement, npc_trigger_system, read_interaction_input,
};
pub use systems::parallax::{
    compute_parallax_translation, despawn_parallax_system, spawn_parallax_system,
    update_parallax_system,
};
pub use systems::player::{
    animate_player, animate_player_sprite, grid_to_world, player_movement, spawn_player,
};
pub use systems::speed::{apply_speed_multiplier_system, compute_speed_move_duration};
pub use systems::spritesheet::{build_spritesheet_atlas, load_spritesheet_assets};
pub use systems::triggers::{
    advance_action_queue, camera_pan_system, check_triggers, fade_system, handle_map_change,
    screen_shake_system, trigger_intro_events, wait_system,
};

pub use effects::{
    compute_fade_opacity, compute_shake_offset, is_blocking_action, is_fade_complete,
    is_shake_complete,
};

pub use dialog::{
    DialogBox, DialogConfig, DialogPanel, DialogPosition, DialogState, DialogText, DialogTextNode,
    DialogTextRegistry, FacePortrait, OverflowIndicator, compute_visible_chars,
    dialog_config_from_data, dialog_text_from_data,
};
pub use systems::dialog::{
    detect_overflow, handle_dialog_event, handle_dialog_input, update_dialog_typewriter,
};
pub use systems::selection::{
    ResolvedChoice, SelectionBox, SelectionCursor, SelectionLabel, SelectionState,
    handle_selection_input,
};

pub use resources::SavePath;
pub use save::{CharacterProgressData, SaveFile, save_game};

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
            .init_resource::<CurrencyState>()
            .init_resource::<InventoryState>()
            .init_resource::<CharacterProgressState>()
            .init_resource::<PartyState>()
            .init_resource::<RendererAnimationTick>()
            .init_resource::<SpeedMultiplier>()
            .init_resource::<PreviousCameraPosition>()
            // Events
            .add_message::<MapChanged>()
            .add_message::<PlayerMoved>()
            .add_message::<ShowDialog>()
            // Startup systems (ungated - run regardless of AppPhase)
            .add_systems(
                Startup,
                (load_spritesheet_assets, spawn_player, spawn_camera).chain(),
            )
            // Fire initial map changed when entering InGame
            .add_systems(OnEnter(AppPhase::InGame), fire_initial_map_changed)
            // Update systems gated on InGame
            .add_systems(
                Update,
                (
                    read_input,
                    tick_renderer_animation,
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
                    animate_renderer_tiles.after(sync_map_sprites),
                    spawn_npc_sprites.after(animate_renderer_tiles),
                    init_npc_positions
                        .after(spawn_npc_sprites)
                        .before(npc_patrol_movement),
                    resort_tile_z_on_elevation_change.after(init_npc_positions),
                    update_character_depth_sort.after(resort_tile_z_on_elevation_change),
                    apply_pixel_scale.after(update_character_depth_sort),
                    update_camera.after(apply_pixel_scale),
                )
                    .run_if(in_state(AppPhase::InGame)),
            )
            // Effect and dialog systems (separate tuple to stay within Bevy's limit)
            .add_systems(
                Update,
                (
                    trigger_intro_events.before(check_triggers),
                    screen_shake_system.after(update_camera),
                    entity_move_system.after(advance_action_queue),
                    camera_pan_system.after(advance_action_queue),
                    fade_system.after(advance_action_queue),
                    wait_system.after(advance_action_queue),
                    jump_animation_system.after(advance_action_queue),
                    apply_speed_multiplier_system.after(advance_action_queue),
                    spawn_parallax_system.after(handle_map_change),
                    update_parallax_system.after(update_camera),
                    handle_dialog_event.after(advance_action_queue),
                    detect_overflow.after(handle_dialog_event),
                    update_dialog_typewriter.after(handle_dialog_event),
                    handle_dialog_input.after(update_dialog_typewriter),
                    handle_selection_input
                        .after(read_input)
                        .before(player_movement),
                    hotkey_input_system
                        .after(read_input)
                        .before(player_movement),
                    handle_intro_skip
                        .after(read_input)
                        .before(open_status_on_escape),
                    open_status_on_escape.after(read_input),
                )
                    .run_if(in_state(AppPhase::InGame)),
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
