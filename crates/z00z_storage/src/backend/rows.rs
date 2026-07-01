use std::collections::BTreeSet;

use crate::backend::redb::state::{CanonExec, LoadState, WriteArts};
use crate::settlement::hjmt_journal::hjmt_journal_digest;
use crate::settlement::SettlementStore;
use z00z_utils::codec::{BincodeCodec, Codec};

use crate::settlement::{DefinitionId, SerialId};
use crate::{
    checkpoint::{
        derive_exec_id, encode_exec_bin, CheckpointExecInput, CheckpointExecTx,
        CheckpointExecVersion,
    },
    settlement::{
        tree_id::TreeRootRef, BucketId, FeeReplayRec, SettlementLeaf, SettlementPath,
        SettlementStateRoot, SnapItem, StoreItem,
    },
    snapshot::{store::build_snapshot_bin, PrepSnapshotId},
};

use super::codec::{def_payload, leaf_payload, map_check_err, ser_payload};
use super::types::{
    terminal_value_hash, ClaimNullRec, ClaimNullStatus, ClaimNullTx, SettlementStoreError, StoreOp,
};

type BucketRow = (DefinitionId, SerialId, BucketId, Vec<u8>);

impl SettlementStore {
    fn empty_snap_id() -> PrepSnapshotId {
        PrepSnapshotId::new([0u8; 32])
    }

    fn off_backend_write_arts(&self, version: u64) -> WriteArts {
        let mut snap_bytes = [0u8; 32];
        snap_bytes[..8].copy_from_slice(&version.to_be_bytes());
        WriteArts::new(
            version,
            PrepSnapshotId::new(snap_bytes),
            Vec::new(),
            None,
            Vec::new(),
            Vec::new(),
        )
    }

    pub(crate) fn ser_claim_null_rows(&self) -> Vec<ClaimNullRec> {
        self.nullifier.values().cloned().collect()
    }

    pub(crate) fn ser_fee_replay_rows(&self) -> Vec<FeeReplayRec> {
        self.fee_replays.values().copied().collect()
    }

    pub(crate) fn ser_hjmt_def_root(&self) -> Option<[u8; 32]> {
        self.hjmt_roots.def_root.map(TreeRootRef::into_bytes)
    }
    pub(crate) fn ser_hjmt_serial_roots(&self) -> Vec<(DefinitionId, [u8; 32])> {
        let mut roots: Vec<_> = self
            .hjmt_roots
            .serial_roots
            .iter()
            .map(|(definition_id, root)| (*definition_id, root.into_bytes()))
            .collect();
        roots.sort_by_key(|(definition_id, _)| definition_id.into_bytes());
        roots
    }
    pub(crate) fn ser_hjmt_bucket_roots(&self) -> Vec<((DefinitionId, SerialId), [u8; 32])> {
        let mut roots: Vec<_> = self
            .hjmt_roots
            .bucket_roots
            .iter()
            .map(|((definition_id, serial_id), root)| {
                ((*definition_id, *serial_id), root.into_bytes())
            })
            .collect();
        roots.sort_by_key(|((definition_id, serial_id), _)| {
            (definition_id.into_bytes(), serial_id.get())
        });
        roots
    }
    pub(crate) fn ser_hjmt_terminal_roots(
        &self,
    ) -> Vec<((DefinitionId, SerialId, BucketId), [u8; 32])> {
        let mut roots: Vec<_> = self
            .hjmt_roots
            .terminal_roots
            .iter()
            .map(|((definition_id, serial_id, bucket_id), root)| {
                ((*definition_id, *serial_id, *bucket_id), root.into_bytes())
            })
            .collect();
        roots.sort_by_key(|((definition_id, serial_id, bucket_id), _)| {
            (
                definition_id.into_bytes(),
                serial_id.get(),
                bucket_id.into_bytes(),
            )
        });
        roots
    }
    pub(crate) fn ser_hjmt_def_rows(
        &self,
    ) -> Result<Vec<(DefinitionId, Vec<u8>)>, SettlementStoreError> {
        let mut def_ids = self.model.def_ids();
        def_ids.sort_by_key(|definition_id| definition_id.into_bytes());

        let mut rows = Vec::with_capacity(def_ids.len());
        for definition_id in def_ids {
            let leaf = self
                .definition_leaf_from_roots(&self.hjmt_roots, &self.model, definition_id)?
                .ok_or(SettlementStoreError::HistMiss)?;
            rows.push((definition_id, def_payload(leaf)));
        }

        Ok(rows)
    }
    pub(crate) fn ser_hjmt_serial_rows(
        &self,
    ) -> Result<Vec<(DefinitionId, SerialId, Vec<u8>)>, SettlementStoreError> {
        let mut roots = self.ser_hjmt_bucket_roots();
        let mut rows = Vec::with_capacity(roots.len());
        for ((definition_id, serial_id), _) in roots.drain(..) {
            let leaf = self
                .serial_leaf_from_roots(&self.hjmt_roots, &self.model, (definition_id, serial_id))?
                .ok_or(SettlementStoreError::HistMiss)?;
            rows.push((definition_id, serial_id, ser_payload(leaf)));
        }

        Ok(rows)
    }
    pub(crate) fn ser_hjmt_bucket_rows(&self) -> Result<Vec<BucketRow>, SettlementStoreError> {
        let mut roots = self.ser_hjmt_terminal_roots();
        let mut rows = Vec::with_capacity(roots.len());
        for ((definition_id, serial_id, bucket_id), _) in roots.drain(..) {
            let leaf = self
                .bucket_leaf_from_roots(
                    &self.hjmt_roots,
                    &self.model,
                    (definition_id, serial_id, bucket_id),
                )?
                .ok_or(SettlementStoreError::HistMiss)?;
            rows.push((definition_id, serial_id, bucket_id, leaf.encode()));
        }

        Ok(rows)
    }

