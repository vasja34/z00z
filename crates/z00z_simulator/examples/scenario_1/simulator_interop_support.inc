use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use z00z_crypto::expert::encoding::SafePassword;
use z00z_simulator::{
    config::ScenarioCfg, scenario_1::claim_pkg_consumer::load_claim_leaves, StageResult,
};
use z00z_utils::{
    codec::{Codec, JsonCodec, Value, YamlCodec},
    io::{read_file, write_file},
};
use z00z_wallets::{
    asset_wire_to_leaf,
    receiver::{receiver_scan_leaf, ScanResult, StealthOutputScanner},
    rpc::types::{common::PersistWalletId, wallet::WalletSource},
    verify_tx_public_spend_contract, wire_decrypt_leaf, TxOutRole, TxOutputWire, TxPackage,
    TxVerifier, TxVerifierImpl, WalletService,
};

use super::{after_file, claim_file, tx_file, DemoError, WalletContext, WirePick, BOB_PASS};

pub(super) fn load_cfg(out: &Path) -> Result<ScenarioCfg, DemoError> {
    let cfg_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/scenario_1/scenario_config.yaml");
    let mut cfg =
        ScenarioCfg::from_file(cfg_path).map_err(|e| DemoError::Scenario(e.to_string()))?;
    cfg.stage1_genesis
        .get_or_insert_with(Default::default)
        .genesis_config = z00z_core::config_paths::devnet_genesis_path()
        .to_string_lossy()
        .to_string();
    cfg.outputs.dir = out.to_string_lossy().to_string();
    Ok(cfg)
}

pub(super) fn write_cfg(path: &Path, cfg: &ScenarioCfg) -> Result<(), DemoError> {
    let cfg_bytes = YamlCodec
        .serialize(cfg)
        .map_err(|e| DemoError::Decode(e.to_string()))?;
    write_file(path, &cfg_bytes).map_err(|e| DemoError::Io(e.to_string()))
}

pub(super) fn ensure_stage4_ok(run: &z00z_simulator::ScenarioResult) -> Result<(), DemoError> {
    let stage4 = run
        .stages
        .iter()
        .find(|item| item.stage == 4)
        .ok_or_else(|| DemoError::Missing("stage 4 result".to_string()))?;
    if matches!(stage4.result, StageResult::Ok) {
        Ok(())
    } else {
        Err(DemoError::Scenario(format!(
            "stage 4 did not finish cleanly: {:?}",
            stage4.result
        )))
    }
}

fn load_json_value(path: &Path) -> Result<Value, DemoError> {
    let bytes = read_file(path).map_err(|e| DemoError::Io(e.to_string()))?;
    JsonCodec
        .deserialize(&bytes)
        .map_err(|e| DemoError::Decode(e.to_string()))
}

fn load_tx_package(out: &Path) -> Result<TxPackage, DemoError> {
    let bytes = read_file(tx_file(out)).map_err(|e| DemoError::Io(e.to_string()))?;
    let verification = TxVerifierImpl
        .verify(&bytes)
        .map_err(|e| DemoError::Scenario(format!("stage4 tx package verification failed: {e}")))?;
    if !verification.valid {
        return Err(DemoError::Scenario(format!(
            "stage4 tx package verification failed: {}",
            verification.errors.join("; ")
        )));
    }
    let pkg: TxPackage = JsonCodec
        .deserialize(&bytes)
        .map_err(|e| DemoError::Decode(e.to_string()))?;
    verify_tx_public_spend_contract(
        pkg.chain_id,
        pkg.version,
        &pkg.chain_type,
        &pkg.chain_name,
        &pkg.tx,
    )
    .map_err(|e| DemoError::Scenario(format!("stage4 public spend contract failed: {e}")))?;
    Ok(pkg)
}

fn find_wallet_id(out: &Path, actor: &str) -> Result<String, DemoError> {
    let dump = load_json_value(&after_file(out))?;
    dump["wallets"]
        .as_array()
        .and_then(|rows| rows.iter().find(|row| row["actor"].as_str() == Some(actor)))
        .and_then(|row| row["wallet_id"].as_str())
        .map(str::to_string)
        .ok_or_else(|| DemoError::Missing(format!("wallet id for {actor}")))
}

