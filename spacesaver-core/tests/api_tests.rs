//! API response parsing and validation tests

use spacesaver_core::api::ApodResponse;

/// Test parsing a complete APOD response with all fields
#[test]
fn test_complete_apod_response() {
    let json = r#"{
        "date": "2024-06-15",
        "title": "The Milky Way over Uluru",
        "explanation": "The central band of the Milky Way rises above Uluru, also known as Ayers Rock, in this night sky panorama. The reddish desert landscape surrounding the famous sandstone formation appears to glow in the light of a setting crescent Moon. Captured on 2024 May 29, the Milky Way's ancient starlight mingles with light from modern civilization visible as bright cities on the horizon. The dark lanes of interstellar dust and the star clouds of our galaxy's central bulge can be traced through Sagittarius and Scorpius toward the zenith.",
        "url": "https://apod.nasa.gov/apod/image/2406/MilkyWayUluru_small.jpg",
        "hdurl": "https://apod.nasa.gov/apod/image/2406/MilkyWayUluru.jpg",
        "media_type": "image",
        "copyright": "John Smith",
        "service_version": "v1"
    }"#;

    let apod: ApodResponse = serde_json::from_str(json).expect("Failed to parse");

    assert_eq!(apod.date, "2024-06-15");
    assert_eq!(apod.title, "The Milky Way over Uluru");
    assert!(apod.explanation.contains("Milky Way"));
    assert!(apod.is_image());
    assert_eq!(apod.copyright, Some("John Smith".to_string()));
    assert_eq!(
        apod.best_image_url(),
        Some("https://apod.nasa.gov/apod/image/2406/MilkyWayUluru.jpg")
    );
}

/// Test parsing minimal APOD response (required fields only)
#[test]
fn test_minimal_apod_response() {
    let json = r#"{
        "date": "2024-01-01",
        "title": "New Year Image",
        "explanation": "Happy New Year!",
        "url": "https://example.com/newyear.jpg",
        "media_type": "image"
    }"#;

    let apod: ApodResponse = serde_json::from_str(json).expect("Failed to parse");

    assert_eq!(apod.date, "2024-01-01");
    assert!(apod.hdurl.is_none());
    assert!(apod.copyright.is_none());
    assert!(apod.thumbnail_url.is_none());
    assert_eq!(apod.best_image_url(), Some("https://example.com/newyear.jpg"));
}

/// Test YouTube video APOD
#[test]
fn test_youtube_video_apod() {
    let json = r#"{
        "date": "2024-02-14",
        "title": "Valentine's Day Aurora",
        "explanation": "A beautiful aurora display.",
        "url": "https://www.youtube.com/embed/dQw4w9WgXcQ",
        "media_type": "video",
        "thumbnail_url": "https://img.youtube.com/vi/dQw4w9WgXcQ/0.jpg"
    }"#;

    let apod: ApodResponse = serde_json::from_str(json).expect("Failed to parse");

    assert!(!apod.is_image());
    assert_eq!(apod.media_type, "video");
    assert_eq!(
        apod.best_image_url(),
        Some("https://img.youtube.com/vi/dQw4w9WgXcQ/0.jpg")
    );
}

/// Test Vimeo video APOD without thumbnail
#[test]
fn test_vimeo_video_no_thumbnail() {
    let json = r#"{
        "date": "2024-03-01",
        "title": "Space Station Timelapse",
        "explanation": "ISS timelapse video.",
        "url": "https://player.vimeo.com/video/123456",
        "media_type": "video"
    }"#;

    let apod: ApodResponse = serde_json::from_str(json).expect("Failed to parse");

    assert!(!apod.is_image());
    // No thumbnail available for this video
    assert_eq!(apod.best_image_url(), None);
}

/// Test APOD with special characters in title and explanation
#[test]
fn test_special_characters() {
    let json = r#"{
        "date": "2024-04-01",
        "title": "M31: The \"Great\" Andromeda Galaxy",
        "explanation": "Andromeda is ~2.5 million light-years away. It's visible to the naked eye & contains over 1 trillion stars!",
        "url": "https://example.com/m31.jpg",
        "media_type": "image"
    }"#;

    let apod: ApodResponse = serde_json::from_str(json).expect("Failed to parse");

    assert!(apod.title.contains("\"Great\""));
    assert!(apod.explanation.contains("~2.5 million"));
    assert!(apod.explanation.contains("&"));
}

