//! Backup importer implementation.
//!
//! Phase 1: Validation-first, file-based backups.
//!
//! Notes:
//! - Reads a JSON-serialized backup container using z00z_utils::codec::JsonCodec.
//! - Reads bytes using z00z_utils::io::read_file.
//! - Validates checksum integrity.
//! - Decrypts using XChaCha20-Poly1305 and Argon2id-derived key.
//! - Decompresses payload via zstd.
//! - Accepts only the canonical `WalletExportPack` plus JSONL history authority
//!   shape; it must not infer a second restore contract.

use crate::backup::{BackupKdf, WalletBackupCrypto};
use crate::key::Z00ZKeyBranch;
use crate::wallet::persistence::WalletExportPack;
use z00z_crypto::aead;
use z00z_crypto::expert::encoding::SafePassword;
use z00z_utils::{
    codec::{Codec, JsonCodec},
    compression::zstd_decompress_bounded,
    io::{read_file, IoError},
};

use super::backup_wire::{
    decode_tx_history_jsonl, BackupAssociatedData, BackupContainer, BackupEncryption,
    BackupPayload, ForensicImportMode, WalletForensicPack, BACKUP_FORMAT_VERSION,
    BACKUP_MAX_PLAINTEXT_BYTES, BACKUP_NONCE_BYTES,
};
use super::{
    BackupImporter, BackupImporterError, BackupImporterResult, BackupMetadata, ImportedWalletData,
};

/// Default BackupImporter implementation.
#[derive(Debug)]
pub struct BackupImporterImpl;

impl BackupImporterImpl {
    /// Create a new backup importer.
    pub fn new() -> Self {
        Self
    }

    /// Read and validate a canonical wallet tx-history JSONL artifact.
    pub fn import_history_jsonl(
        path: &str,
    ) -> BackupImporterResult<Vec<crate::persistence::tx::TxRecord>> {
        let bytes = read_file(path).map_err(Self::map_io_error)?;
        decode_tx_history_jsonl(&bytes).map_err(BackupImporterError::InvalidFormat)
    }

    fn map_io_error(err: IoError) -> BackupImporterError {
        match err {
            IoError::Io(e) => BackupImporterError::Io(e),
            other => BackupImporterError::ImportFailed(other.to_string()),
        }
    }

    fn decode_container(bytes: &[u8]) -> BackupImporterResult<BackupContainer> {
        let codec = JsonCodec;
        codec
            .deserialize(bytes)
            .map_err(|e| BackupImporterError::Deserialization(e.to_string()))
    }

    fn validate_version(metadata: &BackupMetadata) -> BackupImporterResult<()> {
        if metadata.version != BACKUP_FORMAT_VERSION {
            return Err(BackupImporterError::VersionMismatch {
                expected: BACKUP_FORMAT_VERSION,
                found: metadata.version,
            });
        }
        Ok(())
    }

    fn resolve_kdf(encryption: &BackupEncryption) -> BackupImporterResult<BackupKdf> {
        encryption
            .kdf
            .to_params()
            .map_err(|e| BackupImporterError::InvalidFormat(e.to_string()))?;
        Ok(encryption.kdf.clone())
    }

    fn build_aad_bytes(aad: &BackupAssociatedData) -> BackupImporterResult<Vec<u8>> {
        let stripped = Self::aad_zero_nonce(aad);
        let codec = JsonCodec;
        let aad_json = codec
            .serialize(&stripped)
            .map_err(|e| BackupImporterError::InvalidFormat(e.to_string()))?;

        let ctx = [Z00ZKeyBranch::WalletBackup.as_aad_byte()];
        let prefix = aead::build_aad_multipart(Z00ZKeyBranch::WalletBackup.label(), &[&ctx[..]])
            .map_err(|e| BackupImporterError::InvalidFormat(e.to_string()))?;

        let checksum = WalletBackupCrypto::aad_tag(&aad_json);
        let mut tagged = Vec::with_capacity(prefix.len() + 32 + aad_json.len());
        tagged.extend_from_slice(&prefix);
        tagged.extend_from_slice(&checksum);
        tagged.extend_from_slice(&aad_json);
        Ok(tagged)
    }

