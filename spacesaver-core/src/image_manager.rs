//! Image manager - coordinates fetching and caching of NASA images

use std::path::PathBuf;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use chrono::{NaiveDate, Utc};
use parking_lot::RwLock;
use rand::seq::SliceRandom;

use crate::api::{ApodResponse, NasaApodApi};
use crate::cache::{CachedImageMetadata, ImageCache};
use crate::config::Config;
use crate::error::{Error, Result};

/// Current image info for display
#[derive(Debug, Clone)]
pub struct CurrentImage {
    pub path: PathBuf,
    pub title: String,
    pub explanation: String,
    pub date: String,
    pub copyright: Option<String>,
}

/// Image manager state
struct ManagerState {
    cache: ImageCache,
    current_index: usize,
    shuffled_dates: Vec<String>,
    is_fetching: bool,
}

/// Manages NASA APOD images - fetching, caching, and serving
pub struct ImageManager {
    api: NasaApodApi,
    config: Config,
    state: Arc<RwLock<ManagerState>>,
}

impl ImageManager {
    /// Create a new image manager
    pub fn new(config: Config) -> Result<Self> {
        let api = NasaApodApi::new(&config.api_key);
        let cache = ImageCache::new(config.cache_size)?;

        // Get shuffled list of cached dates
        let mut dates = cache.get_all_dates();
        {
            let mut rng = rand::thread_rng();
            dates.shuffle(&mut rng);
        }

        let state = ManagerState {
            cache,
            current_index: 0,
            shuffled_dates: dates,
            is_fetching: false,
        };

        Ok(Self {
            api,
            config,
            state: Arc::new(RwLock::new(state)),
        })
    }

    /// Get the number of cached images
    pub fn cached_count(&self) -> usize {
        self.state.read().cache.image_count()
    }

    /// Check if currently fetching images
    pub fn is_fetching(&self) -> bool {
        self.state.read().is_fetching
    }

    /// Fetch and cache a single image by date
    pub fn fetch_image(&self, date: NaiveDate) -> Result<PathBuf> {
        let date_str = date.format("%Y-%m-%d").to_string();

        // Check if already cached
        {
            let state = self.state.read();
            if let Some(path) = state.cache.get_image_path(&date_str) {
                if path.exists() {
                    return Ok(path);
                }
            }
        }

        // Fetch from API
        let apod = self.api.fetch_by_date(date)?;

        // Only process images, skip videos
        if !apod.is_image() {
            return Err(Error::Api(format!(
                "APOD for {} is a video, not an image",
                date_str
            )));
        }

        // Download the image
        let image_url = apod
            .best_image_url()
            .ok_or_else(|| Error::Api("No image URL available".to_string()))?;

        log::info!("Downloading image: {}", image_url);

        let client = reqwest::blocking::Client::new();
        let response = client.get(image_url).send()?;
        let image_data = response.bytes()?;

        // Store in cache
        let mut state = self.state.write();
        let path = state.cache.store_image(&apod, &image_data)?;

        // Update shuffled dates
        if !state.shuffled_dates.contains(&date_str) {
            state.shuffled_dates.push(date_str);
        }

        Ok(path)
    }

    /// Fetch multiple random images to populate cache
    pub fn fetch_random_images(&self, count: u32) -> Result<Vec<PathBuf>> {
        {
            let mut state = self.state.write();
            state.is_fetching = true;
        }

        let result = self.do_fetch_random(count);

        {
            let mut state = self.state.write();
            state.is_fetching = false;
        }

        result
    }

    fn do_fetch_random(&self, count: u32) -> Result<Vec<PathBuf>> {
        let apods = self.api.fetch_random(count)?;
        let mut paths = Vec::new();

        for apod in apods {
            if !apod.is_image() {
                continue;
            }

            // Check if already cached
            {
                let state = self.state.read();
                if state.cache.has_image(&apod.date) {
                    if let Some(path) = state.cache.get_image_path(&apod.date) {
                        paths.push(path);
                        continue;
                    }
                }
            }

            // Download image
            if let Some(image_url) = apod.best_image_url() {
                log::info!("Downloading: {} - {}", apod.date, apod.title);

                match self.download_and_cache(&apod, image_url) {
                    Ok(path) => paths.push(path),
                    Err(e) => log::warn!("Failed to download {}: {}", apod.date, e),
                }

                // Small delay to be nice to the API
                thread::sleep(Duration::from_millis(100));
            }
        }

        Ok(paths)
    }

    /// Fetch recent images (last N days)
    pub fn fetch_recent_images(&self, days: u32) -> Result<Vec<PathBuf>> {
        {
            let mut state = self.state.write();
            state.is_fetching = true;
        }

        let result = self.do_fetch_recent(days);

        {
            let mut state = self.state.write();
            state.is_fetching = false;
        }

        result
    }

