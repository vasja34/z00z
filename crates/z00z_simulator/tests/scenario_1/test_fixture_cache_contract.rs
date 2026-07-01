use z00z_simulator::scenario_1::support::fixture_cache;
use z00z_simulator::scenario_1::support::stage_runner_support;

use std::fs::{remove_file, OpenOptions};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Mutex, MutexGuard,
};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use z00z_utils::io::{create_dir_all, read_to_string, remove_dir_all, write_file};

const FIXTURE_CACHE_SRC: &str = include_str!("../../src/scenario_1/support/fixture_cache.rs");
const RUNNER_SRC: &str = include_str!("../../src/scenario_1/runner.rs");
const SCENARIO_SUPPORT_SRC: &str = include_str!("../../src/scenario_1/support/scenario_support.rs");

struct ContractLockGuard {
    lock_path: PathBuf,
    _process_guard: stage_runner_support::ProcessLock,
    _thread_guard: MutexGuard<'static, ()>,
}

impl Drop for ContractLockGuard {
    fn drop(&mut self) {
        let _ = remove_file(&self.lock_path);
    }
}

fn contract_lock_path() -> PathBuf {
    fixture_cache::scenario_cache_root().join(".fixture_cache_contract.lock")
}

fn clear_dead_contract_lock(lock_path: &Path) -> bool {
    let Ok(owner) = read_to_string(lock_path) else {
        return false;
    };
    let Ok(pid) = owner.trim().parse::<u32>() else {
        return false;
    };
    let proc_path = PathBuf::from(format!("/proc/{pid}"));
    if proc_path.exists() {
        return false;
    }
    remove_file(lock_path).is_ok()
}

fn contract_lock() -> ContractLockGuard {
    static LOCK: Mutex<()> = Mutex::new(());
    let process_guard = stage_runner_support::acquire_process_lock();
    let thread_guard = LOCK.lock().expect("fixture cache contract lock");
    let lock_path = contract_lock_path();
    let lock_dir = lock_path.parent().expect("contract lock dir");
    create_dir_all(lock_dir).expect("create contract lock dir");

    loop {
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&lock_path)
        {
            Ok(_) => {
                write_file(&lock_path, std::process::id().to_string().as_bytes())
                    .expect("write contract lock owner");
                return ContractLockGuard {
                    lock_path,
                    _process_guard: process_guard,
                    _thread_guard: thread_guard,
                };
            }
            Err(err) if err.kind() == ErrorKind::AlreadyExists => {
                if clear_dead_contract_lock(&lock_path) {
                    continue;
                }
                thread::sleep(Duration::from_millis(50));
            }
            Err(err) => panic!("open contract lock {} failed: {err}", lock_path.display()),
        }
    }
}

fn unique_temp_root(label: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_micros();
    std::env::temp_dir().join(format!(
        "z00z_fixture_cache_contract_{label}_{}_{}",
        std::process::id(),
        nonce
    ))
}

fn sandbox_cache_root(
    label: &str,
) -> (
    PathBuf,
    z00z_simulator::scenario_1::support::fixture_cache::CacheRootOverrideGuard,
) {
    let sandbox_root = unique_temp_root(label);
    let cache_root = sandbox_root.join("scenario_1");
    if sandbox_root.exists() {
        remove_dir_all(&sandbox_root).expect("clear stale sandbox root");
    }
    let guard = fixture_cache::override_cache_root_for_thread(&cache_root);
    (sandbox_root, guard)
}

#[test]
fn test_cache_fingerprint_scope() {
    let _guard = contract_lock();
    for needle in [
        "const FINGERPRINT_SCHEMA: &str = \"fixture-cache-fingerprint-v2\";",
        "\"Cargo.toml\"",
        "\"Cargo.lock\"",
        "\".cargo/config.toml\"",
        "\"crates/z00z_utils/src\"",
        "\"crates/z00z_crypto/src\"",
        "\"crates/z00z_networks/rpc/src\"",
        "stable_current_exe_scope(\"unknown_test_binary\")",
    ] {
        assert!(
            FIXTURE_CACHE_SRC.contains(needle),
            "fixture cache fingerprint contract must include {needle}"
        );
    }
}

#[test]
fn test_cache_fingerprint_modes() {
    let _guard = contract_lock();
    for needle in [
        "hasher.update(b\"scope=local\\n\");",
        "hasher.update(b\"scope=shared\\n\");",
        "hasher.update(b\"scope=shared-precise\\n\");",
        "hash_file(&mut hasher, \"current_exe\", &exe_path);",
        "const CONTENT_FINGERPRINT_FILE: &str = \".content-fingerprint\";",
        "const CONTENT_FINGERPRINT_SCHEMA: &str = \"fixture-cache-content-v1\";",
        "cache_content_fingerprint(cache_dir)",
    ] {
        assert!(
            FIXTURE_CACHE_SRC.contains(needle),
            "fixture cache fingerprint mode contract must include {needle}"
        );
    }
}

