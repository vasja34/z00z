//! Backup exporter implementation.
//!
//! Phase 1: Validation-first, file-based backups.
//!
//! Notes:
//! - Produces a JSON-serialized backup container using z00z_utils::codec::JsonCodec.
//! - Persists bytes using z00z_utils::io::write_file.
//! - Encrypts the compressed payload with XChaCha20-Poly1305.
//! - Derives the encryption key from the provided password using Argon2id.
//! - Includes a domain-separated checksum for explicit integrity validation.
//! - Preserves the canonical `WalletExportPack` plus JSONL history authority
//!   shape instead of introducing a second wallet-state bundle.

use std::convert::TryInto;

use crate::backup::{BackupKdf, WalletBackupCrypto};
use crate::key::Z00ZKeyBranch;
use crate::persistence::tx::TxRecord;
use crate::wallet::persistence::WalletExportPack;
use z00z_crypto::aead;
use z00z_crypto::expert::encoding::SafePassword;
use z00z_utils::{
    codec::{Codec, JsonCodec},
    compression::{zstd_compress, zstd_decompress_bounded},
    io::{read_file, write_file, IoError},
    rng::{RngCoreExt, SecureRngProvider},
    time::TimeProvider,
};

use super::backup_wire::{
    decode_tx_history_jsonl, BackupAssociatedData, BackupCompression, BackupContainer,
    BackupEncryption, BackupPayload, WalletForensicPack, BACKUP_FORMAT_VERSION,
    BACKUP_MAX_PLAINTEXT_BYTES, BACKUP_NONCE_BYTES, BACKUP_SALT_BYTES,
};
use super::{BackupExporter, BackupExporterError, BackupExporterResult, BackupMetadata};

#[cfg(test)]
#[path = "test_backup_exporter_suite.rs"]
mod test_backup_exporter_suite;

/// Default BackupExporter implementation.
#[derive(Debug)]
pub struct BackupExporterImpl<T: TimeProvider, R: SecureRngProvider> {
    wallet_id: String,
    network: String,
    chain: String,
    export_pack: WalletExportPack,
    forensic_history: Option<Vec<TxRecord>>,
    time_provider: T,
    rng_provider: R,
}

impl<T: TimeProvider, R: SecureRngProvider> BackupExporterImpl<T, R> {
    fn new_inner(
        wallet_id: String,
        network: String,
        chain: String,
        export_pack: WalletExportPack,
        forensic_history: Option<Vec<TxRecord>>,
        time_provider: T,
        rng_provider: R,
    ) -> Self {
        Self {
            wallet_id,
            network,
            chain,
            export_pack,
            forensic_history,
            time_provider,
            rng_provider,
        }
    }

    fn build_encryption(&self) -> (BackupEncryption, BackupKdf) {
        let mut salt = [0u8; BACKUP_SALT_BYTES];
        let mut rng = self.rng_provider.rng();
        rng.fill_bytes_ext(&mut salt);

        let kdf = BackupKdf::default(salt);
        let encryption = BackupEncryption {
            algorithm: "xchacha20poly1305".to_string(),
            kdf: kdf.clone(),
            salt: None,
            nonce: [0u8; BACKUP_NONCE_BYTES],
        };

        (encryption, kdf)
    }

    fn encrypt_payload(
        password: &SafePassword,
        kdf: &BackupKdf,
        aad_bytes: &[u8],
        plain: &[u8],
    ) -> BackupExporterResult<(Vec<u8>, [u8; BACKUP_NONCE_BYTES])> {
        let comp =
            zstd_compress(plain).map_err(|e| BackupExporterError::ExportFailed(e.to_string()))?;
        let key = WalletBackupCrypto::derive_key_with_kdf(password, kdf)
            .map_err(|e| BackupExporterError::EncryptionFailed(e.to_string()))?;
        let ciphertext = WalletBackupCrypto::encrypt(&key, aad_bytes, &comp)
            .map_err(|e| BackupExporterError::EncryptionFailed(e.to_string()))?;
        let nonce = ciphertext
            .get(1..(1 + BACKUP_NONCE_BYTES))
            .ok_or_else(|| {
                BackupExporterError::EncryptionFailed("ciphertext missing nonce".to_string())
            })?
            .try_into()
            .map_err(|_| {
                BackupExporterError::EncryptionFailed("ciphertext nonce invalid".to_string())
            })?;
        Ok((ciphertext, nonce))
    }

    /// Create a new backup exporter.
    pub fn new(
        wallet_id: String,
        network: String,
        export_pack: WalletExportPack,
        time_provider: T,
        rng_provider: R,
    ) -> Self {
        Self::new_inner(
            wallet_id,
            network,
            String::new(),
            export_pack,
            None,
            time_provider,
            rng_provider,
        )
    }