    pub(crate) fn ser_hjmt_terminal_rows(
        &self,
    ) -> Result<Vec<(SettlementPath, BucketId, Vec<u8>)>, SettlementStoreError> {
        let mut paths = self.model.paths();
        paths.sort();

        let mut rows = Vec::with_capacity(paths.len());
        for path in paths {
            let item = self
                .model
                .item_opt(&path)?
                .ok_or(SettlementStoreError::HistMiss)?;
            rows.push((
                path,
                path.bucket_id(self.bucket_policy()),
                leaf_payload(item.leaf())?,
            ));
        }
        Ok(rows)
    }

    pub(crate) fn ser_hjmt_settlement_path_rows(
        &self,
    ) -> Result<Vec<(SettlementPath, Vec<u8>)>, SettlementStoreError> {
        let mut paths = self.model.paths();
        paths.sort();

        let codec = BincodeCodec;
        let mut rows = Vec::with_capacity(paths.len());
        for path in paths {
            rows.push((path, codec.serialize(&path)?));
        }
        Ok(rows)
    }
    pub(crate) fn ser_hjmt_path_order(&self) -> Vec<SettlementPath> {
        let mut paths = self.model.paths();
        paths.sort();
        paths
    }

    pub(crate) fn ser_hjmt_root_rows(&self, version: u64) -> Vec<(Vec<u8>, [u8; 32])> {
        let mut rows = Vec::new();
        if let Some(root) = self.hjmt_roots.def_root {
            rows.push((hjmt_root_key(version, 1, &[]), root.into_bytes()));
        }

        let mut serial_roots: Vec<_> = self.hjmt_roots.serial_roots.iter().collect();
        serial_roots.sort_by_key(|(definition_id, _)| definition_id.into_bytes());
        for (definition_id, root) in serial_roots {
            rows.push((
                hjmt_root_key(version, 2, &[definition_id.as_bytes()]),
                root.into_bytes(),
            ));
        }

        let mut bucket_roots: Vec<_> = self.hjmt_roots.bucket_roots.iter().collect();
        bucket_roots.sort_by_key(|((definition_id, serial_id), _)| {
            (definition_id.into_bytes(), serial_id.get())
        });
        for ((definition_id, serial_id), root) in bucket_roots {
            let serial = serial_id.get().to_be_bytes();
            rows.push((
                hjmt_root_key(version, 3, &[definition_id.as_bytes(), &serial]),
                root.into_bytes(),
            ));
        }

        let mut terminal_roots: Vec<_> = self.hjmt_roots.terminal_roots.iter().collect();
        terminal_roots.sort_by_key(|((definition_id, serial_id, bucket_id), _)| {
            (
                definition_id.into_bytes(),
                serial_id.get(),
                bucket_id.into_bytes(),
            )
        });
        for ((definition_id, serial_id, bucket_id), root) in terminal_roots {
            let serial = serial_id.get().to_be_bytes();
            rows.push((
                hjmt_root_key(
                    version,
                    4,
                    &[definition_id.as_bytes(), &serial, bucket_id.as_bytes()],
                ),
                root.into_bytes(),
            ));
        }

        rows
    }

