//! Verified consumers for portable claim packages.

use std::{
    collections::{BTreeSet, HashSet},
    path::Path,
};

use z00z_core::AssetWire;
use z00z_crypto::ClaimSourceProof;
use z00z_storage::settlement::{
    ClaimNullTx, ClaimNullifier, DefinitionId, SerialId, SettlementListReq, SettlementPath,
    SettlementStore, StoreItem, StoreOp, TerminalId,
};
use z00z_utils::config::{ConfigSource, EnvConfig};
use z00z_utils::{
    codec::{Codec, JsonCodec},
    io::read_file_bounded,
};
use z00z_wallets::claim::NullifierLease;
use z00z_wallets::tx::{
    asset_wire_to_leaf, claim_tx::require_claim_auth_simulator_anchor, ClaimTxPackage,
    ClaimTxVerifier, ClaimTxVerifierImpl,
};

use super::stage_3::CLAIM_STORE_FILE;

pub(crate) use super::claim_pkg_store::{
    claim_nulls, reserve_nulls, rollback_leases, with_pkg_store,
};
use super::claim_pkg_store::{commit_leases, load_reserved_nulls};

const CLAIM_PKG_BUNDLE_KIND: &str = "TxPackageBundle";
const CLAIM_PKG_BUNDLE_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
/// Explicit file wrapper for persisted claim package bundles.
pub struct ClaimTxBundle {
    /// Bundle kind discriminator.
    pub kind: String,
    /// Bundle subtype discriminator.
    pub package_type: String,
    /// Bundle schema version.
    pub version: u32,
    /// Claim packages carried by this file.
    pub packages: Vec<ClaimTxPackage>,
}

/// Wrap claim packages into the canonical persisted file envelope.
pub fn wrap_claim_packages(packages: Vec<ClaimTxPackage>) -> ClaimTxBundle {
    ClaimTxBundle {
        kind: CLAIM_PKG_BUNDLE_KIND.to_string(),
        package_type: "claim_tx".to_string(),
        version: 1,
        packages,
    }
}

/// Maximum size accepted for `tx_claim_pkg.json`.
pub const CLAIM_PKG_MAX_BYTES: u64 = 256 * 1024 * 1024;

/// Summary of a successful claim publish into storage.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ClaimStorePublishSummary {
    /// Number of claim packages consumed from the file.
    pub package_count: usize,
    /// Number of portable leaves extracted from all packages.
    pub leaf_count: usize,
    /// Number of JMT insert operations applied to the store.
    pub inserted_count: usize,
}

const FAIL_CLAIM_PUB: &str = "Z00Z_FAIL_CLAIM_PUB";

fn serialize_claim_packages(
    packages: &[ClaimTxPackage],
    source: &str,
) -> Result<Vec<Vec<u8>>, String> {
    let mut out = Vec::with_capacity(packages.len());
    for (idx, pkg) in packages.iter().enumerate() {
        let bytes = JsonCodec
            .serialize(pkg)
            .map_err(|e| format!("{source} re-serialize failed at index {idx}: {e}"))?;
        out.push(bytes);
    }
    Ok(out)
}

fn decode_claim_bundle(raw: &[u8]) -> Result<Vec<Vec<u8>>, String> {
    let bundle: ClaimTxBundle = JsonCodec
        .deserialize(raw)
        .map_err(|e| format!("claim package bundle parse failed: {e}"))?;

    if bundle.kind != CLAIM_PKG_BUNDLE_KIND {
        return Err(format!(
            "claim package bundle kind mismatch: expected '{CLAIM_PKG_BUNDLE_KIND}', got '{}'",
            bundle.kind
        ));
    }
    if bundle.package_type != "claim_tx" {
        return Err(format!(
            "claim package bundle package_type mismatch: expected 'claim_tx', got '{}'",
            bundle.package_type
        ));
    }
    if bundle.version != CLAIM_PKG_BUNDLE_VERSION {
        return Err(format!(
            "claim package bundle version mismatch: expected {CLAIM_PKG_BUNDLE_VERSION}, got {}",
            bundle.version
        ));
    }
    if bundle.packages.is_empty() {
        return Err("claim package bundle must carry at least one package".to_string());
    }

    serialize_claim_packages(&bundle.packages, "claim package bundle")
}

