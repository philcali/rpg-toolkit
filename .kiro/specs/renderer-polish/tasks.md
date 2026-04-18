# Implementation Plan: Renderer Polish

## Overview

Convert the feature design into a series of prompts for a code-generation LLM that will implement each step with incremental progress. Make sure that each prompt builds on the previous prompts, and ends with wiring things together. There should be no hanging or orphaned code that isn't integrated into a previous step. Focus ONLY on tasks that involve writing, modifying, or testing code.

The implementation proceeds in five stages: (1) add the new `PixelScaleConfig` resource and update `AnimationConfig`, (2) update the `walk_animation_frame` function and its existing property test, (3) add sprite scaling to player and NPC spawning, (4) implement the `apply_pixel_scale` system and update camera bounds, and (5) wire everything into the plugin and write remaining property tests.

## Tasks

- [x] 1. Add `PixelScaleConfig` resource and update `AnimationConfig`
  - [x] 1.1 Create `PixelScaleConfig` resource with `PixelScaleMode` enum in `crates/rpg-toolkit-renderer/src/resources.rs`
    - Add `PixelScaleMode` enum with `ZoomToFit` and `Fixed(u32)` variants
    - Add `PixelScaleConfig` struct with `mode: PixelScaleMode` and `effective_scale: u32` fields
    - Implement `Default` for `PixelScaleConfig` (mode=ZoomToFit, effective_scale=1)
    - Export the new types from `crates/rpg-toolkit-renderer/src/lib.rs`
    - _Requirements: 1.1_

  - [x] 1.2 Add `clamped_frame_duration()` method to `AnimationConfig` in `crates/rpg-toolkit-renderer/src/resources.rs`
    - Add `clamped_frame_duration(&self) -> f32` that returns `self.frame_duration.max(0.01)`
    - _Requirements: 5.4_

  - [ ]* 1.3 Write property test for frame duration clamping (Property 7)
    - **Property 7: Frame duration clamping**
    - Test that for any `f32` value `d`, `clamped_frame_duration(d)` returns `max(d, 0.01)`
    - For `d <= 0.0` result is `0.01`; for `d >= 0.01` result is `d`
    - Generator: `d` in `-10.0..10.0`
    - Add a new `[[test]]` entry in `tests/properties/Cargo.toml` and add `rpg-toolkit-renderer` as a dev-dependency
    - Create `tests/properties/renderer_polish.rs` for all renderer-polish property tests
    - **Validates: Requirements 5.4**

- [x] 2. Update `walk_animation_frame` to use `[0, 1, 2, 1]` pattern
  - [x] 2.1 Modify `walk_animation_frame` in `crates/rpg-toolkit-common/src/spritesheet.rs`
    - Change from `(elapsed / frame_duration).floor() as usize % 3` to indexing into `WALK_PATTERN: [usize; 4] = [0, 1, 2, 1]` using `% 4`
    - Keep the function signature identical: `pub fn walk_animation_frame(elapsed: f32, frame_duration: f32) -> usize`
    - _Requirements: 3.1, 3.4, 3.5_

  - [x] 2.2 Update `animate_player_sprite` in `crates/rpg-toolkit-renderer/src/systems/player.rs` to use `animation_config.clamped_frame_duration()` instead of `animation_config.frame_duration`
    - _Requirements: 5.2, 5.4_

  - [x] 2.3 Update existing property test in `tests/properties/walk_animation.rs` to validate the new `[0, 1, 2, 1]` pattern instead of the old `[0, 1, 2]` cycle
    - Change expected formula from `(elapsed / frame_duration).floor() as usize % 3` to `[0, 1, 2, 1][(elapsed / frame_duration).floor() as usize % 4]`
    - Update the test comment header to reference `renderer-polish` feature and Property 5
    - _Requirements: 3.4, 3.5_

  - [ ]* 2.4 Write property test for walk animation step pattern (Property 5)
    - **Property 5: Walk animation frame follows [0, 1, 2, 1] pattern**
    - Test that `walk_animation_frame(elapsed, frame_duration)` returns `[0, 1, 2, 1][floor(elapsed / frame_duration) % 4]`
    - Test that returned value is always one of 0, 1, or 2
    - Generator: `elapsed` in `0.0..100.0`, `frame_duration` in `0.01..2.0`
    - Add to `tests/properties/walk_animation.rs`
    - **Validates: Requirements 3.1, 3.4, 3.5**

  - [ ]* 2.5 Write property test for animation speed (Property 6)
    - **Property 6: Smaller frame duration produces faster animation**
    - Test that for `fd1 < fd2`, `floor(elapsed / fd1) >= floor(elapsed / fd2)` for all non-negative `elapsed`
    - Generator: `elapsed` in `0.0..100.0`, `fd1` in `0.01..1.0`, `fd2` in `fd1..2.0`
    - Add to `tests/properties/walk_animation.rs`
    - **Validates: Requirements 5.2, 5.3**