    pub(crate) fn hjmt_plan_arts(
        &self,
        version: u64,
        ops: &[StoreOp],
        txs: Option<&[CheckpointExecTx]>,
    ) -> Result<WriteArts, SettlementStoreError> {
        if !self.backend.is_on() {
            return Ok(self.off_backend_write_arts(version));
        }

        let mut snap_id = Self::empty_snap_id();
        let mut snap_bytes = Vec::new();
        let canon_exec = if let Some(txs) = txs {
            let prev_root: crate::settlement::CheckRoot = self.hjmt_root()?.into();
            let (_snapshot, next_snap_id, next_snap_bytes) =
                build_snapshot_bin(prev_root, self.snap_items()?)
                    .map_err(|err| SettlementStoreError::Backend(err.to_string()))?;
            snap_id = next_snap_id;
            snap_bytes = next_snap_bytes;
            let exec = CheckpointExecInput::new_settlement(
                CheckpointExecVersion::CURRENT,
                snap_id,
                self.hjmt_roots.settlement_root(),
                txs.to_vec(),
            )
            .map_err(map_check_err)?;
            let exec_bytes = encode_exec_bin(&exec).map_err(map_check_err)?;
            Some(CanonExec::new(derive_exec_id(&exec_bytes), exec_bytes))
        } else {
            None
        };
        let mut spent = Vec::new();
        let mut created = Vec::new();

        for op in ops {
            match op {
                StoreOp::Put(item) => {
                    let path = item.path();
                    if self.hjmt_get_settlement_item(&path)?.is_some() {
                        spent.push(crate::checkpoint::SpentEnt::new(
                            path.terminal_id().into_bytes(),
                        ));
                    }
                    created.push(crate::checkpoint::CreatedEnt::new(
                        path.terminal_id().into_bytes(),
                        terminal_value_hash(item.leaf())?.0,
                    ));
                }
                StoreOp::Delete(path) => {
                    spent.push(crate::checkpoint::SpentEnt::new(
                        path.terminal_id().into_bytes(),
                    ));
                }
            }
        }

        Ok(WriteArts::new(
            version, snap_id, snap_bytes, canon_exec, spent, created,
        ))
    }