    fn aad_zero_nonce(aad: &BackupAssociatedData) -> BackupAssociatedData {
        let mut out = aad.clone();
        out.encryption.nonce = [0u8; BACKUP_NONCE_BYTES];
        out
    }

    fn compute_checksum(aad_bytes: &[u8], ciphertext: &[u8]) -> [u8; 32] {
        WalletBackupCrypto::checksum(aad_bytes, ciphertext)
    }

    fn validate_integrity(container: &BackupContainer) -> BackupImporterResult<Vec<u8>> {
        let aad = BackupAssociatedData {
            metadata: container.metadata.clone(),
            encryption: container.encryption.clone(),
            compression: container.compression.clone(),
        };

        let aad_bytes = Self::build_aad_bytes(&aad)?;
        let expected = Self::compute_checksum(&aad_bytes, &container.ciphertext);
        if expected == container.checksum {
            return Ok(aad_bytes);
        }

        Err(BackupImporterError::IntegrityMismatch)
    }

    fn decrypt_plain(
        decoded: &BackupContainer,
        password: &SafePassword,
    ) -> BackupImporterResult<Vec<u8>> {
        let kdf = Self::resolve_kdf(&decoded.encryption)?;
        let aad = Self::validate_integrity(decoded)?;
        let key = WalletBackupCrypto::derive_key_with_kdf(password, &kdf)
            .map_err(|e| BackupImporterError::DecryptionFailed(e.to_string()))?;
        let comp = WalletBackupCrypto::decrypt(&key, &aad, &decoded.ciphertext)
            .map_err(|e| BackupImporterError::DecryptionFailed(e.to_string()))?;

        if decoded.compression.algorithm != "zstd" {
            return Err(BackupImporterError::InvalidFormat(
                "unsupported compression algorithm".to_string(),
            ));
        }

        zstd_decompress_bounded(&comp, BACKUP_MAX_PLAINTEXT_BYTES)
            .map_err(|e| BackupImporterError::ImportFailed(e.to_string()))
    }