fn load_claim_rows(path: &Path) -> Result<Vec<Vec<u8>>, String> {
    let raw = read_file_bounded(path, CLAIM_PKG_MAX_BYTES).map_err(|e| e.to_string())?;
    decode_claim_bundle(&raw)
}

fn rollback_leases_with_error<T>(
    leases: &[NullifierLease],
    msg: impl Into<String>,
) -> Result<T, String> {
    rollback_leases(leases);
    Err(msg.into())
}

fn decode_claim_source_proof(
    pkg_idx: usize,
    pkg: &ClaimTxPackage,
) -> Result<ClaimSourceProof, String> {
    let proof_bytes = hex::decode(&pkg.tx.proof.proof_hex)
        .map_err(|_| format!("package[{pkg_idx}] claim proof hex decode failed"))?;
    ClaimSourceProof::from_bytes(&proof_bytes)
        .map_err(|err| format!("package[{pkg_idx}] claim proof decode failed: {err}"))
}

fn authoritative_claim_item(pkg_idx: usize, pkg: &ClaimTxPackage) -> Result<StoreItem, String> {
    let leaf_pkg = pkg
        .tx
        .outputs
        .first()
        .and_then(|output| output.asset_wire.as_ref())
        .ok_or_else(|| format!("package[{pkg_idx}] missing output asset_wire"))?;
    let wire = leaf_pkg
        .clone()
        .to_wire()
        .map_err(|err| format!("package[{pkg_idx}] output to_wire failed: {err}"))?;
    let leaf = asset_wire_to_leaf(&wire)
        .map_err(|err| format!("package[{pkg_idx}] output leaf conversion failed: {err}"))?;
    let path = SettlementPath::new(
        DefinitionId::new(wire.definition.id),
        SerialId::new(leaf.serial_id),
        TerminalId::new(leaf.asset_id),
    );
    StoreItem::new(path, leaf)
        .map_err(|err| format!("package[{pkg_idx}] authoritative claim item failed: {err}"))
}

fn authoritative_claim_store(bundle_path: &Path) -> Result<SettlementStore, String> {
    let claim_dir = bundle_path.parent().ok_or_else(|| {
        format!(
            "claim bundle path has no parent directory: {}",
            bundle_path.display()
        )
    })?;
    let store_path = claim_dir.join(CLAIM_STORE_FILE);
    if !store_path.exists() {
        return Err(format!(
            "persisted claim membership store missing: {}",
            store_path.display()
        ));
    }
    SettlementStore::load(&store_path).map_err(|err| {
        format!(
            "authoritative claim store open failed at {}: {err}",
            store_path.display()
        )
    })
}

fn persisted_claim_paths(store: &SettlementStore) -> Result<BTreeSet<SettlementPath>, String> {
    let page = store
        .list_settlement(SettlementListReq::all(usize::MAX))
        .map_err(|err| format!("persisted claim store list failed: {err}"))?;
    if page.next().is_some() {
        return Err("persisted claim store membership listing truncated".to_string());
    }
    Ok(page.items().iter().map(|item| item.path()).collect())
}

fn verify_authoritative_claim_pkg(
    store: &SettlementStore,
    pkg_idx: usize,
    pkg: &ClaimTxPackage,
    item: &StoreItem,
) -> Result<(), String> {
    // This verifier keeps producer, persisted bundle, and wallet validation on
    // the same storage-backed membership contract for each carried claim leaf.
    require_claim_auth_simulator_anchor(pkg.chain_id, &pkg.chain_type, &pkg.chain_name)
        .map_err(|err| format!("package[{pkg_idx}] {err}"))?;

    let (expected_root, expected_proof) =
        store.claim_source_contract_for_item(item).map_err(|err| {
            format!("package[{pkg_idx}] persisted claim store contract failed: {err}")
        })?;
    let actual_proof = decode_claim_source_proof(pkg_idx, pkg)?;

    if actual_proof.root_version() != expected_root.root_version() {
        return Err(format!(
            "package[{pkg_idx}] claim root version mismatch against bundle-backed canonical root"
        ));
    }
    if actual_proof.source_root() != expected_root.into_bytes() {
        return Err(format!(
            "package[{pkg_idx}] claim source root mismatch against bundle-backed canonical root"
        ));
    }
    if actual_proof.proof_ver() != expected_proof.proof_ver() {
        return Err(format!(
            "package[{pkg_idx}] claim proof version mismatch against bundle-backed canonical proof"
        ));
    }
    if actual_proof.proof_blob() != expected_proof.proof_blob() {
        return Err(format!(
            "package[{pkg_idx}] claim proof blob mismatch against bundle-backed canonical proof"
        ));
    }

    Ok(())
}

