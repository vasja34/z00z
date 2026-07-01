use super::objects::write_object_by_id;
use super::queries::{read_objects_by_index, validate_object_index_rows};
use super::{
    allocate_object_id, commit_redb_write_txn_flush, decode_bincode, encode_bincode,
    encrypt_object_record, read_object_by_id, read_scan_state, write_object,
    write_object_with_indexes, AssetSeenRef, ConfirmRef, IndexTable, IndexUpdate, ObjectKindId,
    OwnedAssetPayload, OwnedAssetPolicy, OwnedAssetSource, OwnedAssetStatus, ReceiveRef, ScanRef,
    ScanStatePayload, WalletError, WalletResult, WalletSession, OBJECTS_TABLE,
    PAYLOAD_VERSION_OWNED_ASSET,
};
use crate::db::index_codecs::{encode_index_semantic_kv, IndexValueBytes};
use crate::rpc::types::common::PersistTxId;
use z00z_core::{Asset, AssetWire};
use z00z_utils::rng::SystemRngProvider;

const OWNED_ASSET_QUERY_BATCH: usize = 256;

#[derive(Debug, Clone, Default)]
pub(crate) struct AssetFilter {
    pub(crate) account_id: Option<u128>,
    pub(crate) asset_definition_id: Option<[u8; 32]>,
    pub(crate) status: Option<OwnedAssetStatus>,
    pub(crate) source: Option<OwnedAssetSource>,
    pub(crate) tx_id: Option<PersistTxId>,
    pub(crate) min_height: Option<u64>,
    pub(crate) max_height: Option<u64>,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct AssetPersistContext {
    pub(crate) scan_ref: Option<ScanRef>,
    pub(crate) receive_ref: Option<ReceiveRef>,
    pub(crate) confirmation_ref: Option<ConfirmRef>,
    pub(crate) now_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PutAssetOutcome {
    Inserted { object_id: u128 },
    AlreadyPresent { object_id: u128 },
}

#[derive(Debug, Clone, Default)]
pub(crate) struct AssetPage {
    pub(crate) items: Vec<OwnedAssetPayload>,
}

pub(crate) trait WalletAssetStore {
    fn put_owned_asset(
        &self,
        session: &WalletSession,
        asset: Asset,
        source: OwnedAssetSource,
        context: AssetPersistContext,
    ) -> WalletResult<PutAssetOutcome>;

    fn put_owned_assets_batch(
        &self,
        session: &WalletSession,
        assets: &[Asset],
        source: OwnedAssetSource,
        context: AssetPersistContext,
    ) -> WalletResult<()>;

    fn get_owned_asset(
        &self,
        session: &WalletSession,
        asset_id: &[u8; 32],
    ) -> WalletResult<Option<OwnedAssetPayload>>;

    fn list_owned_assets(
        &self,
        session: &WalletSession,
        filter: AssetFilter,
        cursor: Option<String>,
        limit: usize,
    ) -> WalletResult<AssetPage>;

    fn list_spendable_assets(
        &self,
        session: &WalletSession,
        asset_definition_id: Option<[u8; 32]>,
        limit: usize,
    ) -> WalletResult<Vec<OwnedAssetPayload>>;

    fn reserve_asset_inputs(
        &self,
        session: &WalletSession,
        tx_id: &PersistTxId,
        asset_ids: &[[u8; 32]],
    ) -> WalletResult<()>;

    fn release_asset_reservation(
        &self,
        session: &WalletSession,
        tx_id: &PersistTxId,
    ) -> WalletResult<()>;

    fn confirm_asset_spend(
        &self,
        session: &WalletSession,
        tx_id: &PersistTxId,
        spent_ids: &[[u8; 32]],
        new_outputs: &[Asset],
        new_output_source: OwnedAssetSource,
    ) -> WalletResult<()>;

    fn replace_assets_for_restore(
        &self,
        session: &WalletSession,
        assets: &[Asset],
    ) -> WalletResult<()>;

    fn replace_payloads_for_restore(
        &self,
        session: &WalletSession,
        payloads: &[OwnedAssetPayload],
    ) -> WalletResult<()>;

    /// Persist one wallet-local scan batch.
    /// Worker-fed evidence cannot call this write path directly; only the
    /// canonical wallet receive lane may translate evidence into persisted
    /// owned assets plus cursor updates.
    fn persist_scan_batch(
        &self,
        session: &WalletSession,
        assets: &[Asset],
        expected_resume: &ScanStatePayload,
        cursor: &ScanStatePayload,
        context: AssetPersistContext,
    ) -> WalletResult<()>;
}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct RedbWalletAssetStore;

pub(crate) fn wallet_asset_store() -> RedbWalletAssetStore {
    RedbWalletAssetStore
}

impl WalletAssetStore for RedbWalletAssetStore {
    fn put_owned_asset(
        &self,
        session: &WalletSession,
        asset: Asset,
        source: OwnedAssetSource,
        context: AssetPersistContext,
    ) -> WalletResult<PutAssetOutcome> {
        asset.validate().map_err(|error| {
            WalletError::InvalidConfig(format!("invalid owned asset candidate: {error}"))
        })?;

        let candidate = build_owned_asset_payload(session, asset, source, &context)?;
        let index_updates = owned_asset_index_updates(&candidate, None)?;

        if let Some(object_id) = self.get_owned_asset_object_id(session, &candidate.asset_id)? {
            let stored = decode_owned_asset_payload(session, object_id)?;
            if same_insert_shape(&stored, &candidate) {
                return Ok(PutAssetOutcome::AlreadyPresent { object_id });
            }
            return Err(WalletError::InvalidConfig(
                "duplicate owned asset id conflicts with stored payload".to_string(),
            ));
        }

        let object_id = write_object(
            session,
            ObjectKindId::OwnedAsset as u8,
            PAYLOAD_VERSION_OWNED_ASSET,
            encode_bincode(&candidate)?,
            &index_updates,
            SystemRngProvider,
        )?;

        Ok(PutAssetOutcome::Inserted { object_id })
    }

    fn put_owned_assets_batch(
        &self,
        session: &WalletSession,
        assets: &[Asset],
        source: OwnedAssetSource,
        context: AssetPersistContext,
    ) -> WalletResult<()> {
        let mut candidates = std::collections::BTreeMap::new();
        for asset in assets {
            asset.validate().map_err(|error| {
                WalletError::InvalidConfig(format!("invalid owned asset candidate: {error}"))
            })?;

            let candidate = build_owned_asset_payload(session, asset.clone(), source, &context)?;
            match candidates.entry(candidate.asset_id) {
                std::collections::btree_map::Entry::Vacant(entry) => {
                    entry.insert(candidate);
                }
                std::collections::btree_map::Entry::Occupied(entry) => {
                    if !same_insert_shape(entry.get(), &candidate) {
                        return Err(WalletError::InvalidConfig(
                            "duplicate owned asset id conflicts inside one import batch"
                                .to_string(),
                        ));
                    }
                }
            }
        }

        let mut pending = Vec::new();
        for candidate in candidates.into_values() {
            if let Some(object_id) = self.get_owned_asset_object_id(session, &candidate.asset_id)? {
                let stored = decode_owned_asset_payload(session, object_id)?;
                if same_insert_shape(&stored, &candidate) {
                    continue;
                }
                return Err(WalletError::InvalidConfig(
                    "duplicate owned asset id conflicts with stored payload".to_string(),
                ));
            }
            pending.push(candidate);
        }

        if pending.is_empty() {
            return Ok(());
        }

        let write_txn = session
            .db
            .begin_write()
            .map_err(|e| WalletError::InvalidConfig(format!("redb begin_write failed: {e}")))?;

        for candidate in &pending {
            insert_owned_asset(session, &write_txn, candidate)?;
        }

        commit_redb_write_txn_flush(session, write_txn)?;
        Ok(())
    }

    fn get_owned_asset(
        &self,
        session: &WalletSession,
        asset_id: &[u8; 32],
    ) -> WalletResult<Option<OwnedAssetPayload>> {
        self.get_owned_asset_object_id(session, asset_id)?
            .map(|object_id| decode_owned_asset_payload(session, object_id))
            .transpose()
    }

    fn list_owned_assets(
        &self,
        session: &WalletSession,
        filter: AssetFilter,
        cursor: Option<String>,
        limit: usize,
    ) -> WalletResult<AssetPage> {
        if limit == 0 {
            return Ok(AssetPage::default());
        }

        let (table, semantic) = choose_asset_query(&filter)?;
        let object_ids = collect_object_ids(session, table, semantic.as_deref())?;

        let mut payloads = Vec::with_capacity(object_ids.len());
        for object_id in object_ids {
            let payload = decode_owned_asset_payload(session, object_id)?;
            if payload_matches_filter(&payload, &filter) {
                payloads.push(payload);
            }
        }

        payloads.sort_by_key(|payload| payload.asset_id);

        let start = parse_offset_cursor(cursor.as_deref())?;
        if start >= payloads.len() {
            return Ok(AssetPage { items: Vec::new() });
        }

        let end = start.saturating_add(limit).min(payloads.len());

        Ok(AssetPage {
            items: payloads[start..end].to_vec(),
        })
    }

    fn list_spendable_assets(
        &self,
        session: &WalletSession,
        asset_definition_id: Option<[u8; 32]>,
        limit: usize,
    ) -> WalletResult<Vec<OwnedAssetPayload>> {
        let page = self.list_owned_assets(
            session,
            AssetFilter {
                asset_definition_id,
                status: Some(OwnedAssetStatus::Spendable),
                ..AssetFilter::default()
            },
            None,
            limit,
        )?;

        Ok(page
            .items
            .into_iter()
            .filter(|payload| !payload.policy.frozen && !payload.policy.manual_review)
            .collect())
    }

    fn reserve_asset_inputs(
        &self,
        session: &WalletSession,
        tx_id: &PersistTxId,
        asset_ids: &[[u8; 32]],
    ) -> WalletResult<()> {
        let now_ms = session.time_provider.compat_unix_timestamp_millis();
        let mut rewrites = Vec::new();
        for asset_id in asset_ids {
            let object_id = self
                .get_owned_asset_object_id(session, asset_id)?
                .ok_or_else(|| WalletError::InvalidConfig("owned asset not found".to_string()))?;
            let mut payload = decode_owned_asset_payload(session, object_id)?;
            if payload.status != OwnedAssetStatus::Spendable || payload.spend_ref.is_some() {
                return Err(WalletError::InvalidConfig(
                    "owned asset is not available for reservation".to_string(),
                ));
            }
            payload.status = OwnedAssetStatus::PendingSpend;
            payload.spend_ref = Some(tx_id.clone());
            payload.last_updated_ms = now_ms;
            payload.checksum = Some(payload.compute_checksum());
            rewrites.push((object_id, payload));
        }

        if rewrites.is_empty() {
            return Ok(());
        }

        let write_txn = session
            .db
            .begin_write()
            .map_err(|e| WalletError::InvalidConfig(format!("redb begin_write failed: {e}")))?;

        for (object_id, payload) in &rewrites {
            rewrite_owned_asset_with_txn(session, &write_txn, *object_id, payload)?;
        }

        commit_redb_write_txn_flush(session, write_txn)?;

        Ok(())
    }

    fn release_asset_reservation(
        &self,
        session: &WalletSession,
        tx_id: &PersistTxId,
    ) -> WalletResult<()> {
        let object_ids = collect_object_ids(
            session,
            IndexTable::OwnedAssetByTx,
            Some(encode_owned_asset_by_tx(tx_id)?.as_slice()),
        )?;
        let now_ms = session.time_provider.compat_unix_timestamp_millis();
        let mut rewrites = Vec::new();

        for object_id in object_ids {
            let mut payload = decode_owned_asset_payload(session, object_id)?;
            if payload.status == OwnedAssetStatus::PendingSpend
                && payload.spend_ref.as_ref() == Some(tx_id)
            {
                payload.status = OwnedAssetStatus::Spendable;
                payload.spend_ref = None;
                payload.last_updated_ms = now_ms;
                payload.checksum = Some(payload.compute_checksum());
                rewrites.push((object_id, payload));
            }
        }

        if rewrites.is_empty() {
            return Ok(());
        }

        let write_txn = session
            .db
            .begin_write()
            .map_err(|e| WalletError::InvalidConfig(format!("redb begin_write failed: {e}")))?;

        for (object_id, payload) in &rewrites {
            rewrite_owned_asset_with_txn(session, &write_txn, *object_id, payload)?;
        }

        commit_redb_write_txn_flush(session, write_txn)?;

        Ok(())
    }

    fn confirm_asset_spend(
        &self,
        session: &WalletSession,
        tx_id: &PersistTxId,
        spent_ids: &[[u8; 32]],
        new_outputs: &[Asset],
        new_output_source: OwnedAssetSource,
    ) -> WalletResult<()> {
        let now_ms = session.time_provider.compat_unix_timestamp_millis();
        let spent_id_set = spent_ids
            .iter()
            .copied()
            .collect::<std::collections::BTreeSet<_>>();
        let local_tx_assets = self.list_owned_assets(
            session,
            AssetFilter {
                tx_id: Some(tx_id.clone()),
                ..AssetFilter::default()
            },
            None,
            usize::MAX,
        )?;

        for payload in &local_tx_assets.items {
            if !spent_id_set.contains(&payload.asset_id) {
                return Err(WalletError::InvalidConfig(
                    "local reserved asset missing from confirmation inputs".to_string(),
                ));
            }
        }

        let mut spent_rewrites = Vec::new();

        for asset_id in spent_ids {
            let Some(object_id) = self.get_owned_asset_object_id(session, asset_id)? else {
                continue;
            };
            let mut payload = decode_owned_asset_payload(session, object_id)?;
            match payload.status {
                OwnedAssetStatus::PendingSpend if payload.spend_ref.as_ref() == Some(tx_id) => {
                    payload.status = OwnedAssetStatus::Spent;
                }
                OwnedAssetStatus::Spent if payload.spend_ref.as_ref() == Some(tx_id) => {
                    continue;
                }
                _ => {
                    return Err(WalletError::InvalidConfig(
                        "owned asset spend confirmation status mismatch".to_string(),
                    ));
                }
            }
            payload.spend_ref = Some(tx_id.clone());
            payload.last_updated_ms = now_ms;
            payload.checksum = Some(payload.compute_checksum());
            spent_rewrites.push((object_id, payload));
        }

        let mut output_candidates = std::collections::BTreeMap::new();
        for asset in new_outputs {
            asset.validate().map_err(|error| {
                WalletError::InvalidConfig(format!("invalid owned asset candidate: {error}"))
            })?;

            let candidate = build_owned_asset_payload(
                session,
                asset.clone(),
                new_output_source,
                &AssetPersistContext {
                    now_ms,
                    ..AssetPersistContext::default()
                },
            )?;

            match output_candidates.entry(candidate.asset_id) {
                std::collections::btree_map::Entry::Vacant(entry) => {
                    entry.insert(candidate);
                }
                std::collections::btree_map::Entry::Occupied(entry) => {
                    if !same_insert_shape(entry.get(), &candidate) {
                        return Err(WalletError::InvalidConfig(
                            "duplicate owned asset id conflicts inside one confirmation batch"
                                .to_string(),
                        ));
                    }
                }
            }
        }

        let mut pending_inserts = Vec::new();
        for candidate in output_candidates.into_values() {
            if let Some(object_id) = self.get_owned_asset_object_id(session, &candidate.asset_id)? {
                let stored = decode_owned_asset_payload(session, object_id)?;
                if same_insert_shape(&stored, &candidate) {
                    continue;
                }
                return Err(WalletError::InvalidConfig(
                    "duplicate owned asset id conflicts with stored payload".to_string(),
                ));
            }
            pending_inserts.push(candidate);
        }

        let write_txn = session
            .db
            .begin_write()
            .map_err(|e| WalletError::InvalidConfig(format!("redb begin_write failed: {e}")))?;

        for (object_id, payload) in &spent_rewrites {
            rewrite_owned_asset_with_txn(session, &write_txn, *object_id, payload)?;
        }

        for candidate in &pending_inserts {
            insert_owned_asset(session, &write_txn, candidate)?;
        }

        commit_redb_write_txn_flush(session, write_txn)?;

        Ok(())
    }

    fn replace_assets_for_restore(
        &self,
        session: &WalletSession,
        assets: &[Asset],
    ) -> WalletResult<()> {
        let now_ms = session.time_provider.compat_unix_timestamp_millis();
        let mut desired_by_id = std::collections::BTreeMap::new();
        for asset in assets {
            asset.validate().map_err(|error| {
                WalletError::InvalidConfig(format!("invalid restore owned asset: {error}"))
            })?;
            let asset_id = asset.asset_id();
            if desired_by_id.insert(asset_id, asset.clone()).is_some() {
                return Err(WalletError::InvalidConfig(
                    "duplicate owned asset id in restore set".to_string(),
                ));
            }
        }

        let existing = self.list_owned_assets(session, AssetFilter::default(), None, usize::MAX)?;

        for payload in existing.items {
            let Some(asset) = desired_by_id.remove(&payload.asset_id) else {
                let object_id = self
                    .get_owned_asset_object_id(session, &payload.asset_id)?
                    .ok_or_else(|| {
                        WalletError::InvalidConfig(
                            "owned asset object id missing during restore replace".to_string(),
                        )
                    })?;
                let mut archived = payload;
                archived.status = OwnedAssetStatus::Archived;
                archived.spend_ref = None;
                archived.last_updated_ms = now_ms;
                archived.checksum = Some(archived.compute_checksum());
                rewrite_owned_asset(session, object_id, &archived)?;
                continue;
            };

            let object_id = self
                .get_owned_asset_object_id(session, &payload.asset_id)?
                .ok_or_else(|| {
                    WalletError::InvalidConfig(
                        "owned asset object id missing during restore update".to_string(),
                    )
                })?;
            let updated = build_owned_asset_payload(
                session,
                asset,
                OwnedAssetSource::Restore,
                &AssetPersistContext {
                    now_ms,
                    ..AssetPersistContext::default()
                },
            )?;
            rewrite_owned_asset(session, object_id, &updated)?;
        }

        for (_, asset) in desired_by_id {
            let _ = self.put_owned_asset(
                session,
                asset,
                OwnedAssetSource::Restore,
                AssetPersistContext {
                    now_ms,
                    ..AssetPersistContext::default()
                },
            )?;
        }

        Ok(())
    }

    fn replace_payloads_for_restore(
        &self,
        session: &WalletSession,
        payloads: &[OwnedAssetPayload],
    ) -> WalletResult<()> {
        let mut desired_by_id = std::collections::BTreeMap::new();
        for payload in payloads {
            let mut canonical = payload.clone().migrate_to_current()?;
            canonical.checksum = Some(canonical.compute_checksum());
            let _ = canonical.validate_invariants()?;
            if canonical.wallet_id != session.opened.wallet_id {
                return Err(WalletError::InvalidConfig(
                    "restore payload wallet id mismatch".to_string(),
                ));
            }
            if desired_by_id
                .insert(canonical.asset_id, canonical)
                .is_some()
            {
                return Err(WalletError::InvalidConfig(
                    "duplicate owned asset id in restore set".to_string(),
                ));
            }
        }

        let now_ms = session.time_provider.compat_unix_timestamp_millis();
        let existing = self.list_owned_assets(session, AssetFilter::default(), None, usize::MAX)?;

        for payload in existing.items {
            let Some(next_payload) = desired_by_id.remove(&payload.asset_id) else {
                let object_id = self
                    .get_owned_asset_object_id(session, &payload.asset_id)?
                    .ok_or_else(|| {
                        WalletError::InvalidConfig(
                            "owned asset object id missing during restore replace".to_string(),
                        )
                    })?;
                let mut archived = payload;
                archived.status = OwnedAssetStatus::Archived;
                archived.spend_ref = None;
                archived.last_updated_ms = now_ms;
                archived.checksum = Some(archived.compute_checksum());
                rewrite_owned_asset(session, object_id, &archived)?;
                continue;
            };

            let object_id = self
                .get_owned_asset_object_id(session, &payload.asset_id)?
                .ok_or_else(|| {
                    WalletError::InvalidConfig(
                        "owned asset object id missing during restore update".to_string(),
                    )
                })?;
            rewrite_owned_asset(session, object_id, &next_payload)?;
        }

        if desired_by_id.is_empty() {
            return Ok(());
        }

        let write_txn = session
            .db
            .begin_write()
            .map_err(|e| WalletError::InvalidConfig(format!("redb begin_write failed: {e}")))?;

        for payload in desired_by_id.into_values() {
            insert_owned_asset(session, &write_txn, &payload)?;
        }

        commit_redb_write_txn_flush(session, write_txn)?;
        Ok(())
    }

    fn persist_scan_batch(
        &self,
        session: &WalletSession,
        assets: &[Asset],
        expected_resume: &ScanStatePayload,
        cursor: &ScanStatePayload,
        context: AssetPersistContext,
    ) -> WalletResult<()> {
        let current =
            read_scan_state(session)?.unwrap_or_else(|| ScanStatePayload::new(0, Vec::new()));
        let current_matches_expected = if expected_resume.is_origin() {
            current.is_origin()
        } else {
            current.matches_chunk(expected_resume.height(), &expected_resume.last_scanned_hash)
        };

        if !current_matches_expected {
            return Err(WalletError::InvalidConfig(
                "scan state changed during receive persistence".to_string(),
            ));
        }

        let mut candidates = std::collections::BTreeMap::new();
        for asset in assets {
            let candidate = build_owned_asset_payload(
                session,
                asset.clone(),
                OwnedAssetSource::Scan,
                &context,
            )?;
            match candidates.entry(candidate.asset_id) {
                std::collections::btree_map::Entry::Vacant(entry) => {
                    entry.insert(candidate);
                }
                std::collections::btree_map::Entry::Occupied(entry) => {
                    if !same_owned_asset_wire(entry.get(), &candidate) {
                        return Err(WalletError::InvalidConfig(
                            "duplicate scan asset id conflicts inside one receive batch"
                                .to_string(),
                        ));
                    }
                }
            }
        }

        let mut pending = Vec::new();
        for candidate in candidates.into_values() {
            if let Some(object_id) = self.get_owned_asset_object_id(session, &candidate.asset_id)? {
                let stored = decode_owned_asset_payload(session, object_id)?;
                if !same_insert_shape(&stored, &candidate) {
                    return Err(WalletError::InvalidConfig(
                        "duplicate scan asset id conflicts with stored payload".to_string(),
                    ));
                }
                continue;
            }
            pending.push(candidate);
        }

        let write_txn = session
            .db
            .begin_write()
            .map_err(|e| WalletError::InvalidConfig(format!("redb begin_write failed: {e}")))?;

        for candidate in &pending {
            insert_owned_asset(session, &write_txn, candidate)?;
        }

        super::mutations::upsert_scan_state_with_txn(
            session,
            &write_txn,
            encode_bincode(cursor)?,
            SystemRngProvider,
        )?;

        commit_redb_write_txn_flush(session, write_txn)?;
        Ok(())
    }
}

impl RedbWalletAssetStore {
    fn get_owned_asset_object_id(
        &self,
        session: &WalletSession,
        asset_id: &[u8; 32],
    ) -> WalletResult<Option<u128>> {
        let matches = collect_object_ids(
            session,
            IndexTable::OwnedAssetById,
            Some(encode_owned_asset_by_id(asset_id)?.as_slice()),
        )?;
        match matches.as_slice() {
            [] => Ok(None),
            [object_id] => Ok(Some(*object_id)),
            _ => Err(WalletError::InvalidConfig(
                "owned asset id resolved to multiple object ids".to_string(),
            )),
        }
    }
}

pub(super) fn find_owned_asset_object_id(
    session: &WalletSession,
    asset_id: &[u8; 32],
) -> WalletResult<Option<u128>> {
    wallet_asset_store().get_owned_asset_object_id(session, asset_id)
}

pub(super) fn collect_object_ids(
    session: &WalletSession,
    table: IndexTable,
    semantic: Option<&[u8]>,
) -> WalletResult<Vec<u128>> {
    let mut cursor = None;
    let mut out = Vec::new();
    let semantic_prefix = semantic.unwrap_or(b"");

    loop {
        let page = match read_objects_by_index(
            session,
            table,
            semantic_prefix,
            OWNED_ASSET_QUERY_BATCH,
            cursor.clone(),
        ) {
            Ok(page) => page,
            Err(WalletError::InvalidConfig(message))
                if message.contains(table.store_name()) && message.contains("does not exist") =>
            {
                return Ok(Vec::new());
            }
            Err(error) => return Err(error),
        };
        out.extend(page.object_ids);
        if !page.has_more {
            break;
        }
        cursor = page.next_cursor;
    }

    Ok(out)
}

fn choose_asset_query(filter: &AssetFilter) -> WalletResult<(IndexTable, Option<Vec<u8>>)> {
    if let Some(tx_id) = filter.tx_id.as_ref() {
        return Ok((
            IndexTable::OwnedAssetByTx,
            Some(encode_owned_asset_by_tx(tx_id)?),
        ));
    }

    if let (Some(asset_definition_id), Some(status)) = (filter.asset_definition_id, filter.status) {
        return Ok((
            IndexTable::OwnedAssetByDefStatus,
            Some(encode_owned_asset_def_status(&asset_definition_id, status)?),
        ));
    }

    if let Some(status) = filter.status {
        return Ok((
            IndexTable::OwnedAssetByStatus,
            Some(encode_owned_asset_by_status(status)?),
        ));
    }

    Ok((IndexTable::OwnedAssetById, None))
}

fn payload_matches_filter(payload: &OwnedAssetPayload, filter: &AssetFilter) -> bool {
    if let Some(account_id) = filter.account_id {
        if payload.account_id != Some(account_id) {
            return false;
        }
    }

    if let Some(asset_definition_id) = filter.asset_definition_id {
        if payload.asset_definition_id != asset_definition_id {
            return false;
        }
    }

    if let Some(status) = filter.status {
        if payload.status != status {
            return false;
        }
    }

    if let Some(source) = filter.source {
        if payload.source != source {
            return false;
        }
    }

    if let Some(tx_id) = filter.tx_id.as_ref() {
        if payload.spend_ref.as_ref() != Some(tx_id) {
            return false;
        }
    }

    if let Some(min_height) = filter.min_height {
        match payload.first_seen.as_ref().and_then(|seen| seen.height) {
            Some(height) if height >= min_height => {}
            _ => return false,
        }
    }

    if let Some(max_height) = filter.max_height {
        match payload.first_seen.as_ref().and_then(|seen| seen.height) {
            Some(height) if height <= max_height => {}
            _ => return false,
        }
    }

    true
}

pub(super) fn parse_offset_cursor(cursor: Option<&str>) -> WalletResult<usize> {
    match cursor {
        None => Ok(0),
        Some("") => Ok(0),
        Some(raw) => raw
            .parse::<usize>()
            .map_err(|_| WalletError::InvalidConfig("invalid owned asset cursor".to_string())),
    }
}

fn decode_owned_asset_payload(
    session: &WalletSession,
    object_id: u128,
) -> WalletResult<OwnedAssetPayload> {
    validate_object_index_rows(session, object_id)?;
    let payload = read_object_by_id(session, object_id)?;
    if payload.kind_id != ObjectKindId::OwnedAsset as u8
        || payload.payload_version != PAYLOAD_VERSION_OWNED_ASSET
    {
        return Err(WalletError::InvalidConfig(
            "owned asset payload kind mismatch".to_string(),
        ));
    }

    let decoded: OwnedAssetPayload = decode_bincode(&payload.data)?;
    let decoded = decoded.migrate_to_current()?;
    decoded.verify_checksum()?;
    let _ = decoded.validate_invariants()?;

    if decoded.wallet_id != session.opened.wallet_id {
        return Err(WalletError::InvalidConfig(
            "owned asset wallet id drifted".to_string(),
        ));
    }

    Ok(decoded)
}

fn build_owned_asset_payload(
    session: &WalletSession,
    asset: Asset,
    source: OwnedAssetSource,
    context: &AssetPersistContext,
) -> WalletResult<OwnedAssetPayload> {
    let mut asset_wire = AssetWire::from_asset(&asset);
    asset_wire.secret = None;
    let first_seen = Some(AssetSeenRef {
        height: context.scan_ref.as_ref().map(|scan| scan.end_height),
        hash_or_root: context
            .scan_ref
            .as_ref()
            .map(|scan| scan.cursor_hash.clone()),
        local_time_ms: context.now_ms,
    });

    let mut payload = OwnedAssetPayload {
        version: OwnedAssetPayload::VERSION,
        wallet_id: session.opened.wallet_id.clone(),
        account_id: None,
        asset_id: asset.asset_id(),
        asset_definition_id: asset.definition.id,
        asset_wire,
        status: OwnedAssetStatus::Spendable,
        source,
        first_seen,
        last_updated_ms: context.now_ms,
        scan_ref: context.scan_ref.clone(),
        receive_ref: context.receive_ref.clone(),
        spend_ref: None,
        confirmation_ref: context.confirmation_ref.clone(),
        labels: Vec::new(),
        policy: OwnedAssetPolicy {
            frozen: false,
            manual_review: false,
            quarantine_reason: None,
        },
        checksum: None,
    };
    payload.checksum = Some(payload.compute_checksum());
    let _ = payload.validate_invariants()?;
    Ok(payload)
}

fn rewrite_owned_asset(
    session: &WalletSession,
    object_id: u128,
    payload: &OwnedAssetPayload,
) -> WalletResult<()> {
    let mut canonical = payload.clone().migrate_to_current()?;
    canonical.checksum = Some(canonical.compute_checksum());
    let _ = canonical.validate_invariants()?;
    let index_updates = owned_asset_index_updates(&canonical, Some(object_id))?;
    let _ = write_object_by_id(
        session,
        object_id,
        ObjectKindId::OwnedAsset as u8,
        PAYLOAD_VERSION_OWNED_ASSET,
        encode_bincode(&canonical)?,
        &index_updates,
        SystemRngProvider,
    )?;
    Ok(())
}

fn rewrite_owned_asset_with_txn(
    session: &WalletSession,
    write_txn: &redb::WriteTransaction,
    object_id: u128,
    payload: &OwnedAssetPayload,
) -> WalletResult<()> {
    let mut canonical = payload.clone().migrate_to_current()?;
    canonical.checksum = Some(canonical.compute_checksum());
    let _ = canonical.validate_invariants()?;
    let index_updates = owned_asset_index_updates(&canonical, Some(object_id))?;
    let mut rng = SystemRngProvider.rng();
    let record = encrypt_object_record(
        &mut rng,
        &session.opened.wallet_id,
        session.opened.derived_keys.data_key.reveal(),
        object_id,
        PAYLOAD_VERSION_OWNED_ASSET,
        ObjectKindId::OwnedAsset as u8,
        encode_bincode(&canonical)?,
    )?;
    let _ = write_object_with_indexes(session, write_txn, object_id, &record, &index_updates)?;
    Ok(())
}

fn insert_owned_asset(
    session: &WalletSession,
    write_txn: &redb::WriteTransaction,
    payload: &OwnedAssetPayload,
) -> WalletResult<u128> {
    let index_updates = owned_asset_index_updates(payload, None)?;
    let mut rng = SystemRngProvider.rng();
    let object_id = {
        let objects = write_txn
            .open_table(OBJECTS_TABLE)
            .map_err(|e| WalletError::InvalidConfig(format!("redb open objects failed: {e}")))?;
        allocate_object_id(&objects, &mut rng)?
    };
    let record = encrypt_object_record(
        &mut rng,
        &session.opened.wallet_id,
        session.opened.derived_keys.data_key.reveal(),
        object_id,
        PAYLOAD_VERSION_OWNED_ASSET,
        ObjectKindId::OwnedAsset as u8,
        encode_bincode(payload)?,
    )?;
    let _save_seq =
        write_object_with_indexes(session, write_txn, object_id, &record, &index_updates)?;
    Ok(object_id)
}

pub(super) fn owned_asset_index_updates(
    payload: &OwnedAssetPayload,
    object_id: Option<u128>,
) -> WalletResult<Vec<IndexUpdate>> {
    let pointer = match object_id {
        Some(object_id) => IndexValueBytes::from_object_id(object_id),
        None => IndexValueBytes::new(Vec::new())?,
    };

    let mut updates = vec![
        IndexUpdate::with_value_bytes(
            IndexTable::OwnedAssetById,
            encode_owned_asset_by_id(&payload.asset_id)?,
            pointer.clone(),
        )?,
        IndexUpdate::with_value_bytes(
            IndexTable::OwnedAssetByDefStatus,
            encode_owned_asset_def_status(&payload.asset_definition_id, payload.status)?,
            pointer.clone(),
        )?,
        IndexUpdate::with_value_bytes(
            IndexTable::OwnedAssetByStatus,
            encode_owned_asset_by_status(payload.status)?,
            pointer.clone(),
        )?,
    ];

    if let Some(tx_id) = payload.spend_ref.as_ref() {
        updates.push(IndexUpdate::with_value_bytes(
            IndexTable::OwnedAssetByTx,
            encode_owned_asset_by_tx(tx_id)?,
            pointer.clone(),
        )?);
    }

    if let Some(scan_ref) = payload.scan_ref.as_ref() {
        updates.push(IndexUpdate::with_value_bytes(
            IndexTable::OwnedAssetByScan,
            encode_owned_asset_by_scan(scan_ref)?,
            pointer,
        )?);
    }

    Ok(updates)
}

fn encode_owned_asset_by_id(asset_id: &[u8; 32]) -> WalletResult<Vec<u8>> {
    encode_index_semantic_kv("wallet.owned_asset", "asset_id", asset_id)
}

fn encode_owned_asset_def_status(
    asset_definition_id: &[u8; 32],
    status: OwnedAssetStatus,
) -> WalletResult<Vec<u8>> {
    let mut value = Vec::with_capacity(33);
    value.extend_from_slice(asset_definition_id);
    value.push(status_tag(status));
    encode_index_semantic_kv(
        "wallet.owned_asset",
        "asset_definition_status",
        value.as_slice(),
    )
}

fn encode_owned_asset_by_status(status: OwnedAssetStatus) -> WalletResult<Vec<u8>> {
    encode_index_semantic_kv("wallet.owned_asset", "status", &[status_tag(status)])
}

fn encode_owned_asset_by_tx(tx_id: &PersistTxId) -> WalletResult<Vec<u8>> {
    encode_index_semantic_kv("wallet.owned_asset", "tx_id", tx_id.0.as_bytes())
}

fn encode_owned_asset_by_scan(scan_ref: &ScanRef) -> WalletResult<Vec<u8>> {
    let mut value = Vec::with_capacity(8 + scan_ref.cursor_hash.len());
    value.extend_from_slice(&scan_ref.end_height.to_be_bytes());
    value.extend_from_slice(scan_ref.cursor_hash.as_slice());
    encode_index_semantic_kv("wallet.owned_asset", "scan_ref", value.as_slice())
}

fn status_tag(status: OwnedAssetStatus) -> u8 {
    match status {
        OwnedAssetStatus::Spendable => 1,
        OwnedAssetStatus::PendingSpend => 2,
        OwnedAssetStatus::Spent => 3,
        OwnedAssetStatus::PendingReceive => 4,
        OwnedAssetStatus::Quarantined => 5,
        OwnedAssetStatus::Archived => 6,
    }
}

fn same_insert_shape(left: &OwnedAssetPayload, right: &OwnedAssetPayload) -> bool {
    left.wallet_id == right.wallet_id
        && left.account_id == right.account_id
        && left.asset_id == right.asset_id
        && left.asset_definition_id == right.asset_definition_id
        && left.asset_wire == right.asset_wire
        && left.status == right.status
        && left.source == right.source
        && left.first_seen.as_ref().map(|seen| seen.height)
            == right.first_seen.as_ref().map(|seen| seen.height)
        && left.scan_ref == right.scan_ref
        && left.receive_ref == right.receive_ref
        && left.spend_ref == right.spend_ref
        && left.confirmation_ref == right.confirmation_ref
        && left.labels == right.labels
        && left.policy.frozen == right.policy.frozen
        && left.policy.manual_review == right.policy.manual_review
        && left.policy.quarantine_reason == right.policy.quarantine_reason
}

fn same_owned_asset_wire(left: &OwnedAssetPayload, right: &OwnedAssetPayload) -> bool {
    left.wallet_id == right.wallet_id
        && left.account_id == right.account_id
        && left.asset_id == right.asset_id
        && left.asset_definition_id == right.asset_definition_id
        && left.asset_wire == right.asset_wire
}
