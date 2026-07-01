//! Output scanning primitives for stealth receiver protocol.

use z00z_core::assets::Asset;
use z00z_crypto::Z00ZScalar;
use z00z_utils::logger::{Logger, TracingLogger};

use super::asset_scan_support::{
    make_wallet_output, scan_cached_keys, scan_owned, DetectState, ScanInput,
};
use crate::db::ScanStatePayload;
use crate::{key::ReceiverKeys, receiver::PaymentRequest};

pub use super::asset_scan_types::{
    CacheStats, DetectedAssetPack, DoSMitigation, ReceiveNext, ReceiveReject, ReceiveReport,
    ReceiveStatus, ScanChunk, ScanDecision, ScanRangeErr, ScanRangeOut, ScanRangeStat, ScanResult,
    ScanStrategy, Tag16Cache, Tag16CacheState, Tag16Context, WalletReveal, WalletStealthOutput,
};

/// Wallet-runtime asset adapter for Spec 6 scan entry points.
///
/// Canonical formulas remain owned by `core/stealth/*` and the full-leaf
/// detection authority remains `receiver_scan_leaf`.
pub struct StealthOutputScanner {
    view_sks: Vec<Z00ZScalar>,
    owner_handle: [u8; 32],
    tag16_cache: Tag16Cache,
}

impl Clone for StealthOutputScanner {
    fn clone(&self) -> Self {
        Self {
            view_sks: self
                .view_sks
                .iter()
                .map(Z00ZScalar::dangerous_clone)
                .collect(),
            owner_handle: self.owner_handle,
            tag16_cache: self.tag16_cache.clone(),
        }
    }
}

impl std::fmt::Debug for StealthOutputScanner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StealthOutputScanner")
            .field("view_sks", &"<redacted>")
            .field("owner_handle", &self.owner_handle)
            .field("tag16_cache", &self.tag16_cache)
            .finish()
    }
}

impl StealthOutputScanner {
    /// Construct scanner from receiver keys.
    pub fn from_keys(keys: &ReceiverKeys) -> Self {
        Self {
            view_sks: keys.all_view_sks(),
            owner_handle: keys.owner_handle,
            tag16_cache: Tag16Cache::new(),
        }
    }

    /// Construct scanner from key material.
    pub fn new(view_sk: Z00ZScalar, owner_handle: [u8; 32]) -> Self {
        Self {
            view_sks: vec![view_sk],
            owner_handle,
            tag16_cache: Tag16Cache::new(),
        }
    }

    /// Insert one precomputed tag16 candidate context.
    pub fn add_tag_context(&mut self, tag16: u16, context: Tag16Context) {
        self.tag16_cache.insert(tag16, context);
    }

    /// Register one active request id.
    pub fn add_active_req(&mut self, req_id: [u8; 32]) {
        self.tag16_cache.add_active_request(req_id);
    }

    /// Register one active payment request as liveness metadata only.
    /// This does not materialize the concrete `Tag16Context` values required
    /// for a strict tag-only ownership claim.
    pub fn add_request(&mut self, request: &PaymentRequest) {
        self.tag16_cache.add_request(request);
    }

    /// Materialize a complete concrete tag-context set and authorize strict tag-only mode.
    pub fn materialize_complete_tag_contexts<I>(&mut self, contexts: I)
    where
        I: IntoIterator<Item = (u16, Tag16Context)>,
    {
        self.tag16_cache.materialize_complete_tag_contexts(contexts);
    }

    /// Scan one checkpoint worth of asset leaves.
    pub fn scan_checkpoint(&self, leaves: &[Asset]) -> Vec<WalletStealthOutput> {
        let mut found = Vec::new();

        for leaf in leaves {
            if let ScanResult::Mine { wallet_output } = self.scan_leaf(leaf) {
                found.push(*wallet_output);
            }
        }

        found
    }

    /// Scan checkpoint with DoS limits.
    pub fn scan_with_dos_protection(
        &self,
        leaves: &[Asset],
        mitigation: &DoSMitigation,
    ) -> Vec<WalletStealthOutput> {
        let mut found = Vec::new();
        let mut candidate_count = 0usize;
        let mut decrypt_count = 0usize;

        for leaf in leaves {
            if leaf_fields(leaf).is_none() {
                continue;
            }

            candidate_count += 1;
            match mitigation.should_try_decrypt(candidate_count, decrypt_count) {
                ScanDecision::Proceed => {}
                ScanDecision::Defer { .. } => {
                    let _ = mitigation.defer_threshold;
                    continue;
                }
            }

            decrypt_count += 1;
            if let ScanResult::Mine { wallet_output } = self.scan_leaf(leaf) {
                found.push(*wallet_output);
            }
        }

        found
    }

