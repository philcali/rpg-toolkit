use rpg_toolkit_common::MapData;

use crate::resources::NpcPositions;

/// Returns `true` if any layer at `(x, y)` has `opacity == true` at the
/// player's elevation, or if any NPC at the same elevation occupies the tile.
///
/// When `player_elevation` is `Some`, only tiles whose elevation matches the
/// player's elevation are considered blocking. When `None`, all opaque tiles
/// block regardless of elevation (legacy behavior for NPC movement checks).
///
/// When `npc_positions` is `Some`, uses the dynamic runtime positions;
/// when `None`, skips the NPC check entirely (opacity only).
pub fn is_tile_blocked(
    map: &MapData,
    x: u32,
    y: u32,
    player_elevation: Option<u32>,
    npc_positions: Option<&NpcPositions>,
) -> bool {
    let opacity_blocked = map.layers.iter().any(|layer| {
        layer
            .attributes
            .cells
            .get(y as usize)
            .and_then(|row| row.get(x as usize))
            .map(|attrs| {
                if attrs.opacity {
                    match player_elevation {
                        Some(elev) => attrs.elevation == elev,
                        None => true, // No elevation filter — block unconditionally
                    }
                } else {
                    false
                }
            })
            .unwrap_or(false)
    });
    let npc_blocked = match npc_positions {
        Some(positions) => match player_elevation {
            Some(elev) => positions.is_occupied_at_elevation(x, y, elev),
            None => positions.is_occupied(x, y),
        },
        None => false,
    };
    opacity_blocked || npc_blocked
}
