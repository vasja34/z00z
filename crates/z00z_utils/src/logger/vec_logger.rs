//! In-memory logger implementation.
//!
//! This logger is intended for tests and deterministic verification of log output.

use crate::logger::traits::Logger;
use std::sync::Mutex;

/// Logger that stores log lines in memory.
#[derive(Default)]
pub struct VecLogger {
    lines: Mutex<Vec<String>>,
}

impl VecLogger {
    /// Create a new empty VecLogger.
    pub fn new() -> Self {
        Self::default()
    }

    /// Return a snapshot of all captured log lines.
    pub fn lines(&self) -> Vec<String> {
        self.lines
            .lock()
            .map(|lines| lines.clone())
            .unwrap_or_default()
    }

    /// Clear all captured log lines.
    pub fn clear(&self) {
        if let Ok(mut lines) = self.lines.lock() {
            lines.clear();
        }
    }

    fn push(&self, level: &str, msg: &str) {
        if let Ok(mut lines) = self.lines.lock() {
            lines.push(format!("[{level}] {msg}"));
        }
    }
}

impl Logger for VecLogger {
    fn error(&self, msg: &str) {
        self.push("ERROR", msg);
    }

    fn warn(&self, msg: &str) {
        self.push("WARN", msg);
    }

    fn info(&self, msg: &str) {
        self.push("INFO", msg);
    }

    fn debug(&self, msg: &str) {
        self.push("DEBUG", msg);
    }

    fn trace(&self, msg: &str) {
        self.push("TRACE", msg);
    }
}
