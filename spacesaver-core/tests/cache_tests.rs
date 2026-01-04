//! Cache-specific integration tests

use spacesaver_core::api::ApodResponse;
use spacesaver_core::cache::{CacheIndex, CachedImageMetadata};
use std::fs;
use tempfile::TempDir;

/// Helper to create a test APOD response
fn create_test_apod(date: &str, title: &str) -> ApodResponse {
    ApodResponse {
        date: date.to_string(),
        title: title.to_string(),
        explanation: format!("Explanation for {}", title),
        url: format!("https://example.com/{}.jpg", date),
        hdurl: Some(format!("https://example.com/{}_hd.jpg", date)),
        media_type: "image".to_string(),
        copyright: None,
        service_version: Some("v1".to_string()),
        thumbnail_url: None,
    }
}

/// Test cache index operations
#[test]
fn test_cache_index_operations() {
    let mut index = CacheIndex::default();

    // Initially empty
    assert!(index.images.is_empty());
    assert_eq!(index.last_updated, 0);

    // Add an entry
    index.images.insert(
        "2024-01-15".to_string(),
        CachedImageMetadata {
            date: "2024-01-15".to_string(),
            title: "Galaxy NGC 1234".to_string(),
            explanation: "A beautiful spiral galaxy".to_string(),
            copyright: Some("Hubble".to_string()),
            original_url: "https://apod.nasa.gov/galaxy.jpg".to_string(),
            filename: "apod_2024-01-15.jpg".to_string(),
            cached_at: 1705312800,
        },
    );

    assert_eq!(index.images.len(), 1);
    assert!(index.images.contains_key("2024-01-15"));

    // Retrieve entry
    let entry = index.images.get("2024-01-15").unwrap();
    assert_eq!(entry.title, "Galaxy NGC 1234");
    assert_eq!(entry.copyright, Some("Hubble".to_string()));
}

/// Test cache index persistence (serialization round-trip)
#[test]
fn test_cache_index_persistence() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let index_path = temp_dir.path().join("index.json");

    // Create and populate index
    let mut index = CacheIndex {
        last_updated: 1705312800,
        ..Default::default()
    };

    for i in 1..=5 {
        let date = format!("2024-01-{:02}", i);
        index.images.insert(
            date.clone(),
            CachedImageMetadata {
                date: date.clone(),
                title: format!("Image {}", i),
                explanation: format!("Explanation {}", i),
                copyright: if i % 2 == 0 {
                    Some("NASA".to_string())
                } else {
                    None
                },
                original_url: format!("https://example.com/{}.jpg", i),
                filename: format!("apod_{}.jpg", date),
                cached_at: 1705312800 + (i as i64 * 86400),
            },
        );
    }

    // Write to file
    let json = serde_json::to_string_pretty(&index).expect("Failed to serialize");
    fs::write(&index_path, &json).expect("Failed to write");

    // Read back
    let content = fs::read_to_string(&index_path).expect("Failed to read");
    let loaded: CacheIndex = serde_json::from_str(&content).expect("Failed to parse");

    // Verify
    assert_eq!(loaded.images.len(), 5);
    assert_eq!(loaded.last_updated, 1705312800);

    let entry = loaded.images.get("2024-01-03").unwrap();
    assert_eq!(entry.title, "Image 3");
    assert_eq!(entry.copyright, None);
}

/// Test cache metadata with various copyright scenarios
#[test]
fn test_cached_image_metadata_copyright() {
    let with_copyright = CachedImageMetadata {
        date: "2024-01-15".to_string(),
        title: "Test".to_string(),
        explanation: "Test".to_string(),
        copyright: Some("John Doe Photography".to_string()),
        original_url: "https://example.com/test.jpg".to_string(),
        filename: "test.jpg".to_string(),
        cached_at: 0,
    };

    let without_copyright = CachedImageMetadata {
        date: "2024-01-16".to_string(),
        title: "Test 2".to_string(),
        explanation: "Test 2".to_string(),
        copyright: None,
        original_url: "https://example.com/test2.jpg".to_string(),
        filename: "test2.jpg".to_string(),
        cached_at: 0,
    };

    assert!(with_copyright.copyright.is_some());
    assert!(without_copyright.copyright.is_none());
}

/// Test cache with file operations in temp directory
#[test]
fn test_cache_file_operations() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let cache_dir = temp_dir.path();

    // Simulate storing an image
    let apod = create_test_apod("2024-01-15", "Nebula Test");
    let filename = format!("apod_{}.jpg", apod.date);
    let file_path = cache_dir.join(&filename);

    // Write fake image data
    let fake_image_data = b"fake jpeg data here";
    fs::write(&file_path, fake_image_data).expect("Failed to write image");

    // Verify file exists
    assert!(file_path.exists());
    assert_eq!(
        fs::read(&file_path).expect("Failed to read"),
        fake_image_data
    );

    // Update cache index
    let mut index = CacheIndex::default();
    index.images.insert(
        apod.date.clone(),
        CachedImageMetadata {
            date: apod.date.clone(),
            title: apod.title.clone(),
            explanation: apod.explanation.clone(),
            copyright: apod.copyright.clone(),
            original_url: apod.url.clone(),
            filename,
            cached_at: chrono::Utc::now().timestamp(),
        },
    );

    // Write index
    let index_path = cache_dir.join("index.json");
    let json = serde_json::to_string_pretty(&index).expect("Failed to serialize");
    fs::write(&index_path, &json).expect("Failed to write index");

    // Read and verify index
    let content = fs::read_to_string(&index_path).expect("Failed to read");
    let loaded: CacheIndex = serde_json::from_str(&content).expect("Failed to parse");

    assert!(loaded.images.contains_key("2024-01-15"));
}

/// Test cache size limit enforcement logic
#[test]
fn test_cache_size_limit_logic() {
    // Simulate enforcing size limit
    let max_size = 3;
    let mut entries: Vec<(String, i64)> = vec![
        ("2024-01-01".to_string(), 1000),
        ("2024-01-02".to_string(), 2000),
        ("2024-01-03".to_string(), 3000),
        ("2024-01-04".to_string(), 4000),
        ("2024-01-05".to_string(), 5000),
    ];

    // Sort by cached_at (oldest first)
    entries.sort_by_key(|(_, cached_at)| *cached_at);

    // Remove oldest until under limit
    while entries.len() > max_size {
        let removed = entries.remove(0);
        assert!(removed.1 < entries.iter().map(|(_, t)| *t).min().unwrap_or(i64::MAX));
    }

    assert_eq!(entries.len(), max_size);
    // Newest should remain
    assert!(entries.iter().any(|(d, _)| d == "2024-01-05"));
    assert!(entries.iter().any(|(d, _)| d == "2024-01-04"));
    assert!(entries.iter().any(|(d, _)| d == "2024-01-03"));
}

/// Test random image selection logic
#[test]
fn test_random_selection_distribution() {
    use rand::seq::SliceRandom;
    use std::collections::HashSet;

    let dates: Vec<String> = (1..=10).map(|i| format!("2024-01-{:02}", i)).collect();

    let mut selected: HashSet<String> = HashSet::new();
    let mut rng = rand::thread_rng();

    // Select multiple times to verify randomness
    for _ in 0..100 {
        if let Some(date) = dates.choose(&mut rng) {
            selected.insert(date.clone());
        }
    }

    // With 100 selections from 10 items, we should hit most of them
    assert!(selected.len() >= 5, "Random selection seems broken");
}
