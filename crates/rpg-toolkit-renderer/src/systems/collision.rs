use rpg_toolkit_common::MapData;

/// Returns `true` if any layer at `(x, y)` has `opacity == true`.
/// Returns `true` if any layer at `(x, y)` has `opacity == true`,
/// or if any NPC occupies the tile at `(x, y)`.
pub fn is_tile_blocked(map: &MapData, x: u32, y: u32) -> bool {
    let opacity_blocked = map.layers.iter().any(|layer| {
        layer
            .attributes
            .cells
            .get(y as usize)
            .and_then(|row| row.get(x as usize))
            .map(|attrs| attrs.opacity)
            .unwrap_or(false)
    });
    let npc_blocked = map.npcs.iter().any(|npc| npc.x == x && npc.y == y);
    opacity_blocked || npc_blocked
}
