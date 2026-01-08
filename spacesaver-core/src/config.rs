//! Configuration management for the screen saver

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::error::{Error, Result};

/// Image source options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ImageSource {
    /// NASA Astronomy Picture of the Day (requires API key)
    #[default]
    Apod,
    /// NASA Image and Video Library (no API key required)
    NasaImages,
    /// Try APOD first, fall back to NASA Images if unavailable
    ApodWithFallback,
}

impl ImageSource {
    /// Get a human-readable name for the source
    pub fn display_name(&self) -> &'static str {
        match self {
            ImageSource::Apod => "NASA APOD",
            ImageSource::NasaImages => "NASA Image Library",
            ImageSource::ApodWithFallback => "APOD with Fallback",
        }
    }
}

/// Screen saver configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// NASA API key (use DEMO_KEY for testing, get free key at api.nasa.gov)
    pub api_key: String,

    /// Number of images to keep in cache
    pub cache_size: usize,

    /// Interval between image changes in seconds
    pub transition_interval: f64,

    /// Whether to show image title overlay
    pub show_title: bool,

    /// Whether to show image description overlay
    pub show_description: bool,

    /// Transition animation duration in seconds
    pub transition_duration: f64,

    /// Maximum days to fetch images from (going back from today)
    pub max_history_days: u32,

    /// Whether to prefetch images in background
    pub prefetch_enabled: bool,

    /// Image source to use
    #[serde(default)]
    pub image_source: ImageSource,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api_key: "DEMO_KEY".to_string(),
            cache_size: 50,
            transition_interval: 30.0,
            show_title: true,
            show_description: false,
            transition_duration: 2.0,
            max_history_days: 365,
            prefetch_enabled: true,
            image_source: ImageSource::ApodWithFallback,
        }
    }
}

impl Config {
    /// Get the configuration directory path
    pub fn config_dir() -> Result<PathBuf> {
        let dir = directories::ProjectDirs::from("com", "spacesaver", "Spacesaver")
            .ok_or_else(|| Error::Config("Could not determine config directory".to_string()))?;
        Ok(dir.config_dir().to_path_buf())
    }

    /// Get the cache directory path
    pub fn cache_dir() -> Result<PathBuf> {
        let dir = directories::ProjectDirs::from("com", "spacesaver", "Spacesaver")
            .ok_or_else(|| Error::Config("Could not determine cache directory".to_string()))?;
        Ok(dir.cache_dir().to_path_buf())
    }

    /// Get the configuration file path
    pub fn config_path() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join("config.json"))
    }

    /// Load configuration from file, or create default if not exists
    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;

        if path.exists() {
            let content = fs::read_to_string(&path)?;
            let config: Config = serde_json::from_str(&content)?;
            Ok(config)
        } else {
            let config = Config::default();
            config.save()?;
            Ok(config)
        }
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<()> {
        let path = Self::config_path()?;

        // Ensure directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(self)?;
        fs::write(&path, content)?;
        Ok(())
    }

    /// Update API key
    pub fn set_api_key(&mut self, key: String) {
        self.api_key = key;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.api_key, "DEMO_KEY");
        assert_eq!(config.cache_size, 50);
        assert_eq!(config.transition_interval, 30.0);
    }
}