    /// Create a backup exporter with an explicit chain-bound restore identity.
    pub fn new_with_chain(
        wallet_id: String,
        network: String,
        chain: String,
        export_pack: WalletExportPack,
        time_provider: T,
        rng_provider: R,
    ) -> Self {
        Self::new_inner(
            wallet_id,
            network,
            chain,
            export_pack,
            None,
            time_provider,
            rng_provider,
        )
    }

    /// Create a backup exporter with an explicit forensic tx-history envelope.
    pub fn new_with_forensic_history(
        wallet_id: String,
        network: String,
        chain: String,
        export_pack: WalletExportPack,
        forensic_history: Vec<TxRecord>,
        time_provider: T,
        rng_provider: R,
    ) -> Self {
        Self::new_inner(
            wallet_id,
            network,
            chain,
            export_pack,
            Some(forensic_history),
            time_provider,
            rng_provider,
        )
    }

    /// Export an encrypted forensic backup from exact live tx-history JSONL bytes.
    pub fn export_with_history_bytes(
        &self,
        path: &str,
        password: &SafePassword,
        history_jsonl: &[u8],
    ) -> BackupExporterResult<BackupMetadata> {
        if self.forensic_history.is_none() {
            return Err(BackupExporterError::ExportFailed(
                "forensic export requires explicit tx-history records".to_string(),
            ));
        }

        self.validate_history_jsonl_bytes(history_jsonl)?;
        let container = self.export_with_history_jsonl(password, history_jsonl)?;

        let codec = JsonCodec;
        let bytes = codec
            .serialize(&container)
            .map_err(|e| BackupExporterError::Serialization(e.to_string()))?;

        write_file(path, &bytes).map_err(Self::map_io_error)?;
        Ok(container.metadata)
    }

    fn resolve_kdf(encryption: &BackupEncryption) -> BackupExporterResult<BackupKdf> {
        encryption
            .kdf
            .to_params()
            .map_err(|e| BackupExporterError::InvalidFormat(e.to_string()))?;
        Ok(encryption.kdf.clone())
    }

    fn map_io_error(err: IoError) -> BackupExporterError {
        match err {
            IoError::Io(e) => BackupExporterError::Io(e),
            other => BackupExporterError::ExportFailed(other.to_string()),
        }
    }

    fn validate_history_jsonl_bytes(&self, bytes: &[u8]) -> BackupExporterResult<()> {
        if let Some(records) = &self.forensic_history {
            let decoded =
                decode_tx_history_jsonl(bytes).map_err(BackupExporterError::ExportFailed)?;
            if decoded != *records {
                return Err(BackupExporterError::ExportFailed(
                    "canonical tx-history JSONL does not match forensic history".to_string(),
                ));
            }
        }

        Ok(())
    }

    fn build_metadata(&self) -> BackupMetadata {
        BackupMetadata {
            version: BACKUP_FORMAT_VERSION,
            created_at: self.time_provider.compat_unix_timestamp_millis(),
            wallet_id: self.wallet_id.clone(),
            network: String::new(),
        }
    }

    fn build_aad_bytes(aad: &BackupAssociatedData) -> BackupExporterResult<Vec<u8>> {
        let stripped = Self::aad_zero_nonce(aad);
        let codec = JsonCodec;
        let aad_json = codec
            .serialize(&stripped)
            .map_err(|e| BackupExporterError::Serialization(e.to_string()))?;

        let ctx = [Z00ZKeyBranch::WalletBackup.as_aad_byte()];
        let prefix = aead::build_aad_multipart(Z00ZKeyBranch::WalletBackup.label(), &[&ctx[..]])
            .map_err(|e| BackupExporterError::ExportFailed(e.to_string()))?;

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
}

include!("backup_exporter_verify.rs");

impl<T: TimeProvider, R: SecureRngProvider> BackupExporter for BackupExporterImpl<T, R> {
    fn export(&self, path: &str, password: &SafePassword) -> BackupExporterResult<BackupMetadata> {
        if self.forensic_history.is_some() {
            return Err(BackupExporterError::ExportFailed(
                "forensic export requires export_with_history_bytes".to_string(),
            ));
        }

        let container = self.export_to_container(password)?;
        let codec = JsonCodec;
        let bytes = codec
            .serialize(&container)
            .map_err(|e| BackupExporterError::Serialization(e.to_string()))?;

        write_file(path, &bytes).map_err(Self::map_io_error)?;
        Ok(container.metadata)
    }

    fn export_to_bytes(&self, _password: &SafePassword) -> BackupExporterResult<Vec<u8>> {
        let container = self.export_to_container(_password)?;
        let codec = JsonCodec;
        codec
            .serialize(&container)
            .map_err(|e| BackupExporterError::Serialization(e.to_string()))
    }

