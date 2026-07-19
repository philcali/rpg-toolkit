# Requirements Document

## Introduction

The RPG Toolkit project currently duplicates asset-loading logic across the editor (`serialization.rs`) and launcher (`main.rs`). Both consumers independently detect project source format, resolve relative asset paths to absolute paths, and load image-backed assets (tilesets, spritesheets, face portraits). This feature introduces a unified asset management layer in `rpg-toolkit-common` that provides a single abstraction for registering, resolving, saving, and loading image-referenced assets from the supported project formats (directory and `.rpg` ZIP archive). It also removes the legacy flat JSON format that is no longer the canonical storage mechanism.

## Glossary

- **Asset_Manager**: The unified module in `rpg-toolkit-common` responsible for registering, resolving, saving, and loading all image-referenced assets within a project.
- **Asset_Reference**: A record associating a logical asset identifier with a relative path inside the project archive and an asset category.
- **Asset_Category**: A classification tag describing the type of image asset (e.g., tileset, spritesheet, face_portrait). The set of categories is extensible.
- **Project_Source**: The storage format from which a project is loaded — either a filesystem directory or a `.rpg` ZIP archive.
- **Path_Resolution**: The process of converting a relative asset path stored in the manifest into an absolute filesystem path usable by the Bevy asset server at runtime.
- **Project_Manifest**: The `manifest.json` file at the root of a project directory or ZIP archive that declares all project metadata including asset references.
- **Editor**: The `rpg-toolkit-editor` crate — a Bevy-based application for authoring RPG projects.
- **Launcher**: The `rpg-toolkit-launcher` crate — a Bevy-based application for playing RPG projects.
- **Renderer**: The `rpg-toolkit-renderer` crate — a Bevy plugin for rendering project assets at runtime.

## Requirements

### Requirement 1: Unified Asset Registry

**User Story:** As a toolkit developer, I want a single registry of all image-referenced assets in a project, so that new asset categories can be added without duplicating resolution logic across consumers.

#### Acceptance Criteria

1. THE Asset_Manager SHALL maintain a registry of Asset_Reference entries keyed by a unique string identifier of 1 to 128 characters.
2. WHEN an Asset_Reference is registered with a unique identifier and a valid Asset_Category, THE Asset_Manager SHALL store the entry and make it retrievable by that identifier.
3. IF an Asset_Reference is registered with an identifier that already exists in the registry, THEN THE Asset_Manager SHALL reject the registration and return an error indicating a duplicate identifier.
4. THE Asset_Manager SHALL support at minimum the following Asset_Category values: tileset, spritesheet, and face_portrait.
5. THE Asset_Manager SHALL treat Asset_Category as an open set represented by a string value, so that future categories (e.g., item_portrait, background, backdrop) can be registered without modifying the Asset_Manager implementation.
6. WHEN a consumer requests an Asset_Reference by identifier, THE Asset_Manager SHALL return the associated entry including its Asset_Category and file path, or return an error indicating the identifier was not found.
7. WHEN a new Asset_Category value is introduced, THE Asset_Manager SHALL accept Asset_Reference entries for that category using the same registration and retrieval operations as existing categories, requiring no code changes to the Asset_Manager module.

### Requirement 2: Unified Path Resolution

**User Story:** As a toolkit developer, I want asset path resolution centralized in one module, so that the editor and launcher produce identical resolved paths for the same project.

#### Acceptance Criteria

1. WHEN a project is loaded from a directory-based Project_Source, THE Asset_Manager SHALL resolve each Asset_Reference relative path to an absolute path by joining the project directory with the relative path and canonicalizing the result.
2. WHEN a project is loaded from a ZIP-based Project_Source, THE Asset_Manager SHALL resolve each Asset_Reference relative path to an absolute path by joining the temporary extraction directory with the relative path and canonicalizing the result.
3. THE Asset_Manager SHALL return byte-for-byte identical resolved path strings for a given Asset_Reference and Project_Source regardless of whether the caller is the Editor or the Launcher.
4. IF an Asset_Reference contains an empty or missing relative path, THEN THE Asset_Manager SHALL skip resolution for that entry and return a warning that includes the asset identifier and Asset_Category.
5. IF an Asset_Reference relative path contains path traversal components that would resolve to a location outside the project root directory, THEN THE Asset_Manager SHALL reject that entry and return an error that includes the asset identifier and the offending relative path.

### Requirement 3: Unified Project Loading

**User Story:** As a toolkit developer, I want a single entry point for loading a project and its assets, so that both the editor and launcher share one implementation.

#### Acceptance Criteria

