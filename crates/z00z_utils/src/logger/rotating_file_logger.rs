//! Rotating file-based Logger implementation.
//!
//! This logger appends log lines to a file and rotates the file when it exceeds
//! a configured size threshold.

use super::sanitize_message;
use crate::logger::traits::Logger;
use crate::time::{format_unix_timestamp_millis_utc, SystemTimeProvider, TimeProvider};
use std::fs::{create_dir_all, remove_file, rename, OpenOptions};
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Mutex,
};

#[cfg(unix)]
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};

/// File rotation configuration.
#[derive(Debug, Clone, Copy)]
pub struct RotationPolicy {
    /// Maximum size of the active log file before rotation.
    pub max_bytes: u64,
    /// Number of rotated files to keep.
    pub keep_files: usize,
}

/// Logger that writes to a file and rotates it when it grows beyond a threshold.
pub struct RotatingFileLogger {
    file_path: PathBuf,
    rotation: RotationPolicy,
    file: Mutex<Option<std::fs::File>>,
    io_failed: AtomicBool,
}

impl RotatingFileLogger {
    /// Create a new RotatingFileLogger.
    ///
    /// Security note: only the final active log file path is checked for a
    /// symlink. Existing parent directories are trusted once selected.
    pub fn new<P: AsRef<Path>>(path: P, rotation: RotationPolicy) -> std::io::Result<Self> {
        let file_path = path.as_ref().to_path_buf();

        if let Some(parent) = file_path.parent() {
            create_dir_all(parent)?;
        }

        let file = Self::open_append(&file_path)?;

        Ok(Self {
            file_path,
            rotation,
            file: Mutex::new(Some(file)),
            io_failed: AtomicBool::new(false),
        })
    }

    /// Get the path to the active log file.
    pub fn path(&self) -> &Path {
        &self.file_path
    }

    fn open_append(path: &Path) -> std::io::Result<std::fs::File> {
        Self::ensure_no_symlink(path)?;

        let mut options = OpenOptions::new();
        options.create(true).append(true);

        #[cfg(unix)]
        options.mode(0o600);

        let file = options.open(path)?;
        Self::set_mode(path)?;
        Ok(file)
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

    #[cfg(unix)]
    fn set_mode(path: &Path) -> std::io::Result<()> {
        let metadata = std::fs::metadata(path)?;
        let mode = metadata.permissions().mode() & 0o777;
        if mode != 0o600 {
            let perms = std::fs::Permissions::from_mode(0o600);
            std::fs::set_permissions(path, perms)?;
        }
        Ok(())
    }

    #[cfg(not(unix))]
    fn set_mode(_path: &Path) -> std::io::Result<()> {
        Ok(())
    }

    fn rotated_path(&self, index: usize) -> PathBuf {
        PathBuf::from(format!("{}.{}", self.file_path.to_string_lossy(), index))
    }

    fn note_io_failure(&self, context: &str, err: &std::io::Error) {
        if !self.io_failed.swap(true, Ordering::SeqCst) {
            eprintln!(
                "rotating log sink degraded at {} during {}: {}",
                self.file_path.display(),
                context,
                err
            );
        }
    }

    fn clear_io_failure(&self) {
        self.io_failed.store(false, Ordering::SeqCst);
    }

    fn rotate_files(&self) {
        if self.rotation.keep_files == 0 {
            let _ = remove_file(&self.file_path);
            return;
        }

        if Self::ensure_no_symlink(&self.file_path).is_err() {
            return;
        }

        let last = self.rotated_path(self.rotation.keep_files);
        let _ = remove_file(last);

        for idx in (1..self.rotation.keep_files).rev() {
            let from = self.rotated_path(idx);
            let to = self.rotated_path(idx + 1);
            let _ = rename(from, to);
        }

        let _ = rename(&self.file_path, self.rotated_path(1));
    }

    fn ensure_open_rotate_needed(
        &self,
        file_guard: &mut Option<std::fs::File>,
        next_write_bytes: u64,
    ) {
        if file_guard.is_none() {
            match Self::open_append(&self.file_path) {
                Ok(file) => {
                    *file_guard = Some(file);
                    self.clear_io_failure();
                }
                Err(err) => {
                    self.note_io_failure("open", &err);
                    return;
                }
            }
        }

        let current_size = std::fs::metadata(&self.file_path)
            .map(|m| m.len())
            .unwrap_or(0);

        if current_size.saturating_add(next_write_bytes) <= self.rotation.max_bytes {
            return;
        }

        if let Some(mut file) = file_guard.take() {
            if let Err(err) = file.flush() {
                self.note_io_failure("flush-before-rotate", &err);
            }
        }

        self.rotate_files();

        match Self::open_append(&self.file_path) {
            Ok(file) => {
                *file_guard = Some(file);
                self.clear_io_failure();
            }
            Err(err) => self.note_io_failure("reopen-after-rotate", &err),
        }
    }

    fn write_log(&self, level: &str, message: &str) {
        let timestamp =
            format_unix_timestamp_millis_utc(SystemTimeProvider.compat_unix_timestamp_millis());
        let msg = sanitize_message(message);
        let log_line = format!("[{}] [{}] {}\n", timestamp, level, msg);

        let mut guard = match self.file.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };

        self.ensure_open_rotate_needed(&mut guard, log_line.len() as u64);

        if let Some(file) = guard.as_mut() {
            if let Err(err) = file.write_all(log_line.as_bytes()) {
                self.note_io_failure("write", &err);
                return;
            }
            if let Err(err) = file.flush() {
                self.note_io_failure("flush", &err);
                return;
            }
            self.clear_io_failure();
        }
    }
}

