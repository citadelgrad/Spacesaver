//! Spacesaver Core Library
//!
//! Rust library for fetching and managing NASA APOD images for the macOS screen saver.

pub mod api;
pub mod cache;
mod config;
pub mod error;
mod ffi;
mod image_manager;

pub use api::NasaApodApi;
pub use cache::ImageCache;
pub use config::Config;
pub use error::{Error, Result};
pub use image_manager::ImageManager;

// Re-export FFI functions
pub use ffi::*;
