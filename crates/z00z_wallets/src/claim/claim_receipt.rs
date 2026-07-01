use crate::{key::ReceiverKeys, WalletError};
use z00z_crypto::KernelSignature;

/// Domain separator for claim-receipt signatures.
pub const CLAIM_CTX: &[u8] = b"z00z.wallet.claim_receipt.v1";
/// Current claim-receipt schema version.
pub const CLAIM_SCHEMA: u8 = 1;

/// Signed claim receipt payload for one asset and one wallet scope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaimReceipt {
    /// Receipt schema version.
    pub schema_ver: u8,
    /// Claimed asset id.
    pub asset_id: [u8; 32],
    /// Wallet identifier bytes.
    pub wallet_id: Vec<u8>,
    /// Scope hash (`scenario_1/stage_3/{chain}`).
    pub claim_scope: [u8; 32],
    /// Signer identity public key bytes.
    pub identity_pk: [u8; 32],
}

impl ClaimReceipt {
    /// Validates receipt fields before signing/verifying.
    pub fn validate(&self) -> Result<(), WalletError> {
        if self.schema_ver != CLAIM_SCHEMA {
            return Err(WalletError::InvalidParams(format!(
                "unsupported claim schema version: {}",
                self.schema_ver
            )));
        }

        if self.wallet_id.is_empty() {
            return Err(WalletError::InvalidParams(
                "wallet_id must not be empty".to_string(),
            ));
        }

        if self.claim_scope == [0u8; 32] {
            return Err(WalletError::InvalidParams(
                "claim_scope must not be all-zero".to_string(),
            ));
        }

        if self.identity_pk == [0u8; 32] {
            return Err(WalletError::InvalidParams(
                "identity_pk must not be all-zero".to_string(),
            ));
        }

        Ok(())
    }

    /// Serializes receipt into signing payload bytes.
    pub fn to_msg(&self) -> Result<Vec<u8>, WalletError> {
        self.validate()?;
        let mut out = Vec::with_capacity(1 + 32 + 2 + self.wallet_id.len() + 32 + 32);
        out.push(self.schema_ver);
        out.extend_from_slice(&self.asset_id);

        let wlen: u16 = self
            .wallet_id
            .len()
            .try_into()
            .map_err(|_| WalletError::InvalidParams("wallet_id too long".to_string()))?;
        out.extend_from_slice(&wlen.to_le_bytes());
        out.extend_from_slice(&self.wallet_id);

        out.extend_from_slice(&self.claim_scope);
        out.extend_from_slice(&self.identity_pk);
        Ok(out)
    }
}

/// Computes claim scope hash for provided chain id.
pub fn claim_scope_hash(chain: &str) -> [u8; 32] {
    z00z_crypto::hash::poseidon2_hash(
        b"z00z/scenario_1/stage_3/claim_scope",
        &[b"scenario_1", b"stage_3", chain.as_bytes()],
    )
}

/// Signs claim receipt with wallet identity secret key derived from receiver keys.
pub fn sign_claim_receipt(
    keys: &ReceiverKeys,
    receipt: &ClaimReceipt,
) -> Result<KernelSignature, WalletError> {
    if receipt.identity_pk != keys.identity_pk.to_bytes() {
        return Err(WalletError::InvalidParams(
            "receipt.identity_pk does not match signer identity key".to_string(),
        ));
    }

    let msg = receipt.to_msg()?;
    crate::key::sign_identity(keys.reveal_identity_sk(), &msg, CLAIM_CTX)
        .map_err(|e| WalletError::InvalidParams(format!("claim sign failed: {e}")))
}

/// Verifies claim receipt signature using `receipt.identity_pk`.
pub fn verify_claim_receipt(
    receipt: &ClaimReceipt,
    sig: &KernelSignature,
) -> Result<(), WalletError> {
    let msg = receipt.to_msg()?;
    let identity_pk = z00z_crypto::Z00ZRistrettoPoint::try_from_bytes(receipt.identity_pk)
        .map_err(|e| WalletError::InvalidParams(format!("invalid identity_pk bytes: {e}")))?;

    crate::key::verify_identity(&identity_pk, &msg, CLAIM_CTX, sig)
        .map_err(|e| WalletError::InvalidParams(format!("claim verify failed: {e}")))
}

#[cfg(test)]
mod tests {
    use super::{
        claim_scope_hash, sign_claim_receipt, verify_claim_receipt, ClaimReceipt, CLAIM_SCHEMA,
    };
    use crate::key::{ReceiverKeys, ReceiverSecret};

    fn make_keys(seed: u8) -> ReceiverKeys {
        let sec = ReceiverSecret::from_bytes([seed; 32]).expect("receiver secret");
        ReceiverKeys::from_receiver_secret(sec).expect("receiver keys")
    }

    #[test]
    fn test_claim_receipt_sign_verify() {
        let keys_a = make_keys(11);
        let keys_b = make_keys(77);

        let receipt = ClaimReceipt {
            schema_ver: CLAIM_SCHEMA,
            asset_id: [9u8; 32],
            wallet_id: b"wallet-a".to_vec(),
            claim_scope: claim_scope_hash("dev-chain"),
            identity_pk: keys_a.identity_pk.to_bytes(),
        };

        let sig = sign_claim_receipt(&keys_a, &receipt).expect("sign");
        verify_claim_receipt(&receipt, &sig).expect("verify with A");

        let mut wrong = receipt.clone();
        wrong.identity_pk = keys_b.identity_pk.to_bytes();
        let bad = verify_claim_receipt(&wrong, &sig);
        assert!(bad.is_err(), "verify must fail for B");
    }

    #[test]
    fn test_receipt_rejects_mismatched_identity() {
        let keys_a = make_keys(11);
        let keys_b = make_keys(77);

        let receipt = ClaimReceipt {
            schema_ver: CLAIM_SCHEMA,
            asset_id: [1u8; 32],
            wallet_id: b"wallet-a".to_vec(),
            claim_scope: claim_scope_hash("dev-chain"),
            identity_pk: keys_b.identity_pk.to_bytes(),
        };

        let err = sign_claim_receipt(&keys_a, &receipt).expect_err("must reject mismatch");
        let msg = err.to_string();
        assert!(
            msg.contains("receipt.identity_pk does not match"),
            "unexpected error: {msg}"
        );
    }
}