    /// Scan a checkpoint range over the existing single-checkpoint primitive.
    /// Remote worker proof or resume hints must be normalized before calling
    /// this helper; only chunk data plus the wallet-local resume state are
    /// authoritative here.
    pub fn scan_range(
        &self,
        chunks: &[ScanChunk],
        resume: Option<&ScanStatePayload>,
        max_ckpt: Option<usize>,
    ) -> Result<ScanRangeOut, ScanRangeErr> {
        let start = range_start(chunks, resume)?;
        let take = max_ckpt.unwrap_or(usize::MAX);
        let end = start.saturating_add(take).min(chunks.len());

        let total_ckpt = chunks.len() as u64;
        let total_leaf = chunks.iter().map(|chunk| chunk.leaves.len() as u64).sum();
        let mut outputs = Vec::new();
        let mut done_ckpt = 0u64;
        let mut done_leaf = 0u64;
        let mut reject_cnt = 0u64;
        let mut cursor = resume
            .cloned()
            .unwrap_or_else(|| ScanStatePayload::new(0, Vec::new()));

        for chunk in &chunks[start..end] {
            for leaf in &chunk.leaves {
                match self.scan_leaf(leaf) {
                    ScanResult::Mine { wallet_output } => outputs.push(*wallet_output),
                    ScanResult::NotMine | ScanResult::MaybeMine { .. } => {
                        reject_cnt = reject_cnt.saturating_add(1);
                    }
                }
                done_leaf = done_leaf.saturating_add(1);
            }

            cursor.advance(chunk.height, chunk.hash.clone());
            done_ckpt = done_ckpt.saturating_add(1);
        }

        let found_cnt = outputs.len() as u64;
        Ok(ScanRangeOut {
            outputs,
            stat: ScanRangeStat {
                cursor,
                done_ckpt,
                total_ckpt,
                done_leaf,
                total_leaf,
                found_cnt,
                reject_cnt,
            },
        })
    }

    /// Scan one asset leaf.
    pub fn scan_leaf(&self, leaf: &Asset) -> ScanResult {
        let (r_pub, owner_tag, enc_pack) = match leaf_fields(leaf) {
            Some(fields) => fields,
            None => {
                Logger::debug(&TracingLogger, "scan_leaf reject: missing stealth fields");
                return ScanResult::NotMine;
            }
        };

        if let Some(result) = self.scan_with_tag(leaf) {
            if matches!(result, ScanResult::Mine { .. }) {
                return result;
            }

            let direct = self.scan_direct(leaf, r_pub, owner_tag, enc_pack);
            if matches!(direct, ScanResult::Mine { .. }) {
                return direct;
            }

            return result;
        }

        self.scan_direct(leaf, r_pub, owner_tag, enc_pack)
    }

    /// Scan one runtime asset through the report-first receive contract.
    /// This returns ownership-detection and receive-classification status only;
    /// downstream import and proof verification stay outside the scanner.
    pub fn scan_report(&self, leaf: &Asset) -> Result<ReceiveReport, ReceiveReject> {
        leaf.validate_stealth_consistency()
            .map_err(|_| ReceiveReject::InvalidInput)?;
        Ok(self.scan_leaf(leaf).recv_report())
    }

    /// Scan one runtime asset using strict tag16 prefilter only.
    /// This path never falls back to direct scan; callers must materialize the
    /// complete concrete tag-context set with
    /// `materialize_complete_tag_contexts(...)` before strict tag-only mode is
    /// authorized. `add_request(...)` alone never authorizes ownership.
    pub fn scan_leaf_tag_only(&self, leaf: &Asset) -> ScanResult {
        if !self.tag16_cache.is_complete() {
            return ScanResult::NotMine;
        }

        let tag16 = match leaf.tag16 {
            Some(value) => value,
            None => return ScanResult::NotMine,
        };

        if !self.tag16_cache.contains(tag16) {
            return ScanResult::NotMine;
        }

        if let Some(contexts) = self.tag16_cache.get_contexts(tag16) {
            return self.handle_decrypt_post_tag16_match(leaf, contexts);
        }

        ScanResult::NotMine
    }

