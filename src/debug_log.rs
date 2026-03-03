//! Debug logging module for Receipt Extractor
//!
//! Provides optional debug logging that can be enabled/disabled via settings.
//! Logs are written to ~/.config/receipt_extractor/debug.log

use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};


/// Global flag to control whether debug logging is enabled
static DEBUG_ENABLED: AtomicBool = AtomicBool::new(false);

/// Enable or disable debug logging
pub fn set_debug_enabled(enabled: bool) {
    DEBUG_ENABLED.store(enabled, Ordering::Relaxed);
}

/// Check if debug logging is enabled
#[allow(dead_code)]
pub fn is_debug_enabled() -> bool {
    DEBUG_ENABLED.load(Ordering::Relaxed)
}

/// Get the debug log file path
pub fn get_log_file_path() -> Option<PathBuf> {
    dirs::config_dir().map(|mut path| {
        path.push("receipt_extractor");
        path.push("debug.log");
        path
    })
}

/// Log a message to the debug log file (if debug logging is enabled)
pub fn log(msg: &str) {
    if !DEBUG_ENABLED.load(Ordering::Relaxed) {
        return;
    }

    if let Some(log_path) = get_log_file_path() {
        // Ensure parent directory exists
        if let Some(parent) = log_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
        {
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            let _ = writeln!(file, "[{}] {}", timestamp, msg);
        }
    }
}

/// Clear the debug log file
pub fn clear_log() -> std::io::Result<()> {
    if let Some(log_path) = get_log_file_path() {
        if log_path.exists() {
            File::create(&log_path)?;
        }
    }
    Ok(())
}

/// Get the size of the debug log file in bytes
pub fn get_log_size() -> Option<u64> {
    get_log_file_path().and_then(|path| {
        std::fs::metadata(&path).ok().map(|m| m.len())
    })
}

/// Format file size for display
pub fn format_file_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}

