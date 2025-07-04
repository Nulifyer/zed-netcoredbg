use std::io::Write;
use std::sync::OnceLock;

pub struct Logger;

static LOGGER: OnceLock<Logger> = OnceLock::new();

impl Logger {
    /// Enable/disable debug logging - set to false for production
    const DEBUG_ENABLED: bool = true;

    pub fn instance() -> &'static Logger {
        LOGGER.get_or_init(|| Logger)
    }

    pub fn debug(message: &str) {
        Self::instance().debug_log(message);
    }

    fn debug_log(&self, message: &str) {
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