1. WHEN a project path is provided, THE Asset_Manager SHALL determine the Project_Source type by checking whether the path is a filesystem directory or a file with a `.rpg` extension.
2. WHEN the Project_Source is a directory, THE Asset_Manager SHALL load the Project_Manifest from `manifest.json` in the directory root and resolve all asset file references (maps, tilesets, spritesheets) relative to that directory.
3. WHEN the Project_Source is a ZIP archive, THE Asset_Manager SHALL extract the archive to a temporary directory and resolve all asset file references (maps, tilesets, spritesheets) relative to the extraction path.
4. IF the provided path does not exist, is neither a directory nor a `.rpg` file, THEN THE Asset_Manager SHALL return an error indicating the unsupported format and the path that was provided.
5. THE Asset_Manager SHALL return a fully-loaded ProjectFile structure containing all parsed map data, tileset metadata, spritesheet metadata, and registry data, requiring no further file I/O by the Editor or the Launcher.
6. IF the Project_Manifest file is missing or contains invalid JSON within the Project_Source, THEN THE Asset_Manager SHALL return an error indicating the parse failure and the file location that could not be read.
7. IF any asset file referenced in the Project_Manifest does not exist at the resolved path, THEN THE Asset_Manager SHALL return an error identifying each missing asset reference.

### Requirement 4: Unified Project Saving

**User Story:** As a toolkit developer, I want a single entry point for saving a project and its assets, so that asset packaging logic is not duplicated in the editor.

#### Acceptance Criteria

1. WHEN saving to a directory-based Project_Source, THE Asset_Manager SHALL write each registered asset file to the subdirectory determined by a configurable mapping from Asset_Category to directory name (e.g., tileset → `tilesets/`, spritesheet → `data/`, face_portrait → `data/`, and any future category to its configured subdirectory) and store the corresponding relative paths in the Project_Manifest.
2. WHEN saving to a ZIP-based Project_Source, THE Asset_Manager SHALL create or overwrite the target archive and package the Project_Manifest, all map data files, and each registered asset file into the archive using relative paths matching the directory layout.
3. THE Asset_Manager SHALL normalize all asset paths to use forward slashes as separators and relative directory prefixes with no leading slash (e.g., `tilesets/base.png`, `data/hero.png`).
4. IF an asset file referenced during save does not exist on the filesystem, THEN THE Asset_Manager SHALL exclude that asset from the written output, report a warning containing the missing asset's identifier and expected path, and continue saving remaining assets.
5. WHEN saving to a directory-based Project_Source, THE Asset_Manager SHALL also write the Project_Manifest and all map data files to the project directory alongside the asset files.

### Requirement 5: Remove Legacy JSON Format Support

**User Story:** As a toolkit developer, I want to remove the legacy flat JSON project format, so that there is one canonical storage format and no dead code paths.

#### Acceptance Criteria

1. THE Asset_Manager SHALL support only directory-based and ZIP-based Project_Source formats for loading projects.
2. THE Editor SHALL NOT load or save projects using the legacy flat JSON format.
3. THE Launcher SHALL NOT load projects using the legacy flat JSON format.
4. WHEN a user attempts to open a `.json` file as a project, THE Editor SHALL display an error message indicating that the legacy JSON format is no longer supported and that the project must be converted to directory or ZIP format.
5. WHEN a `.json` path is provided to the Launcher as the project argument, THE Launcher SHALL exit with a non-zero exit code and print an error message to stderr indicating that the legacy JSON format is no longer supported and that the project must be converted to directory or ZIP format.
6. THE Editor SHALL NOT offer the legacy JSON format as an option in file-save or file-open dialogs.

### Requirement 6: Asset Validation on Load

**User Story:** As a toolkit developer, I want the asset manager to validate that referenced asset files exist when loading a project, so that missing assets are detected early with clear messages.

#### Acceptance Criteria

1. WHEN a project is loaded, THE Asset_Manager SHALL verify that each Asset_Reference points to an existing file at the resolved absolute path on the filesystem.
2. IF one or more Asset_Reference entries point to non-existent files, THEN THE Asset_Manager SHALL collect all missing references and return them as a list of validation errors without aborting the load of remaining valid assets.
3. THE Asset_Manager SHALL include the asset identifier, Asset_Category, and resolved absolute path in each validation error entry.
4. IF all Asset_Reference entries point to existing files, THEN THE Asset_Manager SHALL complete loading with an empty validation error list.

### Requirement 7: Asset Reference Round-Trip Integrity

**User Story:** As a toolkit developer, I want saving and then loading a project to produce an equivalent set of asset references, so that no data is lost during persistence cycles.

#### Acceptance Criteria

1. FOR ALL valid project states containing at least one tileset, spritesheet, or face portrait entry, serializing the project to the directory-based format (manifest.json and maps/) and then deserializing from that directory SHALL produce a ProjectFile whose tilesets, spritesheets, and face_portraits maps contain the same keys and values (identifier, file_path, and numeric metadata fields) as the original.
2. FOR ALL valid project states, serializing the project to a ZIP archive and then deserializing from that ZIP archive SHALL produce a ProjectFile whose tilesets, spritesheets, and face_portraits maps contain the same keys and values as the original.
3. FOR ALL asset files (tileset images, spritesheet images) written into a ZIP archive during save, extracting those files from the archive SHALL produce byte-identical content compared to the source files.
4. IF a referenced asset file does not exist on disk at save time, THEN THE system SHALL skip that file in the ZIP archive without failing the overall save operation, and SHALL log a warning identifying the missing file path and its referencing identifier.
