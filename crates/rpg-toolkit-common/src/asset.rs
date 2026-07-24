use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::CommonError;

/// Open set of asset categories represented as strings.
/// Well-known constants are provided for common types.
pub type AssetCategory = String;

pub const CATEGORY_TILESET: &str = "tileset";
pub const CATEGORY_SPRITESHEET: &str = "spritesheet";
pub const CATEGORY_FACE_PORTRAIT: &str = "face_portrait";

/// A record associating a logical asset identifier with a relative file path
/// and an asset category.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetReference {
    /// Unique identifier (1–128 characters).
    pub id: String,
    /// Relative path within the project (forward-slash separated, no leading slash).
    pub relative_path: String,
    /// Classification tag (open string set).
    pub category: AssetCategory,
}

/// Registry of all image-referenced assets in a project, keyed by unique identifier.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetRegistry {
    entries: HashMap<String, AssetReference>,
}

impl AssetRegistry {
    /// Registers an asset reference in the registry.
    ///
    /// Validates that the identifier is 1–128 characters and is not already registered.
    pub fn register(&mut self, entry: AssetReference) -> Result<(), CommonError> {
        if entry.id.is_empty() || entry.id.len() > 128 {
            return Err(CommonError::AssetRegistryError(format!(
                "Asset identifier must be 1–128 characters, got {}",
                entry.id.len()
            )));
        }

        if self.entries.contains_key(&entry.id) {
            return Err(CommonError::AssetRegistryError(format!(
                "Asset with identifier '{}' is already registered",
                entry.id
            )));
        }

        self.entries.insert(entry.id.clone(), entry);
        Ok(())
    }

    /// Retrieves an asset reference by identifier.
    ///
    /// Returns an error if the identifier is not found.
    pub fn get(&self, id: &str) -> Result<&AssetReference, CommonError> {
        self.entries.get(id).ok_or_else(|| {
            CommonError::AssetRegistryError(format!("Asset with identifier '{}' not found", id))
        })
    }

    /// Removes and returns an asset reference by identifier.
    ///
    /// Returns an error if the identifier is not found.
    pub fn remove(&mut self, id: &str) -> Result<AssetReference, CommonError> {
        self.entries.remove(id).ok_or_else(|| {
            CommonError::AssetRegistryError(format!("Asset with identifier '{}' not found", id))
        })
    }

    /// Returns an iterator over all registered entries.
    pub fn iter(&self) -> impl Iterator<Item = (&String, &AssetReference)> {
        self.entries.iter()
    }

    /// Returns the number of registered entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns true if the registry contains no entries.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Describes a single missing asset reference discovered during validation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssetValidationError {
    pub asset_id: String,
    pub category: AssetCategory,
    pub resolved_path: PathBuf,
}

/// Non-fatal warning emitted during save or resolution.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssetWarning {
    pub asset_id: String,
    pub category: AssetCategory,
    pub message: String,
}

/// Supported project storage formats.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProjectSource {
    Directory(PathBuf),
    Zip(PathBuf),
}

/// Unified entry point for loading, saving, and resolving project assets.
pub struct AssetManager {
    registry: AssetRegistry,
    /// Configurable mapping from AssetCategory to subdirectory name.
    category_dirs: HashMap<AssetCategory, String>,
}

impl AssetManager {
    /// Create a new AssetManager with default category→directory mappings.
    ///
    /// Default mappings:
    /// - `tileset` → `tilesets/`
    /// - `spritesheet` → `data/`
    /// - `face_portrait` → `data/`
    pub fn new() -> Self {
        let mut category_dirs = HashMap::new();
        category_dirs.insert(CATEGORY_TILESET.to_string(), "tilesets/".to_string());
        category_dirs.insert(CATEGORY_SPRITESHEET.to_string(), "data/".to_string());
        category_dirs.insert(CATEGORY_FACE_PORTRAIT.to_string(), "data/".to_string());

        Self {
            registry: AssetRegistry::default(),
            category_dirs,
        }
    }

    /// Configure a category→subdirectory mapping.
    pub fn set_category_dir(&mut self, category: &str, dir: &str) {
        self.category_dirs
            .insert(category.to_string(), dir.to_string());
    }

    /// Build an AssetRegistry from an existing ProjectFile (migration helper).
    ///
    /// Iterates tilesets, spritesheets, and face_portraits from the project
    /// and registers each as an AssetReference. Registration errors (e.g. from
    /// IDs exceeding 128 characters) are silently ignored since project data
    /// is assumed to be valid.
    pub fn registry_from_project_file(project: &crate::ProjectFile) -> AssetRegistry {
        let mut registry = AssetRegistry::default();

        // Register tilesets
        for (tileset_id, meta) in &project.tilesets {
            let _ = registry.register(AssetReference {
                id: tileset_id.clone(),
                relative_path: meta.file_path.clone(),
                category: CATEGORY_TILESET.to_string(),
            });
        }

        // Register spritesheets
        for (spritesheet_id, spritesheet) in &project.spritesheets {
            let _ = registry.register(AssetReference {
                id: spritesheet_id.clone(),
                relative_path: spritesheet.file_path.clone(),
                category: CATEGORY_SPRITESHEET.to_string(),
            });
        }

        // Register face portraits
        for (portrait_key, portrait_path) in &project.face_portraits {
            let _ = registry.register(AssetReference {
                id: portrait_key.clone(),
                relative_path: portrait_path.clone(),
                category: CATEGORY_FACE_PORTRAIT.to_string(),
            });
        }

        registry
    }

    /// Detect ProjectSource from a filesystem path.
    ///
    /// - If the path is a directory, returns `ProjectSource::Directory`.
    /// - If the path has a `.rpg` extension, returns `ProjectSource::Zip`.
    /// - Otherwise, returns an `UnsupportedFormat` error.
    pub fn detect_source(path: &Path) -> Result<ProjectSource, CommonError> {
        if path.is_dir() {
            return Ok(ProjectSource::Directory(path.to_path_buf()));
        }

        if path.extension().and_then(|ext| ext.to_str()) == Some("rpg") {
            return Ok(ProjectSource::Zip(path.to_path_buf()));
        }

        Err(CommonError::UnsupportedFormat(format!(
            "{}",
            path.display()
        )))
    }

    /// Resolve a single relative path against a project root.
    /// Validates no path traversal escapes the root.
    ///
    /// This performs logical path normalization (no filesystem access required).
    /// Returns an error if the relative path is empty or contains path traversal
    /// components (`..`) that would escape the root.
    pub fn resolve_path(root: &Path, relative: &str) -> Result<PathBuf, CommonError> {
        if relative.is_empty() {
            return Err(CommonError::AssetPathError(
                "relative path is empty".to_string(),
            ));
        }

        // Reject any `.` or `..` components per path normalization rules
        for component in relative.split('/') {
            if component == "." {
                return Err(CommonError::AssetPathError(format!(
                    "path contains '.' component: {}",
                    relative
                )));
            }
            if component == ".." {
                return Err(CommonError::AssetPathError(format!(
                    "path traversal escapes project root: {}",
                    relative
                )));
            }
        }

        // Logically join root with relative path
        Ok(root.join(relative))
    }

    /// Resolve all asset references in the registry against a root directory.
    /// Returns resolved paths and warnings for empty/invalid entries.
    pub fn resolve_all(&self, root: &Path) -> (HashMap<String, PathBuf>, Vec<AssetWarning>) {
        let mut resolved = HashMap::new();
        let mut warnings = Vec::new();

        for (id, reference) in self.registry.iter() {
            if reference.relative_path.is_empty() {
                warnings.push(AssetWarning {
                    asset_id: id.clone(),
                    category: reference.category.clone(),
                    message: format!(
                        "asset '{}' has an empty relative path, skipping resolution",
                        id
                    ),
                });
                continue;
            }

            match Self::resolve_path(root, &reference.relative_path) {
                Ok(path) => {
                    resolved.insert(id.clone(), path);
                }
                Err(e) => {
                    warnings.push(AssetWarning {
                        asset_id: id.clone(),
                        category: reference.category.clone(),
                        message: format!("failed to resolve path for asset '{}': {}", id, e),
                    });
                }
            }
        }

        (resolved, warnings)
    }