fn find_wallet_path(out: &Path, actor: &str) -> Result<PathBuf, DemoError> {
    let dump = load_json_value(&after_file(out))?;
    dump["wallets"]
        .as_array()
        .and_then(|rows| rows.iter().find(|row| row["actor"].as_str() == Some(actor)))
        .and_then(|row| row["wlt_path"].as_str())
        .map(PathBuf::from)
        .ok_or_else(|| DemoError::Missing(format!("wallet path for {actor}")))
}

pub(super) fn open_bob_wallet(out: &Path) -> Result<WalletContext, DemoError> {
    let id = PersistWalletId(find_wallet_id(out, "bob")?);
    let path = find_wallet_path(out, "bob")?;
    let dir = path
        .parent()
        .ok_or_else(|| DemoError::Missing("bob wallet parent".to_string()))?
        .to_path_buf();
    let svc = Arc::new(WalletService::with_output_dir(dir));

    let rt = tokio::runtime::Runtime::new().map_err(|e| DemoError::Wallet(e.to_string()))?;
    let keys = rt.block_on(async {
        svc.open_wallet_source(WalletSource::Path {
            path: path.to_string_lossy().to_string(),
        })
        .await
        .map_err(|e| DemoError::Wallet(e.to_string()))?;
        svc.unlock_wallet_in_memory(&id, &SafePassword::from(BOB_PASS))
            .await
            .map_err(|e| DemoError::Wallet(e.to_string()))?;
        svc.receiver_keys(&id)
            .await
            .map_err(|e| DemoError::Wallet(e.to_string()))
    })?;

    Ok(WalletContext { svc, id, keys })
}

pub(super) fn scan_asset(
    wallet: &WalletContext,
    asset: &z00z_core::Asset,
) -> Result<ScanResult, DemoError> {
    Ok(StealthOutputScanner::from_keys(&wallet.keys).scan_leaf(asset))
}

fn decrypt_claim_leaf(
    wire: &z00z_core::AssetWire,
) -> Result<z00z_core::assets::AssetLeaf, DemoError> {
    wire_decrypt_leaf(wire)
        .map(Into::into)
        .map_err(|e| DemoError::Scenario(e.to_string()))
}

fn pick_claim(wallet: &WalletContext, out: &Path) -> Result<WirePick, DemoError> {
    let rows =
        load_claim_leaves(&claim_file(out)).map_err(|e| DemoError::Scenario(e.to_string()))?;

    for row in rows {
        let decrypt_leaf = decrypt_claim_leaf(&row)?;
        let leaf = asset_wire_to_leaf(&row).map_err(DemoError::Scenario)?;
        let asset = row
            .to_asset()
            .map_err(|e| DemoError::Scenario(e.to_string()))?;
        if receiver_scan_leaf(&wallet.keys, &decrypt_leaf)
            .map_err(|e| DemoError::Wallet(format!("{e:?}")))?
            .is_some()
        {
            return Ok(WirePick {
                label: "stage3_claim",
                leaf: leaf.into(),
                asset,
            });
        }
    }

    Err(DemoError::Missing("owned stage3 claim leaf".to_string()))
}

pub(super) fn load_rows(
    wallet: &WalletContext,
    out: &Path,
) -> Result<(WirePick, WirePick, WirePick), DemoError> {
    Ok((
        pick_claim(wallet, out)?,
        pick_owned(wallet, out)?,
        pick_foreign(wallet, out)?,
    ))
}

fn pick_owned(wallet: &WalletContext, out: &Path) -> Result<WirePick, DemoError> {
    let pkg = load_tx_package(out)?;

    for row in pkg.tx.outputs {
        if !matches!(row.role, TxOutRole::Recipient) {
            continue;
        }

        let pick = tx_pick(row, wallet, "stage4_owned")?;
        if matches!(pick.1, ScanResult::Mine { .. }) {
            return Ok(pick.0);
        }
    }

    Err(DemoError::Missing("owned stage4 output".to_string()))
}

