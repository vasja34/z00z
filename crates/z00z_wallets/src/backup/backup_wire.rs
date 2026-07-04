use crate::{persistence::tx::TxRecord, wallet::persistence::WalletExportPack};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use z00z_utils::codec::{Codec, JsonCodec};

use super::{BackupKdf, BackupMetadata};

pub(super) const BACKUP_FORMAT_VERSION: u32 = 4;
pub(super) const BACKUP_SALT_BYTES: usize = 16;
pub(super) const BACKUP_NONCE_BYTES: usize = 24;
pub(super) const BACKUP_MAX_PLAINTEXT_BYTES: usize = 64 * 1024 * 1024;
pub const FORENSIC_ARCHIVE_VERSION: u32 = 1;
pub const FORENSIC_HISTORY_SCHEMA_VERSION: u32 = 1;

/// Explicit import intent for forensic archive consumers.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ForensicImportMode {
    /// Restore only wallet state.
    WalletOnly,
    /// Import only tx-history records.
    TxHistoryOnly,
    /// Restore wallet state and tx-history records.
    WalletPlusHistory,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(super) struct BackupEncryption {
    pub algorithm: String,
    pub kdf: BackupKdf,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub salt: Option<[u8; BACKUP_SALT_BYTES]>,
    pub nonce: [u8; BACKUP_NONCE_BYTES],
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(super) struct BackupCompression {
    pub algorithm: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(super) struct BackupAssociatedData {
    pub metadata: BackupMetadata,
    pub encryption: BackupEncryption,
    pub compression: BackupCompression,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(super) struct BackupContainer {
    pub metadata: BackupMetadata,
    pub encryption: BackupEncryption,
    pub compression: BackupCompression,
    pub checksum: [u8; 32],
    pub ciphertext: Vec<u8>,
}

/// Manifest entry for one archived tx-history record.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WalletTxHistoryManifestEntry {
    /// Physical JSONL row sequence.
    pub sequence: u64,
    /// Stored tx hash label.
    pub tx_hash: String,
    /// Journal operation kind for this row.
    pub entry_kind: WalletTxHistoryEntryKind,
    /// Hash of the serialized tx record.
    pub record_hash: [u8; 32],
    /// Hash of the stored tx_bytes payload.
    pub tx_bytes_hash: [u8; 32],
    /// Hash of this physical JSONL entry.
    pub entry_hash: [u8; 32],
}

/// Manifest metadata for an archived tx-history set.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WalletTxHistoryManifest {
    /// Number of physical journal rows captured in the archive.
    pub record_count: usize,
    /// Bounded manifest entries.
    pub entries: Vec<WalletTxHistoryManifestEntry>,
}

/// Canonical append-only operation kind for one tx-history JSONL row.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum WalletTxHistoryEntryKind {
    /// First locally-created package row.
    Created,
    /// Row imported from a forensic backup or external JSONL.
    Imported,
    /// Row recorded during an explicit export action.
    Exported,
    /// Transaction was submitted.
    Submitted,
    /// Transaction was admitted by the network/mempool.
    Admitted,
    /// Transaction was confirmed on-chain.
    Confirmed,
    /// Transaction failed.
    Failed,
    /// Transaction was cancelled.
    Cancelled,
    /// Transaction conflicted with durable wallet or asset state.
    Conflicted,
    /// Transaction was rejected because an input was already spent.
    AlreadySpent,
    /// Forensic delete marker. The package remains in this row.
    Tombstoned,
}

/// Canonical plaintext JSONL row for wallet tx-history replay.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WalletTxHistoryJsonlEntry {
    /// Tx-history JSONL schema version.
    pub schema_version: u32,
    /// Wallet stem owning this physical journal.
    pub wallet_stem: String,
    /// Monotonic physical row sequence, starting at 1.
    pub sequence: u64,
    /// Wall-clock time when this journal row was recorded.
    pub recorded_at_ms: u64,
    /// Stored tx hash label.
    pub tx_hash: String,
    /// Journal operation kind.
    pub entry_kind: WalletTxHistoryEntryKind,
    /// Hash of the serialized tx record.
    pub record_hash: [u8; 32],
    /// Hash of the stored tx_bytes payload.
    pub tx_bytes_hash: [u8; 32],
    /// Entry hash from the immediately previous physical row.
    pub previous_entry_hash: Option<[u8; 32]>,
    /// Hash of this physical row, excluding this field itself.
    pub entry_hash: [u8; 32],
    /// Full replayable tx-history record.
    pub record: TxRecord,
}

