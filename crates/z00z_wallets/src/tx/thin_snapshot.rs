use serde::Serialize;
use z00z_crypto::expert::encoding::ByteArray;
use z00z_crypto::{KernelSignature, Z00ZRistrettoPoint, Z00ZScalar};
use z00z_utils::codec::{Codec, JsonCodec};

use crate::{
    key::{derive_identity_public_key, sign_identity, verify_identity},
    tx::{verify_full_tx_package, TxInputWire, TxPackage, TxProofWire},
};

use super::thin_types::{
    ThinAssetPathRef, ThinIndexEntry, ThinIndexError, ThinSnapshot, ThinSnapshotContext,
    ThinSnapshotPin, ThinWalletTxPackage, THIN_SNAPSHOT_VERSION, THIN_TX_PACKAGE_VERSION,
};

const THIN_SNAPSHOT_SIGNATURE_CONTEXT: &[u8] = b"z00z.wallet.tx.thin.snapshot.v1";

#[derive(Serialize)]
struct ThinSnapshotUnsigned<'a> {
    snapshot_version: u16,
    context: &'a ThinSnapshotContext,
    entries: &'a [ThinIndexEntry],
}

#[derive(Serialize)]
struct ThinWalletTxUnsigned<'a> {
    package_version: u16,
    chain_id: &'a str,
    package_kind: &'a str,
    package_type: &'a str,
    tx_hash_hex: &'a str,
    snapshot_digest_hex: &'a str,
    compatibility_generation: u64,
    prev_root_hex: &'a str,
    checkpoint_id_hex: &'a Option<String>,
    snapshot_entry_id_hex: &'a str,
    input_refs: &'a [ThinAssetPathRef],
}

fn lowercase_hex<const N: usize>(
    value: &str,
    field: &'static str,
) -> Result<[u8; N], ThinIndexError> {
    let bytes = hex::decode(value).map_err(|_| ThinIndexError::InvalidHex {
        field,
        expected_len: N,
    })?;
    let bytes: [u8; N] = bytes.try_into().map_err(|_| ThinIndexError::InvalidHex {
        field,
        expected_len: N,
    })?;
    if hex::encode(bytes) != value {
        return Err(ThinIndexError::InvalidHex {
            field,
            expected_len: N,
        });
    }
    Ok(bytes)
}

fn encode_signature_hex(signature: &KernelSignature) -> String {
    let mut bytes = Vec::with_capacity(64);
    bytes.extend_from_slice(signature.get_public_nonce().as_bytes());
    bytes.extend_from_slice(signature.get_signature().as_bytes());
    hex::encode(bytes)
}

fn decode_signature_hex(value: &str) -> Result<KernelSignature, ThinIndexError> {
    let bytes = lowercase_hex::<64>(value, "signature_hex")?;
    let nonce = Z00ZRistrettoPoint::try_from_bytes(bytes[..32].try_into().expect("slice length"))
        .map_err(|_| ThinIndexError::InvalidSnapshotSignature)?;
    let scalar = Z00ZScalar::try_from_bytes(bytes[32..].try_into().expect("slice length"))
        .map_err(|_| ThinIndexError::InvalidSnapshotSignature)?;
    if scalar.is_zero() {
        return Err(ThinIndexError::InvalidSnapshotSignature);
    }
    Ok(KernelSignature::new(
        nonce.reveal().clone(),
        scalar.reveal().clone(),
    ))
}

fn package_prev_root_hex(pkg: &TxPackage) -> Result<&str, ThinIndexError> {
    pkg.tx
        .proof
        .spend
        .as_ref()
        .map(|proof| proof.prev_root_hex.as_str())
        .ok_or_else(|| {
            ThinIndexError::PackageVerificationFailed(
                "tx package is missing canonical spend proof context".to_string(),
            )
        })
}

fn proof_digest_hex(tx_proof: &TxProofWire) -> Result<String, ThinIndexError> {
    let bytes = JsonCodec.serialize(tx_proof).map_err(|error| {
        ThinIndexError::PackageVerificationFailed(format!("tx proof serialization failed: {error}"))
    })?;
    Ok(hex::encode(z00z_crypto::blake2b_hash(
        b"z00z.wallet.thin.proof_digest.v1",
        &[bytes.as_slice()],
    )))
}

