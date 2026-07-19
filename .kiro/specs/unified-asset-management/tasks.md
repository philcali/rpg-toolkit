# Implementation Plan: Unified Asset Management

## Overview

This plan implements a unified asset management layer in `rpg-toolkit-common` that consolidates asset registration, path resolution, project loading/saving, and validation into a single `AssetManager` module. It also removes legacy flat JSON format support from the editor and launcher. The implementation follows an incremental approach: core data types first, then business logic, then consumer integration, and finally legacy removal.

## Tasks

- [x] 1. Define core asset types and error variants
  - [x] 1.1 Create `crates/rpg-toolkit-common/src/asset.rs` with `AssetCategory` type alias, category constants, `AssetReference` struct, `AssetRegistry` struct, `AssetValidationError`, `AssetWarning`, and `ProjectSource` enum
    - Define `pub type AssetCategory = String`
    - Define constants `CATEGORY_TILESET`, `CATEGORY_SPRITESHEET`, `CATEGORY_FACE_PORTRAIT`
    - Define `AssetReference` with `id: String`, `relative_path: String`, `category: AssetCategory` (derive Serialize, Deserialize, Clone, Debug, PartialEq, Eq)
    - Define `AssetRegistry` with `entries: HashMap<String, AssetReference>` and implement `register`, `get`, `remove`, `iter`, `len`, `is_empty`
    - Define `AssetValidationError` struct with `asset_id`, `category`, `resolved_path` fields
    - Define `AssetWarning` struct with `asset_id`, `category`, `message` fields
    - Define `ProjectSource` enum with `Directory(PathBuf)` and `Zip(PathBuf)` variants only
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 1.7_

  - [x] 1.2 Add new error variants to `CommonError` in `crates/rpg-toolkit-common/src/error.rs`
    - Add `AssetRegistryError(String)` variant
    - Add `AssetPathError(String)` variant
    - Add `UnsupportedFormat(String)` variant
    - _Requirements: 1.3, 1.6, 2.5, 3.4_

  - [x] 1.3 Register the `asset` module in `crates/rpg-toolkit-common/src/lib.rs` and add public exports
    - Add `pub mod asset;` to lib.rs
    - Export key types: `AssetCategory`, `AssetReference`, `AssetRegistry`, `AssetValidationError`, `AssetWarning`, `ProjectSource`, and category constants
    - _Requirements: 1.1, 1.4_

- [x] 2. Implement AssetRegistry logic and property tests
  - [x] 2.1 Implement `AssetRegistry` methods in `asset.rs`
    - `register`: validate id length (1–128 chars), reject duplicates with `AssetRegistryError`, insert entry
    - `get`: return reference or `AssetRegistryError` for not-found
    - `remove`: remove and return entry or error if not found
    - `iter`, `len`, `is_empty`: standard collection accessors
    - _Requirements: 1.1, 1.2, 1.3, 1.5, 1.6, 1.7_

  - [x] 2.2 Write property test: Registration round-trip
    - **Property 1: Registration round-trip**
    - **Validates: Requirements 1.1, 1.2**

  - [ ]* 2.3 Write property test: Duplicate registration rejection
    - **Property 2: Duplicate registration rejection**
    - **Validates: Requirements 1.3**

  - [ ]* 2.4 Write property test: Open category acceptance
    - **Property 3: Open category acceptance**
    - **Validates: Requirements 1.5, 1.7**

  - [ ]* 2.5 Write property test: Not-found retrieval error
    - **Property 4: Not-found retrieval error**
    - **Validates: Requirements 1.6**

- [x] 3. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [x] 4. Implement path resolution and format detection
  - [x] 4.1 Implement `AssetManager::resolve_path` and `AssetManager::resolve_all` in `asset.rs`
    - `resolve_path`: join root with relative path, reject if relative path is empty (return warning), reject if contains `..` components that escape root (return `AssetPathError`), canonicalize result
    - `resolve_all`: iterate registry entries, call `resolve_path` for each, collect resolved paths and warnings for empty/invalid entries
    - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5_

  - [x] 4.2 Implement `AssetManager::detect_source` in `asset.rs`
    - Check if path is a directory → `ProjectSource::Directory`
    - Check if path extension is `.rpg` → `ProjectSource::Zip`
    - Otherwise return `UnsupportedFormat` error with the path
    - _Requirements: 3.1, 3.4, 5.1_

  - [ ]* 4.3 Write property test: Path resolution correctness
    - **Property 5: Path resolution correctness**
    - **Validates: Requirements 2.1, 2.2, 2.3**

  - [ ]* 4.4 Write property test: Path traversal rejection
    - **Property 6: Path traversal rejection**
    - **Validates: Requirements 2.5**

  - [ ]* 4.5 Write property test: Format detection correctness
    - **Property 7: Format detection correctness**
    - **Validates: Requirements 3.1, 3.4, 5.1**

