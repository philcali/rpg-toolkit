use bevy::prelude::*;

use crate::components::{GameCamera, ParallaxSprite};
use crate::events::MapChanged;
use crate::resources::{PreviousCameraPosition, RendererProjectData};

/// Reacts to `MapChanged` events: despawns all existing parallax sprites and
/// spawns new parallax sprite entities for the target map's `parallax_layers`.
///
/// Z-ordering: all parallax z values are < 0.0 (behind tile layers which start at z=0.0).
/// Layers are sorted by `(z_order, list_index)` for stable ordering, then assigned
/// z values starting from a base of -1000.0, offset by sorted index * 0.1.
pub fn spawn_parallax_system(
    mut map_changed: MessageReader<MapChanged>,
    project_data: Res<RendererProjectData>,
    existing_parallax: Query<Entity, With<ParallaxSprite>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    // Only process the most recent map change if multiple arrived in one frame
    let Some(event) = map_changed.read().last() else {
        return;
    };

    // Despawn all existing parallax sprites from the previous map
    for entity in existing_parallax.iter() {
        commands.entity(entity).despawn();
    }

    let Some(map) = project_data.project_file.maps.get(&event.new_map_id) else {
        warn!(
            "MapChanged references non-existent map '{}'; skipping parallax spawn",
            event.new_map_id
        );
        return;
    };

    if map.parallax_layers.is_empty() {
        return;
    }

    // Build a list of (z_order, list_index) pairs for stable sorting
    let mut sorted_indices: Vec<(i32, usize)> = map
        .parallax_layers
        .iter()
        .enumerate()
        .map(|(idx, layer)| (layer.z_order, idx))
        .collect();
    sorted_indices.sort();

    // Spawn sprite entities for each parallax layer in sorted order
    for (sorted_idx, &(_z_order, list_index)) in sorted_indices.iter().enumerate() {
        let layer = &map.parallax_layers[list_index];

        // Skip layers with empty image paths
        if layer.image_path.is_empty() {
            warn!(
                "Parallax layer {} has empty image_path; skipping",
                list_index
            );
            continue;
        }

        // Load the image via AssetServer (Bevy will log its own warning if the file is missing)
        let texture: Handle<Image> = asset_server.load(&layer.image_path);

        // Compute z: all values < 0.0, base at -1000.0, offset by sorted index
        let z = -1000.0 + sorted_idx as f32 * 0.1;

        commands.spawn((
            Sprite {
                image: texture,
                ..default()
            },
            Transform::from_translation(Vec3::new(0.0, 0.0, z)),
            ParallaxSprite {
                scroll_factor: layer.scroll_factor,
                layer_index: list_index,
            },
        ));
    }
}

/// Despawns all entities with the `ParallaxSprite` component.
/// This can be used independently if needed, but the spawn system already handles
/// despawning on map change.
pub fn despawn_parallax_system(
    existing_parallax: Query<Entity, With<ParallaxSprite>>,
    mut commands: Commands,
) {
    for entity in existing_parallax.iter() {
        commands.entity(entity).despawn();
    }
}

/// Computes the parallax translation delta for a given camera delta and scroll factor.
/// This is the core computation extracted for testability.
pub fn compute_parallax_translation(camera_delta: Vec2, scroll_factor: f32) -> Vec2 {
    camera_delta * scroll_factor
}

/// Updates parallax sprite positions based on camera movement delta.
///
/// Each frame, computes the camera position delta from the previous frame, then
/// translates each `ParallaxSprite` entity by `delta * scroll_factor`.
/// A `scroll_factor` of 0.0 means no movement; 1.0 means full camera movement.
pub fn update_parallax_system(
    camera_query: Query<&Transform, With<GameCamera>>,
    mut prev_cam_pos: ResMut<PreviousCameraPosition>,
    mut parallax_query: Query<(&mut Transform, &ParallaxSprite), Without<GameCamera>>,
) {
    let Ok(cam_tf) = camera_query.single() else {
        return;
    };

    let current_pos = Vec2::new(cam_tf.translation.x, cam_tf.translation.y);
    let delta = current_pos - prev_cam_pos.position;
    prev_cam_pos.position = current_pos;

    // Apply parallax scrolling
    for (mut transform, parallax) in parallax_query.iter_mut() {
        let translation = compute_parallax_translation(delta, parallax.scroll_factor);
        transform.translation.x += translation.x;
        transform.translation.y += translation.y;
    }
}
