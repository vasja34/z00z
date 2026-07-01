use z00z_core::assets::{AssetLeaf, AssetWire};
use z00z_utils::{
    codec::{Codec, JsonCodec},
    io::{load_json, read_file},
    time::{SystemTimeProvider, TimeProvider},
};
use z00z_wallets::tx::TxPackage;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SimIn {
    pub(crate) asset_id_hex: String,
    pub(crate) serial_id: u32,
    pub(crate) amount: u64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SimOut {
    pub(crate) receiver: String,
    pub(crate) value: u64,
    pub(crate) leaf: AssetLeaf,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SimPkg {
    pub(crate) version: u32,
    pub(crate) created_at_unix: u64,
    pub(crate) sender: String,
    pub(crate) receiver: String,
    pub(crate) inputs: Vec<SimIn>,
    pub(crate) outputs: Vec<SimOut>,
    pub(crate) tx_digest_hex: String,
    pub(crate) status: String,
}

fn sim_leaf_from_wire(wire: &AssetWire) -> Result<AssetLeaf, String> {
    z00z_wallets::tx::wire_decrypt_leaf(wire)
        .map(Into::into)
        .map_err(|e| e.to_string())
}

fn sim_from_tx_pkg(pkg: &TxPackage) -> Result<SimPkg, String> {
    let created_at_unix = SystemTimeProvider
        .try_unix_timestamp()
        .map_err(|err| err.to_string())?;

    let inputs = pkg
        .tx
        .inputs
        .iter()
        .map(|inp| SimIn {
            asset_id_hex: inp.asset_id_hex.clone(),
            serial_id: inp.serial_id,
            amount: 1,
        })
        .collect();

    let outputs = pkg
        .tx
        .outputs
        .iter()
        .map(|out| {
            let wire = out
                .asset_wire
                .clone()
                .to_wire()
                .map_err(|e| format!("tx output asset_wire conversion failed: {e}"))?;
            Ok(SimOut {
                receiver: "bob".to_string(),
                value: wire.amount,
                leaf: sim_leaf_from_wire(&wire)?,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;

    Ok(SimPkg {
        version: u32::from(pkg.version),
        created_at_unix,
        sender: "alice".to_string(),
        receiver: "bob".to_string(),
        inputs,
        outputs,
        tx_digest_hex: pkg.tx_digest_hex.clone(),
        status: pkg.status.clone(),
    })
}

pub(crate) fn load_sim_pkg_any(path: &std::path::Path) -> SimPkg {
    if let Ok(sim) = load_json::<SimPkg>(path) {
        return sim;
    }

    let pkg: TxPackage = load_json(path).expect("load canonical tx package");
    sim_from_tx_pkg(&pkg).expect("convert tx package to sim schema")
}

pub(crate) fn check_schema(pkg: &SimPkg) {
    assert_eq!(pkg.version, 1, "sim pkg version must be 1");
    assert!(!pkg.sender.is_empty(), "sender must not be empty");
    assert!(!pkg.receiver.is_empty(), "receiver must not be empty");
    assert!(!pkg.inputs.is_empty(), "inputs must not be empty");
    assert!(!pkg.outputs.is_empty(), "outputs must not be empty");
    assert_eq!(pkg.tx_digest_hex.len(), 64, "digest must be 32-byte hex");
    assert_eq!(pkg.status, "prepared", "status must be prepared");

    for row in &pkg.inputs {
        assert_eq!(
            row.asset_id_hex.len(),
            64,
            "input asset id must be 32-byte hex"
        );
        assert!(row.amount > 0, "input amount must be positive");
    }

    for row in &pkg.outputs {
        assert!(
            !row.receiver.is_empty(),
            "output receiver must not be empty"
        );
        assert!(row.value > 0, "output value must be positive");
        assert_ne!(
            row.leaf.asset_id, [0u8; 32],
            "leaf asset_id must be non-zero"
        );
        assert_ne!(row.leaf.r_pub, [0u8; 32], "leaf r_pub must be non-zero");
        assert_ne!(
            row.leaf.owner_tag, [0u8; 32],
            "leaf owner_tag must be non-zero"
        );
        assert_ne!(
            row.leaf.c_amount, [0u8; 32],
            "leaf c_amount must be non-zero"
        );
        assert_eq!(row.leaf.enc_pack.version, 1, "leaf pack version must be 1");
        assert!(
            !row.leaf.enc_pack.ciphertext.is_empty(),
            "leaf ciphertext must be non-empty"
        );
        assert!(
            !row.leaf.range_proof.is_empty(),
            "leaf range proof must be non-empty"
        );
    }
}

pub fn load_pkg_bundle(out_dir: &std::path::Path) -> (std::path::PathBuf, Vec<u8>, TxPackage) {
    let sim_file = out_dir
        .join("transactions")
        .join("tx_alice_to_bob_pkg.json");
    assert!(
        sim_file.exists(),
        "stage-4 tx package must exist at {}",
        sim_file.display()
    );
    let can_bytes = read_file(&sim_file).expect("read tx package");
    let can_pkg: TxPackage = JsonCodec
        .deserialize(&can_bytes)
        .expect("decode tx package");
    let sim_pkg = load_sim_pkg_any(&sim_file);
    check_schema(&sim_pkg);
    assert_eq!(can_pkg.kind, "TxPackage", "kind must be TxPackage");
    assert_eq!(can_pkg.version, 1, "version must be 1");
    assert_eq!(can_pkg.status, "prepared", "status must be prepared");
    assert!(
        !can_pkg.tx_digest_hex.is_empty(),
        "tx_digest_hex must be present and non-empty"
    );
    (sim_file, can_bytes, can_pkg)
}
