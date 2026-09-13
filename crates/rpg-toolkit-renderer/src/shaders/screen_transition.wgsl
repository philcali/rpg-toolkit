// Fullscreen screen-transition overlay shader.
//
// This material is drawn on a UI node that covers the whole screen. It outputs
// the transition `color`, computing a per-pixel alpha that describes how much of
// the scene is covered at the current animation `progress`.
//
// Convention: `progress` runs 0 -> 1 where
//   progress = 0.0  => the effect fully COVERS the screen (alpha = 1 everywhere)
//   progress = 1.0  => the effect has fully REVEALED the screen (alpha = 0)
//
// The In/Out direction is handled on the CPU by ramping `progress` up (reveal)
// or down (obscure); the shader is direction-agnostic and only needs progress.
//
// `kind` selects the visual style and MUST stay in sync with
// `TransitionKind::shader_index` in rpg-toolkit-common:
//   0 = Fade, 1 = Mosaic, 2 = DistortionWave, 3 = Whirlpool

#import bevy_ui::ui_vertex_output::UiVertexOutput

const KIND_FADE: u32 = 0u;
const KIND_MOSAIC: u32 = 1u;
const KIND_DISTORTION_WAVE: u32 = 2u;
const KIND_WHIRLPOOL: u32 = 3u;

const PI: f32 = 3.14159265359;
const TAU: f32 = 6.28318530718;

// Number of mosaic blocks along the shortest screen axis at full coverage.
const MOSAIC_BLOCKS: f32 = 24.0;

// group(1) is the material bind group for UiMaterial.
// color: the RGBA cover color. Its .a is the maximum opacity (usually 1.0).
@group(1) @binding(0) var<uniform> color: vec4<f32>;
// params.x = progress in [0,1]; params.y = kind (as f32); params.z, params.w reserved.
// Packed into a vec4 because WebGL2 requires 16-byte-aligned uniforms.
@group(1) @binding(1) var<uniform> params: vec4<f32>;

// Cheap hash -> [0,1) for a 2D cell coordinate, used to stagger mosaic blocks.
fn hash21(p: vec2<f32>) -> f32 {
    let h = dot(p, vec2<f32>(127.1, 311.7));
    return fract(sin(h) * 43758.5453123);
}

// Coverage alpha for a plain fade: uniform across the screen.
fn cover_fade(progress: f32) -> f32 {
    // progress 0 => covered (1), progress 1 => revealed (0)
    return 1.0 - progress;
}

// Coverage alpha for the mosaic dissolve. The screen is divided into a grid of
// blocks; each block is assigned a random threshold and flips from covered to
// revealed as progress crosses that threshold, with a soft edge so blocks fade
// individually rather than popping.
fn cover_mosaic(uv: vec2<f32>, size: vec2<f32>, progress: f32) -> f32 {
    // Keep blocks square by scaling the grid to the aspect ratio.
    let aspect = size.x / max(size.y, 1.0);
    let cells = vec2<f32>(MOSAIC_BLOCKS * aspect, MOSAIC_BLOCKS);
    let cell = floor(uv * cells);
    let threshold = hash21(cell);
    // Reveal window: block becomes transparent as progress passes its threshold.
    let edge = 0.15;
    // reveal = 1 when fully revealed (transparent), 0 when covered.
    let reveal = smoothstep(threshold, threshold + edge, progress);
    return 1.0 - reveal;
}

// Coverage alpha for a horizontal distortion wave. A sine wave sweeps a soft
// vertical boundary across the screen; the boundary position is warped per-row
// so the reveal edge ripples instead of being a straight line.
fn cover_distortion_wave(uv: vec2<f32>, progress: f32) -> f32 {
    let amplitude = 0.08;
    let frequency = 5.0;
    // Vertical ripple offset applied to the horizontal reveal boundary.
    let wave = sin(uv.y * frequency * TAU + progress * TAU) * amplitude;
    // Expand the boundary range slightly so amplitude at the extremes still
    // fully covers/reveals at progress 0/1.
    let boundary = progress * (1.0 + 2.0 * amplitude) - amplitude + wave;
    // Pixels to the left of the boundary are revealed.
    let reveal = smoothstep(boundary - 0.02, boundary + 0.02, uv.x);
    // reveal here == 1 where still covered; invert so 1 == revealed/transparent.
    return reveal;
}

// Coverage alpha for a whirlpool swirl. Coverage is driven by an expanding
// spiral: the reveal radius grows from the center with progress, and the
// threshold is modulated by the pixel's angle so the boundary spirals.
fn cover_whirlpool(uv: vec2<f32>, size: vec2<f32>, progress: f32) -> f32 {
    // Center-relative, aspect-corrected coordinates.
    let aspect = size.x / max(size.y, 1.0);
    var p = uv - vec2<f32>(0.5, 0.5);
    p.x = p.x * aspect;
    let radius = length(p) / (0.5 * sqrt(aspect * aspect + 1.0));
    let angle = atan2(p.y, p.x);
    // Spiral arms: shift the effective progress by the angle so the reveal
    // boundary rotates as it expands.
    let swirl = 3.0;
    let spiral = fract((angle / TAU) + progress * swirl);
    // Combine an expanding radius with the angular spiral term.
    let boundary = progress * 1.3 + (spiral - 0.5) * 0.25;
    let reveal = smoothstep(boundary - 0.05, boundary + 0.05, radius);
    // reveal == 1 where radius is beyond the boundary (still covered).
    return reveal;
}

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    let progress = clamp(params.x, 0.0, 1.0);
    let kind = u32(params.y + 0.5);

    var coverage: f32;
    if kind == KIND_MOSAIC {
        coverage = cover_mosaic(in.uv, in.size, progress);
    } else if kind == KIND_DISTORTION_WAVE {
        coverage = cover_distortion_wave(in.uv, progress);
    } else if kind == KIND_WHIRLPOOL {
        coverage = cover_whirlpool(in.uv, in.size, progress);
    } else {
        coverage = cover_fade(progress);
    }

    coverage = clamp(coverage, 0.0, 1.0);
    return vec4<f32>(color.rgb, color.a * coverage);
}
