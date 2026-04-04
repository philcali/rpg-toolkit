use std::collections::VecDeque;

use crate::data::map::TileRef;

/// Computes the set of tile coordinates to flood-fill starting from `start`.
///
/// Uses BFS with 4-directional adjacency (up, down, left, right).
/// Returns an empty vec if `start` is out of bounds or if the target already
/// equals the replacement.
pub fn flood_fill(
    grid: &[Vec<Option<TileRef>>],
    start: (u32, u32),
    target: &Option<TileRef>,
    replacement: &TileRef,
) -> Vec<(u32, u32)> {
    let height = grid.len();
    if height == 0 {
        return Vec::new();
    }
    let width = grid[0].len();

    let (sx, sy) = (start.0 as usize, start.1 as usize);
    if sy >= height || sx >= width {
        return Vec::new();
    }

    // If the target is already the replacement, nothing to do.
    if *target == Some(replacement.clone()) {
        return Vec::new();
    }

    let mut visited = vec![vec![false; width]; height];
    let mut result = Vec::new();
    let mut queue = VecDeque::new();

    queue.push_back((sx, sy));
    visited[sy][sx] = true;

    while let Some((cx, cy)) = queue.pop_front() {
        if grid[cy][cx] != *target {
            continue;
        }

        result.push((cx as u32, cy as u32));

        for (dx, dy) in [(0i64, -1i64), (0, 1), (-1, 0), (1, 0)] {
            let nx = cx as i64 + dx;
            let ny = cy as i64 + dy;
            if nx >= 0 && ny >= 0 && (nx as usize) < width && (ny as usize) < height {
                let (ux, uy) = (nx as usize, ny as usize);
                if !visited[uy][ux] {
                    visited[uy][ux] = true;
                    queue.push_back((ux, uy));
                }
            }
        }
    }

    result
}
