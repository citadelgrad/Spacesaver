//! NASA Image and Video Library API client
//!
//! This API does NOT require an API key and provides access to NASA's
//! extensive image and video library (140,000+ assets).
//!
//! Documentation: https://images.nasa.gov/docs/images.nasa.gov_api_docs.pdf

use reqwest::blocking::Client;
use serde::Deserialize;

use crate::api::ApodResponse;
use crate::error::{Error, Result};

const NASA_IMAGES_BASE_URL: &str = "https://images-api.nasa.gov";

/// A single item from the NASA Image Library search results
#[derive(Debug, Clone, Deserialize)]
pub struct NasaImageItem {
    pub href: String,
    pub data: Vec<NasaImageData>,
    pub links: Option<Vec<NasaImageLink>>,
}

/// Metadata for a NASA image
#[derive(Debug, Clone, Deserialize)]
pub struct NasaImageData {
    pub title: String,
    pub description: Option<String>,
    pub date_created: Option<String>,
    pub nasa_id: String,
    pub media_type: String,
    pub keywords: Option<Vec<String>>,
    pub center: Option<String>,
    pub photographer: Option<String>,
}

/// Link to image asset
#[derive(Debug, Clone, Deserialize)]
pub struct NasaImageLink {
    pub href: String,
    pub rel: String,
    pub render: Option<String>,
}

/// Collection response from NASA Image Library
#[derive(Debug, Clone, Deserialize)]
pub struct NasaImagesCollection {
    pub items: Vec<NasaImageItem>,
}

/// Root response from NASA Image Library
#[derive(Debug, Clone, Deserialize)]
pub struct NasaImagesResponse {
    pub collection: NasaImagesCollection,
}

/// Asset manifest response
#[derive(Debug, Clone, Deserialize)]
pub struct AssetManifest {
    pub collection: AssetCollection,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AssetCollection {
    pub items: Vec<AssetItem>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AssetItem {
    pub href: String,
}

/// NASA Image Library API client (no API key required)
pub struct NasaImagesApi {
    client: Client,
}

impl NasaImagesApi {
    /// Create a new NASA Images API client
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .user_agent("Spacesaver-macOS-ScreenSaver/0.1")
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("Failed to create HTTP client"),
        }
    }

    /// Search for space/astronomy images
    pub fn search(&self, query: &str, page: u32) -> Result<Vec<NasaImageItem>> {
        let url = format!(
            "{}/search?q={}&media_type=image&page={}",
            NASA_IMAGES_BASE_URL,
            urlencoding::encode(query),
            page
        );

        log::debug!("Searching NASA Images: {}", url);

        let response = self.client.get(&url).send()?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().unwrap_or_default();
            return Err(Error::Api(format!(
                "NASA Images API request failed with status {}: {}",
                status, body
            )));
        }