fn verify_authoritative_claim_pkgs(
    bundle_path: &Path,
    packages: &[ClaimTxPackage],
) -> Result<(), String> {
    let store = authoritative_claim_store(bundle_path)?;
    let items = packages
        .iter()
        .enumerate()
        .map(|(pkg_idx, pkg)| authoritative_claim_item(pkg_idx, pkg))
        .collect::<Result<Vec<_>, _>>()?;

    let mut carried_paths = BTreeSet::new();
    for (pkg_idx, item) in items.iter().enumerate() {
        if !carried_paths.insert(item.path()) {
            return Err(format!(
                "package[{pkg_idx}] duplicate claim asset path across carried packages"
            ));
        }
    }

    let stored_paths = persisted_claim_paths(&store)?;
    if carried_paths != stored_paths {
        return Err(format!(
            "persisted claim store membership mismatch: carried_paths={} stored_paths={} missing_from_bundle={} unexpected_in_bundle={}",
            carried_paths.len(),
            stored_paths.len(),
            stored_paths.difference(&carried_paths).count(),
            carried_paths.difference(&stored_paths).count(),
        ));
    }

    for ((pkg_idx, pkg), item) in packages.iter().enumerate().zip(items.iter()) {
        verify_authoritative_claim_pkg(&store, pkg_idx, pkg, item)?;
    }
    Ok(())
}

/// Verify all claim package bytes before any consumer path proceeds.
pub fn verify_claim_packages(packages: &[Vec<u8>]) -> Result<(), String> {
    let verifier = ClaimTxVerifierImpl::new();
    for (idx, pkg_bytes) in packages.iter().enumerate() {
        let result = verifier.verify(pkg_bytes);
        if !result.valid {
            return Err(format!(
                "package[{idx}] failed: class='{}' errors={:?}",
                result.reject_class, result.errors
            ));
        }
    }
    Ok(())
}

/// Load and verify claim packages from `tx_claim_pkg.json`.
pub fn load_claim_packages(path: &Path) -> Result<Vec<ClaimTxPackage>, String> {
    let rows = load_claim_rows(path)?;
    verify_claim_packages(&rows)?;

    let packages: Vec<ClaimTxPackage> = rows
        .iter()
        .enumerate()
        .map(|(idx, bytes)| {
            JsonCodec
                .deserialize(bytes)
                .map_err(|e| format!("claim package decode failed at index {idx}: {e}"))
        })
        .collect::<Result<_, _>>()?;

    verify_authoritative_claim_pkgs(path, &packages)?;
    Ok(packages)
}

/// Extract portable leaf wires from verified claim packages.
pub fn claim_leaves(packages: &[ClaimTxPackage]) -> Result<Vec<AssetWire>, String> {
    let mut out = Vec::new();
    for (pkg_idx, pkg) in packages.iter().enumerate() {
        for (out_idx, item) in pkg.tx.outputs.iter().enumerate() {
            let asset_wire = item.asset_wire.clone().ok_or_else(|| {
                format!("package[{pkg_idx}] output[{out_idx}] missing output asset_wire")
            })?;
            let wire = asset_wire
                .to_wire()
                .map_err(|e| format!("package[{pkg_idx}] output[{out_idx}] to_wire failed: {e}"))?;
            out.push(wire);
        }
    }
    Ok(out)
}

/// Load, verify, and extract portable leaf wires from `tx_claim_pkg.json`.
pub fn load_claim_leaves(path: &Path) -> Result<Vec<AssetWire>, String> {
    let packages = load_claim_packages(path)?;
    claim_leaves(&packages)
}

