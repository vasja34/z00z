/// Information about a cryptographic backend implementation.
///
/// Provides capability introspection for debugging and compatibility checking.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackendInfo {
    /// Name of the backend (e.g., "TariCryptoBackend")
    pub name: &'static str,
    /// Version string (e.g., "1.0.0")
    pub version: &'static str,
    /// List of cryptographic algorithms supported
    pub algorithms: &'static [&'static str],
    /// Additional backend-specific metadata
    pub metadata: &'static [(&'static str, &'static str)],
}

impl BackendInfo {
    /// Create a new BackendInfo struct
    pub const fn new(
        name: &'static str,
        version: &'static str,
        algorithms: &'static [&'static str],
        metadata: &'static [(&'static str, &'static str)],
    ) -> Self {
        Self {
            name,
            version,
            algorithms,
            metadata,
        }
    }
}
