use redb::{ReadableTable, TableDefinition};
use z00z_utils::codec::{BincodeCodec, Codec};

use super::{
    helpers::{
        claim_null_key, decode_claim_null_key, decode_claim_null_row, decode_fee_replay_key,
        decode_fee_replay_row, decode_hjmt_settlement_path_key, decode_hjmt_terminal_key,
        def_root_key, fee_replay_digest, fee_replay_key, hjmt_settlement_path_key,
        hjmt_terminal_row_key, map_check, map_store,
    },
    state::{LoadState, StateMeta, WriteArts},
    validate, RedbBackend, AST_ROW_TABLE, CHECK_TABLE, CLAIM_NULL_TABLE, DEF_ROOT_TABLE,
    DRAFT_TABLE, EXEC_TABLE, FEE_REPLAY_TABLE, HJMT_SETTLEMENT_PATH_TABLE, HJMT_TERMINAL_ROW_TABLE,
    KEY_ACTIVE, KEY_CHECK_ID, KEY_DRAFT_ID, KEY_EXEC_ID, KEY_SNAP_ID, KEY_STATE, KEY_STATE_ROOT,
    LINK_TABLE, META_TABLE, OBJECT_DELTA_TABLE, PATH_ROW_TABLE, SNAP_TABLE,
};
use crate::settlement::hjmt_config::env_opt;
use crate::settlement::hjmt_journal::{
    decode_journal, encode_journal, hjmt_child_digest, hjmt_fee_replay_digests, hjmt_parent_digest,
    validate_fee_replay_state, HjmtCommitJournalEntry, HjmtCommitStatus,
};
use crate::{
    backend::{error::StoreBackendError, roots::HjmtBucketKey},
    checkpoint::{
        derive_checkpoint_id, derive_draft_id, encode_art_bin, encode_draft_bin, encode_link_bin,
        CheckpointDraft, CheckpointLink, CheckpointLinkVersion,
    },
    settlement::{
        BucketId, ClaimNullRec, FeeReplayRec, ObjectDeltaSetV1, SettlementPath, SettlementRouteCtx,
        SettlementStateRoot,
    },
};

const HJMT_JOURNAL_TABLE: TableDefinition<&[u8], &[u8]> =
    TableDefinition::new("settlement_hjmt_journal");
const HJMT_ROOT_TABLE: TableDefinition<&[u8], &[u8]> =
    TableDefinition::new("settlement_hjmt_roots");
const HJMT_PENDING_META_TABLE: TableDefinition<&[u8], &[u8]> =
    TableDefinition::new("settlement_hjmt_pending_meta");
const HJMT_INJ_STAGE_ENV: &str = "Z00Z_STORAGE_HJMT_INJ_STAGE";
type HjmtRootRow = (Vec<u8>, [u8; 32]);

pub(crate) struct HjmtPersistWork {
    pub(crate) write_arts: WriteArts,
    pub(crate) version: u64,
    pub(crate) prev_root: SettlementStateRoot,
    pub(crate) next_root: SettlementStateRoot,
    pub(crate) route: Option<SettlementRouteCtx>,
    pub(crate) bucket_policy_id: [u8; 32],
    pub(crate) touched_buckets: Vec<HjmtBucketKey>,
    pub(crate) terminal_rows: Vec<(SettlementPath, BucketId, Vec<u8>)>,
    pub(crate) settlement_path_rows: Vec<(SettlementPath, Vec<u8>)>,
    pub(crate) claim_rows: Vec<ClaimNullRec>,
    pub(crate) fee_rows: Vec<FeeReplayRec>,
    pub(crate) root_rows: Vec<HjmtRootRow>,
    pub(crate) def_root: Option<[u8; 32]>,
    pub(crate) object_delta: Option<ObjectDeltaSetV1>,
}

impl RedbBackend {
    pub(super) fn sync_hjmt_work(&self, work: HjmtPersistWork) -> Result<(), StoreBackendError> {
        if self.root.is_none() {
            return Ok(());
        }

        let (child_root_rows, parent_root_rows) = split_root_rows(&work.root_rows);
        let child_digest = hjmt_child_digest(
            &work.terminal_rows,
            &work.settlement_path_rows,
            &work.claim_rows,
            &work.fee_rows,
            &child_root_rows,
        )
        .map_err(map_store)?;
        let parent_digest = hjmt_parent_digest(&parent_root_rows).map_err(map_store)?;
        let mut entry = HjmtCommitJournalEntry::new(
            work.version,
            work.version,
            work.bucket_policy_id,
            work.prev_root,
            work.next_root,
            &work.touched_buckets,
            child_digest,
            parent_digest,
        );
        if let Some(route) = work.route {
            entry = entry.with_route(route);
        }
        entry.seal_fee_replay_state(&work.fee_rows);

        self.write_journal(entry.clone())?;
        if inject_hjmt_stage("prepared") {
            return Err(StoreBackendError::Commit(
                "hjmt journal injection after Prepared".to_string(),
            ));
        }

        let check_bundle = checkpoint_bundle(
            &work.write_arts,
            SettlementStateRoot::settlement_v1(entry.next_semantic_state_root),
            SettlementStateRoot::settlement_v1(entry.previous_semantic_state_root),
        )?;

        self.write_children(
            &work.write_arts,
            &check_bundle,
            &work.terminal_rows,
            &work.settlement_path_rows,
            &work.claim_rows,
            &work.fee_rows,
            &child_root_rows,
            work.object_delta.as_ref(),
            entry
                .clone()
                .with_status(HjmtCommitStatus::ChildrenCommitted),
        )?;
        if inject_hjmt_stage("children") {
            return Err(StoreBackendError::Commit(
                "hjmt journal injection after ChildrenCommitted".to_string(),
            ));
        }

        self.write_parents(
            work.version,
            &parent_root_rows,
            entry
                .clone()
                .with_status(HjmtCommitStatus::ParentsCommitted),
        )?;
        if inject_hjmt_stage("parents") {
            return Err(StoreBackendError::Commit(
                "hjmt journal injection after ParentsCommitted".to_string(),
            ));
        }

        self.publish_root_work(
            &work,
            &check_bundle.ids,
            entry.with_status(HjmtCommitStatus::RootPublished),
        )
    }

