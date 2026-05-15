use clap::{Parser, Subcommand};
use image::{Rgba, RgbaImage};
use rand::Rng;

#[derive(Parser)]
#[command(
    name = "rpg-toolkit-asset-gen",
    about = "Generate placeholder game assets"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a 24x32, 3-frame, 4-direction character spritesheet (72x128 PNG).
    Character {
        #[arg(short, long, default_value = "character.png")]
        output: String,
    },
    /// Generate a random 16x16 scene tileset PNG.
    Tileset {
        #[arg(short, long, default_value = "tileset.png")]
        output: String,
        /// Number of tile columns.
        #[arg(long, default_value_t = 8)]
        cols: u32,
        /// Number of tile rows.
        #[arg(long, default_value_t = 8)]
        rows: u32,
    },
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Character { output } => generate_character(&output),
        Commands::Tileset { output, cols, rows } => generate_tileset(&output, cols, rows),
    }
}

// ---------------------------------------------------------------------------
// Character spritesheet generator (72x128: 3 cols x 4 rows, 24x32 each)
// ---------------------------------------------------------------------------

/// Skin/body palette per direction row so you can visually tell them apart.
/// Row order: Up (0), Right (1), Down (2), Left (3).
const DIR_BODY_COLORS: [Rgba<u8>; 4] = [
    Rgba([180, 90, 200, 255]), // Row 0: Up    – purple-ish
    Rgba([130, 200, 90, 255]), // Row 1: Right – green-ish
    Rgba([90, 130, 200, 255]), // Row 2: Down  – blue-ish
    Rgba([200, 130, 90, 255]), // Row 3: Left  – orange-ish
];

const SKIN: Rgba<u8> = Rgba([240, 200, 160, 255]);
const HAIR: Rgba<u8> = Rgba([80, 50, 30, 255]);
const TRANSPARENT: Rgba<u8> = Rgba([0, 0, 0, 0]);

fn generate_character(path: &str) {
    let (fw, fh) = (24u32, 32u32);
    let (cols, rows) = (3u32, 4u32);
    let mut img = RgbaImage::from_pixel(fw * cols, fh * rows, TRANSPARENT);

    for dir in 0..rows {
        for frame in 0..cols {
            let ox = frame * fw;
            let oy = dir * fh;
            draw_character_frame(&mut img, ox, oy, fw, fh, dir, frame);
        }
    }

    img.save(path)
        .expect("failed to write character spritesheet");
    println!("Wrote character spritesheet to {path}");
}

fn draw_character_frame(
    img: &mut RgbaImage,
    ox: u32,
    oy: u32,
    fw: u32,
    _fh: u32,
    dir: u32,
    frame: u32,
) {
    let body = DIR_BODY_COLORS[dir as usize];
    // Slight vertical bob for walk animation: frame 0 & 2 shift body down 1px.
    let bob: i32 = if frame == 1 { 0 } else { 1 };

    // Head (rows 2..9, cols 7..17) – skin colored
    for y in 2..9 {
        for x in 7..17 {
            put(img, ox + x, (oy as i32 + y + bob) as u32, SKIN);
        }
    }
    // Hair top (rows 1..4)
    for y in 1..4 {
        for x in 7..17 {
            put(img, ox + x, (oy as i32 + y + bob) as u32, HAIR);
        }
    }
    // Eyes – only visible on Down/Left/Right (not Up, which faces away)
    // Row order: 0=Up, 1=Right, 2=Down, 3=Left
    if dir != 0 {
        let (lx, rx) = match dir {
            1 => (12, 15), // Right – eyes shifted right
            3 => (9, 12),  // Left – eyes shifted left
            _ => (9, 14),  // Down – centered
        };
        let ey = (oy as i32 + 5 + bob) as u32;
        put(img, ox + lx, ey, Rgba([20, 20, 60, 255]));
        put(img, ox + rx, ey, Rgba([20, 20, 60, 255]));
    }

    // Body (rows 10..24)
    for y in 10..24 {
        for x in 6..18 {
            put(img, ox + x, (oy as i32 + y + bob) as u32, body);
        }
    }

    // Arms – swing with frame for a simple walk cycle
    let arm_swing: i32 = match frame {
        0 => -1,
        2 => 1,
        _ => 0,
    };
    for y in 11..20 {
        let ay = (oy as i32 + y + bob + arm_swing) as u32;
        put(img, ox + 4, ay, body);
        put(img, ox + 5, ay, body);
        let ay2 = (oy as i32 + y + bob - arm_swing) as u32;
        put(img, ox + 18, ay2, body);
        put(img, ox + 19, ay2, body);
    }

    // Legs (rows 24..31) – alternate leg forward per frame
    let leg_offset: i32 = match frame {
        0 => -1,
        2 => 1,
        _ => 0,
    };
    // Left leg
    for y in 24..31 {
        let ly = (oy as i32 + y + bob + leg_offset) as u32;
        for x in 8..12 {
            put(img, ox + x, ly, Rgba([50, 50, 80, 255]));
        }
    }
    // Right leg
    for y in 24..31 {
        let ry = (oy as i32 + y + bob - leg_offset) as u32;
        for x in 13..17 {
            put(img, ox + x, ry, Rgba([50, 50, 80, 255]));
        }
    }

    // Direction indicator arrow on the chest
    draw_direction_arrow(img, ox + fw / 2, oy + 15 + bob as u32, dir);
}

