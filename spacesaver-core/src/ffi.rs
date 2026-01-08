//! FFI (Foreign Function Interface) exports for Swift/Objective-C integration
//!
//! This module exposes C-compatible functions that can be called from Swift.

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;
use std::sync::Arc;

use once_cell::sync::OnceCell;
use parking_lot::RwLock;

use crate::config::Config;
use crate::image_manager::ImageManager;
use crate::logging;

/// Global image manager instance
static IMAGE_MANAGER: OnceCell<Arc<RwLock<Option<ImageManager>>>> = OnceCell::new();

/// Bundle resources path for fallback images
static BUNDLE_RESOURCES_PATH: OnceCell<RwLock<Option<std::path::PathBuf>>> = OnceCell::new();

fn get_manager() -> &'static Arc<RwLock<Option<ImageManager>>> {
    IMAGE_MANAGER.get_or_init(|| Arc::new(RwLock::new(None)))
}

fn get_bundle_path() -> &'static RwLock<Option<std::path::PathBuf>> {
    BUNDLE_RESOURCES_PATH.get_or_init(|| RwLock::new(None))
}

/// Image information returned to Swift
#[repr(C)]
pub struct SpacesaverImage {
    /// Path to the image file (owned, must be freed)
    pub path: *mut c_char,
    /// Image title (owned, must be freed)
    pub title: *mut c_char,
    /// Image description (owned, must be freed)
    pub description: *mut c_char,
    /// Image date (owned, must be freed)
    pub date: *mut c_char,
    /// Copyright info if any (owned, must be freed, may be null)
    pub copyright: *mut c_char,
}

impl SpacesaverImage {
    fn null() -> Self {
        Self {
            path: ptr::null_mut(),
            title: ptr::null_mut(),
            description: ptr::null_mut(),
            date: ptr::null_mut(),
            copyright: ptr::null_mut(),
        }
    }
}

/// Result type for FFI operations
#[repr(C)]
pub struct SpacesaverResult {
    /// Success flag
    pub success: bool,
    /// Error message if failed (owned, must be freed)
    pub error: *mut c_char,
}

impl SpacesaverResult {
    fn ok() -> Self {
        Self {
            success: true,
            error: ptr::null_mut(),
        }
    }

    fn err(msg: &str) -> Self {
        Self {
            success: false,
            error: CString::new(msg)
                .map(|s| s.into_raw())
                .unwrap_or(ptr::null_mut()),
        }
    }
}

/// Helper to convert Rust string to C string
fn to_c_string(s: &str) -> *mut c_char {
    CString::new(s)
        .map(|s| s.into_raw())
        .unwrap_or(ptr::null_mut())
}

/// Helper to convert Option<String> to C string
fn option_to_c_string(s: Option<&str>) -> *mut c_char {
    s.map(to_c_string).unwrap_or(ptr::null_mut())
}

// ============================================================================
// FFI Functions
// ============================================================================

/// Initialize the spacesaver library with optional API key
/// If api_key is null, uses DEMO_KEY
///
/// # Safety
/// api_key must be a valid null-terminated string or null
#[no_mangle]
pub unsafe extern "C" fn spacesaver_init(api_key: *const c_char) -> SpacesaverResult {
    // Initialize file-based logging (works in screen saver context)
    logging::init_logging();
    logging::log_info("spacesaver_init called");

    let mut config = match Config::load() {
        Ok(c) => {
            logging::log_info(&format!(
                "Config loaded, API key: {}...",
                &c.api_key[..8.min(c.api_key.len())]
            ));
            c
        }
        Err(e) => {
            logging::log_warn(&format!("Failed to load config, using defaults: {}", e));
            Config::default()
        }
    };

    // Override API key if provided
    if !api_key.is_null() {
        if let Ok(key) = CStr::from_ptr(api_key).to_str() {
            if !key.is_empty() {
                config.set_api_key(key.to_string());
            }
        }
    }

    match ImageManager::new(config) {
        Ok(manager) => {
            let cached = manager.cached_count();
            logging::log_info(&format!(
                "ImageManager initialized, {} cached images",
                cached
            ));
            let mut guard = get_manager().write();
            *guard = Some(manager);
            SpacesaverResult::ok()
        }
        Err(e) => {
            logging::log_error(&format!("Failed to initialize ImageManager: {}", e));
            SpacesaverResult::err(&format!("Failed to initialize: {}", e))
        }
    }
}

