use crate::{
    domains::TxHashDomain,
    persistence::TxConfirmationEvidence,
    rpc::types::{
        common::PersistTxId,
        security::IdempotencyKey,
        tx::{
            PersistReceiptInfo, RuntimeAdmissionReceipt, RuntimeConfirmationReceipt,
            TxSubmitterRole,
        },
    },
    tx::TxPackage,
};
use z00z_core::assets::registry::AssetId;
use z00z_crypto::expert::{encoding::to_hex, traits::DomainSeparation};
use z00z_utils::codec::{Codec, JsonCodec};

#[derive(Debug, Clone)]
pub(crate) struct WalletTxAdmissionRequest {
    pub(crate) tx_id: PersistTxId,
    pub(crate) tx_hash_hex: String,
    pub(crate) tx_bytes: Vec<u8>,
    pub(crate) chain_id: u32,
    pub(crate) submitter_role: TxSubmitterRole,
    pub(crate) idempotency_key: Option<IdempotencyKey>,
    pub(crate) requested_at: u64,
}

#[derive(Debug, Clone)]
pub(crate) struct WalletTxAdmissionReceipt {
    pub(crate) tx_id: PersistTxId,
    pub(crate) tx_hash_hex: String,
    pub(crate) tx_bytes: Vec<u8>,
    pub(crate) chain_id: u32,
    pub(crate) submitter_role: TxSubmitterRole,
    pub(crate) admission_id_hex: String,
    pub(crate) admitted_at: u64,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum AdmissionError {
    #[error("tx admission package decode failed: {0}")]
    PackageDecode(String),
    #[error("tx admission verification failed: {0}")]
    PackageVerify(String),
    #[error("tx admission hash mismatch")]
    HashMismatch,
    #[error("tx admission chain mismatch")]
    ChainMismatch,
    #[error("tx admission idempotency key is invalid")]
    InvalidIdempotencyKey,
}

pub(crate) trait WalletTxAdmitter: Send + Sync {
    fn admit(
        &self,
        request: WalletTxAdmissionRequest,
    ) -> Result<WalletTxAdmissionReceipt, AdmissionError>;

    fn confirm(
        &self,
        receipt: &WalletTxAdmissionReceipt,
    ) -> Result<RuntimeConfirmationReceipt, AdmissionError>;
}

#[derive(Debug, Default)]
pub(crate) struct SimulatedWalletTxAdmitter;

impl SimulatedWalletTxAdmitter {
    fn domain_hash(label: &[u8], tx_hash_hex: &str, chain_id: u32) -> String {
        let chain_bytes = chain_id.to_be_bytes();
        to_hex(&z00z_crypto::derive_hash(
            TxHashDomain::domain().as_bytes(),
            &[label, tx_hash_hex.as_bytes(), &chain_bytes],
        ))
    }

    fn package(receipt: &WalletTxAdmissionReceipt) -> Result<TxPackage, AdmissionError> {
        JsonCodec
            .deserialize::<TxPackage>(&receipt.tx_bytes)
            .map_err(|error| AdmissionError::PackageDecode(error.to_string()))
    }

    fn output_ids(package: &TxPackage) -> Vec<String> {
        package
            .tx
            .outputs
            .iter()
            .filter_map(|output| output.asset_wire.clone().to_asset().ok())
            .map(|asset| hex::encode(asset.asset_id()))
            .collect()
    }

    fn input_ids(package: &TxPackage) -> Vec<String> {
        package
            .tx
            .inputs
            .iter()
            .filter_map(|input| decode_input_id(&input.asset_id_hex))
            .map(hex::encode)
            .collect()
    }
}

impl WalletTxAdmitter for SimulatedWalletTxAdmitter {
    fn admit(
        &self,
        request: WalletTxAdmissionRequest,
    ) -> Result<WalletTxAdmissionReceipt, AdmissionError> {
        let package: TxPackage = JsonCodec
            .deserialize(&request.tx_bytes)
            .map_err(|error| AdmissionError::PackageDecode(error.to_string()))?;
        let verify = crate::tx::verify_full_tx_package(&request.tx_bytes)
            .map_err(|error| AdmissionError::PackageVerify(error.to_string()))?;

        if !verify.valid {
            return Err(AdmissionError::PackageVerify(verify.errors.join("; ")));
        }

        if package.chain_id != request.chain_id {
            return Err(AdmissionError::ChainMismatch);
        }

        if package.tx_digest_hex != request.tx_hash_hex {
            return Err(AdmissionError::HashMismatch);
        }

        if request
            .idempotency_key
            .as_ref()
            .is_some_and(|key| !key.is_valid())
        {
            return Err(AdmissionError::InvalidIdempotencyKey);
        }

        Ok(WalletTxAdmissionReceipt {
            tx_id: request.tx_id,
            tx_hash_hex: request.tx_hash_hex.clone(),
            tx_bytes: request.tx_bytes,
            chain_id: request.chain_id,
            submitter_role: request.submitter_role,
            admission_id_hex: Self::domain_hash(
                b"wallet_tx_admission",
                &request.tx_hash_hex,
                request.chain_id,
            ),
            admitted_at: request.requested_at,
        })
    }

