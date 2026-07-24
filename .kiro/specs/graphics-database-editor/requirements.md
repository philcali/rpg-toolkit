# Requirements Document

## Introduction

The RPG Toolkit editor already supports assigning visual asset file paths to characters (spritesheet, face portrait, status portrait) and enemies (portrait) via file picker dialogs. However, items and abilities have no icon/graphic fields, there is no image preview when a graphic is assigned, and file-loading utilities are not exposed by the existing AssetManager. This feature extends the AssetManager with file-existence checking and byte-loading methods, introduces a shared `EntityGraphics` struct for attaching per-entity graphics to items and abilities, and enhances all entity editors with inline thumbnail previews of assigned graphics. Category-level graphics (e.g., a default icon for all "Weapon" items) are deferred to a future category editor feature.

## Glossary

- **Asset_Manager**: The existing unified module in rpg-toolkit-common responsible for registering, resolving, saving, and loading all image-referenced assets within a project. Extended in this feature with file-loading capabilities.
- **Entity_Graphics**: A shared struct containing optional graphic file path fields for game entities (items, abilities). Designed for extensibility when category-level graphics are added in a future feature.
- **Item_Editor**: The editor panel responsible for creating and managing items in the items database.
- **Item_Registry**: The data structure (`ItemRegistry`) that stores all defined items keyed by their unique ID.
- **Ability_Editor**: The editor panel responsible for creating and managing abilities in the abilities database.
- **Ability_Registry**: The data structure (`AbilityRegistry`) that stores all defined abilities keyed by their unique ID.
- **Enemy_Editor**: The editor panel responsible for creating and managing enemy definitions.
- **Character_Editor**: The editor panel responsible for creating and managing playable character definitions.
- **Thumbnail_Preview**: A small rendered preview of a graphic image file displayed inline in an entity editor when a graphic is assigned.
- **File_Picker**: A native OS file dialog (powered by the `rfd` crate) that allows the user to browse and select image files from the filesystem.

## Requirements

### Requirement 1: Asset Manager File Loading Extensions

**User Story:** As a toolkit developer, I want the AssetManager to provide file-existence checking and byte-loading methods, so that both the editor (for previews) and the renderer/launcher (for gameplay) can load image files through a single unified interface without duplicating resolution and I/O logic.

#### Acceptance Criteria

1. THE Asset_Manager SHALL provide a method to check whether a resolved absolute path points to an existing regular file on the filesystem, returning a boolean result.
2. THE Asset_Manager SHALL provide a method to load raw bytes from a resolved absolute path, returning the bytes on success or an error if the file cannot be read or does not exist.
3. IF the resolved absolute path points to a directory rather than a regular file, THEN THE Asset_Manager load-bytes method SHALL return an error indicating the path does not reference a readable file.
4. THE Asset_Manager SHALL provide a convenience method that accepts a relative file path and a project root, trims whitespace, resolves the path, validates the file exists, and returns the raw bytes in a single call.
5. IF the relative file path passed to the convenience method is empty or contains only whitespace after trimming, THEN THE Asset_Manager SHALL return an error indicating the path is invalid.
6. IF the relative file path passed to the convenience method contains path traversal components (`.` or `..`) in any segment, THEN THE Asset_Manager SHALL return an error indicating the path is invalid.
7. THE new file-loading methods SHALL be defined on the existing Asset_Manager in rpg-toolkit-common so that they are usable from the editor, renderer, and launcher crates without circular dependencies.

### Requirement 2: EntityGraphics Struct

**User Story:** As a toolkit developer, I want a shared struct for attaching graphics to game entities (items and abilities), so that all entities use a consistent pattern for graphic storage and the struct can be extended when category-level graphics are added in the future.

#### Acceptance Criteria

