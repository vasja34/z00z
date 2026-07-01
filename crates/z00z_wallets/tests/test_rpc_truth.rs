const MOD_RS: &str = include_str!("../src/rpc/methods.rs");
const APP_IMPL: &str = include_str!("../src/rpc/app_rpc_impl.rs");
const ASSET_IMPL: &str = include_str!("../src/rpc/asset_rpc_impl.rs");
const BACKUP_IMPL: &str = include_str!("../src/rpc/backup_rpc_impl.rs");
const KEY_API: &str = include_str!("../src/rpc/key_rpc.rs");
const STORAGE_API: &str = include_str!("../src/rpc/storage_rpc.rs");
const WALLET_IMPL: &str = include_str!("../src/rpc/wallet_rpc_impl.rs");
const APP_SERVICE_CONSTRUCTION: &str = include_str!("../src/services/app_service_construction.rs");
const BACKUP_SERVICE: &str = include_str!("../src/services/backup_service.rs");
const KEY_SERVICE: &str = include_str!("../src/services/key_service.rs");
const NETWORK_SERVICE: &str = include_str!("../src/services/network_service.rs");
const STORAGE_SERVICE: &str = include_str!("../src/services/storage_service.rs");
const WALLET_SERVICE_TYPES: &str = concat!(
    include_str!("../src/services/wallet_service_core.rs"),
    include_str!("../src/services/wallet_service_state.rs")
);
const REACHABILITY: &str = include_str!("../src/services/wallet_actions_reachability.rs");
const RECEIVE: &str = include_str!("../src/services/wallet_actions_receive.rs");
const ASSETS: &str = include_str!("../src/services/wallet_actions_assets_inactive.rs");
const BACKUP_RPC: &str = include_str!("../src/services/wallet_actions_backup_rpc_inactive.rs");
const KEY_IMPL: &str = include_str!("../src/rpc/key_rpc_impl.rs");
const STORAGE_IMPL: &str = include_str!("../src/rpc/storage_rpc_impl.rs");
const CHAIN_RPC: &str = include_str!("../src/rpc/chain_rpc.rs");
const CHAIN_IMPL: &str = include_str!("../src/rpc/chain_rpc_impl.rs");
const CHAIN_TYPES: &str = include_str!("../src/rpc/chain_types.rs");
const APP_CHAIN_WIRING: &str = include_str!("../src/rpc/app_dispatcher_wiring.rs");
const APP_CHAIN_NETWORK: &str = include_str!("../src/services/app_chain_network.rs");
const CHAIN_SERVICE: &str = include_str!("../src/services/chain_service.rs");
const CREATE_WALLET_E2E: &str = include_str!("test_create_wallet_crypto_e2e.rs");
const SHOW_SEED_PLAINTEXT: &str = include_str!("test_show_seed_phrase_plaintext.rs");
const BACKUP_RESTORE_IDENTITY: &str = include_str!("test_backup_restore_identity.rs");

fn phrase(parts: &[&str]) -> String {
    parts.concat()
}

