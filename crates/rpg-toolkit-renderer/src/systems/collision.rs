use rpg_toolkit_common::MapData;

use crate::resources::NpcPositions;

/// Returns `true` if any layer at `(x, y)` has `opacity == true`,
/// or if any NPC occupies the tile at `(x, y)`.
///
/// When `npc_positions` is `Some`, uses the dynamic runtime positions;
/// when `None`, skips the NPC check entirely (opacity only).
pub fn is_tile_blocked(
    map: &MapData,
    x: u32,
    y: u32,
    npc_positions: Option<&NpcPositions>,
) -> bool {
    let opacity_blocked = map.layers.iter().any(|layer| {
        layer
            .attributes
            .cells
            .get(y as usize)
            .and_then(|row| row.get(x as usize))
            .map(|attrs| attrs.opacity)
            .unwrap_or(false)
    });
    let npc_blocked = match npc_positions {
        Some(positions) => positions.is_occupied(x, y),
        None => false, // None means skip NPC check (caller handles NPCs separately)
    };
    opacity_blocked || npc_blocked
}
