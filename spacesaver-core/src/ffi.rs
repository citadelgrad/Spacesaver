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

/// Global image manager instance
static IMAGE_MANAGER: OnceCell<Arc<RwLock<Option<ImageManager>>>> = OnceCell::new();

fn get_manager() -> &'static Arc<RwLock<Option<ImageManager>>> {
    IMAGE_MANAGER.get_or_init(|| Arc::new(RwLock::new(None)))
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
    // Initialize logging
    let _ = env_logger::try_init();

    let mut config = match Config::load() {
        Ok(c) => c,
        Err(e) => {
            log::warn!("Failed to load config, using defaults: {}", e);
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
            let mut guard = get_manager().write();
            *guard = Some(manager);
            SpacesaverResult::ok()
        }
        Err(e) => SpacesaverResult::err(&format!("Failed to initialize: {}", e)),
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
    let guard = get_manager().read();
    if let Some(manager) = guard.as_ref() {
        match manager.fetch_random_images(count as u32) {
            Ok(paths) => paths.len() as i32,
            Err(e) => {
                log::error!("Failed to fetch images: {}", e);
                -1
            }
        }
    } else {
        -1
    }
}

/// Fetch recent images (last N days)
/// Returns the number of images fetched, or -1 on error
#[no_mangle]
pub extern "C" fn spacesaver_fetch_recent(days: i32) -> i32 {
    let guard = get_manager().read();
    if let Some(manager) = guard.as_ref() {
        match manager.fetch_recent_images(days as u32) {
            Ok(paths) => paths.len() as i32,
            Err(e) => {
                log::error!("Failed to fetch images: {}", e);
                -1
            }
        }
    } else {
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