    fn write_journal(&self, entry: HjmtCommitJournalEntry) -> Result<(), StoreBackendError> {
        let write = self
            .db()?
            .begin_write()
            .map_err(|err| StoreBackendError::Tx(err.to_string()))?;
        super::guard_write_head(&write, entry.version)?;
        {
            let table = write
                .open_table(HJMT_JOURNAL_TABLE)
                .map_err(|err| StoreBackendError::Tx(err.to_string()))?;
            if table
                .get(entry.version.to_be_bytes().as_slice())
                .map_err(|err| StoreBackendError::Tx(err.to_string()))?
                .is_some()
            {
                return Err(StoreBackendError::Tx(
                    "hjmt journal entry already exists for persisted version".to_string(),
                ));
            }
        }
        write_journal_entry(&write, entry)?;
        write
            .commit()
            .map_err(|err| StoreBackendError::Commit(err.to_string()))
    }

    fn write_children(
        &self,
        write_arts: &WriteArts,
        check_bundle: &CheckBundle,
        terminal_rows: &[(SettlementPath, BucketId, Vec<u8>)],
        settlement_path_rows: &[(SettlementPath, Vec<u8>)],
        claim_rows: &[ClaimNullRec],
        fee_rows: &[FeeReplayRec],
        child_root_rows: &[(Vec<u8>, [u8; 32])],
        object_delta: Option<&ObjectDeltaSetV1>,
        entry: HjmtCommitJournalEntry,
    ) -> Result<(), StoreBackendError> {
        let write = self
            .db()?
            .begin_write()
            .map_err(|err| StoreBackendError::Tx(err.to_string()))?;
        let version = entry.version;
        let codec = BincodeCodec;
        let ids = &check_bundle.ids;
        let bytes = &check_bundle.bytes;
        let pending_meta = StateMeta {
            version,
            state_root: entry.next_semantic_state_root,
            flat_root: entry.next_semantic_state_root,
            snap_id: write_arts.snap_id.into_bytes(),
            draft_id: ids.draft_id,
            check_id: ids.check_id,
            exec_id: ids.exec_id,
            def_root: None,
            fee_replay_count: u64::try_from(fee_rows.len()).unwrap_or(u64::MAX),
            fee_replay_digest: fee_replay_digest(fee_rows),
        };
        let pending_meta_bytes = codec.serialize(&pending_meta)?;

        {
            let mut terminal_table = write
                .open_table(HJMT_TERMINAL_ROW_TABLE)
                .map_err(|err| StoreBackendError::Tx(err.to_string()))?;
            for (path, bucket_id, payload) in terminal_rows {
                let key = hjmt_terminal_row_key(version, *path, *bucket_id);
                terminal_table
                    .insert(key.as_slice(), payload.as_slice())
                    .map_err(|err| StoreBackendError::Tx(err.to_string()))?;
            }
            drop(terminal_table);

            let mut path_table = write
                .open_table(HJMT_SETTLEMENT_PATH_TABLE)
                .map_err(|err| StoreBackendError::Tx(err.to_string()))?;
            for (path, payload) in settlement_path_rows {
                let key = hjmt_settlement_path_key(version, path.terminal_id);
                path_table
                    .insert(key.as_slice(), payload.as_slice())
                    .map_err(|err| StoreBackendError::Tx(err.to_string()))?;
            }
            drop(path_table);

            let mut claim_table = write
                .open_table(CLAIM_NULL_TABLE)
                .map_err(|err| StoreBackendError::Tx(err.to_string()))?;
            for row in claim_rows {
                let key = claim_null_key(version, row.nullifier);
                let payload = codec.serialize(row)?;
                claim_table
                    .insert(key.as_slice(), payload.as_slice())
                    .map_err(|err| StoreBackendError::Tx(err.to_string()))?;
            }
            drop(claim_table);

            let mut fee_table = write
                .open_table(FEE_REPLAY_TABLE)
                .map_err(|err| StoreBackendError::Tx(err.to_string()))?;
            for row in fee_rows {
                let key = fee_replay_key(version, row.replay_key);
                let payload = codec.serialize(row)?;
                fee_table
                    .insert(key.as_slice(), payload.as_slice())
                    .map_err(|err| StoreBackendError::Tx(err.to_string()))?;
            }
            drop(fee_table);

            if let Some(object_delta) = object_delta {
                let payload = codec.serialize(object_delta)?;
                let mut object_delta_table = write
                    .open_table(OBJECT_DELTA_TABLE)
                    .map_err(|err| StoreBackendError::Tx(err.to_string()))?;
                object_delta_table
                    .insert(version.to_be_bytes().as_slice(), payload.as_slice())
                    .map_err(|err| StoreBackendError::Tx(err.to_string()))?;
                drop(object_delta_table);
            }

            let mut root_table = write
                .open_table(HJMT_ROOT_TABLE)
                .map_err(|err| StoreBackendError::Tx(err.to_string()))?;
            for (key, root) in child_root_rows {
                root_table
                    .insert(key.as_slice(), root.as_slice())
                    .map_err(|err| StoreBackendError::Tx(err.to_string()))?;
            }
            drop(root_table);

            if !write_arts.snap_bytes.is_empty() {
                let mut snap_table = write
                    .open_table(SNAP_TABLE)
                    .map_err(|err| StoreBackendError::Tx(err.to_string()))?;
                snap_table
                    .insert(
                        &write_arts.snap_id.into_bytes()[..],
                        write_arts.snap_bytes.as_slice(),
                    )
                    .map_err(|err| StoreBackendError::Tx(err.to_string()))?;
                drop(snap_table);
            }

            let mut pending_meta_table = write
                .open_table(HJMT_PENDING_META_TABLE)
                .map_err(|err| StoreBackendError::Tx(err.to_string()))?;
            pending_meta_table
                .insert(
                    version.to_be_bytes().as_slice(),
                    pending_meta_bytes.as_slice(),
                )
                .map_err(|err| StoreBackendError::Tx(err.to_string()))?;
            drop(pending_meta_table);

            if ids.has_checkpoint() {
                let mut draft_table = write
                    .open_table(DRAFT_TABLE)
                    .map_err(|err| StoreBackendError::Tx(err.to_string()))?;
                draft_table
                    .insert(&ids.draft_id[..], bytes.draft.as_slice())
                    .map_err(|err| StoreBackendError::Tx(err.to_string()))?;
                drop(draft_table);

                let mut check_table = write
                    .open_table(CHECK_TABLE)
                    .map_err(|err| StoreBackendError::Tx(err.to_string()))?;
                check_table
                    .insert(&ids.check_id[..], bytes.check.as_slice())
                    .map_err(|err| StoreBackendError::Tx(err.to_string()))?;
                drop(check_table);

                let mut exec_table = write
                    .open_table(EXEC_TABLE)
                    .map_err(|err| StoreBackendError::Tx(err.to_string()))?;
                exec_table
                    .insert(&ids.exec_id[..], bytes.exec.as_slice())
                    .map_err(|err| StoreBackendError::Tx(err.to_string()))?;
                drop(exec_table);

                let mut link_table = write
                    .open_table(LINK_TABLE)
                    .map_err(|err| StoreBackendError::Tx(err.to_string()))?;
                link_table
                    .insert(&ids.check_id[..], bytes.link.as_slice())
                    .map_err(|err| StoreBackendError::Tx(err.to_string()))?;
                drop(link_table);
            }
        }

        write_journal_entry(&write, entry)?;
        write
            .commit()
            .map_err(|err| StoreBackendError::Commit(err.to_string()))
    }

