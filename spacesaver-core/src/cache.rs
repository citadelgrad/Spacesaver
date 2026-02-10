//! Image cache management

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::api::ApodResponse;
use crate::config::Config;
use crate::error::Result;

/// Metadata for a cached image
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedImageMetadata {
    /// Date of the APOD
    pub date: String,
    /// Title of the image
    pub title: String,
    /// Description/explanation
    pub explanation: String,
    /// Copyright info if any
    pub copyright: Option<String>,
    /// Original URL
    pub original_url: String,
    /// Local filename
    pub filename: String,
    /// When this was cached (Unix timestamp)
    pub cached_at: i64,
}

/// Image cache index
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CacheIndex {
    /// Map of date -> metadata
    pub images: HashMap<String, CachedImageMetadata>,
    /// Last update timestamp
    pub last_updated: i64,
}

/// Image cache manager
pub struct ImageCache {
    cache_dir: PathBuf,
    index: CacheIndex,
    max_size: usize,
}

impl ImageCache {
    /// Create or open an image cache
    pub fn new(max_size: usize) -> Result<Self> {
        let cache_dir = Config::cache_dir()?;
        fs::create_dir_all(&cache_dir)?;

        let index = Self::load_index(&cache_dir)?;

        Ok(Self {
            cache_dir,
            index,
            max_size,
        })
    }

