use std::io::Write;

pub struct Logger;

impl Logger {
    /// Enable/disable debug logging - set to false for production
    const DEBUG_ENABLED: bool = true;

    pub fn new() -> Self {
        Self
    }

    /// Logs a debug message with timestamp
    pub fn debug_log(&self, message: &str) {
        if !Self::DEBUG_ENABLED {
            return;
        }

        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("netcoredbg_extension_debug.log")
        {
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            let _ = writeln!(file, "[{}] {}", timestamp, message);
        }
    }
}