    fn write_parents(
        &self,
        version: u64,
        root_rows: &[(Vec<u8>, [u8; 32])],
        entry: HjmtCommitJournalEntry,
    ) -> Result<(), StoreBackendError> {
        let write = self
            .db()?
            .begin_write()
            .map_err(|err| StoreBackendError::Tx(err.to_string()))?;
        {
            let mut root_table = write
                .open_table(HJMT_ROOT_TABLE)
                .map_err(|err| StoreBackendError::Tx(err.to_string()))?;
            for (key, root) in root_rows {
                root_table
                    .insert(key.as_slice(), root.as_slice())
                    .map_err(|err| StoreBackendError::Tx(err.to_string()))?;
            }
            drop(root_table);

            let mut def_root_table = write
                .open_table(DEF_ROOT_TABLE)
                .map_err(|err| StoreBackendError::Tx(err.to_string()))?;
            if let Some((_, root)) = root_rows
                .iter()
                .find(|(key, _)| key.len() == 9 && key[..8] == version.to_be_bytes() && key[8] == 1)
            {
                let key = def_root_key(version);
                def_root_table
                    .insert(key.as_slice(), root.as_slice())
                    .map_err(|err| StoreBackendError::Tx(err.to_string()))?;
            }
            drop(def_root_table);
        }

        write_journal_entry(&write, entry)?;
        write
            .commit()
            .map_err(|err| StoreBackendError::Commit(err.to_string()))
    }

    fn publish_root_work(
        &self,
        work: &HjmtPersistWork,
        check_ids: &CheckIds,
        entry: HjmtCommitJournalEntry,
    ) -> Result<(), StoreBackendError> {
        let meta = StateMeta {
            version: entry.version,
            state_root: entry.next_semantic_state_root,
            flat_root: entry.next_semantic_state_root,
            snap_id: work.write_arts.snap_id.into_bytes(),
            draft_id: check_ids.draft_id,
            check_id: check_ids.check_id,
            exec_id: check_ids.exec_id,
            def_root: work.def_root,
            fee_replay_count: u64::try_from(work.fee_rows.len()).unwrap_or(u64::MAX),
            fee_replay_digest: fee_replay_digest(&work.fee_rows),
        };
        let codec = BincodeCodec;
        let meta_bytes = codec.serialize(&meta)?;
        let write = self
            .db()?
            .begin_write()
            .map_err(|err| StoreBackendError::Tx(err.to_string()))?;
        {
            let mut meta_table = write
                .open_table(META_TABLE)
                .map_err(|err| StoreBackendError::Tx(err.to_string()))?;
            meta_table
                .insert(KEY_ACTIVE, entry.version.to_be_bytes().as_slice())
                .map_err(|err| StoreBackendError::Tx(err.to_string()))?;
            meta_table
                .insert(KEY_STATE, meta_bytes.as_slice())
                .map_err(|err| StoreBackendError::Tx(err.to_string()))?;
            meta_table
                .insert(KEY_STATE_ROOT, &entry.next_semantic_state_root[..])
                .map_err(|err| StoreBackendError::Tx(err.to_string()))?;
            meta_table
                .insert(KEY_SNAP_ID, &work.write_arts.snap_id.into_bytes()[..])
                .map_err(|err| StoreBackendError::Tx(err.to_string()))?;
            if check_ids.has_checkpoint() {
                meta_table
                    .insert(KEY_DRAFT_ID, &check_ids.draft_id[..])
                    .map_err(|err| StoreBackendError::Tx(err.to_string()))?;
                meta_table
                    .insert(KEY_CHECK_ID, &check_ids.check_id[..])
                    .map_err(|err| StoreBackendError::Tx(err.to_string()))?;
                meta_table
                    .insert(KEY_EXEC_ID, &check_ids.exec_id[..])
                    .map_err(|err| StoreBackendError::Tx(err.to_string()))?;
            } else {
                meta_table
                    .remove(KEY_DRAFT_ID)
                    .map_err(|err| StoreBackendError::Tx(err.to_string()))?;
                meta_table
                    .remove(KEY_CHECK_ID)
                    .map_err(|err| StoreBackendError::Tx(err.to_string()))?;
                meta_table
                    .remove(KEY_EXEC_ID)
                    .map_err(|err| StoreBackendError::Tx(err.to_string()))?;
            }
            drop(meta_table);
        }

        remove_pending_meta(&write, entry.version)?;
        write_journal_entry(&write, entry)?;
        write
            .commit()
            .map_err(|err| StoreBackendError::Commit(err.to_string()))
    }

