use bevy::prelude::*;
use rpg_toolkit_common::{TilesetId, compute_animation_frame_index};

use crate::data::Project;

/// Global animation clock for the editor canvas.
/// Accumulates elapsed time each frame for animation cycling.
#[derive(Resource, Default)]
pub struct EditorAnimationTick {
    pub elapsed_ms: u64,
}

/// System that increments the editor animation clock by delta time each frame.
pub fn tick_editor_animation(time: Res<Time>, mut tick: ResMut<EditorAnimationTick>) {
    tick.elapsed_ms += time.delta().as_millis() as u64;
}

/// Marker component for tile sprites managed by the render system.
#[allow(dead_code)]
#[derive(Component)]
pub struct TileSprite {
    pub layer_index: usize,
    pub x: u32,
    pub y: u32,
}

/// Marker component for tiles that participate in an animation.
/// Stored on the sprite entity so the animation system can update its atlas index.
#[allow(dead_code)]
#[derive(Component)]
pub struct AnimatedTile {
    pub tileset_id: TilesetId,
    pub animation_index: usize,
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

                // Check if this tile is the first frame of any animation
                let animation_match =
                    tileset
                        .meta
                        .animations
                        .iter()
                        .enumerate()
                        .find(|(_, anim)| {
                            anim.frames
                                .first()
                                .is_some_and(|f| f.col == tile_ref.col && f.row == tile_ref.row)
                        });

                // World position: x goes right, y goes down (negative in Bevy)
                let world_x = col_idx as f32 * tile_size + tile_size / 2.0;
                let world_y = -(row_idx as f32 * tile_size + tile_size / 2.0);

                let mut entity_commands = commands.spawn((
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

                if let Some((idx, _)) = animation_match {
                    entity_commands.insert(AnimatedTile {
                        tileset_id: tile_ref.tileset_id.clone(),
                        animation_index: idx,
                    });
                }
            }
        }
    }
}

/// System that animates tiles with the AnimatedTile component.
/// Runs each frame to update atlas indices based on the global animation clock.
pub fn animate_editor_tiles(
    tick: Res<EditorAnimationTick>,
    project: Res<Project>,
    mut query: Query<(&AnimatedTile, &mut Sprite)>,
) {
    for (anim_tile, mut sprite) in query.iter_mut() {
        let Some(tileset) = project.tilesets.get(&anim_tile.tileset_id) else {
            continue;
        };
        let Some(animation) = tileset.meta.animations.get(anim_tile.animation_index) else {
            continue;
        };

        let frame_idx = compute_animation_frame_index(
            tick.elapsed_ms,
            animation.frame_duration_ms,
            animation.frames.len(),
        );

        let frame = &animation.frames[frame_idx];
        let atlas_index = (frame.row * tileset.meta.columns + frame.col) as usize;

        // Update the sprite's texture atlas index
        if let Some(ref mut atlas) = sprite.texture_atlas {
            atlas.index = atlas_index;
        }
    }
}