#[test]
fn test_precise_scope_contract() {
    let _guard = contract_lock();
    for needle in [
        "const SHARED_FINGERPRINT_CORE_TREES: &[&str] = &[",
        "const SHARED_FINGERPRINT_TEST_TREES: &[&str] = &[\"crates/z00z_simulator/tests\"];",
        "const SHARED_PRECISE_SCOPE_DIR: &str = \"shared_precise\";",
        "pub fn ensure_shared_case_precise(case_name: &str, build: impl FnOnce(&Path)) -> PathBuf {",
        "shared_precise_cache_root(case_name),",
        "CacheScope::SharedPrecise => shared_precise_case_fingerprint(),",
        "for rel in SHARED_FINGERPRINT_TEST_TREES {",
    ] {
        assert!(
            FIXTURE_CACHE_SRC.contains(needle),
            "fixture cache precise-scope contract must include {needle}"
        );
    }
}

#[test]
fn test_simulator_cache_root_contract() {
    let _guard = contract_lock();
    for src in [FIXTURE_CACHE_SRC, RUNNER_SRC, SCENARIO_SUPPORT_SRC] {
        assert!(
            src.contains("join(\".cache\")"),
            "simulator cache contract must preserve repo/.cache as the non-verifier fallback"
        );
        assert!(
            src.contains("current_exe_run_root"),
            "simulator cache contract must recognize verifier-owned run roots before fallback"
        );
        assert!(
            !src.contains("join(\"workdir\").join(\"cache\")"),
            "verification run-root cache contract must use run-root/cache"
        );
    }
}

#[test]
fn test_cache_rebuilds_stale() {
    let _guard = contract_lock();
    let (sandbox_root, _cache_root_guard) = sandbox_cache_root("cache_rebuilds_stale");
    let case_name = "fixture_cache_contract_rebuilds";
    let cache_dir = fixture_cache::cache_root(case_name);
    create_dir_all(&cache_dir).expect("create stale contract cache");
    write_file(cache_dir.join(".ready"), b"").expect("write stale ready");
    write_file(cache_dir.join(".fingerprint"), b"bogus-fingerprint")
        .expect("write stale fingerprint");
    write_file(cache_dir.join("marker.txt"), b"stale").expect("write stale marker");

    let out = fixture_cache::ensure_case(case_name, |base| {
        write_file(base.join(".fingerprint"), b"builder-should-not-win").expect("tmp fingerprint");
        write_file(base.join("marker.txt"), b"fresh").expect("fresh marker");
    });

    assert_eq!(
        read_to_string(out.join("marker.txt")).expect("read rebuilt marker"),
        "fresh"
    );
    assert_ne!(
        read_to_string(out.join(".fingerprint")).expect("read final fingerprint"),
        "bogus-fingerprint"
    );
    if sandbox_root.exists() {
        remove_dir_all(&sandbox_root).expect("cleanup cache rebuild sandbox");
    }
}

#[test]
fn test_cache_clears_stale_scope() {
    let _guard = contract_lock();
    let (sandbox_root, _cache_root_guard) = sandbox_cache_root("cache_clears_stale_scope");
    let keep_case = "fixture_cache_contract_scope_keep";
    let stale_case = "fixture_cache_contract_scope_old";
    let keep_dir = fixture_cache::cache_root(keep_case);
    let scope_dir = keep_dir.parent().expect("scope dir").to_path_buf();

    let stale_dir = scope_dir.join(stale_case);
    create_dir_all(&stale_dir).expect("create stale case dir");
    write_file(scope_dir.join(".scope-fingerprint"), b"stale-scope").expect("write scope mark");
    write_file(stale_dir.join(".ready"), b"").expect("write stale ready");
    write_file(stale_dir.join(".fingerprint"), b"stale-case").expect("write stale fingerprint");
    write_file(stale_dir.join("marker.txt"), b"old").expect("write stale marker");

    let out = fixture_cache::ensure_case(keep_case, |base| {
        write_file(base.join("marker.txt"), b"fresh").expect("write fresh marker");
    });

    assert_eq!(
        read_to_string(out.join("marker.txt")).expect("read fresh marker"),
        "fresh"
    );
    assert!(
        !stale_dir.exists(),
        "scope drift must clear stale sibling cases under {}",
        scope_dir.display()
    );
    if sandbox_root.exists() {
        remove_dir_all(&sandbox_root).expect("cleanup stale scope sandbox");
    }
}