    pub(super) fn recover_hjmt_journals(&self) -> Result<(), StoreBackendError> {
        if self.root.is_none() {
            return Ok(());
        }

        let db = self.db()?;
        let mut active_version = {
            let read = db
                .begin_read()
                .map_err(|err| StoreBackendError::Tx(err.to_string()))?;
            load_active_version(&read)?
        };
        let journals = {
            let read = db
                .begin_read()
                .map_err(|err| StoreBackendError::Tx(err.to_string()))?;
            load_all_journals(&read)?
        };

        for journal in journals {
            if active_version.is_some_and(|version| journal.version <= version) {
                continue;
            }

            match journal.status {
                HjmtCommitStatus::Prepared => {
                    validate_prepared_rollback(db, &journal)?;
                    rollback_version(db, journal.version)?;
                }
                HjmtCommitStatus::ChildrenCommitted => {
                    validate_child_stage(db, &journal)?;
                    rollback_version(db, journal.version)?;
                }
                HjmtCommitStatus::ParentsCommitted => {
                    validate_parent_stage(db, &journal)?;
                    publish_pending_root(db, journal.clone())?;
                    active_version = Some(journal.version);
                }
                HjmtCommitStatus::RootPublished => {
                    return Err(StoreBackendError::Tx(
                        "root-published hjmt journal is missing active metadata".to_string(),
                    ));
                }
            }
        }

        Ok(())
    }

    pub(super) fn load_hjmt_state_at(
        &self,
        version: u64,
    ) -> Result<Option<LoadState>, StoreBackendError> {
        if self.root.is_none() {
            return Ok(None);
        }

        let read = self
            .db()?
            .begin_read()
            .map_err(|err| StoreBackendError::Tx(err.to_string()))?;
        let Some(journal) = load_journal(&read, version)? else {
            return Ok(None);
        };
        if journal.status != HjmtCommitStatus::RootPublished {
            return Err(StoreBackendError::Tx(
                "historical hjmt state is not root-published".to_string(),
            ));
        }

        let rows = load_version_rows(&read, version)?;
        let state_root = SettlementStateRoot::settlement_v1(journal.next_semantic_state_root);
        validate_loaded(
            &journal,
            version,
            state_root,
            &rows.terminal_rows,
            &rows.settlement_path_rows,
            &rows.claim_rows,
            &rows.fee_rows,
            &rows.root_rows,
        )?;

        Ok(Some(LoadState {
            version,
            state_root,
            flat_root: journal.next_semantic_state_root,
            hjmt_terminal_rows: rows.terminal_rows,
            hjmt_settlement_path_rows: rows.settlement_path_rows,
            claim_null_rows: rows.claim_rows,
            fee_replay_rows: rows.fee_rows,
            object_delta: rows.object_delta,
            hjmt_journal: Some(journal),
        }))
    }

    pub(super) fn hjmt_last_version_for_path(
        &self,
        path: SettlementPath,
    ) -> Result<Option<u64>, StoreBackendError> {
        if self.root.is_none() {
            return Ok(None);
        }

        let read = self
            .db()?
            .begin_read()
            .map_err(|err| StoreBackendError::Tx(err.to_string()))?;
        let journals = load_all_journals(&read)?;
        drop(read);
        for journal in journals.into_iter().rev() {
            if journal.status != HjmtCommitStatus::RootPublished {
                continue;
            }

            if self
                .load_hjmt_state_at(journal.version)?
                .into_iter()
                .flat_map(|state| state.hjmt_settlement_path_rows.into_iter())
                .into_iter()
                .any(|(row_path, _)| row_path == path)
            {
                return Ok(Some(journal.version));
            }
        }

        Ok(None)
    }
}

pub(super) fn load_journal(
    read: &redb::ReadTransaction,
    version: u64,
) -> Result<Option<HjmtCommitJournalEntry>, StoreBackendError> {
    let table = match read.open_table(HJMT_JOURNAL_TABLE) {
        Ok(table) => table,
        Err(_) => return Ok(None),
    };
    let Some(bytes) = table
        .get(version.to_be_bytes().as_slice())
        .map_err(|err| StoreBackendError::Tx(err.to_string()))?
    else {
        return Ok(None);
    };
    decode_journal(bytes.value()).map(Some).map_err(map_store)
}

pub(super) fn load_root_rows(
    read: &redb::ReadTransaction,
    version: u64,
) -> Result<Vec<HjmtRootRow>, StoreBackendError> {
    let table = match read.open_table(HJMT_ROOT_TABLE) {
        Ok(table) => table,
        Err(_) => return Ok(Vec::new()),
    };
    let mut rows = Vec::new();
    for entry in table
        .iter()
        .map_err(|err| StoreBackendError::Tx(err.to_string()))?
    {
        let (key, value) = entry.map_err(|err| StoreBackendError::Tx(err.to_string()))?;
        if !key.value().starts_with(&version.to_be_bytes()) {
            continue;
        }
        let value = value.value();
        if value.len() != 32 {
            return Err(StoreBackendError::Tx(
                "hjmt root row has invalid root length".to_string(),
            ));
        }
        let mut root = [0u8; 32];
        root.copy_from_slice(value);
        rows.push((key.value().to_vec(), root));
    }
    Ok(rows)
}

pub(super) fn validate_loaded(
    journal: &HjmtCommitJournalEntry,
    version: u64,
    state_root: SettlementStateRoot,
    terminal_rows: &[(SettlementPath, BucketId, Vec<u8>)],
    settlement_path_rows: &[(SettlementPath, Vec<u8>)],
    claim_rows: &[ClaimNullRec],
    fee_rows: &[FeeReplayRec],
    root_rows: &[(Vec<u8>, [u8; 32])],
) -> Result<(), StoreBackendError> {
    journal
        .require_root_published(version, state_root)
        .map_err(map_store)?;
    let (child_root_rows, parent_root_rows) = split_root_rows(root_rows);
    let child_digest = hjmt_child_digest(
        terminal_rows,
        settlement_path_rows,
        claim_rows,
        fee_rows,
        &child_root_rows,
    )
    .map_err(map_store)?;
    if child_digest != journal.child_commit_digest {
        return Err(StoreBackendError::Tx(
            "hjmt child commit digest mismatch".to_string(),
        ));
    }
    if hjmt_fee_replay_digests(fee_rows) != journal.fee_replay_digests {
        return Err(StoreBackendError::Tx(
            "hjmt fee replay digest mismatch".to_string(),
        ));
    }
    validate_fee_replay_state(
        fee_rows,
        journal.fee_replay_count,
        journal.fee_replay_digest,
    )
    .map_err(map_store)?;
    let parent_digest = hjmt_parent_digest(&parent_root_rows).map_err(map_store)?;
    if parent_digest != journal.parent_commit_digest {
        return Err(StoreBackendError::Tx(
            "hjmt parent commit digest mismatch".to_string(),
        ));
    }
    Ok(())
}