/// Shutdown the spacesaver library and free resources
#[no_mangle]
pub extern "C" fn spacesaver_shutdown() {
    let mut guard = get_manager().write();
    *guard = None;
}

/// Get the number of cached images
#[no_mangle]
pub extern "C" fn spacesaver_cached_count() -> i32 {
    let guard = get_manager().read();
    guard.as_ref().map(|m| m.cached_count() as i32).unwrap_or(0)
}

/// Check if currently fetching images
#[no_mangle]
pub extern "C" fn spacesaver_is_fetching() -> bool {
    let guard = get_manager().read();
    guard.as_ref().map(|m| m.is_fetching()).unwrap_or(false)
}

/// Fetch random images to populate the cache
/// Returns the number of images fetched, or -1 on error
#[no_mangle]
pub extern "C" fn spacesaver_fetch_random(count: i32) -> i32 {
    logging::log_info(&format!(
        "spacesaver_fetch_random called with count={}",
        count
    ));
    let guard = get_manager().read();
    if let Some(manager) = guard.as_ref() {
        match manager.fetch_random_images(count as u32) {
            Ok(paths) => {
                logging::log_info(&format!("Successfully fetched {} images", paths.len()));
                paths.len() as i32
            }
            Err(e) => {
                logging::log_error(&format!("Failed to fetch random images: {}", e));
                -1
            }
        }
    } else {
        logging::log_error("spacesaver_fetch_random: manager not initialized");
        -1
    }
}

/// Fetch recent images (last N days)
/// Returns the number of images fetched, or -1 on error
#[no_mangle]
pub extern "C" fn spacesaver_fetch_recent(days: i32) -> i32 {
    logging::log_info(&format!(
        "spacesaver_fetch_recent called with days={}",
        days
    ));
    let guard = get_manager().read();
    if let Some(manager) = guard.as_ref() {
        match manager.fetch_recent_images(days as u32) {
            Ok(paths) => {
                logging::log_info(&format!(
                    "Successfully fetched {} recent images",
                    paths.len()
                ));
                paths.len() as i32
            }
            Err(e) => {
                logging::log_error(&format!("Failed to fetch recent images: {}", e));
                -1
            }
        }
    } else {
        logging::log_error("spacesaver_fetch_recent: manager not initialized");
        -1
    }
}

/// Start background prefetching of images
#[no_mangle]
pub extern "C" fn spacesaver_start_prefetch(count: i32) {
    let guard = get_manager().read();
    if let Some(manager) = guard.as_ref() {
        manager.start_prefetch(count as u32);
    }
}

/// Get the next image in rotation
/// Returns a SpacesaverImage struct. Caller must free strings with spacesaver_free_string.
#[no_mangle]
pub extern "C" fn spacesaver_next_image() -> SpacesaverImage {
    let guard = get_manager().read();
    if let Some(manager) = guard.as_ref() {
        if let Some(img) = manager.next_image() {
            return SpacesaverImage {
                path: to_c_string(img.path.to_string_lossy().as_ref()),
                title: to_c_string(&img.title),
                description: to_c_string(&img.explanation),
                date: to_c_string(&img.date),
                copyright: option_to_c_string(img.copyright.as_deref()),
            };
        }
    }
    SpacesaverImage::null()
}

/// Get a random image
/// Returns a SpacesaverImage struct. Caller must free strings with spacesaver_free_string.
#[no_mangle]
pub extern "C" fn spacesaver_random_image() -> SpacesaverImage {
    let guard = get_manager().read();
    if let Some(manager) = guard.as_ref() {
        if let Some(img) = manager.random_image() {
            return SpacesaverImage {
                path: to_c_string(img.path.to_string_lossy().as_ref()),
                title: to_c_string(&img.title),
                description: to_c_string(&img.explanation),
                date: to_c_string(&img.date),
                copyright: option_to_c_string(img.copyright.as_deref()),
            };
        }
    }
    SpacesaverImage::null()
}

