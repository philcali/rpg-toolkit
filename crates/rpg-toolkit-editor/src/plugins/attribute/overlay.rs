//! Gizmo overlay rendering for attribute mode — draws opacity indicators,
//! event trigger markers, spawn point markers, NPC positions, patrol paths,
//! and elevation/transition overlays.

use bevy::prelude::*;

use super::npc_dialog::NpcPlacementDialog;
use crate::data::{AttributeTool, EditorMode, EditorState, Project};

/// Draws gizmo overlays on tiles with attributes when in attribute mode.
pub fn attribute_overlay_system(
    editor_state: Res<EditorState>,
    project: Res<Project>,
    npc_dialog: Res<NpcPlacementDialog>,
    mut gizmos: Gizmos,
) {
    if editor_state.editor_mode != EditorMode::Attribute {
        return;
    }

    let Some(map) = project.active_map() else {
        return;
    };

    let tile = map.tile_width as f32;

    // Draw opacity overlays (red semi-transparent) for the active layer
    if let Some(layer) = map.layers.get(map.active_layer_index) {
        for (y, row) in layer.attributes.cells.iter().enumerate() {
            for (x, attrs) in row.iter().enumerate() {
                if attrs.opacity {
                    let px = x as f32 * tile + tile / 2.0;
                    let py = -(y as f32 * tile + tile / 2.0);
                    gizmos.rect_2d(
                        Isometry2d::from_translation(Vec2::new(px, py)),
                        Vec2::splat(tile),
                        Color::srgba(1.0, 0.0, 0.0, 0.35),
                    );
                }

                if !attrs.event_trigger.is_empty() {
                    let px = x as f32 * tile + tile / 2.0;
                    let py = -(y as f32 * tile + tile / 2.0);
                    gizmos.rect_2d(
                        Isometry2d::from_translation(Vec2::new(px, py)),
                        Vec2::splat(tile * 0.8),
                        Color::srgba(0.0, 0.4, 1.0, 0.45),
                    );
                }
            }
        }
    }

    // Draw spawn point marker if on the current map
    if let Some(ref sp) = project.spawn_point
        && let Some(active_map_id) = project.active_map_id()
        && sp.map_id == *active_map_id
    {
        let px = sp.x as f32 * tile + tile / 2.0;
        let py = -(sp.y as f32 * tile + tile / 2.0);
        gizmos.rect_2d(
            Isometry2d::from_translation(Vec2::new(px, py)),
            Vec2::splat(tile * 0.9),
            Color::srgba(0.0, 1.0, 0.0, 0.5),
        );
        // Draw a cross inside the spawn point marker
        let half = tile * 0.35;
        let center = Vec2::new(px, py);
        gizmos.line_2d(
            center + Vec2::new(-half, -half),
            center + Vec2::new(half, half),
            Color::srgba(0.0, 1.0, 0.0, 0.8),
        );
        gizmos.line_2d(
            center + Vec2::new(-half, half),
            center + Vec2::new(half, -half),
            Color::srgba(0.0, 1.0, 0.0, 0.8),
        );
    }

    // Draw NPC overlays (purple/magenta) when in NPC placement mode
    if editor_state.attribute_tool == AttributeTool::NpcPlacement {
        for npc in &map.npcs {
            let px = npc.x as f32 * tile + tile / 2.0;
            let py = -(npc.y as f32 * tile + tile / 2.0);
            gizmos.rect_2d(
                Isometry2d::from_translation(Vec2::new(px, py)),
                Vec2::splat(tile * 0.85),
                Color::srgba(0.8, 0.2, 0.8, 0.45),
            );
        }
    }

    // Draw patrol path overlay when NPC placement dialog is open
    if editor_state.attribute_tool == AttributeTool::NpcPlacement && npc_dialog.open {
        let waypoints = &npc_dialog.patrol_waypoints;
        let color = Color::srgba(1.0, 0.8, 0.0, 0.8); // Yellow/orange for patrol paths
        let marker_color = Color::srgba(1.0, 0.6, 0.0, 0.9);

        // Draw connected line segments between waypoints
        for i in 0..waypoints.len() {
            let (wx, wy) = waypoints[i];
            let px = wx as f32 * tile + tile / 2.0;
            let py = -(wy as f32 * tile + tile / 2.0);

            // Draw line to next waypoint
            if i + 1 < waypoints.len() {
                let (nx, ny) = waypoints[i + 1];
                let npx = nx as f32 * tile + tile / 2.0;
                let npy = -(ny as f32 * tile + tile / 2.0);
                gizmos.line_2d(Vec2::new(px, py), Vec2::new(npx, npy), color);
            }

            // Draw numbered marker at each waypoint (small rect)
            gizmos.rect_2d(
                Isometry2d::from_translation(Vec2::new(px, py)),
                Vec2::splat(tile * 0.4),
                marker_color,
            );
        }
    }

    // Also draw patrol paths for existing NPCs that have patrol configs
    if editor_state.attribute_tool == AttributeTool::NpcPlacement {
        let path_color = Color::srgba(0.8, 0.6, 0.0, 0.5); // Dimmer for non-selected NPCs
        for npc in &map.npcs {
            if let Some(ref config) = npc.patrol_config {
                for i in 0..config.waypoints.len() {
                    let (wx, wy) = config.waypoints[i];
                    let px = wx as f32 * tile + tile / 2.0;
                    let py = -(wy as f32 * tile + tile / 2.0);

                    if i + 1 < config.waypoints.len() {
                        let (nx, ny) = config.waypoints[i + 1];
                        let npx = nx as f32 * tile + tile / 2.0;
                        let npy = -(ny as f32 * tile + tile / 2.0);
                        gizmos.line_2d(Vec2::new(px, py), Vec2::new(npx, npy), path_color);
                    }

                    gizmos.rect_2d(
                        Isometry2d::from_translation(Vec2::new(px, py)),
                        Vec2::splat(tile * 0.3),
                        path_color,
                    );
                }
            }
        }
    }

    // Draw elevation overlays when the Elevation tool is active
    if editor_state.attribute_tool == AttributeTool::Elevation
        && let Some(layer) = map.layers.get(map.active_layer_index)
    {
        for (y, row) in layer.attributes.cells.iter().enumerate() {
            for (x, attrs) in row.iter().enumerate() {
                if attrs.elevation > 0 {
                    let px = x as f32 * tile + tile / 2.0;
                    let py = -(y as f32 * tile + tile / 2.0);

                    // Color intensity scales with elevation level (capped at 5 for visibility)
                    let intensity = (attrs.elevation as f32 / 5.0).min(1.0);
                    gizmos.rect_2d(
                        Isometry2d::from_translation(Vec2::new(px, py)),
                        Vec2::splat(tile * 0.9),
                        Color::srgba(0.2, 0.6, 1.0, 0.2 + intensity * 0.3),
                    );

                    // Draw a smaller inner rect to indicate the level number visually
                    let inner_size = tile * 0.3;
                    for i in 0..attrs.elevation.min(4) {
                        let offset_y = (i as f32 - attrs.elevation.min(4) as f32 / 2.0 + 0.5)
                            * (inner_size * 0.6);
                        gizmos.rect_2d(
                            Isometry2d::from_translation(Vec2::new(px, py + offset_y)),
                            Vec2::new(inner_size, inner_size * 0.4),
                            Color::srgba(0.2, 0.6, 1.0, 0.7),
                        );
                    }
                }
            }
        }
    }

    // Draw elevation transition overlays when the ElevationTransition tool is active
    if editor_state.attribute_tool == AttributeTool::ElevationTransition
        && let Some(layer) = map.layers.get(map.active_layer_index)
    {
        for (y, row) in layer.attributes.cells.iter().enumerate() {
            for (x, attrs) in row.iter().enumerate() {
                if attrs.target_elevation.is_some() {
                    let px = x as f32 * tile + tile / 2.0;
                    let py = -(y as f32 * tile + tile / 2.0);

                    // Draw a distinct diamond/arrow marker for transitions
                    let half = tile * 0.35;
                    let color = Color::srgba(1.0, 0.5, 0.0, 0.6);

                    // Diamond shape
                    gizmos.line_2d(Vec2::new(px, py + half), Vec2::new(px + half, py), color);
                    gizmos.line_2d(Vec2::new(px + half, py), Vec2::new(px, py - half), color);
                    gizmos.line_2d(Vec2::new(px, py - half), Vec2::new(px - half, py), color);
                    gizmos.line_2d(Vec2::new(px - half, py), Vec2::new(px, py + half), color);

                    // Upward arrow inside
                    let arrow_half = tile * 0.15;
                    gizmos.line_2d(
                        Vec2::new(px, py - arrow_half),
                        Vec2::new(px, py + arrow_half),
                        color,
                    );
                    gizmos.line_2d(
                        Vec2::new(px - arrow_half * 0.6, py + arrow_half * 0.3),
                        Vec2::new(px, py + arrow_half),
                        color,
                    );
                    gizmos.line_2d(
                        Vec2::new(px + arrow_half * 0.6, py + arrow_half * 0.3),
                        Vec2::new(px, py + arrow_half),
                        color,
                    );
                }
            }
        }
    }
}
