use std::collections::HashMap;
use std::path::Path;

use bevy::prelude::*;
use bevy_egui::egui;
use rpg_toolkit_common::asset::AssetManager;

/// LRU texture cache entry.
struct CacheEntry {
    texture: egui::TextureHandle,
    last_used: u64,
}

/// Shared thumbnail rendering utility with LRU caching.
///
/// Loads images from disk via `AssetManager`, decodes them with the `image` crate,
/// uploads them as egui textures, and renders aspect-ratio-preserving previews.
/// Caches up to `max_entries` textures with LRU eviction.
#[derive(Resource)]
pub struct ThumbnailCache {
    entries: HashMap<String, CacheEntry>,
    max_entries: usize,
    frame_counter: u64,
}

impl ThumbnailCache {
    /// Creates a new `ThumbnailCache` with the given maximum entry capacity.
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: HashMap::new(),
            max_entries,
            frame_counter: 0,
        }
    }

    /// Renders a thumbnail for the given relative path.
    ///
    /// - Resolves and loads bytes via `AssetManager::resolve_and_load`
    /// - Checks LRU cache; on miss, loads/decodes/uploads texture
    /// - Renders aspect-ratio-preserving image within `max_size × max_size`
    /// - On failure, renders "Image not found" placeholder label
    pub fn render_thumbnail(
        &mut self,
        ui: &mut egui::Ui,
        project_root: &Path,
        relative_path: &str,
        max_size: u32,
    ) {
        let key = relative_path.to_string();

        // Cache hit: update last_used and render
        if let Some(entry) = self.entries.get_mut(&key) {
            entry.last_used = self.frame_counter;
            let texture = &entry.texture;
            let [w, h] = texture.size();
            let (display_w, display_h) = compute_scaled_size(w as u32, h as u32, max_size);
            ui.image(egui::load::SizedTexture::new(
                texture.id(),
                egui::vec2(display_w, display_h),
            ));
            return;
        }

        // Cache miss: load, decode, upload
        // Try resolve_and_load first (handles relative paths via AssetManager).
        // If the path is absolute, read it directly from the filesystem.
        let bytes = if std::path::Path::new(relative_path).is_absolute() {
            match AssetManager::load_file_bytes(std::path::Path::new(relative_path)) {
                Ok(b) => b,
                Err(_) => {
                    ui.label("Image not found");
                    return;
                }
            }
        } else {
            match AssetManager::resolve_and_load(project_root, relative_path) {
                Ok(b) => b,
                Err(_) => {
                    ui.label("Image not found");
                    return;
                }
            }
        };

        let dynamic_image = match image::load_from_memory(&bytes) {
            Ok(img) => img,
            Err(_) => {
                ui.label("Image not found");
                return;
            }
        };

        let rgba = dynamic_image.to_rgba8();
        let (img_w, img_h) = (rgba.width(), rgba.height());
        let pixels = rgba.into_raw();

        let color_image =
            egui::ColorImage::from_rgba_unmultiplied([img_w as usize, img_h as usize], &pixels);

        let texture = ui.ctx().load_texture(
            format!("thumbnail_{}", relative_path),
            color_image,
            egui::TextureOptions::LINEAR,
        );

        let (display_w, display_h) = compute_scaled_size(img_w, img_h, max_size);
        ui.image(egui::load::SizedTexture::new(
            texture.id(),
            egui::vec2(display_w, display_h),
        ));

        // Evict LRU entry if at capacity
        if self.entries.len() >= self.max_entries {
            self.evict_lru();
        }

        self.entries.insert(
            key,
            CacheEntry {
                texture,
                last_used: self.frame_counter,
            },
        );
    }

    /// Invalidates a specific cache entry (called when path changes).
    pub fn invalidate(&mut self, path: &str) {
        self.entries.remove(path);
    }

    /// Advances the frame counter (called once per frame).
    #[allow(dead_code)]
    pub fn tick(&mut self) {
        self.frame_counter += 1;
    }

    /// Evicts the least-recently-used cache entry.
    fn evict_lru(&mut self) {
        if let Some(lru_key) = self
            .entries
            .iter()
            .min_by_key(|(_, entry)| entry.last_used)
            .map(|(key, _)| key.clone())
        {
            self.entries.remove(&lru_key);
        }
    }
}

impl Default for ThumbnailCache {
    fn default() -> Self {
        Self::new(128)
    }
}

/// Computes the display size for an image that fits within a `max_size × max_size`
/// bounding box while preserving aspect ratio. Images smaller than `max_size` in
/// both dimensions are displayed at their native size (no upscaling).
pub fn compute_scaled_size(width: u32, height: u32, max_size: u32) -> (f32, f32) {
    let max = max_size as f32;
    let scale = (max / width as f32).min(max / height as f32).min(1.0);
    (width as f32 * scale, height as f32 * scale)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_scaled_size_landscape() {
        // 100×50, max 64: scale = min(64/100, 64/50, 1.0) = min(0.64, 1.28, 1.0) = 0.64
        let (w, h) = compute_scaled_size(100, 50, 64);
        assert!((w - 64.0).abs() < 0.01, "width should be 64.0, got {}", w);
        assert!((h - 32.0).abs() < 0.01, "height should be 32.0, got {}", h);
    }

    #[test]
    fn test_compute_scaled_size_no_upscale() {
        // 32×32, max 64: scale = min(64/32, 64/32, 1.0) = min(2.0, 2.0, 1.0) = 1.0
        let (w, h) = compute_scaled_size(32, 32, 64);
        assert!((w - 32.0).abs() < 0.01, "width should be 32.0, got {}", w);
        assert!((h - 32.0).abs() < 0.01, "height should be 32.0, got {}", h);
    }

    #[test]
    fn test_compute_scaled_size_large_landscape() {
        // 200×100, max 64: scale = min(64/200, 64/100, 1.0) = min(0.32, 0.64, 1.0) = 0.32
        let (w, h) = compute_scaled_size(200, 100, 64);
        assert!((w - 64.0).abs() < 0.01, "width should be 64.0, got {}", w);
        assert!((h - 32.0).abs() < 0.01, "height should be 32.0, got {}", h);
    }

    #[test]
    fn test_compute_scaled_size_tiny() {
        // 1×1, max 64: scale = min(64/1, 64/1, 1.0) = min(64.0, 64.0, 1.0) = 1.0
        let (w, h) = compute_scaled_size(1, 1, 64);
        assert!((w - 1.0).abs() < 0.01, "width should be 1.0, got {}", w);
        assert!((h - 1.0).abs() < 0.01, "height should be 1.0, got {}", h);
    }
}