1. THE system SHALL provide an `EntityGraphics` struct containing an optional icon file path field (maximum 260 characters after trimming), defaulting to None.
2. THE Entity_Graphics struct SHALL be defined in rpg-toolkit-common and usable by Item_Registry, Ability_Registry, and their corresponding editors.
3. THE Entity_Graphics struct SHALL implement Serialize, Deserialize, Clone, Debug, Default, PartialEq, and Eq traits.
4. THE Entity_Graphics struct SHALL use `#[serde(default)]` on all optional fields for backward compatibility with existing data files that do not contain graphics fields.
5. THE Entity_Graphics struct SHALL persist through serialization and deserialization without data loss (None remains None, a set path remains the same string byte-for-byte).

### Requirement 3: Item Graphics Field

**User Story:** As a game designer, I want to assign an icon graphic to each item, so that items can display a visual representation in the shop UI, inventory, and other game interfaces.

#### Acceptance Criteria

1. THE Item_Registry SHALL store an Entity_Graphics struct for each item, defaulting to a default (empty) EntityGraphics for newly created items.
2. THE Item_Editor SHALL display an "Icon" section in the item detail view containing a single-line text input field for the icon path and a "Browse..." button for native File_Picker access.
3. WHILE no icon path is set, THE Item_Editor SHALL display a placeholder label reading "No icon assigned" in the "Icon" section.
4. WHEN the user activates the "Browse..." button, THE Item_Editor SHALL open a native file dialog filtered to image file types (png, jpg, jpeg).
5. WHEN the user selects a file in the native file dialog, THE Item_Editor SHALL populate the icon text input with the selected file path relative to the project root, truncated to 260 characters, and commit the value to the item model.
6. WHEN the user cancels the native file dialog, THE Item_Editor SHALL leave the icon text input unchanged.
7. WHEN the user enters an icon path manually and commits (on lost focus), THE Item_Editor SHALL trim whitespace and store the resulting value, treating empty-after-trim as None, and truncating to 260 characters if the trimmed input exceeds the limit.
8. THE Item_Editor SHALL allow the user to clear the icon path by activating a "Clear" control, resetting the field to None.
9. IF the user modifies the item icon field, THEN THE Item_Editor SHALL mark the project as having unsaved item changes.
10. THE Item_Registry SHALL persist the Entity_Graphics struct through serialization and deserialization without data loss, using `#[serde(default)]` for backward compatibility with existing item files.

### Requirement 4: Ability Graphics Field

**User Story:** As a game designer, I want to assign an icon graphic to each ability, so that abilities can display a visual representation in battle menus, skill lists, and other game interfaces.

#### Acceptance Criteria

1. THE Ability_Registry SHALL store an Entity_Graphics struct for each ability, defaulting to a default (empty) EntityGraphics for newly created abilities.
2. THE Ability_Editor SHALL display an "Icon" section in the ability detail view containing a single-line text input field for the icon path and a "Browse..." button for native File_Picker access.
3. WHILE no icon path is set, THE Ability_Editor SHALL display a placeholder label reading "No icon assigned" in the "Icon" section.
4. WHEN the user activates the "Browse..." button, THE Ability_Editor SHALL open a native file dialog filtered to image file types (png, jpg, jpeg).
5. WHEN the user selects a file in the native file dialog, THE Ability_Editor SHALL populate the icon text input with the selected file path relative to the project root, truncated to 260 characters, and commit the value to the ability model.
6. WHEN the user cancels the native file dialog, THE Ability_Editor SHALL leave the icon text input unchanged.
7. WHEN the user enters an icon path manually and commits (on lost focus), THE Ability_Editor SHALL trim whitespace and store the resulting value, treating empty-after-trim as None, and truncating to 260 characters if the trimmed input exceeds the limit.
8. THE Ability_Editor SHALL allow the user to clear the icon path by activating a "Clear" control, resetting the field to None.
9. IF the user modifies the ability icon field, THEN THE Ability_Editor SHALL mark the project as having unsaved ability changes.
10. THE Ability_Registry SHALL persist the Entity_Graphics struct through serialization and deserialization without data loss, using `#[serde(default)]` for backward compatibility with existing ability files.

### Requirement 5: Thumbnail Preview in Item Editor