    /// Load a project from a path, detecting the source format automatically.
    ///
    /// This is a standalone function that creates a fresh registry for the loaded project.
    /// It detects the source format, loads the manifest, converts to a ProjectFile,
    /// builds the asset registry, and validates that referenced asset files exist.
    ///
    /// Validation errors (missing asset files) are non-aborting and returned alongside
    /// the loaded project.
    pub fn load_project(
        path: &Path,
    ) -> Result<(crate::ProjectFile, Vec<AssetValidationError>), CommonError> {
        let source = Self::detect_source(path)?;

        match source {
            ProjectSource::Directory(dir) => Self::load_from_directory(&dir),
            ProjectSource::Zip(zip_path) => Self::load_from_zip(&zip_path),
        }
    }

    /// Load project from a directory source.
    fn load_from_directory(
        dir: &Path,
    ) -> Result<(crate::ProjectFile, Vec<AssetValidationError>), CommonError> {
        let manifest = crate::manifest::ProjectManifest::load_from_dir(dir)?;
        let project = manifest.into_project_file(dir)?;
        let registry = Self::registry_from_project_file(&project);
        let validation_errors = Self::validate_registry_files(&registry, dir);
        Ok((project, validation_errors))
    }

    /// Load project from a ZIP source.
    fn load_from_zip(
        zip_path: &Path,
    ) -> Result<(crate::ProjectFile, Vec<AssetValidationError>), CommonError> {
        let zip_data = std::fs::read(zip_path).map_err(|e| {
            CommonError::ZipError(format!(
                "failed to read zip file {}: {}",
                zip_path.display(),
                e
            ))
        })?;

        let temp_dir = tempfile::tempdir().map_err(|e| {
            CommonError::ZipError(format!("failed to create temp directory: {}", e))
        })?;

        let mut archive = zip::ZipArchive::new(std::io::Cursor::new(&zip_data))
            .map_err(|e| CommonError::ZipError(format!("failed to open zip archive: {}", e)))?;

        archive
            .extract(temp_dir.path())
            .map_err(|e| CommonError::ZipError(format!("failed to extract zip archive: {}", e)))?;

        let manifest = crate::manifest::ProjectManifest::load_from_dir(temp_dir.path())?;
        let project = manifest.into_project_file(temp_dir.path())?;
        let registry = Self::registry_from_project_file(&project);
        let validation_errors = Self::validate_registry_files(&registry, temp_dir.path());
        Ok((project, validation_errors))
    }

    /// Validate that all asset files in the registry exist on the filesystem.
    /// Returns an `AssetValidationError` for each missing file.
    fn validate_registry_files(registry: &AssetRegistry, root: &Path) -> Vec<AssetValidationError> {
        let mut errors = Vec::new();

        for (_id, reference) in registry.iter() {
            if reference.relative_path.is_empty() {
                continue;
            }

            match Self::resolve_path(root, &reference.relative_path) {
                Ok(resolved) => {
                    if !resolved.exists() {
                        errors.push(AssetValidationError {
                            asset_id: reference.id.clone(),
                            category: reference.category.clone(),
                            resolved_path: resolved,
                        });
                    }
                }
                Err(_) => {
                    // Path resolution failed (e.g., traversal) — treat as missing
                    let resolved = root.join(&reference.relative_path);
                    errors.push(AssetValidationError {
                        asset_id: reference.id.clone(),
                        category: reference.category.clone(),
                        resolved_path: resolved,
                    });
                }
            }
        }

        errors
    }

    /// Validate that all resolved asset paths point to existing files.
    ///
    /// Iterates the internal registry, resolves each entry's relative path against the
    /// given root, and returns an `AssetValidationError` for every file that does not
    /// exist on the filesystem. Entries with empty relative paths are skipped.
    pub fn validate_assets(&self, root: &Path) -> Vec<AssetValidationError> {
        Self::validate_registry_files(&self.registry, root)
    }

    /// Save a project to the given target path.
    /// Returns warnings for assets that could not be written (e.g., missing source files).
    ///
    /// Detects the target format: if `target` is an existing directory or doesn't have a `.rpg`
    /// extension, saves as directory format. If `target` has a `.rpg` extension, saves as ZIP.
    pub fn save_project(
        &self,
        project: &crate::ProjectFile,
        target: &Path,
        source_dir: &Path,
    ) -> Result<Vec<AssetWarning>, CommonError> {
        // Determine format: if target exists, use detect_source; otherwise check extension.
        let source = if target.exists() {
            Self::detect_source(target)?
        } else if target.extension().and_then(|ext| ext.to_str()) == Some("rpg") {
            ProjectSource::Zip(target.to_path_buf())
        } else {
            ProjectSource::Directory(target.to_path_buf())
        };

        match source {
            ProjectSource::Directory(dir) => self.save_to_directory(project, &dir, source_dir),
            ProjectSource::Zip(zip_path) => self.save_to_zip(project, &zip_path, source_dir),
        }
    }

    /// Save project to directory format.
    fn save_to_directory(
        &self,
        project: &crate::ProjectFile,
        target: &Path,
        source_dir: &Path,
    ) -> Result<Vec<AssetWarning>, CommonError> {
        let mut warnings = Vec::new();

        // Create target directory if needed
        std::fs::create_dir_all(target).map_err(|e| {
            CommonError::ProjectParseError(format!(
                "could not create target directory {}: {}",
                target.display(),
                e
            ))
        })?;

        // Write manifest.json and maps/*.json
        project.serialize_to_dir(target)?;

        // Copy each asset file to category-mapped subdirectory
        for (_id, reference) in self.registry.iter() {
            let category_dir = self
                .category_dirs
                .get(&reference.category)
                .cloned()
                .unwrap_or_else(|| format!("{}/", reference.category));

            // Normalize relative path: forward slashes, no leading slash
            let normalized = Self::normalize_path(&reference.relative_path);

            // Resolve source file
            let source_file = source_dir.join(&normalized);

            if !source_file.exists() {
                warnings.push(AssetWarning {
                    asset_id: reference.id.clone(),
                    category: reference.category.clone(),
                    message: format!(
                        "source file not found during save: {}",
                        source_file.display()
                    ),
                });
                continue;
            }

            // Determine destination: target/<category_dir>/<filename>
            let file_name = Path::new(&normalized)
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            let dest_subdir = target.join(&category_dir);
            std::fs::create_dir_all(&dest_subdir).map_err(|e| {
                CommonError::ProjectParseError(format!(
                    "could not create asset directory {}: {}",
                    dest_subdir.display(),
                    e
                ))
            })?;
            let dest_file = dest_subdir.join(&file_name);

            std::fs::copy(&source_file, &dest_file).map_err(|e| {
                CommonError::ProjectParseError(format!(
                    "could not copy asset {} to {}: {}",
                    source_file.display(),
                    dest_file.display(),
                    e
                ))
            })?;
        }

        Ok(warnings)
    }