fn build_claim_put_op(
    pkg_idx: usize,
    out_idx: usize,
    output: &z00z_wallets::tx::ClaimOutputWire,
    seen_ids: &mut HashSet<TerminalId>,
) -> Result<StoreOp, String> {
    let leaf_wire = output
        .asset_wire
        .as_ref()
        .ok_or_else(|| format!("package[{pkg_idx}] output[{out_idx}] missing output asset_wire"))?;
    let wire = leaf_wire
        .clone()
        .to_wire()
        .map_err(|e| format!("package[{pkg_idx}] output[{out_idx}] to_wire failed: {e}"))?;
    let leaf = asset_wire_to_leaf(&wire).map_err(|e| {
        format!("package[{pkg_idx}] output[{out_idx}] jmt leaf conversion failed: {e}")
    })?;

    let terminal_id = TerminalId::new(leaf.asset_id);
    if !seen_ids.insert(terminal_id) {
        return Err(format!(
            "duplicate settlement terminal across claim packages at package[{pkg_idx}] output[{out_idx}]"
        ));
    }

    let path = SettlementPath::new(
        DefinitionId::new(wire.definition.id),
        SerialId::new(wire.serial_id),
        terminal_id,
    );
    let item = StoreItem::new(path, leaf).map_err(|e| {
        format!("package[{pkg_idx}] output[{out_idx}] store item build failed: {e}")
    })?;

    Ok(StoreOp::Put(Box::new(item)))
}

/// Build storage insert operations for all portable outputs in verified claim packages.
pub fn build_claim_store_ops(packages: &[ClaimTxPackage]) -> Result<Vec<StoreOp>, String> {
    let mut seen_ids = HashSet::new();
    let mut ops = Vec::new();

    for (pkg_idx, pkg) in packages.iter().enumerate() {
        for (out_idx, output) in pkg.tx.outputs.iter().enumerate() {
            ops.push(build_claim_put_op(pkg_idx, out_idx, output, &mut seen_ids)?);
        }
    }

    Ok(ops)
}

/// Load verified claim packages from disk and publish their outputs into storage.
pub fn publish_claims_store(
    path: &Path,
    store: &mut SettlementStore,
) -> Result<ClaimStorePublishSummary, String> {
    with_pkg_store(path, || {
        let packages = load_claim_packages(path)?;
        let claims = claim_nulls(&packages)?;
        let storage_claims: Vec<ClaimNullTx> = claims
            .iter()
            .map(|claim| {
                Ok::<ClaimNullTx, String>(ClaimNullTx {
                    nullifier: ClaimNullifier::from_hex(&claim.nullifier_hex)
                        .map_err(|err| format!("claim package nullifier parse failed: {err}"))?,
                    claim_id_hex: claim.claim_id_hex.clone(),
                    chain_id: claim.chain_id,
                    tx_digest_hex: claim.tx_digest_hex.clone(),
                })
            })
            .collect::<Result<_, _>>()?;
        for claim in &storage_claims {
            let row = store
                .settlement_claim_null_rec(&claim.nullifier)
                .map_err(|err| format!("claim nullifier lookup failed: {err}"))?;
            if let Some(row) = row {
                return Err(format!(
                    "claim nullifier replay rejected: nullifier={} status={:?} tx_digest={}",
                    row.nullifier, row.status, row.tx_digest_hex
                ));
            }
        }
        let leases = load_reserved_nulls(&claims)?;

        if EnvConfig.get(FAIL_CLAIM_PUB).ok().flatten().is_some() {
            return rollback_leases_with_error(
                &leases,
                "claim store publish failed: forced publish fault",
            );
        }

        let ops = build_claim_store_ops(&packages)
            .or_else(|err| rollback_leases_with_error(&leases, err))?;
        let inserted_count = ops.len();
        store
            .apply_settlement_claim_ops(ops, &storage_claims)
            .map_err(|err| format!("claim store publish failed: {err}"))
            .or_else(|err| rollback_leases_with_error(&leases, err))?;
        commit_leases(&leases);

        Ok(ClaimStorePublishSummary {
            package_count: packages.len(),
            leaf_count: inserted_count,
            inserted_count,
        })
    })
}

#[cfg(test)]
mod tests {
    use super::{claim_nulls, with_pkg_store};
    use crate::scenario_1::claim_pkg_store::NULL_ROWS_FILE;
    use z00z_utils::codec::{Codec, JsonCodec, Value};
    use z00z_utils::io::write_file;
    use z00z_wallets::claim::{clear_bind, clear_rows, read_entry};
    use z00z_wallets::tx::{
        ClaimAuthWire, ClaimContextWire, ClaimInputWire, ClaimOutputWire, ClaimProofWire,
        ClaimTxPackage, ClaimTxWire,
    };

