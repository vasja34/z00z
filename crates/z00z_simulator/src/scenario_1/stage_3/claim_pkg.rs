use std::{
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use z00z_core::assets::{AssetPkgWire, AssetWire};
use z00z_storage::settlement::{
    DefinitionId, SerialId, SettlementPath, SettlementStore, StoreItem, TerminalId,
};
use z00z_utils::{
    codec::{Codec, JsonCodec},
    io::{create_dir_all, path_exists, remove_dir_all, remove_file, write_file},
    time::{SystemTimeProvider, TimeProvider},
};
use z00z_wallets::{
    key::ReceiverKeys,
    tx::{
        build_claim_stmt, build_claim_tx_digest, compute_claim_scope_hash, derive_output_nonce,
        sign_claim_auth, sign_owner_attest, ClaimAuthWire, ClaimContextWire, ClaimInputWire,
        ClaimOutputWire, ClaimProofWire, ClaimScopeKey, ClaimTxPackage, ClaimTxWire, CLAIM_PKG,
    },
};

use crate::scenario_1::claim_pkg_consumer::{
    claim_nulls, load_claim_packages, reserve_nulls, rollback_leases, with_pkg_store,
    wrap_claim_packages,
};

use super::{asset_wire_to_leaf, derive_nullifier};

pub const CLAIM_STORE_FILE: &str = "claim_source_store.redb";
static CLAIM_DIR_SEQ: AtomicU64 = AtomicU64::new(0);

fn claim_store_path(claim_dir: &Path) -> PathBuf {
    claim_dir.join(CLAIM_STORE_FILE)
}

fn reset_claim_store(claim_dir: &Path) -> Result<PathBuf, String> {
    create_dir_all(claim_dir).map_err(|e| format!("claim dir create failed: {e}"))?;
    let store_path = claim_store_path(claim_dir);
    if path_exists(&store_path).map_err(|e| format!("claim store stat failed: {e}"))? {
        if store_path.is_dir() {
            remove_dir_all(&store_path).map_err(|e| format!("claim store reset failed: {e}"))?;
        } else {
            remove_file(&store_path).map_err(|e| format!("claim store reset failed: {e}"))?;
        }
    }
    Ok(store_path)
}

fn load_claim_store(claim_dir: &Path) -> Result<SettlementStore, String> {
    let store_path = claim_store_path(claim_dir);
    if !path_exists(&store_path).map_err(|e| format!("claim store stat failed: {e}"))? {
        return Err(format!(
            "persisted claim membership store missing: {}",
            store_path.display()
        ));
    }
    SettlementStore::load(&store_path)
        .map_err(|e| format!("claim store open failed at {}: {e}", store_path.display()))
}

fn remove_claim_store(claim_dir: &Path) -> Result<(), String> {
    let store_path = claim_store_path(claim_dir);
    if !path_exists(&store_path).map_err(|e| format!("claim store stat failed: {e}"))? {
        return Ok(());
    }
    if store_path.is_dir() {
        remove_dir_all(&store_path).map_err(|e| format!("claim store cleanup failed: {e}"))?;
    } else {
        remove_file(&store_path).map_err(|e| format!("claim store cleanup failed: {e}"))?;
    }
    Ok(())
}

fn cleanup_temp_claim_dir(claim_dir: &Path) {
    let _ = remove_claim_store(claim_dir);
    let _ = remove_dir_all(claim_dir);
}

fn temp_claim_dir() -> PathBuf {
    let stamp = SystemTimeProvider.compat_unix_timestamp_micros();
    let seq = CLAIM_DIR_SEQ.fetch_add(1, Ordering::Relaxed);
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(format!(
            "target/claim-store-{}-{stamp}-{seq}",
            std::process::id()
        ))
}

fn make_claim_source_item(leaf_pkg: &AssetPkgWire) -> Result<StoreItem, String> {
    let wire = leaf_pkg
        .clone()
        .to_wire()
        .map_err(|e| format!("claim source to_wire failed: {e}"))?;
    let leaf = asset_wire_to_leaf(&wire).map_err(|e| format!("claim source leaf failed: {e}"))?;
    let path = SettlementPath::new(
        DefinitionId::new(wire.definition.id),
        SerialId::new(leaf.serial_id),
        TerminalId::new(leaf.asset_id),
    );
    StoreItem::new(path, leaf).map_err(|e| format!("claim source item failed: {e}"))
}

/// Build the canonical claim bundle used by stage 3 before any test faults are injected.
pub fn build_claim_package(
    chain_id: u32,
    chain_type: &str,
    chain_name: &str,
    actor_name: &str,
    asset_id_hex: &str,
    amount: u64,
    claim_id_bytes: &[u8; 32],
    recipient_owner_bytes: &[u8; 32],
    nonce: u64,
    asset_wire: Option<AssetWire>,
    recipient_keys: Option<&ReceiverKeys>,
) -> Result<Vec<u8>, String> {
    build_claim_package_inner(
        chain_id,
        chain_type,
        chain_name,
        actor_name,
        asset_id_hex,
        amount,
        claim_id_bytes,
        recipient_owner_bytes,
        nonce,
        asset_wire,
        recipient_keys,
        None,
    )
}

fn claim_source_item_from_pkg(pkg: &ClaimTxPackage) -> Result<StoreItem, String> {
    let leaf_pkg = pkg
        .tx
        .outputs
        .first()
        .and_then(|output| output.asset_wire.as_ref())
        .ok_or_else(|| "claim package requires output asset_wire".to_string())?;
    make_claim_source_item(leaf_pkg)
}

fn patch_claim_package_crypto(
    pkg: &mut ClaimTxPackage,
    proof: &z00z_crypto::ClaimSourceProof,
) -> Result<(), String> {
    pkg.tx.proof.proof_type = "claim_source".to_string();
    pkg.tx.proof.proof_hex = hex::encode(proof.to_bytes().map_err(|e| e.to_string())?);

    let stmt = build_claim_stmt(pkg).map_err(|e| format!("claim stmt build failed: {e}"))?;
    if stmt.source_root() != proof.source_root() {
        return Err(
            "claim stmt source_root does not match storage-backed claim source root".to_string(),
        );
    }

    let sig = sign_claim_auth(&stmt).map_err(|e| format!("claim authority build failed: {e}"))?;
    pkg.tx.auth.claim_authority_sig_hex = hex::encode(sig.to_bytes().map_err(|e| e.to_string())?);
    pkg.tx_digest_hex = build_claim_tx_digest(
        "TxPackage",
        "claim_tx",
        pkg.version,
        pkg.chain_id,
        &pkg.chain_type,
        &pkg.chain_name,
        &pkg.tx,
    )
    .map_err(|e| format!("claim digest error: {e}"))?;

    Ok(())
}

pub fn write_claim_bundle_store(
    claim_dir: &Path,
    packages: &[ClaimTxPackage],
) -> Result<(), String> {
    if packages.is_empty() {
        return Err("claim package bundle must carry at least one package".to_string());
    }

    let items = packages
        .iter()
        .enumerate()
        .map(|(pkg_idx, pkg)| {
            claim_source_item_from_pkg(pkg)
                .map_err(|e| format!("package[{pkg_idx}] claim source item failed: {e}"))
        })
        .collect::<Result<Vec<_>, _>>()?;

    let store_path = reset_claim_store(claim_dir)?;
    let mut store = SettlementStore::load(&store_path)
        .map_err(|e| format!("claim store open failed at {}: {e}", store_path.display()))?;
    for (pkg_idx, item) in items.iter().enumerate() {
        store
            .put_settlement_item(item.clone())
            .map_err(|e| format!("package[{pkg_idx}] claim source store insert failed: {e}"))?;
    }

    Ok(())
}

pub fn patch_claim_bundle_membership_in(
    packages: &mut [ClaimTxPackage],
    claim_dir: &Path,
) -> Result<(), String> {
    write_claim_bundle_store(claim_dir, packages)?;
    let store = load_claim_store(claim_dir)?;
    let items = packages
        .iter()
        .enumerate()
        .map(|(pkg_idx, pkg)| {
            claim_source_item_from_pkg(pkg)
                .map_err(|e| format!("package[{pkg_idx}] claim source item failed: {e}"))
        })
        .collect::<Result<Vec<_>, _>>()?;

    for ((pkg_idx, pkg), item) in packages.iter_mut().enumerate().zip(items.iter()) {
        let (_, proof) = store
            .claim_source_contract_for_item(item)
            .map_err(|e| format!("package[{pkg_idx}] persisted claim source proof failed: {e}"))?;
        patch_claim_package_crypto(pkg, &proof).map_err(|e| format!("package[{pkg_idx}] {e}"))?;
    }

    Ok(())
}

pub fn patch_claim_bundle_membership(packages: &mut [ClaimTxPackage]) -> Result<(), String> {
    let claim_dir = temp_claim_dir();
    patch_claim_bundle_membership_in(packages, &claim_dir)?;
    cleanup_temp_claim_dir(&claim_dir);
    Ok(())
}

fn build_claim_package_inner(
    chain_id: u32,
    chain_type: &str,
    chain_name: &str,
    actor_name: &str,
    asset_id_hex: &str,
    amount: u64,
    claim_id_bytes: &[u8; 32],
    recipient_owner_bytes: &[u8; 32],
    nonce: u64,
    asset_wire: Option<AssetWire>,
    recipient_keys: Option<&ReceiverKeys>,
    emit_fault: Option<&str>,
) -> Result<Vec<u8>, String> {
    let version = CLAIM_PKG;
    let asset_wire =
        asset_wire.ok_or_else(|| "claim package requires output asset_wire".to_string())?;
    let asset_class = asset_wire.definition.class.to_string().to_lowercase();
    let scope_key = ClaimScopeKey {
        chain_id,
        scenario_tag: "scenario_1_genesis_claim".to_string(),
        ruleset_version: 1,
    };
    let scope_hash = compute_claim_scope_hash(&scope_key);
    let scope_hash_hex = hex::encode(scope_hash);

    let recipient_owner = if emit_fault == Some(super::EMIT_BIND_MISMATCH) {
        let mut bad = *recipient_owner_bytes;
        bad[0] ^= 0x01;
        bad
    } else {
        *recipient_owner_bytes
    };

    let nullifier = derive_nullifier(claim_id_bytes, &recipient_owner, chain_id);
    let nullifier_hex = nullifier.to_hex();
    let claim_id_hex = hex::encode(claim_id_bytes);
    let recipient_owner_hex = hex::encode(recipient_owner);
    let keys = recipient_keys
        .ok_or_else(|| "claim package requires recipient_keys for owner attestation".to_string())?;
    if keys.owner_handle != recipient_owner {
        return Err("recipient_keys.owner_handle does not match recipient_owner_bytes".to_string());
    }
    let card = keys
        .export_receiver_card()
        .map_err(|e| format!("export receiver card failed: {e}"))?;
    let recipient_card_hex = Some(hex::encode(card.canonical_encoding()));

    let nonce_hex = hex::encode(derive_output_nonce(claim_id_bytes, 0));
    let claim_source_commitment_hex = hex::encode(asset_wire.commitment.as_bytes());
    let output = ClaimOutputWire {
        asset_id_hex: asset_id_hex.to_string(),
        amount,
        asset_class,
        owner_binding_hex: recipient_owner_hex.clone(),
        nonce_hex,
        asset_wire: Some(AssetPkgWire::from_wire(&asset_wire)),
        owner_attest_hex: None,
    };

    let mut tx = ClaimTxWire {
        tx_type: "claim_tx".to_string(),
        inputs: vec![ClaimInputWire {
            claim_id_hex,
            claim_source_asset_id_hex: asset_id_hex.to_string(),
            claim_source_commitment_hex,
        }],
        outputs: vec![output],
        fee: 0,
        nonce,
        context: ClaimContextWire {
            recipient_wallet_id: actor_name.to_string(),
            recipient_owner_hex,
            claim_scope_hash_hex: scope_hash_hex,
            recipient_card_hex,
            nullifier_hex,
        },
        proof: ClaimProofWire {
            proof_type: "genesis_claim".to_string(),
            proof_hex: String::new(),
        },
        auth: ClaimAuthWire {
            claim_authority_sig_hex: String::new(),
        },
    };

    let leaf = tx.outputs[0]
        .asset_wire
        .as_ref()
        .ok_or_else(|| "claim package requires output asset_wire".to_string())?;
    let owner_attest_hex = sign_owner_attest(keys, chain_id, &tx, 0, leaf)?;
    tx.outputs[0].owner_attest_hex = Some(owner_attest_hex);

    let pkg = ClaimTxPackage {
        kind: "TxPackage".to_string(),
        package_type: "claim_tx".to_string(),
        version,
        chain_id,
        chain_type: chain_type.to_string(),
        chain_name: chain_name.to_string(),
        tx,
        tx_digest_hex: String::new(),
        status: "prepared".to_string(),
    };
    if emit_fault == Some(super::EMIT_PROOF_FAIL) {
        return Err("claim proof build failed: forced emit fault".to_string());
    }
    if emit_fault == Some(super::EMIT_AUTH_FAIL) {
        return Err("claim authority build failed: forced emit fault".to_string());
    }

    let mut packages = vec![pkg];
    let claim_dir = temp_claim_dir();
    patch_claim_bundle_membership_in(&mut packages, &claim_dir)
        .map_err(|e| format!("claim proof build failed: {e}"))?;
    cleanup_temp_claim_dir(&claim_dir);
    let pkg = packages.pop().expect("single claim pkg");

    JsonCodec
        .serialize(&pkg)
        .map_err(|e| format!("claim package serialize error: {e}"))
}

/// Expose the fault-injected path only to keep failure-mode coverage isolated.
pub fn build_claim_package_fault(
    chain_id: u32,
    chain_type: &str,
    chain_name: &str,
    actor_name: &str,
    asset_id_hex: &str,
    amount: u64,
    claim_id_bytes: &[u8; 32],
    recipient_owner_bytes: &[u8; 32],
    nonce: u64,
    asset_wire: Option<AssetWire>,
    recipient_keys: Option<&ReceiverKeys>,
    emit_fault: &str,
) -> Result<Vec<u8>, String> {
    build_claim_package_inner(
        chain_id,
        chain_type,
        chain_name,
        actor_name,
        asset_id_hex,
        amount,
        claim_id_bytes,
        recipient_owner_bytes,
        nonce,
        asset_wire,
        recipient_keys,
        Some(emit_fault),
    )
}

/// Persist one claim package bundle after early nullifier reservation succeeds.
pub fn write_claim_bundle(out_claim: &Path, packages: Vec<ClaimTxPackage>) -> Result<(), String> {
    write_claim_bundle_inner(out_claim, packages, None)
}

fn write_claim_bundle_inner(
    out_claim: &Path,
    packages: Vec<ClaimTxPackage>,
    write_fault: Option<&str>,
) -> Result<(), String> {
    let claim_pkg_path = out_claim.join("tx_claim_pkg.json");
    with_pkg_store(&claim_pkg_path, || {
        if packages.is_empty() {
            return Err("claim package bundle must carry at least one package".to_string());
        }

        let claims = claim_nulls(&packages)?;
        let leases = reserve_nulls(&claims)?;
        let bundle = wrap_claim_packages(packages);

        if write_fault == Some(super::WRITE_SERIALIZE_FAIL) {
            rollback_leases(&leases);
            return Err("claim package serialize failed: forced write fault".to_string());
        }

        let bundle_bytes = JsonCodec
            .serialize(&bundle)
            .map_err(|e| format!("claim package serialize failed: {e}"))?;

        if let Err(err) = write_claim_bundle_store(out_claim, &bundle.packages) {
            rollback_leases(&leases);
            let _ = remove_claim_store(out_claim);
            return Err(err);
        }

        if write_fault == Some(super::WRITE_IO_FAIL) {
            let _ = remove_claim_store(out_claim);
            rollback_leases(&leases);
            return Err("claim package write failed: forced write fault".to_string());
        }

        if let Err(err) = write_file(&claim_pkg_path, &bundle_bytes) {
            let _ = remove_claim_store(out_claim);
            rollback_leases(&leases);
            return Err(format!("claim package write failed: {err}"));
        }

        if write_fault == Some(super::WRITE_VERIFY_FAIL) {
            let _ = remove_file(&claim_pkg_path);
            let _ = remove_claim_store(out_claim);
            rollback_leases(&leases);
            return Err("claim package verify failed: forced write fault".to_string());
        }

        if let Err(err) = load_claim_packages(&claim_pkg_path) {
            let _ = remove_file(&claim_pkg_path);
            let _ = remove_claim_store(out_claim);
            rollback_leases(&leases);
            return Err(format!("claim package verify failed: {err}"));
        }

        Ok(())
    })
}

/// Persist one claim package bundle with a forced fault for fail-closed tests.
pub fn write_claim_bundle_fault(
    out_claim: &Path,
    packages: Vec<ClaimTxPackage>,
    write_fault: &str,
) -> Result<(), String> {
    write_claim_bundle_inner(out_claim, packages, Some(write_fault))
}
