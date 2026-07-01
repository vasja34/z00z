/// Synchronous receiver-key derivation and cache management interface.
pub trait ReceiverManager {
    /// Derive a receiver spend public key for the given derivation path.
    ///
    /// This helper returns the derived spend key and does not encode an address.
    fn derive_spend_key(&mut self, path: Bip44Path) -> ReceiverManagerResult<Z00ZRistrettoPoint>;

    /// Derive both spend and view public keys for the given derivation path.
    ///
    /// - Spend key is derived from the provided canonical spend path.
    /// - View key is derived from `path.to_view_key_path()` (deterministic mapping).
    ///
    /// Implementations must reject identity keys and cache the pair as a single entry.
    fn derive_wallet_keys(&mut self, path: Bip44Path) -> ReceiverManagerResult<DerivedWalletKeys>;

    /// Get a cached receiver spend key for the given derivation path.
    fn get_receiver_key(&mut self, path: Bip44Path) -> ReceiverManagerResult<Z00ZRistrettoPoint>;

    /// List all cached receiver spend keys.
    fn list_receivers(&self) -> ReceiverManagerResult<Vec<(Bip44Path, Z00ZRistrettoPoint)>>;

    /// Clear the receiver cache.
    fn clear_cache(&mut self) -> ReceiverManagerResult<()>;

    /// Derive multiple receiver spend keys in a single operation.
    ///
    /// This is optimized for gap-limit scanning where many consecutive paths
    /// need to be derived. Provides better performance than sequential calls
    /// through single lock acquisition and bulk cache insertion.
    ///
    /// # Arguments
    ///
    /// * `paths` - Slice of derivation paths to derive
    ///
    /// # Returns
    ///
    /// Vector of public keys in the same order as input paths
    fn derive_batch(
        &mut self,
        paths: &[Bip44Path],
    ) -> ReceiverManagerResult<Vec<Z00ZRistrettoPoint>> {
        let mut results = Vec::with_capacity(paths.len());
        for path in paths {
            let pubkey = self.derive_spend_key(*path)?;
            results.push(pubkey);
        }
        Ok(results)
    }

    /// Get cache metrics snapshot for performance monitoring.
    fn metrics(&self) -> CacheMetricsSnapshot;

    /// Reset cache metrics.
    fn reset_metrics(&mut self);

    /// Create receiver card from receiver keys.
    fn create_receiver_card(&self, keys: &ReceiverKeys) -> ReceiverManagerResult<ReceiverCard> {
        let mut card = ReceiverCard {
            version: 1,
            owner_handle: keys.owner_handle,
            view_pk: keys.view_pk.as_bytes().try_into().map_err(|_| {
                ReceiverManagerError::StealthIntegration("invalid view key".to_string())
            })?,
            identity_pk: keys.identity_pk.as_bytes().try_into().map_err(|_| {
                ReceiverManagerError::StealthIntegration("invalid identity key".to_string())
            })?,
            card_id: None,
            metadata: None,
            signature: [0u8; 64],
        };

        card.sign(keys.reveal_identity_sk())
            .map_err(|e| ReceiverManagerError::StealthIntegration(e.to_string()))?;

        Ok(card)
    }

    /// Generate signed payment request.
    fn generate_payment_request(
        &self,
        keys: &ReceiverKeys,
        params: RequestParams,
        chain_id: u32,
    ) -> ReceiverManagerResult<PaymentRequest> {
        PaymentRequest::generate(keys, params, chain_id)
            .map_err(|e| ReceiverManagerError::StealthIntegration(e.to_string()))
    }

    /// Scan checkpoint and return owned outputs.
    fn scan_checkpoint(
        &self,
        keys: &ReceiverKeys,
        leaves: &[Asset],
    ) -> ReceiverManagerResult<Vec<WalletStealthOutput>> {
        self.scan_checkpoint_with_requests(keys, leaves, &[])
    }

