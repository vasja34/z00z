//! Directory service authentication stubs.

/// Directory service authentication placeholder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectoryAuth {
    /// API key used for directory service calls.
    pub api_key: String,
    /// Directory endpoint URL.
    pub endpoint: String,
}

impl DirectoryAuth {
    /// Build directory authentication descriptor.
    pub fn new(api_key: String, endpoint: String) -> Self {
        Self { api_key, endpoint }
    }
}

#[cfg(test)]
mod tests {
    use super::DirectoryAuth;

    #[test]
    fn test_directory_auth_new() {
        let auth = DirectoryAuth::new("key".to_string(), "https://dir.local".to_string());
        assert_eq!(auth.api_key, "key");
        assert_eq!(auth.endpoint, "https://dir.local");
    }
}