impl Logger for RotatingFileLogger {
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

    #[test]
    fn test_rotates_and_keeps_files() {
        let dir = tempfile::tempdir().unwrap();
        let log_path = dir.path().join("rpc.log");

        let logger = RotatingFileLogger::new(
            &log_path,
            RotationPolicy {
                max_bytes: 128,
                keep_files: 2,
            },
        )
        .unwrap();

        for _ in 0..50 {
            logger.info("0123456789012345678901234567890123456789");
        }

        assert!(log_path.exists());

        // Active file must exist, and rotated files should be bounded.
        let rotated_1 = PathBuf::from(format!("{}.1", log_path.to_string_lossy()));
        let rotated_2 = PathBuf::from(format!("{}.2", log_path.to_string_lossy()));

        // At least one rotation should happen.
        assert!(rotated_1.exists() || rotated_2.exists());

        // If keep_files = 2, there must not be a .3.
        let rotated_3 = PathBuf::from(format!("{}.3", log_path.to_string_lossy()));
        assert!(!rotated_3.exists());

        // Sanity: log files contain lines.
        let content = fs::read_to_string(&log_path).unwrap_or_default();
        assert!(content.contains("0123456789"));
    }

    #[test]
    #[cfg(unix)]
    fn test_log_file_mode() {
        let dir = tempfile::tempdir().unwrap();
        let log_path = dir.path().join("perm.log");

        let logger = RotatingFileLogger::new(
            &log_path,
            RotationPolicy {
                max_bytes: 1024,
                keep_files: 1,
            },
        )
        .unwrap();

        logger.info("hello");

        let mode = fs::metadata(&log_path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);
    }

    #[test]
    fn test_log_msg_sanitize() {
        let dir = tempfile::tempdir().unwrap();
        let log_path = dir.path().join("sanitize.log");

        let logger = RotatingFileLogger::new(
            &log_path,
            RotationPolicy {
                max_bytes: 1024,
                keep_files: 1,
            },
        )
        .unwrap();

        logger.info("good\nFAKE\rLINE\0END");

        let content = fs::read_to_string(&log_path).unwrap();
        assert!(content.contains("good\\nFAKE\\rLINEEND"));
        assert!(!content.contains("FAKE\r"));
        assert!(content.contains("[INFO]"));
    }

    #[test]
    #[cfg(unix)]
    fn test_log_symlink_reject() {
        let dir = tempfile::tempdir().unwrap();
        let real = dir.path().join("real.log");
        let link = dir.path().join("link.log");
        fs::write(&real, b"seed").unwrap();
        std::os::unix::fs::symlink(&real, &link).unwrap();

        let err = match RotatingFileLogger::new(
            &link,
            RotationPolicy {
                max_bytes: 1024,
                keep_files: 1,
            },
        ) {
            Ok(_) => panic!("symlink log path must be rejected"),
            Err(err) => err,
        };

        assert_eq!(err.kind(), ErrorKind::InvalidInput);

        let content = fs::read_to_string(&real).unwrap();
        assert_eq!(content, "seed");
    }
}