/// Get the current image (without advancing)
/// Returns a SpacesaverImage struct. Caller must free strings with spacesaver_free_string.
#[no_mangle]
pub extern "C" fn spacesaver_current_image() -> SpacesaverImage {
    let guard = get_manager().read();
    if let Some(manager) = guard.as_ref() {
        if let Some(img) = manager.current_image() {
            return SpacesaverImage {
                path: to_c_string(img.path.to_string_lossy().as_ref()),
                title: to_c_string(&img.title),
                description: to_c_string(&img.explanation),
                date: to_c_string(&img.date),
                copyright: option_to_c_string(img.copyright.as_deref()),
            };
        }
    }
    SpacesaverImage::null()
}

/// Reshuffle the image order
#[no_mangle]
pub extern "C" fn spacesaver_reshuffle() {
    let guard = get_manager().read();
    if let Some(manager) = guard.as_ref() {
        manager.reshuffle_dates();
    }
}

/// Clear the image cache
#[no_mangle]
pub extern "C" fn spacesaver_clear_cache() -> SpacesaverResult {
    let guard = get_manager().read();
    if let Some(manager) = guard.as_ref() {
        match manager.clear_cache() {
            Ok(_) => SpacesaverResult::ok(),
            Err(e) => SpacesaverResult::err(&format!("Failed to clear cache: {}", e)),
        }
    } else {
        SpacesaverResult::err("Not initialized")
    }
}

/// Free a C string allocated by this library
///
/// # Safety
/// The pointer must have been allocated by this library
#[no_mangle]
pub unsafe extern "C" fn spacesaver_free_string(s: *mut c_char) {
    if !s.is_null() {
        drop(CString::from_raw(s));
    }
}

/// Free a SpacesaverImage struct
///
/// # Safety
/// The struct must have been returned by this library
#[no_mangle]
pub unsafe extern "C" fn spacesaver_free_image(img: *mut SpacesaverImage) {
    if img.is_null() {
        return;
    }

    let img = &mut *img;
    spacesaver_free_string(img.path);
    spacesaver_free_string(img.title);
    spacesaver_free_string(img.description);
    spacesaver_free_string(img.date);
    spacesaver_free_string(img.copyright);

    img.path = ptr::null_mut();
    img.title = ptr::null_mut();
    img.description = ptr::null_mut();
    img.date = ptr::null_mut();
    img.copyright = ptr::null_mut();
}

/// Free a SpacesaverResult struct (just the error string if present)
///
/// # Safety
/// The struct must have been returned by this library
#[no_mangle]
pub unsafe extern "C" fn spacesaver_free_result(result: *mut SpacesaverResult) {
    if result.is_null() {
        return;
    }

    let result = &mut *result;
    spacesaver_free_string(result.error);
    result.error = ptr::null_mut();
}

/// Get the cache directory path
/// Returns a C string that must be freed with spacesaver_free_string
#[no_mangle]
pub extern "C" fn spacesaver_get_cache_dir() -> *mut c_char {
    match Config::cache_dir() {
        Ok(path) => to_c_string(path.to_string_lossy().as_ref()),
        Err(_) => ptr::null_mut(),
    }
}

/// Get the config directory path
/// Returns a C string that must be freed with spacesaver_free_string
#[no_mangle]
pub extern "C" fn spacesaver_get_config_dir() -> *mut c_char {
    match Config::config_dir() {
        Ok(path) => to_c_string(path.to_string_lossy().as_ref()),
        Err(_) => ptr::null_mut(),
    }
}

/// Set the API key for NASA API
///
/// # Safety
/// api_key must be a valid null-terminated string
#[no_mangle]
pub unsafe extern "C" fn spacesaver_set_api_key(api_key: *const c_char) -> SpacesaverResult {
    if api_key.is_null() {
        return SpacesaverResult::err("API key is null");
    }

    let key = match CStr::from_ptr(api_key).to_str() {
        Ok(s) => s.to_string(),
        Err(_) => return SpacesaverResult::err("Invalid UTF-8 in API key"),
    };

    let mut config = match Config::load() {
        Ok(c) => c,
        Err(e) => return SpacesaverResult::err(&format!("Failed to load config: {}", e)),
    };

    config.set_api_key(key);

    match config.save() {
        Ok(_) => SpacesaverResult::ok(),
        Err(e) => SpacesaverResult::err(&format!("Failed to save config: {}", e)),
    }
}

