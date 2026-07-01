use z00z_utils::{
    codec::{json, Codec, JsonCodec, Value},
    io::write_file,
};
use z00z_wallets::tx::{TxPackage, TxVerifier, TxVerifierImpl};

pub fn check_verifier(ver: &TxVerifierImpl, can_bytes: &[u8]) -> Value {
    assert!(
        ver.verify_structure(can_bytes).is_ok(),
        "canonical structure must pass"
    );
    assert!(
        ver.verify_balance(can_bytes).is_ok(),
        "canonical balance must pass"
    );
    assert!(
        ver.verify_signatures(can_bytes).is_ok(),
        "canonical signatures must pass"
    );
    assert!(
        ver.verify_range_proofs(can_bytes).is_ok(),
        "canonical range proofs must pass"
    );

    let can_res = ver.verify(can_bytes).expect("canonical verify run");
    assert!(can_res.valid, "canonical wallet pkg must be valid");
    json!({
        "canonical_valid": can_res.valid,
        "canonical_errs": can_res.errors.len(),
    })
}

pub fn check_empty_out(ver: &TxVerifierImpl, can_pkg: &TxPackage) -> (Vec<u8>, Value) {
    let mut bad_pkg = can_pkg.clone();
    bad_pkg.tx.outputs.clear();
    let wallet_bad = JsonCodec.serialize(&bad_pkg).expect("serialize bad pkg");
    let bad_res = ver.verify(&wallet_bad).expect("tampered verify run");
    assert!(
        !bad_res.valid,
        "tampered wallet pkg must be rejected by verifier"
    );
    assert!(
        bad_res
            .errors
            .iter()
            .any(|row| row.contains("tx package outputs are empty")),
        "unexpected tampered verifier errors: {:?}",
        bad_res.errors
    );

    (
        wallet_bad,
        json!({
            "tampered_valid": bad_res.valid,
            "tampered_errs": bad_res.errors.len(),
        }),
    )
}

pub fn write_ver_log(
    out_dir: &std::path::Path,
    sim_file: &std::path::Path,
    can_bytes: &[u8],
    wallet_bad: &[u8],
    can_meta: &Value,
    bad_meta: &Value,
) {
    write_file(out_dir.join("wallet_pkg_canonical.json"), can_bytes).expect("write wallet can");
    write_file(out_dir.join("wallet_pkg_tampered.json"), wallet_bad).expect("write wallet bad");

    let mut ver_log = String::from("E2E-18 verifier\n");
    ver_log.push_str(&format!("sim_file={}\n", sim_file.display()));
    ver_log.push_str(&format!(
        "canonical_valid={}\n",
        can_meta["canonical_valid"]
    ));
    ver_log.push_str(&format!("canonical_errs={}\n", can_meta["canonical_errs"]));
    ver_log.push_str(&format!("tampered_valid={}\n", bad_meta["tampered_valid"]));
    ver_log.push_str(&format!("tampered_errs={}\n", bad_meta["tampered_errs"]));
    write_file(out_dir.join("verifier_log.txt"), ver_log.as_bytes()).expect("write ver log");
}