fn write_journal_entry(
    write: &redb::WriteTransaction,
    entry: HjmtCommitJournalEntry,
) -> Result<(), StoreBackendError> {
    let bytes = encode_journal(&entry);
    let mut table = write
        .open_table(HJMT_JOURNAL_TABLE)
        .map_err(|err| StoreBackendError::Tx(err.to_string()))?;
    if let Some(old) = table
        .get(entry.version.to_be_bytes().as_slice())
        .map_err(|err| StoreBackendError::Tx(err.to_string()))?
    {
        let old = decode_journal(old.value()).map_err(map_store)?;
        if old.status.rank() > entry.status.rank() {
            return Err(StoreBackendError::Tx(
                "hjmt journal status regression".to_string(),
            ));
        }
    }
    table
        .insert(entry.version.to_be_bytes().as_slice(), bytes.as_slice())
        .map_err(|err| StoreBackendError::Tx(err.to_string()))?;
    Ok(())
}

fn load_active_version(read: &redb::ReadTransaction) -> Result<Option<u64>, StoreBackendError> {
    let meta_table = match read.open_table(META_TABLE) {
        Ok(table) => table,
        Err(_) => return Ok(None),
    };
    let Some(meta_bytes) = meta_table
        .get(KEY_STATE)
        .map_err(|err| StoreBackendError::Tx(err.to_string()))?
    else {
        return Ok(None);
    };
    let codec = BincodeCodec;
    let meta: StateMeta = codec.deserialize(meta_bytes.value())?;
    Ok(Some(meta.version))
}

fn load_all_journals(
    read: &redb::ReadTransaction,
) -> Result<Vec<HjmtCommitJournalEntry>, StoreBackendError> {
    let table = match read.open_table(HJMT_JOURNAL_TABLE) {
        Ok(table) => table,
        Err(_) => return Ok(Vec::new()),
    };
    let mut entries = Vec::new();
    for entry in table
        .iter()
        .map_err(|err| StoreBackendError::Tx(err.to_string()))?
    {
        let (_, value) = entry.map_err(|err| StoreBackendError::Tx(err.to_string()))?;
        entries.push(decode_journal(value.value()).map_err(map_store)?);
    }
    entries.sort_by_key(|entry| entry.version);
    Ok(entries)
}

fn validate_prepared_rollback(
    db: &redb::Database,
    journal: &HjmtCommitJournalEntry,
) -> Result<(), StoreBackendError> {
    let read = db
        .begin_read()
        .map_err(|err| StoreBackendError::Tx(err.to_string()))?;
    if load_pending_meta_if_present(&read, journal.version)?.is_some() {
        return Err(StoreBackendError::Tx(
            "prepared hjmt journal has pending checkpoint metadata".to_string(),
        ));
    }
    let pending = load_version_rows(&read, journal.version)?;
    if !pending.terminal_rows.is_empty()
        || !pending.settlement_path_rows.is_empty()
        || !pending.claim_rows.is_empty()
        || !pending.fee_rows.is_empty()
        || !pending.root_rows.is_empty()
    {
        return Err(StoreBackendError::Tx(
            "prepared hjmt journal has durable child or parent rows".to_string(),
        ));
    }
    Ok(())
}

fn validate_child_stage(
    db: &redb::Database,
    journal: &HjmtCommitJournalEntry,
) -> Result<(), StoreBackendError> {
    let read = db
        .begin_read()
        .map_err(|err| StoreBackendError::Tx(err.to_string()))?;
    let pending = load_version_rows(&read, journal.version)?;
    let pending_meta = load_pending_meta(&read, journal.version)?;
    validate_pending_meta(&read, &pending_meta, journal, &pending.fee_rows)?;
    let (child_root_rows, parent_root_rows) = split_root_rows(&pending.root_rows);
    if !parent_root_rows.is_empty() {
        return Err(StoreBackendError::Tx(
            "children-committed hjmt journal has durable parent rows".to_string(),
        ));
    }
    let child_digest = hjmt_child_digest(
        &pending.terminal_rows,
        &pending.settlement_path_rows,
        &pending.claim_rows,
        &pending.fee_rows,
        &child_root_rows,
    )
    .map_err(map_store)?;
    if child_digest != journal.child_commit_digest {
        return Err(StoreBackendError::Tx(
            "hjmt child commit digest mismatch".to_string(),
        ));
    }
    if hjmt_fee_replay_digests(&pending.fee_rows) != journal.fee_replay_digests {
        return Err(StoreBackendError::Tx(
            "hjmt fee replay digest mismatch".to_string(),
        ));
    }
    validate_fee_replay_state(
        &pending.fee_rows,
        journal.fee_replay_count,
        journal.fee_replay_digest,
    )
    .map_err(map_store)?;
    Ok(())
}

fn validate_parent_stage(
    db: &redb::Database,
    journal: &HjmtCommitJournalEntry,
) -> Result<(), StoreBackendError> {
    let read = db
        .begin_read()
        .map_err(|err| StoreBackendError::Tx(err.to_string()))?;
    let pending = load_version_rows(&read, journal.version)?;
    let pending_meta = load_pending_meta(&read, journal.version)?;
    validate_pending_meta(&read, &pending_meta, journal, &pending.fee_rows)?;
    let (child_root_rows, parent_root_rows) = split_root_rows(&pending.root_rows);
    let child_digest = hjmt_child_digest(
        &pending.terminal_rows,
        &pending.settlement_path_rows,
        &pending.claim_rows,
        &pending.fee_rows,
        &child_root_rows,
    )
    .map_err(map_store)?;
    if child_digest != journal.child_commit_digest {
        return Err(StoreBackendError::Tx(
            "hjmt child commit digest mismatch".to_string(),
        ));
    }
    if hjmt_fee_replay_digests(&pending.fee_rows) != journal.fee_replay_digests {
        return Err(StoreBackendError::Tx(
            "hjmt fee replay digest mismatch".to_string(),
        ));
    }
    validate_fee_replay_state(
        &pending.fee_rows,
        journal.fee_replay_count,
        journal.fee_replay_digest,
    )
    .map_err(map_store)?;
    let parent_digest = hjmt_parent_digest(&parent_root_rows).map_err(map_store)?;
    if parent_digest != journal.parent_commit_digest {
        return Err(StoreBackendError::Tx(
            "hjmt parent commit digest mismatch".to_string(),
        ));
    }
    Ok(())
}

