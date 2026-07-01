use std::collections::{HashMap, HashSet, VecDeque};

use crate::WalletError;

/// In-memory dedup cache for ephemeral sender public keys per receiver handle.
#[derive(Clone, Debug)]
pub struct EphemeralCache {
    capacity: usize,
    map: HashMap<[u8; 32], CacheBucket>,
}

#[derive(Clone, Debug, Default)]
struct CacheBucket {
    seen: HashSet<[u8; 32]>,
    order: VecDeque<[u8; 32]>,
}

impl EphemeralCache {
    /// Create bounded cache.
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity: capacity.max(1),
            map: HashMap::new(),
        }
    }

    /// Insert a receiver-handle / R_pub pair and reject duplicates.
    pub fn check_and_insert(
        &mut self,
        handle: &[u8; 32],
        r_pub: &[u8; 32],
    ) -> Result<(), WalletError> {
        let entry = self.map.entry(*handle).or_default();
        if entry.seen.contains(r_pub) {
            return Err(WalletError::DuplicateEphemeralR);
        }

        if entry.order.len() == self.capacity {
            if let Some(oldest) = entry.order.pop_front() {
                entry.seen.remove(&oldest);
            }
        }

        entry.order.push_back(*r_pub);
        entry.seen.insert(*r_pub);
        Ok(())
    }
}

impl Default for EphemeralCache {
    fn default() -> Self {
        Self::new(64)
    }
}
