use serde::{Deserialize, Serialize};

/// Graphics associated with a game entity (items, abilities).
///
/// Currently holds a single icon field. Designed for future extensibility
/// when category-level graphics are added via the category editor feature.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntityGraphics {
    /// Per-instance icon graphic (relative path within the project).
    /// Maximum 260 characters after trimming. None = no icon assigned.
    #[serde(default)]
    pub icon: Option<String>,
}

impl EntityGraphics {
    /// Sets the icon path. Trims whitespace, rejects empty-after-trim,
    /// truncates to 260 characters.
    pub fn set_icon(&mut self, path: &str) -> Result<(), crate::error::CommonError> {
        let trimmed = path.trim();
        if trimmed.is_empty() {
            return Err(crate::error::CommonError::AssetPathError(
                "Icon path must not be empty or whitespace-only".to_string(),
            ));
        }
        let truncated: String = trimmed.chars().take(260).collect();
        self.icon = Some(truncated);
        Ok(())
    }

    /// Clears the icon path, setting it to None.
    pub fn clear_icon(&mut self) {
        self.icon = None;
    }

    /// Returns true if an icon path is set.
    pub fn has_icon(&self) -> bool {
        self.icon.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_valid_icon() {
        let mut gfx = EntityGraphics::default();
        assert!(gfx.set_icon("sprites/sword.png").is_ok());
        assert_eq!(gfx.icon, Some("sprites/sword.png".to_string()));
    }

    #[test]
    fn test_set_icon_trims_whitespace() {
        let mut gfx = EntityGraphics::default();
        assert!(gfx.set_icon("  sprites/sword.png  ").is_ok());
        assert_eq!(gfx.icon, Some("sprites/sword.png".to_string()));
    }

    #[test]
    fn test_reject_empty_icon() {
        let mut gfx = EntityGraphics::default();
        let result = gfx.set_icon("");
        assert!(result.is_err());
        assert!(gfx.icon.is_none());
    }

    #[test]
    fn test_reject_whitespace_only_icon() {
        let mut gfx = EntityGraphics::default();
        let result = gfx.set_icon("   \t\n  ");
        assert!(result.is_err());
        assert!(gfx.icon.is_none());
    }

    #[test]
    fn test_truncation_to_260_chars() {
        let mut gfx = EntityGraphics::default();
        let long_path: String = "a".repeat(300);
        assert!(gfx.set_icon(&long_path).is_ok());
        assert_eq!(gfx.icon.as_ref().unwrap().len(), 260);
    }

    #[test]
    fn test_exactly_260_chars_not_truncated() {
        let mut gfx = EntityGraphics::default();
        let exact_path: String = "b".repeat(260);
        assert!(gfx.set_icon(&exact_path).is_ok());
        assert_eq!(gfx.icon.as_ref().unwrap().len(), 260);
        assert_eq!(gfx.icon.as_ref().unwrap(), &exact_path);
    }

    #[test]
    fn test_clear_icon() {
        let mut gfx = EntityGraphics::default();
        gfx.set_icon("sprites/sword.png").unwrap();
        assert!(gfx.has_icon());
        gfx.clear_icon();
        assert!(!gfx.has_icon());
        assert!(gfx.icon.is_none());
    }

    #[test]
    fn test_has_icon() {
        let mut gfx = EntityGraphics::default();
        assert!(!gfx.has_icon());
        gfx.set_icon("icons/potion.png").unwrap();
        assert!(gfx.has_icon());
    }

    #[test]
    fn test_default_has_no_icon() {
        let gfx = EntityGraphics::default();
        assert!(!gfx.has_icon());
        assert!(gfx.icon.is_none());
    }
}