#[test]
fn test_hash_suffixed_scope_aliases() {
    let _guard = contract_lock();
    let (sandbox_root, _cache_root_guard) = sandbox_cache_root("hash_suffixed_scope_aliases");
    let keep_case = "fixture_cache_contract_scope_alias_keep";
    let keep_dir = fixture_cache::cache_root(keep_case);
    let scope_dir = keep_dir.parent().expect("scope dir").to_path_buf();
    let scope_root = scope_dir.parent().expect("scope root").to_path_buf();
    let scope_name = scope_dir
        .file_name()
        .and_then(|name| name.to_str())
        .expect("scope name");
    let alias_dir = scope_root.join(format!("{scope_name}-9d4bd19d594c355f"));

    create_dir_all(alias_dir.join(keep_case)).expect("create stale alias case dir");
    write_file(alias_dir.join(".scope-fingerprint"), b"stale-scope").expect("write alias scope");
    write_file(alias_dir.join(keep_case).join(".ready"), b"").expect("write alias ready");
    write_file(
        alias_dir.join(keep_case).join(".fingerprint"),
        b"stale-fingerprint",
    )
    .expect("write alias case fingerprint");
    write_file(alias_dir.join(keep_case).join("marker.txt"), b"old").expect("write alias marker");

    let out = fixture_cache::ensure_case(keep_case, |base| {
        write_file(base.join("marker.txt"), b"fresh").expect("write fresh alias marker");
    });

    assert_eq!(
        read_to_string(out.join("marker.txt")).expect("read fresh alias marker"),
        "fresh"
    );
    assert!(
        !alias_dir.exists(),
        "scope rebuild must drop stale hash-suffixed alias scope under {}",
        scope_root.display()
    );
    if sandbox_root.exists() {
        remove_dir_all(&sandbox_root).expect("cleanup alias scope sandbox");
    }
}

#[test]
fn test_cache_reuses_match() {
    let _guard = contract_lock();
    let (sandbox_root, _cache_root_guard) = sandbox_cache_root("cache_reuses_match");
    let case_name = "fixture_cache_contract_reuse";

    static BUILDS: AtomicUsize = AtomicUsize::new(0);
    BUILDS.store(0, Ordering::SeqCst);

    let first = fixture_cache::ensure_case(case_name, |base| {
        BUILDS.fetch_add(1, Ordering::SeqCst);
        write_file(base.join("marker.txt"), b"fresh").expect("write marker");
    });
    let second = fixture_cache::ensure_case(case_name, |base| {
        BUILDS.fetch_add(1, Ordering::SeqCst);
        write_file(base.join("marker.txt"), b"should-not-rebuild").expect("write marker");
    });

    assert_eq!(first, second);
    assert_eq!(BUILDS.load(Ordering::SeqCst), 1);
    assert_eq!(
        read_to_string(second.join("marker.txt")).expect("read reused marker"),
        "fresh"
    );
    if sandbox_root.exists() {
        remove_dir_all(&sandbox_root).expect("cleanup reuse sandbox");
    }
}

#[test]
fn test_tampered_scope_rebuilds_cache() {
    let _guard = contract_lock();
    let (sandbox_root, _cache_root_guard) = sandbox_cache_root("tampered_scope_rebuilds");
    let case_name = "fixture_cache_contract_tamper_rebuild";

    static BUILDS: AtomicUsize = AtomicUsize::new(0);
    BUILDS.store(0, Ordering::SeqCst);

    let first = fixture_cache::ensure_case(case_name, |base| {
        BUILDS.fetch_add(1, Ordering::SeqCst);
        write_file(base.join("marker.txt"), b"fresh").expect("write fresh marker");
    });
    write_file(first.join("marker.txt"), b"tampered").expect("tamper cached payload");

    let second = fixture_cache::ensure_case(case_name, |base| {
        BUILDS.fetch_add(1, Ordering::SeqCst);
        write_file(base.join("marker.txt"), b"rebuilt").expect("write rebuilt marker");
    });

    assert_eq!(first, second);
    assert_eq!(BUILDS.load(Ordering::SeqCst), 2);
    assert_eq!(
        read_to_string(second.join("marker.txt")).expect("read rebuilt marker"),
        "rebuilt"
    );
    if sandbox_root.exists() {
        remove_dir_all(&sandbox_root).expect("cleanup tamper rebuild sandbox");
    }
}

