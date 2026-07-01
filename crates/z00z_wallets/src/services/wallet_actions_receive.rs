use crate::rpc::types::chain::RuntimeReceiveScanOutcome;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ReceiveRangeOutcome {
    done_ckpt: u64,
    hit_count: u64,
    resume_height: u64,
    cursor_height: u64,
    outcome: RuntimeReceiveScanOutcome,
}

#[cfg(test)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ReceivePersistTestHook {
    AbortBeforePersist,
    MutateCursorBeforePersist { height: u64, hash: Vec<u8> },
}

#[cfg(test)]
static RECEIVE_PERSIST_TEST_HOOKS: std::sync::LazyLock<
    std::sync::Mutex<std::collections::BTreeMap<PersistWalletId, ReceivePersistTestHook>>,
> = std::sync::LazyLock::new(|| std::sync::Mutex::new(std::collections::BTreeMap::new()));

impl WalletService {
    fn persisted_wallet_chain_id(chain: crate::ChainType) -> u32 {
        match chain {
            crate::ChainType::Mainnet => 1,
            crate::ChainType::Testnet => 2,
            crate::ChainType::Devnet => 3,
        }
    }

    fn request_inbox_error(
        validation: Option<crate::receiver::RequestInboxValidation>,
    ) -> WalletError {
        match validation {
            Some(validation) => WalletError::InvalidParams(validation.as_str().to_string()),
            None => WalletError::InvalidParams("request inbox has no approved request".to_string()),
        }
    }

    fn map_remote_scan_worker_error(error: crate::chain::RemoteScanWorkerError) -> WalletError {
        match error {
            crate::chain::RemoteScanWorkerError::Deferred(message) => {
                WalletError::InvalidConfig(format!("remote scan worker deferred: {message}"))
            }
            crate::chain::RemoteScanWorkerError::EvidenceUnavailable(message) => {
                WalletError::InvalidConfig(format!(
                    "remote scan worker evidence unavailable: {message}"
                ))
            }
            crate::chain::RemoteScanWorkerError::ChainClient(message) => {
                WalletError::InvalidConfig(format!(
                    "remote scan worker chain client error: {message}"
                ))
            }
            crate::chain::RemoteScanWorkerError::ProofHint(message) => {
                WalletError::InvalidConfig(format!(
                    "remote scan worker proof hint error: {message}"
                ))
            }
            crate::chain::RemoteScanWorkerError::Transport(message) => {
                WalletError::InvalidConfig(format!(
                    "remote scan worker transport error: {message}"
                ))
            }
        }
    }

    fn validate_worker_evidence(
        evidence: &crate::chain::RemoteScanEvidence,
        resume: &ScanStatePayload,
    ) -> WalletResult<()> {
        let mut previous_chunk_height = None;
        for chunk in &evidence.chunks {
            if chunk.hash.is_empty() {
                return Err(WalletError::InvalidConfig(
                    "remote chunk hash must not be empty".to_string(),
                ));
            }

            if let Some(previous_chunk_height) = previous_chunk_height {
                if chunk.height <= previous_chunk_height {
                    return Err(WalletError::InvalidConfig(
                        "remote chunks must be strictly increasing".to_string(),
                    ));
                }

                if chunk.height != previous_chunk_height.saturating_add(1) {
                    return Err(WalletError::InvalidConfig(
                        "remote chunks must be contiguous".to_string(),
                    ));
                }
            }

            previous_chunk_height = Some(chunk.height);
        }

        let chunk_heights = evidence
            .chunks
            .iter()
            .map(|chunk| chunk.height)
            .collect::<std::collections::BTreeSet<_>>();

        for proof_hint in &evidence.proof_hints {
            if proof_hint.proof_bytes.is_empty() {
                return Err(WalletError::InvalidConfig(
                    "remote proof hint bytes must not be empty".to_string(),
                ));
            }

            if !chunk_heights.contains(&proof_hint.checkpoint_height) {
                return Err(WalletError::InvalidConfig(
                    "remote proof hint must match a returned chunk".to_string(),
                ));
            }
        }

        if let Some(resume_hint) = &evidence.resume_hint {
            if resume.is_origin() {
                if !resume_hint.last_chunk_hash.is_empty() {
                    return Err(WalletError::InvalidConfig(
                        "remote resume hint cannot set local cursor".to_string(),
                    ));
                }
            } else {
                if resume_hint.last_chunk_hash != resume.last_scanned_hash {
                    return Err(WalletError::InvalidConfig(
                        "remote resume hint mismatches local cursor".to_string(),
                    ));
                }

                if resume_hint.next_height < resume.height().saturating_add(1) {
                    return Err(WalletError::InvalidConfig(
                        "remote resume hint rewinds local cursor".to_string(),
                    ));
                }
            }
        }

        Ok(())
    }