fn pick_foreign(wallet: &WalletContext, out: &Path) -> Result<WirePick, DemoError> {
    let pkg = load_tx_package(out)?;

    for row in pkg.tx.outputs {
        let pick = tx_pick(row, wallet, "stage4_foreign")?;
        if matches!(pick.1, ScanResult::NotMine) {
            return Ok(pick.0);
        }
    }

    Err(DemoError::Missing("foreign stage4 output".to_string()))
}

fn tx_pick(
    row: TxOutputWire,
    wallet: &WalletContext,
    label: &'static str,
) -> Result<(WirePick, ScanResult), DemoError> {
    let wire = row
        .asset_wire
        .to_wire()
        .map_err(|e| DemoError::Scenario(e.to_string()))?;
    let leaf = asset_wire_to_leaf(&wire).map_err(DemoError::Scenario)?;
    let asset = wire
        .to_asset()
        .map_err(|e| DemoError::Scenario(e.to_string()))?;
    let scan = scan_asset(wallet, &asset)?;
    let pick = WirePick {
        label,
        leaf: leaf.into(),
        asset,
    };
    Ok((pick, scan))
}

pub(super) fn print_foreign(row: &WirePick, scan: &ScanResult) -> Result<(), DemoError> {
    if matches!(scan, ScanResult::NotMine) {
        println!(
            "foreign_boundary label={} status={} asset_id={}",
            row.label,
            scan.recv_status().rpc_code(),
            hex::encode(row.leaf.asset_id)
        );
        Ok(())
    } else {
        Err(DemoError::Scenario(
            "foreign simulator output crossed the wallet boundary as owned".to_string(),
        ))
    }
}

pub(super) fn print_row(row: &WirePick, scan: &ScanResult) -> Result<(), DemoError> {
    let ScanResult::Mine { wallet_output } = scan else {
        return Err(DemoError::Scenario(format!(
            "{} stopped being owned after wallet receive",
            row.label
        )));
    };

    if wallet_output.asset_id != row.leaf.asset_id
        || wallet_output.serial_id != row.leaf.serial_id
        || wallet_output.amount != row.asset.amount
        || wallet_output.r_pub != row.leaf.r_pub
        || wallet_output.owner_tag != row.leaf.owner_tag
        || row.asset.tag16 != Some(row.leaf.tag16)
    {
        return Err(DemoError::Scenario(format!(
            "{} drifted across simulator and wallet semantics",
            row.label
        )));
    }

    println!(
        "bridge label={} status={} asset_id={} serial_id={} amount={} r_pub={} owner_tag={} tag16={:04x}",
        row.label,
        scan.recv_status().rpc_code(),
        hex::encode(wallet_output.asset_id),
        wallet_output.serial_id,
        wallet_output.amount,
        hex::encode(wallet_output.r_pub),
        hex::encode(wallet_output.owner_tag),
        row.leaf.tag16
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use z00z_utils::io::save_json;

    #[test]
    fn test_reject_tampered_stage4_metadata() {
        let run = super::super::run_case().expect("scenario run");
        let pkg_path = tx_file(&run.out);
        let mut pkg = load_tx_package(&run.out).expect("load verified package");
        pkg.chain_id += 1;
        pkg.tx_digest_hex = z00z_wallets::build_tx_package_digest(
            &pkg.kind,
            &pkg.package_type,
            pkg.version,
            pkg.chain_id,
            &pkg.chain_type,
            &pkg.chain_name,
            &pkg.tx,
        )
        .expect("rebuild digest");
        save_json(&pkg_path, &pkg).expect("rewrite forged package");

        let err = load_tx_package(&run.out).expect_err("forged chain metadata must reject");
        let msg = err.to_string();
        assert!(
            msg.contains("stage4 public spend contract failed")
                || msg.contains("stage4 tx package verification failed"),
            "unexpected package loader failure: {msg}"
        );
    }
}
