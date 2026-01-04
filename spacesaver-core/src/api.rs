//! NASA APOD API client

use chrono::{Duration, NaiveDate, Utc};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

const APOD_BASE_URL: &str = "https://api.nasa.gov/planetary/apod";

/// Response from the NASA APOD API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApodResponse {
    /// The date of the APOD
    pub date: String,

    /// Title of the image/video
    pub title: String,

    /// Explanation/description of the image
    pub explanation: String,

    /// URL of the image (standard resolution)
    pub url: String,

    /// URL of the high-definition image (if available)
    pub hdurl: Option<String>,

    /// Media type (image or video)
    pub media_type: String,

    /// Copyright information (if applicable)
    pub copyright: Option<String>,

    /// Service version
    pub service_version: Option<String>,

    /// Thumbnail URL for videos
    pub thumbnail_url: Option<String>,
}

impl ApodResponse {
    /// Get the best available image URL (prefers HD)
    pub fn best_image_url(&self) -> Option<&str> {
        if self.media_type == "image" {
            self.hdurl.as_deref().or(Some(self.url.as_str()))
        } else {
            // For videos, use thumbnail if available
            self.thumbnail_url.as_deref()
        }
    }

    /// Check if this is an image (not a video)
    pub fn is_image(&self) -> bool {
        self.media_type == "image"
    }
}

/// NASA APOD API client
pub struct NasaApodApi {
    client: Client,
    api_key: String,
}

impl NasaApodApi {
    /// Create a new API client with the given API key
    pub fn new(api_key: &str) -> Self {
        Self {
            client: Client::builder()
                .user_agent("Spacesaver-macOS-ScreenSaver/0.1")
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("Failed to create HTTP client"),
            api_key: api_key.to_string(),
        }
    }

    /// Fetch the APOD for a specific date
    pub fn fetch_by_date(&self, date: NaiveDate) -> Result<ApodResponse> {
        let url = format!(
            "{}?api_key={}&date={}",
            APOD_BASE_URL,
            self.api_key,
            date.format("%Y-%m-%d")
        );

        log::debug!("Fetching APOD for date: {}", date);

        let response = self.client.get(&url).send()?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().unwrap_or_default();
            return Err(Error::Api(format!(
                "API request failed with status {}: {}",
                status, body
            )));
        }

        let apod: ApodResponse = response.json()?;
        Ok(apod)
    }

    /// Fetch today's APOD
    pub fn fetch_today(&self) -> Result<ApodResponse> {
        let today = Utc::now().date_naive();
        self.fetch_by_date(today)
    }

    /// Fetch APODs for a date range
    pub fn fetch_range(&self, start_date: NaiveDate, end_date: NaiveDate) -> Result<Vec<ApodResponse>> {
        let url = format!(
            "{}?api_key={}&start_date={}&end_date={}",
            APOD_BASE_URL,
            self.api_key,
            start_date.format("%Y-%m-%d"),
            end_date.format("%Y-%m-%d")
        );

        log::debug!("Fetching APOD range: {} to {}", start_date, end_date);

        let response = self.client.get(&url).send()?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().unwrap_or_default();
            return Err(Error::Api(format!(
                "API request failed with status {}: {}",
                status, body
            )));
        }

        let apods: Vec<ApodResponse> = response.json()?;
        Ok(apods)
    }

    /// Fetch random APODs
    pub fn fetch_random(&self, count: u32) -> Result<Vec<ApodResponse>> {
        let url = format!(
            "{}?api_key={}&count={}",
            APOD_BASE_URL, self.api_key, count
        );

        log::debug!("Fetching {} random APODs", count);

        let response = self.client.get(&url).send()?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().unwrap_or_default();
            return Err(Error::Api(format!(
                "API request failed with status {}: {}",
                status, body
            )));
        }

        let apods: Vec<ApodResponse> = response.json()?;
        Ok(apods)
    }

    /// Fetch recent APODs (last N days)
    pub fn fetch_recent(&self, days: u32) -> Result<Vec<ApodResponse>> {
        let end_date = Utc::now().date_naive();
        let start_date = end_date - Duration::days(days as i64);
        self.fetch_range(start_date, end_date)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apod_response_parsing() {
        let json = r#"{
            "date": "2024-01-15",
            "title": "Test Image",
            "explanation": "This is a test",
            "url": "https://example.com/image.jpg",
            "hdurl": "https://example.com/image_hd.jpg",
            "media_type": "image"
        }"#;

        let apod: ApodResponse = serde_json::from_str(json).unwrap();
        assert_eq!(apod.title, "Test Image");
        assert_eq!(apod.best_image_url(), Some("https://example.com/image_hd.jpg"));
        assert!(apod.is_image());
    }

    #[test]
    fn test_video_response() {
        let json = r#"{
            "date": "2024-01-15",
            "title": "Test Video",
            "explanation": "This is a test",
            "url": "https://example.com/video",
            "media_type": "video",
            "thumbnail_url": "https://example.com/thumb.jpg"
        }"#;

        let apod: ApodResponse = serde_json::from_str(json).unwrap();
        assert!(!apod.is_image());
        assert_eq!(apod.best_image_url(), Some("https://example.com/thumb.jpg"));
    }
}