    fn build_receive_range_outcome(
        resume: &ScanStatePayload,
        out: &ScanRangeOut,
        imported_count: u64,
    ) -> ReceiveRangeOutcome {
        let hit_count = out.outputs.len() as u64;
        let outcome = if imported_count > 0 {
            RuntimeReceiveScanOutcome::ImportedHit
        } else if hit_count == 0 && resume.is_origin() {
            RuntimeReceiveScanOutcome::NoHit
        } else if hit_count == 0 {
            RuntimeReceiveScanOutcome::Resumed
        } else {
            RuntimeReceiveScanOutcome::Scanned
        };

        ReceiveRangeOutcome {
            done_ckpt: out.stat.done_ckpt,
            hit_count,
            resume_height: resume.height(),
            cursor_height: out.stat.cursor.height(),
            outcome,
        }
    }

    pub(crate) fn classify_receive_scan_error(
        error: &WalletError,
    ) -> Option<RuntimeReceiveScanOutcome> {
        let lower = error.to_string().to_ascii_lowercase();

        if lower.contains("remote chunk hash must not be empty")
            || lower.contains("remote chunks must be strictly increasing")
            || lower.contains("remote chunks must be contiguous")
            || lower.contains("remote proof hint bytes must not be empty")
            || lower.contains("remote proof hint must match a returned chunk")
            || lower.contains("remote resume hint cannot set local cursor")
            || lower.contains("remote resume hint mismatches local cursor")
            || lower.contains("remote resume hint rewinds local cursor")
        {
            return Some(RuntimeReceiveScanOutcome::WorkerEvidenceRejected);
        }

        if lower.contains("scan state changed during receive persistence")
            || lower.contains("scan range cursor mismatch")
        {
            return Some(RuntimeReceiveScanOutcome::CursorConflict);
        }

        if matches!(error, WalletError::UnsupportedVersion(_))
            || matches!(error, WalletError::InvalidAssetPack(message) if message.to_ascii_lowercase().contains("unsupported"))
            || lower.contains("unsupported version")
            || lower.contains("unsupported asset pack")
        {
            return Some(RuntimeReceiveScanOutcome::UnsupportedVersion);
        }

        None
    }

    pub(crate) async fn record_receive_scan_outcome(
        &self,
        wallet_id: &PersistWalletId,
        outcome: RuntimeReceiveScanOutcome,
    ) {
        let mut store = self.last_receive_scan_outcomes.write().await;
        store.insert(wallet_id.clone(), outcome);
    }

    async fn record_receive_scan_error(
        &self,
        wallet_id: &PersistWalletId,
        error: &WalletError,
    ) {
        if let Some(outcome) = Self::classify_receive_scan_error(error) {
            self.record_receive_scan_outcome(wallet_id, outcome).await;
        }
    }

    pub(crate) async fn last_receive_scan_outcome(
        &self,
        wallet_id: &PersistWalletId,
    ) -> Option<RuntimeReceiveScanOutcome> {
        let store = self.last_receive_scan_outcomes.read().await;
        store.get(wallet_id).copied()
    }

    #[cfg(test)]
    pub(crate) fn set_receive_persist_test_hook(
        wallet_id: Option<PersistWalletId>,
        hook: Option<ReceivePersistTestHook>,
    ) {
        let mut guard = RECEIVE_PERSIST_TEST_HOOKS
            .lock()
            .expect("receive persist hook mutex poisoned");
        match (wallet_id, hook) {
            (Some(wallet_id), Some(hook)) => {
                guard.insert(wallet_id, hook);
            }
            (Some(wallet_id), None) => {
                guard.remove(&wallet_id);
            }
            (None, None) => {
                guard.clear();
            }
            (None, Some(_)) => {}
        }
    }

