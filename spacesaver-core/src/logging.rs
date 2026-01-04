//! File-based logging for screen saver context
//!
//! Since screen savers don't have access to environment variables,
//! we use file-based logging instead of env_logger.

use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

use once_cell::sync::OnceCell;

use crate::config::Config;

/// Global log file handle
static LOG_FILE: OnceCell<Mutex<Option<File>>> = OnceCell::new();

/// Maximum log file size before rotation (1 MB)
const MAX_LOG_SIZE: u64 = 1024 * 1024;

/// Initialize file logging
pub fn init_logging() {
    let _ = LOG_FILE.get_or_init(|| {
        let file = match get_log_path() {
            Ok(path) => {
                // Rotate log if too large
                if let Ok(metadata) = std::fs::metadata(&path) {
                    if metadata.len() > MAX_LOG_SIZE {
                        let _ = std::fs::remove_file(&path);
                    }
                }

                OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&path)
                    .ok()
            }
            Err(_) => None,
        };
        Mutex::new(file)
    });

    // Log startup
    log_info("Spacesaver logging initialized");
}

/// Get the log file path
fn get_log_path() -> Result<PathBuf, crate::error::Error> {
    let cache_dir = Config::cache_dir()?;
    std::fs::create_dir_all(&cache_dir)?;
    Ok(cache_dir.join("spacesaver.log"))
}

/// Get the public log file path (for external access)
pub fn log_file_path() -> Option<PathBuf> {
    get_log_path().ok()
}

/// Write a log message
fn write_log(level: &str, message: &str) {
    if let Some(mutex) = LOG_FILE.get() {
        if let Ok(mut guard) = mutex.lock() {
            if let Some(ref mut file) = *guard {
                let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
                let _ = writeln!(file, "[{}] {} - {}", timestamp, level, message);
                let _ = file.flush();
            }
        }
    }
}

/// Log an info message
pub fn log_info(message: &str) {
    write_log("INFO", message);
}

/// Log a warning message
pub fn log_warn(message: &str) {
    write_log("WARN", message);
}

/// Log an error message
pub fn log_error(message: &str) {
    write_log("ERROR", message);
}

/// Log a debug message
pub fn log_debug(message: &str) {
    write_log("DEBUG", message);
}
