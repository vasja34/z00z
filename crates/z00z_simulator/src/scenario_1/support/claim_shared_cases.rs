use std::{
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock},
};

use crate::config::ScenarioCfg;
use z00z_utils::{
    codec::{Codec, YamlCodec},
    io::{remove_dir_all, write_file},
};
use z00z_wallets::claim::registry as claim_registry;

use super::{fixture_cache, stage_runner_support};

fn claim_build_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

fn make_cfg_in(
    base: &Path,
    edit_cfg: impl FnOnce(&mut ScenarioCfg),
) -> (PathBuf, PathBuf, PathBuf) {
    let out = base.join("outputs/scenario_1");
    let mut cfg = ScenarioCfg::from_file(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/scenario_1/scenario_config.yaml"),
    )
    .expect("load scenario config");
    cfg.stage1_genesis
        .get_or_insert_with(Default::default)
        .genesis_config = z00z_core::config_paths::devnet_genesis_path()
        .to_string_lossy()
        .to_string();
    cfg.outputs.dir = out.to_string_lossy().to_string();
    cfg.stage3_claim.as_mut().expect("stage3 cfg").consume_bins = Some(false);
    edit_cfg(&mut cfg);

    let cfg_path = base.join("scenario_config.yaml");
    let cfg_bytes = YamlCodec.serialize(&cfg).expect("serialize cfg");
    write_file(&cfg_path, &cfg_bytes).expect("write cfg");

    let design_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/scenario_1/scenario_design.yaml");
    (cfg_path, design_path, out)
}

fn build_out(
    case_name: &str,
    stage_ids: &[u32],
    edit_cfg: impl FnOnce(&mut ScenarioCfg),
    validate_out: impl FnOnce(&Path),
) -> PathBuf {
    let root = fixture_cache::ensure_shared_case_precise(case_name, |base| {
        let _guard = claim_build_lock()
            .lock()
            .unwrap_or_else(|err| err.into_inner());
        claim_registry::clear_rows();
        let (cfg_path, design_path, out) = make_cfg_in(base, edit_cfg);
        let _ctx = stage_runner_support::run_stage_setup(&cfg_path, &design_path, stage_ids);
        validate_out(&out);
    });
    root.join("outputs/scenario_1")
}

pub fn default_stage3_out() -> PathBuf {
    static OUT: OnceLock<PathBuf> = OnceLock::new();
    OUT.get_or_init(|| {
        build_out(
            "claim_stage3_default_shared_v1",
            &[1_u32, 2, 3],
            |_| {},
            |out| {
                assert!(
                    out.join("stage_3_snapshot.json").exists(),
                    "claim stage3 shared case missing stage_3_snapshot.json"
                );
            },
        )
    })
    .clone()
}

pub fn consume_stage3_out() -> PathBuf {
    static OUT: OnceLock<PathBuf> = OnceLock::new();
    OUT.get_or_init(|| {
        build_out(
            "claim_stage3_consume_shared_v1",
            &[1_u32, 2, 3],
            |cfg| {
                cfg.stage3_claim.as_mut().expect("stage3 cfg").consume_bins = Some(true);
            },
            |out| {
                assert!(
                    out.join("stage_3_snapshot.json").exists(),
                    "claim consume shared case missing stage_3_snapshot.json"
                );
            },
        )
    })
    .clone()
}

pub fn class_split_stage3_out() -> PathBuf {
    static OUT: OnceLock<PathBuf> = OnceLock::new();
    OUT.get_or_init(|| {
        build_out(
            "claim_stage3_class_split_shared_v1",
            &[1_u32, 2, 3],
            |cfg| {
                let stage3 = cfg.stage3_claim.as_mut().expect("stage3 cfg");
                stage3.active = Some("class_split".to_string());
                stage3.consume_bins = Some(false);
            },
            |out| {
                assert!(
                    out.join("stage_3_snapshot.json").exists(),
                    "claim class-split shared case missing stage_3_snapshot.json"
                );
            },
        )
    })
    .clone()
}

pub fn reject_first_stage3_out() -> PathBuf {
    static OUT: OnceLock<PathBuf> = OnceLock::new();
    OUT.get_or_init(|| {
        build_out(
            "claim_stage3_reject_first_shared_v1",
            &[1_u32, 2, 3],
            |cfg| {
                cfg.stage3_claim.as_mut().expect("stage3 cfg").resume_fault =
                    Some("reject_first".to_string());
            },
            |out| {
                assert!(
                    out.join("claim/audit_log.json").exists(),
                    "claim reject-first shared case missing claim audit log"
                );
            },
        )
    })
    .clone()
}

pub fn deterministic_stage3_a_out() -> PathBuf {
    build_deterministic_stage3_out("claim_stage3_deterministic_a_shared_v2")
}

