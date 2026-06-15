#[derive(Debug, thiserror::Error)]
pub enum CommonError {
    #[error("Invalid map dimensions: width and height must be between 1 and 256")]
    InvalidDimensions,
    #[error("Invalid tile size: must be one of 8, 16, 32, 64")]
    InvalidTileSize,
    #[error("Failed to parse project file: {0}")]
    ProjectParseError(String),
    #[error("Invalid project data: {0}")]
    ProjectValidationError(String),
    #[error("Failed to read project directory: {0}")]
    ProjectDirectoryError(String),
    #[error("Failed to process ZIP archive: {0}")]
    ZipError(String),
    #[error("animation must have at least 2 frames")]
    AnimationTooFewFrames,
    #[error("frame duration must be greater than zero")]
    AnimationInvalidDuration,
    #[error("frame ({col}, {row}) out of tileset bounds")]
    AnimationFrameOutOfBounds { col: u32, row: u32 },
    #[error("Character validation error: {0}")]
    CharacterValidationError(String),
    #[error("Item validation error: {0}")]
    ItemValidationError(String),
    #[error("Ability validation error: {0}")]
    AbilityValidationError(String),
    #[error("Enemy validation error: {0}")]
    EnemyValidationError(String),
}