    /// Scan checkpoint with active payment requests for req-bound outputs.
    /// Cursor persistence stays outside this batch helper and remains bound to
    /// `ScanStatePayload` rather than a scanner-local resume model.
    fn scan_checkpoint_with_requests(
        &self,
        keys: &ReceiverKeys,
        leaves: &[Asset],
        requests: &[PaymentRequest],
    ) -> ReceiverManagerResult<Vec<WalletStealthOutput>> {
        let scanner = StealthOutputScanner::from_keys(keys);
        let mut scanner = scanner;

        for request in requests {
            scanner.add_request(request);
        }

        Ok(scanner.scan_checkpoint(leaves))
    }

    /// Scan a checkpoint range with optional resume cursor and optional checkpoint cap.
    ///
    /// Remote worker chunks or proof hints, if introduced later, remain advisory
    /// inputs to this wallet-local detector and do not bypass local validation.
    fn scan_range(
        &self,
        keys: &ReceiverKeys,
        chunks: &[ScanChunk],
        resume: Option<&ScanStatePayload>,
        max_ckpt: Option<usize>,
    ) -> ReceiverManagerResult<ScanRangeOut> {
        self.scan_range_with_requests(keys, chunks, &[], resume, max_ckpt)
    }

    /// Scan a checkpoint range with active requests and resume cursor support.
    /// This wrapper delegates into the stable scanner core and does not own
    /// persistence or outward API mapping. Remote resume hints stay advisory and
    /// cannot mutate `ScanStatePayload` outside the wallet-local receive lane.
    fn scan_range_with_requests(
        &self,
        keys: &ReceiverKeys,
        chunks: &[ScanChunk],
        requests: &[PaymentRequest],
        resume: Option<&ScanStatePayload>,
        max_ckpt: Option<usize>,
    ) -> ReceiverManagerResult<ScanRangeOut> {
        let scanner = StealthOutputScanner::from_keys(keys);
        let mut scanner = scanner;

        for request in requests {
            scanner.add_request(request);
        }

        scanner
            .scan_range(chunks, resume, max_ckpt)
            .map_err(|err| ReceiverManagerError::StealthIntegration(err.to_string()))
    }
}

/// Listener invoked when the receiver cache evicts an entry due to LRU capacity pressure.
///
/// This is intended as an integration hook for higher layers (e.g. persistence, logging,
/// metrics) without coupling the core receiver manager to any specific storage backend.

#[async_trait]
pub trait AsyncReceiverManager: Send + Sync {
    /// Derive a receiver spend key asynchronously.
    async fn derive_spend_key(&self, path: Bip44Path)
        -> ReceiverManagerResult<Z00ZRistrettoPoint>;

    /// Derive both spend and view public keys asynchronously.
    async fn derive_wallet_keys(&self, path: Bip44Path) -> ReceiverManagerResult<DerivedWalletKeys>;

    /// Get a cached receiver spend key asynchronously.
    async fn get_receiver_key(&self, path: Bip44Path)
        -> ReceiverManagerResult<Z00ZRistrettoPoint>;

    /// List all cached receivers asynchronously.
    async fn list_receivers(&self)
        -> ReceiverManagerResult<Vec<(Bip44Path, Z00ZRistrettoPoint)>>;

    /// Clear the receiver cache asynchronously.
    async fn clear_cache(&self) -> ReceiverManagerResult<()>;

    /// Derive multiple receiver spend keys in a single async operation.
    async fn derive_batch(
        &self,
        paths: &[Bip44Path],
    ) -> ReceiverManagerResult<Vec<Z00ZRistrettoPoint>>;

    /// Get cache metrics asynchronously.
    async fn metrics(&self) -> CacheMetricsSnapshot;

    /// Reset cache metrics asynchronously.
    async fn reset_metrics(&self);
}

/// Async wrapper for ReceiverManagerImpl.
///
/// Provides non-blocking operations for receiver-key derivation and caching.
/// Useful for wallet services that need to remain responsive during
/// long-running key derivation operations.

impl From<Bip44Error> for ReceiverManagerError {
    fn from(error: Bip44Error) -> Self {
        ReceiverManagerError::KeyDerivation(error.to_string())
    }
}