pub fn deterministic_stage3_b_out() -> PathBuf {
    build_deterministic_stage3_out("claim_stage3_deterministic_b_shared_v2")
}

fn build_deterministic_stage3_out(case_name: &str) -> PathBuf {
    build_out(
        case_name,
        &[1_u32, 2, 3],
        |cfg| {
            cfg.stage3_claim.as_mut().expect("stage3 cfg").rng_seed = Some(7);
        },
        |out| {
            assert!(
                out.join("stage_3_snapshot.json").exists(),
                "claim deterministic shared case missing stage_3_snapshot.json"
            );
        },
    )
}

pub fn publish_paths_stage4_out() -> PathBuf {
    static OUT: OnceLock<PathBuf> = OnceLock::new();
    OUT.get_or_init(|| {
        build_out(
            "claim_publish_paths_stage4_shared_v1",
            &[1_u32, 2, 3, 4],
            |cfg| {
                let stage3 = cfg.stage3_claim.as_mut().expect("stage3 cfg");
                stage3.paths.claim_dir = "claim_custom".to_string();
                stage3.paths.snapshot_file = "stage_3_custom_snapshot.json".to_string();
            },
            |out| {
                assert!(
                    out.join("stage_3_custom_snapshot.json").exists(),
                    "claim publish-paths shared case missing custom snapshot"
                );
                assert!(
                    out.join("claim_custom/tx_claim_pkg.json").exists(),
                    "claim publish-paths shared case missing custom claim bundle"
                );
            },
        )
    })
    .clone()
}

pub fn persist_stage6_out() -> PathBuf {
    stage6_out("default")
}

pub fn stage6_out(case_suffix: &str) -> PathBuf {
    let suffix = sanitize_case_suffix(case_suffix);
    let shared_root = persist_stage6_shared_root();
    // The fixture cache already isolates local cases by process scope. Keep
    // the case name stable inside that scope and self-heal stale local copies
    // if a previous interrupted run left an incomplete tree behind.
    let local_case = format!("claim_stage6_persist_local_{}_v2", suffix);
    let local_root = ensure_local_persist_copy(&local_case, &shared_root);
    local_root.join("outputs/scenario_1")
}

fn persist_stage6_shared_root() -> PathBuf {
    static OUT: OnceLock<PathBuf> = OnceLock::new();
    OUT.get_or_init(|| {
        fixture_cache::ensure_shared_case_precise("claim_stage6_persist_shared_v1", |base| {
            let _guard = claim_build_lock()
                .lock()
                .unwrap_or_else(|err| err.into_inner());
            claim_registry::clear_rows();
            let (cfg_path, design_path, out) = make_cfg_in(base, |_| {});
            let _ctx = stage_runner_support::run_stage_setup(
                &cfg_path,
                &design_path,
                &[1_u32, 2, 3, 4, 5, 6],
            );
            assert!(
                persist_stage6_outputs_ready(&out),
                "claim persist shared case missing one or more required stage6 outputs"
            );
        })
    })
    .clone()
}

fn sanitize_case_suffix(case_suffix: &str) -> String {
    let trimmed = case_suffix.trim();
    assert!(
        !trimmed.is_empty(),
        "claim persist localized case suffix must not be empty"
    );
    let sanitized: String = trimmed
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '_' })
        .collect();
    assert!(
        sanitized.chars().any(|ch| ch.is_ascii_alphanumeric()),
        "claim persist localized case suffix must contain alphanumeric characters"
    );
    sanitized
}

fn ensure_local_persist_copy(local_case: &str, shared_root: &Path) -> PathBuf {
    let local_root = fixture_cache::ensure_case(local_case, |base| {
        fixture_cache::copy_tree(shared_root, base);
    });
    if persist_local_copy_ready(&local_root) {
        return local_root;
    }

    remove_dir_all(&local_root).expect("remove incomplete local claim persist case");
    let rebuilt = fixture_cache::ensure_case(local_case, |base| {
        fixture_cache::copy_tree(shared_root, base);
    });
    assert!(
        persist_local_copy_ready(&rebuilt),
        "localized claim persist fixture missing claim_source_store.redb after rebuild"
    );
    rebuilt
}

fn persist_local_copy_ready(local_root: &Path) -> bool {
    persist_stage6_outputs_ready(&local_root.join("outputs/scenario_1"))
}

fn persist_stage6_outputs_ready(out: &Path) -> bool {
    out.join("stage_3_snapshot.json").exists()
        && out.join("stage_4_snapshot.json").exists()
        && out.join("claim_publish/audit_log.json").exists()
        && out.join("claim/claim_source_store.redb").exists()
}
