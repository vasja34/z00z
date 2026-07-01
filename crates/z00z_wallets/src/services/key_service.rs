//! Zero-state key-domain service marker.

/// Zero-state key-domain service marker used by app-level wiring.
pub struct KeyService;

impl Default for KeyService {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyService {
    /// Create a new zero-state `KeyService` marker.
    pub fn new() -> Self {
        Self
    }
}