    fn validate_export_pack(
        export_pack: WalletExportPack,
        expected_wallet_id: &str,
    ) -> BackupImporterResult<WalletExportPack> {
        if export_pack.seed_phrase.trim().is_empty() {
            return Err(BackupImporterError::InvalidFormat(
                "restore seed phrase is required".to_string(),
            ));
        }

        let profile = export_pack.wallet_profile.as_ref().ok_or_else(|| {
            BackupImporterError::InvalidFormat("backup payload missing wallet profile".to_string())
        })?;
        let profile = profile
            .clone()
            .migrate_to_current()
            .map_err(|e| BackupImporterError::InvalidFormat(e.to_string()))?;
        profile
            .verify_checksum()
            .map_err(|e| BackupImporterError::InvalidFormat(e.to_string()))?;
        if profile.wallet_id.0 != expected_wallet_id {
            return Err(BackupImporterError::InvalidFormat(
                "payload does not match metadata".to_string(),
            ));
        }

        let manifest = export_pack.manifest.as_ref().ok_or_else(|| {
            BackupImporterError::InvalidFormat("backup payload missing manifest".to_string())
        })?;
        manifest
            .verify_checksum()
            .map_err(|e| BackupImporterError::InvalidFormat(e.to_string()))?;
        if manifest.wallet_id.0 != expected_wallet_id {
            return Err(BackupImporterError::InvalidFormat(
                "payload does not match metadata".to_string(),
            ));
        }
        if export_pack.tx_history_plane.as_deref()
            != Some(crate::db::BackupManifestPayload::TX_HISTORY_JSONL)
        {
            return Err(BackupImporterError::InvalidFormat(
                "wallet export tx-history plane invalid".to_string(),
            ));
        }

        let mut seen_asset_ids = std::collections::BTreeSet::new();
        for payload in &export_pack.owned_assets {
            let migrated = payload
                .clone()
                .migrate_to_current()
                .map_err(|e| BackupImporterError::InvalidFormat(e.to_string()))?;
            migrated
                .verify_checksum()
                .map_err(|e| BackupImporterError::InvalidFormat(e.to_string()))?;
            let _ = migrated
                .validate_invariants()
                .map_err(|e| BackupImporterError::InvalidFormat(e.to_string()))?;
            if migrated.wallet_id.0 != expected_wallet_id {
                return Err(BackupImporterError::InvalidFormat(
                    "owned asset wallet id mismatch".to_string(),
                ));
            }
            if !seen_asset_ids.insert(migrated.asset_id) {
                return Err(BackupImporterError::InvalidFormat(
                    "duplicate owned asset id in backup payload".to_string(),
                ));
            }
        }

        let mut seen_object_keys = std::collections::BTreeSet::new();
        for payload in &export_pack.owned_objects {
            let migrated = match payload.clone() {
                crate::db::OwnedObjectPayload::Asset(_) => {
                    return Err(BackupImporterError::InvalidFormat(
                        "backup owned_objects must not carry asset variants".to_string(),
                    ));
                }
                crate::db::OwnedObjectPayload::Voucher(payload) => {
                    let migrated = payload
                        .migrate_to_current()
                        .map_err(|e| BackupImporterError::InvalidFormat(e.to_string()))?;
                    migrated
                        .verify_checksum()
                        .map_err(|e| BackupImporterError::InvalidFormat(e.to_string()))?;
                    migrated
                        .validate_invariants()
                        .map_err(|e| BackupImporterError::InvalidFormat(e.to_string()))?;
                    if migrated.wallet_id.0 != expected_wallet_id {
                        return Err(BackupImporterError::InvalidFormat(
                            "owned voucher wallet id mismatch".to_string(),
                        ));
                    }
                    crate::db::OwnedObjectPayload::Voucher(migrated)
                }
                crate::db::OwnedObjectPayload::Right(payload) => {
                    let migrated = payload
                        .migrate_to_current()
                        .map_err(|e| BackupImporterError::InvalidFormat(e.to_string()))?;
                    migrated
                        .verify_checksum()
                        .map_err(|e| BackupImporterError::InvalidFormat(e.to_string()))?;
                    migrated
                        .validate_invariants()
                        .map_err(|e| BackupImporterError::InvalidFormat(e.to_string()))?;
                    if migrated.wallet_id.0 != expected_wallet_id {
                        return Err(BackupImporterError::InvalidFormat(
                            "owned right wallet id mismatch".to_string(),
                        ));
                    }
                    crate::db::OwnedObjectPayload::Right(migrated)
                }
            };

            let family_tag = match migrated.family() {
                crate::db::OwnedObjectFamily::Asset => 1u8,
                crate::db::OwnedObjectFamily::Voucher => 2u8,
                crate::db::OwnedObjectFamily::Right => 3u8,
            };
            if !seen_object_keys.insert((family_tag, migrated.stable_object_key())) {
                return Err(BackupImporterError::InvalidFormat(
                    "duplicate owned object stable key in backup payload".to_string(),
                ));
            }
        }

        Ok(export_pack)
    }

    fn validate_forensic_archive(
        forensic: &WalletForensicPack,
        metadata: &BackupMetadata,
        network: &str,
        chain: &str,
    ) -> BackupImporterResult<()> {
        forensic
            .validate(metadata, network, chain)
            .map_err(BackupImporterError::InvalidFormat)
    }