fn publish_pending_root(
    db: &redb::Database,
    journal: HjmtCommitJournalEntry,
) -> Result<(), StoreBackendError> {
    let read = db
        .begin_read()
        .map_err(|err| StoreBackendError::Tx(err.to_string()))?;
    let version_rows = load_version_rows(&read, journal.version)?;
    let pending_meta = load_pending_meta(&read, journal.version)?;
    validate_pending_meta(&read, &pending_meta, &journal, &version_rows.fee_rows)?;
    drop(read);

    let def_root = version_rows
        .root_rows
        .iter()
        .find(|(key, _)| is_def_root_key(journal.version, key))
        .map(|(_, root)| *root);
    let mut meta = pending_meta;
    meta.state_root = journal.next_semantic_state_root;
    meta.flat_root = journal.next_semantic_state_root;
    meta.def_root = def_root;
    meta.fee_replay_count = u64::try_from(version_rows.fee_rows.len()).unwrap_or(u64::MAX);
    meta.fee_replay_digest = fee_replay_digest(&version_rows.fee_rows);
    let codec = BincodeCodec;
    let meta_bytes = codec.serialize(&meta)?;
    let write = db
        .begin_write()
        .map_err(|err| StoreBackendError::Tx(err.to_string()))?;
    {
        let mut meta_table = write
            .open_table(META_TABLE)
            .map_err(|err| StoreBackendError::Tx(err.to_string()))?;
        meta_table
            .insert(KEY_ACTIVE, journal.version.to_be_bytes().as_slice())
            .map_err(|err| StoreBackendError::Tx(err.to_string()))?;
        meta_table
            .insert(KEY_STATE, meta_bytes.as_slice())
            .map_err(|err| StoreBackendError::Tx(err.to_string()))?;
        meta_table
            .insert(KEY_STATE_ROOT, &journal.next_semantic_state_root[..])
            .map_err(|err| StoreBackendError::Tx(err.to_string()))?;
        meta_table
            .insert(KEY_SNAP_ID, &meta.snap_id[..])
            .map_err(|err| StoreBackendError::Tx(err.to_string()))?;
        if meta.exec_id != [0u8; 32] {
            meta_table
                .insert(KEY_DRAFT_ID, &meta.draft_id[..])
                .map_err(|err| StoreBackendError::Tx(err.to_string()))?;
            meta_table
                .insert(KEY_CHECK_ID, &meta.check_id[..])
                .map_err(|err| StoreBackendError::Tx(err.to_string()))?;
            meta_table
                .insert(KEY_EXEC_ID, &meta.exec_id[..])
                .map_err(|err| StoreBackendError::Tx(err.to_string()))?;
        } else {
            meta_table
                .remove(KEY_DRAFT_ID)
                .map_err(|err| StoreBackendError::Tx(err.to_string()))?;
            meta_table
                .remove(KEY_CHECK_ID)
                .map_err(|err| StoreBackendError::Tx(err.to_string()))?;
            meta_table
                .remove(KEY_EXEC_ID)
                .map_err(|err| StoreBackendError::Tx(err.to_string()))?;
        }
        remove_pending_meta(&write, journal.version)?;
    }
    write_journal_entry(&write, journal.with_status(HjmtCommitStatus::RootPublished))?;
    write
        .commit()
        .map_err(|err| StoreBackendError::Commit(err.to_string()))
}

fn rollback_version(db: &redb::Database, version: u64) -> Result<(), StoreBackendError> {
    let write = db
        .begin_write()
        .map_err(|err| StoreBackendError::Tx(err.to_string()))?;
    remove_version_rows(&write, version)?;
    remove_pending_meta(&write, version)?;
    remove_journal_entry(&write, version)?;
    write
        .commit()
        .map_err(|err| StoreBackendError::Commit(err.to_string()))
}

fn load_pending_meta(
    read: &redb::ReadTransaction,
    version: u64,
) -> Result<StateMeta, StoreBackendError> {
    let table = read
        .open_table(HJMT_PENDING_META_TABLE)
        .map_err(|err| StoreBackendError::Tx(err.to_string()))?;
    let bytes = table
        .get(version.to_be_bytes().as_slice())
        .map_err(|err| StoreBackendError::Tx(err.to_string()))?
        .ok_or_else(|| StoreBackendError::Tx("missing hjmt pending state metadata".to_string()))?;
    let codec = BincodeCodec;
    codec.deserialize(bytes.value()).map_err(Into::into)
}

fn load_pending_meta_if_present(
    read: &redb::ReadTransaction,
    version: u64,
) -> Result<Option<StateMeta>, StoreBackendError> {
    let table = match read.open_table(HJMT_PENDING_META_TABLE) {
        Ok(table) => table,
        Err(_) => return Ok(None),
    };
    let Some(bytes) = table
        .get(version.to_be_bytes().as_slice())
        .map_err(|err| StoreBackendError::Tx(err.to_string()))?
    else {
        return Ok(None);
    };
    let codec = BincodeCodec;
    codec
        .deserialize(bytes.value())
        .map(Some)
        .map_err(Into::into)
}

fn validate_pending_meta(
    read: &redb::ReadTransaction,
    pending_meta: &StateMeta,
    journal: &HjmtCommitJournalEntry,
    fee_rows: &[FeeReplayRec],
) -> Result<(), StoreBackendError> {
    validate::validate_checkpoint_meta(read, pending_meta)?;
    if pending_meta.version != journal.version {
        return Err(StoreBackendError::Tx(
            "pending checkpoint metadata version does not match hjmt journal".to_string(),
        ));
    }
    if pending_meta.state_root != journal.next_semantic_state_root
        || pending_meta.flat_root != journal.next_semantic_state_root
    {
        return Err(StoreBackendError::Tx(
            "pending checkpoint metadata next root does not match hjmt journal".to_string(),
        ));
    }
    validate_fee_meta(pending_meta, fee_rows)
}

