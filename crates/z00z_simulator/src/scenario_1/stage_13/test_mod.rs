use std::path::Path;

use crate::config::Stage13HjmtCfg;
use z00z_storage::settlement::{ForestCacheMetrics, ForestSchedulerMetrics};

use super::{
    report::{redaction_violation, Stage13CacheSchedulerReport, STAGE13_LOG_FILE},
    resolve_stage13_path,
    scan::FORBIDDEN_SOURCE_TERMS,
    storage::typed_error,
    validate_contract_cfg_fields,
};

fn base_cfg() -> Stage13HjmtCfg {
    Stage13HjmtCfg {
        enabled: true,
        backend_modes: vec!["generalized".to_string(), "adaptive".to_string()],
        rights_manifest_file: "genesis/genesis_settlement_manifest.json".to_string(),
        output_dir: "hjmt".to_string(),
        examples_file: "hjmt/hjmt_settlement_examples.json".to_string(),
        tamper_report_file: "hjmt/hjmt_tamper_report.json".to_string(),
        proof_size_report_file: "hjmt/hjmt_proof_size_report.json".to_string(),
        cache_scheduler_metrics_file: "hjmt/hjmt_cache_scheduler_metrics.json".to_string(),
        replay_roots_file: "hjmt/hjmt_replay_roots.json".to_string(),
        expected_right_classes: vec![
            "machine_capability".to_string(),
            "data_access".to_string(),
            "service_entitlement".to_string(),
            "validator_mandate".to_string(),
            "one_time_use".to_string(),
        ],
    }
}

#[test]
fn s13_cfg_guard_hjmt_ok() {
    assert!(validate_contract_cfg_fields(&base_cfg()).is_ok());
}

#[test]
fn s13_cfg_guard_rejects() {
    let mut cfg = base_cfg();
    cfg.enabled = false;
    assert!(validate_contract_cfg_fields(&cfg)
        .unwrap_err()
        .contains("must stay enabled"));

    let mut cfg = base_cfg();
    cfg.backend_modes = vec!["generalized".to_string()];
    assert!(validate_contract_cfg_fields(&cfg)
        .unwrap_err()
        .contains("must include adaptive"));

    let mut cfg = base_cfg();
    cfg.expected_right_classes.clear();
    assert!(validate_contract_cfg_fields(&cfg)
        .unwrap_err()
        .contains("expected_right_classes"));

    let mut cfg = base_cfg();
    cfg.examples_file = "   ".to_string();
    assert!(validate_contract_cfg_fields(&cfg)
        .unwrap_err()
        .contains("examples_file must not be empty"));
}

#[test]
fn s13_path_escape_rejects() {
    let base = Path::new("/tmp/stage13");
    assert!(
        resolve_stage13_path(base, "/tmp/escape.json", "report_file")
            .unwrap_err()
            .contains("scenario outputs sandbox")
    );
    assert!(resolve_stage13_path(base, "../escape.json", "report_file")
        .unwrap_err()
        .contains("parent segments"));

    let ok = resolve_stage13_path(base, "hjmt/report.json", "examples_file")
        .expect("relative path stays under outputs dir");
    assert_eq!(ok, base.join("hjmt/report.json"));
}

#[test]
fn s13_logger_path_stays_deterministic() {
    assert_eq!(STAGE13_LOG_FILE, "hjmt/stage13_hjmt_examples.log");
}

#[test]
fn s13_source_rejects_layout_terms() {
    let src = include_str!("hjmt_examples.rs");
    for forbidden in FORBIDDEN_SOURCE_TERMS {
        assert!(
            !src.contains(forbidden),
            "hjmt_examples.rs must not depend on forbidden source-shape term {forbidden}"
        );
    }
}

#[test]
fn s13_error_redacts_secret_hex() {
    let err = typed_error(&String::from(
        "owner_sk witness_bytes deadbeefdeadbeefdeadbeefdeadbeef",
    ));
    assert_eq!(err.class, "String");
    assert_eq!(err.message, "redacted failure details");
    assert!(redaction_violation(&err.class).is_none());
    assert!(redaction_violation(&err.message).is_none());
}

#[test]
fn s13_metrics_report_bounds_ok() {
    let report = Stage13CacheSchedulerReport {
        schema_version: 1,
        scenario_id: 1,
        stage: 13,
        example_id: "E8_cache_scheduler".to_string(),
        backend_mode: "adaptive".to_string(),
        api_surface: "settlement_proof_blobs + forest_cache_metrics + forest_scheduler_metrics"
            .to_string(),
        verifier_status: "verified".to_string(),
        root_generation: 1,
        typed_error: None,
        settlement_state_root_hex: "11".repeat(32),
        cache_hit_count: 5,
        cache_miss_count: 4,
        invalidation_count: 2,
        root_reuse_ratio: 0.5,
        proof_segment_reuse_ratio: 0.5,
        scheduler_queue_depth: 4,
        scheduler_backpressure_count: 0,
        deterministic_parent_ordering: true,
        cache_metrics: ForestCacheMetrics {
            subtree_root: z00z_storage::settlement::CacheLayerMetrics {
                hits: 2,
                misses: 1,
                invalidations: 1,
                evictions: 1,
                ..Default::default()
            },
            parent_leaf: z00z_storage::settlement::CacheLayerMetrics {
                hits: 1,
                misses: 1,
                ..Default::default()
            },
            terminal_leaf: z00z_storage::settlement::CacheLayerMetrics {
                hits: 1,
                misses: 1,
                ..Default::default()
            },
            proof_segment: z00z_storage::settlement::CacheLayerMetrics {
                hits: 1,
                misses: 0,
                invalidations: 1,
                evictions: 1,
                ..Default::default()
            },
            nonexistence: z00z_storage::settlement::CacheLayerMetrics {
                hits: 0,
                misses: 1,
                ..Default::default()
            },
            ..Default::default()
        },
        scheduler_metrics: ForestSchedulerMetrics {
            last_batch: 4,
            last_queued: 4,
            max_queued: 4,
            last_ordered: true,
            ..Default::default()
        },
    };
    assert!(report.validate_bounded().is_ok());
}

#[test]
fn s13_metrics_rejects_ratio_drift() {
    let report = Stage13CacheSchedulerReport {
        schema_version: 1,
        scenario_id: 1,
        stage: 13,
        example_id: "E8_cache_scheduler".to_string(),
        backend_mode: "adaptive".to_string(),
        api_surface: "settlement_proof_blobs + forest_cache_metrics + forest_scheduler_metrics"
            .to_string(),
        verifier_status: "verified".to_string(),
        root_generation: 1,
        typed_error: None,
        settlement_state_root_hex: "22".repeat(32),
        cache_hit_count: 1,
        cache_miss_count: 1,
        invalidation_count: 0,
        root_reuse_ratio: 1.5,
        proof_segment_reuse_ratio: 0.5,
        scheduler_queue_depth: 0,
        scheduler_backpressure_count: 0,
        deterministic_parent_ordering: true,
        cache_metrics: ForestCacheMetrics {
            subtree_root: z00z_storage::settlement::CacheLayerMetrics {
                hits: 1,
                misses: 1,
                ..Default::default()
            },
            ..Default::default()
        },
        scheduler_metrics: ForestSchedulerMetrics::default(),
    };
    assert!(report
        .validate_bounded()
        .expect_err("ratio drift must reject")
        .contains("out of bounds"));
}
