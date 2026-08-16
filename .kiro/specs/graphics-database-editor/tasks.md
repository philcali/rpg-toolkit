# Implementation Plan: Graphics Database Editor

## Overview

This plan implements graphics support for game entities in the RPG Toolkit. It extends the AssetManager with file-loading utilities, introduces a shared `EntityGraphics` struct for items and abilities, creates a thumbnail caching/rendering utility for the editor, and integrates thumbnail previews into all four entity editor panels (item, ability, enemy, character).

## Tasks

- [x] 1. Extend AssetManager with file-loading methods
  - [x] 1.1 Add `file_exists`, `load_file_bytes`, and `resolve_and_load` methods to AssetManager
    - Add three new methods to the existing `impl AssetManager` block in `crates/rpg-toolkit-common/src/asset.rs`
    - `file_exists(path: &Path) -> bool`: checks if path is a regular file
    - `load_file_bytes(path: &Path) -> Result<Vec<u8>, CommonError>`: reads raw bytes, errors on missing/directory/unreadable
    - `resolve_and_load(root: &Path, relative_path: &str) -> Result<Vec<u8>, CommonError>`: trims, validates non-empty, resolves, then loads
    - Add unit tests for: valid file read, directory rejection, missing file, empty path, whitespace-only path
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 1.7_

  - [x] 1.2 Write property test for AssetManager resolution round-trip
    - **Property 1: AssetManager resolution round-trip**
    - Create `crates/rpg-toolkit-common/tests/properties/asset_resolution_round_trip.rs`
    - For any valid relative path (non-empty, no `.`/`..`, ≤260 chars, forward-slash separators) and any root, resolving and stripping the root prefix reproduces the original path
    - **Validates: Requirements 10.1, 10.4**

  - [ ]* 1.3 Write property test for AssetManager resolution idempotence
    - **Property 2: AssetManager resolution idempotence**
    - Create `crates/rpg-toolkit-common/tests/properties/asset_resolution_idempotence.rs`
    - Calling `resolve_path` twice with the same args produces identical results
    - **Validates: Requirements 10.3**

  - [ ]* 1.4 Write property test for invalid path rejection
    - **Property 6: Invalid paths rejected by AssetManager**
    - Create `crates/rpg-toolkit-common/tests/properties/asset_invalid_path_rejection.rs`
    - Any empty, whitespace-only, or `.`/`..`-containing path returns an error from `resolve_and_load`
    - **Validates: Requirements 1.5, 1.6**