#[derive(Debug, Clone, Serialize)]
struct WalletTxHistoryHashInput<'a> {
    schema_version: u32,
    wallet_stem: &'a str,
    sequence: u64,
    recorded_at_ms: u64,
    tx_hash: &'a str,
    entry_kind: WalletTxHistoryEntryKind,
    record_hash: [u8; 32],
    tx_bytes_hash: [u8; 32],
    previous_entry_hash: Option<[u8; 32]>,
    record: &'a TxRecord,
}

/// Optional forensic archive payload attached to the encrypted backup payload.
///
/// This preserves the same wallet authority packet that already lives in
/// `WalletExportPack` plus the explicit JSONL tx-history plane. It must not be
/// treated as a second wallet-state authority or a compact-only export format.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WalletForensicPack {
    /// Archive envelope version.
    pub version: u32,
    /// Tx-history schema version.
    pub schema_version: u32,
    /// Export metadata copied from the outer backup container.
    pub export_metadata: BackupMetadata,
    /// Network identity for the archived history.
    pub network: String,
    /// Chain identity for the archived history.
    pub chain: String,
    /// History manifest metadata and entry hashes.
    pub manifest: WalletTxHistoryManifest,
    /// Serialized tx-history records.
    pub records: Vec<TxRecord>,
    /// Raw canonical tx-history JSONL bytes, preserved exactly as stored on disk.
    pub history_jsonl: Vec<u8>,
}

impl WalletTxHistoryManifestEntry {
    fn build(row: &WalletTxHistoryJsonlEntry) -> Result<Self, String> {
        row.validate()?;
        Ok(Self {
            sequence: row.sequence,
            tx_hash: row.tx_hash.clone(),
            entry_kind: row.entry_kind,
            record_hash: row.record_hash,
            tx_bytes_hash: row.tx_bytes_hash,
            entry_hash: row.entry_hash,
        })
    }
}

impl WalletTxHistoryJsonlEntry {
    /// Build a canonical append-only JSONL entry.
    pub fn build_event(
        wallet_stem: &str,
        sequence: u64,
        recorded_at_ms: u64,
        entry_kind: WalletTxHistoryEntryKind,
        previous_entry_hash: Option<[u8; 32]>,
        record: TxRecord,
    ) -> Result<Self, String> {
        Self::build_event_with_source(
            wallet_stem,
            sequence,
            recorded_at_ms,
            entry_kind,
            previous_entry_hash,
            record,
        )
    }

    fn build_event_with_source(
        wallet_stem: &str,
        sequence: u64,
        recorded_at_ms: u64,
        entry_kind: WalletTxHistoryEntryKind,
        previous_entry_hash: Option<[u8; 32]>,
        record: TxRecord,
    ) -> Result<Self, String> {
        validate_wallet_stem(wallet_stem)?;
        validate_tx_hash_label(&record.tx_hash)?;
        if sequence == 0 {
            return Err("tx-history sequence must start at 1".to_string());
        }

        let mut row = Self {
            schema_version: FORENSIC_HISTORY_SCHEMA_VERSION,
            wallet_stem: wallet_stem.to_string(),
            sequence,
            recorded_at_ms,
            tx_hash: record.tx_hash.clone(),
            entry_kind,
            record_hash: hash_tx_record(&record)?,
            tx_bytes_hash: hash_tx_bytes(&record.tx_bytes),
            previous_entry_hash,
            entry_hash: [0u8; 32],
            record,
        };
        row.entry_hash = row.compute_entry_hash()?;
        row.validate()?;
        Ok(row)
    }

