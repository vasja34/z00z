//! Zero-state network-domain service marker.

/// Zero-state network-domain service marker used by app-level wiring.
pub struct NetworkService;

impl Default for NetworkService {
    fn default() -> Self {
        Self::new()
    }
}

impl NetworkService {
    /// Create a new zero-state `NetworkService` marker.
    pub fn new() -> Self {
        Self
    }
}
