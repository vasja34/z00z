mod tests {
    use super::*;
    use crate::key::Bip44Path;
    use crate::rpc::logging::RpcLoggingConfig;
    use crate::security::password::PasswordValidator;
    use crate::services::{ChainService, WalletService};
    use std::ffi::OsString;
    use tempfile::TempDir;
    use z00z_utils::rng::SystemRngProvider;

    #[tokio::test]
    async fn test_rejects_wallet_create_whitespace() {
        let _lock = RpcLoggingConfig::__lock_wallet_config_env_async().await;
        let _restore = WalletConfigEnvRestore::capture();
        std::env::remove_var("Z00Z_WALLET_CONFIG_PATH");
        std::env::remove_var("Z00Z_WALLET_NETWORK");
        std::env::remove_var("Z00Z_WALLET_CHAIN");

        let service = AppService::new();

        let result = service
            .create_wallet("   ".to_string(), "Aa1!aaaaaaaa".to_string(), None)
            .await;

        assert!(matches!(result, Err(WalletError::InvalidParams(_))));
    }

    #[tokio::test]
    async fn test_rejects_common_password() {
        let _lock = RpcLoggingConfig::__lock_wallet_config_env_async().await;
        let _restore = WalletConfigEnvRestore::capture();
        std::env::remove_var("Z00Z_WALLET_CONFIG_PATH");
        std::env::remove_var("Z00Z_WALLET_NETWORK");
        std::env::remove_var("Z00Z_WALLET_CHAIN");

        let service = AppService::new();

        let result = service
            .create_wallet("wallet1".to_string(), "Password123!".to_string(), None)
            .await;

        assert!(matches!(result, Err(WalletError::InvalidParams(_))));
    }

    #[tokio::test]
    async fn test_create_wallet_strength_score() {
        let _lock = RpcLoggingConfig::__lock_wallet_config_env_async().await;
        let _restore = WalletConfigEnvRestore::capture();
        std::env::remove_var("Z00Z_WALLET_CONFIG_PATH");
        std::env::remove_var("Z00Z_WALLET_NETWORK");
        std::env::remove_var("Z00Z_WALLET_CHAIN");

        let (service, _wallets, _chain, _dir) = test_service_with_tempdir();

        let password = "Aa1!bB2@cC3#dD4$eE5%";
        let response = service
            .create_wallet("wallet1".to_string(), password.to_string(), None)
            .await
            .unwrap();

        let expected = PasswordValidator::new(
            crate::services::wallet_runtime_config::resolve_wallet_password_policy().unwrap(),
        )
        .strength_score(password);
        assert_eq!(response.password_strength_score, expected);
    }

    #[tokio::test]
    async fn test_create_yaml_settings_defaults() {
        let _lock = RpcLoggingConfig::__lock_wallet_config_env_async().await;
        let _restore = WalletConfigEnvRestore::capture();
        std::env::remove_var("Z00Z_WALLET_NETWORK");
        std::env::remove_var("Z00Z_WALLET_CHAIN");

        let cfg_dir = tempfile::tempdir().unwrap();
        let cfg_path = write_wallet_config(
            &cfg_dir,
            r#"
wallet:
  network:
    type: "p2p"
  chain:
    type: "devnet"
  settings:
    auto_lock_timeout_secs: 42
    default_fee: "0.125"
    currency_display: "TOK"
  auto_lock:
    timeout_secs: 42
    triggers: []
"#,
        );
        std::env::set_var("Z00Z_WALLET_CONFIG_PATH", &cfg_path);

        let (service, wallets, _chain, _dir) = test_service_with_tempdir();
        let response = service
            .create_wallet(
                "wallet1".to_string(),
                "Aa1!bB2@cC3#dD4$eE5%".to_string(),
                None,
            )
            .await
            .unwrap();

        let settings = wallets
            .get_wallet_settings(&response.wallet_id)
            .await
            .unwrap();
        assert_eq!(
            settings.auto_lock_timeout, 42,
            "phase-047 requires wallet create to seed auto-lock from wallet.settings.auto_lock_timeout_secs"
        );
        assert_eq!(
            settings.default_fee, "0.125",
            "phase-047 requires wallet create to seed default_fee from wallet.settings.default_fee"
        );
        assert_eq!(
            settings.currency_display, "TOK",
            "phase-047 requires wallet create to seed currency_display from wallet.settings.currency_display"
        );
    }

    #[tokio::test]
    async fn test_create_yaml_password_policy() {
        let _lock = RpcLoggingConfig::__lock_wallet_config_env_async().await;
        let _restore = WalletConfigEnvRestore::capture();
        std::env::remove_var("Z00Z_WALLET_NETWORK");
        std::env::remove_var("Z00Z_WALLET_CHAIN");

        let cfg_dir = tempfile::tempdir().unwrap();
        let cfg_path = write_wallet_config(
            &cfg_dir,
            r#"
wallet:
  network:
    type: "p2p"
  chain:
    type: "devnet"
  settings:
    auto_lock_timeout_secs: 42
    default_fee: "0.125"
    currency_display: "TOK"
  auto_lock:
    timeout_secs: 42
    triggers: []
  security:
    password_policy:
      min_length: 24
      recommended_length: 28
      max_length: 64
"#,
        );
        std::env::set_var("Z00Z_WALLET_CONFIG_PATH", &cfg_path);

        let (service, _wallets, _chain, _dir) = test_service_with_tempdir();
        let err = service
            .create_wallet(
                "wallet1".to_string(),
                "Aa1!bB2@cC3#dD4$eE5%".to_string(),
                None,
            )
            .await
            .expect_err("phase-047 requires create_wallet to use YAML-backed password policy");
        assert!(err.to_string().contains("Password too short"));
    }

    #[tokio::test]
    async fn test_invalid_yaml_password_policy() {
        let _lock = RpcLoggingConfig::__lock_wallet_config_env_async().await;
        let _restore = WalletConfigEnvRestore::capture();
        std::env::remove_var("Z00Z_WALLET_NETWORK");
        std::env::remove_var("Z00Z_WALLET_CHAIN");

        let cfg_dir = tempfile::tempdir().unwrap();
        let cfg_path = write_wallet_config(
            &cfg_dir,
            r#"
wallet:
  network:
    type: "p2p"
  chain:
    type: "devnet"
  settings:
    auto_lock_timeout_secs: 42
    default_fee: "0.125"
    currency_display: "TOK"
  auto_lock:
    timeout_secs: 42
    triggers: []
  security:
    password_policy:
      min_length: 20
      recommended_length: 10
      max_length: 64
"#,
        );
        std::env::set_var("Z00Z_WALLET_CONFIG_PATH", &cfg_path);

        let (service, _wallets, _chain, _dir) = test_service_with_tempdir();
        let err = service
            .create_wallet(
                "wallet1".to_string(),
                "Aa1!bB2@cC3#dD4$eE5%".to_string(),
                None,
            )
            .await
            .expect_err("invalid wallet password policy config must fail closed");
        assert!(err.to_string().contains("recommended_length"));
    }

    #[test]
    fn test_wallet_create_request() {
        let src = include_str!("app_wallet_lifecycle.rs");

        let discarded_request = "let _request = self.core_app.create_wallet".to_string() + "(";
        let discarded_result = "let _ = self.core_app.create_wallet".to_string() + "(";

        assert!(
            !src.contains(&discarded_request),
            "core wallet creation request must not be discarded"
        );
        assert!(
            !src.contains(&discarded_result),
            "core wallet creation request must not be discarded"
        );
    }

    #[test]
    fn test_wallet_create_no_bypass() {
        let facade = include_str!("wallet_service.rs");
        let store_create = include_str!("wallet_store_create_unlock.rs");

        assert!(
            facade.contains("mod wallet_store;"),
            "WalletService facade must keep wallet_store behind a named internal module"
        );

        assert!(
            store_create.contains("pub(crate) async fn create_wallet_using_explicit_identity"),
            "WalletService wallet creation must stay crate-internal for the app orchestrator"
        );

        assert!(
            !store_create.contains("pub async fn create_wallet_using_explicit_identity"),
            "WalletService wallet creation must not be public"
        );

        assert!(
            store_create.contains("pub(crate) async fn create_wallet_using_explicit_identity\n")
                || store_create
                    .contains("pub(crate) async fn create_wallet_using_explicit_identity("),
            "Expected WalletService to expose a crate-internal create_wallet_using_explicit_identity"
        );

        assert!(
            store_create.contains("pub(crate) async fn create_wallet_using_explicit_identity")
                && store_create.contains("seed_phrase: &str"),
            "WalletService wallet creation must require a seed phrase (validated by orchestrator)"
        );
    }

    #[tokio::test]
    async fn test_phrase_validation_no_leak() {
        let _lock = RpcLoggingConfig::__lock_wallet_config_env_async().await;
        let _restore = WalletConfigEnvRestore::capture();
        std::env::remove_var("Z00Z_WALLET_CONFIG_PATH");
        std::env::remove_var("Z00Z_WALLET_NETWORK");
        std::env::remove_var("Z00Z_WALLET_CHAIN");

        let service = AppService::new();

        let invalid_word = "notaword";
        let seed = format!(
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon {invalid_word}"
        );

        let err = service
            .create_wallet(
                "wallet1".to_string(),
                "CorrectHorseBatteryStaple1!".to_string(),
                Some(seed.clone()),
            )
            .await
            .expect_err("expected create_wallet to fail");

        let msg = err.to_string();
        assert!(
            !msg.contains(invalid_word),
            "error message leaked a seed word"
        );
        assert!(
            !msg.contains(&seed),
            "error message leaked the full seed phrase"
        );
    }

    fn test_seed_phrase_24() -> &'static str {
        "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art"
    }

    fn test_wlt_path(
        output_dir: &std::path::Path,
        wallet_id: &PersistWalletId,
    ) -> std::path::PathBuf {
        use crate::domains::hashing::compute_wallet_file_id;
        let hash = compute_wallet_file_id(&wallet_id.0);
        let wallet_id_hex = hex::encode(&hash[..8]);
        output_dir.join(format!("wallet_{wallet_id_hex}.wlt"))
    }

    fn test_service_with_tempdir() -> (AppService, Arc<WalletService>, Arc<ChainService>, TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let time_provider: Arc<dyn TimeProvider> = Arc::new(SystemTimeProvider);

        let wallets = Arc::new(WalletService::create_service_custom_output_directory(
            dir.path().to_path_buf(),
            Arc::clone(&time_provider),
            SystemRngProvider,
        ));

        let chain_service = Arc::new(ChainService::with_dependencies(Arc::clone(&time_provider)));

        let app = AppService::with_dependencies_and_services(
            time_provider,
            Arc::clone(&wallets),
            Arc::clone(&chain_service),
        );

        (app, wallets, chain_service, dir)
    }

    struct WalletConfigEnvRestore {
        prev_path: Option<OsString>,
        prev_network: Option<OsString>,
        prev_chain: Option<OsString>,
    }

    impl WalletConfigEnvRestore {
        fn capture() -> Self {
            Self {
                prev_path: std::env::var_os("Z00Z_WALLET_CONFIG_PATH"),
                prev_network: std::env::var_os("Z00Z_WALLET_NETWORK"),
                prev_chain: std::env::var_os("Z00Z_WALLET_CHAIN"),
            }
        }
    }

    impl Drop for WalletConfigEnvRestore {
        fn drop(&mut self) {
            match &self.prev_path {
                Some(value) => std::env::set_var("Z00Z_WALLET_CONFIG_PATH", value),
                None => std::env::remove_var("Z00Z_WALLET_CONFIG_PATH"),
            }
            match &self.prev_network {
                Some(value) => std::env::set_var("Z00Z_WALLET_NETWORK", value),
                None => std::env::remove_var("Z00Z_WALLET_NETWORK"),
            }
            match &self.prev_chain {
                Some(value) => std::env::set_var("Z00Z_WALLET_CHAIN", value),
                None => std::env::remove_var("Z00Z_WALLET_CHAIN"),
            }
        }
    }

    fn write_wallet_config(dir: &TempDir, yaml: &str) -> std::path::PathBuf {
        let path = dir.path().join("wallet_config.yaml");
        z00z_utils::io::write_file(&path, yaml.as_bytes()).expect("wallet config must write");
        path
    }

    #[tokio::test]
    async fn test_recover_persists_indexes() {
        let _lock = RpcLoggingConfig::__lock_wallet_config_env_async().await;
        let _restore = WalletConfigEnvRestore::capture();
        std::env::remove_var("Z00Z_WALLET_CONFIG_PATH");
        std::env::set_var("Z00Z_WALLET_NETWORK", "p2p");
        std::env::set_var("Z00Z_WALLET_CHAIN", "devnet");

        let (service, wallets, chain, dir) = test_service_with_tempdir();

        chain
            .set_used_paths(vec![
                Bip44Path::payment_for_account(0, 5).unwrap(),
                Bip44Path::change_path_for_account(0, 2).unwrap(),
            ])
            .await;

        let response = service
            .recover_from_seed(
                "recovered_wallet".to_string(),
                "Aa1!bB2@cC3#dD4$eE5%".to_string(),
                test_seed_phrase_24().to_string(),
                test_seed_phrase_24().to_string(),
                "p2p".to_string(),
                "devnet".to_string(),
            )
            .await
            .unwrap();

        let expected = PasswordValidator::default().strength_score("Aa1!bB2@cC3#dD4$eE5%");
        assert_eq!(response.password_strength_score, expected);

        assert_eq!(response.name, "recovered_wallet");
        assert_eq!(response.network, "p2p");
        assert_eq!(response.chain, "devnet");

        let state = wallets.get_wallet_state(&response.wallet_id).await.unwrap();
        assert!(state.is_locked());

        let counters = wallets
            .get_deriver_state(&response.wallet_id)
            .await
            .unwrap();
        assert_eq!(counters.next_payment_index, 6);
        assert_eq!(counters.next_change_index, 3);

        let time_provider: Arc<dyn TimeProvider> = Arc::new(SystemTimeProvider);
        let wallets_restart = WalletService::create_service_custom_output_directory(
            dir.path().to_path_buf(),
            Arc::clone(&time_provider),
            SystemRngProvider,
        );

        let wlt_path = test_wlt_path(dir.path(), &response.wallet_id);
        assert!(wlt_path.exists(), "expected .wlt file to exist");

        wallets_restart
            .open_wallet_source(WalletSource::Path {
                path: wlt_path.to_string_lossy().to_string(),
            })
            .await
            .unwrap();

        wallets_restart
            .load_wallet(&response.wallet_id, "Aa1!bB2@cC3#dD4$eE5%")
            .await
            .unwrap();

        let counters_restart = wallets_restart
            .get_deriver_state(&response.wallet_id)
            .await
            .unwrap();
        assert_eq!(counters_restart.next_payment_index, 6);
        assert_eq!(counters_restart.next_change_index, 3);
    }

    #[tokio::test]
    async fn test_recover_requires_match() {
        let _lock = RpcLoggingConfig::__lock_wallet_config_env_async().await;
        let _restore = WalletConfigEnvRestore::capture();
        std::env::remove_var("Z00Z_WALLET_CONFIG_PATH");
        let (service, _wallets, _chain, dir) = test_service_with_tempdir();

        let err = service
            .recover_from_seed(
                "recovered_wallet".to_string(),
                "Aa1!bB2@cC3#dD4$eE5%".to_string(),
                test_seed_phrase_24().to_string(),
                "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon".to_string(),
                "p2p".to_string(),
                "devnet".to_string(),
            )
            .await
            .expect_err("expected recover_from_seed to fail");

        assert!(
            err.to_string().contains("do not match"),
            "expected a mismatch error"
        );

        let wlt_count = std::fs::read_dir(dir.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "wlt"))
            .count();

        assert_eq!(wlt_count, 0, "expected no .wlt files to be created");
    }

    #[tokio::test]
    async fn test_recover_rejects_empty_network() {
        let _lock = RpcLoggingConfig::__lock_wallet_config_env_async().await;
        let _restore = WalletConfigEnvRestore::capture();
        std::env::remove_var("Z00Z_WALLET_CONFIG_PATH");
        let (service, _wallets, _chain, dir) = test_service_with_tempdir();

        let err = service
            .recover_from_seed(
                "recovered_wallet".to_string(),
                "Aa1!aaaaaaaa".to_string(),
                test_seed_phrase_24().to_string(),
                test_seed_phrase_24().to_string(),
                "   ".to_string(),
                "devnet".to_string(),
            )
            .await
            .expect_err("expected recover_from_seed to fail");

        assert!(
            err.to_string().contains("Wallet network cannot be empty"),
            "expected an empty-network error"
        );

        let wlt_count = std::fs::read_dir(dir.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "wlt"))
            .count();

        assert_eq!(wlt_count, 0, "expected no .wlt files to be created");
    }

    #[tokio::test]
    async fn test_recover_rejects_empty_chain() {
        let _lock = RpcLoggingConfig::__lock_wallet_config_env_async().await;
        let _restore = WalletConfigEnvRestore::capture();
        std::env::remove_var("Z00Z_WALLET_CONFIG_PATH");
        let (service, _wallets, _chain, dir) = test_service_with_tempdir();

        let err = service
            .recover_from_seed(
                "recovered_wallet".to_string(),
                "Aa1!aaaaaaaa".to_string(),
                test_seed_phrase_24().to_string(),
                test_seed_phrase_24().to_string(),
                "p2p".to_string(),
                "".to_string(),
            )
            .await
            .expect_err("expected recover_from_seed to fail");

        assert!(
            err.to_string().contains("Wallet chain cannot be empty"),
            "expected an empty-chain error"
        );

        let wlt_count = std::fs::read_dir(dir.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "wlt"))
            .count();

        assert_eq!(wlt_count, 0, "expected no .wlt files to be created");
    }

    #[tokio::test]
    async fn test_recover_keeps_counters_zero() {
        let _lock = RpcLoggingConfig::__lock_wallet_config_env_async().await;
        let _restore = WalletConfigEnvRestore::capture();
        std::env::remove_var("Z00Z_WALLET_CONFIG_PATH");
        std::env::set_var("Z00Z_WALLET_NETWORK", "p2p");
        std::env::set_var("Z00Z_WALLET_CHAIN", "devnet");

        let (service, wallets, chain, dir) = test_service_with_tempdir();

        chain.set_used_paths(vec![]).await;

        let response = service
            .recover_from_seed(
                "recovered_wallet".to_string(),
                "Aa1!bB2@cC3#dD4$eE5%".to_string(),
                test_seed_phrase_24().to_string(),
                test_seed_phrase_24().to_string(),
                "p2p".to_string(),
                "devnet".to_string(),
            )
            .await
            .unwrap();

        let counters = wallets
            .get_deriver_state(&response.wallet_id)
            .await
            .unwrap();
        assert_eq!(counters.next_payment_index, 0);
        assert_eq!(counters.next_change_index, 0);

        let time_provider: Arc<dyn TimeProvider> = Arc::new(SystemTimeProvider);
        let wallets_restart = WalletService::create_service_custom_output_directory(
            dir.path().to_path_buf(),
            Arc::clone(&time_provider),
            SystemRngProvider,
        );

        let wlt_path = test_wlt_path(dir.path(), &response.wallet_id);
        assert!(wlt_path.exists(), "expected .wlt file to exist");

        wallets_restart
            .open_wallet_source(WalletSource::Path {
                path: wlt_path.to_string_lossy().to_string(),
            })
            .await
            .unwrap();

        wallets_restart
            .load_wallet(&response.wallet_id, "Aa1!bB2@cC3#dD4$eE5%")
            .await
            .unwrap();

        let counters_restart = wallets_restart
            .get_deriver_state(&response.wallet_id)
            .await
            .unwrap();
        assert_eq!(counters_restart.next_payment_index, 0);
        assert_eq!(counters_restart.next_change_index, 0);
    }

    #[tokio::test]
    async fn test_gap_zero_rejected() {
        let _lock = RpcLoggingConfig::__lock_wallet_config_env_async().await;
        let _restore = WalletConfigEnvRestore::capture();
        std::env::remove_var("Z00Z_WALLET_CONFIG_PATH");
        std::env::remove_var("Z00Z_WALLET_NETWORK");
        std::env::remove_var("Z00Z_WALLET_CHAIN");

        let (service, wallets, _chain, _dir) = test_service_with_tempdir();

        let response = service
            .create_wallet(
                "wallet1".to_string(),
                "Aa1!bB2@cC3#dD4$eE5%".to_string(),
                Some(test_seed_phrase_24().to_string()),
            )
            .await
            .unwrap();

        let password = SafePassword::from("Aa1!bB2@cC3#dD4$eE5%");
        let _token = wallets
            .unlock_wallet_in_memory(&response.wallet_id, &password)
            .await
            .unwrap();

        let is_used: crate::services::wallet_service::ReceiverUsageOracle =
            Arc::new(|_path, _pk| Box::pin(async move { Ok(false) }));

        let err = wallets
            .reconcile_persist_gap_limit(&response.wallet_id, 0, is_used)
            .await
            .expect_err("expected gap_limit=0 to be rejected");

        assert!(
            err.to_string().contains("gap_limit")
                && (err.to_string().contains("> 0") || err.to_string().contains("greater than 0")),
            "expected an error mentioning gap_limit must be > 0, got: {err}"
        );
    }

    #[tokio::test]
    async fn test_gap_stops_unused() {
        let _lock = RpcLoggingConfig::__lock_wallet_config_env_async().await;
        let _restore = WalletConfigEnvRestore::capture();
        std::env::remove_var("Z00Z_WALLET_CONFIG_PATH");
        std::env::remove_var("Z00Z_WALLET_NETWORK");
        std::env::remove_var("Z00Z_WALLET_CHAIN");

        let (service, wallets, _chain, _dir) = test_service_with_tempdir();

        let response = service
            .create_wallet(
                "wallet1".to_string(),
                "Aa1!bB2@cC3#dD4$eE5%".to_string(),
                Some(test_seed_phrase_24().to_string()),
            )
            .await
            .unwrap();

        let password = SafePassword::from("Aa1!bB2@cC3#dD4$eE5%");
        let _token = wallets
            .unlock_wallet_in_memory(&response.wallet_id, &password)
            .await
            .unwrap();

        let counters_before = wallets
            .get_deriver_state(&response.wallet_id)
            .await
            .unwrap();

        let is_used: crate::services::wallet_service::ReceiverUsageOracle =
            Arc::new(|path, _pk| {
                Box::pin(async move { Ok(path == Bip44Path::payment_for_account(0, 25).unwrap()) })
            });

        wallets
            .reconcile_persist_gap_limit(&response.wallet_id, 5, is_used)
            .await
            .unwrap();

        let counters_after = wallets
            .get_deriver_state(&response.wallet_id)
            .await
            .unwrap();
        assert_eq!(
            counters_after.next_payment_index,
            counters_before.next_payment_index
        );
        assert_eq!(
            counters_after.next_change_index,
            counters_before.next_change_index
        );
    }

    #[tokio::test]
    async fn test_seed_yaml_gap_limit() {
        let _lock = RpcLoggingConfig::__lock_wallet_config_env_async().await;
        let _restore = WalletConfigEnvRestore::capture();
        std::env::remove_var("Z00Z_WALLET_NETWORK");
        std::env::remove_var("Z00Z_WALLET_CHAIN");

        let cfg_dir = tempfile::tempdir().unwrap();
        let cfg_path = write_wallet_config(
            &cfg_dir,
            r#"
wallet:
  recovery:
    gap_limit: 5
"#,
        );
        std::env::set_var("Z00Z_WALLET_CONFIG_PATH", &cfg_path);

        let (service, wallets, chain, _dir) = test_service_with_tempdir();
        chain
            .set_used_paths(vec![Bip44Path::payment_for_account(0, 5).unwrap()])
            .await;

        let response = service
            .recover_from_seed(
                "recovered_wallet".to_string(),
                "Aa1!bB2@cC3#dD4$eE5%".to_string(),
                test_seed_phrase_24().to_string(),
                test_seed_phrase_24().to_string(),
                "p2p".to_string(),
                "devnet".to_string(),
            )
            .await
            .unwrap();

        let counters = wallets
            .get_deriver_state(&response.wallet_id)
            .await
            .unwrap();
        assert_eq!(
            counters.next_payment_index, 0,
            "phase-047 requires recovery to stop after wallet.recovery.gap_limit consecutive unused addresses instead of scanning with the hardcoded width"
        );
        assert_eq!(counters.next_change_index, 0);
    }
}