    fn validate(&self) -> Result<(), String> {
        if self.schema_version != FORENSIC_HISTORY_SCHEMA_VERSION {
            return Err(format!(
                "tx-history JSONL schema mismatch: expected {}, found {}",
                FORENSIC_HISTORY_SCHEMA_VERSION, self.schema_version
            ));
        }

        validate_wallet_stem(&self.wallet_stem)?;
        if self.sequence == 0 {
            return Err("tx-history sequence must start at 1".to_string());
        }

        validate_tx_hash_label(&self.tx_hash)?;
        if self.tx_hash != self.record.tx_hash {
            return Err(format!(
                "tx hash label mismatch: jsonl={}, record={}",
                self.tx_hash, self.record.tx_hash
            ));
        }

        if self.record_hash != hash_tx_record(&self.record)? {
            return Err(format!(
                "record hash mismatch for tx hash: {}",
                self.tx_hash
            ));
        }

        if self.tx_bytes_hash != hash_tx_bytes(&self.record.tx_bytes) {
            return Err(format!(
                "tx_bytes hash mismatch for tx hash: {}",
                self.tx_hash
            ));
        }

        if self.entry_hash != self.compute_entry_hash()? {
            return Err(format!("entry hash mismatch for tx hash: {}", self.tx_hash));
        }

        Ok(())
    }

    fn hash_input(&self) -> WalletTxHistoryHashInput<'_> {
        WalletTxHistoryHashInput {
            schema_version: self.schema_version,
            wallet_stem: &self.wallet_stem,
            sequence: self.sequence,
            recorded_at_ms: self.recorded_at_ms,
            tx_hash: &self.tx_hash,
            entry_kind: self.entry_kind,
            record_hash: self.record_hash,
            tx_bytes_hash: self.tx_bytes_hash,
            previous_entry_hash: self.previous_entry_hash,
            record: &self.record,
        }
    }

    fn compute_entry_hash(&self) -> Result<[u8; 32], String> {
        let bytes = JsonCodec
            .serialize(&self.hash_input())
            .map_err(|err| format!("tx-history entry serialization failed: {err}"))?;
        Ok(z00z_crypto::blake2b_hash(
            b"z00z.wallet.tx_history.entry_hash.v1",
            &[bytes.as_slice()],
        ))
    }
}

impl WalletTxHistoryManifest {
    fn build(rows: &[WalletTxHistoryJsonlEntry]) -> Result<Self, String> {
        validate_tx_history_rows(rows)?;
        let mut entries = Vec::with_capacity(rows.len());

        for row in rows {
            let entry = WalletTxHistoryManifestEntry::build(row)?;
            entries.push(entry);
        }

        Ok(Self {
            record_count: rows.len(),
            entries,
        })
    }

    fn validate(&self, rows: &[WalletTxHistoryJsonlEntry]) -> Result<(), String> {
        validate_tx_history_rows(rows)?;

        if self.record_count != rows.len() {
            return Err(format!(
                "tx-history row count mismatch: manifest={}, rows={}",
                self.record_count,
                rows.len()
            ));
        }

        if self.entries.len() != rows.len() {
            return Err(format!(
                "tx-history entry count mismatch: manifest={}, records={}",
                self.entries.len(),
                rows.len()
            ));
        }

        for (entry, row) in self.entries.iter().zip(rows.iter()) {
            validate_tx_hash_label(&entry.tx_hash)?;

            if entry.sequence != row.sequence {
                return Err(format!(
                    "tx-history sequence mismatch: manifest={}, row={}",
                    entry.sequence, row.sequence
                ));
            }

            if entry.tx_hash != row.tx_hash {
                return Err(format!(
                    "tx hash label mismatch: manifest={}, record={}",
                    entry.tx_hash, row.tx_hash
                ));
            }

            if entry.entry_kind != row.entry_kind {
                return Err(format!(
                    "tx-history entry kind mismatch for tx hash: {}",
                    entry.tx_hash
                ));
            }

            if entry.record_hash != row.record_hash {
                return Err(format!(
                    "record hash mismatch for tx hash: {}",
                    entry.tx_hash
                ));
            }

            if entry.tx_bytes_hash != row.tx_bytes_hash {
                return Err(format!(
                    "tx_bytes hash mismatch for tx hash: {}",
                    entry.tx_hash
                ));
            }

            if entry.entry_hash != row.entry_hash {
                return Err(format!(
                    "entry hash mismatch for tx hash: {}",
                    entry.tx_hash
                ));
            }
        }

        Ok(())
    }
}