    /// Save project to ZIP format.
    fn save_to_zip(
        &self,
        project: &crate::ProjectFile,
        zip_path: &Path,
        source_dir: &Path,
    ) -> Result<Vec<AssetWarning>, CommonError> {
        use std::io::Write;

        let mut warnings = Vec::new();

        let file = std::fs::File::create(zip_path).map_err(|e| {
            CommonError::ZipError(format!(
                "could not create ZIP file {}: {}",
                zip_path.display(),
                e
            ))
        })?;

        let mut zip = zip::ZipWriter::new(file);
        let options =
            zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        // Write manifest.json
        let manifest = project.to_manifest();
        let manifest_json = serde_json::to_string_pretty(&manifest).map_err(|e| {
            CommonError::ProjectParseError(format!("could not serialize manifest: {}", e))
        })?;
        zip.start_file("manifest.json", options).map_err(|e| {
            CommonError::ZipError(format!("could not write manifest to zip: {}", e))
        })?;
        zip.write_all(manifest_json.as_bytes())
            .map_err(|e| CommonError::ZipError(format!("could not write manifest data: {}", e)))?;

        // Write each map as maps/<id>.json
        for (map_id, map) in &project.maps {
            let map_json = serde_json::to_string_pretty(map).map_err(|e| {
                CommonError::ProjectParseError(format!(
                    "could not serialize map '{}': {}",
                    map_id, e
                ))
            })?;
            let entry_name = format!("maps/{}.json", map_id);
            zip.start_file(&entry_name, options)
                .map_err(|e| CommonError::ZipError(format!("could not write map entry: {}", e)))?;
            zip.write_all(map_json.as_bytes())
                .map_err(|e| CommonError::ZipError(format!("could not write map data: {}", e)))?;
        }

        // Write each asset file at its normalized relative path
        for (_id, reference) in self.registry.iter() {
            let normalized = Self::normalize_path(&reference.relative_path);

            let source_file = source_dir.join(&normalized);

            if !source_file.exists() {
                warnings.push(AssetWarning {
                    asset_id: reference.id.clone(),
                    category: reference.category.clone(),
                    message: format!(
                        "source file not found during save: {}",
                        source_file.display()
                    ),
                });
                continue;
            }

            let data = std::fs::read(&source_file).map_err(|e| {
                CommonError::ZipError(format!(
                    "could not read asset file {}: {}",
                    source_file.display(),
                    e
                ))
            })?;

            zip.start_file(&normalized, options).map_err(|e| {
                CommonError::ZipError(format!(
                    "could not write asset entry '{}' to zip: {}",
                    normalized, e
                ))
            })?;
            zip.write_all(&data).map_err(|e| {
                CommonError::ZipError(format!("could not write asset data to zip: {}", e))
            })?;
        }

        zip.finish()
            .map_err(|e| CommonError::ZipError(format!("could not finish ZIP archive: {}", e)))?;

        Ok(warnings)
    }

    /// Normalize a path string: forward slashes, no leading slash, no `.` or `..` components.
    fn normalize_path(path: &str) -> String {
        let replaced = path.replace('\\', "/");
        let trimmed = replaced.trim_start_matches('/');
        trimmed.to_string()
    }

    /// Access the underlying registry.
    pub fn registry(&self) -> &AssetRegistry {
        &self.registry
    }

    /// Replace the internal registry with the given one.
    ///
    /// This is useful when building an `AssetManager` for saving: populate a registry
    /// from a `ProjectFile` and set it on the manager before calling `save_project`.
    pub fn set_registry(&mut self, registry: AssetRegistry) {
        self.registry = registry;
    }

    /// Access the category directory mappings.
    pub fn category_dirs(&self) -> &HashMap<AssetCategory, String> {
        &self.category_dirs
    }

    /// Checks whether a path points to an existing regular file.
    pub fn file_exists(path: &Path) -> bool {
        path.is_file()
    }

    /// Loads raw bytes from a resolved absolute path.
    ///
    /// Returns an error if:
    /// - The path does not exist
    /// - The path is a directory
    /// - The file cannot be read
    pub fn load_file_bytes(path: &Path) -> Result<Vec<u8>, CommonError> {
        if !path.exists() {
            return Err(CommonError::AssetPathError(format!(
                "file does not exist: {}",
                path.display()
            )));
        }
        if path.is_dir() {
            return Err(CommonError::AssetPathError(format!(
                "path is a directory, not a file: {}",
                path.display()
            )));
        }
        std::fs::read(path).map_err(|e| {
            CommonError::AssetPathError(format!("failed to read file {}: {}", path.display(), e))
        })
    }

    /// Convenience method: trims a relative path, resolves it against root,
    /// validates the target is a regular file, and loads its bytes.
    ///
    /// Combines trim → resolve_path → file validation → read in one call.
    pub fn resolve_and_load(root: &Path, relative_path: &str) -> Result<Vec<u8>, CommonError> {
        let trimmed = relative_path.trim();
        if trimmed.is_empty() {
            return Err(CommonError::AssetPathError(
                "file path is empty or whitespace-only".to_string(),
            ));
        }
        let resolved = Self::resolve_path(root, trimmed)?;
        Self::load_file_bytes(&resolved)
    }
}

