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
}
