const WALLET_SERVICE_ACTIONS_SOURCE: &str = include_str!("../src/services/wallet_actions.rs");
const WALLET_SERVICE_ACTIONS_RUNTIME_SOURCE: &str =
    include_str!("../src/services/wallet_actions_runtime_inactive.rs");
const ASSET_IMPL_SOURCE: &str = include_str!("../src/rpc/asset_rpc_impl.rs");

#[test]
fn test_runtime_duplicate() {
    assert!(
        WALLET_SERVICE_ACTIONS_SOURCE.contains("include!(\"wallet_actions_receive.rs\");"),
        "canonical receive include must remain wired"
    );
    assert!(
        !WALLET_SERVICE_ACTIONS_SOURCE
            .contains("include!(\"wallet_actions_runtime_inactive.rs\");"),
        "duplicate runtime helper must remain unwired"
    );
    assert!(
        WALLET_SERVICE_ACTIONS_SOURCE
            .contains("Canonical receive wiring stays in wallet_actions_receive.rs."),
        "live include stack should keep the canonical receive note"
    );
    assert!(
        WALLET_SERVICE_ACTIONS_RUNTIME_SOURCE.contains("non-canonical duplicate"),
        "duplicate runtime helper should stay explicitly quarantined"
    );
}

#[test]
fn test_asset_test_suite() {
    assert!(
        ASSET_IMPL_SOURCE.contains("mod test_asset_impl;"),
        "asset_impl.rs must bind the canonical receive test module"
    );
    assert!(
        ASSET_IMPL_SOURCE.contains(
            "Canonical RPC receive tests stay module-local under src/rpc/test_asset_impl.rs."
        ),
        "asset_impl.rs must document the canonical test module path"
    );
}