impl Default for AssetManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_and_get() {
        let mut registry = AssetRegistry::default();
        let entry = AssetReference {
            id: "hero_tileset".to_string(),
            relative_path: "tilesets/hero.png".to_string(),
            category: CATEGORY_TILESET.to_string(),
        };
        registry.register(entry.clone()).unwrap();
        let retrieved = registry.get("hero_tileset").unwrap();
        assert_eq!(retrieved, &entry);
    }

    #[test]
    fn test_register_duplicate_rejected() {
        let mut registry = AssetRegistry::default();
        let entry = AssetReference {
            id: "hero".to_string(),
            relative_path: "tilesets/hero.png".to_string(),
            category: CATEGORY_TILESET.to_string(),
        };
        registry.register(entry.clone()).unwrap();
        let result = registry.register(entry);
        assert!(result.is_err());
        assert_eq!(registry.len(), 1);
    }

    #[test]
    fn test_register_empty_id_rejected() {
        let mut registry = AssetRegistry::default();
        let entry = AssetReference {
            id: "".to_string(),
            relative_path: "tilesets/hero.png".to_string(),
            category: CATEGORY_TILESET.to_string(),
        };
        let result = registry.register(entry);
        assert!(result.is_err());
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn test_register_id_too_long_rejected() {
        let mut registry = AssetRegistry::default();
        let entry = AssetReference {
            id: "a".repeat(129),
            relative_path: "tilesets/hero.png".to_string(),
            category: CATEGORY_TILESET.to_string(),
        };
        let result = registry.register(entry);
        assert!(result.is_err());
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn test_register_id_exactly_128_chars() {
        let mut registry = AssetRegistry::default();
        let entry = AssetReference {
            id: "a".repeat(128),
            relative_path: "tilesets/hero.png".to_string(),
            category: CATEGORY_TILESET.to_string(),
        };
        assert!(registry.register(entry).is_ok());
        assert_eq!(registry.len(), 1);
    }

    #[test]
    fn test_get_not_found() {
        let registry = AssetRegistry::default();
        let result = registry.get("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_remove_success() {
        let mut registry = AssetRegistry::default();
        let entry = AssetReference {
            id: "hero".to_string(),
            relative_path: "tilesets/hero.png".to_string(),
            category: CATEGORY_TILESET.to_string(),
        };
        registry.register(entry.clone()).unwrap();
        let removed = registry.remove("hero").unwrap();
        assert_eq!(removed, entry);
        assert!(registry.is_empty());
    }

    #[test]
    fn test_remove_not_found() {
        let mut registry = AssetRegistry::default();
        let result = registry.remove("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_iter_entries() {
        let mut registry = AssetRegistry::default();
        registry
            .register(AssetReference {
                id: "a".to_string(),
                relative_path: "tilesets/a.png".to_string(),
                category: CATEGORY_TILESET.to_string(),
            })
            .unwrap();
        registry
            .register(AssetReference {
                id: "b".to_string(),
                relative_path: "data/b.png".to_string(),
                category: CATEGORY_SPRITESHEET.to_string(),
            })
            .unwrap();
        let entries: Vec<_> = registry.iter().collect();
        assert_eq!(entries.len(), 2);
    }

    #[test]
    fn test_len_and_is_empty() {
        let mut registry = AssetRegistry::default();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);

        registry
            .register(AssetReference {
                id: "x".to_string(),
                relative_path: "data/x.png".to_string(),
                category: CATEGORY_FACE_PORTRAIT.to_string(),
            })
            .unwrap();
        assert!(!registry.is_empty());
        assert_eq!(registry.len(), 1);
    }

    #[test]
    fn test_open_category_accepted() {
        let mut registry = AssetRegistry::default();
        let entry = AssetReference {
            id: "custom_asset".to_string(),
            relative_path: "custom/asset.png".to_string(),
            category: "custom_category".to_string(),
        };
        registry.register(entry.clone()).unwrap();
        let retrieved = registry.get("custom_asset").unwrap();
        assert_eq!(retrieved.category, "custom_category");
    }

    #[test]
    fn test_project_source_variants() {
        let dir_source = ProjectSource::Directory(PathBuf::from("/tmp/project"));
        let zip_source = ProjectSource::Zip(PathBuf::from("/tmp/project.rpg"));
        assert_eq!(
            dir_source,
            ProjectSource::Directory(PathBuf::from("/tmp/project"))
        );
        assert_eq!(
            zip_source,
            ProjectSource::Zip(PathBuf::from("/tmp/project.rpg"))
        );
    }

    // --- AssetManager::resolve_path tests ---

    // --- AssetManager::detect_source tests ---

    #[test]
    fn test_detect_source_directory() {
        let dir = std::env::temp_dir().join("rpg_detect_source_test_dir");
        std::fs::create_dir_all(&dir).unwrap();
        let result = AssetManager::detect_source(&dir);
        assert_eq!(result.unwrap(), ProjectSource::Directory(dir.clone()));
        std::fs::remove_dir(&dir).ok();
    }

    #[test]
    fn test_detect_source_rpg_extension() {
        let path = Path::new("/tmp/my_project.rpg");
        let result = AssetManager::detect_source(path);
        assert_eq!(result.unwrap(), ProjectSource::Zip(path.to_path_buf()));
    }

    #[test]
    fn test_detect_source_unsupported_json() {
        let path = Path::new("/tmp/project.json");
        let result = AssetManager::detect_source(path);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CommonError::UnsupportedFormat(_)
        ));
    }

    #[test]
    fn test_detect_source_unsupported_txt() {
        let path = Path::new("/tmp/notes.txt");
        let result = AssetManager::detect_source(path);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CommonError::UnsupportedFormat(_)
        ));
    }

    #[test]
    fn test_detect_source_nonexistent_rpg_file() {
        // A non-existent path with .rpg extension should still return Zip
        // because detection is based on extension, not file existence.
        let path = Path::new("/nonexistent/path/game.rpg");
        let result = AssetManager::detect_source(path);
        assert_eq!(result.unwrap(), ProjectSource::Zip(path.to_path_buf()));
    }

    // --- AssetManager::resolve_path tests ---

    #[test]
    fn test_resolve_path_valid_relative() {
        let root = Path::new("/home/user/project");
        let result = AssetManager::resolve_path(root, "tilesets/hero.png");
        assert_eq!(
            result.unwrap(),
            PathBuf::from("/home/user/project/tilesets/hero.png")
        );
    }

    #[test]
    fn test_resolve_path_valid_subdirectories() {
        let root = Path::new("/home/user/project");
        let result = AssetManager::resolve_path(root, "data/sprites/characters/hero.png");
        assert_eq!(
            result.unwrap(),
            PathBuf::from("/home/user/project/data/sprites/characters/hero.png")
        );
    }

    #[test]
    fn test_resolve_path_empty_relative() {
        let root = Path::new("/home/user/project");
        let result = AssetManager::resolve_path(root, "");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, CommonError::AssetPathError(_)));
    }

    #[test]
    fn test_resolve_path_traversal_escape() {
        let root = Path::new("/home/user/project");
        let result = AssetManager::resolve_path(root, "../../etc/passwd");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, CommonError::AssetPathError(_)));
    }

    #[test]
    fn test_resolve_path_dot_component_rejected() {
        let root = Path::new("/home/user/project");
        let result = AssetManager::resolve_path(root, "tilesets/./hero.png");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, CommonError::AssetPathError(_)));
    }

    #[test]
    fn test_resolve_path_traversal_within_bounds() {
        // Even "a/../b" is rejected because `..` components are not allowed
        // per path normalization rules (rule 3: no `..` or `.` components).
        let root = Path::new("/home/user/project");
        let result = AssetManager::resolve_path(root, "a/../b");
        assert!(result.is_err());
    }

    #[test]
    fn test_resolve_path_simple_filename() {
        let root = Path::new("/project");
        let result = AssetManager::resolve_path(root, "readme.txt");
        assert_eq!(result.unwrap(), PathBuf::from("/project/readme.txt"));
    }

    // --- AssetManager::resolve_all tests ---

    #[test]
    fn test_resolve_all_mixed_entries() {
        let mut registry = AssetRegistry::default();
        registry
            .register(AssetReference {
                id: "valid_tile".to_string(),
                relative_path: "tilesets/base.png".to_string(),
                category: CATEGORY_TILESET.to_string(),
            })
            .unwrap();
        registry
            .register(AssetReference {
                id: "empty_path".to_string(),
                relative_path: "".to_string(),
                category: CATEGORY_SPRITESHEET.to_string(),
            })
            .unwrap();
        registry
            .register(AssetReference {
                id: "traversal".to_string(),
                relative_path: "../../etc/passwd".to_string(),
                category: CATEGORY_FACE_PORTRAIT.to_string(),
            })
            .unwrap();
        registry
            .register(AssetReference {
                id: "valid_sprite".to_string(),
                relative_path: "data/hero.png".to_string(),
                category: CATEGORY_SPRITESHEET.to_string(),
            })
            .unwrap();

        let manager = AssetManager {
            registry,
            category_dirs: HashMap::new(),
        };

        let root = Path::new("/home/user/project");
        let (resolved, warnings) = manager.resolve_all(root);

        // Two valid entries should resolve
        assert_eq!(resolved.len(), 2);
        assert_eq!(
            resolved.get("valid_tile").unwrap(),
            &PathBuf::from("/home/user/project/tilesets/base.png")
        );
        assert_eq!(
            resolved.get("valid_sprite").unwrap(),
            &PathBuf::from("/home/user/project/data/hero.png")
        );

        // Two warnings: one for empty path, one for traversal
        assert_eq!(warnings.len(), 2);
        let warning_ids: Vec<&str> = warnings.iter().map(|w| w.asset_id.as_str()).collect();
        assert!(warning_ids.contains(&"empty_path"));
        assert!(warning_ids.contains(&"traversal"));
    }

    #[test]
    fn test_resolve_all_all_valid() {
        let mut registry = AssetRegistry::default();
        registry
            .register(AssetReference {
                id: "tile1".to_string(),
                relative_path: "tilesets/tile1.png".to_string(),
                category: CATEGORY_TILESET.to_string(),
            })
            .unwrap();

        let manager = AssetManager {
            registry,
            category_dirs: HashMap::new(),
        };

        let root = Path::new("/project");
        let (resolved, warnings) = manager.resolve_all(root);

        assert_eq!(resolved.len(), 1);
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_resolve_all_empty_registry() {
        let manager = AssetManager {
            registry: AssetRegistry::default(),
            category_dirs: HashMap::new(),
        };

        let root = Path::new("/project");
        let (resolved, warnings) = manager.resolve_all(root);

        assert!(resolved.is_empty());
        assert!(warnings.is_empty());
    }

    // --- AssetManager::new tests ---

    #[test]
    fn test_new_default_category_dirs() {
        let manager = AssetManager::new();
        let dirs = manager.category_dirs();
        assert_eq!(dirs.get(CATEGORY_TILESET).unwrap(), "tilesets/");
        assert_eq!(dirs.get(CATEGORY_SPRITESHEET).unwrap(), "data/");
        assert_eq!(dirs.get(CATEGORY_FACE_PORTRAIT).unwrap(), "data/");
        assert_eq!(dirs.len(), 3);
    }

    #[test]
    fn test_new_empty_registry() {
        let manager = AssetManager::new();
        assert!(manager.registry().is_empty());
    }

    // --- AssetManager::load_project tests ---

    #[test]
    fn test_load_project_from_directory() {
        use crate::map::MapData;
        use crate::tileset::TilesetMeta;
        use std::collections::HashMap as StdHashMap;

        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();

        // Create a manifest with a tileset
        let mut tilesets = StdHashMap::new();
        tilesets.insert(
            "base".to_string(),
            TilesetMeta {
                file_path: "tilesets/base.png".to_string(),
                tile_width: 16,
                tile_height: 16,
                columns: 10,
                rows: 10,
                animations: vec![],
            },
        );

        let manifest = crate::manifest::ProjectManifest {
            maps: vec!["test_map".to_string()],
            tilesets,
            spawn_point: None,
            spritesheets: StdHashMap::new(),
            player_spritesheet: None,
            dialog_texts: StdHashMap::new(),
            face_portraits: StdHashMap::new(),
            characters: Default::default(),
            items: Default::default(),
            abilities: Default::default(),
            enemies: Default::default(),
            shops: Default::default(),
        };

        // Write manifest
        manifest.save_to_dir(root).unwrap();

        // Write map file
        let maps_dir = root.join("maps");
        std::fs::create_dir_all(&maps_dir).unwrap();
        let map = MapData::new("test_map", 8, 8, 16, 16).unwrap();
        let map_json = serde_json::to_string_pretty(&map).unwrap();
        std::fs::write(maps_dir.join("test_map.json"), &map_json).unwrap();

        // Create tileset file so validation passes
        let tilesets_dir = root.join("tilesets");
        std::fs::create_dir_all(&tilesets_dir).unwrap();
        std::fs::write(tilesets_dir.join("base.png"), b"fake png data").unwrap();

        let (project, errors) = AssetManager::load_project(root).unwrap();
        assert!(errors.is_empty());
        assert!(project.maps.contains_key("test_map"));
        assert!(project.tilesets.contains_key("base"));
    }

    #[test]
    fn test_load_project_from_directory_missing_asset() {
        use crate::map::MapData;
        use crate::tileset::TilesetMeta;
        use std::collections::HashMap as StdHashMap;

        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();

        let mut tilesets = StdHashMap::new();
        tilesets.insert(
            "missing_tileset".to_string(),
            TilesetMeta {
                file_path: "tilesets/missing.png".to_string(),
                tile_width: 16,
                tile_height: 16,
                columns: 10,
                rows: 10,
                animations: vec![],
            },
        );

        let manifest = crate::manifest::ProjectManifest {
            maps: vec!["map1".to_string()],
            tilesets,
            spawn_point: None,
            spritesheets: StdHashMap::new(),
            player_spritesheet: None,
            dialog_texts: StdHashMap::new(),
            face_portraits: StdHashMap::new(),
            characters: Default::default(),
            items: Default::default(),
            abilities: Default::default(),
            enemies: Default::default(),
            shops: Default::default(),
        };

        manifest.save_to_dir(root).unwrap();

        let maps_dir = root.join("maps");
        std::fs::create_dir_all(&maps_dir).unwrap();
        let map = MapData::new("map1", 4, 4, 16, 16).unwrap();
        let map_json = serde_json::to_string_pretty(&map).unwrap();
        std::fs::write(maps_dir.join("map1.json"), &map_json).unwrap();

        // Don't create the tileset file — it should appear in validation errors
        let (project, errors) = AssetManager::load_project(root).unwrap();
        assert!(project.tilesets.contains_key("missing_tileset"));
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].asset_id, "missing_tileset");
        assert_eq!(errors[0].category, CATEGORY_TILESET);
    }

    #[test]
    fn test_load_project_from_zip() {
        use crate::map::MapData;
        use crate::tileset::TilesetMeta;
        use std::collections::HashMap as StdHashMap;
        use std::io::Write;

        let tmp = tempfile::tempdir().unwrap();
        let zip_path = tmp.path().join("test_project.rpg");

        // Build a zip in memory
        let mut zip_buffer = std::io::Cursor::new(Vec::new());
        {
            let mut zip_writer = zip::ZipWriter::new(&mut zip_buffer);
            let options = zip::write::FileOptions::default();

            // Write manifest.json
            let mut tilesets = StdHashMap::new();
            tilesets.insert(
                "world".to_string(),
                TilesetMeta {
                    file_path: "tilesets/world.png".to_string(),
                    tile_width: 16,
                    tile_height: 16,
                    columns: 8,
                    rows: 8,
                    animations: vec![],
                },
            );

            let manifest = crate::manifest::ProjectManifest {
                maps: vec!["dungeon".to_string()],
                tilesets,
                spawn_point: None,
                spritesheets: StdHashMap::new(),
                player_spritesheet: None,
                dialog_texts: StdHashMap::new(),
                face_portraits: StdHashMap::new(),
                characters: Default::default(),
                items: Default::default(),
                abilities: Default::default(),
                enemies: Default::default(),
                shops: Default::default(),
            };

            let manifest_json = serde_json::to_string_pretty(&manifest).unwrap();
            zip_writer.start_file("manifest.json", options).unwrap();
            zip_writer.write_all(manifest_json.as_bytes()).unwrap();

            // Write map file
            let map = MapData::new("dungeon", 6, 6, 16, 16).unwrap();
            let map_json = serde_json::to_string_pretty(&map).unwrap();
            zip_writer.start_file("maps/dungeon.json", options).unwrap();
            zip_writer.write_all(map_json.as_bytes()).unwrap();

            // Write tileset file
            zip_writer
                .start_file("tilesets/world.png", options)
                .unwrap();
            zip_writer.write_all(b"fake tileset data").unwrap();

            zip_writer.finish().unwrap();
        }

        std::fs::write(&zip_path, zip_buffer.into_inner()).unwrap();

        let (project, errors) = AssetManager::load_project(&zip_path).unwrap();
        assert!(errors.is_empty());
        assert!(project.maps.contains_key("dungeon"));
        assert!(project.tilesets.contains_key("world"));
    }

    #[test]
    fn test_load_project_from_zip_missing_asset() {
        use crate::map::MapData;
        use crate::tileset::TilesetMeta;
        use std::collections::HashMap as StdHashMap;
        use std::io::Write;

        let tmp = tempfile::tempdir().unwrap();
        let zip_path = tmp.path().join("missing_asset.rpg");

        let mut zip_buffer = std::io::Cursor::new(Vec::new());
        {
            let mut zip_writer = zip::ZipWriter::new(&mut zip_buffer);
            let options = zip::write::FileOptions::default();

            let mut tilesets = StdHashMap::new();
            tilesets.insert(
                "absent".to_string(),
                TilesetMeta {
                    file_path: "tilesets/absent.png".to_string(),
                    tile_width: 32,
                    tile_height: 32,
                    columns: 4,
                    rows: 4,
                    animations: vec![],
                },
            );

            let manifest = crate::manifest::ProjectManifest {
                maps: vec!["level1".to_string()],
                tilesets,
                spawn_point: None,
                spritesheets: StdHashMap::new(),
                player_spritesheet: None,
                dialog_texts: StdHashMap::new(),
                face_portraits: StdHashMap::new(),
                characters: Default::default(),
                items: Default::default(),
                abilities: Default::default(),
                enemies: Default::default(),
                shops: Default::default(),
            };

            let manifest_json = serde_json::to_string_pretty(&manifest).unwrap();
            zip_writer.start_file("manifest.json", options).unwrap();
            zip_writer.write_all(manifest_json.as_bytes()).unwrap();

            let map = MapData::new("level1", 4, 4, 32, 32).unwrap();
            let map_json = serde_json::to_string_pretty(&map).unwrap();
            zip_writer.start_file("maps/level1.json", options).unwrap();
            zip_writer.write_all(map_json.as_bytes()).unwrap();

            // Intentionally NOT writing tilesets/absent.png
            zip_writer.finish().unwrap();
        }

        std::fs::write(&zip_path, zip_buffer.into_inner()).unwrap();

        let (project, errors) = AssetManager::load_project(&zip_path).unwrap();
        assert!(project.tilesets.contains_key("absent"));
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].asset_id, "absent");
        assert_eq!(errors[0].category, CATEGORY_TILESET);
    }

    #[test]
    fn test_load_project_unsupported_format() {
        let path = Path::new("/tmp/nonexistent_project.json");
        let result = AssetManager::load_project(path);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CommonError::UnsupportedFormat(_)
        ));
    }

    #[test]
    fn test_default_equals_new() {
        let from_new = AssetManager::new();
        let from_default = AssetManager::default();
        assert_eq!(from_new.category_dirs(), from_default.category_dirs());
        assert_eq!(from_new.registry().len(), from_default.registry().len());
    }

    // --- AssetManager::set_category_dir tests ---

    #[test]
    fn test_set_category_dir_overrides_existing() {
        let mut manager = AssetManager::new();
        manager.set_category_dir(CATEGORY_TILESET, "custom_tilesets/");
        assert_eq!(
            manager.category_dirs().get(CATEGORY_TILESET).unwrap(),
            "custom_tilesets/"
        );
    }

    #[test]
    fn test_set_category_dir_adds_new_category() {
        let mut manager = AssetManager::new();
        manager.set_category_dir("music", "audio/music/");
        assert_eq!(
            manager.category_dirs().get("music").unwrap(),
            "audio/music/"
        );
        // Existing entries unchanged
        assert_eq!(manager.category_dirs().len(), 4);
    }

    // --- AssetManager::registry_from_project_file tests ---

    #[test]
    fn test_registry_from_project_file_empty() {
        use crate::ProjectFile;
        use std::collections::HashMap as StdHashMap;

        let project = ProjectFile::new(
            StdHashMap::new(),
            StdHashMap::new(),
            None,
            StdHashMap::new(),
            None,
            StdHashMap::new(),
            StdHashMap::new(),
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
        );

        let registry = AssetManager::registry_from_project_file(&project);
        assert!(registry.is_empty());
    }

    #[test]
    fn test_registry_from_project_file_with_tilesets() {
        use crate::ProjectFile;
        use crate::tileset::TilesetMeta;
        use std::collections::HashMap as StdHashMap;

        let mut tilesets = StdHashMap::new();
        tilesets.insert(
            "overworld".to_string(),
            TilesetMeta {
                file_path: "tilesets/overworld.png".to_string(),
                tile_width: 16,
                tile_height: 16,
                columns: 10,
                rows: 10,
                animations: vec![],
            },
        );

        let project = ProjectFile::new(
            StdHashMap::new(),
            tilesets,
            None,
            StdHashMap::new(),
            None,
            StdHashMap::new(),
            StdHashMap::new(),
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
        );

        let registry = AssetManager::registry_from_project_file(&project);
        assert_eq!(registry.len(), 1);
        let entry = registry.get("overworld").unwrap();
        assert_eq!(entry.relative_path, "tilesets/overworld.png");
        assert_eq!(entry.category, CATEGORY_TILESET);
    }

    #[test]
    fn test_registry_from_project_file_with_all_types() {
        use crate::ProjectFile;
        use crate::spritesheet::CharacterSpritesheet;
        use crate::tileset::TilesetMeta;
        use std::collections::HashMap as StdHashMap;

        let mut tilesets = StdHashMap::new();
        tilesets.insert(
            "dungeon".to_string(),
            TilesetMeta {
                file_path: "tilesets/dungeon.png".to_string(),
                tile_width: 32,
                tile_height: 32,
                columns: 8,
                rows: 8,
                animations: vec![],
            },
        );

        let mut spritesheets = StdHashMap::new();
        spritesheets.insert(
            "hero_sprite".to_string(),
            CharacterSpritesheet {
                file_path: "data/hero.png".to_string(),
                sprite_width: 24,
                sprite_height: 32,
                frame_count: 3,
                direction_count: 4,
            },
        );

        let mut face_portraits = StdHashMap::new();
        face_portraits.insert("hero_face".to_string(), "data/hero_face.png".to_string());

        let project = ProjectFile::new(
            StdHashMap::new(),
            tilesets,
            None,
            spritesheets,
            None,
            StdHashMap::new(),
            face_portraits,
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
        );

        let registry = AssetManager::registry_from_project_file(&project);
        assert_eq!(registry.len(), 3);

        let tileset_entry = registry.get("dungeon").unwrap();
        assert_eq!(tileset_entry.relative_path, "tilesets/dungeon.png");
        assert_eq!(tileset_entry.category, CATEGORY_TILESET);

        let sprite_entry = registry.get("hero_sprite").unwrap();
        assert_eq!(sprite_entry.relative_path, "data/hero.png");
        assert_eq!(sprite_entry.category, CATEGORY_SPRITESHEET);

        let portrait_entry = registry.get("hero_face").unwrap();
        assert_eq!(portrait_entry.relative_path, "data/hero_face.png");
        assert_eq!(portrait_entry.category, CATEGORY_FACE_PORTRAIT);
    }

    // --- AssetManager::validate_assets tests ---

    #[test]
    fn test_validate_assets_all_files_exist() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();

        // Create asset files on disk
        let tilesets_dir = root.join("tilesets");
        std::fs::create_dir_all(&tilesets_dir).unwrap();
        std::fs::write(tilesets_dir.join("world.png"), b"png data").unwrap();

        let data_dir = root.join("data");
        std::fs::create_dir_all(&data_dir).unwrap();
        std::fs::write(data_dir.join("hero.png"), b"sprite data").unwrap();

        // Build manager with matching registry entries
        let mut registry = AssetRegistry::default();
        registry
            .register(AssetReference {
                id: "world_tile".to_string(),
                relative_path: "tilesets/world.png".to_string(),
                category: CATEGORY_TILESET.to_string(),
            })
            .unwrap();
        registry
            .register(AssetReference {
                id: "hero_sprite".to_string(),
                relative_path: "data/hero.png".to_string(),
                category: CATEGORY_SPRITESHEET.to_string(),
            })
            .unwrap();

        let manager = AssetManager {
            registry,
            category_dirs: HashMap::new(),
        };

        let errors = manager.validate_assets(root);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_validate_assets_returns_k_errors_for_k_missing() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();

        // Create only one of three asset files
        let tilesets_dir = root.join("tilesets");
        std::fs::create_dir_all(&tilesets_dir).unwrap();
        std::fs::write(tilesets_dir.join("existing.png"), b"data").unwrap();

        let mut registry = AssetRegistry::default();
        registry
            .register(AssetReference {
                id: "exists".to_string(),
                relative_path: "tilesets/existing.png".to_string(),
                category: CATEGORY_TILESET.to_string(),
            })
            .unwrap();
        registry
            .register(AssetReference {
                id: "missing1".to_string(),
                relative_path: "tilesets/gone1.png".to_string(),
                category: CATEGORY_TILESET.to_string(),
            })
            .unwrap();
        registry
            .register(AssetReference {
                id: "missing2".to_string(),
                relative_path: "data/gone2.png".to_string(),
                category: CATEGORY_SPRITESHEET.to_string(),
            })
            .unwrap();

        let manager = AssetManager {
            registry,
            category_dirs: HashMap::new(),
        };

        let errors = manager.validate_assets(root);
        // Exactly 2 missing files
        assert_eq!(errors.len(), 2);
        let error_ids: Vec<&str> = errors.iter().map(|e| e.asset_id.as_str()).collect();
        assert!(error_ids.contains(&"missing1"));
        assert!(error_ids.contains(&"missing2"));
    }

    #[test]
    fn test_validate_assets_includes_correct_fields() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();

        let mut registry = AssetRegistry::default();
        registry
            .register(AssetReference {
                id: "my_portrait".to_string(),
                relative_path: "data/face.png".to_string(),
                category: CATEGORY_FACE_PORTRAIT.to_string(),
            })
            .unwrap();

        let manager = AssetManager {
            registry,
            category_dirs: HashMap::new(),
        };

        let errors = manager.validate_assets(root);
        assert_eq!(errors.len(), 1);
        let err = &errors[0];
        assert_eq!(err.asset_id, "my_portrait");
        assert_eq!(err.category, CATEGORY_FACE_PORTRAIT);
        assert_eq!(err.resolved_path, root.join("data/face.png"));
    }

    #[test]
    fn test_validate_assets_skips_empty_relative_paths() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();

        let mut registry = AssetRegistry::default();
        // Entry with empty relative path — should be skipped
        registry
            .register(AssetReference {
                id: "empty_path_asset".to_string(),
                relative_path: "".to_string(),
                category: CATEGORY_TILESET.to_string(),
            })
            .unwrap();
        // Entry with a non-empty path that doesn't exist
        registry
            .register(AssetReference {
                id: "real_missing".to_string(),
                relative_path: "tilesets/nope.png".to_string(),
                category: CATEGORY_TILESET.to_string(),
            })
            .unwrap();

        let manager = AssetManager {
            registry,
            category_dirs: HashMap::new(),
        };

        let errors = manager.validate_assets(root);
        // Only "real_missing" should appear; "empty_path_asset" is skipped
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].asset_id, "real_missing");
    }

    // --- AssetManager::save_project tests ---

    #[test]
    fn test_save_project_to_directory_creates_structure() {
        use crate::map::MapData;
        use crate::spritesheet::CharacterSpritesheet;
        use crate::tileset::TilesetMeta;
        use std::collections::HashMap as StdHashMap;

        let source_tmp = tempfile::tempdir().unwrap();
        let source_dir = source_tmp.path();
        let target_tmp = tempfile::tempdir().unwrap();
        let target_dir = target_tmp.path().join("output_project");

        // Create source asset files
        let tilesets_dir = source_dir.join("tilesets");
        std::fs::create_dir_all(&tilesets_dir).unwrap();
        std::fs::write(tilesets_dir.join("world.png"), b"tileset image data").unwrap();

        let data_dir = source_dir.join("data");
        std::fs::create_dir_all(&data_dir).unwrap();
        std::fs::write(data_dir.join("hero.png"), b"spritesheet image data").unwrap();

        // Build project
        let mut tilesets = StdHashMap::new();
        tilesets.insert(
            "world".to_string(),
            TilesetMeta {
                file_path: "tilesets/world.png".to_string(),
                tile_width: 16,
                tile_height: 16,
                columns: 10,
                rows: 10,
                animations: vec![],
            },
        );

        let mut spritesheets = StdHashMap::new();
        spritesheets.insert(
            "hero".to_string(),
            CharacterSpritesheet {
                file_path: "data/hero.png".to_string(),
                sprite_width: 24,
                sprite_height: 32,
                frame_count: 3,
                direction_count: 4,
            },
        );

        let mut maps = StdHashMap::new();
        maps.insert(
            "level1".to_string(),
            MapData::new("level1", 4, 4, 16, 16).unwrap(),
        );

        let project = crate::ProjectFile::new(
            maps,
            tilesets,
            None,
            spritesheets,
            None,
            StdHashMap::new(),
            StdHashMap::new(),
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
        );

        // Build registry
        let mut registry = AssetRegistry::default();
        registry
            .register(AssetReference {
                id: "world".to_string(),
                relative_path: "tilesets/world.png".to_string(),
                category: CATEGORY_TILESET.to_string(),
            })
            .unwrap();
        registry
            .register(AssetReference {
                id: "hero".to_string(),
                relative_path: "data/hero.png".to_string(),
                category: CATEGORY_SPRITESHEET.to_string(),
            })
            .unwrap();

        let mut category_dirs = HashMap::new();
        category_dirs.insert(CATEGORY_TILESET.to_string(), "tilesets/".to_string());
        category_dirs.insert(CATEGORY_SPRITESHEET.to_string(), "data/".to_string());

        let manager = AssetManager {
            registry,
            category_dirs,
        };

        let warnings = manager
            .save_project(&project, &target_dir, source_dir)
            .unwrap();
        assert!(warnings.is_empty());

        // Verify structure
        assert!(target_dir.join("manifest.json").exists());
        assert!(target_dir.join("maps/level1.json").exists());
        assert!(target_dir.join("tilesets/world.png").exists());
        assert!(target_dir.join("data/hero.png").exists());

        // Verify file content
        let tileset_data = std::fs::read(target_dir.join("tilesets/world.png")).unwrap();
        assert_eq!(tileset_data, b"tileset image data");
        let sprite_data = std::fs::read(target_dir.join("data/hero.png")).unwrap();
        assert_eq!(sprite_data, b"spritesheet image data");
    }

    #[test]
    fn test_save_project_to_zip_creates_valid_archive() {
        use crate::map::MapData;
        use crate::tileset::TilesetMeta;
        use std::collections::HashMap as StdHashMap;
        use std::io::Read;

        let source_tmp = tempfile::tempdir().unwrap();
        let source_dir = source_tmp.path();
        let target_tmp = tempfile::tempdir().unwrap();
        let zip_path = target_tmp.path().join("project.rpg");

        // Create source asset file
        let tilesets_dir = source_dir.join("tilesets");
        std::fs::create_dir_all(&tilesets_dir).unwrap();
        std::fs::write(tilesets_dir.join("base.png"), b"base tileset bytes").unwrap();

        // Build project
        let mut tilesets = StdHashMap::new();
        tilesets.insert(
            "base".to_string(),
            TilesetMeta {
                file_path: "tilesets/base.png".to_string(),
                tile_width: 16,
                tile_height: 16,
                columns: 8,
                rows: 8,
                animations: vec![],
            },
        );

        let mut maps = StdHashMap::new();
        maps.insert(
            "town".to_string(),
            MapData::new("town", 8, 8, 16, 16).unwrap(),
        );

        let project = crate::ProjectFile::new(
            maps,
            tilesets,
            None,
            StdHashMap::new(),
            None,
            StdHashMap::new(),
            StdHashMap::new(),
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
        );

        // Build registry
        let mut registry = AssetRegistry::default();
        registry
            .register(AssetReference {
                id: "base".to_string(),
                relative_path: "tilesets/base.png".to_string(),
                category: CATEGORY_TILESET.to_string(),
            })
            .unwrap();

        let mut category_dirs = HashMap::new();
        category_dirs.insert(CATEGORY_TILESET.to_string(), "tilesets/".to_string());

        let manager = AssetManager {
            registry,
            category_dirs,
        };

        let warnings = manager
            .save_project(&project, &zip_path, source_dir)
            .unwrap();
        assert!(warnings.is_empty());

        // Verify ZIP contents
        assert!(zip_path.exists());
        let zip_data = std::fs::read(&zip_path).unwrap();
        let mut archive = zip::ZipArchive::new(std::io::Cursor::new(zip_data)).unwrap();

        // Check manifest exists
        let mut manifest_content = String::new();
        archive
            .by_name("manifest.json")
            .unwrap()
            .read_to_string(&mut manifest_content)
            .unwrap();
        assert!(manifest_content.contains("base"));

        // Check map exists
        let mut map_content = String::new();
        archive
            .by_name("maps/town.json")
            .unwrap()
            .read_to_string(&mut map_content)
            .unwrap();
        assert!(!map_content.is_empty());

        // Check asset file and content
        let mut asset_data = Vec::new();
        archive
            .by_name("tilesets/base.png")
            .unwrap()
            .read_to_end(&mut asset_data)
            .unwrap();
        assert_eq!(asset_data, b"base tileset bytes");
    }

    #[test]
    fn test_save_project_missing_source_emits_warning() {
        use crate::map::MapData;
        use crate::tileset::TilesetMeta;
        use std::collections::HashMap as StdHashMap;

        let source_tmp = tempfile::tempdir().unwrap();
        let source_dir = source_tmp.path();
        let target_tmp = tempfile::tempdir().unwrap();
        let target_dir = target_tmp.path().join("partial_save");

        // Do NOT create any source files

        // Build project with a tileset reference
        let mut tilesets = StdHashMap::new();
        tilesets.insert(
            "missing".to_string(),
            TilesetMeta {
                file_path: "tilesets/missing.png".to_string(),
                tile_width: 16,
                tile_height: 16,
                columns: 4,
                rows: 4,
                animations: vec![],
            },
        );

        let mut maps = StdHashMap::new();
        maps.insert(
            "map1".to_string(),
            MapData::new("map1", 4, 4, 16, 16).unwrap(),
        );

        let project = crate::ProjectFile::new(
            maps,
            tilesets,
            None,
            StdHashMap::new(),
            None,
            StdHashMap::new(),
            StdHashMap::new(),
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
        );

        // Build registry with the missing asset
        let mut registry = AssetRegistry::default();
        registry
            .register(AssetReference {
                id: "missing".to_string(),
                relative_path: "tilesets/missing.png".to_string(),
                category: CATEGORY_TILESET.to_string(),
            })
            .unwrap();

        let mut category_dirs = HashMap::new();
        category_dirs.insert(CATEGORY_TILESET.to_string(), "tilesets/".to_string());

        let manager = AssetManager {
            registry,
            category_dirs,
        };

        // Save to directory — should succeed but with a warning
        let warnings = manager
            .save_project(&project, &target_dir, source_dir)
            .unwrap();
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].asset_id, "missing");
        assert_eq!(warnings[0].category, CATEGORY_TILESET);
        assert!(warnings[0].message.contains("not found"));

        // Manifest and maps should still be written
        assert!(target_dir.join("manifest.json").exists());
        assert!(target_dir.join("maps/map1.json").exists());
    }

    #[test]
    fn test_save_project_to_zip_missing_source_emits_warning() {
        use crate::map::MapData;
        use crate::tileset::TilesetMeta;
        use std::collections::HashMap as StdHashMap;
        use std::io::Read;

        let source_tmp = tempfile::tempdir().unwrap();
        let source_dir = source_tmp.path();
        let target_tmp = tempfile::tempdir().unwrap();
        let zip_path = target_tmp.path().join("partial.rpg");

        // Do NOT create source files

        let mut tilesets = StdHashMap::new();
        tilesets.insert(
            "ghost".to_string(),
            TilesetMeta {
                file_path: "tilesets/ghost.png".to_string(),
                tile_width: 16,
                tile_height: 16,
                columns: 4,
                rows: 4,
                animations: vec![],
            },
        );

        let mut maps = StdHashMap::new();
        maps.insert(
            "arena".to_string(),
            MapData::new("arena", 4, 4, 16, 16).unwrap(),
        );

        let project = crate::ProjectFile::new(
            maps,
            tilesets,
            None,
            StdHashMap::new(),
            None,
            StdHashMap::new(),
            StdHashMap::new(),
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
        );

        let mut registry = AssetRegistry::default();
        registry
            .register(AssetReference {
                id: "ghost".to_string(),
                relative_path: "tilesets/ghost.png".to_string(),
                category: CATEGORY_TILESET.to_string(),
            })
            .unwrap();

        let mut category_dirs = HashMap::new();
        category_dirs.insert(CATEGORY_TILESET.to_string(), "tilesets/".to_string());

        let manager = AssetManager {
            registry,
            category_dirs,
        };

        let warnings = manager
            .save_project(&project, &zip_path, source_dir)
            .unwrap();
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].asset_id, "ghost");
        assert!(warnings[0].message.contains("not found"));

        // ZIP should still be valid with manifest and map
        let zip_data = std::fs::read(&zip_path).unwrap();
        let mut archive = zip::ZipArchive::new(std::io::Cursor::new(zip_data)).unwrap();
        let mut manifest_content = String::new();
        archive
            .by_name("manifest.json")
            .unwrap()
            .read_to_string(&mut manifest_content)
            .unwrap();
        assert!(manifest_content.contains("ghost"));

        // The asset file should NOT be in the archive
        assert!(archive.by_name("tilesets/ghost.png").is_err());
    }

    // --- AssetManager::file_exists tests ---

    #[test]
    fn test_file_exists_with_existing_file() {
        let tmp = tempfile::tempdir().unwrap();
        let file_path = tmp.path().join("test.txt");
        std::fs::write(&file_path, b"hello").unwrap();
        assert!(AssetManager::file_exists(&file_path));
    }

    #[test]
    fn test_file_exists_with_missing_path() {
        let path = Path::new("/tmp/nonexistent_file_12345.txt");
        assert!(!AssetManager::file_exists(path));
    }

    #[test]
    fn test_file_exists_with_directory() {
        let tmp = tempfile::tempdir().unwrap();
        assert!(!AssetManager::file_exists(tmp.path()));
    }

    // --- AssetManager::load_file_bytes tests ---

    #[test]
    fn test_load_file_bytes_valid_file() {
        let tmp = tempfile::tempdir().unwrap();
        let file_path = tmp.path().join("data.bin");
        let content = b"binary content here";
        std::fs::write(&file_path, content).unwrap();

        let result = AssetManager::load_file_bytes(&file_path);
        assert_eq!(result.unwrap(), content.to_vec());
    }

    #[test]
    fn test_load_file_bytes_directory_rejected() {
        let tmp = tempfile::tempdir().unwrap();
        let result = AssetManager::load_file_bytes(tmp.path());
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, CommonError::AssetPathError(_)));
        assert!(err.to_string().contains("directory"));
    }

    #[test]
    fn test_load_file_bytes_missing_file() {
        let path = Path::new("/tmp/nonexistent_load_test_12345.bin");
        let result = AssetManager::load_file_bytes(path);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, CommonError::AssetPathError(_)));
        assert!(err.to_string().contains("does not exist"));
    }

    // --- AssetManager::resolve_and_load tests ---

    #[test]
    fn test_resolve_and_load_valid_file() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let subdir = root.join("images");
        std::fs::create_dir_all(&subdir).unwrap();
        let file_path = subdir.join("icon.png");
        let content = b"png bytes";
        std::fs::write(&file_path, content).unwrap();

        let result = AssetManager::resolve_and_load(root, "images/icon.png");
        assert_eq!(result.unwrap(), content.to_vec());
    }

    #[test]
    fn test_resolve_and_load_empty_path() {
        let tmp = tempfile::tempdir().unwrap();
        let result = AssetManager::resolve_and_load(tmp.path(), "");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, CommonError::AssetPathError(_)));
        assert!(err.to_string().contains("empty or whitespace-only"));
    }

    #[test]
    fn test_resolve_and_load_whitespace_only_path() {
        let tmp = tempfile::tempdir().unwrap();
        let result = AssetManager::resolve_and_load(tmp.path(), "   \t  ");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, CommonError::AssetPathError(_)));
        assert!(err.to_string().contains("empty or whitespace-only"));
    }

    #[test]
    fn test_resolve_and_load_trims_whitespace() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let file_path = root.join("hello.txt");
        std::fs::write(&file_path, b"world").unwrap();

        // Path with leading/trailing whitespace should still resolve
        let result = AssetManager::resolve_and_load(root, "  hello.txt  ");
        assert_eq!(result.unwrap(), b"world".to_vec());
    }

    #[test]
    fn test_resolve_and_load_path_traversal_rejected() {
        let tmp = tempfile::tempdir().unwrap();
        let result = AssetManager::resolve_and_load(tmp.path(), "../etc/passwd");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, CommonError::AssetPathError(_)));
    }
}
