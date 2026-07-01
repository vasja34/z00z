use std::path::PathBuf;

use serde_json::Value;
use z00z_utils::io::load_json;
use z00z_wallets::claim::{map_import_err, map_import_ok, map_replay_code};

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

#[test]
fn test_reason_codes_ok_replay() {
    let fixture: Value =
        load_json(fixture_path("claim_import_reason_codes.json")).expect("fixture");

    let ok_new = &fixture["ok_new"];
    let got = map_import_ok(&ok_new["input"]);
    assert_eq!(got.code, ok_new["expected_code"].as_str().expect("code"));

    let ok_dup = &fixture["ok_dup"];
    let got = map_import_ok(&ok_dup["input"]);
    assert_eq!(got.code, ok_dup["expected_code"].as_str().expect("code"));

    let ok_rejected = &fixture["ok_rejected"];
    let got = map_import_ok(&ok_rejected["input"]);
    assert_eq!(
        got.code,
        ok_rejected["expected_code"].as_str().expect("code")
    );

    let replay_dup = &fixture["replay_dup"];
    let got = map_replay_code(&replay_dup["input"]);
    assert_eq!(got, replay_dup["expected_code"].as_str().expect("code"));

    let replay_unknown = &fixture["replay_unknown"];
    let got = map_replay_code(&replay_unknown["input"]);
    assert_eq!(got, replay_unknown["expected_code"].as_str().expect("code"));
}

#[test]
fn test_reason_codes_error() {
    let fixture: Value =
        load_json(fixture_path("claim_import_reason_codes.json")).expect("fixture");

    let err_token = &fixture["err_token"];
    let got = map_import_err(err_token["input"].as_str().expect("text"));
    assert_eq!(got.code, err_token["expected_code"].as_str().expect("code"));

    let err_default = &fixture["err_default"];
    let got = map_import_err(err_default["input"].as_str().expect("text"));
    assert_eq!(
        got.code,
        err_default["expected_code"].as_str().expect("code")
    );
}