#[test]
fn test_clears_stale_tmp_dirs() {
    let _guard = contract_lock();
    let (sandbox_root, _cache_root_guard) = sandbox_cache_root("clears_stale_tmp_dirs");
    let case_name = "fixture_cache_contract_tmp_cleanup";
    let cache_dir = fixture_cache::cache_root(case_name);
    let scope_dir = cache_dir.parent().expect("scope dir").to_path_buf();

    let stale_tmp = scope_dir.join(format!(".{case_name}.tmp.424242"));
    create_dir_all(stale_tmp.join("nested")).expect("create stale tmp dir");
    write_file(stale_tmp.join("nested/marker.txt"), b"stale").expect("write stale tmp marker");

    let out = fixture_cache::ensure_case(case_name, |base| {
        write_file(base.join("marker.txt"), b"fresh").expect("write fresh tmp cleanup marker");
    });

    assert!(
        !stale_tmp.exists(),
        "ensure_case must remove stale tmp dirs under {}",
        scope_dir.display()
    );
    assert_eq!(
        read_to_string(out.join("marker.txt")).expect("read fresh tmp cleanup marker"),
        "fresh"
    );
    if sandbox_root.exists() {
        remove_dir_all(&sandbox_root).expect("cleanup tmp cleanup sandbox");
    }
}

#[test]
fn test_precise_scope_keeps_shared() {
    let _guard = contract_lock();
    let (sandbox_root, _cache_root_guard) = sandbox_cache_root("precise_scope_keeps_shared");
    let shared_case = "fixture_cache_contract_shared_mode";
    let precise_case = "fixture_cache_contract_shared_precise_mode";
    let shared_dir = fixture_cache::shared_cache_root(shared_case);
    let precise_dir = fixture_cache::shared_precise_cache_root(precise_case);
    let shared_scope = shared_dir.parent().expect("shared scope");
    let precise_scope = precise_dir.parent().expect("precise scope");

    if shared_dir.exists() {
        remove_dir_all(&shared_dir).expect("clear shared case");
    }
    if precise_dir.exists() {
        remove_dir_all(&precise_dir).expect("clear precise case");
    }

    fixture_cache::ensure_shared_case(shared_case, |base| {
        write_file(base.join("marker.txt"), b"shared").expect("write shared marker");
    });
    fixture_cache::ensure_shared_case_precise(precise_case, |base| {
        write_file(base.join("marker.txt"), b"precise").expect("write precise marker");
    });

    assert_eq!(
        read_to_string(shared_dir.join("marker.txt")).expect("read shared marker"),
        "shared"
    );
    assert_eq!(
        read_to_string(precise_dir.join("marker.txt")).expect("read precise marker"),
        "precise"
    );
    assert_ne!(
        shared_scope, precise_scope,
        "shared and shared_precise cache scopes must stay physically isolated"
    );
    if sandbox_root.exists() {
        remove_dir_all(&sandbox_root).expect("cleanup shared precise sandbox");
    }
}

#[test]
fn test_precise_scope_prunes_stale() {
    let _guard = contract_lock();
    let (sandbox_root, _cache_root_guard) = sandbox_cache_root("verifier_shared_scope_reset");
    let keep_case = "fixture_cache_contract_verifier_scope_keep";
    let stale_case = "fixture_cache_contract_verifier_scope_old";
    let keep_dir = fixture_cache::shared_precise_cache_root(keep_case);
    let scope_dir = keep_dir
        .parent()
        .expect("shared precise scope")
        .to_path_buf();
    let stale_dir = scope_dir.join(stale_case);

    create_dir_all(&stale_dir).expect("create stale verifier-owned case dir");
    write_file(scope_dir.join(".scope-fingerprint"), b"stale-scope")
        .expect("write stale verifier-owned scope mark");
    write_file(stale_dir.join(".ready"), b"").expect("write stale verifier-owned ready");
    write_file(stale_dir.join(".fingerprint"), b"stale-case")
        .expect("write stale verifier-owned fingerprint");
    write_file(stale_dir.join("marker.txt"), b"old").expect("write stale verifier-owned marker");

    let out = fixture_cache::ensure_shared_case_precise(keep_case, |base| {
        write_file(base.join("marker.txt"), b"fresh").expect("write fresh verifier-owned marker");
    });

    assert_eq!(
        read_to_string(out.join("marker.txt")).expect("read fresh verifier-owned marker"),
        "fresh"
    );
    assert!(
        !stale_dir.exists(),
        "verifier-owned shared precise scope drift must clear stale siblings under {}",
        scope_dir.display()
    );

    if sandbox_root.exists() {
        remove_dir_all(&sandbox_root).expect("cleanup verifier-owned cache sandbox");
    }
}
