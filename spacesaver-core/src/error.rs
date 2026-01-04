//! Error types for the spacesaver library

use thiserror::Error;

/// Custom error type for spacesaver operations
#[derive(Error, Debug)]
pub enum Error {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON parsing failed: {0}")]
    Json(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Image processing error: {0}")]
    Image(#[from] image::ImageError),

    #[error("API error: {0}")]
    Api(String),

    #[error("Cache error: {0}")]
    Cache(String),

    #[error("No images available")]
    NoImages,

    #[error("Invalid date format: {0}")]
    InvalidDate(String),

    #[error("Configuration error: {0}")]
    Config(String),
}

/// Result type alias for spacesaver operations
pub type Result<T> = std::result::Result<T, Error>;