    fn decode_payload(
        decoded: BackupContainer,
        plain: &[u8],
        mode: ForensicImportMode,
    ) -> BackupImporterResult<ImportedWalletData> {
        use z00z_utils::codec::{BincodeCodec, Codec};

        let payload: BackupPayload = JsonCodec
            .deserialize(plain)
            .map_err(|e| BackupImporterError::InvalidFormat(e.to_string()))?;

        if let Some(forensic) = payload.forensic.as_ref() {
            Self::validate_forensic_archive(
                forensic,
                &decoded.metadata,
                &payload.network,
                &payload.chain,
            )?;
        }

        let export_pack = match mode {
            ForensicImportMode::WalletOnly | ForensicImportMode::WalletPlusHistory => Some(
                Self::validate_export_pack(payload.export_pack, &decoded.metadata.wallet_id)?,
            ),
            ForensicImportMode::TxHistoryOnly => None,
        };

        let forensic_archive = match mode {
            ForensicImportMode::WalletOnly => None,
            ForensicImportMode::TxHistoryOnly | ForensicImportMode::WalletPlusHistory => {
                Some(payload.forensic.ok_or_else(|| {
                    BackupImporterError::InvalidFormat(
                        "forensic archive section is required".to_string(),
                    )
                })?)
            }
        };

        let transactions = forensic_archive
            .as_ref()
            .map(|archive| {
                archive.history_jsonl_bytes().map_err(|e| {
                    BackupImporterError::InvalidFormat(format!(
                        "forensic tx-history JSONL serialization failed: {e}"
                    ))
                })
            })
            .transpose()?
            .unwrap_or_default();

        let keys = export_pack
            .as_ref()
            .and_then(|pack| pack.keys.as_ref())
            .map(|payload| {
                BincodeCodec
                    .serialize(payload)
                    .map_err(|e| BackupImporterError::InvalidFormat(e.to_string()))
            })
            .transpose()?
            .unwrap_or_default();

        let assets = export_pack
            .as_ref()
            .map(|pack| {
                BincodeCodec
                    .serialize(&pack.owned_assets)
                    .map_err(|e| BackupImporterError::InvalidFormat(e.to_string()))
            })
            .transpose()?
            .unwrap_or_default();

        Ok(ImportedWalletData {
            wallet_id: decoded.metadata.wallet_id.clone(),
            network: payload.network,
            chain: payload.chain,
            export_pack,
            forensic_archive,
            keys,
            transactions,
            assets,
        })
    }
}

impl Default for BackupImporterImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl BackupImporter for BackupImporterImpl {
    fn import_with_mode(
        &self,
        path: &str,
        password: &SafePassword,
        mode: ForensicImportMode,
    ) -> BackupImporterResult<ImportedWalletData> {
        let bytes = read_file(path).map_err(Self::map_io_error)?;
        self.import_from_bytes_with_mode(&bytes, password, mode)
    }

    fn import_from_bytes_with_mode(
        &self,
        data: &[u8],
        password: &SafePassword,
        mode: ForensicImportMode,
    ) -> BackupImporterResult<ImportedWalletData> {
        let decoded = Self::decode_container(data)?;
        Self::validate_version(&decoded.metadata)?;
        let plaintext = Self::decrypt_plain(&decoded, password)?;
        Self::decode_payload(decoded, &plaintext, mode)
    }

    fn read_metadata(&self, path: &str) -> BackupImporterResult<BackupMetadata> {
        let bytes = read_file(path).map_err(Self::map_io_error)?;
        let decoded = Self::decode_container(&bytes)?;
        Self::validate_version(&decoded.metadata)?;
        Ok(decoded.metadata)
    }

    fn verify_password(&self, path: &str, password: &SafePassword) -> BackupImporterResult<bool> {
        let bytes = read_file(path).map_err(Self::map_io_error)?;
        let decoded = Self::decode_container(&bytes)?;
        Self::validate_version(&decoded.metadata)?;
        match Self::decrypt_plain(&decoded, password) {
            Ok(plaintext) => {
                Ok(
                    Self::decode_payload(decoded, &plaintext, ForensicImportMode::WalletOnly)
                        .is_ok(),
                )
            }
            Err(BackupImporterError::DecryptionFailed(_)) => Ok(false),
            Err(BackupImporterError::IntegrityMismatch) => Ok(false),
            Err(BackupImporterError::InvalidFormat(_)) => Ok(false),
            Err(BackupImporterError::ImportFailed(_)) => Ok(false),
            Err(err) => Err(err),
        }
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
#[path = "test_backup_importer_impl.rs"]
mod tests;
