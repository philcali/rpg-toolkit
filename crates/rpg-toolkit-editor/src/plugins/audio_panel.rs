//! Shared audio helpers used by the Audio database page (`audio_page.rs`).
//!
//! Provides file-name labeling, category path normalization
//! (`audio/music/<file>` and `audio/sfx/<file>`), and a native audio file
//! picker filtered to the supported formats. The registration UI itself now
//! lives on the dedicated Audio database page.

/// File-name label helper: shows just the file name for a selected path.
///
/// Shared with the Audio database page (`audio_page.rs`).
pub(crate) fn file_label(path: &Option<String>) -> String {
    match path {
        Some(p) => std::path::Path::new(p)
            .file_name()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| p.clone()),
        None => "No file selected".to_string(),
    }
}

/// Normalizes a picked absolute path to the on-disk relative path used by the
/// music loop category subdirectory (`audio/music/<file-name>`).
///
/// Shared with the Audio database page (`audio_page.rs`).
pub(crate) fn music_relative_path(abs_path: &str) -> String {
    let name = std::path::Path::new(abs_path)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");
    format!("audio/music/{}", name)
}

/// Normalizes a picked absolute path to the on-disk relative path used by the
/// sound effect category subdirectory (`audio/sfx/<file-name>`).
///
/// Shared with the Audio database page (`audio_page.rs`).
pub(crate) fn sfx_relative_path(abs_path: &str) -> String {
    let name = std::path::Path::new(abs_path)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");
    format!("audio/sfx/{}", name)
}

/// Opens a native file picker filtered to supported audio formats, returning
/// the selected absolute path (or `None` if the dialog was cancelled).
///
/// Shared with the Audio database page (`audio_page.rs`).
pub(crate) fn pick_audio_file() -> Option<String> {
    rfd::FileDialog::new()
        .add_filter("Audio", &["ogg", "wav", "mp3"])
        .pick_file()
        .map(|path| path.to_string_lossy().to_string())
}