/// Get library version
#[no_mangle]
pub extern "C" fn spacesaver_version() -> *mut c_char {
    to_c_string(env!("CARGO_PKG_VERSION"))
}

/// Set the bundle resources path for bundled fallback images
///
/// # Safety
/// path must be a valid null-terminated string
#[no_mangle]
pub unsafe extern "C" fn spacesaver_set_bundle_path(path: *const c_char) -> SpacesaverResult {
    if path.is_null() {
        return SpacesaverResult::err("Bundle path is null");
    }

    let path_str = match CStr::from_ptr(path).to_str() {
        Ok(s) => s,
        Err(_) => return SpacesaverResult::err("Invalid UTF-8 in bundle path"),
    };

    let path_buf = std::path::PathBuf::from(path_str);
    logging::log_info(&format!("Bundle resources path set to: {:?}", path_buf));

    let mut guard = get_bundle_path().write();
    *guard = Some(path_buf);

    SpacesaverResult::ok()
}

/// Load bundled fallback images into the cache
/// Returns the number of images loaded, or -1 on error
#[no_mangle]
pub extern "C" fn spacesaver_load_bundled_images() -> i32 {
    let bundle_path = {
        let guard = get_bundle_path().read();
        match guard.as_ref() {
            Some(p) => p.clone(),
            None => {
                logging::log_warn("No bundle path set, cannot load bundled images");
                return -1;
            }
        }
    };

    let bundled_images_dir = bundle_path.join("BundledImages");
    let metadata_path = bundled_images_dir.join("metadata.json");

    if !metadata_path.exists() {
        logging::log_warn(&format!("Bundled images metadata not found at {:?}", metadata_path));
        return -1;
    }

    // Read metadata
    let metadata_content = match std::fs::read_to_string(&metadata_path) {
        Ok(c) => c,
        Err(e) => {
            logging::log_error(&format!("Failed to read bundled metadata: {}", e));
            return -1;
        }
    };

    #[derive(serde::Deserialize)]
    struct BundledImage {
        filename: String,
        title: String,
        date: String,
        copyright: Option<String>,
    }

    let images: Vec<BundledImage> = match serde_json::from_str(&metadata_content) {
        Ok(i) => i,
        Err(e) => {
            logging::log_error(&format!("Failed to parse bundled metadata: {}", e));
            return -1;
        }
    };

    let guard = get_manager().read();
    let manager = match guard.as_ref() {
        Some(m) => m,
        None => {
            logging::log_error("Manager not initialized, cannot load bundled images");
            return -1;
        }
    };

    let mut loaded = 0;
    for img in images {
        let image_path = bundled_images_dir.join(&img.filename);
        if image_path.exists() {
            match std::fs::read(&image_path) {
                Ok(data) => {
                    let apod = crate::api::ApodResponse {
                        date: img.date.clone(),
                        title: img.title.clone(),
                        explanation: format!("NASA Image: {}", img.title),
                        media_type: "image".to_string(),
                        url: String::new(),
                        hdurl: None,
                        copyright: img.copyright,
                        service_version: None,
                        thumbnail_url: None,
                    };

                    if manager.store_bundled_image(&apod, &data).is_ok() {
                        logging::log_info(&format!("Loaded bundled image: {}", img.title));
                        loaded += 1;
                    }
                }
                Err(e) => {
                    logging::log_warn(&format!("Failed to read bundled image {}: {}", img.filename, e));
                }
            }
        } else {
            logging::log_warn(&format!("Bundled image not found: {:?}", image_path));
        }
    }

    logging::log_info(&format!("Loaded {} bundled images as fallback", loaded));
    loaded
}

/// Get the log file path
/// Returns a C string that must be freed with spacesaver_free_string
#[no_mangle]
pub extern "C" fn spacesaver_get_log_path() -> *mut c_char {
    match logging::log_file_path() {
        Some(path) => to_c_string(path.to_string_lossy().as_ref()),
        None => ptr::null_mut(),
    }
}
