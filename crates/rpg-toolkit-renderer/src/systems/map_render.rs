use bevy::prelude::*;

use crate::components::{
    NpcPatrolState, NpcSprite, NpcSpriteState, PlayerCharacter, RendererTileSprite,
};
use crate::events::MapChanged;
use crate::resources::{NpcPositions, RendererProjectData, RendererState};
use crate::systems::player::grid_to_world;
use rpg_toolkit_common::sprite_atlas_index;

/// Computes the Z value for a tile sprite based on its elevation relative to the player.
///
/// Tiles with `elevation <= player_elevation` render below the player sprite.
/// Tiles with `elevation > player_elevation` render above the player sprite.
///
/// The player Z is `num_layers + 1.0`. Tiles below use their layer_index as Z (0..num_layers).
/// Tiles above use `num_layers + 2.0 + layer_index` to ensure they render above the player.
pub fn compute_tile_z(
    tile_elevation: u32,
    player_elevation: u32,
    layer_index: usize,
    num_layers: usize,
) -> f32 {
    if tile_elevation > player_elevation {
        // Render above the player: player Z is num_layers + 1.0, so use num_layers + 2.0 + layer_index
        num_layers as f32 + 2.0 + layer_index as f32
    } else {
        // Render below the player: use layer_index as Z (0..num_layers)
        layer_index as f32
    }
}

/// Reacts to `MapChanged` events: despawns all existing tile sprites and
/// spawns new tile sprites for the new active map.
/// Computes Z values based on tile elevation relative to the player's elevation.
pub fn sync_map_sprites(
    mut map_changed: MessageReader<MapChanged>,
    project_data: Res<RendererProjectData>,
    existing_tiles: Query<Entity, With<RendererTileSprite>>,
    player_query: Query<&PlayerCharacter>,
    mut commands: Commands,
) {
    // Only process the most recent map change if multiple arrived in one frame
    let Some(event) = map_changed.read().last() else {
        return;
    };

    let Some(map) = project_data.project_file.maps.get(&event.new_map_id) else {
        warn!(
            "MapChanged references non-existent map '{}'; skipping sprite sync",
            event.new_map_id
        );
        return;
    };

    // Despawn all existing tile sprites from the previous map
    for entity in existing_tiles.iter() {
        commands.entity(entity).despawn();
    }

    let tw = map.tile_width;
    let th = map.tile_height;
    let num_layers = map.layers.len();

    // Get the player's current elevation (default to 0 if player not yet spawned)
    let player_elevation = player_query.iter().next().map_or(0, |pc| pc.elevation);

    // Spawn tile sprites for each visible layer
    for (layer_index, layer) in map.layers.iter().enumerate() {
        if !layer.visible {
            continue;
        }

        for (y, row) in layer.tiles.iter().enumerate() {
            for (x, tile_opt) in row.iter().enumerate() {
                let Some(tile_ref) = tile_opt else {
                    continue;
                };

                // Resolve tileset texture and atlas layout
                let Some(texture) = project_data.tileset_textures.get(&tile_ref.tileset_id) else {
                    continue;
                };
                let Some(atlas_layout) =
                    project_data.tileset_atlas_layouts.get(&tile_ref.tileset_id)
                else {
                    continue;
                };

                // Look up the tileset metadata to compute the atlas index
                let Some(tileset_meta) =
                    project_data.project_file.tilesets.get(&tile_ref.tileset_id)
                else {
                    continue;
                };

                let atlas_index = (tile_ref.row * tileset_meta.columns + tile_ref.col) as usize;
                let world_pos = grid_to_world(x as u32, y as u32, tw, th);

                // Get tile elevation from attributes
                let tile_elevation = layer
                    .attributes
                    .cells
                    .get(y)
                    .and_then(|row| row.get(x))
                    .map_or(0, |attrs| attrs.elevation);

                let z = compute_tile_z(tile_elevation, player_elevation, layer_index, num_layers);

                commands.spawn((
                    RendererTileSprite {
                        layer_index,
                        x: x as u32,
                        y: y as u32,
                    },
                    Sprite {
                        image: texture.clone(),
                        texture_atlas: Some(TextureAtlas {
                            layout: atlas_layout.clone(),
                            index: atlas_index,
                        }),
                        ..default()
                    },
                    Transform::from_xyz(world_pos.x, world_pos.y, z),
                ));
            }
        }
    }
}
/// Reacts to `MapChanged` events: despawns existing NPC sprites and spawns
/// new NPC sprites for each `NpcInstance` on the new active map.
/// Each NPC entity gets an `NpcSpriteState` component for independent animation.
/// Uses elevation-aware Z ordering: NPCs above the player's elevation render
/// above the player sprite; NPCs at or below render below.
pub fn spawn_npc_sprites(
    mut map_changed: MessageReader<MapChanged>,
    project_data: Res<RendererProjectData>,
    existing_npcs: Query<Entity, With<NpcSprite>>,
    player_query: Query<&PlayerCharacter>,
    mut commands: Commands,
) {
    let Some(event) = map_changed.read().last() else {
        return;
    };

    // Despawn existing NPC sprites from the previous map
    for entity in existing_npcs.iter() {
        commands.entity(entity).despawn();
    }

    let Some(map) = project_data.project_file.maps.get(&event.new_map_id) else {
        return;
    };

    let tw = map.tile_width;
    let th = map.tile_height;
    let num_layers = map.layers.len();

    // Get the player's current elevation (default to 0 if player not yet spawned)
    let player_elevation = player_query.iter().next().map_or(0, |pc| pc.elevation);

    for (npc_idx, npc) in map.npcs.iter().enumerate() {
        let Some(texture) = project_data.spritesheet_textures.get(&npc.spritesheet_id) else {
            warn!(
                "NPC {} references spritesheet '{}' with no loaded texture; skipping",
                npc_idx, npc.spritesheet_id
            );
            continue;
        };
        let Some(atlas_layout) = project_data
            .spritesheet_atlas_layouts
            .get(&npc.spritesheet_id)
        else {
            continue;
        };

        // Idle pose: middle frame (frame 1) for the NPC's facing direction
        let idle_index = sprite_atlas_index(npc.facing, 1);
        let world_pos = grid_to_world(npc.x, npc.y, tw, th);

        // Render NPC sprites at 1:1 pixel scale — same as the player.
        // The y_offset keeps the character's feet aligned with the tile bottom.
        let (sprite_scale, y_offset) = project_data
            .project_file
            .spritesheets
            .get(&npc.spritesheet_id)
            .map(|ss| {
                let scale = 1.0_f32;
                let scaled_height = ss.sprite_height as f32 * scale;
                let offset = (scaled_height - th as f32) / 2.0;
                (scale, offset)
            })
            .unwrap_or((1.0, 0.0));

        // Build optional patrol state from the NPC's patrol config
        let patrol = npc.patrol_config.as_ref().map(|config| NpcPatrolState {
            current_waypoint_index: 0,
            forward: true,
            pause_timer: config.pause.max(0.0),
            paused: true,
        });

        // Compute NPC Z using the same elevation-aware rules as tiles.
        // NPCs above the player's elevation render above the player sprite;
        // NPCs at or below render below the player but above tile layers.
        let npc_z = if npc.elevation > player_elevation {
            // Above player: render above player Z (num_layers + 1.0)
            num_layers as f32 + 2.0 + num_layers as f32 + 0.5
        } else {
            // Below or equal to player: render between tiles and player
            num_layers as f32 + 0.5
        };

        commands.spawn((
            NpcSprite { npc_index: npc_idx },
            NpcSpriteState {
                facing: npc.facing,
                animation_frame: 1,
                animation_timer: 0.0,
                is_moving: false,
                grid_x: npc.x,
                grid_y: npc.y,
                move_animation: None,
                patrol,
                y_offset,
            },
            Sprite {
                image: texture.clone(),
                texture_atlas: Some(TextureAtlas {
                    layout: atlas_layout.clone(),
                    index: idle_index,
                }),
                ..default()
            },
            Transform::from_xyz(world_pos.x, world_pos.y + y_offset, npc_z)
                .with_scale(Vec3::splat(sprite_scale)),
        ));
    }
}

