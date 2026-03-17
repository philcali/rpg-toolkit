use bevy::prelude::*;

use crate::data::{MapData, TilesetData};

/// Marker component for tile sprites managed by the render system.
#[derive(Component)]
pub struct TileSprite {
    pub layer_index: usize,
    pub x: u32,
    pub y: u32,
}

/// System that synchronizes tile sprites with the current `MapData` and `TilesetData`.
///
/// On each frame where `MapData` has changed, this despawns all existing tile sprites
/// and respawns them from scratch. This is simple and correct for the current scope;
/// incremental updates can be added later if performance requires it.
pub fn sync_tile_sprites(
    mut commands: Commands,
    map: Option<Res<MapData>>,
    tileset: Option<Res<TilesetData>>,
    existing_tiles: Query<Entity, With<TileSprite>>,
) {
    let Some(map) = map else { return };
    let Some(tileset) = tileset else { return };

    if !map.is_changed() {
        return;
    }

    // Despawn all existing tile sprites
    for entity in existing_tiles.iter() {
        commands.entity(entity).despawn();
    }

    let tile_size = tileset.meta.tile_width as f32;
    let cols = tileset.meta.columns;

    // Spawn sprites for each placed tile across all visible layers
    for (layer_idx, layer) in map.layers.iter().enumerate() {
        if !layer.visible {
            continue;
        }

        // Z-order: lower layers have lower z values
        let z = layer_idx as f32;

        for (row_idx, row) in layer.tiles.iter().enumerate() {
            for (col_idx, cell) in row.iter().enumerate() {
                let Some(tile_index) = cell else { continue };

                // Compute atlas index (row-major)
                let atlas_index = (tile_index.row * cols + tile_index.col) as usize;

                // World position: x goes right, y goes down (negative in Bevy)
                let world_x = col_idx as f32 * tile_size + tile_size / 2.0;
                let world_y = -(row_idx as f32 * tile_size + tile_size / 2.0);

                commands.spawn((
                    Sprite::from_atlas_image(
                        tileset.texture.clone(),
                        TextureAtlas {
                            layout: tileset.atlas_layout.clone(),
                            index: atlas_index,
                        },
                    ),
                    Transform::from_xyz(world_x, world_y, z),
                    TileSprite {
                        layer_index: layer_idx,
                        x: col_idx as u32,
                        y: row_idx as u32,
                    },
                ));
            }
        }
    }
}
