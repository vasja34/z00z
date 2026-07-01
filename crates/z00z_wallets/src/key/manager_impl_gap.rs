impl<R: SecureRngProvider> KeyManagerImpl<R> {
    fn gap_span(&self, next_index: u32, last_used_plus1: u32, branch: &'static str) -> Result<u32> {
        next_index.checked_sub(last_used_plus1).ok_or_else(|| {
            self.logger
                .error(&format!("Gap state corrupted for {branch} branch"));
            KeyManagerError::StateCorrupted
        })
    }

    fn next_gap(&self, next_index: u32, branch: &'static str) -> Result<u32> {
        next_index.checked_add(1).ok_or_else(|| {
            self.logger
                .error(&format!("Gap counter overflow for {branch} branch"));
            KeyManagerError::StateCorrupted
        })
    }

    /// Reserve the next external address index while enforcing the BIP-44 gap limit.
    pub fn next_external(&self) -> Result<u32> {
        loop {
            let next_index = self.gap_external.load(Ordering::Acquire);
            let last_used_plus1 = self.last_used_ext.load(Ordering::Acquire);
            let gap = self.gap_span(next_index, last_used_plus1, "external")?;

            if gap >= BIP44_GAP_LIMIT {
                let next_index_now = self.gap_external.load(Ordering::Acquire);
                let last_used_plus1_now = self.last_used_ext.load(Ordering::Acquire);
                let gap_now = self.gap_span(next_index_now, last_used_plus1_now, "external")?;
                if gap_now >= BIP44_GAP_LIMIT {
                    return Err(KeyManagerError::GapLimitExceeded { gap: gap_now });
                }
                continue;
            }

            let new_next = self.next_gap(next_index, "external")?;
            if self
                .gap_external
                .compare_exchange_weak(next_index, new_next, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
            {
                return Ok(next_index);
            }
        }
    }

    /// Reserve the next internal address index while enforcing the BIP-44 gap limit.
    pub fn next_internal(&self) -> Result<u32> {
        loop {
            let next_index = self.gap_internal.load(Ordering::Acquire);
            let last_used_plus1 = self.last_used_int.load(Ordering::Acquire);
            let gap = self.gap_span(next_index, last_used_plus1, "internal")?;

            if gap >= BIP44_GAP_LIMIT {
                let next_index_now = self.gap_internal.load(Ordering::Acquire);
                let last_used_plus1_now = self.last_used_int.load(Ordering::Acquire);
                let gap_now = self.gap_span(next_index_now, last_used_plus1_now, "internal")?;
                if gap_now >= BIP44_GAP_LIMIT {
                    return Err(KeyManagerError::GapLimitExceeded { gap: gap_now });
                }
                continue;
            }

            let new_next = self.next_gap(next_index, "internal")?;
            if self
                .gap_internal
                .compare_exchange_weak(next_index, new_next, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
            {
                return Ok(next_index);
            }
        }
    }

    /// Mark an external address as used and advance tracked external gap state.
    pub fn mark_external_used(&self, index: u32) {
        let index_plus1 = index.saturating_add(1);

        loop {
            let current = self.last_used_ext.load(Ordering::Acquire);
            if index_plus1 <= current {
                break;
            }

            if self
                .last_used_ext
                .compare_exchange_weak(current, index_plus1, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
            {
                break;
            }
        }

        loop {
            let current = self.gap_external.load(Ordering::Acquire);
            if index_plus1 <= current {
                break;
            }

            if self
                .gap_external
                .compare_exchange_weak(current, index_plus1, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
            {
                break;
            }
        }
    }

    /// Mark an internal address as used and advance tracked internal gap state.
    pub fn mark_internal_used(&self, index: u32) {
        let index_plus1 = index.saturating_add(1);

        loop {
            let current = self.last_used_int.load(Ordering::Acquire);
            if index_plus1 <= current {
                break;
            }

            if self
                .last_used_int
                .compare_exchange_weak(current, index_plus1, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
            {
                break;
            }
        }

        loop {
            let current = self.gap_internal.load(Ordering::Acquire);
            if index_plus1 <= current {
                break;
            }

            if self
                .gap_internal
                .compare_exchange_weak(current, index_plus1, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
            {
                break;
            }
        }
    }
}