- [x] 2. Create EntityGraphics struct and integrate into Item and Ability
  - [x] 2.1 Create the `EntityGraphics` struct in a new `graphics.rs` module
    - Create `crates/rpg-toolkit-common/src/graphics.rs` with `EntityGraphics` struct
    - Implement `set_icon`, `clear_icon`, `has_icon` methods
    - Derive Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize
    - Trim whitespace, reject empty-after-trim, truncate to 260 chars in `set_icon`
    - Register `pub mod graphics;` in `crates/rpg-toolkit-common/src/lib.rs` and add re-export for `EntityGraphics`
    - Add unit tests for: set valid icon, reject empty/whitespace, truncation, clear, has_icon
    - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5_

  - [x] 2.2 Add `graphics: EntityGraphics` field to `Item` struct and registry methods
    - Add `#[serde(default)] pub graphics: EntityGraphics` field to `Item` struct in `crates/rpg-toolkit-common/src/item.rs`
    - Add `use crate::graphics::EntityGraphics;` import
    - Initialize `graphics: EntityGraphics::default()` in `create_item`
    - Add `set_icon(&mut self, id: &ItemId, path: &str)` and `clear_icon(&mut self, id: &ItemId)` methods to `ItemRegistry`
    - Add unit tests for: set_icon, clear_icon, item-not-found error, backward-compat serde (missing graphics field)
    - _Requirements: 3.1, 3.7, 3.8, 3.10_

  - [x] 2.3 Add `graphics: EntityGraphics` field to `Ability` struct and registry methods
    - Add `#[serde(default)] pub graphics: EntityGraphics` field to `Ability` struct in `crates/rpg-toolkit-common/src/ability.rs`
    - Add `use crate::graphics::EntityGraphics;` import
    - Initialize `graphics: EntityGraphics::default()` in ability creation
    - Add `set_icon(&mut self, id: &AbilityId, path: &str)` and `clear_icon(&mut self, id: &AbilityId)` methods to `AbilityRegistry`
    - Add unit tests for: set_icon, clear_icon, ability-not-found error, backward-compat serde (missing graphics field)
    - _Requirements: 4.1, 4.7, 4.8, 4.10_

  - [x] 2.4 Write property test for EntityGraphics serialization round-trip
    - **Property 3: EntityGraphics serialization round-trip**
    - Create `crates/rpg-toolkit-common/tests/properties/entity_graphics_round_trip.rs`
    - For any valid EntityGraphics (icon None or Some 1–260 chars, no traversal), serialize to JSON and back produces identical value
    - **Validates: Requirements 2.5, 10.2**

  - [x] 2.5 Write property test for EntityGraphics icon trim and truncation
    - **Property 7: EntityGraphics icon trim and truncation**
    - Create `crates/rpg-toolkit-common/tests/properties/entity_graphics_trim_truncation.rs`
    - For any string input to `set_icon`, stored value equals trimmed + truncated to 260; empty-after-trim returns error and icon unchanged
    - **Validates: Requirements 3.7, 4.7**

  - [ ]* 2.6 Write property test for Item graphics serialization round-trip
    - **Property 4: Item graphics serialization round-trip**
    - Create `crates/rpg-toolkit-common/tests/properties/item_graphics_round_trip.rs`
    - For any ItemRegistry with arbitrary EntityGraphics, serialize to JSON and back produces identical registry
    - **Validates: Requirements 3.10, 10.2**

  - [ ]* 2.7 Write property test for Ability graphics serialization round-trip
    - **Property 5: Ability graphics serialization round-trip**
    - Create `crates/rpg-toolkit-common/tests/properties/ability_graphics_round_trip.rs`
    - For any AbilityRegistry with arbitrary EntityGraphics, serialize to JSON and back produces identical registry
    - **Validates: Requirements 4.10, 10.2**

- [x] 3. Checkpoint - Ensure all common crate tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [x] 4. Implement ThumbnailCache utility in the editor crate
  - [x] 4.1 Create the `ThumbnailCache` struct and scaling logic
    - Create `crates/rpg-toolkit-editor/src/plugins/thumbnail.rs`
    - Implement `ThumbnailCache` with LRU eviction (max 128 entries), keyed by path string
    - Implement `compute_scaled_size(width, height, max_size) -> (f32, f32)` with aspect-ratio preservation and no-upscale logic
    - Implement `render_thumbnail(&mut self, ui, project_root, relative_path, max_size)` method
    - Implement `invalidate(&mut self, path: &str)` and `tick(&mut self)` methods
    - Add `pub mod thumbnail;` to `crates/rpg-toolkit-editor/src/plugins/mod.rs`
    - Register `ThumbnailCache` as a Bevy `Resource` (Default impl with 128 capacity)
    - On decode/load failure, render "Image not found" placeholder label
    - Add `image` crate dependency to `crates/rpg-toolkit-editor/Cargo.toml` if not present
    - Add unit tests for `compute_scaled_size`: 100×50 max 64, 32×32 max 64, 200×100 max 64, 1×1 max 64
    - _Requirements: 9.1, 9.2, 9.3, 9.4, 9.5, 9.6_

  - [ ]* 4.2 Write property test for thumbnail scaling aspect ratio
    - **Property 8: Thumbnail scaling preserves aspect ratio**
    - Add test in `crates/rpg-toolkit-editor` test module (or a dedicated test file)
    - For any (width > 0, height > 0, max_size > 0), computed size satisfies: both ≤ max_size, aspect ratio preserved within f32 tolerance, no upscaling
    - **Validates: Requirements 5.4, 6.4, 7.4, 8.4, 9.2**