    pub(crate) fn hjmt_rehydrate(&mut self, state: LoadState) -> Result<(), SettlementStoreError> {
        let mut expected_paths = std::collections::BTreeMap::new();
        let mut ops = Vec::new();
        for (path, bucket_id, payload) in &state.hjmt_terminal_rows {
            path.check()
                .map_err(|err| SettlementStoreError::Backend(err.to_string()))?;
            if path.bucket_id(self.bucket_policy()) != *bucket_id {
                return Err(SettlementStoreError::Backend(
                    "hjmt terminal row bucket id does not match committed settlement path"
                        .to_string(),
                ));
            }
            let leaf = SettlementLeaf::decode(payload)?;
            let item = StoreItem::new(*path, leaf)?;
            expected_paths.insert(path.terminal_id, *path);
            ops.push(StoreOp::Put(Box::new(item)));
        }

        let mut got_paths = std::collections::BTreeMap::new();
        for (path, payload) in &state.hjmt_settlement_path_rows {
            let codec = BincodeCodec;
            let persisted_path: SettlementPath = codec.deserialize(payload)?;
            persisted_path
                .check()
                .map_err(|err| SettlementStoreError::Backend(err.to_string()))?;
            if persisted_path != *path || persisted_path.terminal_id != path.terminal_id {
                return Err(SettlementStoreError::Backend(
                    "hjmt settlement path row does not match persisted key".to_string(),
                ));
            }
            got_paths.insert(path.terminal_id, *path);
        }
        if got_paths != expected_paths {
            return Err(SettlementStoreError::Backend(
                "hjmt settlement path index drift from committed terminal rows".to_string(),
            ));
        }

        let journal_digest = state.hjmt_journal.as_ref().map(hjmt_journal_digest);
        if let Some(journal) = state.hjmt_journal.as_ref() {
            if journal.bucket_policy_id != self.bucket_policy().bucket_policy_id() {
                return Err(SettlementStoreError::Backend(
                    "hjmt persisted bucket policy id does not match active bucket policy"
                        .to_string(),
                ));
            }
            if journal.bucket_epoch != state.version {
                return Err(SettlementStoreError::Backend(
                    "hjmt persisted bucket epoch does not match active version".to_string(),
                ));
            }
        }

        if !ops.is_empty() {
            let plan = self.sched_plan_ops(&ops)?;
            let _ = self.commit_hjmt_plan_at(plan, state.version, &[], None, false, None, None)?;
        } else {
            self.hjmt_roots.version = state.version;
        }
        if let Some(journal_digest) = journal_digest {
            self.hjmt_roots.journal_digest = Some(journal_digest);
        }

        let live_state_root = self.hjmt_root()?;
        if live_state_root != state.state_root {
            return Err(SettlementStoreError::Backend(
                "hjmt reload semantic root does not match persisted metadata".to_string(),
            ));
        }
        if live_state_root.into_bytes() != state.flat_root {
            return Err(SettlementStoreError::Backend(
                "hjmt reload flat_root does not match persisted metadata".to_string(),
            ));
        }

        self.claim_null_seq = state
            .claim_null_rows
            .iter()
            .map(|row| row.created_at_seq)
            .max()
            .unwrap_or(0);
        self.nullifier = state
            .claim_null_rows
            .into_iter()
            .map(|row| (row.nullifier, row))
            .collect();
        self.fee_replay_seq = state
            .fee_replay_rows
            .iter()
            .map(|row| row.accepted_at_seq)
            .max()
            .unwrap_or(0);
        self.fee_replays = state
            .fee_replay_rows
            .into_iter()
            .map(|row| (row.replay_key, row))
            .collect();
        self.settlement_root_by_ver.clear();
        self.model_by_ver.clear();
        self.hjmt_roots_by_ver.clear();
        self.settlement_root_by_ver.insert(
            state.version,
            SettlementStateRoot::settlement_v1(live_state_root.into_bytes()),
        );
        self.model_by_ver.insert(state.version, self.model.clone());
        self.hjmt_roots_by_ver
            .insert(state.version, self.hjmt_roots.clone());
        self.last_object_delta = state.object_delta.clone();
        self.object_deltas_by_ver.clear();
        if let Some(object_delta) = state.object_delta {
            self.object_deltas_by_ver
                .insert(state.version, object_delta);
        }
        self.forest_cache.clear_all();
        self.sched_run_local("hjmt_reload_cache_warm", || {
            self.warm_forest_cache_current()
        })?;
        self.sched_run_local("hjmt_reload_cache_verify", || {
            self.verify_forest_cache_sample()
        })?;
        Ok(())
    }

    fn snap_items(&self) -> Result<Vec<SnapItem>, SettlementStoreError> {
        let mut items = Vec::new();
        for path in self.sorted_paths() {
            let item = self.load_item(&path)?;
            let wit = self.settlement_proof_blob(&path)?.encode()?;
            items.push(SnapItem::new(path, item.leaf().clone(), wit)?);
        }
        Ok(items)
    }

