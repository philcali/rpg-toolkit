use rpg_toolkit_common::MapData;

/// Returns `true` if any layer at `(x, y)` has `opacity == true`.
pub fn is_tile_blocked(map: &MapData, x: u32, y: u32) -> bool {
    map.layers.iter().any(|layer| {
        layer
            .attributes
            .cells
            .get(y as usize)
            .and_then(|row| row.get(x as usize))
            .map(|attrs| attrs.opacity)
            .unwrap_or(false)
    })
}
