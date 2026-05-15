use bevy::prelude::*;

use crate::components::{NpcPatrolState, NpcSprite, NpcSpriteState, RendererTileSprite};
use crate::events::MapChanged;
use crate::resources::{NpcPositions, RendererProjectData};
use crate::systems::player::grid_to_world;
use rpg_toolkit_common::sprite_atlas_index;

/// Reacts to `MapChanged` events: despawns all existing tile sprites and
/// spawns new tile sprites for the new active map.
pub fn sync_map_sprites(
    mut map_changed: MessageReader<MapChanged>,
    project_data: Res<RendererProjectData>,
    existing_tiles: Query<Entity, With<RendererTileSprite>>,
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

    // Spawn tile sprites for each visible layer
    for (layer_index, layer) in map.layers.iter().enumerate() {
        if !layer.visible {
            continue;
        }

        let z = layer_index as f32;

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
pub fn spawn_npc_sprites(
    mut map_changed: MessageReader<MapChanged>,
    project_data: Res<RendererProjectData>,
    existing_npcs: Query<Entity, With<NpcSprite>>,
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
    // NPC sprites render above tile layers but below the player
    let npc_z = map.layers.len() as f32 + 0.5;

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

    npc_positions.positions = map.npcs.iter().map(|npc| (npc.x, npc.y)).collect();
}
