use z00z_core::assets::generate_asset_secret_checked;

#[test]
fn test_secret_len() {
    let secret = generate_asset_secret_checked().expect("secret");
    assert_eq!(secret.len(), 32);
}

#[test]
fn test_secret_not_zero() {
    let secret = generate_asset_secret_checked().expect("secret");
    assert_ne!(secret, [0u8; 32]);
}

#[test]
fn test_secret_unique() {
    let left = generate_asset_secret_checked().expect("left");
    let right = generate_asset_secret_checked().expect("right");
    assert_ne!(left, right);
}
