#![forbid(unsafe_code)]

use z00z_wallets::tx::{build_claim_tx_digest, build_tx_package_digest};

use crate::types::{
    decode_hex32, CanonicalDigest, RejectClass, RejectRecord, WorkItem, WorkPayload,
};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct IngressBoundary;

impl IngressBoundary {
    pub fn normalize(&self, payload: WorkPayload) -> Result<WorkItem, RejectRecord> {
        match payload {
            WorkPayload::Tx(pkg) => {
                let digest = canonical_tx_digest(&pkg)?;
                Ok(WorkItem::new(WorkPayload::Tx(pkg), digest))
            }
            WorkPayload::Claim(pkg) => {
                let digest = canonical_claim_digest(&pkg)?;
                Ok(WorkItem::new(WorkPayload::Claim(pkg), digest))
            }
        }
    }
}

fn canonical_tx_digest(pkg: &z00z_wallets::tx::TxPackage) -> Result<CanonicalDigest, RejectRecord> {
    let expected = build_tx_package_digest(
        &pkg.kind,
        &pkg.package_type,
        pkg.version,
        pkg.chain_id,
        &pkg.chain_type,
        &pkg.chain_name,
        &pkg.tx,
    )
    .map_err(|err| shape_invalid(format!("tx package digest build failed: {err}")))?;
    canonical_digest(&pkg.tx_digest_hex, expected)
}

fn canonical_claim_digest(
    pkg: &z00z_wallets::tx::ClaimTxPackage,
) -> Result<CanonicalDigest, RejectRecord> {
    let expected = build_claim_tx_digest(
        &pkg.kind,
        &pkg.package_type,
        pkg.version,
        pkg.chain_id,
        &pkg.chain_type,
        &pkg.chain_name,
        &pkg.tx,
    )
    .map_err(|err| shape_invalid(format!("claim package digest build failed: {err}")))?;
    canonical_digest(&pkg.tx_digest_hex, expected)
}

fn canonical_digest(actual: &str, expected: String) -> Result<CanonicalDigest, RejectRecord> {
    if actual != expected {
        return Err(shape_invalid("tx_digest_hex does not match payload"));
    }

    let bytes = decode_hex32(&expected)
        .map_err(|err| shape_invalid(format!("canonical digest must stay lowercase hex: {err}")))?;
    Ok(CanonicalDigest::new(expected, bytes))
}

fn shape_invalid(detail: impl Into<String>) -> RejectRecord {
    RejectRecord {
        intake_id: None,
        class: RejectClass::ShapeInvalid,
        detail: detail.into(),
    }
}

#[cfg(test)]
mod tests {
    use z00z_wallets::tx::{
        build_claim_tx_digest, build_tx_package_digest, ClaimAuthWire, ClaimContextWire,
        ClaimProofWire, ClaimTxPackage, ClaimTxWire, TxAuthWire, TxContextWire, TxPackage,
        TxProofWire, TxWire,
    };

    use super::*;

    #[test]
    fn test_tx_rejects_forged_digest() {
        let ingress = IngressBoundary;
        let payload = WorkPayload::Tx(Box::new(tx_package("aa".repeat(32))));

        let err = ingress
            .normalize(payload)
            .expect_err("forged tx digest must reject");

        assert_eq!(err.class, RejectClass::ShapeInvalid);
        assert!(err.detail.contains("does not match payload"));
        assert!(err.intake_id.is_none());
    }

    #[test]
    fn test_claim_rejects_forged_digest() {
        let ingress = IngressBoundary;
        let payload = WorkPayload::Claim(Box::new(claim_package("bb".repeat(32))));

        let err = ingress
            .normalize(payload)
            .expect_err("forged claim digest must reject");

        assert_eq!(err.class, RejectClass::ShapeInvalid);
        assert!(err.detail.contains("does not match payload"));
        assert!(err.intake_id.is_none());
    }

    #[test]
    fn test_normalize_rebinds_payload_digest() {
        let ingress = IngressBoundary;
        let pkg = tx_package(String::new());
        let expected = build_tx_package_digest(
            &pkg.kind,
            &pkg.package_type,
            pkg.version,
            pkg.chain_id,
            &pkg.chain_type,
            &pkg.chain_name,
            &pkg.tx,
        )
        .expect("digest");
        let payload = WorkPayload::Tx(Box::new(TxPackage {
            tx_digest_hex: expected.clone(),
            ..pkg
        }));

        let item = ingress.normalize(payload).expect("payload-bound digest");

        assert_eq!(item.digest_hex(), expected);
        assert_eq!(item.intake_id().digest_hex(), expected);
    }

    fn tx_package(tx_digest_hex: String) -> TxPackage {
        TxPackage {
            kind: "TxPackage".to_string(),
            package_type: "regular_tx".to_string(),
            version: 1,
            chain_id: 3,
            chain_type: "devnet".to_string(),
            chain_name: "z00z-devnet-1".to_string(),
            tx: TxWire {
                tx_type: "regular_tx".to_string(),
                inputs: Vec::new(),
                outputs: Vec::new(),
                fee: 0,
                nonce: 0,
                context: TxContextWire::default(),
                proof: TxProofWire::default(),
                auth: TxAuthWire::default(),
            },
            tx_digest_hex,
            status: "received".to_string(),
        }
    }

    fn claim_package(tx_digest_hex: String) -> ClaimTxPackage {
        let mut pkg = ClaimTxPackage {
            kind: "ClaimTxPackage".to_string(),
            package_type: "claim_tx".to_string(),
            version: 1,
            chain_id: 3,
            chain_type: "devnet".to_string(),
            chain_name: "z00z-devnet-1".to_string(),
            tx: ClaimTxWire {
                tx_type: "claim_tx".to_string(),
                inputs: Vec::new(),
                outputs: Vec::new(),
                fee: 0,
                nonce: 0,
                context: ClaimContextWire {
                    recipient_wallet_id: "wallet".to_string(),
                    recipient_owner_hex: "00".repeat(32),
                    claim_scope_hash_hex: "11".repeat(32),
                    recipient_card_hex: None,
                    nullifier_hex: "22".repeat(32),
                },
                proof: ClaimProofWire {
                    proof_type: "genesis_claim".to_string(),
                    proof_hex: "33".repeat(32),
                },
                auth: ClaimAuthWire {
                    claim_authority_sig_hex: "44".repeat(64),
                },
            },
            tx_digest_hex,
            status: "received".to_string(),
        };

        if pkg.tx_digest_hex.is_empty() {
            pkg.tx_digest_hex = build_claim_tx_digest(
                &pkg.kind,
                &pkg.package_type,
                pkg.version,
                pkg.chain_id,
                &pkg.chain_type,
                &pkg.chain_name,
                &pkg.tx,
            )
            .expect("claim digest");
        }

        pkg
    }
}
