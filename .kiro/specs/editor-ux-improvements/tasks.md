# Implementation Plan: Editor UX Improvements

## Overview

This plan implements seven editor UX improvements in incremental order: data model first (rpg-toolkit-common), then editor UI and rendering (rpg-toolkit-editor), then game renderer animation (rpg-toolkit-renderer). Each task builds on the previous, ensuring no orphaned code. Property-based tests validate correctness properties from the design document.

## Tasks

- [ ] 1. Add TileAnimation data model to rpg-toolkit-common
  - [x] 1.1 Define AnimationFrame and TileAnimation structs in a new `animation.rs` module
    - Create `crates/rpg-toolkit-common/src/animation.rs`
    - Define `AnimationFrame { col: u32, row: u32 }` with Clone, Debug, PartialEq, Eq, Serialize, Deserialize
    - Define `TileAnimation { frames: Vec<AnimationFrame>, frame_duration_ms: u32 }` with same derives
    - Export from `lib.rs`
    - _Requirements: 1.1_

  - [x] 1.2 Add `animations` field to TilesetMeta
    - Add `#[serde(default)] pub animations: Vec<TileAnimation>` to `TilesetMeta` in `tileset.rs`
    - Ensure backward compatibility with existing project files (empty vec default)
    - _Requirements: 1.1, 1.2, 1.3_

  - [x] 1.3 Implement `validate_tile_animation` function
    - Add validation function in `animation.rs` that checks: frames.len() >= 2, frame_duration_ms > 0, all frame coords within tileset bounds
    - Return `Result<(), CommonError>` with descriptive error messages
    - Add appropriate error variants to `CommonError` if needed
    - _Requirements: 1.5, 1.6, 1.7_

  - [x] 1.4 Implement `compute_animation_frame_index` pure function
    - Add `pub fn compute_animation_frame_index(elapsed_ms: u64, frame_duration_ms: u32, frame_count: usize) -> usize`
    - Formula: `(elapsed_ms / frame_duration_ms as u64) % frame_count as u64`
    - _Requirements: 3.1, 3.2, 3.3, 4.1, 4.2, 4.3_

  - [x] 1.5 Write property test: Animation Serialization Round-Trip
    - **Property 1: Animation Serialization Round-Trip**
    - Create `tests/properties/tile_animation.rs`
    - Generate arbitrary valid TileAnimation (frame_count >= 2, frame_duration_ms > 0, coords within bounds)
    - Serialize TilesetMeta with animations to JSON, deserialize, assert equality
    - **Validates: Requirements 1.2, 1.3, 1.4**

  - [x]* 1.6 Write property test: Animation Validation Correctness
    - **Property 2: Animation Validation Correctness**
    - Add to `tests/properties/tile_animation.rs`
    - Generate arbitrary TileAnimation and tileset dimensions
    - Assert validate_tile_animation returns Ok iff all conditions met (>= 2 frames, duration > 0, coords in bounds)
    - **Validates: Requirements 1.5, 1.6, 1.7**

  - [x]* 1.7 Write property test: Frame Cycling Correctness
    - **Property 3: Frame Cycling Correctness**
    - Add to `tests/properties/tile_animation.rs`
    - Generate arbitrary valid animation params and elapsed_ms
    - Assert compute_animation_frame_index returns value in [0, frame_count) matching the formula
    - **Validates: Requirements 3.1, 3.2, 3.3, 4.1, 4.2, 4.3**

  - [x]* 1.8 Write property test: Animation Lockstep Synchronization
    - **Property 4: Animation Lockstep Synchronization**
    - Add to `tests/properties/tile_animation.rs`
    - For two tile instances referencing the same animation, given same elapsed_ms, assert same frame index
    - **Validates: Requirements 3.4, 4.4**

