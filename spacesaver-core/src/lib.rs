//! Spacesaver Core Library
//!
//! Rust library for fetching and managing NASA space images for the macOS screen saver.
//! Supports multiple image sources: NASA APOD, NASA Image Library, and more.

pub mod api;
pub mod cache;
mod config;
pub mod error;
mod ffi;
mod image_manager;
pub mod logging;
pub mod nasa_images;

pub use api::NasaApodApi;
pub use cache::ImageCache;
pub use config::{Config, ImageSource};
pub use error::{Error, Result};
pub use image_manager::ImageManager;
pub use nasa_images::NasaImagesApi;

// Re-export FFI functions
pub use ffi::*;