/// Encode physical tx-history journal rows as canonical JSONL bytes.
pub fn encode_tx_history_rows(rows: &[WalletTxHistoryJsonlEntry]) -> Result<Vec<u8>, String> {
    validate_tx_history_rows(rows)?;
    let codec = JsonCodec;
    let mut out = Vec::new();

    for entry in rows {
        let mut line = codec
            .serialize(&entry)
            .map_err(|err| format!("tx-history JSONL serialization failed: {err}"))?;
        out.append(&mut line);
        out.push(b'\n');
    }

    Ok(out)
}

/// Encode current tx-history records as `Created` journal rows.
pub fn encode_tx_history_jsonl(wallet_stem: &str, records: &[TxRecord]) -> Result<Vec<u8>, String> {
    validate_wallet_stem(wallet_stem)?;

    let mut rows = Vec::with_capacity(records.len());
    let mut seen = HashSet::new();

    for record in records.iter().cloned() {
        if !seen.insert(record.tx_hash.clone()) {
            return Err(format!("duplicate tx hash label: {}", record.tx_hash));
        }

        let previous_entry_hash = rows
            .last()
            .map(|row: &WalletTxHistoryJsonlEntry| row.entry_hash);
        let entry = WalletTxHistoryJsonlEntry::build_event(
            wallet_stem,
            rows.len() as u64 + 1,
            record.timestamp_ms,
            WalletTxHistoryEntryKind::Created,
            previous_entry_hash,
            record,
        )?;
        rows.push(entry);
    }

    encode_tx_history_rows(&rows)
}

/// Decode and validate physical wallet tx-history JSONL journal rows.
pub fn decode_tx_history_rows(bytes: &[u8]) -> Result<Vec<WalletTxHistoryJsonlEntry>, String> {
    let codec = JsonCodec;
    let mut rows = Vec::new();

    for (index, raw_line) in bytes.split(|byte| *byte == b'\n').enumerate() {
        if raw_line.is_empty() {
            continue;
        }

        let entry: WalletTxHistoryJsonlEntry = codec
            .deserialize(raw_line)
            .map_err(|err| format!("tx-history JSONL line {} decode failed: {err}", index + 1))?;
        entry.validate()?;
        rows.push(entry);
    }

    validate_tx_history_rows(&rows)?;
    Ok(rows)
}

/// Decode and validate the canonical wallet JSONL tx-history artifact.
pub fn decode_tx_history_jsonl(bytes: &[u8]) -> Result<Vec<TxRecord>, String> {
    let rows = decode_tx_history_rows(bytes)?;
    Ok(fold_tx_history_rows(&rows))
}

/// Fold physical tx-history journal rows into the current non-tombstoned view.
pub fn fold_tx_history_rows(rows: &[WalletTxHistoryJsonlEntry]) -> Vec<TxRecord> {
    let mut current: Vec<TxRecord> = Vec::new();

    for row in rows {
        if row.entry_kind == WalletTxHistoryEntryKind::Tombstoned {
            current.retain(|record| record.tx_hash != row.tx_hash);
            continue;
        }

        if let Some(record) = current
            .iter_mut()
            .find(|record| record.tx_hash == row.tx_hash)
        {
            *record = row.record.clone();
        } else {
            current.push(row.record.clone());
        }
    }

    current
}

impl WalletForensicPack {
    /// Build a versioned forensic archive envelope from raw canonical JSONL bytes.
    pub fn build_with_history_jsonl(
        export_metadata: BackupMetadata,
        network: String,
        chain: String,
        records: Vec<TxRecord>,
        history_jsonl: Vec<u8>,
    ) -> Result<Self, String> {
        let rows = decode_tx_history_rows(&history_jsonl)?;
        let decoded_history = fold_tx_history_rows(&rows);
        if decoded_history != records {
            return Err("forensic JSONL payload mismatch".to_string());
        }

        if export_metadata.wallet_id.trim().is_empty() {
            return Err("export metadata wallet_id is required".to_string());
        }
        if network.trim().is_empty() {
            return Err("forensic archive network is required".to_string());
        }
        if chain.trim().is_empty() {
            return Err("forensic archive chain is required".to_string());
        }

        Ok(Self {
            version: FORENSIC_ARCHIVE_VERSION,
            schema_version: FORENSIC_HISTORY_SCHEMA_VERSION,
            export_metadata,
            network,
            chain,
            manifest: WalletTxHistoryManifest::build(&rows)?,
            records,
            history_jsonl,
        })
    }