**User Story:** As a game designer, I want to see a small preview of the assigned item icon in the editor, so that I can visually confirm the correct graphic is assigned without opening an external viewer.

#### Acceptance Criteria

1. WHEN an item has an icon path assigned that points to an existing image file (png, jpg, jpeg), THE Item_Editor SHALL resolve the path through the Asset_Manager and render a Thumbnail_Preview of the image at a maximum size of 64x64 pixels in the "Icon" section.
2. IF the icon path is assigned but the resolved file does not exist or cannot be loaded as a valid image, THEN THE Item_Editor SHALL display a placeholder label indicating "Image not found" in place of the Thumbnail_Preview.
3. WHEN the icon path changes (via file picker, manual entry, or clear), THE Item_Editor SHALL update the Thumbnail_Preview within the same frame to show one of: the new image thumbnail (if valid), the "Image not found" placeholder (if path is set but invalid), or the "No icon assigned" label (if path is None).
4. THE Thumbnail_Preview SHALL preserve the aspect ratio of the source image, scaling down to fit within the 64x64 pixel bounding box without cropping.
5. WHEN the icon path is None (no icon assigned), THE Item_Editor SHALL display the "No icon assigned" label in the "Icon" section instead of a Thumbnail_Preview or error placeholder.

### Requirement 6: Thumbnail Preview in Ability Editor

**User Story:** As a game designer, I want to see a small preview of the assigned ability icon in the editor, so that I can visually confirm the correct graphic is assigned.

#### Acceptance Criteria

1. WHEN an ability has an icon path assigned that points to an existing image file (png, jpg, jpeg), THE Ability_Editor SHALL resolve the path through the Asset_Manager and render a Thumbnail_Preview of the image at a maximum size of 64x64 pixels in the "Icon" section.
2. IF the icon path is assigned but the resolved file does not exist or cannot be loaded as a valid image, THEN THE Ability_Editor SHALL display a placeholder label indicating "Image not found" in place of the Thumbnail_Preview.
3. WHEN the icon path changes (via file picker, manual entry, or clear), THE Ability_Editor SHALL update the Thumbnail_Preview to reflect the new state.
4. THE Thumbnail_Preview SHALL preserve the aspect ratio of the source image, scaling down to fit within the 64x64 pixel bounding box without cropping or upscaling (images smaller than 64x64 are displayed at their original size).
5. WHEN no icon path is assigned (None), THE Ability_Editor SHALL display the "No icon assigned" label without rendering a Thumbnail_Preview or error placeholder.

### Requirement 7: Thumbnail Preview in Enemy Editor

**User Story:** As a game designer, I want to see a small preview of the assigned enemy portrait in the editor, so that I can visually confirm the correct graphic is assigned.

#### Acceptance Criteria

1. WHEN an enemy has a portrait path assigned that points to an existing image file (png, jpg, jpeg), THE Enemy_Editor SHALL render a Thumbnail_Preview of the image at a maximum size of 64x64 pixels in the "Portrait" section.
2. IF the file referenced by the portrait path does not exist or cannot be loaded as an image, THEN THE Enemy_Editor SHALL display a placeholder label indicating "Image not found" in place of the Thumbnail_Preview.
3. WHEN the portrait path changes (via file picker, manual entry, or clear), THE Enemy_Editor SHALL update the Thumbnail_Preview to reflect the new state.
4. THE Thumbnail_Preview SHALL preserve the aspect ratio of the source image, scaling down to fit within the 64x64 pixel bounding box without cropping or upscaling (images smaller than 64x64 are displayed at their original size).
5. WHEN no portrait path is assigned (None), THE Enemy_Editor SHALL display the existing "No portrait assigned" label without rendering a Thumbnail_Preview or error placeholder.

### Requirement 8: Thumbnail Preview in Character Editor

**User Story:** As a game designer, I want to see small previews of the assigned character graphics (spritesheet, face portrait, status portrait) in the editor, so that I can visually confirm the correct graphics are assigned.

#### Acceptance Criteria