fn derive_entry_id_hex(
    pkg: &TxPackage,
    input_refs: &[ThinAssetPathRef],
    prev_root_hex: &str,
) -> String {
    let mut preimage = format!(
        "{}:{}:{}:{}:{}",
        pkg.chain_id, pkg.kind, pkg.package_type, pkg.tx_digest_hex, prev_root_hex
    )
    .into_bytes();
    for input in input_refs {
        preimage.extend_from_slice(input.asset_id_hex.as_bytes());
        preimage.extend_from_slice(&input.serial_id.to_le_bytes());
    }
    hex::encode(z00z_crypto::blake2b_hash(
        b"z00z.wallet.thin.entry_id.v1",
        &[preimage.as_slice()],
    ))
}

fn verify_tx_bytes(tx_bytes: &[u8]) -> Result<(Vec<u8>, TxPackage), ThinIndexError> {
    let package: TxPackage = JsonCodec.deserialize(tx_bytes).map_err(|error| {
        ThinIndexError::PackageVerificationFailed(format!("tx package decode failed: {error}"))
    })?;
    let canonical = JsonCodec.serialize(&package).map_err(|error| {
        ThinIndexError::PackageVerificationFailed(format!(
            "tx package re-serialization failed: {error}"
        ))
    })?;
    let verify = verify_full_tx_package(&canonical).map_err(|error| {
        ThinIndexError::PackageVerificationFailed(format!(
            "tx package verification errored: {error}"
        ))
    })?;
    if !verify.valid {
        return Err(ThinIndexError::PackageVerificationFailed(
            verify.errors.join("; "),
        ));
    }
    Ok((canonical, package))
}

impl ThinAssetPathRef {
    /// Lift one canonical input wire into the thin helper path family.
    #[must_use]
    pub fn from_tx_input(input: &TxInputWire) -> Self {
        Self {
            asset_id_hex: input.asset_id_hex.clone(),
            serial_id: input.serial_id,
        }
    }
}

impl ThinSnapshotContext {
    fn check_shape(&self) -> Result<(), ThinIndexError> {
        if self.chain_id.trim().is_empty() {
            return Err(ThinIndexError::InvalidSnapshotShape(
                "chain_id must not be empty".to_string(),
            ));
        }
        if self.expires_at_ms <= self.issued_at_ms {
            return Err(ThinIndexError::InvalidSnapshotShape(
                "expires_at_ms must be greater than issued_at_ms".to_string(),
            ));
        }
        lowercase_hex::<32>(&self.prev_root_hex, "prev_root_hex")?;
        if let Some(checkpoint_id_hex) = &self.checkpoint_id_hex {
            lowercase_hex::<32>(checkpoint_id_hex, "checkpoint_id_hex")?;
        }
        Ok(())
    }

    fn check_at(&self, now_ms: u64) -> Result<(), ThinIndexError> {
        self.check_shape()?;
        if now_ms > self.expires_at_ms {
            return Err(ThinIndexError::SnapshotExpired {
                expires_at_ms: self.expires_at_ms,
                now_ms,
            });
        }
        Ok(())
    }
}

impl ThinIndexEntry {
    /// Build one helper entry over canonical thick package bytes.
    pub fn from_tx_bytes(tx_bytes: Vec<u8>) -> Result<Self, ThinIndexError> {
        let (canonical, package) = verify_tx_bytes(&tx_bytes)?;
        let input_refs = package
            .tx
            .inputs
            .iter()
            .map(ThinAssetPathRef::from_tx_input)
            .collect::<Vec<_>>();
        let prev_root_hex = package_prev_root_hex(&package)?.to_string();
        let proof_digest_hex = proof_digest_hex(&package.tx.proof)?;
        Ok(Self {
            entry_id_hex: derive_entry_id_hex(&package, &input_refs, &prev_root_hex),
            tx_hash_hex: package.tx_digest_hex.clone(),
            package_kind: package.kind.clone(),
            package_type: package.package_type.clone(),
            chain_id: package.chain_id.to_string(),
            prev_root_hex,
            proof_digest_hex,
            input_refs,
            tx_bytes: canonical,
        })
    }

