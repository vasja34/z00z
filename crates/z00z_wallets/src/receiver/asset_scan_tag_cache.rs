/// Context bucket entry for tag16 prefilter candidates.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tag16Context {
    /// Candidate DH key for decryption attempt.
    pub k_dh: [u8; 32],
    /// Optional request id binding.
    pub req_id: Option<[u8; 32]>,
}

/// Completeness state for tag16 cache-driven strict scan mode.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Tag16CacheState {
    /// Best-effort cache state; strict tag-only mode is not yet authorized.
    #[default]
    BestEffort,
    /// Caller materialized the complete concrete tag-context set.
    Complete,
}

/// Tag16 cache used for fast scan prefiltering.
/// This cache is advisory: active request registration does not, by itself,
/// materialize every decrypt context needed for a complete tag-only scan.
/// Callers must use `materialize_complete_tag_contexts(...)` when they hold the
/// full concrete tag-context set and want to authorize strict tag-only mode.
#[derive(Debug, Default)]
pub struct Tag16Cache {
    cache: HashMap<u16, Vec<Tag16Context>>,
    active_req_ids: BTreeSet<[u8; 32]>,
    completeness: Tag16CacheState,
    hits: AtomicU64,
    misses: AtomicU64,
    colls: AtomicU64,
}

impl Clone for Tag16Cache {
    fn clone(&self) -> Self {
        Self {
            cache: self.cache.clone(),
            active_req_ids: self.active_req_ids.clone(),
            completeness: self.completeness,
            hits: AtomicU64::new(self.hits.load(Ordering::Relaxed)),
            misses: AtomicU64::new(self.misses.load(Ordering::Relaxed)),
            colls: AtomicU64::new(self.colls.load(Ordering::Relaxed)),
        }
    }
}

/// Tag16 cache statistics.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CacheStats {
    /// Number of cache hits.
    pub hits: u64,
    /// Number of cache misses.
    pub misses: u64,
    /// Number of collisions (multiple contexts per tag16).
    pub collisions: u64,
    /// Current cache size.
    pub size: usize,
}

impl Tag16Cache {
    /// Create an empty tag16 cache.
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            active_req_ids: BTreeSet::new(),
            completeness: Tag16CacheState::BestEffort,
            hits: AtomicU64::new(0),
            misses: AtomicU64::new(0),
            colls: AtomicU64::new(0),
        }
    }

    /// Insert one context under a tag16 key.
    pub fn insert(&mut self, tag16: u16, context: Tag16Context) {
        if let Some(existing) = self.cache.get(&tag16) {
            if !existing.is_empty() {
                self.colls.fetch_add(1, Ordering::Relaxed);
            }
        }

        if let Some(req_id) = context.req_id {
            self.active_req_ids.insert(req_id);
        }

        self.cache.entry(tag16).or_default().push(context);
    }

    /// Check whether tag16 has at least one candidate context.
    pub fn contains(&self, tag16: u16) -> bool {
        let has = self.cache.contains_key(&tag16);
        if has {
            self.hits.fetch_add(1, Ordering::Relaxed);
        } else {
            self.misses.fetch_add(1, Ordering::Relaxed);
        }
        has
    }

    /// Get candidate contexts for tag16 key.
    pub fn get_contexts(&self, tag16: u16) -> Option<&[Tag16Context]> {
        self.cache.get(&tag16).map(Vec::as_slice)
    }

    /// Clear cache and active request set.
    pub fn clear(&mut self) {
        self.cache.clear();
        self.active_req_ids.clear();
        self.completeness = Tag16CacheState::BestEffort;
    }

    /// Materialize a complete concrete tag-context set and authorize strict tag-only mode.
    pub fn materialize_complete_tag_contexts<I>(&mut self, contexts: I)
    where
        I: IntoIterator<Item = (u16, Tag16Context)>,
    {
        for (tag16, context) in contexts {
            self.insert(tag16, context);
        }

        self.completeness = Tag16CacheState::Complete;
    }

    /// Return cache statistics snapshot.
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            hits: self.hits.load(Ordering::Relaxed),
            misses: self.misses.load(Ordering::Relaxed),
            collisions: self.colls.load(Ordering::Relaxed),
            size: self.cache.len(),
        }
    }

    /// Return cache completeness state.
    pub fn completeness(&self) -> Tag16CacheState {
        self.completeness
    }

    /// Check whether the cache holds the full concrete tag-context set.
    pub fn is_complete(&self) -> bool {
        matches!(self.completeness, Tag16CacheState::Complete)
    }

    /// Register request id as active.
    pub fn add_active_request(&mut self, req_id: [u8; 32]) {
        self.active_req_ids.insert(req_id);
    }

    /// Register full payment request as active context.
    /// This only records the active request id; callers still need explicit tag
    /// contexts for a strict tag-only prefilter path.
    pub fn add_request(&mut self, request: &PaymentRequest) {
        if request.is_expired() {
            return;
        }

        self.active_req_ids.insert(request.req_id);
    }

    /// Check whether request id is active.
    pub fn is_active_request(&self, req_id: &[u8; 32]) -> bool {
        self.active_req_ids.contains(req_id)
    }

    pub(crate) fn active_requests(&self) -> impl Iterator<Item = &[u8; 32]> {
        self.active_req_ids.iter()
    }

    pub(crate) fn size(&self) -> usize {
        self.cache.len()
    }
}

#[cfg(test)]
mod tests {
    use super::{PaymentRequest, Tag16Cache, Tag16CacheState, Tag16Context};

    fn request(req_id: [u8; 32], expiry: u64) -> PaymentRequest {
        PaymentRequest {
            version: 1,
            owner_handle: [1u8; 32],
            view_pk: [2u8; 32],
            identity_pk: [3u8; 32],
            req_id,
            chain_id: 7,
            amount: None,
            expiry,
            metadata: None,
            signature: [0u8; 64],
        }
    }

    #[test]
    fn test_active_requests_skip_expired() {
        let mut cache = Tag16Cache::new();

        cache.add_request(&request([0x02; 32], u64::MAX));
        cache.add_request(&request([0x03; 32], 0));
        cache.add_request(&request([0x01; 32], u64::MAX));

        let active: Vec<[u8; 32]> = cache.active_requests().copied().collect();
        assert_eq!(active, vec![[0x01; 32], [0x02; 32]]);
        assert!(cache.is_active_request(&[0x01; 32]));
        assert!(cache.is_active_request(&[0x02; 32]));
        assert!(!cache.is_active_request(&[0x03; 32]));
    }

    #[test]
    fn test_complete_state_is_explicit() {
        let mut cache = Tag16Cache::new();

        assert_eq!(cache.completeness(), Tag16CacheState::BestEffort);
        cache.add_request(&request([0x04; 32], u64::MAX));
        assert!(!cache.is_complete());

        cache.materialize_complete_tag_contexts([(
            7,
            Tag16Context {
                k_dh: [7u8; 32],
                req_id: None,
            },
        )]);

        assert!(cache.is_complete());
        assert_eq!(cache.completeness(), Tag16CacheState::Complete);
        assert!(cache.get_contexts(7).is_some());

        cache.clear();
        assert_eq!(cache.completeness(), Tag16CacheState::BestEffort);
        assert!(!cache.is_complete());
    }
}