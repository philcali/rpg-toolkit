use bevy::prelude::*;

use crate::data::Project;

/// Marker component for tile sprites managed by the render system.
#[allow(dead_code)]
#[derive(Component)]
pub struct TileSprite {
    pub layer_index: usize,
    pub x: u32,
    pub y: u32,
}

/// System that synchronizes tile sprites with the active map in the `Project`.
///
/// On each frame where `Project` has changed, this despawns all existing tile sprites
/// and respawns them from scratch. For each `TileRef`, the correct tileset is looked up
/// from `project.tilesets` to resolve the texture and atlas index.
pub fn sync_tile_sprites(
    mut commands: Commands,
    project: Res<Project>,
    existing_tiles: Query<Entity, With<TileSprite>>,
) {
    if !project.is_changed() {
        return;
    }

    // Despawn all existing tile sprites
    for entity in existing_tiles.iter() {
        commands.entity(entity).despawn();
    }

    let Some(active_map) = project.active_map() else {
        return;
    };

    let tile_size = active_map.tile_width as f32;

    // Spawn sprites for each placed tile across all visible layers
    for (layer_idx, layer) in active_map.layers.iter().enumerate() {
        if !layer.visible {
            continue;
        }

        // Z-order: lower layers have lower z values
        let z = layer_idx as f32;

        for (row_idx, row) in layer.tiles.iter().enumerate() {
            for (col_idx, cell) in row.iter().enumerate() {
                let Some(tile_ref) = cell else { continue };

                // Look up the tileset for this TileRef
                let Some(tileset) = project.tilesets.get(&tile_ref.tileset_id) else {
                    warn!(
                        "TileRef at ({}, {}) layer {} references unknown tileset '{}'",
                        col_idx, row_idx, layer_idx, tile_ref.tileset_id
                    );
                    continue;
                };

                // Compute atlas index (row-major)
                let atlas_index = (tile_ref.row * tileset.meta.columns + tile_ref.col) as usize;

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
