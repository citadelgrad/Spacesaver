//! Integration tests for spacesaver-core
//!
//! These tests verify the library's core functionality works correctly.

use spacesaver_core::{Config, NasaApodApi};

/// Test that the default configuration is valid
#[test]
fn test_default_config() {
    let config = Config::default();

    assert_eq!(config.api_key, "DEMO_KEY");
    assert!(config.cache_size > 0);
    assert!(config.transition_interval > 0.0);
    assert!(config.transition_duration > 0.0);
    assert!(config.max_history_days > 0);
}

/// Test configuration serialization/deserialization
#[test]
fn test_config_serialization() {
    let config = Config::default();

    // Serialize to JSON
    let json = serde_json::to_string(&config).expect("Failed to serialize config");

    // Deserialize back
    let deserialized: Config = serde_json::from_str(&json).expect("Failed to deserialize config");

    assert_eq!(config.api_key, deserialized.api_key);
    assert_eq!(config.cache_size, deserialized.cache_size);
    assert_eq!(config.transition_interval, deserialized.transition_interval);
}

/// Test API client creation
#[test]
fn test_api_client_creation() {
    let api = NasaApodApi::new("DEMO_KEY");
    // Just verify it doesn't panic
    drop(api);
}

/// Test API response parsing
#[test]
fn test_apod_response_parsing() {
    use spacesaver_core::api::ApodResponse;

    let json = r#"{
        "date": "2024-01-15",
        "title": "The Orion Nebula in Infrared",
        "explanation": "The Great Nebula in Orion is a gorgeous stellar nursery.",
        "url": "https://apod.nasa.gov/apod/image/2401/OrionNebula.jpg",
        "hdurl": "https://apod.nasa.gov/apod/image/2401/OrionNebula_hd.jpg",
        "media_type": "image",
        "service_version": "v1"
    }"#;

    let apod: ApodResponse = serde_json::from_str(json).expect("Failed to parse APOD response");

    assert_eq!(apod.date, "2024-01-15");
    assert_eq!(apod.title, "The Orion Nebula in Infrared");
    assert!(apod.is_image());
    assert_eq!(
        apod.best_image_url(),
        Some("https://apod.nasa.gov/apod/image/2401/OrionNebula_hd.jpg")
    );
}

/// Test video APOD response parsing
#[test]
fn test_video_apod_response() {
    use spacesaver_core::api::ApodResponse;

    let json = r#"{
        "date": "2024-01-16",
        "title": "Perseverance Rover Video",
        "explanation": "A video from Mars.",
        "url": "https://www.youtube.com/embed/xyz123",
        "media_type": "video",
        "thumbnail_url": "https://img.youtube.com/vi/xyz123/0.jpg"
    }"#;

    let apod: ApodResponse = serde_json::from_str(json).expect("Failed to parse video APOD");

    assert!(!apod.is_image());
    assert_eq!(
        apod.best_image_url(),
        Some("https://img.youtube.com/vi/xyz123/0.jpg")
    );
}

/// Test APOD response without HD URL
#[test]
fn test_apod_response_no_hd() {
    use spacesaver_core::api::ApodResponse;

    let json = r#"{
        "date": "2024-01-17",
        "title": "Simple Image",
        "explanation": "Just a simple image.",
        "url": "https://apod.nasa.gov/apod/image/2401/simple.jpg",
        "media_type": "image"
    }"#;

    let apod: ApodResponse = serde_json::from_str(json).expect("Failed to parse APOD");

    assert_eq!(
        apod.best_image_url(),
        Some("https://apod.nasa.gov/apod/image/2401/simple.jpg")
    );
}

/// Test APOD response with copyright
#[test]
fn test_apod_response_with_copyright() {
    use spacesaver_core::api::ApodResponse;

    let json = r#"{
        "date": "2024-01-18",
        "title": "Copyrighted Image",
        "explanation": "An image with copyright.",
        "url": "https://example.com/image.jpg",
        "media_type": "image",
        "copyright": "John Doe"
    }"#;

    let apod: ApodResponse = serde_json::from_str(json).expect("Failed to parse APOD");

    assert_eq!(apod.copyright, Some("John Doe".to_string()));
}

mod cache_tests {
    use spacesaver_core::cache::{CacheIndex, CachedImageMetadata};