    /// Return the preserved canonical JSONL bytes.
    pub fn history_jsonl_bytes(&self) -> Result<Vec<u8>, String> {
        Ok(self.history_jsonl.clone())
    }

    /// Validate the archive against the outer container metadata and chain identity.
    pub fn validate(
        &self,
        expected_metadata: &BackupMetadata,
        expected_network: &str,
        expected_chain: &str,
    ) -> Result<(), String> {
        if self.version != FORENSIC_ARCHIVE_VERSION {
            return Err(format!(
                "forensic archive version mismatch: expected {}, found {}",
                FORENSIC_ARCHIVE_VERSION, self.version
            ));
        }

        if self.schema_version != FORENSIC_HISTORY_SCHEMA_VERSION {
            return Err(format!(
                "forensic history schema mismatch: expected {}, found {}",
                FORENSIC_HISTORY_SCHEMA_VERSION, self.schema_version
            ));
        }

        if &self.export_metadata != expected_metadata {
            return Err("forensic export metadata mismatch".to_string());
        }

        if self.network != expected_network {
            return Err(format!(
                "forensic network mismatch: expected {expected_network}, found {}",
                self.network
            ));
        }

        if self.chain != expected_chain {
            return Err(format!(
                "forensic chain mismatch: expected {expected_chain}, found {}",
                self.chain
            ));
        }

        let rows = decode_tx_history_rows(&self.history_jsonl)?;
        let decoded = fold_tx_history_rows(&rows);
        if decoded != self.records {
            return Err("forensic JSONL payload mismatch".to_string());
        }
        self.manifest.validate(&rows)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct BackupPayload {
    pub network: String,
    pub chain: String,
    pub export_pack: WalletExportPack,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub forensic: Option<WalletForensicPack>,
}

fn validate_tx_hash_label(tx_hash: &str) -> Result<(), String> {
    if tx_hash.is_empty()
        || tx_hash.contains('/')
        || tx_hash.contains('\\')
        || tx_hash.contains("..")
    {
        return Err(format!("invalid tx hash label: {tx_hash}"));
    }

    Ok(())
}

fn validate_wallet_stem(wallet_stem: &str) -> Result<(), String> {
    if wallet_stem.is_empty()
        || wallet_stem.contains('/')
        || wallet_stem.contains('\\')
        || wallet_stem.contains("..")
    {
        return Err(format!("invalid wallet stem: {wallet_stem}"));
    }

    Ok(())
}

fn validate_tx_history_rows(rows: &[WalletTxHistoryJsonlEntry]) -> Result<(), String> {
    let mut previous_entry_hash = None;
    let mut wallet_stem: Option<&str> = None;

    for (index, row) in rows.iter().enumerate() {
        row.validate()?;

        let expected_sequence = index as u64 + 1;
        if row.sequence != expected_sequence {
            return Err(format!(
                "tx-history sequence mismatch: expected {}, found {}",
                expected_sequence, row.sequence
            ));
        }

        if row.previous_entry_hash != previous_entry_hash {
            return Err(format!(
                "tx-history hash chain mismatch at sequence {}",
                row.sequence
            ));
        }

        match wallet_stem {
            Some(stem) if stem != row.wallet_stem => {
                return Err(format!(
                    "tx-history wallet stem mismatch: expected {}, found {}",
                    stem, row.wallet_stem
                ));
            }
            Some(_) => {}
            None => wallet_stem = Some(&row.wallet_stem),
        }

        previous_entry_hash = Some(row.entry_hash);
    }

    Ok(())
}

fn hash_tx_record(record: &TxRecord) -> Result<[u8; 32], String> {
    let bytes = JsonCodec
        .serialize(record)
        .map_err(|err| format!("tx record serialization failed: {err}"))?;
    Ok(z00z_crypto::blake2b_hash(
        b"z00z.wallet.tx_history.record_hash.v1",
        &[bytes.as_slice()],
    ))
}

fn hash_tx_bytes(tx_bytes: &[u8]) -> [u8; 32] {
    z00z_crypto::blake2b_hash(b"z00z.wallet.tx_history.tx_bytes_hash.v1", &[tx_bytes])
}

#[cfg(test)]
mod tests {
    use super::{
        decode_tx_history_rows, encode_tx_history_rows, fold_tx_history_rows,
        WalletTxHistoryEntryKind, WalletTxHistoryJsonlEntry,
    };
    use crate::persistence::tx::{TxRecord, TxStatus};
    use z00z_utils::codec::{Codec, JsonCodec};

    fn sample_record(status: TxStatus) -> TxRecord {
        TxRecord {
            tx_hash: "tx-history-1".to_string(),
            tx_bytes: vec![1, 2, 3, 4],
            imported: false,
            status,
            timestamp_ms: 1_700_000_000_000,
            block_height: None,
            confirmation_evidence: None,
        }
    }

    fn build_row(
        sequence: u64,
        entry_kind: WalletTxHistoryEntryKind,
        previous_entry_hash: Option<[u8; 32]>,
        record: TxRecord,
    ) -> WalletTxHistoryJsonlEntry {
        WalletTxHistoryJsonlEntry::build_event(
            "abc",
            sequence,
            record.timestamp_ms,
            entry_kind,
            previous_entry_hash,
            record,
        )
        .expect("build tx-history row")
    }

    #[test]
    fn test_entry_kinds_encode_decode() {
        let cases = [
            WalletTxHistoryEntryKind::Conflicted,
            WalletTxHistoryEntryKind::AlreadySpent,
        ];

        for entry_kind in cases {
            let row = build_row(1, entry_kind, None, sample_record(TxStatus::Failed));
            let bytes =
                encode_tx_history_rows(std::slice::from_ref(&row)).expect("encode tx-history rows");
            let json = std::str::from_utf8(&bytes).expect("jsonl utf8");
            let expected = match entry_kind {
                WalletTxHistoryEntryKind::Conflicted => "\"entry_kind\":\"Conflicted\"",
                WalletTxHistoryEntryKind::AlreadySpent => "\"entry_kind\":\"AlreadySpent\"",
                _ => unreachable!("new entry kind table only"),
            };
            assert!(json.contains(expected));

            let decoded = decode_tx_history_rows(&bytes).expect("decode tx-history rows");
            assert_eq!(decoded.len(), 1);
            assert_eq!(decoded[0].entry_kind, entry_kind);
        }
    }

    #[test]
    fn test_entry_kinds_roundtrip() {
        let codec = JsonCodec;
        for entry_kind in [
            WalletTxHistoryEntryKind::Conflicted,
            WalletTxHistoryEntryKind::AlreadySpent,
        ] {
            let bytes = codec.serialize(&entry_kind).expect("serialize entry kind");
            let decoded: WalletTxHistoryEntryKind =
                codec.deserialize(&bytes).expect("deserialize entry kind");
            assert_eq!(decoded, entry_kind);
        }
    }

    #[test]
    fn test_prefers_terminal_rows() {
        for entry_kind in [
            WalletTxHistoryEntryKind::Conflicted,
            WalletTxHistoryEntryKind::AlreadySpent,
        ] {
            let created = build_row(
                1,
                WalletTxHistoryEntryKind::Created,
                None,
                sample_record(TxStatus::Pending),
            );
            let terminal = build_row(
                2,
                entry_kind,
                Some(created.entry_hash),
                sample_record(TxStatus::Failed),
            );
            let rows = vec![created, terminal];

            let folded = fold_tx_history_rows(&rows);
            assert_eq!(folded.len(), 1);
            assert_eq!(folded[0].tx_hash, "tx-history-1");
            assert_eq!(folded[0].status, TxStatus::Failed);
        }
    }
}