    #[cfg(test)]
    fn take_receive_persist_test_hook(wallet_id: &PersistWalletId) -> Option<ReceivePersistTestHook> {
        RECEIVE_PERSIST_TEST_HOOKS
            .lock()
            .expect("receive persist hook mutex poisoned")
            .remove(wallet_id)
    }

    pub(crate) fn live_claimed_assets_from_page(
        page: crate::db::AssetPage,
    ) -> WalletResult<Vec<Asset>> {
        page.items
            .into_iter()
            .filter(|payload| payload.is_live_claimed_status())
            .map(|payload| payload.validate_invariants())
            .collect()
    }

    fn filter_validator_mandate_locked_assets(
        spendable_rows: Vec<crate::db::redb_store::OwnedAssetPayload>,
        right_rows: &[crate::db::OwnedRightPayload],
    ) -> WalletResult<Vec<crate::db::redb_store::OwnedAssetPayload>> {
        if right_rows.is_empty() {
            return Ok(spendable_rows);
        }

        let mut filtered = Vec::with_capacity(spendable_rows.len());
        for row in spendable_rows {
            let asset = row.validate_invariants()?;
            let locked = right_rows.iter().any(|right| {
                crate::tx::has_validator_mandate_lock_profile(&right.labels)
                    && crate::tx::validator_mandate_lock_matches_asset(
                        &right.right_leaf,
                        &row.asset_id,
                        asset.amount,
                    )
            });
            if !locked {
                filtered.push(row);
            }
        }

        Ok(filtered)
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn load_claimed_assets_session(
        &self,
        session: &crate::db::WalletSession,
    ) -> WalletResult<Vec<Asset>> {
        let page = crate::db::wallet_asset_store().list_owned_assets(
            session,
            crate::db::AssetFilter::default(),
            None,
            usize::MAX,
        )?;
        Self::live_claimed_assets_from_page(page)
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) async fn list_spendable_asset_rows(
        &self,
        wallet_id: &PersistWalletId,
        asset_definition_id: Option<[u8; 32]>,
    ) -> WalletResult<Vec<crate::db::redb_store::OwnedAssetPayload>> {
        let now_ms = self.require_now_ms()?;
        let timeout_ms = self.timeout_ms();
        let session = self
            .wallet_sessions
            .session_for_wallet(wallet_id, now_ms, timeout_ms)
            .await?;

        session.with_wallet_session(|wlt_session| {
            let spendable_rows = crate::db::wallet_asset_store().list_spendable_assets(
                wlt_session,
                asset_definition_id,
                usize::MAX,
            )?;
            let right_rows = crate::db::object_inventory_store().list_right_inventory(
                wlt_session,
                None,
                None,
                usize::MAX,
            )?;
            let active_lock_rows = right_rows
                .into_iter()
                .filter(|right| {
                    matches!(
                        right.status,
                        crate::db::OwnedRightStatus::Granted
                            | crate::db::OwnedRightStatus::Held
                            | crate::db::OwnedRightStatus::Delegated
                    ) && right.policy.availability == crate::db::WalletPolicyAvailability::Available
                })
                .collect::<Vec<_>>();

            Self::filter_validator_mandate_locked_assets(spendable_rows, &active_lock_rows)
        })
    }

    #[cfg(target_arch = "wasm32")]
    pub(crate) async fn list_spendable_asset_rows(
        &self,
        wallet_id: &PersistWalletId,
        asset_definition_id: Option<[u8; 32]>,
    ) -> WalletResult<Vec<crate::db::redb_store::OwnedAssetPayload>> {
        let _ = (wallet_id, asset_definition_id);
        Err(WalletError::InvalidConfig(
            "live spendable asset query is not supported on wasm32".to_string(),
        ))
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) async fn sync_claimed_asset_cache(
        &self,
        wallet_id: &PersistWalletId,
    ) -> WalletResult<Vec<Asset>> {
        let now_ms = self.require_now_ms()?;
        let timeout_ms = self.timeout_ms();
        let session = self
            .wallet_sessions
            .session_for_wallet(wallet_id, now_ms, timeout_ms)
            .await?;

        let claimed_assets =
            session.with_wallet_session(|wlt_session| self.load_claimed_assets_session(wlt_session))?;
        self.install_claimed_assets(wallet_id, claimed_assets.clone()).await;
        Ok(claimed_assets)
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) async fn list_claimed_assets_live_cache(
        &self,
        wallet_id: &PersistWalletId,
    ) -> WalletResult<Vec<Asset>> {
        let now_ms = self.require_now_ms()?;
        let timeout_ms = self.timeout_ms();
        match self
            .wallet_sessions
            .session_for_wallet(wallet_id, now_ms, timeout_ms)
            .await
        {
            Ok(session) => session
                .with_wallet_session(|wlt_session| self.load_claimed_assets_session(wlt_session)),
            Err(WalletError::SessionExpired) | Err(WalletError::SessionInvalid) => {
                self.list_claimed_assets(wallet_id).await
            }
            Err(error) => Err(error),
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub(crate) async fn list_claimed_assets_live_cache(
        &self,
        wallet_id: &PersistWalletId,
    ) -> WalletResult<Vec<Asset>> {
        self.list_claimed_assets(wallet_id).await
    }

    async fn load_scan_state(&self, wallet_id: &PersistWalletId) -> WalletResult<ScanStatePayload> {
        let now_ms = self.require_now_ms()?;
        let timeout_ms = self.timeout_ms();
        let session = self
            .wallet_sessions
            .session_for_wallet(wallet_id, now_ms, timeout_ms)
            .await?;

        session.with_wallet_session(|wlt_session| {
            Ok(crate::db::read_scan_state(wlt_session)?
                .unwrap_or_else(|| ScanStatePayload::new(0, Vec::new())))
        })
    }

    fn collect_scan_hits(
        &self,
        chunks: &[ScanChunk],
        scanner: &StealthOutputScanner,
        start: usize,
        done_ckpt: u64,
    ) -> WalletResult<Vec<Asset>> {
        let end = start.saturating_add(done_ckpt as usize).min(chunks.len());
        let mut assets = Vec::new();

        for chunk in &chunks[start..end] {
            for leaf in &chunk.leaves {
                if matches!(scanner.scan_leaf(leaf), ScanResult::Mine { .. }) {
                    if let Some(asset) = recv_claim_asset(leaf) {
                        assets.push(asset);
                    }
                }
            }
        }

        Ok(assets)
    }

    async fn persist_scan_batch(
        &self,
        wallet_id: &PersistWalletId,
        assets: &[Asset],
        expected_resume: &ScanStatePayload,
        start: usize,
        done_ckpt: u64,
        chunks: &[ScanChunk],
        cursor: &ScanStatePayload,
    ) -> WalletResult<u64> {
        let scan_ref = if done_ckpt == 0 {
            None
        } else {
            let end = start.saturating_add(done_ckpt as usize).min(chunks.len());
            let start_height = chunks
                .get(start)
                .map(|chunk| chunk.height)
                .unwrap_or_else(|| cursor.height());
            let end_height = chunks
                .get(end.saturating_sub(1))
                .map(|chunk| chunk.height)
                .unwrap_or_else(|| cursor.height());
            Some(crate::db::redb_store::ScanRef {
                start_height,
                end_height,
                cursor_hash: cursor.last_scanned_hash.clone(),
            })
        };

        let now_ms = self.require_now_ms()?;
        let timeout_ms = self.timeout_ms();
        let session = self
            .wallet_sessions
            .session_for_wallet(wallet_id, now_ms, timeout_ms)
            .await?;

        let imported_count = session.with_wallet_session(|wlt_session| {
            let mut count = 0u64;
            for asset in assets {
                if crate::db::wallet_asset_store()
                    .get_owned_asset(wlt_session, &asset.asset_id())?
                    .is_none()
                {
                    count = count.saturating_add(1);
                }
            }
            Ok(count)
        })?;

        #[cfg(test)]
        if let Some(hook) = Self::take_receive_persist_test_hook(wallet_id) {
            match hook {
                ReceivePersistTestHook::AbortBeforePersist => {
                    return Err(WalletError::InvalidConfig(
                        "scan batch persist failpoint enabled".to_string(),
                    ));
                }
                ReceivePersistTestHook::MutateCursorBeforePersist { height, hash } => {
                    session.with_wallet_session(|wlt_session| {
                        crate::db::redb_store::upsert_scan_state(
                            wlt_session,
                            &ScanStatePayload::new(height, hash),
                            z00z_utils::rng::SystemRngProvider,
                        )?;
                        Ok(())
                    })?;
                }
            }
        }

        session.with_wallet_session(|wlt_session| {
            crate::db::wallet_asset_store().persist_scan_batch(
                wlt_session,
                assets,
                expected_resume,
                cursor,
                crate::db::AssetPersistContext {
                    scan_ref,
                    now_ms,
                    ..crate::db::AssetPersistContext::default()
                },
            )
        })?;

        self.sync_claimed_asset_cache(wallet_id).await?;
        Ok(imported_count)
    }

    async fn recv_range_authoritative(
        &self,
        wallet_id: &PersistWalletId,
        chunks: &[ScanChunk],
        requests: &[PaymentRequest],
        max_ckpt: Option<usize>,
        resume: ScanStatePayload,
    ) -> WalletResult<ScanRangeOut> {
        let recv_keys = self.live_receiver_keys(wallet_id).await?;

        let scanner = StealthOutputScanner::from_keys(&recv_keys);
        let mut scanner = scanner;
        for request in requests.iter().filter(|request| !request.is_expired()) {
            scanner.add_request(request);
        }

        let out = match scanner.scan_range(chunks, Some(&resume), max_ckpt) {
            Ok(out) => out,
            Err(err) => {
                let error = WalletError::InvalidConfig(err.to_string());
                self.record_receive_scan_error(wallet_id, &error).await;
                return Err(error);
            }
        };

        let start = recv_range_start(chunks, &resume)?;
        let assets = self.collect_scan_hits(chunks, &scanner, start, out.stat.done_ckpt)?;
        let imported_count = match self
            .persist_scan_batch(
            wallet_id,
            &assets,
            &resume,
            start,
            out.stat.done_ckpt,
            chunks,
            &out.stat.cursor,
        )
        .await
        {
            Ok(imported_count) => imported_count,
            Err(error) => {
                self.record_receive_scan_error(wallet_id, &error).await;
                return Err(error);
            }
        };

        let receive_outcome = Self::build_receive_range_outcome(&resume, &out, imported_count);
        let _ = (
            receive_outcome.done_ckpt,
            receive_outcome.hit_count,
            receive_outcome.resume_height,
            receive_outcome.cursor_height,
        );
        self.record_receive_scan_outcome(wallet_id, receive_outcome.outcome)
            .await;

        Ok(out)
    }

    /// Scan one runtime asset through the shared receiver logic.
    ///
    /// This single-asset surface remains a noncanonical lane. It reuses the
    /// live receiver key derivation and the shared detector, but the canonical
    /// Phase 037 receive path is `recv_range(...)`, not this helper.
    pub async fn scan_asset_report(
        &self,
        wallet_id: &PersistWalletId,
        asset: &Asset,
    ) -> Result<ReceiveReport, ReceiveReject> {
        let recv_keys = self
            .live_receiver_keys(wallet_id)
            .await
            .map_err(|_| ReceiveReject::RuntimeFail)?;
        let scanner = StealthOutputScanner::from_keys(&recv_keys);
        scanner.scan_report(asset)
    }

    /// Scan a checkpoint range through the canonical Phase 037 receive lane.
    ///
    /// This preferred request-aware receive lane derives the live receiver keys, builds the shared
    /// `StealthOutputScanner`, registers request metadata, scans the supplied
    /// chunks, persists detected hits through the owned-asset store in the same
    /// `.wlt` write transaction that advances the wallet-native
    /// `ScanStatePayload` cursor, and keeps single-asset receive helpers
    /// outside this canonical lane. Request-bound inbox ordering now remains a
    /// live wallet-local and off-consensus helper only through
    /// `recv_range_with_inbox(...)`, which feeds metadata into this
    /// authoritative receive lane instead of replacing it.
    /// Optional batching wrappers such as `OptimizedScanner` stay subordinate
    /// to this lane and do not become the canonical receive path in Phase 037.
    /// Plain or card-path helpers are
    /// not an equivalent privacy theorem for this canonical receive lane.
    /// Post-scan exclusivity remains a receiver-secret plus `s_out`
    /// wallet-local accepted-path boundary, not a public trustless theorem.
    pub async fn recv_range(
        &self,
        wallet_id: &PersistWalletId,
        chunks: &[ScanChunk],
        requests: &[PaymentRequest],
        max_ckpt: Option<usize>,
    ) -> WalletResult<ScanRangeOut> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let resume = self.load_scan_state(wallet_id).await?;
            self.recv_range_authoritative(wallet_id, chunks, requests, max_ckpt, resume)
                .await
        }

        #[cfg(target_arch = "wasm32")]
        {
            let _ = (wallet_id, chunks, requests, max_ckpt);
            Err(WalletError::InvalidConfig(
                "wallet range receive is not supported on wasm32".to_string(),
            ))
        }
    }

    /// Validate request-bound inbox metadata and re-enter the canonical receive lane.
    ///
    /// The request-bound inbox is advisory and off-consensus. It stores
    /// validation plus ordering hints only, and wallet mutation happens only by
    /// calling the same authoritative `recv_range(...)` receive lane.
    pub async fn recv_range_with_inbox(
        &self,
        wallet_id: &PersistWalletId,
        chunks: &[ScanChunk],
        requests: &[PaymentRequest],
        inbox: &mut crate::receiver::RequestInbox,
        max_ckpt: Option<usize>,
    ) -> WalletResult<ScanRangeOut> {
        let chain_type = self.resolve_persisted_wallet_chain_type(wallet_id).await?;
        let chain_id = Self::persisted_wallet_chain_id(chain_type);
        let mut pins = self.load_tofu(wallet_id).await?;
        let now_ms = self.require_now_ms()?;
        let range_hint = crate::receiver::RequestRangeHint::from_chunks(chunks);
        let mut first_reject = None;
        let mut approved = Vec::new();

        for request in requests {
            let result =
                crate::receiver::ValidatePaymentRequest::validate_all(request, &mut pins, chain_id);
            let record = inbox.record_result(request, &result, range_hint.clone(), now_ms);
            if record.validation.is_approved() {
                approved.push(request.clone());
            } else if first_reject.is_none() {
                first_reject = Some(record.validation);
            }
        }

        if approved.is_empty() {
            return Err(Self::request_inbox_error(first_reject));
        }

        let ordered = inbox
            .ordered_requests(&approved)
            .into_iter()
            .cloned()
            .collect::<Vec<_>>();

        self.recv_range(wallet_id, chunks, &ordered, max_ckpt).await
    }

    /// Feed worker-fetched evidence into the canonical wallet-local range receive lane.
    ///
    /// Remote chunks, proof hints, and resume hints remain advisory. The wallet
    /// still performs local scan evaluation, asset persistence, and cursor
    /// advancement through the same authoritative `recv_range(...)` core.
    pub async fn recv_range_with_worker(
        &self,
        wallet_id: &PersistWalletId,
        evidence: &crate::chain::RemoteScanEvidence,
        requests: &[PaymentRequest],
        max_ckpt: Option<usize>,
    ) -> WalletResult<ScanRangeOut> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let resume = self.load_scan_state(wallet_id).await?;
            if let Err(error) = Self::validate_worker_evidence(evidence, &resume) {
                self.record_receive_scan_outcome(
                    wallet_id,
                    RuntimeReceiveScanOutcome::WorkerEvidenceRejected,
                )
                .await;
                return Err(error);
            }
            self.recv_range_authoritative(wallet_id, &evidence.chunks, requests, max_ckpt, resume)
                .await
        }