fn draw_direction_arrow(img: &mut RgbaImage, cx: u32, cy: u32, dir: u32) {
    let white = Rgba([255, 255, 255, 255]);
    match dir {
        0 => {
            // Up arrow (row 0)
            put(img, cx, cy, white);
            put(img, cx, cy - 1, white);
            put(img, cx - 1, cy, white);
            put(img, cx + 1, cy, white);
        }
        1 => {
            // Right arrow (row 1)
            put(img, cx, cy, white);
            put(img, cx + 1, cy, white);
            put(img, cx, cy - 1, white);
            put(img, cx, cy + 1, white);
        }
        2 => {
            // Down arrow (row 2)
            put(img, cx, cy, white);
            put(img, cx, cy + 1, white);
            put(img, cx - 1, cy, white);
            put(img, cx + 1, cy, white);
        }
        3 => {
            // Left arrow (row 3)
            put(img, cx, cy, white);
            put(img, cx - 1, cy, white);
            put(img, cx, cy - 1, white);
            put(img, cx, cy + 1, white);
        }
        _ => {}
    }
}

fn put(img: &mut RgbaImage, x: u32, y: u32, c: Rgba<u8>) {
    if x < img.width() && y < img.height() {
        img.put_pixel(x, y, c);
    }
}

// ---------------------------------------------------------------------------
// Tileset generator (16x16 tiles in a grid)
// ---------------------------------------------------------------------------

/// Palette of base ground/terrain colors to pick from randomly.
const TERRAIN_PALETTE: [Rgba<u8>; 6] = [
    Rgba([80, 160, 60, 255]),   // grass green
    Rgba([60, 140, 50, 255]),   // dark grass
    Rgba([190, 170, 120, 255]), // sand / dirt
    Rgba([100, 100, 110, 255]), // stone
    Rgba([50, 100, 180, 255]),  // water blue
    Rgba([70, 50, 40, 255]),    // dark earth
];

fn generate_tileset(path: &str, cols: u32, rows: u32) {
    let tile = 16u32;
    let mut img = RgbaImage::new(tile * cols, tile * rows);
    let mut rng = rand::rng();

    for ty in 0..rows {
        for tx in 0..cols {
            let base = TERRAIN_PALETTE[rng.random_range(0..TERRAIN_PALETTE.len())];
            let ox = tx * tile;
            let oy = ty * tile;
            fill_tile(&mut img, ox, oy, tile, base, &mut rng);
        }
    }

    img.save(path).expect("failed to write tileset");
    println!("Wrote tileset ({cols}x{rows} tiles, {tile}x{tile}px each) to {path}");
}

fn fill_tile(img: &mut RgbaImage, ox: u32, oy: u32, size: u32, base: Rgba<u8>, rng: &mut impl Rng) {
    for y in 0..size {
        for x in 0..size {
            // Add per-pixel noise for a hand-drawn pixel art feel.
            let noise = rng.random_range(-15i16..=15);
            let r = (base.0[0] as i16 + noise).clamp(0, 255) as u8;
            let g = (base.0[1] as i16 + noise).clamp(0, 255) as u8;
            let b = (base.0[2] as i16 + noise).clamp(0, 255) as u8;
            img.put_pixel(ox + x, oy + y, Rgba([r, g, b, 255]));
        }
    }

    // Randomly scatter a few detail pixels (pebbles, grass blades, etc.)
    let detail_count = rng.random_range(0..6u32);
    for _ in 0..detail_count {
        let dx = rng.random_range(1..size - 1);
        let dy = rng.random_range(1..size - 1);
        let bright = rng.random_range(0..2) == 0;
        let shift: i16 = if bright { 30 } else { -30 };
        let p = img.get_pixel(ox + dx, oy + dy).0;
        let r = (p[0] as i16 + shift).clamp(0, 255) as u8;
        let g = (p[1] as i16 + shift).clamp(0, 255) as u8;
        let b = (p[2] as i16 + shift).clamp(0, 255) as u8;
        img.put_pixel(ox + dx, oy + dy, Rgba([r, g, b, 255]));
    }
}