    /// Handle decryption attempts after a tag16 prefilter match.
    pub(crate) fn handle_decrypt_post_tag16_match(
        &self,
        leaf: &Asset,
        contexts: &[Tag16Context],
    ) -> ScanResult {
        let (r_pub, owner_tag, enc_pack) = match leaf_fields(leaf) {
            Some(fields) => fields,
            None => return ScanResult::NotMine,
        };
        let leaf_ad_id = match leaf.leaf_ad_id() {
            Ok(value) => value,
            Err(_) => return invalid_owned_output_result(leaf),
        };
        let mut c_amount = [0u8; 32];
        c_amount.copy_from_slice(leaf.commitment.as_bytes());
        let input = ScanInput {
            serial_id: leaf.serial_id,
            leaf_ad_id: &leaf_ad_id,
            r_pub,
            owner_tag,
            c_amount: &c_amount,
            enc_pack,
            tag16: leaf.tag16,
        };
        let cached = scan_cached_keys(
            &self.owner_handle,
            &input,
            contexts
                .iter()
                .map(|context| (context.k_dh, context.req_id)),
        );

        match cached.state {
            DetectState::Mine(pack) => ScanResult::Mine {
                wallet_output: Box::new(make_wallet_output(leaf, &pack, r_pub, owner_tag)),
            },
            DetectState::Invalid(_) => invalid_owned_output_result(leaf),
            DetectState::NotMine => ScanResult::MaybeMine {
                tag16_match: true,
                m1_failed: !cached.owner_hit,
            },
        }
    }

    /// Determine background scan strategy by current cache pressure and completeness.
    pub fn background_scan_strategy(&self) -> ScanStrategy {
        let cache_size = self.tag16_cache.size();
        if self.tag16_cache.is_complete() && cache_size > 10_000 {
            ScanStrategy::TagFilterOnly
        } else if cache_size > 1_000 {
            ScanStrategy::Balanced
        } else {
            ScanStrategy::FullScan
        }
    }

    fn scan_with_tag(&self, leaf: &Asset) -> Option<ScanResult> {
        let tag16 = leaf.tag16?;
        if !self.tag16_cache.contains(tag16) {
            return None;
        }

        let contexts = self.tag16_cache.get_contexts(tag16)?;
        Some(self.handle_decrypt_post_tag16_match(leaf, contexts))
    }

    fn scan_direct(
        &self,
        leaf: &Asset,
        r_pub: &[u8; 32],
        owner_tag: &[u8; 32],
        enc_pack: &z00z_crypto::ZkPackEncrypted,
    ) -> ScanResult {
        let leaf_ad_id = match leaf.leaf_ad_id() {
            Ok(value) => value,
            Err(_) => return invalid_owned_output_result(leaf),
        };
        let mut c_amount = [0u8; 32];
        c_amount.copy_from_slice(leaf.commitment.as_bytes());
        let input = ScanInput {
            serial_id: leaf.serial_id,
            leaf_ad_id: &leaf_ad_id,
            r_pub,
            owner_tag,
            c_amount: &c_amount,
            enc_pack,
            tag16: leaf.tag16,
        };

        match scan_owned(
            self.view_sks.iter(),
            &self.owner_handle,
            &input,
            self.tag16_cache.active_requests().copied(),
        ) {
            DetectState::Mine(pack) => ScanResult::Mine {
                wallet_output: Box::new(make_wallet_output(leaf, &pack, r_pub, owner_tag)),
            },
            DetectState::NotMine => ScanResult::NotMine,
            DetectState::Invalid(_) => invalid_owned_output_result(leaf),
        }
    }
}

fn range_start(
    chunks: &[ScanChunk],
    resume: Option<&ScanStatePayload>,
) -> Result<usize, ScanRangeErr> {
    let Some(resume) = resume else {
        return Ok(0);
    };

    if resume.is_origin() {
        return Ok(0);
    }

    let Some(pos) = chunks
        .iter()
        .position(|chunk| resume.matches_chunk(chunk.height, &chunk.hash))
    else {
        return Err(ScanRangeErr::BadCursor);
    };

    Ok(pos.saturating_add(1))
}

/// Example helper that scans one prepared output leaf.
#[expect(
    dead_code,
    reason = "Public example helper stays available for external scan examples."
)]
pub fn example_scan_tx_output(scanner: &StealthOutputScanner, leaf: &Asset) -> ScanResult {
    scanner.scan_leaf(leaf)
}

fn leaf_fields(leaf: &Asset) -> Option<(&[u8; 32], &[u8; 32], &z00z_crypto::ZkPackEncrypted)> {
    match (&leaf.r_pub, &leaf.owner_tag, &leaf.enc_pack) {
        (Some(r_pub), Some(owner_tag), Some(enc_pack)) => Some((r_pub, owner_tag, enc_pack)),
        _ => None,
    }
}

fn invalid_owned_output_result(leaf: &Asset) -> ScanResult {
    ScanResult::MaybeMine {
        tag16_match: leaf.tag16.is_some(),
        m1_failed: false,
    }
}

#[cfg(test)]
#[path = "test_wallet_asset_scanner.rs"]
mod test_wallet_asset_scanner;
