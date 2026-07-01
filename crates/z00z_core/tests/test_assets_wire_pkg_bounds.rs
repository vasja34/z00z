use super::fixtures::{create_coin_asset, random_blinding, random_nonce};
use std::sync::Arc;
use z00z_core::assets::{
    decode_asset_pkg_json, encode_asset_pkg_json, payload_has_secret_field, Asset,
    ASSET_PKG_JSON_MAX_BYTES,
};
use z00z_utils::rng::SystemRngProvider;

fn make_asset(serial_id: u32, amount: u64) -> Asset {
    let definition = Arc::new(create_coin_asset([serial_id as u8; 32], 8));
    let blinding = random_blinding();
    let nonce = random_nonce();
    let mut rng = SystemRngProvider.rng();

    Asset::new(definition, serial_id, amount, &blinding, nonce, &mut rng).expect("test asset")
}

#[test]
fn test_asset_package_decode_oversized() {
    let asset = make_asset(0, 10);
    let dto = z00z_core::assets::AssetPkgWire::from_asset(&asset);
    let mut bytes = encode_asset_pkg_json(&dto).expect("encode dto json");
    bytes.extend(std::iter::repeat_n(b' ', ASSET_PKG_JSON_MAX_BYTES + 1));

    let err = decode_asset_pkg_json(&bytes).expect_err("oversized dto must fail");

    assert!(err.to_string().contains("payload too large"));
}

#[test]
fn test_asset_package_probe_oversized() {
    let asset = make_asset(0, 10);
    let dto = z00z_core::assets::AssetPkgWire::from_asset(&asset);
    let mut bytes = encode_asset_pkg_json(&dto).expect("encode dto json");
    bytes.extend(std::iter::repeat_n(b'\n', ASSET_PKG_JSON_MAX_BYTES + 1));

    let err = payload_has_secret_field(&bytes).expect_err("oversized probe must fail");

    assert!(err.to_string().contains("payload too large"));
}
