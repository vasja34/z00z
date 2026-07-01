//! Zero-state storage-domain service marker.

/// Zero-state storage-domain service marker used by app-level wiring.
pub struct StorageService;

impl Default for StorageService {
    fn default() -> Self {
        Self::new()
    }
}

impl StorageService {
    /// Create a new zero-state `StorageService` marker.
    pub fn new() -> Self {
        Self
    }
}