    pub(crate) fn check_exec_ops(
        &self,
        ops: &[StoreOp],
        txs: &[CheckpointExecTx],
    ) -> Result<(), SettlementStoreError> {
        if txs.is_empty() {
            return Err(SettlementStoreError::Backend(
                "canonical checkpoint exec must contain tx rows".to_string(),
            ));
        }

        let mut want_spent = BTreeSet::new();
        let mut want_created = BTreeSet::new();
        for op in ops {
            match op {
                StoreOp::Put(item) => {
                    let path = item.path();
                    if self.hjmt_get_settlement_item(&path)?.is_some() {
                        want_spent.insert((path.terminal_id().into_bytes(), path.serial_id.get()));
                    }
                    want_created.insert((
                        path.terminal_id().into_bytes(),
                        path.definition_id.into_bytes(),
                        path.serial_id.get(),
                        terminal_value_hash(item.leaf())?.0,
                    ));
                }
                StoreOp::Delete(path) => {
                    let old = self.load_item(path)?;
                    want_spent.insert((
                        old.path().terminal_id().into_bytes(),
                        old.path().serial_id.get(),
                    ));
                }
            }
        }

        let mut got_spent = BTreeSet::new();
        let mut got_created = BTreeSet::new();
        for tx in txs {
            for input in tx.input_refs() {
                got_spent.insert((input.terminal_id().into_bytes(), input.serial_id().get()));
            }
            for output in tx.outputs() {
                got_created.insert((
                    output.leaf().asset_id,
                    output.definition_id().into_bytes(),
                    output.leaf().serial_id,
                    terminal_value_hash(output.leaf())?.0,
                ));
            }
        }

        if got_spent != want_spent || got_created != want_created {
            return Err(SettlementStoreError::Backend(
                "checkpoint exec does not match store ops".to_string(),
            ));
        }

        Ok(())
    }

    pub(crate) fn build_claim_rows(
        &self,
        claims: &[ClaimNullTx],
    ) -> Result<Vec<ClaimNullRec>, SettlementStoreError> {
        // Storage keys replay by ClaimNullifier and retains chain_id as audit evidence.
        let mut seen = BTreeSet::new();
        let mut next_seq = self.claim_null_seq;
        let mut rows = Vec::with_capacity(claims.len());

        for claim in claims {
            if let Some(row) = self.nullifier.get(&claim.nullifier) {
                return Err(SettlementStoreError::ClaimReplay(format!(
                    "nullifier={} status={:?} tx_digest={}",
                    row.nullifier, row.status, row.tx_digest_hex
                )));
            }
            if !seen.insert(claim.nullifier) {
                return Err(SettlementStoreError::ClaimReplay(format!(
                    "duplicate nullifier in one claim publish: {}",
                    claim.nullifier
                )));
            }

            next_seq = next_seq.saturating_add(1);
            rows.push(ClaimNullRec {
                nullifier: claim.nullifier,
                status: ClaimNullStatus::Spent,
                claim_id_hex: claim.claim_id_hex.clone(),
                chain_id: claim.chain_id,
                tx_digest_hex: claim.tx_digest_hex.clone(),
                created_at_seq: next_seq,
            });
        }

        Ok(rows)
    }

    pub(crate) fn commit_claim_rows(&mut self, rows: &[ClaimNullRec]) {
        for row in rows {
            self.claim_null_seq = self.claim_null_seq.max(row.created_at_seq);
            self.nullifier.insert(row.nullifier, row.clone());
        }
    }

    pub(crate) fn commit_fee_replay_row(&mut self, row: FeeReplayRec) {
        self.fee_replay_seq = self.fee_replay_seq.max(row.accepted_at_seq);
        self.fee_replays.insert(row.replay_key, row);
    }
}

fn hjmt_root_key(version: u64, tag: u8, parts: &[&[u8]]) -> Vec<u8> {
    let mut key = Vec::with_capacity(1 + 8 + parts.iter().map(|part| part.len()).sum::<usize>());
    key.extend_from_slice(&version.to_be_bytes());
    key.push(tag);
    for part in parts {
        key.extend_from_slice(part);
    }
    key
}