    /// Test cache index serialization
    #[test]
    fn test_cache_index_serialization() {
        let mut index = CacheIndex::default();
        index.images.insert(
            "2024-01-15".to_string(),
            CachedImageMetadata {
                date: "2024-01-15".to_string(),
                title: "Test".to_string(),
                explanation: "Test explanation".to_string(),
                copyright: None,
                original_url: "https://example.com/test.jpg".to_string(),
                filename: "apod_2024-01-15.jpg".to_string(),
                cached_at: 1705312800,
            },
        );

        let json = serde_json::to_string(&index).expect("Failed to serialize index");
        let deserialized: CacheIndex =
            serde_json::from_str(&json).expect("Failed to deserialize index");

        assert_eq!(index.images.len(), deserialized.images.len());
        assert!(deserialized.images.contains_key("2024-01-15"));
    }

    /// Test cached image metadata
    #[test]
    fn test_cached_image_metadata() {
        let metadata = CachedImageMetadata {
            date: "2024-01-15".to_string(),
            title: "Nebula".to_string(),
            explanation: "A beautiful nebula".to_string(),
            copyright: Some("NASA".to_string()),
            original_url: "https://apod.nasa.gov/image.jpg".to_string(),
            filename: "apod_2024-01-15.jpg".to_string(),
            cached_at: chrono::Utc::now().timestamp(),
        };

        assert_eq!(metadata.date, "2024-01-15");
        assert_eq!(metadata.copyright, Some("NASA".to_string()));
    }
}

mod ffi_tests {
    use std::ffi::CString;
    use std::ptr;

    /// Test FFI initialization and shutdown
    #[test]
    fn test_ffi_init_shutdown() {
        unsafe {
            // Initialize with null API key (uses default)
            let result = spacesaver_core::spacesaver_init(ptr::null());
            assert!(result.success);

            // Check cached count
            let count = spacesaver_core::spacesaver_cached_count();
            assert!(count >= 0);

            // Check is_fetching
            let fetching = spacesaver_core::spacesaver_is_fetching();
            // Should not be fetching immediately after init
            assert!(!fetching);

            // Shutdown
            spacesaver_core::spacesaver_shutdown();
        }
    }

    /// Test FFI with custom API key
    #[test]
    fn test_ffi_custom_api_key() {
        unsafe {
            let api_key = CString::new("TEST_KEY").expect("CString failed");
            let result = spacesaver_core::spacesaver_init(api_key.as_ptr());
            assert!(result.success);

            spacesaver_core::spacesaver_shutdown();
        }
    }

    /// Test FFI version function
    #[test]
    fn test_ffi_version() {
        unsafe {
            let version_ptr = spacesaver_core::spacesaver_version();
            assert!(!version_ptr.is_null());

            let version = std::ffi::CStr::from_ptr(version_ptr)
                .to_str()
                .expect("Invalid version string");
            assert!(!version.is_empty());

            spacesaver_core::spacesaver_free_string(version_ptr);
        }
    }

    /// Test FFI result error handling
    #[test]
    fn test_ffi_result_handling() {
        unsafe {
            // Initialize first
            let result = spacesaver_core::spacesaver_init(ptr::null());
            assert!(result.success);

            // Try to get an image when none are cached
            let image = spacesaver_core::spacesaver_next_image();
            // Path should be null if no images
            // (This is expected behavior for empty cache)

            // Clean up
            if !image.path.is_null() {
                let mut img = image;
                spacesaver_core::spacesaver_free_image(&mut img);
            }

            spacesaver_core::spacesaver_shutdown();
        }
    }
}

mod error_tests {
    use spacesaver_core::error::Error;

    #[test]
    fn test_error_display() {
        let err = Error::NoImages;
        let msg = format!("{}", err);
        assert!(msg.contains("No images"));
    }

    #[test]
    fn test_error_api() {
        let err = Error::Api("Rate limit exceeded".to_string());
        let msg = format!("{}", err);
        assert!(msg.contains("Rate limit"));
    }

    #[test]
    fn test_error_cache() {
        let err = Error::Cache("Disk full".to_string());
        let msg = format!("{}", err);
        assert!(msg.contains("Disk full"));
    }
}
