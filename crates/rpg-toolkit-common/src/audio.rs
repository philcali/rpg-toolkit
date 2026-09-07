//! Shared audio loading utility.
//!
//! Provides [`AudioLoader`], a single consistent path for loading music loops and
//! sound effects from a project. Loading resolves a relative path against a project
//! root (rejecting `..` traversal and empty paths), classifies the audio format by
//! its file extension, and reads the raw bytes of the target file.

use std::path::Path;

use crate::asset::AssetManager;
use crate::error::CommonError;

/// Supported audio container formats, classified by file extension.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AudioFormat {
    /// Ogg Vorbis (`.ogg`).
    Ogg,
    /// Waveform audio (`.wav`).
    Wav,
    /// MPEG-1 Audio Layer III (`.mp3`).
    Mp3,
}

/// Loads audio assets from a project through one consistent path.
///
/// All methods are associated functions; `AudioLoader` holds no state.
pub struct AudioLoader;

impl AudioLoader {
    /// Classify an audio file's format by the extension of `relative_path`.
    ///
    /// The extension is matched case-insensitively against the supported formats
    /// `ogg`, `wav`, and `mp3`.
    ///
    /// Returns an error if:
    /// - The path has no file extension (format could not be determined).
    /// - The extension is present but unsupported (error names the extension).
    pub fn classify_format(relative_path: &str) -> Result<AudioFormat, CommonError> {
        let extension = Path::new(relative_path)
            .extension()
            .and_then(|ext| ext.to_str());

        match extension {
            None => Err(CommonError::AudioLoadError(
                "could not determine audio format: path has no file extension".to_string(),
            )),
            Some(ext) => match ext.to_ascii_lowercase().as_str() {
                "ogg" => Ok(AudioFormat::Ogg),
                "wav" => Ok(AudioFormat::Wav),
                "mp3" => Ok(AudioFormat::Mp3),
                _ => Err(CommonError::AudioLoadError(format!(
                    "unsupported audio format extension: {}",
                    ext
                ))),
            },
        }
    }

    /// Load an audio asset's format and raw bytes.
    ///
    /// Resolves `relative_path` against `root` and returns the classified
    /// [`AudioFormat`] alongside the complete raw byte contents of the file.
    ///
    /// The first failing condition wins, in this order:
    /// 1. `relative_path` is empty or whitespace-only → error, no file read.
    /// 2. `relative_path` contains a `..` component that escapes the root → error,
    ///    no file read.
    /// 3. The extension is missing or unsupported → error, no file read.
    /// 4. The resolved path does not exist, is a directory, or cannot be read → error.
    pub fn load(root: &Path, relative_path: &str) -> Result<(AudioFormat, Vec<u8>), CommonError> {
        // (1) Trim/empty guard — do not read any file.
        let trimmed = relative_path.trim();
        if trimmed.is_empty() {
            return Err(CommonError::AudioLoadError(
                "audio path is empty or whitespace-only".to_string(),
            ));
        }

        // (2) Resolve the path, rejecting `..` traversal without reading a file.
        let resolved = AssetManager::resolve_path(root, trimmed)?;

        // (3) Classify the format by extension before touching the filesystem.
        let format = Self::classify_format(trimmed)?;

        // (4) Load the raw bytes (non-existent / directory / read-failure).
        let bytes = AssetManager::load_file_bytes(&resolved)?;

        Ok((format, bytes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Req 9.6: A relative path resolving to a directory (even one whose name has a
    /// supported audio extension) yields an error indicating the path is not a
    /// regular file, and does not classify it as a loadable audio file.
    #[test]
    fn load_directory_path_returns_not_a_regular_file_error() {
        let root = tempfile::tempdir().expect("create temp root");

        // Create a directory whose name carries a supported audio extension.
        let dir_path = root.path().join("music.ogg");
        std::fs::create_dir(&dir_path).expect("create directory named music.ogg");

        let result = AudioLoader::load(root.path(), "music.ogg");

        let err = result.expect_err("loading a directory path should fail");
        match err {
            CommonError::AssetPathError(message) => {
                assert!(
                    message.contains("is a directory, not a file"),
                    "expected a not-a-regular-file error, got: {message}"
                );
            }
            other => panic!("expected AssetPathError for a directory path, got: {other:?}"),
        }
    }

    /// Req 9.10: A relative path resolving to an existing regular file with a
    /// supported extension that cannot be read yields an error indicating the file
    /// could not be read.
    ///
    /// This is platform-dependent: it relies on Unix file permission bits, so it is
    /// guarded with `#[cfg(unix)]`. Running as root bypasses permission checks, in
    /// which case the read would succeed; the test skips its assertion in that case.
    #[cfg(unix)]
    #[test]
    fn load_unreadable_file_returns_read_error() {
        use std::os::unix::fs::PermissionsExt;

        let root = tempfile::tempdir().expect("create temp root");

        // Create a real audio file with a supported extension.
        let file_path = root.path().join("theme.wav");
        std::fs::write(&file_path, b"RIFF....WAVEfmt ").expect("write audio file");

        // Remove all permissions so the file cannot be read.
        let mut perms = std::fs::metadata(&file_path)
            .expect("read metadata")
            .permissions();
        perms.set_mode(0o000);
        std::fs::set_permissions(&file_path, perms).expect("set no-access permissions");

        let result = AudioLoader::load(root.path(), "theme.wav");

        // Restore permissions so tempfile cleanup can remove the file.
        let mut restore = std::fs::metadata(&file_path)
            .expect("read metadata for restore")
            .permissions();
        restore.set_mode(0o644);
        let _ = std::fs::set_permissions(&file_path, restore);

        // Running as root bypasses permission checks; skip the assertion in that case.
        if result.is_ok() {
            eprintln!(
                "skipping unreadable-file assertion: read succeeded (likely running as root, \
                 permission checks bypassed)"
            );
            return;
        }

        match result.expect_err("reading an unreadable file should fail") {
            CommonError::AssetPathError(message) => {
                assert!(
                    message.contains("failed to read file"),
                    "expected a read error, got: {message}"
                );
            }
            other => panic!("expected AssetPathError for an unreadable file, got: {other:?}"),
        }
    }
}
