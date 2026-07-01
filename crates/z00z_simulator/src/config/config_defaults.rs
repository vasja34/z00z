use super::{
    Stage1PathsCfg, Stage2PathsCfg, Stage3PathsCfg, Stage4FeeSinkCfg, Stage4OutputsCfg,
    Stage4SelectionCfg, Stage5PathsCfg, Stage6PathsCfg, Stage7PathsCfg, Stage8PathsCfg,
};

impl Default for Stage1PathsCfg {
    fn default() -> Self {
        Self {
            genesis_dir: "genesis".to_string(),
            logs_dir: "logs".to_string(),
            snapshot_file: "stage_1_snapshot.json".to_string(),
            state_hash_file: "genesis_state_hash.txt".to_string(),
            logger_file: "logger.json".to_string(),
            fallback_genesis_dir: format!(
                "crates/z00z_core/{}",
                z00z_core::config_paths::CORE_CONFIG_DIR
            ),
            cli_target_dir: "_cargo_target_stage1_cli".to_string(),
        }
    }
}

impl Default for Stage2PathsCfg {
    fn default() -> Self {
        Self {
            wallets_dir: "wallets".to_string(),
            keys_dir: "keys".to_string(),
            logs_dir: "logs".to_string(),
            snapshot_file: "stage_2_snapshot.json".to_string(),
            logger_file: "logger.json".to_string(),
            rpc_logger_file: "rpc_logger.json".to_string(),
        }
    }
}

impl Default for Stage3PathsCfg {
    fn default() -> Self {
        Self {
            genesis_dir: "genesis".to_string(),
            claim_dir: "claim".to_string(),
            wallets_dir: "wallets".to_string(),
            events_dir: "events".to_string(),
            logs_dir: "logs".to_string(),
            export_dir: "wallets_export_import".to_string(),
            snapshot_file: "stage_3_snapshot.json".to_string(),
            claim_state_file: "claim_state.json".to_string(),
            logger_file: "logger.json".to_string(),
            rpc_logger_file: "rpc_logger.json".to_string(),
        }
    }
}

impl Default for Stage5PathsCfg {
    fn default() -> Self {
        Self {
            logs_dir: "logs".to_string(),
            transactions_dir: "transactions".to_string(),
            tx_file: "leaf_alice_to_bob.json".to_string(),
            snapshot_file: "stage_5_snapshot.json".to_string(),
            logger_file: "logger.json".to_string(),
        }
    }
}

impl Default for Stage6PathsCfg {
    fn default() -> Self {
        Self {
            logs_dir: "logs".to_string(),
            transactions_dir: "transactions".to_string(),
            frag1_file: "leaf_alice_to_charlie_frag1.json".to_string(),
            frag2_file: "leaf_alice_to_charlie_frag2.json".to_string(),
            checkpoint_file: "checkpoint_bridge_s6.json".to_string(),
            report_file: "report.md".to_string(),
            logger_file: "logger.json".to_string(),
        }
    }
}

impl Default for Stage7PathsCfg {
    fn default() -> Self {
        Self {
            logs_dir: "logs".to_string(),
            transactions_dir: "transactions".to_string(),
            checkpoint_file: "checkpoint_s7.json".to_string(),
            logger_file: "logger.json".to_string(),
        }
    }
}

impl Default for Stage8PathsCfg {
    fn default() -> Self {
        Self {
            logs_dir: "logs".to_string(),
            transactions_dir: "transactions".to_string(),
            checkpoint_file: "checkpoint_s8.json".to_string(),
            logger_file: "logger.json".to_string(),
        }
    }
}

impl Default for Stage4SelectionCfg {
    fn default() -> Self {
        Self {
            distinct_serial_ids_min: 3,
            distinct_serial_ids_target: 3,
            distinct_serial_ids_max: 10,
        }
    }
}

pub(super) fn default_bob_outputs_count() -> u32 {
    1
}

impl Default for Stage4OutputsCfg {
    fn default() -> Self {
        Self {
            bob_outputs_count: default_bob_outputs_count(),
        }
    }
}

pub(super) fn default_fee_wallet_id() -> String {
    "sequencer".to_string()
}

impl Default for Stage4FeeSinkCfg {
    fn default() -> Self {
        Self {
            wallet_id: default_fee_wallet_id(),
            receiver_card_hex: None,
            password: None,
            rng_seed: None,
        }
    }
}

pub(super) fn default_build_transaction_method() -> String {
    "wallet.tx.build_transaction".to_string()
}

pub(super) fn stage5_recipient_output_index() -> usize {
    0
}
