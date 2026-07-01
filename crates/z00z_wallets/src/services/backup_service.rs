//! Zero-state backup-domain service marker.

/// Zero-state backup-domain service marker used by app-level wiring.
pub struct BackupService;

impl Default for BackupService {
    fn default() -> Self {
        Self::new()
    }
}

impl BackupService {
    /// Create a new zero-state `BackupService` marker.
    pub fn new() -> Self {
        Self
    }
}