- [x] 5. Integrate thumbnail previews into editor panels
  - [x] 5.1 Add icon section with thumbnail preview to Item Editor panel
    - Modify `crates/rpg-toolkit-editor/src/plugins/item_panel.rs`
    - Add `icon_buffer: String` field to `ItemPanelState`
    - Add "Icon" section in item detail view with: text input, "Browse..." button (rfd FileDialog filtered to png/jpg/jpeg), "Clear" button
    - Call `ThumbnailCache::render_thumbnail(...)` when `graphics.icon.is_some()` and file exists
    - Show "No icon assigned" label when icon is None
    - Show "Image not found" placeholder when path is set but file invalid
    - On browse select: compute relative path from project root, truncate to 260 chars, commit via `ItemRegistry::set_icon`
    - On clear: call `ItemRegistry::clear_icon`, invalidate cache entry
    - Mark project as having unsaved changes on icon modification
    - _Requirements: 3.2, 3.3, 3.4, 3.5, 3.6, 3.7, 3.8, 3.9, 5.1, 5.2, 5.3, 5.4, 5.5_

  - [x] 5.2 Add icon section with thumbnail preview to Ability Editor panel
    - Modify `crates/rpg-toolkit-editor/src/plugins/ability_panel.rs`
    - Add `icon_buffer: String` field to ability panel state
    - Add "Icon" section in ability detail view with: text input, "Browse..." button (rfd FileDialog filtered to png/jpg/jpeg), "Clear" button
    - Call `ThumbnailCache::render_thumbnail(...)` when `graphics.icon.is_some()` and file exists
    - Show "No icon assigned" label when icon is None
    - Show "Image not found" placeholder when path is set but file invalid
    - On browse select: compute relative path from project root, truncate to 260 chars, commit via `AbilityRegistry::set_icon`
    - On clear: call `AbilityRegistry::clear_icon`, invalidate cache entry
    - Mark project as having unsaved changes on icon modification
    - _Requirements: 4.2, 4.3, 4.4, 4.5, 4.6, 4.7, 4.8, 4.9, 6.1, 6.2, 6.3, 6.4, 6.5_

  - [x] 5.3 Add thumbnail preview to Enemy Editor portrait section
    - Modify `crates/rpg-toolkit-editor/src/plugins/enemy_panel.rs`
    - In the Portrait section, call `ThumbnailCache::render_thumbnail(...)` when `portrait.is_some()`
    - Show "Image not found" placeholder when portrait path is set but file is invalid
    - Update preview when portrait path changes
    - _Requirements: 7.1, 7.2, 7.3, 7.4, 7.5_

  - [x] 5.4 Add thumbnail previews to Character Editor visual asset slots
    - Modify `crates/rpg-toolkit-editor/src/plugins/character_panel.rs`
    - For each of the three visual asset slots (spritesheet, face portrait, status portrait), call `ThumbnailCache::render_thumbnail(...)` when the field is `Some`
    - Show "Image not found" placeholder when path is set but file is invalid
    - Do not render thumbnail or placeholder when path is None
    - Each slot's preview is independent (changing one does not affect the others)
    - _Requirements: 8.1, 8.2, 8.3, 8.4, 8.5, 8.6_

- [ ] 6. Final checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

## Notes

- Tasks marked with `*` are optional and can be skipped for faster MVP
- Each task references specific requirements for traceability
- Checkpoints ensure incremental validation
- Property tests validate universal correctness properties from the design document
- Unit tests validate specific examples and edge cases
- The `image` crate is used for decoding; `rfd` crate is used for native file dialogs (both likely already in editor dependencies)
- ThumbnailCache is registered as a Bevy Resource for shared access across panels
- Editor panel integration tasks (5.x) are manual/visual in nature but implemented as code changes to the panel systems

## Task Dependency Graph

```json
{
  "waves": [
    { "id": 0, "tasks": ["1.1", "2.1"] },
    { "id": 1, "tasks": ["1.2", "1.3", "1.4", "2.2", "2.3", "2.4", "2.5"] },
    { "id": 2, "tasks": ["2.6", "2.7", "4.1"] },
    { "id": 3, "tasks": ["4.2", "5.1", "5.2", "5.3", "5.4"] }
  ]
}
```