- [x] 5. Implement AssetManager load and save operations
  - [x] 5.1 Implement `AssetManager::new`, `set_category_dir`, and `registry_from_project_file`
    - `new()`: initialize with default category→directory mappings (tileset→`tilesets/`, spritesheet→`data/`, face_portrait→`data/`)
    - `set_category_dir`: update the mapping for a given category
    - `registry_from_project_file`: build an `AssetRegistry` from a `ProjectFile`'s tilesets, spritesheets, and face_portraits maps
    - _Requirements: 4.1, 1.4_

  - [x] 5.2 Implement `AssetManager::load_project`
    - Detect source format via `detect_source`
    - For Directory: load `ProjectManifest` from `manifest.json`, call `into_project_file`, populate registry from project file, validate asset files exist
    - For Zip: extract to temp dir, load manifest from extracted dir, call `into_project_file`, populate registry, validate
    - Return `(ProjectFile, Vec<AssetValidationError>)` — validation errors are non-aborting
    - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5, 3.6, 3.7, 6.1, 6.2, 6.3, 6.4_

  - [x] 5.3 Implement `AssetManager::save_project`
    - For Directory target: write asset files to category-mapped subdirectories, normalize paths (forward slashes, no leading slash), write manifest and maps
    - For Zip target: create ZIP archive with manifest, maps, and asset files using relative paths matching directory layout
    - If asset file missing on disk during save: skip it, emit `AssetWarning`, continue
    - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5_

  - [x] 5.4 Implement `AssetManager::validate_assets`
    - Resolve all registry entries, check each resolved path exists on filesystem
    - Return exactly K `AssetValidationError` entries for K missing files
    - Include asset_id, category, and resolved_path in each error
    - _Requirements: 6.1, 6.2, 6.3, 6.4_

  - [ ]* 5.5 Write property test: Validation reports exactly missing files
    - **Property 8: Validation reports exactly missing files**
    - **Validates: Requirements 3.7, 6.1, 6.2, 6.3, 6.4**

  - [ ]* 5.6 Write property test: Path normalization
    - **Property 9: Path normalization**
    - **Validates: Requirements 4.3**

- [~] 6. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [ ] 7. Implement round-trip property tests
  - [ ]* 7.1 Write property test: Directory format round-trip
    - **Property 10: Directory format round-trip**
    - **Validates: Requirements 7.1**

  - [ ]* 7.2 Write property test: ZIP format round-trip
    - **Property 11: ZIP format round-trip**
    - **Validates: Requirements 7.2**

  - [ ]* 7.3 Write property test: ZIP content byte-identity
    - **Property 12: ZIP content byte-identity**
    - **Validates: Requirements 7.3**

- [x] 8. Integrate AssetManager into the Editor
  - [x] 8.1 Refactor `crates/rpg-toolkit-editor/src/plugins/serialization.rs` to use `AssetManager`
    - Replace the local `ProjectSource` enum and `detect_project_source` function with `AssetManager::detect_source`
    - Replace `load_project_from_dir` and `load_project_from_zip` with calls to `AssetManager::load_project`
    - Replace `save_to_directory` and `save_to_zip` with calls to `AssetManager::save_project`
    - Remove `prepare_assets_for_save`, `to_project_file_for_save`, `resolve_spritesheet_paths` helper functions (logic now in AssetManager)
    - _Requirements: 2.3, 3.2, 3.3, 3.5, 4.1, 4.2_

  - [x] 8.2 Remove legacy JSON support from the Editor
    - Remove `load_project_from_json` function from `serialization.rs`
    - Remove `save_to_json` function from `serialization.rs`
    - Remove `LegacyJson` variant from local enum (already replaced by AssetManager)
    - Update `save_project_with_dialog` to remove `.json` file filter
    - Update open file dialog to only show `.rpg` filter (remove `.json`)
    - When a `.json` file is selected in the open dialog, display an error message that the legacy format is no longer supported
    - _Requirements: 5.2, 5.4, 5.6_

- [x] 9. Integrate AssetManager into the Launcher
  - [~] 9.1 Refactor `crates/rpg-toolkit-launcher/src/main.rs` to use `AssetManager`
    - Replace the local `ProjectSource` enum and `detect_project_source` function with `AssetManager::detect_source`
    - Replace `load_from_dir` and `load_from_zip` with calls to `AssetManager::load_project`
    - Remove `load_from_legacy_json` function entirely
    - _Requirements: 2.3, 3.2, 3.3, 3.5_

  - [~] 9.2 Remove legacy JSON support from the Launcher
    - Remove `LegacyJson` variant handling from `main()`
    - When a `.json` path is provided, print error to stderr: "Error: legacy JSON format is no longer supported. Please convert your project to directory or .rpg ZIP format."
    - Exit with non-zero exit code (1)
    - _Requirements: 5.3, 5.5_

- [~] 10. Final checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

## Notes

- Tasks marked with `*` are optional and can be skipped for faster MVP
- Each task references specific requirements for traceability
- Checkpoints ensure incremental validation
- Property tests validate universal correctness properties from the design document (12 total)
- Property tests go in `crates/rpg-toolkit-common/tests/properties/` following the existing pattern (one file per feature group)
- The project uses `proptest` with 100 cases per property (existing convention)
- The `AssetManager` struct lives in `rpg-toolkit-common` since both editor and launcher already depend on it
- Legacy JSON removal in tasks 8.2 and 9.2 should be done after integration to avoid breaking builds mid-implementation

## Task Dependency Graph

```json
{
  "waves": [
    { "id": 0, "tasks": ["1.1"] },
    { "id": 1, "tasks": ["1.2", "1.3"] },
    { "id": 2, "tasks": ["2.1"] },
    { "id": 3, "tasks": ["2.2", "2.3", "2.4", "2.5"] },
    { "id": 4, "tasks": ["4.1", "4.2"] },
    { "id": 5, "tasks": ["4.3", "4.4", "4.5"] },
    { "id": 6, "tasks": ["5.1"] },
    { "id": 7, "tasks": ["5.2", "5.3", "5.4"] },
    { "id": 8, "tasks": ["5.5", "5.6"] },
    { "id": 9, "tasks": ["7.1", "7.2", "7.3"] },
    { "id": 10, "tasks": ["8.1", "9.1"] },
    { "id": 11, "tasks": ["8.2", "9.2"] }
  ]
}
```