        #[cfg(target_arch = "wasm32")]
        {
            let _ = (wallet_id, evidence, requests, max_ckpt);
            Err(WalletError::InvalidConfig(
                "worker-assisted range receive is not supported on wasm32".to_string(),
            ))
        }
    }

    /// Fetch worker evidence and feed it into the canonical wallet-local range receive lane.
    ///
    /// The worker remains a subordinate fetch-only helper. All wallet-state
    /// mutation, verification, and persistence still happen inside the same
    /// authoritative receive core used by `recv_range(...)`.
    pub async fn recv_range_from_worker<W>(
        &self,
        wallet_id: &PersistWalletId,
        worker: &mut W,
        range: &crate::chain::RemoteScanRange,
        requests: &[PaymentRequest],
        max_ckpt: Option<usize>,
    ) -> WalletResult<ScanRangeOut>
    where
        W: crate::chain::RemoteScanWorker + ?Sized,
    {
        let evidence = worker
            .fetch_range_evidence(range)
            .map_err(Self::map_remote_scan_worker_error)?;
        self.recv_range_with_worker(wallet_id, &evidence, requests, max_ckpt)
            .await
    }

    /// List claimed assets for one wallet from wallet-native state.
    pub async fn list_claimed_assets(
        &self,
        wallet_id: &PersistWalletId,
    ) -> WalletResult<Vec<Asset>> {
        let store = self.wallet_claimed_assets.read().await;
        Ok(store.get(wallet_id).cloned().unwrap_or_default())
    }

    /// List claimed assets across all wallets from wallet-native state.
    pub async fn list_claimed_all(&self) -> WalletResult<Vec<Asset>> {
        let store = self.wallet_claimed_assets.read().await;
        let mut out = Vec::new();
        for rows in store.values() {
            out.extend(rows.iter().cloned());
        }
        Ok(out)
    }

    /// Guarded catalog surface called by `asset.list`.
    ///
    /// Returns deterministic empty pagination until a dedicated catalog lane
    /// lands; live asset authority remains the `.wlt` owned-asset store.
    pub fn list_assets(
        &self,
        wallet_id: &PersistWalletId,
        limit: Option<usize>,
        cursor: Option<String>,
        filter: Option<RuntimeAssetListFilter>,
    ) -> RuntimeListAssetsResponse {
        let _ = (wallet_id, limit, cursor, filter);
        RuntimeListAssetsResponse {
            items: Vec::new(),
            next_cursor: None,
            has_more: false,
            total_count: None,
        }
    }

    /// Guarded asset-op surface called by `asset.merge`.
    ///
    /// Returns deterministic guard data; canonical spend and confirmation stay
    /// on the `wallet.tx.*` plus reconcile lane.
    pub fn merge_assets(
        &self,
        wallet_id: &PersistWalletId,
        asset_ids: Vec<AssetId>,
    ) -> RuntimeMergeAssetsResponse {
        let _ = (wallet_id, asset_ids);
        RuntimeMergeAssetsResponse {
            asset: crate::rpc::types::common::RuntimeAssetRef {
                asset_id: [0u8; 32],
                serial_id: 0,
                symbol: String::new(),
                class: z00z_core::assets::AssetClass::Coin,
            },
            merged_count: 0,
            total_amount: 0,
            tx_id: None,
        }
    }

    /// Guarded metadata surface called by `asset.metadata`.
    ///
    /// Returns deterministic guard metadata when no live asset-definition
    /// registry is attached to this path.
    pub fn get_asset_metadata(&self, asset_id: AssetId) -> RuntimeAssetMetadataResponse {
        let _ = asset_id;
        RuntimeAssetMetadataResponse {
            asset: crate::rpc::types::common::RuntimeAssetRef {
                asset_id: [0u8; 32],
                serial_id: 0,
                symbol: String::new(),
                class: z00z_core::assets::AssetClass::Coin,
            },
            name: String::new(),
            decimals: 0,
            domain_name: String::new(),
            version: 1,
            metadata: Some(std::collections::BTreeMap::from([(
                "error".to_string(),
                "ASSET_METADATA_NOT_AVAILABLE_PHASE044_SERVICE_GUARD".to_string(),
            )])),
        }
    }

    /// Guarded receive surface called by `asset.receive`.
    ///
    /// This remains a restricted reachability surface. It is not the canonical
    /// Phase 037 receive execution path and does not replace `recv_range(...)`.
    pub fn receive_asset(
        &self,
        wallet_id: &PersistWalletId,
        asset_id: AssetId,
    ) -> RuntimeReceiveAssetResponse {
        let _ = (wallet_id, asset_id);
        RuntimeReceiveAssetResponse {
            asset: crate::rpc::types::common::RuntimeAssetRef {
                asset_id: [0u8; 32],
                serial_id: 0,
                symbol: String::new(),
                class: z00z_core::assets::AssetClass::Coin,
            },
            status: "RECEIVE_REJECT_INVALID_PROOF".to_string(),
            owner_handle: String::new(),
            view_key: String::new(),
            expires_at: None,
        }
    }

    /// Guarded asset-op surface called by `asset.send`.
    ///
    /// Returns deterministic guard data; canonical confirmed spend remains the
    /// `wallet.tx.*` plus reconcile lane.
    pub fn send_asset(
        &self,
        wallet_id: &PersistWalletId,
        asset_id: AssetId,
        recipient: String,
        amount: u64,
    ) -> RuntimeSendAssetResponse {
        let _ = (wallet_id, asset_id, recipient, amount);
        RuntimeSendAssetResponse {
            tx_id: crate::rpc::types::common::PersistTxId::new(
                "phase044_service_guard_send_disabled".to_string(),
            ),
            asset: crate::rpc::types::common::RuntimeAssetRef {
                asset_id: [0u8; 32],
                serial_id: 0,
                symbol: String::new(),
                class: z00z_core::assets::AssetClass::Coin,
            },
            owner_handle: String::new(),
            amount: 0,
            recipient: String::new(),
            fee: 0,
            status: "phase044_service_guard_send_disabled".to_string(),
        }
    }

    /// Restricted asset-op surface called by `asset.split`.
    ///
    /// Returns deterministic guard data; it does not claim canonical ledger
    /// mutation authority.
    pub fn split_asset(
        &self,
        wallet_id: &PersistWalletId,
        asset_id: AssetId,
        amounts: Vec<u64>,
    ) -> RuntimeSplitAssetResponse {
        let _ = (wallet_id, asset_id, amounts);
        RuntimeSplitAssetResponse {
            original_asset_id: [0u8; 32],
            splits: Vec::new(),
            tx_id: None,
        }
    }

    /// Restricted asset-op surface called by `asset.staking`.
    ///
    /// Returns deterministic guard data; it does not claim canonical ledger
    /// mutation authority.
    pub fn stake_assets(
        &self,
        wallet_id: &PersistWalletId,
        asset_id: AssetId,
        amount: u64,
    ) -> RuntimeStakeAssetsResponse {
        let _ = (wallet_id, asset_id, amount);
        RuntimeStakeAssetsResponse {
            stake_id: String::new(),
            asset: crate::rpc::types::common::RuntimeAssetRef {
                asset_id: [0u8; 32],
                serial_id: 0,
                symbol: String::new(),
                class: z00z_core::assets::AssetClass::Coin,
            },
            amount: 0,
            start_time: 0,
            end_time: 0,
            apy: 0.0,
        }
    }

    /// Restricted asset-op surface called by `asset.swap`.
    ///
    /// Returns deterministic guard data; it does not claim canonical ledger
    /// mutation authority.
    pub fn swap_assets(
        &self,
        wallet_id: &PersistWalletId,
        from_asset_id: AssetId,
        to_asset_id: AssetId,
        amount: u64,
    ) -> RuntimeSwapAssetsResponse {
        let _ = (wallet_id, from_asset_id, to_asset_id, amount);
        RuntimeSwapAssetsResponse {
            from_asset_id: [0u8; 32],
            from_serial_id: 0,
            from_symbol: String::new(),
            from_class: z00z_core::assets::AssetClass::Coin,
            to_asset_id: [0u8; 32],
            to_serial_id: 0,
            to_symbol: String::new(),
            to_class: z00z_core::assets::AssetClass::Coin,
            from_amount: 0,
            to_amount: 0,
            exchange_rate: 0.0,
            fee: 0,
            tx_id: crate::rpc::types::common::PersistTxId::new(
                "phase044_service_guard_swap_disabled".to_string(),
            ),
        }
    }

    /// Restricted asset-op surface called by `asset.unstaking`.
    ///
    /// Returns deterministic guard data; it does not claim canonical ledger
    /// mutation authority.
    pub fn unstake_assets(
        &self,
        wallet_id: &PersistWalletId,
        stake_id: String,
    ) -> RuntimeUnstakeAssetsResponse {
        let _ = (wallet_id, stake_id);
        RuntimeUnstakeAssetsResponse {
            stake_id: String::new(),
            asset: crate::rpc::types::common::RuntimeAssetRef {
                asset_id: [0u8; 32],
                serial_id: 0,
                symbol: String::new(),
                class: z00z_core::assets::AssetClass::Coin,
            },
            amount: 0,
            reward: 0,
            unstaked_at: 0,
        }
    }

}
