#![cfg(not(target_arch = "wasm32"))]

const DOC: &str = include_str!("../../../wiki/04-wallet-and-rpc/wallet-object-packages.md");
const ROUTES: &str = include_str!("../src/rpc/wallet_dispatcher_routes.rs");
const OBJECT_RPC: &str = include_str!("../src/rpc/object_rpc.rs");
const OBJECT_IMPL: &str = include_str!("../src/rpc/object_rpc_impl.rs");

#[test]
fn docs_keep_wallet_object_live() {
    assert!(DOC.contains("The live post-genesis wallet path is `wallet.object.*`"));
    assert!(DOC.contains("Public wallet-visible typed-object namespace"));
    assert!(!DOC.contains("wallet.object.* stub"));
    assert!(!DOC.contains("wallet.object.* is genesis-only"));
}

#[test]
fn routes_keep_object_namespace_registered() {
    for rpc in [
        "wallet.object.list_objects",
        "wallet.object.list_vouchers",
        "wallet.object.list_rights",
        "wallet.object.preview_package",
        "wallet.object.build_package",
        "wallet.object.accept_voucher",
        "wallet.object.reject_voucher",
        "wallet.object.redeem_voucher",
        "wallet.object.refund_voucher",
        "wallet.object.transfer_voucher",
        "wallet.object.delegate_right",
        "wallet.object.consume_right",
        "wallet.object.revoke_right",
        "wallet.object.challenge_right",
    ] {
        assert!(
            OBJECT_RPC.contains(rpc),
            "{rpc} must stay in the public trait"
        );
        assert!(ROUTES.contains(rpc), "{rpc} must stay in dispatcher routes");
    }

    assert!(OBJECT_IMPL.contains("build_object_package_impl"));
    assert!(OBJECT_IMPL.contains("preview_object_package_impl"));
    assert!(!OBJECT_IMPL.contains("stub_default"));
}