/// Re-sorts tile sprite Z values when the player's elevation changes.
/// Queries all `RendererTileSprite` entities and updates their `Transform.translation.z`
/// based on the current player elevation. This ensures draw order is correct within
/// the same frame as an elevation change.
#[allow(clippy::type_complexity)]
pub fn resort_tile_z_on_elevation_change(
    project_data: Res<RendererProjectData>,
    renderer_state: Res<RendererState>,
    player_query: Query<&PlayerCharacter, Changed<PlayerCharacter>>,
    mut tile_query: Query<(&RendererTileSprite, &mut Transform), Without<PlayerCharacter>>,
    mut npc_query: Query<
        (&NpcSprite, &mut Transform),
        (Without<RendererTileSprite>, Without<PlayerCharacter>),
    >,
) {
    // Only run when the PlayerCharacter component has changed
    let Some(player) = player_query.iter().next() else {
        return;
    };

    let Some(map_id) = &renderer_state.active_map_id else {
        return;
    };
    let Some(map) = project_data.project_file.maps.get(map_id) else {
        return;
    };

    let num_layers = map.layers.len();
    let player_elevation = player.elevation;

    // Update all tile sprite Z values
    for (tile_sprite, mut transform) in tile_query.iter_mut() {
        let tile_elevation = map
            .layers
            .get(tile_sprite.layer_index)
            .and_then(|layer| layer.attributes.cells.get(tile_sprite.y as usize))
            .and_then(|row| row.get(tile_sprite.x as usize))
            .map_or(0, |attrs| attrs.elevation);

        let z = compute_tile_z(
            tile_elevation,
            player_elevation,
            tile_sprite.layer_index,
            num_layers,
        );
        transform.translation.z = z;
    }

    // Update all NPC sprite Z values
    for (npc_sprite, mut transform) in npc_query.iter_mut() {
        let npc_elevation = map
            .npcs
            .get(npc_sprite.npc_index)
            .map_or(0, |npc| npc.elevation);

        let npc_z = if npc_elevation > player_elevation {
            num_layers as f32 + 2.0 + num_layers as f32 + 0.5
        } else {
            num_layers as f32 + 0.5
        };
        transform.translation.z = npc_z;
    }
}

/// Reacts to `MapChanged` events: rebuilds the `NpcPositions` resource from
/// the active map's NPC instances so the collision system uses dynamic positions.
pub fn init_npc_positions(
    mut map_changed: MessageReader<MapChanged>,
    project_data: Res<RendererProjectData>,
    mut npc_positions: ResMut<NpcPositions>,
) {
    let Some(event) = map_changed.read().last() else {
        return;
    };

    let Some(map) = project_data.project_file.maps.get(&event.new_map_id) else {
        npc_positions.positions.clear();
        return;
    };

    npc_positions.positions = map
        .npcs
        .iter()
        .map(|npc| (npc.x, npc.y, npc.elevation))
        .collect();
}