#[test]
fn test_text_truth() {
    let targets = [
        ("methods_mod", MOD_RS),
        ("app_impl", APP_IMPL),
        ("asset_impl", ASSET_IMPL),
        ("backup_impl", BACKUP_IMPL),
        ("key_api", KEY_API),
        ("storage_api", STORAGE_API),
        ("wallet_impl", WALLET_IMPL),
        ("app_service_construction", APP_SERVICE_CONSTRUCTION),
        ("backup_service", BACKUP_SERVICE),
        ("key_service", KEY_SERVICE),
        ("network_service", NETWORK_SERVICE),
        ("storage_service", STORAGE_SERVICE),
        ("wallet_service_types", WALLET_SERVICE_TYPES),
        ("reachability", REACHABILITY),
        ("receive", RECEIVE),
        ("assets", ASSETS),
        ("backup_rpc", BACKUP_RPC),
        ("key_impl", KEY_IMPL),
        ("storage_impl", STORAGE_IMPL),
        ("create_wallet_e2e", CREATE_WALLET_E2E),
        ("show_seed_plaintext", SHOW_SEED_PLAINTEXT),
        ("backup_restore_identity", BACKUP_RESTORE_IDENTITY),
    ];

    let banned = [
        phrase(&["placeholder RPC", " paths"]),
        phrase(&["Phase 1 reachability", " stub"]),
        phrase(&["Phase 1 reachability", " placeholder"]),
        phrase(&["Returns a deterministic", " placeholder"]),
        phrase(&[
            "Phase 030 residue placeholder pending deletion",
            " or replacement.",
        ]),
        phrase(&["KeyRpc stub implementation", " (Phase 1)"]),
        phrase(&["Returns placeholder data for", " testing RPC layer."]),
        phrase(&["Real implementation will be added", " in Phase 2."]),
        phrase(&["Storage RPC implementation", " (Phase 1 stubs)."]),
        phrase(&[
            "Returns realistic placeholder data",
            " for privileged operations.",
        ]),
        phrase(&["Stub implementations for asset.* RPC methods", " (Phase 1)"]),
        phrase(&["Backup RPC service implementation", " (Phase 1)"]),
        phrase(&["Stub implementation for app.* RPC methods", " (Phase 1)"]),
        phrase(&["Run the Phase 1", " master-key rotation flow."]),
        phrase(&[
            "Current Phase 1 behavior performs the full authorization, audit,",
            " and rate-limit flow,",
        ]),
        phrase(&["Stub implementations", " (Phase 1)"]),
        phrase(&["Wallet RPC service implementation", " (stub for Phase 1)"]),
        phrase(&["Create a new `AppService`", " (Phase 1 stub)."]),
        phrase(&["Wallet service", " (stub implementation for Phase 1)"]),
        phrase(&["Stub wallet service for", " Phase 1 RPC testing"]),
        phrase(&[
            "Returns placeholder data to allow",
            " RPC layer testing without",
        ]),
        phrase(&["Backup service placeholder", " (stub)."]),
        phrase(&["Key management service placeholder", " (stub)."]),
        phrase(&["Network service placeholder", " (stub)."]),
        phrase(&["Storage service placeholder", " (stub)."]),
        phrase(&[
            "Temporary stub that returns placeholder",
            " data for RPC testing.",
        ]),
        phrase(&["format!(\"wallet_{wallet_id_hex}", ".bin\")"]),
    ];

    for (name, source) in targets {
        for needle in &banned {
            assert!(
                !source.contains(needle),
                "{name} still contains stale 047 wording: {needle}"
            );
        }
    }
}

#[test]
fn test_truth_marks() {
    assert!(REACHABILITY.contains("structural audit wallet facade"));
    assert!(REACHABILITY.contains("explicit guard paths"));
    assert!(APP_IMPL.contains("App RPC implementations backed by `AppService`."));
    assert!(ASSET_IMPL.contains("Canonical send or receive authority flows through"));
    assert!(BACKUP_IMPL.contains("manifest-backed `.wlt` plus JSONL"));
    assert!(WALLET_IMPL.contains("Wallet RPC implementations backed by `WalletService`."));
    assert!(APP_SERVICE_CONSTRUCTION.contains("default wallet and chain services"));
    assert!(BACKUP_SERVICE.contains("Zero-state backup-domain service marker"));
    assert!(KEY_SERVICE.contains("Zero-state key-domain service marker"));
    assert!(NETWORK_SERVICE.contains("Zero-state network-domain service marker"));
    assert!(STORAGE_SERVICE.contains("Zero-state storage-domain service marker"));
    assert!(WALLET_SERVICE_TYPES.contains("wallet-facing authority lanes"));
    assert!(WALLET_SERVICE_TYPES.contains("pub type ReceiverUsageOracle"));
    assert!(WALLET_SERVICE_TYPES.contains("pub enum RateLimitPrecheck"));
    assert!(RECEIVE.contains("Restricted asset-op surface"));
    assert!(KEY_IMPL.contains("KeyRpc live implementation."));
    assert!(KEY_API.contains("current master-key rotation flow"));
    assert!(STORAGE_API.contains("current wallet outputs layout"));
    assert!(STORAGE_IMPL.contains("`.wlt` packs plus JSONL sidecars"));
}

#[test]
fn test_local_chain_scan_contract_truth() {
    for route in [
        "app.chain.start_local_scan",
        "app.chain.stop_local_scan",
        "app.chain.get_local_scan_status",
        "app.chain.get_local_scan_tip",
    ] {
        assert!(
            APP_CHAIN_WIRING.contains(route),
            "dispatcher wiring must keep the explicit local-only route {route}"
        );
    }

    for retired in [
        "app.chain.start_scan",
        "app.chain.stop_scan",
        "app.chain.get_scan_status",
        "app.chain.get_blockchain_tip",
    ] {
        assert!(
            !APP_CHAIN_WIRING.contains(retired),
            "retired production-looking route must stay absent: {retired}"
        );
    }

    assert!(CHAIN_RPC.contains("wallet-local scan orchestration control"));
    assert!(CHAIN_IMPL.contains("Wallet-local ChainScanRpc implementation."));
    assert!(CHAIN_TYPES.contains("Wallet-local chain-tip observation."));
    assert!(APP_CHAIN_NETWORK.contains("wallet-local chain-tip observation"));
    assert!(CHAIN_SERVICE.contains("wallet-local chain-tip observation"));
}