    fn get_metadata(&self) -> BackupExporterResult<BackupMetadata> {
        Ok(self.build_metadata())
    }

    fn verify_backup(&self, path: &str, password: &SafePassword) -> BackupExporterResult<bool> {
        let bytes = read_file(path).map_err(Self::map_io_error)?;
        let codec = JsonCodec;
        let decoded: BackupContainer = codec
            .deserialize(&bytes)
            .map_err(|e| BackupExporterError::InvalidFormat(e.to_string()))?;

        if decoded.metadata.wallet_id != self.wallet_id {
            return Ok(false);
        }

        let Some(plaintext) = Self::decrypt_payload(&decoded, password)? else {
            return Ok(false);
        };

        Self::verify_payload_matches_metadata(&decoded, &plaintext, &self.network, &self.chain)
    }
}

impl<T: TimeProvider, R: SecureRngProvider> BackupExporterImpl<T, R> {
    fn export_to_container(
        &self,
        password: &SafePassword,
    ) -> BackupExporterResult<BackupContainer> {
        if self.forensic_history.is_some() {
            return Err(BackupExporterError::ExportFailed(
                "forensic export requires canonical tx-history JSONL bytes".to_string(),
            ));
        }

        self.export_to_container_inner(password, None)
    }

    fn build_forensic_archive(
        &self,
        history_jsonl: &[u8],
        metadata: BackupMetadata,
    ) -> BackupExporterResult<WalletForensicPack> {
        let records = self.forensic_history.as_ref().ok_or_else(|| {
            BackupExporterError::ExportFailed(
                "forensic export requires explicit tx-history records".to_string(),
            )
        })?;

        WalletForensicPack::build_with_history_jsonl(
            metadata,
            self.network.clone(),
            self.chain.clone(),
            records.clone(),
            history_jsonl.to_vec(),
        )
        .map_err(BackupExporterError::ExportFailed)
    }

    fn export_with_history_jsonl(
        &self,
        password: &SafePassword,
        history_jsonl: &[u8],
    ) -> BackupExporterResult<BackupContainer> {
        let metadata = self.build_metadata();
        let forensic = self.build_forensic_archive(history_jsonl, metadata.clone())?;
        self.export_inner_with_metadata(password, metadata, Some(forensic))
    }

    fn export_to_container_inner(
        &self,
        password: &SafePassword,
        forensic: Option<WalletForensicPack>,
    ) -> BackupExporterResult<BackupContainer> {
        let metadata = self.build_metadata();
        self.export_inner_with_metadata(password, metadata, forensic)
    }

    fn export_inner_with_metadata(
        &self,
        password: &SafePassword,
        metadata: BackupMetadata,
        forensic: Option<WalletForensicPack>,
    ) -> BackupExporterResult<BackupContainer> {
        let mut export_pack = self.export_pack.clone();
        let export_wallet_id = export_pack
            .wallet_profile
            .as_ref()
            .map(|profile| profile.wallet_id.0.clone())
            .ok_or_else(|| {
                BackupExporterError::ExportFailed("export pack missing wallet profile".to_string())
            })?;

        if export_wallet_id != self.wallet_id {
            return Err(BackupExporterError::ExportFailed(
                "export pack wallet id does not match backup metadata".to_string(),
            ));
        }

        if let Some(manifest) = export_pack.manifest.as_mut() {
            manifest.network = self.network.clone();
            manifest.chain = self.chain.clone();
            manifest.tx_record_count = forensic
                .as_ref()
                .map(|archive| archive.records.len() as u32)
                .unwrap_or(0);
            manifest.checksum = Some(manifest.compute_checksum());
        }

        let compression = BackupCompression {
            algorithm: "zstd".to_string(),
        };
        let (mut encryption, kdf) = self.build_encryption();

        let codec = JsonCodec;
        let payload = BackupPayload {
            network: self.network.clone(),
            chain: self.chain.clone(),
            export_pack,
            forensic,
        };
        let plaintext = codec
            .serialize(&payload)
            .map_err(|e| BackupExporterError::Serialization(e.to_string()))?;

        let aad = BackupAssociatedData {
            metadata: metadata.clone(),
            encryption: encryption.clone(),
            compression: compression.clone(),
        };
        let aad_bytes = Self::build_aad_bytes(&aad)?;
        let (ciphertext, nonce) = Self::encrypt_payload(password, &kdf, &aad_bytes, &plaintext)?;

        encryption.nonce = nonce;

        let checksum = Self::compute_checksum(&aad_bytes, &ciphertext);

        let container = BackupContainer {
            metadata,
            encryption,
            compression,
            checksum,
            ciphertext,
        };

        if !self.verify_container_integrity(&container) {
            return Err(BackupExporterError::IntegrityMismatch);
        }

        Ok(container)
    }
}