    /// Verify the helper entry and return canonical package bytes plus the parsed package.
    pub fn verify_and_load(&self) -> Result<(Vec<u8>, TxPackage), ThinIndexError> {
        let (canonical, package) = verify_tx_bytes(&self.tx_bytes)?;
        if package.tx_digest_hex != self.tx_hash_hex {
            return Err(ThinIndexError::PackageDigestMismatch {
                expected: self.tx_hash_hex.clone(),
                actual: package.tx_digest_hex.clone(),
            });
        }
        if package.kind != self.package_kind {
            return Err(ThinIndexError::PackageKindMismatch {
                expected: self.package_kind.clone(),
                actual: package.kind.clone(),
            });
        }
        if package.package_type != self.package_type {
            return Err(ThinIndexError::PackageTypeMismatch {
                expected: self.package_type.clone(),
                actual: package.package_type.clone(),
            });
        }
        if package.chain_id.to_string() != self.chain_id {
            return Err(ThinIndexError::PackageChainMismatch {
                expected: self.chain_id.clone(),
                actual: package.chain_id.to_string(),
            });
        }
        let prev_root_hex = package_prev_root_hex(&package)?;
        if prev_root_hex != self.prev_root_hex {
            return Err(ThinIndexError::PackageRootMismatch {
                expected: self.prev_root_hex.clone(),
                actual: prev_root_hex.to_string(),
            });
        }
        let actual_proof_digest = proof_digest_hex(&package.tx.proof)?;
        if actual_proof_digest != self.proof_digest_hex {
            return Err(ThinIndexError::PackageDigestMismatch {
                expected: self.proof_digest_hex.clone(),
                actual: actual_proof_digest,
            });
        }
        let actual_refs = package
            .tx
            .inputs
            .iter()
            .map(ThinAssetPathRef::from_tx_input)
            .collect::<Vec<_>>();
        if actual_refs != self.input_refs {
            return Err(ThinIndexError::InputRefMismatch);
        }
        Ok((canonical, package))
    }
}

impl ThinSnapshot {
    fn unsigned_bytes(&self) -> Result<Vec<u8>, ThinIndexError> {
        let unsigned = ThinSnapshotUnsigned {
            snapshot_version: self.snapshot_version,
            context: &self.context,
            entries: &self.entries,
        };
        JsonCodec.serialize(&unsigned).map_err(|error| {
            ThinIndexError::InvalidSnapshotShape(format!(
                "snapshot unsigned-body serialization failed: {error}"
            ))
        })
    }

    /// Compute the digest over the unsigned snapshot body.
    pub fn compute_digest_hex(&self) -> Result<String, ThinIndexError> {
        let bytes = self.unsigned_bytes()?;
        Ok(hex::encode(z00z_crypto::blake2b_hash(
            b"z00z.wallet.thin.snapshot.digest.v1",
            &[bytes.as_slice()],
        )))
    }

