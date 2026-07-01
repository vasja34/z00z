#[cfg(target_os = "linux")]
#[used]
#[link_section = ".init_array"]
static SCENARIO_1_TEST_THREADS_INIT: extern "C" fn() = {
    extern "C" fn init() {
        if std::env::var_os("RUST_TEST_THREADS").is_none() {
            std::env::set_var("RUST_TEST_THREADS", "1");
        }
    }

    init
};

mod claim_pkg_crypto;
mod output_roots;
mod stage4_bob;
mod stage4_paths;
mod stage4_root;
mod test_architecture_boundaries;
mod test_checkpoint_acceptance;
mod test_claim_acceptance;
mod test_claim_audit_log_integrity;
mod test_claim_conservation;
mod test_claim_crypto;
mod test_claim_emit;
mod test_claim_integration;
mod test_claim_persist;
mod test_claim_pkg_runtime;
mod test_claim_post;
mod test_claim_resume;
mod test_claim_snapshot;
mod test_claim_tx_pipeline;
mod test_e2e_stage4;
mod test_fixture_cache_contract;
mod test_genesis_integration;
mod test_genesis_unit;
mod test_hjmt_e2e;
mod test_hjmt_runtime_config;
mod test_pipeline_genesis_tx;
mod test_s7_examples;
mod test_scenario1_filtered_runs;
mod test_scenario1_object_flows;
mod test_scenario1_spend_gate;
mod test_scenario1_stage_surface;
mod test_scenario1_tx_proof_roundtrip;
mod test_scenario1_unified_gate;
mod test_scenario_settlement;
mod test_stage2_secret_artifacts;
mod test_stage3_nullifier_store;
mod test_stage4_bob_flow;
mod test_stage4_card_gate;
mod test_stage4_cfg_guards;
mod test_stage4_cfg_paths;
mod test_stage4_chain_path;
mod test_stage4_claim_gate;
mod test_stage4_digest;
mod test_stage4_gates;
mod test_stage4_output_crypto;
mod test_stage4_root_support;
mod test_stage4_selection;
mod test_stage4_source_shape;
mod test_stage4_split;
mod test_stage4_tamper;
mod test_stage4_wallet_persist;
mod test_stage5_receive_bridge;
mod test_stage5_source_shape;
mod test_stage6_checkpoint;
mod test_stage6_checkpoint_final_gate;
mod test_stage6_checkpoint_storage_bridge;
mod test_stage7_jmt_wallet_scan;
mod test_stage8_proof_path;
mod test_transport_rng_boundaries;
mod test_tx_handoff_integration;
mod test_wallet_claim_replay;
mod test_wallet_integration;
mod test_wallet_unit;
mod test_workspace_target_dir;