    fn confirm(
        &self,
        receipt: &WalletTxAdmissionReceipt,
    ) -> Result<RuntimeConfirmationReceipt, AdmissionError> {
        let package = Self::package(receipt)?;
        let checkpoint_id_hex = Self::domain_hash(
            b"wallet_tx_checkpoint",
            &receipt.tx_hash_hex,
            receipt.chain_id,
        );
        let prev_root_hex = Self::domain_hash(
            b"wallet_tx_prev_root",
            &receipt.tx_hash_hex,
            receipt.chain_id,
        );
        let new_root_hex = Self::domain_hash(
            b"wallet_tx_new_root",
            &receipt.tx_hash_hex,
            receipt.chain_id,
        );
        let mut height_bytes = [0u8; 8];
        let raw_height = z00z_crypto::derive_hash(
            TxHashDomain::domain().as_bytes(),
            &[b"wallet_tx_height", receipt.tx_hash_hex.as_bytes()],
        );
        height_bytes.copy_from_slice(&raw_height[..8]);

        Ok(RuntimeConfirmationReceipt {
            tx_id: receipt.tx_id.clone(),
            tx_hash_hex: receipt.tx_hash_hex.clone(),
            block_height: u64::from_be_bytes(height_bytes).max(1),
            checkpoint_id_hex,
            prev_root_hex,
            new_root_hex,
            spent_asset_ids_hex: Self::input_ids(&package),
            created_asset_ids_hex: Self::output_ids(&package),
            confirmed_at: receipt.admitted_at,
            verified: true,
        })
    }
}

impl From<&WalletTxAdmissionReceipt> for RuntimeAdmissionReceipt {
    fn from(receipt: &WalletTxAdmissionReceipt) -> Self {
        Self {
            tx_id: receipt.tx_id.clone(),
            tx_hash_hex: receipt.tx_hash_hex.clone(),
            chain_id: receipt.chain_id,
            submitter_role: receipt.submitter_role,
            admission_id_hex: receipt.admission_id_hex.clone(),
            admitted_at: receipt.admitted_at,
            verified: true,
        }
    }
}

pub(crate) fn receipt_to_persist(receipt: &RuntimeConfirmationReceipt) -> PersistReceiptInfo {
    PersistReceiptInfo {
        tx_id: receipt.tx_id.clone(),
        block_height: receipt.block_height,
        block_hash: receipt.checkpoint_id_hex.clone(),
        tx_index: 0,
        confirmations: 1,
        confirmed_at: receipt.confirmed_at,
        verified: receipt.verified,
        merkle_proof: None,
    }
}

pub(crate) fn confirmation_to_evidence(
    receipt: &RuntimeConfirmationReceipt,
    chain_id: u32,
) -> TxConfirmationEvidence {
    TxConfirmationEvidence {
        tx_id: receipt.tx_id.0.clone(),
        tx_hash_hex: receipt.tx_hash_hex.clone(),
        chain_id,
        block_height: receipt.block_height,
        checkpoint_id_hex: receipt.checkpoint_id_hex.clone(),
        prev_root_hex: receipt.prev_root_hex.clone(),
        new_root_hex: receipt.new_root_hex.clone(),
        spent_asset_ids_hex: receipt.spent_asset_ids_hex.clone(),
        created_asset_ids_hex: receipt.created_asset_ids_hex.clone(),
        confirmed_at: receipt.confirmed_at,
        verified: receipt.verified,
    }
}

pub(crate) fn evidence_to_confirmation(
    evidence: &TxConfirmationEvidence,
) -> RuntimeConfirmationReceipt {
    RuntimeConfirmationReceipt {
        tx_id: PersistTxId::new(evidence.tx_id.clone()),
        tx_hash_hex: evidence.tx_hash_hex.clone(),
        block_height: evidence.block_height,
        checkpoint_id_hex: evidence.checkpoint_id_hex.clone(),
        prev_root_hex: evidence.prev_root_hex.clone(),
        new_root_hex: evidence.new_root_hex.clone(),
        spent_asset_ids_hex: evidence.spent_asset_ids_hex.clone(),
        created_asset_ids_hex: evidence.created_asset_ids_hex.clone(),
        confirmed_at: evidence.confirmed_at,
        verified: evidence.verified,
    }
}

fn decode_input_id(value: &str) -> Option<AssetId> {
    let bytes = hex::decode(value).ok()?;
    let bytes: [u8; 32] = bytes.try_into().ok()?;
    if hex::encode(bytes) != value {
        return None;
    }
    Some(bytes)
}