    pub(crate) fn check_shape(&self) -> Result<(), ThinIndexError> {
        if self.snapshot_version != THIN_SNAPSHOT_VERSION {
            return Err(ThinIndexError::UnsupportedSnapshotVersion(
                self.snapshot_version,
            ));
        }
        lowercase_hex::<32>(&self.signer_identity_hex, "signer_identity_hex")?;
        lowercase_hex::<32>(&self.snapshot_digest_hex, "snapshot_digest_hex")?;
        decode_signature_hex(&self.signature_hex)?;
        self.context.check_shape()?;
        if self.entries.is_empty() {
            return Err(ThinIndexError::InvalidSnapshotShape(
                "entries must not be empty".to_string(),
            ));
        }
        let mut seen = std::collections::BTreeSet::new();
        for entry in &self.entries {
            if !seen.insert(entry.entry_id_hex.clone()) {
                return Err(ThinIndexError::EntryConflict(entry.entry_id_hex.clone()));
            }
            let _ = entry.verify_and_load()?;
            if entry.chain_id != self.context.chain_id {
                return Err(ThinIndexError::SnapshotContextMismatch {
                    field: "chain_id",
                    expected: self.context.chain_id.clone(),
                    actual: entry.chain_id.clone(),
                });
            }
            if entry.prev_root_hex != self.context.prev_root_hex {
                return Err(ThinIndexError::SnapshotContextMismatch {
                    field: "prev_root_hex",
                    expected: self.context.prev_root_hex.clone(),
                    actual: entry.prev_root_hex.clone(),
                });
            }
        }
        let expected_digest = self.compute_digest_hex()?;
        if expected_digest != self.snapshot_digest_hex {
            return Err(ThinIndexError::InvalidSnapshotDigest);
        }
        let public_key = Z00ZRistrettoPoint::try_from_bytes(lowercase_hex::<32>(
            &self.signer_identity_hex,
            "signer_identity_hex",
        )?)
        .map_err(|_| ThinIndexError::InvalidSnapshotSignature)?;
        let signature = decode_signature_hex(&self.signature_hex)?;
        verify_identity(
            &public_key,
            &self.unsigned_bytes()?,
            THIN_SNAPSHOT_SIGNATURE_CONTEXT,
            &signature,
        )
        .map_err(|_| ThinIndexError::InvalidSnapshotSignature)?;
        Ok(())
    }

    /// Verify the snapshot body, signature, and freshness at one policy time.
    pub fn verify_at(&self, now_ms: u64) -> Result<(), ThinIndexError> {
        self.check_shape()?;
        self.context.check_at(now_ms)
    }

    /// Build one signed snapshot over canonical entries.
    pub fn new_signed(
        context: ThinSnapshotContext,
        entries: Vec<ThinIndexEntry>,
        signer_identity_sk: &Z00ZScalar,
    ) -> Result<Self, ThinIndexError> {
        let identity_pk = derive_identity_public_key(signer_identity_sk).map_err(|error| {
            ThinIndexError::InvalidSnapshotShape(format!(
                "signer identity public key derivation failed: {error}"
            ))
        })?;
        let mut snapshot = Self {
            snapshot_version: THIN_SNAPSHOT_VERSION,
            signer_identity_hex: hex::encode(identity_pk.as_bytes()),
            snapshot_digest_hex: String::new(),
            signature_hex: String::new(),
            context,
            entries,
        };
        snapshot.snapshot_digest_hex = snapshot.compute_digest_hex()?;
        let signature = sign_identity(
            signer_identity_sk,
            &snapshot.unsigned_bytes()?,
            THIN_SNAPSHOT_SIGNATURE_CONTEXT,
        )
        .map_err(|error| {
            ThinIndexError::InvalidSnapshotShape(format!("signing thin snapshot failed: {error}"))
        })?;
        snapshot.signature_hex = encode_signature_hex(&signature);
        snapshot.check_shape()?;
        Ok(snapshot)
    }

    /// Return the store conflict key for this signed snapshot context.
    #[must_use]
    pub fn context_key(&self) -> String {
        format!(
            "{}:{}:{}:{}",
            self.context.chain_id,
            self.context.compatibility_generation,
            self.context.prev_root_hex,
            self.context.checkpoint_id_hex.as_deref().unwrap_or("-")
        )
    }

    /// Lookup one entry by id.
    pub fn entry(&self, entry_id_hex: &str) -> Result<&ThinIndexEntry, ThinIndexError> {
        self.entries
            .iter()
            .find(|entry| entry.entry_id_hex == entry_id_hex)
            .ok_or_else(|| ThinIndexError::EntryMissing(entry_id_hex.to_string()))
    }
}

impl ThinSnapshotPin {
    /// Pin one signed helper snapshot for wallet-side use.
    pub fn new(snapshot: &ThinSnapshot, now_ms: u64) -> Result<Self, ThinIndexError> {
        snapshot.verify_at(now_ms)?;
        Ok(Self {
            snapshot_digest_hex: snapshot.snapshot_digest_hex.clone(),
            chain_id: snapshot.context.chain_id.clone(),
            compatibility_generation: snapshot.context.compatibility_generation,
            prev_root_hex: snapshot.context.prev_root_hex.clone(),
            checkpoint_id_hex: snapshot.context.checkpoint_id_hex.clone(),
            expires_at_ms: snapshot.context.expires_at_ms,
        })
    }
}