/// Test APOD with Unicode characters
#[test]
fn test_unicode_characters() {
    let json = r#"{
        "date": "2024-05-01",
        "title": "星空 - Starry Night",
        "explanation": "A beautiful view of the stars from Japan. 美しい星空。",
        "url": "https://example.com/stars.jpg",
        "media_type": "image",
        "copyright": "田中太郎"
    }"#;

    let apod: ApodResponse = serde_json::from_str(json).expect("Failed to parse");

    assert!(apod.title.contains("星空"));
    assert!(apod.explanation.contains("美しい"));
    assert_eq!(apod.copyright, Some("田中太郎".to_string()));
}

/// Test date parsing edge cases
#[test]
fn test_date_formats() {
    // Standard format
    let json = r#"{
        "date": "1995-06-16",
        "title": "First APOD",
        "explanation": "The very first Astronomy Picture of the Day.",
        "url": "https://apod.nasa.gov/apod/ap950616.html",
        "media_type": "image"
    }"#;

    let apod: ApodResponse = serde_json::from_str(json).expect("Failed to parse");
    assert_eq!(apod.date, "1995-06-16");
}

/// Test long explanation text
#[test]
fn test_long_explanation() {
    let long_explanation = "A".repeat(5000);
    let json = format!(
        r#"{{
        "date": "2024-01-01",
        "title": "Test",
        "explanation": "{}",
        "url": "https://example.com/test.jpg",
        "media_type": "image"
    }}"#,
        long_explanation
    );

    let apod: ApodResponse = serde_json::from_str(&json).expect("Failed to parse");
    assert_eq!(apod.explanation.len(), 5000);
}

/// Test image URL extraction with query parameters
#[test]
fn test_url_with_query_params() {
    let json = r#"{
        "date": "2024-01-01",
        "title": "Test",
        "explanation": "Test",
        "url": "https://example.com/image.jpg?width=1024&height=768",
        "hdurl": "https://example.com/image.jpg?width=4096&height=3072",
        "media_type": "image"
    }"#;

    let apod: ApodResponse = serde_json::from_str(json).expect("Failed to parse");

    let best_url = apod.best_image_url().expect("Should have URL");
    assert!(best_url.contains("width=4096"));
}

/// Test serialization round-trip
#[test]
fn test_serialization_roundtrip() {
    let original = ApodResponse {
        date: "2024-01-15".to_string(),
        title: "Test Image".to_string(),
        explanation: "A test explanation.".to_string(),
        url: "https://example.com/test.jpg".to_string(),
        hdurl: Some("https://example.com/test_hd.jpg".to_string()),
        media_type: "image".to_string(),
        copyright: Some("Test Author".to_string()),
        service_version: Some("v1".to_string()),
        thumbnail_url: None,
    };

    // Serialize
    let json = serde_json::to_string(&original).expect("Failed to serialize");

    // Deserialize
    let restored: ApodResponse = serde_json::from_str(&json).expect("Failed to deserialize");

    // Verify
    assert_eq!(original.date, restored.date);
    assert_eq!(original.title, restored.title);
    assert_eq!(original.hdurl, restored.hdurl);
    assert_eq!(original.copyright, restored.copyright);
}

/// Test multiple APOD responses (for range/random endpoints)
#[test]
fn test_multiple_apod_responses() {
    let json = r#"[
        {
            "date": "2024-01-01",
            "title": "Image 1",
            "explanation": "First",
            "url": "https://example.com/1.jpg",
            "media_type": "image"
        },
        {
            "date": "2024-01-02",
            "title": "Image 2",
            "explanation": "Second",
            "url": "https://example.com/2.jpg",
            "media_type": "image"
        },
        {
            "date": "2024-01-03",
            "title": "Video 1",
            "explanation": "Third",
            "url": "https://youtube.com/v1",
            "media_type": "video",
            "thumbnail_url": "https://img.youtube.com/1.jpg"
        }
    ]"#;

    let apods: Vec<ApodResponse> = serde_json::from_str(json).expect("Failed to parse");

    assert_eq!(apods.len(), 3);
    assert!(apods[0].is_image());
    assert!(apods[1].is_image());
    assert!(!apods[2].is_image());

    // Count images only
    let image_count = apods.iter().filter(|a| a.is_image()).count();
    assert_eq!(image_count, 2);
}