- [x] 2. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [ ] 3. Implement searchable combobox widget and filter logic
  - [x] 3.1 Create `searchable_combobox` helper function
    - Add a new file `crates/rpg-toolkit-editor/src/plugins/searchable_combobox.rs`
    - Implement `searchable_combobox(ui, id_salt, current_label, items, search_buffer) -> Option<String>`
    - Items are `&[(String, String)]` pairs of (id, display_label)
    - Filter items case-insensitively by search query substring match
    - Sort items alphabetically when filter is empty
    - Show "No results" placeholder when filter matches nothing
    - Register module in `plugins/mod.rs`
    - _Requirements: 5.1, 5.2, 5.5, 6.1, 6.2, 6.5_

  - [x] 3.2 Write property test: Case-Insensitive Substring Filter
    - **Property 5: Case-Insensitive Substring Filter**
    - Create `tests/properties/searchable_filter.rs`
    - Generate arbitrary list of names and non-empty query
    - Assert filter returns exactly those names whose lowercase contains lowercase query
    - **Validates: Requirements 5.2, 6.2**

  - [x]* 3.3 Write property test: Alphabetical Sort with Empty Filter
    - **Property 6: Alphabetical Sort with Empty Filter**
    - Add to `tests/properties/searchable_filter.rs`
    - Generate arbitrary list of names with empty query
    - Assert result contains all names sorted case-insensitively
    - **Validates: Requirements 5.5, 6.5**

- [ ] 4. Implement palette tile scaling
  - [x] 4.1 Add `palette_tile_scale` field to EditorState
    - Add `pub palette_tile_scale: f32` to `EditorState` in `data/state.rs`
    - Default to 24.0 in `Default` impl
    - Add helper `pub fn clamp_palette_scale(scale: f32) -> f32` that clamps to [16.0, 128.0]
    - _Requirements: 7.1, 7.2, 7.3, 7.5_

  - [x] 4.2 Add zoom slider to tile palette UI
    - In `tile_palette_ui`, add an `egui::Slider` for `palette_tile_scale` with range 16.0..=128.0
    - Use `palette_tile_scale` as the `display_tile_size` in the tile grid rendering
    - Compute default as `max(tile_width, 24)` when switching tilesets (only if not already set by user)
    - _Requirements: 7.1, 7.4, 7.6_

  - [x]* 4.3 Write property test: Display Tile Size Clamping
    - **Property 7: Display Tile Size Clamping**
    - Create `tests/properties/palette_scale.rs`
    - Generate arbitrary f32 scale values
    - Assert clamp_palette_scale always returns value in [16.0, 128.0]
    - **Validates: Requirements 7.2, 7.3**

  - [x]* 4.4 Write property test: Default Display Tile Size Computation
    - **Property 8: Default Display Tile Size Computation**
    - Add to `tests/properties/palette_scale.rs`
    - For tile_width in {8, 16, 32, 64}, assert default equals max(tile_width, 24)
    - **Validates: Requirements 7.6**

- [x] 5. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [ ] 6. Replace map browser with searchable dropdown
  - [x] 6.1 Refactor `render_map_browser` to use `searchable_combobox`
    - Add a `search_buffer: String` field to `MapBrowserState`
    - Replace the `ScrollArea` list with `searchable_combobox` call
    - Pass sorted map entries as items (map_id, map_name)
    - Display currently active map name as the selected value
    - On selection, open the map tab and set as active
    - _Requirements: 5.1, 5.3, 5.4, 5.5_

  - [x] 6.2 Preserve context menu support on dropdown items
    - Ensure right-click context menus (Open, Rename, Delete) still work on each entry in the dropdown list
    - _Requirements: 5.6_

- [ ] 7. Replace tileset tab bar with searchable dropdown
  - [x] 7.1 Refactor `tile_palette_ui` tileset selector to use `searchable_combobox`
    - Add a tileset search buffer (can be stored in `EditorState` or local resource)
    - Replace the `horizontal_wrapped` tab bar with `searchable_combobox` call
    - Pass sorted tileset entries as items (tileset_id, file_path label)
    - Display currently active tileset file name as selected value
    - On selection, set `active_tileset_tab`
    - _Requirements: 6.1, 6.3, 6.4, 6.5_

