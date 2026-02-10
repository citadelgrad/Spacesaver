//! Integration tests for spacesaver-core
//!
//! These tests verify the library's core functionality works correctly.

use spacesaver_core::{Config, NasaApodApi};

/// Test that the default configuration is valid
#[test]
fn test_default_config() {
    let config = Config::default();

    assert_eq!(config.api_key, "DEMO_KEY");
    assert_eq!(config.cache_size, 50);
    assert_eq!(config.transition_interval, 30.0);
    assert_eq!(config.transition_duration, 2.0);
    assert_eq!(config.max_history_days, 365);
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