1. WHEN a character has a visual asset path assigned (spritesheet, face portrait, or status portrait) that points to an existing image file (png, jpg, jpeg), THE Character_Editor SHALL render a Thumbnail_Preview of the image at a maximum size of 64x64 pixels adjacent to the corresponding text input field.
2. IF the file referenced by a visual asset path does not exist or cannot be loaded as an image, THEN THE Character_Editor SHALL display a placeholder label indicating "Image not found" in place of the Thumbnail_Preview for that asset slot.
3. WHEN a visual asset path changes (via file picker, manual entry, or clear), THE Character_Editor SHALL update the corresponding Thumbnail_Preview to display the new image if valid, display the "Image not found" placeholder if the new path is invalid, or hide the Thumbnail_Preview if the path is cleared to None.
4. THE Thumbnail_Preview SHALL preserve the aspect ratio of the source image, scaling down to fit within the 64x64 pixel bounding box without cropping.
5. THE Character_Editor SHALL display Thumbnail_Previews for all three visual asset types (spritesheet, face portrait, status portrait) independently, such that changing one asset path does not affect the Thumbnail_Preview of the other two asset slots.
6. WHEN a visual asset path is None (no asset assigned), THE Character_Editor SHALL not render a Thumbnail_Preview or an error placeholder for that asset slot.

### Requirement 9: Thumbnail Rendering Utility

**User Story:** As a toolkit developer, I want a shared thumbnail rendering utility, so that all entity editors render image previews consistently without duplicating image loading and scaling logic.

#### Acceptance Criteria

1. THE system SHALL provide a reusable thumbnail rendering function that accepts an image file path, a maximum bounding box dimension in pixels (as a single u32 value applied to both width and height), and an egui UI context, and renders the image preview or a placeholder label if the image cannot be loaded or decoded.
2. THE thumbnail rendering function SHALL load the image from the filesystem using the Asset_Manager file-loading methods, decode it as a supported image format (png, jpg, jpeg), and scale it to fit within the bounding box while preserving aspect ratio.
3. THE thumbnail rendering function SHALL cache loaded textures keyed by file path, storing at most 128 entries and evicting the least-recently-used entry when the limit is reached, to avoid reloading the same image from disk on every frame.
4. WHEN the file path passed to the thumbnail rendering function for a given UI location differs from the file path of the previously cached texture for that location, THE thumbnail rendering function SHALL discard the previously cached texture and load the image at the new path.
5. IF the image file cannot be loaded or decoded, THEN THE thumbnail rendering function SHALL render a placeholder label indicating "Image not found" sized to the bounding box dimensions.
6. THE thumbnail rendering function SHALL be defined as a shared module within the editor crate and be callable from the Item_Editor, Ability_Editor, Enemy_Editor, and Character_Editor without duplicating image loading, decoding, or scaling logic.

### Requirement 10: Asset Manager Resolution Round-Trip Property

**User Story:** As a toolkit developer, I want confidence that path resolution is consistent and correct, so that assets referenced by entities are reliably loadable across editing and runtime sessions.

#### Acceptance Criteria

1. FOR ALL valid relative file paths (non-empty, no path traversal components, no single-dot components, within 260 characters, using forward-slash separators), resolving through the Asset_Manager and then stripping the project root prefix from the resolved absolute path SHALL reproduce the original relative path exactly.
2. FOR ALL valid EntityGraphics icon paths persisted on items and abilities (non-empty strings of at most 260 characters after trimming, with no path traversal), serializing the respective registry to JSON and then deserializing it back SHALL produce identical icon path values for every entity (None remains None, a set path remains the same string byte-for-byte).
3. THE Asset_Manager resolve operation SHALL be idempotent: resolving the same relative path against the same project root 2 or more consecutive times SHALL produce identical absolute paths on each invocation.
4. IF the project root path used for resolution differs between invocations (e.g., different absolute roots), THEN THE Asset_Manager SHALL produce different absolute paths, but the relative suffix extracted by stripping the respective root prefix SHALL remain equal to the original relative path in both cases.