    fn sample_pkg() -> ClaimTxPackage {
        ClaimTxPackage {
            kind: "TxPackage".to_string(),
            package_type: "claim_tx".to_string(),
            version: 1,
            chain_id: 3,
            chain_type: "devnet".to_string(),
            chain_name: "z00z-devnet-1".to_string(),
            tx: ClaimTxWire {
                tx_type: "claim_tx".to_string(),
                inputs: vec![ClaimInputWire {
                    claim_id_hex: "11".repeat(32),
                    claim_source_asset_id_hex: "22".repeat(32),
                    claim_source_commitment_hex: "33".repeat(32),
                }],
                outputs: vec![ClaimOutputWire {
                    asset_id_hex: "44".repeat(32),
                    amount: 7,
                    asset_class: "coin".to_string(),
                    owner_binding_hex: "55".repeat(32),
                    nonce_hex: "66".repeat(32),
                    asset_wire: None,
                    owner_attest_hex: None,
                }],
                fee: 0,
                nonce: 0,
                context: ClaimContextWire {
                    recipient_wallet_id: "alice".to_string(),
                    recipient_owner_hex: "77".repeat(32),
                    claim_scope_hash_hex: "88".repeat(32),
                    recipient_card_hex: None,
                    nullifier_hex: "99".repeat(32),
                },
                proof: ClaimProofWire {
                    proof_type: "genesis_claim".to_string(),
                    proof_hex: String::new(),
                },
                auth: ClaimAuthWire {
                    claim_authority_sig_hex: String::new(),
                },
            },
            tx_digest_hex: "aa".repeat(32),
            status: "prepared".to_string(),
        }
    }

    #[test]
    fn test_nulls_reject_digest_collision() {
        let left = sample_pkg();
        let mut right = sample_pkg();
        right.tx_digest_hex = "11".repeat(32);

        let err = claim_nulls(&[left, right]).expect_err("digest collision must fail");
        assert!(err.contains("claim nullifier collision across package rows"));
    }

    #[test]
    fn test_pkg_store_clears_bind() {
        clear_bind();
        clear_rows();

        let dir = tempfile::tempdir().expect("tempdir");
        let path_buf = dir.path().join("tx_claim_pkg.json");
        let path = path_buf.as_path();
        with_pkg_store(path, || Ok(())).expect("bind package store");

        let row_path = dir.path().join(NULL_ROWS_FILE);
        write_file(&row_path, b"{bad json").expect("write corrupt rows");

        assert!(read_entry(&"00".repeat(32)).is_ok());
        clear_rows();
    }

    #[test]
    fn test_packages_serializes_explicit_discriminators() {
        let bundle = super::wrap_claim_packages(vec![sample_pkg()]);
        let value: Value = JsonCodec
            .deserialize(&JsonCodec.serialize(&bundle).expect("bundle bytes"))
            .expect("bundle to value");
        let root = value.as_object().expect("bundle object");

        assert_eq!(
            root.get("kind").and_then(Value::as_str),
            Some("TxPackageBundle")
        );
        assert_eq!(
            root.get("package_type").and_then(Value::as_str),
            Some("claim_tx")
        );
        assert_eq!(root.get("version").and_then(Value::as_u64), Some(1));
        assert!(
            root.get("packages")
                .and_then(Value::as_array)
                .is_some_and(|packages| packages.len() == 1),
            "bundle must serialize explicit package rows"
        );
    }

    #[test]
    fn test_rejects_missing_explicit_discriminators() {
        let value: Value = JsonCodec
            .deserialize(
                &JsonCodec
                    .serialize(&super::wrap_claim_packages(vec![sample_pkg()]))
                    .expect("bundle bytes"),
            )
            .expect("bundle to value");

        for missing in ["kind", "package_type", "version"] {
            let mut mutated = value.clone();
            mutated
                .as_object_mut()
                .expect("bundle object")
                .remove(missing);
            let raw = JsonCodec.serialize(&mutated).expect("bundle bytes");

            let err =
                super::decode_claim_bundle(&raw).expect_err("missing discriminator must reject");
            assert!(
                err.contains("claim package bundle parse failed") && err.contains("missing field"),
                "unexpected error for missing {missing}: {err}"
            );
        }
    }
}
