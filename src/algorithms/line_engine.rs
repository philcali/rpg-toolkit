/// Computes tile coordinates along a straight line from (x0, y0) to (x1, y1)
/// using Bresenham's line algorithm.
///
/// Uses `i64` intermediates for safe unsigned coordinate math.
/// Returns an ordered list from start to end, inclusive.
pub fn bresenham_line(x0: u32, y0: u32, x1: u32, y1: u32) -> Vec<(u32, u32)> {
    let mut points = Vec::new();

    let mut x = x0 as i64;
    let mut y = y0 as i64;
    let ex = x1 as i64;
    let ey = y1 as i64;

    let dx = (ex - x).abs();
    let dy = -(ey - y).abs();
    let sx: i64 = if x < ex { 1 } else { -1 };
    let sy: i64 = if y < ey { 1 } else { -1 };
    let mut err = dx + dy;

    loop {
        points.push((x as u32, y as u32));

        if x == ex && y == ey {
            break;
        }

        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x += sx;
        }
        if e2 <= dx {
            err += dx;
            y += sy;
        }
    }

    points
}