pub(super) fn validate_fee_meta(
    meta: &StateMeta,
    fee_rows: &[FeeReplayRec],
) -> Result<(), StoreBackendError> {
    let fee_replay_count = u64::try_from(fee_rows.len()).unwrap_or(u64::MAX);
    let fee_replay_digest = fee_replay_digest(fee_rows);
    if fee_replay_count != meta.fee_replay_count || fee_replay_digest != meta.fee_replay_digest {
        return Err(StoreBackendError::Tx(
            "fee replay rows do not match persisted replay metadata".to_owned(),
        ));
    }
    Ok(())
}

fn remove_pending_meta(
    write: &redb::WriteTransaction,
    version: u64,
) -> Result<(), StoreBackendError> {
    let mut table = write
        .open_table(HJMT_PENDING_META_TABLE)
        .map_err(|err| StoreBackendError::Tx(err.to_string()))?;
    table
        .remove(version.to_be_bytes().as_slice())
        .map_err(|err| StoreBackendError::Tx(err.to_string()))?;
    Ok(())
}

struct VersionRows {
    terminal_rows: Vec<(SettlementPath, BucketId, Vec<u8>)>,
    settlement_path_rows: Vec<(SettlementPath, Vec<u8>)>,
    claim_rows: Vec<ClaimNullRec>,
    fee_rows: Vec<FeeReplayRec>,
    object_delta: Option<ObjectDeltaSetV1>,
    root_rows: Vec<(Vec<u8>, [u8; 32])>,
}

fn load_version_rows(
    read: &redb::ReadTransaction,
    version: u64,
) -> Result<VersionRows, StoreBackendError> {
    Ok(VersionRows {
        terminal_rows: load_terminal_rows(read, version)?,
        settlement_path_rows: load_settlement_path_rows(read, version)?,
        claim_rows: load_claim_rows(read, version)?,
        fee_rows: load_fee_rows(read, version)?,
        object_delta: load_object_delta(read, version)?,
        root_rows: load_root_rows(read, version)?,
    })
}

pub(super) fn load_terminal_rows(
    read: &redb::ReadTransaction,
    version: u64,
) -> Result<Vec<(SettlementPath, BucketId, Vec<u8>)>, StoreBackendError> {
    let table = match read.open_table(HJMT_TERMINAL_ROW_TABLE) {
        Ok(table) => table,
        Err(_) => return Ok(Vec::new()),
    };
    let mut rows = Vec::new();
    for entry in table
        .iter()
        .map_err(|err| StoreBackendError::Tx(err.to_string()))?
    {
        let (key, value) = entry.map_err(|err| StoreBackendError::Tx(err.to_string()))?;
        if let Some((path, bucket_id)) = decode_hjmt_terminal_key(key.value(), version) {
            rows.push((path, bucket_id, value.value().to_vec()));
        }
    }
    Ok(rows)
}

pub(super) fn load_settlement_path_rows(
    read: &redb::ReadTransaction,
    version: u64,
) -> Result<Vec<(SettlementPath, Vec<u8>)>, StoreBackendError> {
    let table = match read.open_table(HJMT_SETTLEMENT_PATH_TABLE) {
        Ok(table) => table,
        Err(_) => return Ok(Vec::new()),
    };
    let codec = BincodeCodec;
    let mut rows = Vec::new();
    for entry in table
        .iter()
        .map_err(|err| StoreBackendError::Tx(err.to_string()))?
    {
        let (key, value) = entry.map_err(|err| StoreBackendError::Tx(err.to_string()))?;
        let Some(terminal_id) = decode_hjmt_settlement_path_key(key.value(), version) else {
            continue;
        };
        let path: SettlementPath = codec.deserialize(value.value())?;
        if path.terminal_id != terminal_id {
            return Err(StoreBackendError::Tx(
                "hjmt settlement path row does not match persisted key".to_string(),
            ));
        }
        rows.push((path, value.value().to_vec()));
    }
    Ok(rows)
}

fn load_claim_rows(
    read: &redb::ReadTransaction,
    version: u64,
) -> Result<Vec<ClaimNullRec>, StoreBackendError> {
    let table = match read.open_table(CLAIM_NULL_TABLE) {
        Ok(table) => table,
        Err(_) => return Ok(Vec::new()),
    };
    let mut rows = Vec::new();
    for entry in table
        .iter()
        .map_err(|err| StoreBackendError::Tx(err.to_string()))?
    {
        let (key, value) = entry.map_err(|err| StoreBackendError::Tx(err.to_string()))?;
        let Some(key_null) = decode_claim_null_key(key.value(), version) else {
            continue;
        };
        let row = decode_claim_null_row(value.value())?;
        if row.nullifier != key_null {
            return Err(StoreBackendError::Tx(
                "claim nullifier row does not match persisted replay key".to_owned(),
            ));
        }
        rows.push(row);
    }
    Ok(rows)
}

fn load_fee_rows(
    read: &redb::ReadTransaction,
    version: u64,
) -> Result<Vec<FeeReplayRec>, StoreBackendError> {
    let table = match read.open_table(FEE_REPLAY_TABLE) {
        Ok(table) => table,
        Err(_) => return Ok(Vec::new()),
    };
    let mut rows = Vec::new();
    for entry in table
        .iter()
        .map_err(|err| StoreBackendError::Tx(err.to_string()))?
    {
        let (key, value) = entry.map_err(|err| StoreBackendError::Tx(err.to_string()))?;
        let Some(key_replay) = decode_fee_replay_key(key.value(), version) else {
            continue;
        };
        let row = decode_fee_replay_row(value.value())?;
        if row.replay_key != key_replay {
            return Err(StoreBackendError::Tx(
                "fee replay row does not match persisted replay key".to_owned(),
            ));
        }
        rows.push(row);
    }
    Ok(rows)
}

