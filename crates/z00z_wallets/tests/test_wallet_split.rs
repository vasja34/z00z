#[test]
fn test_wallet_service_split() {
    let source = include_str!("../src/services/wallet_service.rs");

    for needle in [
        "mod wallet_service_core;",
        "mod wallet_service_reachability;",
        "mod wallet_service_state;",
        "pub use self::wallet_service_core::{ReceiverUsageOracle, Sleeper, WalletService};",
        "pub use self::wallet_service_state::RateLimitPrecheck;",
        "mod wallet_actions;",
        "mod wallet_session;",
        "mod wallet_store;",
        "mod wallet_store;",
        "stable",
        "provisional",
        "reachability",
    ] {
        assert!(
            source.contains(needle),
            "wallet_service.rs must expose the explicit module seam {needle}"
        );
    }

    for part in [
        "wallet_service_core.rs",
        "wallet_service_reachability.rs",
        "wallet_service_state.rs",
        "wallet_actions.rs",
        "wallet_session.rs",
        "wallet_store.rs",
    ] {
        let inline_include = format!("include!(\"{part}\");");
        assert!(
            !source.contains(&inline_include),
            "wallet_service.rs must no longer use old include assembly for {part}"
        );
    }
}

#[test]
fn test_wallet_keeps_lanes_demoted() {
    let source = include_str!("../src/services/mod.rs");

    for needle in [
        "pub(crate) mod wallet_service;",
        "pub use self::wallet_service::RateLimitPrecheck;",
        "pub use self::wallet_service::WalletService;",
    ] {
        assert!(
            source.contains(needle),
            "services/mod.rs must keep the shallow wallet service facade contract {needle}"
        );
    }

    for needle in [
        "wallet_actions",
        "wallet_session",
        "wallet_store",
        "ReceiverUsageOracle",
        "Sleeper",
    ] {
        let leaked_reexport = format!("pub use {needle}");
        assert!(
            !source.contains(&leaked_reexport),
            "services/mod.rs must not require callers to import deep wallet service seam {needle}"
        );
    }
}

#[test]
fn test_app_main_view_split() {
    let source = include_str!("../src/egui_views/app_main_view.rs");

    for part in [
        "ui_config.rs",
        "ui_state_machine.rs",
        "ui_theme.rs",
        "tab_registry.rs",
        "main_view_loaders.rs",
        "main_view.rs",
    ] {
        let needle = format!("include!(\"{part}\");");
        assert!(
            source.contains(&needle),
            "app_main_view.rs must keep facade include for {part}"
        );
    }
}

#[test]
fn test_wallet_source_split() {
    let source = include_str!("../src/wallet/core.rs");

    for part in [
        "chain_id.rs",
        "wallet_id.rs",
        "wallet_kernel.rs",
        "wallet_record.rs",
        "entity.rs",
    ] {
        let needle = format!("include!(\"{part}\");");
        assert!(
            source.contains(&needle),
            "core.rs must keep facade include for {part}"
        );
    }
}
