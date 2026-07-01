mod tests {
    use super::*;

    #[test]
    fn test_default_output_stable_env() {
        let resolved = resolve_output_dir_sources(
            &|_key| None,
            r#"
wallet:
  paths:
    output_dir: "outputs/wallets"
"#,
        );
        assert_eq!(resolved, PathBuf::from("outputs").join("wallets"));
    }

    #[test]
    fn test_output_yaml_default_used() {
        let resolved = resolve_output_dir_sources(
            &|_key| None,
            r#"
wallet:
  paths:
    output_dir: "custom_outputs/wallets"
"#,
        );

        assert_eq!(resolved, PathBuf::from("custom_outputs/wallets"));
    }

    #[test]
    fn test_output_env_override_takes() {
        let resolved = resolve_output_dir_sources(
            &|key| {
                if key == "Z00Z_WALLET_OUTPUT_DIR" {
                    Some("/tmp/z00z_wallets_override".into())
                } else {
                    None
                }
            },
            r#"
wallet:
  paths:
    output_dir: "custom_outputs/wallets"
"#,
        );

        assert_eq!(resolved, PathBuf::from("/tmp/z00z_wallets_override"));
    }

    #[test]
    fn test_wallet_identity_defaults() {
        let identity = resolve_wallet_identity_sources(&|_key| None, r#"wallet: {}"#);
        assert_eq!(identity.network, "p2p");
        assert_eq!(identity.chain, "devnet");
    }

    #[test]
    fn test_rate_limit_env_ok() {
        let resolved = resolve_receiver_rate_limit(
            &|key| match key {
                "Z00Z_WALLET_RECEIVER_DERIVE_RATE_PER_SEC" => Some("100".into()),
                "Z00Z_WALLET_RECEIVER_DERIVE_BURST" => Some("500".into()),
                _ => None,
            },
            "wallet: {}",
        )
        .unwrap();

        assert_eq!(
            resolved,
            Some(ReceiverDeriveRateLimit {
                rate_per_sec: 100,
                burst: 500
            })
        );
    }

    #[test]
    fn test_rate_limit_yaml_ok() {
        let resolved = resolve_receiver_rate_limit(
            &|_key| None,
            r#"
wallet:
  receiver:
    rate_limit:
      enabled: true
      rate_per_sec: 10
      burst: 20
"#,
        )
        .unwrap();

        assert_eq!(
            resolved,
            Some(ReceiverDeriveRateLimit {
                rate_per_sec: 10,
                burst: 20
            })
        );
    }

    #[test]
    fn test_rate_limit_yaml_off() {
        let resolved = resolve_receiver_rate_limit(
            &|_key| None,
            r#"
wallet:
  receiver:
    rate_limit:
      enabled: false
      rate_per_sec: 10
      burst: 20
"#,
        )
        .unwrap();

        assert_eq!(resolved, None);
    }

    #[test]
    fn test_rate_limit_partial_rejected() {
        let err = resolve_receiver_rate_limit(
            &|_key| None,
            r#"
wallet:
    receiver:
        rate_limit:
            rate_per_sec: 10
"#,
        )
        .unwrap_err();

        let msg = err.to_string();
        assert!(msg.contains("wallet.receiver.rate_limit"));
    }

    #[test]
    fn test_wallet_identity_yaml_used() {
        let identity = resolve_wallet_identity_sources(
            &|_key| None,
            r#"
wallet:
  network:
    type: "tor"
  chain:
        type: "mainnet"
"#,
        );
        assert_eq!(identity.network, "tor");
        assert_eq!(identity.chain, "mainnet");
    }

    #[test]
    fn test_wallet_identity_id_compat() {
        let identity = resolve_wallet_identity_sources(
            &|_key| None,
            r#"
wallet:
    network:
        type: "tor"
    chain:
        id: "mainnet"
"#,
        );
        assert_eq!(identity.network, "tor");
        assert_eq!(identity.chain, "mainnet");
    }

    #[test]
    fn test_receiver_cache_size_used() {
        let size = resolve_receiver_cache_size_sources(&|_key| None, "wallet: {}").unwrap();

        assert_eq!(size, DEFAULT_CACHE_SIZE);
    }

    #[test]
    fn test_receiver_size_yaml_valid() {
        let size = resolve_receiver_cache_size_sources(
            &|_key| None,
            r#"
wallet:
  receiver:
    cache_size: 123
"#,
        )
        .unwrap();

        assert_eq!(size, 123);
    }

    #[test]
    fn test_receiver_size_zero_rejected() {
        let err = resolve_receiver_cache_size_sources(
            &|_key| None,
            r#"
wallet:
      receiver:
        cache_size: 0
"#,
        )
        .unwrap_err();

        assert!(matches!(err, WalletError::InvalidParams(_)));
    }

    #[test]
    fn test_cache_size_max_rejected() {
        let too_large = MAX_CACHE_SIZE + 1;
        let yaml = format!("\nwallet:\n  receiver:\n    cache_size: {too_large}\n");

        let err = resolve_receiver_cache_size_sources(&|_key| None, &yaml).unwrap_err();
        assert!(matches!(err, WalletError::InvalidParams(_)));
    }

    #[test]
    fn test_embedded_wallet_defaults() {
        let yaml = include_str!("../config/wallet_config.yaml");
        let config = z00z_utils::config::YamlConfig::from_yaml_str(yaml)
            .expect("embedded wallet_config.yaml must stay parseable");

        for key in [
            "wallet.settings.auto_lock_timeout_secs",
            "wallet.settings.default_fee",
            "wallet.settings.currency_display",
            "wallet.auto_lock.timeout_secs",
            "wallet.backup.auto_backup_enabled",
            "wallet.backup.backup_interval_hours",
            "wallet.backup.location",
            "wallet.backup.encrypt_backups",
            "wallet.recovery.gap_limit",
        ] {
            let value = config
                .get(key)
                .expect("phase-047 wallet config lookup must not error");
            assert!(
                value.is_some(),
                "phase-047 requires `{key}` to be present in embedded wallet_config.yaml"
            );
        }

        let settings_timeout = config
            .get_typed::<u64>("wallet.settings.auto_lock_timeout_secs")
            .expect("settings timeout must parse")
            .expect("settings timeout must exist");
        let policy_timeout = config
            .get_typed::<u64>("wallet.auto_lock.timeout_secs")
            .expect("auto-lock timeout must parse")
            .expect("auto-lock timeout must exist");

        assert_eq!(
            settings_timeout, policy_timeout,
            "phase-047 requires wallet.settings.auto_lock_timeout_secs and wallet.auto_lock.timeout_secs to share one authoritative value"
        );
    }
}
