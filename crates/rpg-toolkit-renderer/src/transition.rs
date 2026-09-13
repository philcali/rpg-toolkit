//! Shader-driven fullscreen screen transitions (fade, mosaic, distortion wave,
//! whirlpool, …).
//!
//! A transition is rendered by a single fullscreen UI node carrying a
//! [`ScreenTransitionMaterial`]. The material's fragment shader
//! (`shaders/screen_transition.wgsl`, embedded in the binary) outputs the cover
//! color with a per-pixel alpha computed from a normalized `progress` value and
//! the selected [`TransitionKind`]. The CPU side only animates `progress` and
//! chooses the direction; all visual work happens on the GPU.

use bevy::asset::embedded_asset;
use bevy::prelude::*;
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::ShaderRef;
use rpg_toolkit_common::{TransitionDirection, TransitionKind};

/// `embedded://` path to the transition shader, embedded into the binary so the
/// renderer is self-contained and does not require consumers to ship an
/// `assets/shaders` directory.
pub const SCREEN_TRANSITION_SHADER_PATH: &str =
    "embedded://rpg_toolkit_renderer/shaders/screen_transition.wgsl";

/// UI material for the fullscreen transition overlay.
///
/// `color` is the RGBA cover color (its alpha is the maximum opacity). `params`
/// packs the animation state into a `Vec4` because WebGL2 requires 16-byte
/// aligned uniforms: `params.x` = progress in `[0, 1]`, `params.y` = the
/// [`TransitionKind::shader_index`] as an `f32`. The remaining lanes are
/// reserved for future effects.
#[derive(AsBindGroup, Asset, TypePath, Debug, Clone)]
pub struct ScreenTransitionMaterial {
    #[uniform(0)]
    pub color: Vec4,
    #[uniform(1)]
    pub params: Vec4,
}

impl ScreenTransitionMaterial {
    /// Builds a material for the given kind, cover color and initial progress.
    pub fn new(kind: TransitionKind, color: [f32; 4], progress: f32) -> Self {
        Self {
            color: Vec4::from_array(color),
            params: Vec4::new(progress, kind.shader_index() as f32, 0.0, 0.0),
        }
    }

    /// Updates the animation progress lane in place.
    pub fn set_progress(&mut self, progress: f32) {
        self.params.x = progress;
    }
}

impl UiMaterial for ScreenTransitionMaterial {
    fn fragment_shader() -> ShaderRef {
        SCREEN_TRANSITION_SHADER_PATH.into()
    }
}

/// Convenience alias for the material-node component used on the overlay entity.
pub type ScreenTransitionNode = MaterialNode<ScreenTransitionMaterial>;

/// Maps a transition direction and normalized time `t` (0..1, elapsed/duration)
/// to the shader `progress` value.
///
/// Recall the shader convention: `progress = 0` fully covers the screen and
/// `progress = 1` fully reveals it.
/// - `In` reveals the scene, so progress ramps 0 -> 1 as `t` goes 0 -> 1.
/// - `Out` obscures the scene, so progress ramps 1 -> 0 as `t` goes 0 -> 1.
pub fn direction_progress(direction: TransitionDirection, t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    match direction {
        TransitionDirection::In => t,
        TransitionDirection::Out => 1.0 - t,
    }
}

/// The `progress` value a transition should hold once complete (or when applied
/// instantly with duration 0).
///
/// - A completed `In` transition has fully revealed the scene: progress = 1.
/// - A completed `Out` transition fully covers the scene: progress = 0.
pub fn final_progress(direction: TransitionDirection) -> f32 {
    match direction {
        TransitionDirection::In => 1.0,
        TransitionDirection::Out => 0.0,
    }
}

/// Registers the embedded transition shader and the UI material plugin.
pub struct ScreenTransitionPlugin;

impl Plugin for ScreenTransitionPlugin {
    fn build(&self, app: &mut App) {
        // The path is relative to this source file; the leading crate-source
        // prefix is stripped and replaced by the crate name in the
        // `embedded://` URL (see SCREEN_TRANSITION_SHADER_PATH).
        embedded_asset!(app, "shaders/screen_transition.wgsl");
        app.add_plugins(UiMaterialPlugin::<ScreenTransitionMaterial>::default());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn direction_progress_in_ramps_up() {
        assert_eq!(direction_progress(TransitionDirection::In, 0.0), 0.0);
        assert_eq!(direction_progress(TransitionDirection::In, 1.0), 1.0);
    }

    #[test]
    fn direction_progress_out_ramps_down() {
        assert_eq!(direction_progress(TransitionDirection::Out, 0.0), 1.0);
        assert_eq!(direction_progress(TransitionDirection::Out, 1.0), 0.0);
    }

    #[test]
    fn direction_progress_clamps() {
        assert_eq!(direction_progress(TransitionDirection::In, -0.5), 0.0);
        assert_eq!(direction_progress(TransitionDirection::In, 2.0), 1.0);
    }

    #[test]
    fn final_progress_matches_direction() {
        assert_eq!(final_progress(TransitionDirection::In), 1.0);
        assert_eq!(final_progress(TransitionDirection::Out), 0.0);
    }

    #[test]
    fn material_packs_kind_into_params() {
        let m = ScreenTransitionMaterial::new(TransitionKind::Whirlpool, [0.0, 0.0, 0.0, 1.0], 0.0);
        assert_eq!(m.params.y, TransitionKind::Whirlpool.shader_index() as f32);
    }
}