        let result: NasaImagesResponse = response.json()?;
        Ok(result.collection.items)
    }

    /// Search for popular space imagery
    pub fn search_space_images(&self, count: u32) -> Result<Vec<NasaImageItem>> {
        // Use varied search terms for diverse results
        let search_terms = [
            "nebula",
            "galaxy",
            "hubble",
            "james webb",
            "supernova",
            "solar system",
            "milky way",
            "aurora",
            "eclipse",
            "saturn",
            "jupiter",
            "mars landscape",
            "earth from space",
            "astronaut spacewalk",
            "international space station",
        ];

        let mut all_items = Vec::new();
        let items_per_term = (count as usize / search_terms.len()).max(1);

        for term in search_terms.iter() {
            match self.search(term, 1) {
                Ok(items) => {
                    // Only take images with preview links
                    let valid_items: Vec<_> = items
                        .into_iter()
                        .filter(|item| {
                            item.links
                                .as_ref()
                                .is_some_and(|links| links.iter().any(|l| l.rel == "preview"))
                        })
                        .take(items_per_term)
                        .collect();
                    all_items.extend(valid_items);
                }
                Err(e) => {
                    log::warn!("Search for '{}' failed: {}", term, e);
                }
            }

            if all_items.len() >= count as usize {
                break;
            }

            // Small delay between requests
            std::thread::sleep(std::time::Duration::from_millis(100));
        }

        Ok(all_items)
    }

    /// Get the full-resolution image URL for an asset
    pub fn get_asset_url(&self, nasa_id: &str) -> Result<String> {
        let url = format!("{}/asset/{}", NASA_IMAGES_BASE_URL, nasa_id);

        log::debug!("Fetching asset manifest: {}", url);

        let response = self.client.get(&url).send()?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().unwrap_or_default();
            return Err(Error::Api(format!(
                "Asset request failed with status {}: {}",
                status, body
            )));
        }

        let manifest: AssetManifest = response.json()?;

        // Find the best quality image (prefer ~orig, then ~large, then any jpg)
        let items = &manifest.collection.items;

        // Priority order: original > large > medium > any image
        let url = items
            .iter()
            .find(|i| i.href.contains("~orig."))
            .or_else(|| items.iter().find(|i| i.href.contains("~large.")))
            .or_else(|| items.iter().find(|i| i.href.contains("~medium.")))
            .or_else(|| {
                items.iter().find(|i| {
                    let href = i.href.to_lowercase();
                    href.ends_with(".jpg") || href.ends_with(".jpeg") || href.ends_with(".png")
                })
            })
            .map(|i| i.href.clone())
            .ok_or_else(|| Error::Api("No image found in asset manifest".to_string()))?;

        Ok(url)
    }

    /// Convert a NASA Image item to an ApodResponse for compatibility with existing cache
    pub fn item_to_apod_response(&self, item: &NasaImageItem) -> Option<ApodResponse> {
        let data = item.data.first()?;

        // Get preview URL from links (thumbnail)
        let preview_url = item.links.as_ref().and_then(|links| {
            links
                .iter()
                .find(|l| l.rel == "preview")
                .map(|l| l.href.clone())
        })?;

        // Try to get high-res URL
        let hdurl = self.get_asset_url(&data.nasa_id).ok();

        // Extract date (use date_created or default to today)
        let date = data
            .date_created
            .as_ref()
            .map(|d| {
                // NASA dates are in ISO format, extract just the date part
                d.split('T').next().unwrap_or(d).to_string()
            })
            .unwrap_or_else(|| chrono::Utc::now().format("%Y-%m-%d").to_string());

        Some(ApodResponse {
            date,
            title: data.title.clone(),
            explanation: data
                .description
                .clone()
                .unwrap_or_else(|| format!("NASA Image: {}", data.title)),
            url: preview_url,
            hdurl,
            media_type: "image".to_string(),
            copyright: data.center.clone().or(data.photographer.clone()),
            service_version: Some("nasa-images-v1".to_string()),
            thumbnail_url: None,
        })
    }

    /// Fetch random space images as ApodResponse objects for cache compatibility
    pub fn fetch_random_as_apod(&self, count: u32) -> Result<Vec<ApodResponse>> {
        let items = self.search_space_images(count)?;

        let apods: Vec<ApodResponse> = items
            .iter()
            .filter_map(|item| self.item_to_apod_response(item))
            .collect();

        if apods.is_empty() {
            return Err(Error::Api("No valid images found".to_string()));
        }

        Ok(apods)
    }
}

impl Default for NasaImagesApi {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nasa_images_response_parsing() {
        let json = r#"{
            "collection": {
                "items": [{
                    "href": "https://images-api.nasa.gov/asset/PIA12345",
                    "data": [{
                        "title": "Test Nebula",
                        "description": "A beautiful nebula",
                        "date_created": "2024-01-15T00:00:00Z",
                        "nasa_id": "PIA12345",
                        "media_type": "image",
                        "center": "JPL"
                    }],
                    "links": [{
                        "href": "https://images.nasa.gov/thumb/PIA12345.jpg",
                        "rel": "preview",
                        "render": "image"
                    }]
                }]
            }
        }"#;

        let response: NasaImagesResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.collection.items.len(), 1);
        assert_eq!(response.collection.items[0].data[0].title, "Test Nebula");
    }
}
