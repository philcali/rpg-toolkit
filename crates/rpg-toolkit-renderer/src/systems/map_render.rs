use bevy::prelude::*;

use crate::components::RendererTileSprite;
use crate::events::MapChanged;
use crate::resources::RendererProjectData;
use crate::systems::player::grid_to_world;

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