    fn do_fetch_recent(&self, days: u32) -> Result<Vec<PathBuf>> {
        let apods = self.api.fetch_recent(days)?;
        let mut paths = Vec::new();

        for apod in apods {
            if !apod.is_image() {
                continue;
            }

            // Check if already cached
            {
                let state = self.state.read();
                if state.cache.has_image(&apod.date) {
                    if let Some(path) = state.cache.get_image_path(&apod.date) {
                        paths.push(path);
                        continue;
                    }
                }
            }

            // Download image
            if let Some(image_url) = apod.best_image_url() {
                log::info!("Downloading: {} - {}", apod.date, apod.title);

                match self.download_and_cache(&apod, image_url) {
                    Ok(path) => paths.push(path),
                    Err(e) => log::warn!("Failed to download {}: {}", apod.date, e),
                }

                // Small delay to be nice to the API
                thread::sleep(Duration::from_millis(100));
            }
        }

        // Reshuffle dates
        self.reshuffle_dates();

        Ok(paths)
    }

    fn download_and_cache(&self, apod: &ApodResponse, image_url: &str) -> Result<PathBuf> {
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(60))
            .build()?;

        let response = client.get(image_url).send()?;
        let image_data = response.bytes()?;

        let mut state = self.state.write();
        let path = state.cache.store_image(apod, &image_data)?;

        if !state.shuffled_dates.contains(&apod.date) {
            state.shuffled_dates.push(apod.date.clone());
        }

        Ok(path)
    }

    /// Reshuffle the image order
    pub fn reshuffle_dates(&self) {
        let mut state = self.state.write();
        let mut rng = rand::thread_rng();
        state.shuffled_dates.shuffle(&mut rng);
        state.current_index = 0;
    }

    /// Get the next image in the shuffle order
    pub fn next_image(&self) -> Option<CurrentImage> {
        let mut state = self.state.write();

        if state.shuffled_dates.is_empty() {
            return None;
        }

        // Advance index
        state.current_index = (state.current_index + 1) % state.shuffled_dates.len();
        let date = &state.shuffled_dates[state.current_index];

        if let Some(metadata) = state.cache.get_metadata(date) {
            let path = state.cache.cache_dir().join(&metadata.filename);
            if path.exists() {
                return Some(CurrentImage {
                    path,
                    title: metadata.title.clone(),
                    explanation: metadata.explanation.clone(),
                    date: metadata.date.clone(),
                    copyright: metadata.copyright.clone(),
                });
            }
        }

        None
    }

    /// Get a random image
    pub fn random_image(&self) -> Option<CurrentImage> {
        let state = self.state.read();

        state.cache.get_random_image().map(|(path, metadata)| CurrentImage {
            path,
            title: metadata.title.clone(),
            explanation: metadata.explanation.clone(),
            date: metadata.date.clone(),
            copyright: metadata.copyright.clone(),
        })
    }

    /// Get current image info
    pub fn current_image(&self) -> Option<CurrentImage> {
        let state = self.state.read();

        if state.shuffled_dates.is_empty() {
            return None;
        }

        let date = &state.shuffled_dates[state.current_index];

        if let Some(metadata) = state.cache.get_metadata(date) {
            let path = state.cache.cache_dir().join(&metadata.filename);
            if path.exists() {
                return Some(CurrentImage {
                    path,
                    title: metadata.title.clone(),
                    explanation: metadata.explanation.clone(),
                    date: metadata.date.clone(),
                    copyright: metadata.copyright.clone(),
                });
            }
        }

        None
    }

    /// Start background prefetching
    pub fn start_prefetch(&self, count: u32) {
        let state = Arc::clone(&self.state);
        let api_key = self.config.api_key.clone();

        thread::spawn(move || {
            let api = NasaApodApi::new(&api_key);

            // Mark as fetching
            {
                let mut s = state.write();
                if s.is_fetching {
                    return; // Already fetching
                }
                s.is_fetching = true;
            }

            // Fetch random images
            match api.fetch_random(count) {
                Ok(apods) => {
                    for apod in apods {
                        if !apod.is_image() {
                            continue;
                        }

                        // Check if already have it
                        {
                            let s = state.read();
                            if s.cache.has_image(&apod.date) {
                                continue;
                            }
                        }

                        if let Some(image_url) = apod.best_image_url() {
                            log::debug!("Prefetching: {}", apod.title);

                            let client = reqwest::blocking::Client::new();
                            if let Ok(response) = client.get(image_url).send() {
                                if let Ok(data) = response.bytes() {
                                    let mut s = state.write();
                                    if let Ok(_) = s.cache.store_image(&apod, &data) {
                                        s.shuffled_dates.push(apod.date);
                                    }
                                }
                            }

                            thread::sleep(Duration::from_millis(200));
                        }
                    }
                }
                Err(e) => log::warn!("Prefetch failed: {}", e),
            }

            // Done fetching
            {
                let mut s = state.write();
                s.is_fetching = false;
            }
        });
    }

    /// Clear the image cache
    pub fn clear_cache(&self) -> Result<()> {
        let mut state = self.state.write();
        state.cache.clear()?;
        state.shuffled_dates.clear();
        state.current_index = 0;
        Ok(())
    }
}