- [ ] 8. Implement tile animation editor UI
  - [x] 8.1 Add AnimationEditorState resource
    - Create or extend editor state with `AnimationEditorState { active: bool, frames: Vec<AnimationFrame>, frame_duration_ms: u32 }`
    - Default frame_duration_ms to 200
    - Register as a Bevy resource
    - _Requirements: 2.1, 2.4_

  - [x] 8.2 Add animation editor toggle and panel to tile palette
    - Add "Animation Editor" toggle button in the tile palette panel
    - When active, show the animation sequence list with tile previews
    - Add numeric input for frame_duration_ms
    - Add remove button next to each frame entry
    - Add move-up/move-down buttons for reordering
    - Add Confirm and Cancel buttons
    - _Requirements: 2.1, 2.2, 2.3, 2.6, 2.7_

  - [x] 8.3 Implement animation editor confirm/cancel logic
    - On confirm: validate via `validate_tile_animation`, store in active tileset's `TilesetMeta.animations`
    - On cancel: discard in-progress frames, reset AnimationEditorState
    - Show inline error if validation fails
    - _Requirements: 2.5, 2.8_

  - [x] 8.4 Add live animation preview in editor panel
    - While animation editor is active, cycle through defined frames at specified frame_duration_ms
    - Display the current frame as a tile preview image
    - _Requirements: 2.9_

- [ ] 9. Implement tile animation rendering in editor canvas
  - [x] 9.1 Add EditorAnimationTick resource and tick system
    - Create `EditorAnimationTick { elapsed_ms: u64 }` resource
    - Add a system that increments `elapsed_ms` by delta time each frame
    - Register in the editor plugin
    - _Requirements: 3.1, 3.4_

  - [x] 9.2 Add AnimatedTile component and tag animated sprites
    - Define `AnimatedTile { tileset_id: TilesetId, animation_index: usize }` component
    - Modify `sync_tile_sprites` to detect tiles whose (col, row) matches the first frame of any animation in their tileset
    - Attach `AnimatedTile` component to those sprite entities
    - _Requirements: 3.1_

  - [x] 9.3 Implement `animate_editor_tiles` system
    - Add system that runs each frame after `sync_tile_sprites`
    - For each entity with `AnimatedTile`, compute current frame index via `compute_animation_frame_index`
    - Update the sprite's `TextureAtlas.index` to the computed frame's atlas position
    - _Requirements: 3.1, 3.2, 3.3, 3.4_

- [ ] 10. Implement tile animation rendering in game renderer
  - [x] 10.1 Add RendererAnimationTick resource and tick system
    - Create `RendererAnimationTick { elapsed_ms: u64 }` resource in rpg-toolkit-renderer
    - Add a system that increments `elapsed_ms` by delta time each frame
    - Register in `ProjectRendererPlugin`
    - _Requirements: 4.1, 4.4_

  - [x] 10.2 Tag animated tile sprites in `sync_map_sprites`
    - Modify `sync_map_sprites` to detect tiles whose (col, row) matches the first frame of any animation
    - Attach `AnimatedTile` component to those sprite entities
    - _Requirements: 4.1_

  - [x] 10.3 Implement `animate_renderer_tiles` system
    - Add system that runs each frame
    - For each entity with `AnimatedTile`, compute current frame index via `compute_animation_frame_index`
    - Update the sprite's `TextureAtlas.index` to the computed frame's atlas position
    - Schedule after `sync_map_sprites` in the Update set
    - _Requirements: 4.1, 4.2, 4.3, 4.4_

- [x] 11. Register property test files in tests/properties/Cargo.toml
  - Add `[[test]]` entries for `tile_animation`, `searchable_filter`, and `palette_scale`
  - Ensure `rpg-toolkit-common` dependency is available for the new test files
  - _Requirements: 1.4, 1.5, 1.6, 1.7, 3.1, 3.2, 3.3, 3.4, 4.1, 4.2, 4.3, 4.4, 5.2, 5.5, 6.2, 6.5, 7.2, 7.3, 7.6_

- [x] 12. Final checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [x] 13. Lint and format check
  - Run `cargo fmt --all -- --check` and fix any formatting issues
  - Run `cargo check` and resolve any compile errors
  - Run `cargo clippy --all-targets -- -W warnings` and resolve all warnings
  - All three commands must pass cleanly with zero warnings/errors

## Notes

- Tasks marked with `*` are optional and can be skipped for faster MVP
- Each task references specific requirements for traceability
- Checkpoints ensure incremental validation
- Property tests validate universal correctness properties from the design document
- Unit tests validate specific examples and edge cases
- The shared `compute_animation_frame_index` function in rpg-toolkit-common ensures WYSIWYG behavior between editor and renderer
- The `searchable_combobox` widget is reused for both map browser and tileset selector
