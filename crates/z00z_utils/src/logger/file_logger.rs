//! File-based Logger Implementation
//!
//! Writes log messages to a file in the specified directory.
//! Creates the directory if it doesn't exist.

use super::sanitize_message;
use crate::logger::traits::Logger;
use crate::time::{format_unix_timestamp_millis_utc, SystemTimeProvider, TimeProvider};
use std::fs::{create_dir_all, OpenOptions};
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

#[cfg(unix)]
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};

/// Logger that writes to a file
///
/// Thread-safe logger that appends messages to a log file.
/// Creates the directory structure if it doesn't exist.
///
/// # Example
///
/// ```rust
/// use z00z_utils::logger::{FileLogger, Logger};
///
/// let logger = FileLogger::new("logs/app.log").unwrap();
/// logger.info("Application started");
/// ```
pub struct FileLogger {
    file_path: PathBuf,
    file: Mutex<std::fs::File>,
}

impl FileLogger {
    /// Create a new FileLogger that writes to the specified path
    ///
    /// Creates parent directories if they don't exist.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the log file
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Cannot create parent directories
    /// - Cannot open/create the log file
    ///
    /// Security note: only the final log path component is rejected when it is a
    /// symlink. Parent directories are still trusted once created or selected.
    pub fn new<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        let file_path = path.as_ref().to_path_buf();

        // Create parent directory if it doesn't exist
        if let Some(parent) = file_path.parent() {
            create_dir_all(parent)?;
        }

        Self::ensure_no_symlink(&file_path)?;

        // Open file in append mode, create if doesn't exist
        let mut options = OpenOptions::new();
        options.create(true).append(true);

        #[cfg(unix)]
        options.mode(0o600);

        let file = options.open(&file_path)?;

        #[cfg(unix)]
        {
            let mode = std::fs::metadata(&file_path)?.permissions().mode() & 0o777;
            if mode != 0o600 {
                let perms = std::fs::Permissions::from_mode(0o600);
                std::fs::set_permissions(&file_path, perms)?;
            }
        }

        Ok(Self {
            file_path,
            file: Mutex::new(file),
        })
    }

    fn ensure_no_symlink(path: &Path) -> std::io::Result<()> {
        match std::fs::symlink_metadata(path) {
            Ok(meta) if meta.file_type().is_symlink() => Err(std::io::Error::new(
                ErrorKind::InvalidInput,
                format!("Refusing to open symlink log path: {}", path.display()),
            )),
            Ok(_) => Ok(()),
            Err(err) if err.kind() == ErrorKind::NotFound => Ok(()),
            Err(err) => Err(err),
        }
    }

    /// Get the path to the log file
    pub fn path(&self) -> &Path {
        &self.file_path
    }

    /// Write a log message to the file
    fn write_log(&self, level: &str, message: &str) {
        let timestamp =
            format_unix_timestamp_millis_utc(SystemTimeProvider.compat_unix_timestamp_millis());
        let msg = sanitize_message(message);
        let log_line = format!("[{}] [{}] {}\n", timestamp, level, msg);

        if let Ok(mut file) = self.file.lock() {
            let _ = file.write_all(log_line.as_bytes());
            let _ = file.flush();
        }
    }
}

impl Logger for FileLogger {
    fn error(&self, message: &str) {
        self.write_log("ERROR", message);
    }

    fn warn(&self, message: &str) {
        self.write_log("WARN", message);
    }

    fn info(&self, message: &str) {
        self.write_log("INFO", message);
    }

    fn debug(&self, message: &str) {
        self.write_log("DEBUG", message);
    }

    fn trace(&self, message: &str) {
        self.write_log("TRACE", message);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_file_logger_creates_directory() {
        let dir = tempdir().unwrap();
        let log_path = dir.path().join("logs/test.log");

        let logger = FileLogger::new(&log_path).unwrap();
        logger.info("test message");

        assert!(log_path.exists());

        let content = fs::read_to_string(&log_path).unwrap();
        assert!(content.contains("INFO"));
        assert!(content.contains("test message"));
    }

    #[test]
    fn test_file_logger_appends() {
        let dir = tempdir().unwrap();
        let log_path = dir.path().join("test.log");

        {
            let logger = FileLogger::new(&log_path).unwrap();
            logger.info("first message");
        }

        {
            let logger = FileLogger::new(&log_path).unwrap();
            logger.info("second message");
        }

        let content = fs::read_to_string(&log_path).unwrap();
        assert!(content.contains("first message"));
        assert!(content.contains("second message"));
    }

    #[test]
    fn test_file_logger_all_levels() {
        let dir = tempdir().unwrap();
        let log_path = dir.path().join("test.log");

        let logger = FileLogger::new(&log_path).unwrap();
        logger.error("error msg");
        logger.warn("warn msg");
        logger.info("info msg");
        logger.debug("debug msg");
        logger.trace("trace msg");

        let content = fs::read_to_string(&log_path).unwrap();
        assert!(content.contains("[ERROR] error msg"));
        assert!(content.contains("[WARN] warn msg"));
        assert!(content.contains("[INFO] info msg"));
        assert!(content.contains("[DEBUG] debug msg"));
        assert!(content.contains("[TRACE] trace msg"));
    }

    #[test]
    #[cfg(unix)]
    fn test_file_mode() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempdir().unwrap();
        let log_path = dir.path().join("mode.log");

        let logger = FileLogger::new(&log_path).unwrap();
        logger.info("ok");

        let mode = fs::metadata(&log_path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);
    }

    #[test]
    fn test_msg_sanitize() {
        let dir = tempdir().unwrap();
        let log_path = dir.path().join("sanitize.log");

        let logger = FileLogger::new(&log_path).unwrap();
        logger.info("good\nFAKE\rLINE\0END");

        let content = fs::read_to_string(&log_path).unwrap();
        assert!(content.contains("good\\nFAKE\\rLINEEND"));
    }

    #[test]
    #[cfg(unix)]
    fn test_symlink_reject() {
        let dir = tempdir().unwrap();
        let real = dir.path().join("real.log");
        let link = dir.path().join("link.log");
        fs::write(&real, b"seed").unwrap();
        std::os::unix::fs::symlink(&real, &link).unwrap();

        let result = FileLogger::new(&link);
        assert!(result.is_err());
    }
}