fn load_object_delta(
    read: &redb::ReadTransaction,
    version: u64,
) -> Result<Option<ObjectDeltaSetV1>, StoreBackendError> {
    let table = match read.open_table(OBJECT_DELTA_TABLE) {
        Ok(table) => table,
        Err(_) => return Ok(None),
    };
    let Some(bytes) = table
        .get(version.to_be_bytes().as_slice())
        .map_err(|err| StoreBackendError::Tx(err.to_string()))?
    else {
        return Ok(None);
    };
    let codec = BincodeCodec;
    codec
        .deserialize(bytes.value())
        .map(Some)
        .map_err(Into::into)
}

fn remove_version_rows(
    write: &redb::WriteTransaction,
    version: u64,
) -> Result<(), StoreBackendError> {
    remove_by_prefix(write, AST_ROW_TABLE, &version.to_be_bytes())?;
    remove_by_prefix(write, PATH_ROW_TABLE, &version.to_be_bytes())?;
    remove_by_prefix(write, HJMT_TERMINAL_ROW_TABLE, &version.to_be_bytes())?;
    remove_by_prefix(write, HJMT_SETTLEMENT_PATH_TABLE, &version.to_be_bytes())?;
    remove_by_prefix(write, CLAIM_NULL_TABLE, &version.to_be_bytes())?;
    remove_by_prefix(write, FEE_REPLAY_TABLE, &version.to_be_bytes())?;
    remove_by_prefix(write, OBJECT_DELTA_TABLE, &version.to_be_bytes())?;
    remove_by_prefix(write, HJMT_ROOT_TABLE, &version.to_be_bytes())?;
    remove_by_prefix(write, DEF_ROOT_TABLE, &version.to_be_bytes())?;
    Ok(())
}

fn remove_by_prefix(
    write: &redb::WriteTransaction,
    table_def: TableDefinition<&[u8], &[u8]>,
    prefix: &[u8],
) -> Result<(), StoreBackendError> {
    let mut table = write
        .open_table(table_def)
        .map_err(|err| StoreBackendError::Tx(err.to_string()))?;
    let mut keys = Vec::new();
    for entry in table
        .iter()
        .map_err(|err| StoreBackendError::Tx(err.to_string()))?
    {
        let (key, _) = entry.map_err(|err| StoreBackendError::Tx(err.to_string()))?;
        if key.value().starts_with(prefix) {
            keys.push(key.value().to_vec());
        }
    }
    for key in keys {
        table
            .remove(key.as_slice())
            .map_err(|err| StoreBackendError::Tx(err.to_string()))?;
    }
    Ok(())
}

fn remove_journal_entry(
    write: &redb::WriteTransaction,
    version: u64,
) -> Result<(), StoreBackendError> {
    let mut table = write
        .open_table(HJMT_JOURNAL_TABLE)
        .map_err(|err| StoreBackendError::Tx(err.to_string()))?;
    table
        .remove(version.to_be_bytes().as_slice())
        .map_err(|err| StoreBackendError::Tx(err.to_string()))?;
    Ok(())
}

fn split_root_rows(root_rows: &[HjmtRootRow]) -> (Vec<HjmtRootRow>, Vec<HjmtRootRow>) {
    root_rows
        .iter()
        .cloned()
        .partition(|(key, _)| key.get(8).copied() == Some(4))
}

fn is_def_root_key(version: u64, key: &[u8]) -> bool {
    key.len() == 9 && key.starts_with(&version.to_be_bytes()) && key[8] == 1
}

struct CheckIds {
    draft_id: [u8; 32],
    check_id: [u8; 32],
    exec_id: [u8; 32],
}

impl CheckIds {
    fn empty() -> Self {
        Self {
            draft_id: [0u8; 32],
            check_id: [0u8; 32],
            exec_id: [0u8; 32],
        }
    }

    fn has_checkpoint(&self) -> bool {
        self.exec_id != [0u8; 32]
    }
}

struct CheckBytes {
    draft: Vec<u8>,
    check: Vec<u8>,
    exec: Vec<u8>,
    link: Vec<u8>,
}

struct CheckBundle {
    ids: CheckIds,
    bytes: CheckBytes,
}

fn checkpoint_bundle(
    write_arts: &WriteArts,
    state_root: SettlementStateRoot,
    prev_settlement_root: SettlementStateRoot,
) -> Result<CheckBundle, StoreBackendError> {
    let Some(canon_exec) = &write_arts.canon_exec else {
        return Ok(CheckBundle {
            ids: CheckIds::empty(),
            bytes: CheckBytes {
                draft: Vec::new(),
                check: Vec::new(),
                exec: Vec::new(),
                link: Vec::new(),
            },
        });
    };

    let draft = CheckpointDraft::new_settlement(
        crate::checkpoint::CheckpointVersion::CURRENT,
        write_arts.version,
        prev_settlement_root,
        state_root,
        write_arts.spent.clone(),
        write_arts.created.clone(),
    );
    let draft_id = derive_draft_id(&draft).map_err(map_check)?;
    let proof = draft
        .attest_proof(write_arts.snap_id, canon_exec.exec_id)
        .map_err(map_check)?;
    let checkpoint = draft.finalize(proof).map_err(map_check)?;
    let check_id = derive_checkpoint_id(&checkpoint).map_err(map_check)?;
    let link = CheckpointLink::new(
        CheckpointLinkVersion::CURRENT,
        check_id,
        write_arts.snap_id,
        canon_exec.exec_id,
    )
    .map_err(map_check)?;

    Ok(CheckBundle {
        ids: CheckIds {
            draft_id: draft_id.into_bytes(),
            check_id: check_id.into_bytes(),
            exec_id: canon_exec.exec_id.into_bytes(),
        },
        bytes: CheckBytes {
            draft: encode_draft_bin(&draft).map_err(map_check)?,
            check: encode_art_bin(&checkpoint).map_err(map_check)?,
            exec: canon_exec.exec_bytes.clone(),
            link: encode_link_bin(&link).map_err(map_check)?,
        },
    })
}

fn inject_hjmt_stage(stage: &str) -> bool {
    env_opt(HJMT_INJ_STAGE_ENV).as_deref() == Some(stage)
}