impl ThinWalletTxPackage {
    fn unsigned_bytes(&self) -> Result<Vec<u8>, ThinIndexError> {
        let unsigned = ThinWalletTxUnsigned {
            package_version: self.package_version,
            chain_id: &self.chain_id,
            package_kind: &self.package_kind,
            package_type: &self.package_type,
            tx_hash_hex: &self.tx_hash_hex,
            snapshot_digest_hex: &self.snapshot_digest_hex,
            compatibility_generation: self.compatibility_generation,
            prev_root_hex: &self.prev_root_hex,
            checkpoint_id_hex: &self.checkpoint_id_hex,
            snapshot_entry_id_hex: &self.snapshot_entry_id_hex,
            input_refs: &self.input_refs,
        };
        JsonCodec.serialize(&unsigned).map_err(|error| {
            ThinIndexError::InvalidSnapshotShape(format!(
                "thin wrapper unsigned-body serialization failed: {error}"
            ))
        })
    }

    /// Compute the metadata hash over the unsigned wrapper body.
    pub fn compute_metadata_hash_hex(&self) -> Result<String, ThinIndexError> {
        let bytes = self.unsigned_bytes()?;
        Ok(hex::encode(z00z_crypto::blake2b_hash(
            b"z00z.wallet.thin.metadata_hash.v1",
            &[bytes.as_slice()],
        )))
    }

    /// Refresh the stored wrapper metadata hash after an intentional field edit.
    pub fn refresh_metadata_hash(&mut self) -> Result<(), ThinIndexError> {
        self.metadata_hash_hex = self.compute_metadata_hash_hex()?;
        Ok(())
    }

    /// Verify the unsigned wrapper body against the stored metadata hash.
    pub fn verify_metadata(&self) -> Result<(), ThinIndexError> {
        if self.package_version != THIN_TX_PACKAGE_VERSION {
            return Err(ThinIndexError::UnsupportedThinVersion(self.package_version));
        }
        lowercase_hex::<32>(&self.tx_hash_hex, "tx_hash_hex")?;
        lowercase_hex::<32>(&self.snapshot_digest_hex, "snapshot_digest_hex")?;
        lowercase_hex::<32>(&self.prev_root_hex, "prev_root_hex")?;
        lowercase_hex::<32>(&self.snapshot_entry_id_hex, "snapshot_entry_id_hex")?;
        if let Some(checkpoint_id_hex) = &self.checkpoint_id_hex {
            lowercase_hex::<32>(checkpoint_id_hex, "checkpoint_id_hex")?;
        }
        let expected = self.compute_metadata_hash_hex()?;
        if expected != self.metadata_hash_hex {
            return Err(ThinIndexError::InvalidMetadataHash);
        }
        Ok(())
    }

    /// Build one thin wrapper from a pinned snapshot plus one helper entry.
    pub fn new(pin: &ThinSnapshotPin, entry: &ThinIndexEntry) -> Result<Self, ThinIndexError> {
        let mut thin = Self {
            package_version: THIN_TX_PACKAGE_VERSION,
            chain_id: pin.chain_id.clone(),
            package_kind: entry.package_kind.clone(),
            package_type: entry.package_type.clone(),
            tx_hash_hex: entry.tx_hash_hex.clone(),
            snapshot_digest_hex: pin.snapshot_digest_hex.clone(),
            compatibility_generation: pin.compatibility_generation,
            prev_root_hex: pin.prev_root_hex.clone(),
            checkpoint_id_hex: pin.checkpoint_id_hex.clone(),
            snapshot_entry_id_hex: entry.entry_id_hex.clone(),
            input_refs: entry.input_refs.clone(),
            metadata_hash_hex: String::new(),
        };
        thin.refresh_metadata_hash()?;
        Ok(thin)
    }
}