- [x] 3. Checkpoint — Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [x] 4. Add sprite scaling to player and NPC spawning
  - [x] 4.1 Modify `spawn_player` in `crates/rpg-toolkit-renderer/src/systems/player.rs` to apply sprite scale
    - When spawning with a spritesheet, compute `sprite_scale = map.tile_width as f32 / ss.sprite_width as f32` using the `CharacterSpritesheet` metadata
    - Apply `.with_scale(Vec3::splat(sprite_scale))` to the `Transform`
    - Access `CharacterSpritesheet` from `project_data.project_file.spritesheets` using the player's spritesheet ID
    - _Requirements: 2.1, 2.3, 2.4_

  - [x] 4.2 Modify `spawn_npc_sprites` in `crates/rpg-toolkit-renderer/src/systems/map_render.rs` to apply sprite scale
    - Compute `sprite_scale = map.tile_width as f32 / spritesheet.sprite_width as f32` for each NPC
    - Apply `.with_scale(Vec3::splat(sprite_scale))` to the NPC's `Transform`
    - Access `CharacterSpritesheet` metadata from `project_data.project_file.spritesheets` using `npc.spritesheet_id`
    - _Requirements: 2.2, 2.3, 2.4_

  - [ ]* 4.3 Write property test for sprite scale proportionality (Property 4)
    - **Property 4: Sprite scale preserves tile-width proportionality**
    - Test that for `tw > 0` and `sw > 0`, the scale equals `tw / sw`, and `sw * scale == tw`
    - Generator: `tile_width` in `{8, 16, 32, 64}`, `sprite_width` in `1u32..128`
    - Add to `tests/properties/renderer_polish.rs`
    - **Validates: Requirements 2.1, 2.3, 2.4**

- [x] 5. Implement `apply_pixel_scale` system and update camera bounds
  - [x] 5.1 Create the `apply_pixel_scale` system in `crates/rpg-toolkit-renderer/src/systems/camera.rs`
    - Add a pure helper function `compute_zoom_to_fit(win_w: f32, win_h: f32, map_pixel_w: f32, map_pixel_h: f32) -> u32` that returns the largest integer `s >= 1` such that `map_pixel_w * s <= win_w` and `map_pixel_h * s <= win_h`, minimum 1
    - Implement `apply_pixel_scale` system that reads `PixelScaleConfig`, `RendererProjectData`, `RendererState`, window, and camera projection
    - For `ZoomToFit` mode: call `compute_zoom_to_fit`, store result in `pixel_scale.effective_scale`
    - For `Fixed(n)` mode: set `effective_scale = max(n, 1)`
    - Set camera `OrthographicProjection.scale = 1.0 / effective_scale as f32`
    - Export `apply_pixel_scale` from `crates/rpg-toolkit-renderer/src/systems/mod.rs` and `crates/rpg-toolkit-renderer/src/lib.rs`
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.7, 1.8_

  - [x] 5.2 Modify `update_camera` in `crates/rpg-toolkit-renderer/src/systems/camera.rs` to use scaled viewport
    - Add `Res<PixelScaleConfig>` parameter
    - Compute `let scale = pixel_scale.effective_scale as f32;` then `half_vp_w = window.width() / scale / 2.0` and `half_vp_h = window.height() / scale / 2.0`
    - Use these scaled viewport halves for all clamping and centering logic
    - _Requirements: 1.5, 1.6, 6.1, 6.2, 6.3, 6.4_

  - [ ]* 5.3 Write property test for zoom-to-fit (Property 1)
    - **Property 1: Zoom-to-fit computes the largest fitting integer scale**
    - Test that `compute_zoom_to_fit` returns the largest `s >= 1` where `map_w * s <= win_w` and `map_h * s <= win_h`
    - Verify that `s + 1` would violate at least one constraint (unless s == 1 and even s == 1 violates)
    - Generator: `win_w/h` in `100.0..4000.0`, `map_w/h` in `1.0..10000.0`
    - Add to `tests/properties/renderer_polish.rs`
    - **Validates: Requirements 1.2, 1.8**

  - [ ]* 5.4 Write property test for fixed pixel scale projection (Property 2)
    - **Property 2: Fixed pixel scale produces correct projection**
    - Test that `effective_scale = max(n, 1)` and projection scale equals `1.0 / max(n, 1)`
    - Generator: `n` in `-10i32..100`
    - Add to `tests/properties/renderer_polish.rs`
    - **Validates: Requirements 1.4, 1.7**

  - [ ]* 5.5 Write property test for camera clamping (Property 3)
    - **Property 3: Camera clamping keeps viewport within map bounds**
    - Test clamping logic for both axes: when `map_dim > vp_dim`, camera stays within `[vp_dim/2, map_dim - vp_dim/2]`; when `map_dim <= vp_dim`, camera centers at `map_dim / 2`
    - Test with Y-axis using negative convention (camera centers at `-map_h / 2`)
    - Generator: `player_pos` in `0.0..1000.0`, `map_dim` in `1.0..2000.0`, `vp_dim` in `1.0..2000.0`
    - Add to `tests/properties/renderer_polish.rs`
    - **Validates: Requirements 1.6, 6.1, 6.2, 6.3**

- [x] 6. Checkpoint — Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [x] 7. Wire everything into the plugin
  - [x] 7.1 Register `PixelScaleConfig` resource and `apply_pixel_scale` system in `ProjectRendererPlugin::build` in `crates/rpg-toolkit-renderer/src/lib.rs`
    - Add `.init_resource::<PixelScaleConfig>()` in the resources section
    - Add `apply_pixel_scale` system in the Update schedule, ordered after `spawn_npc_sprites` and before `update_camera`
    - Update imports to include `PixelScaleConfig`, `apply_pixel_scale`
    - _Requirements: 1.1, 1.3_

- [x] 8. Final checkpoint — Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

## Notes

- Tasks marked with `*` are optional and can be skipped for faster MVP
- Each task references specific requirements for traceability
- Checkpoints ensure incremental validation
- Property tests validate universal correctness properties from the design document
- Unit tests validate specific examples and edge cases
- The existing `walk_animation.rs` property test is updated in-place (task 2.3) since it tests the same function
- New renderer-polish property tests go in a new `tests/properties/renderer_polish.rs` file
- All code is Rust, using the Bevy ECS framework consistent with the existing codebase