    /// Get the cache directory path
    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }

    /// Load the cache index from disk
    fn load_index(cache_dir: &Path) -> Result<CacheIndex> {
        let index_path = cache_dir.join("index.json");

        if index_path.exists() {
            let content = fs::read_to_string(&index_path)?;
            let index: CacheIndex = serde_json::from_str(&content)?;
            Ok(index)
        } else {
            Ok(CacheIndex::default())
        }
    }

    /// Save the cache index to disk
    fn save_index(&self) -> Result<()> {
        let index_path = self.cache_dir.join("index.json");
        let content = serde_json::to_string_pretty(&self.index)?;
        fs::write(&index_path, content)?;
        Ok(())
    }

    /// Check if an image for a date is cached
    pub fn has_image(&self, date: &str) -> bool {
        if let Some(metadata) = self.index.images.get(date) {
            let image_path = self.cache_dir.join(&metadata.filename);
            image_path.exists()
        } else {
            false
        }
    }

    /// Get metadata for a cached image
    pub fn get_metadata(&self, date: &str) -> Option<&CachedImageMetadata> {
        self.index.images.get(date)
    }

    /// Get the file path for a cached image
    pub fn get_image_path(&self, date: &str) -> Option<PathBuf> {
        self.index
            .images
            .get(date)
            .map(|m| self.cache_dir.join(&m.filename))
    }

    /// Store an image in the cache
    pub fn store_image(&mut self, apod: &ApodResponse, image_data: &[u8]) -> Result<PathBuf> {
        // Determine file extension from URL
        let extension = apod
            .best_image_url()
            .and_then(|url| {
                url.rsplit('.')
                    .next()
                    .map(|ext| ext.split('?').next().unwrap_or(ext))
            })
            .unwrap_or("jpg");

        let filename = format!("apod_{}.{}", apod.date, extension);
        let file_path = self.cache_dir.join(&filename);

        // Write image data
        fs::write(&file_path, image_data)?;

        // Create metadata
        let metadata = CachedImageMetadata {
            date: apod.date.clone(),
            title: apod.title.clone(),
            explanation: apod.explanation.clone(),
            copyright: apod.copyright.clone(),
            original_url: apod.best_image_url().unwrap_or(&apod.url).to_string(),
            filename,
            cached_at: chrono::Utc::now().timestamp(),
        };

        // Update index
        self.index.images.insert(apod.date.clone(), metadata);
        self.index.last_updated = chrono::Utc::now().timestamp();

        // Enforce cache size limit
        self.enforce_size_limit()?;

        // Save index
        self.save_index()?;

        Ok(file_path)
    }

    /// Get all cached image dates, sorted by date (newest first)
    pub fn get_all_dates(&self) -> Vec<String> {
        let mut dates: Vec<_> = self.index.images.keys().cloned().collect();
        dates.sort();
        dates.reverse();
        dates
    }

    /// Get all cached images metadata
    pub fn get_all_metadata(&self) -> Vec<&CachedImageMetadata> {
        self.index.images.values().collect()
    }

    /// Get count of cached images
    pub fn image_count(&self) -> usize {
        self.index.images.len()
    }

    /// Enforce the maximum cache size by removing oldest entries
    fn enforce_size_limit(&mut self) -> Result<()> {
        while self.index.images.len() > self.max_size {
            // Find oldest entry by cached_at
            if let Some((oldest_date, _)) = self
                .index
                .images
                .iter()
                .min_by_key(|(_, m)| m.cached_at)
                .map(|(d, m)| (d.clone(), m.filename.clone()))
            {
                self.remove_image(&oldest_date)?;
            } else {
                break;
            }
        }
        Ok(())
    }

    /// Remove an image from the cache
    pub fn remove_image(&mut self, date: &str) -> Result<()> {
        if let Some(metadata) = self.index.images.remove(date) {
            let file_path = self.cache_dir.join(&metadata.filename);
            if file_path.exists() {
                fs::remove_file(file_path)?;
            }
        }
        Ok(())
    }

    /// Clear all cached images
    pub fn clear(&mut self) -> Result<()> {
        for metadata in self.index.images.values() {
            let file_path = self.cache_dir.join(&metadata.filename);
            if file_path.exists() {
                let _ = fs::remove_file(file_path);
            }
        }
        self.index.images.clear();
        self.index.last_updated = chrono::Utc::now().timestamp();
        self.save_index()?;
        Ok(())
    }

    /// Get a random cached image path
    pub fn get_random_image(&self) -> Option<(PathBuf, &CachedImageMetadata)> {
        use rand::seq::SliceRandom;

        let dates: Vec<_> = self.index.images.keys().collect();
        if dates.is_empty() {
            return None;
        }

        let mut rng = rand::thread_rng();
        let date = dates.choose(&mut rng)?;

        self.index
            .images
            .get(*date)
            .map(|m| (self.cache_dir.join(&m.filename), m))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_cache() -> (ImageCache, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let cache = ImageCache {
            cache_dir: temp_dir.path().to_path_buf(),
            index: CacheIndex::default(),
            max_size: 10,
        };
        (cache, temp_dir)
    }

    fn make_apod(date: &str) -> ApodResponse {
        ApodResponse {
            date: date.to_string(),
            title: format!("Test {}", date),
            explanation: "Test explanation".to_string(),
            url: format!("https://example.com/{}.jpg", date),
            hdurl: None,
            media_type: "image".to_string(),
            copyright: None,
            service_version: None,
            thumbnail_url: None,
        }
    }

    fn create_test_cache_with_max(max_size: usize) -> (ImageCache, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let cache = ImageCache {
            cache_dir: temp_dir.path().to_path_buf(),
            index: CacheIndex::default(),
            max_size,
        };
        (cache, temp_dir)
    }

    #[test]
    fn test_cache_operations() {
        let (mut cache, _temp) = create_test_cache();

        let apod = make_apod("2024-01-15");
        let data = b"fake image data";
        let path = cache.store_image(&apod, data).unwrap();

        assert!(cache.has_image("2024-01-15"));
        assert!(path.exists());
        assert_eq!(cache.image_count(), 1);
    }

    #[test]
    fn test_has_image_returns_false_for_missing_date() {
        let (cache, _temp) = create_test_cache();
        assert!(!cache.has_image("2024-01-01"));
    }

    #[test]
    fn test_enforce_size_limit_evicts_oldest() {
        let (mut cache, _temp) = create_test_cache_with_max(3);

        // Add exactly max_size items with staggered cached_at times
        for i in 1..=3 {
            let apod = make_apod(&format!("2024-01-{:02}", i));
            cache.store_image(&apod, b"data").unwrap();
            // Ensure different cached_at by manually adjusting
            if let Some(m) = cache.index.images.get_mut(&format!("2024-01-{:02}", i)) {
                m.cached_at = i as i64;
            }
        }
        assert_eq!(cache.image_count(), 3);

        // Add one more — oldest (cached_at=1, date 2024-01-01) should be evicted
        let apod = make_apod("2024-01-04");
        cache.store_image(&apod, b"data").unwrap();
        if let Some(m) = cache.index.images.get_mut("2024-01-04") {
            m.cached_at = 4;
        }

        assert_eq!(cache.image_count(), 3);
        assert!(!cache.has_image("2024-01-01"), "oldest entry should be evicted");
        assert!(cache.has_image("2024-01-04"), "newest entry should exist");
    }

    #[test]
    fn test_enforce_size_limit_boundary_at_max() {
        let (mut cache, _temp) = create_test_cache_with_max(3);

        // Add exactly max_size items — no eviction should happen
        for i in 1..=3 {
            cache.store_image(&make_apod(&format!("2024-01-{:02}", i)), b"data").unwrap();
        }
        assert_eq!(cache.image_count(), 3);
        assert!(cache.has_image("2024-01-01"));
        assert!(cache.has_image("2024-01-02"));
        assert!(cache.has_image("2024-01-03"));
    }

    #[test]
    fn test_get_all_dates_sorted_newest_first() {
        let (mut cache, _temp) = create_test_cache();

        cache.store_image(&make_apod("2024-01-01"), b"data").unwrap();
        cache.store_image(&make_apod("2024-01-03"), b"data").unwrap();
        cache.store_image(&make_apod("2024-01-02"), b"data").unwrap();

        let dates = cache.get_all_dates();
        assert_eq!(dates, vec!["2024-01-03", "2024-01-02", "2024-01-01"]);
    }
}
