---
phase: 030-refactor-long-files
plan: 13
subsystem: database
tags: [rust, wallets, redb, facade, storage, crypto, migration, verification]
requires:
  - phase: 030-01
    provides: initial wallet DB seam split and stable shallow `crate::db` facade baseline
provides:
  - thin wallet DB facade over focused RedB persistence seams
  - split RedB crypto, migration, object, discovery, and debug support modules under `crate::db`
  - preserved shallow wallet DB contract with no deep-import callers outside `src/db`
affects: [z00z_wallets, crate::db, planning]
tech-stack:
  added: []
  patterns: [thin facade plus sibling seam modules, extracted test support modules, all-features verification rerun after transient gate failure]
key-files:
  created:
    - crates/z00z_wallets/src/db/index_codecs_tests.rs
    - crates/z00z_wallets/src/db/index_codecs_tx_time.rs
    - crates/z00z_wallets/src/db/redb_wallet_crypto_aad.rs
    - crates/z00z_wallets/src/db/redb_wallet_crypto_kdf_helpers.rs
    - crates/z00z_wallets/src/db/redb_wallet_crypto_models.rs
    - crates/z00z_wallets/src/db/redb_wallet_crypto_tests.rs
    - crates/z00z_wallets/src/db/redb_wallet_store_create.rs
    - crates/z00z_wallets/src/db/redb_wallet_store_crypto_ops_seed.rs
    - crates/z00z_wallets/src/db/redb_wallet_store_debug.rs
    - crates/z00z_wallets/src/db/redb_wallet_store_debug_export.rs
    - crates/z00z_wallets/src/db/redb_wallet_store_debug_types.rs
    - crates/z00z_wallets/src/db/redb_wallet_store_discovery.rs
    - crates/z00z_wallets/src/db/redb_wallet_store_initial_objects.rs
    - crates/z00z_wallets/src/db/redb_wallet_store_meta.rs
    - crates/z00z_wallets/src/db/redb_wallet_store_migrations_tables.rs
    - crates/z00z_wallets/src/db/redb_wallet_store_mutations.rs
    - crates/z00z_wallets/src/db/redb_wallet_store_objects_test_support.rs
    - crates/z00z_wallets/src/db/redb_wallet_store_open.rs
    - crates/z00z_wallets/src/db/redb_wallet_store_open_session.rs
    - crates/z00z_wallets/src/db/redb_wallet_store_upserts.rs
    - crates/z00z_wallets/src/db/storage_backend_tests.rs
    - crates/z00z_wallets/src/db/tests/redb_wallet_store.rs
  modified:
    - crates/z00z_wallets/src/db/index_codecs.rs
    - crates/z00z_wallets/src/db/redb_wallet_crypto.rs
    - crates/z00z_wallets/src/db/redb_wallet_store.rs
    - crates/z00z_wallets/src/db/redb_wallet_store_crypto_ops.rs
    - crates/z00z_wallets/src/db/redb_wallet_store_migrations.rs
    - crates/z00z_wallets/src/db/redb_wallet_store_objects.rs
    - crates/z00z_wallets/src/db/storage_backend.rs
    - reports/full_verify-report-long-running-tests.txt
key-decisions:
  - Keep `redb_wallet_store.rs` as the stable caller-visible wallet DB facade while moving debug, meta, open, create, and mutation responsibilities into sibling seam files.
  - Keep `crate::db` as the only shallow caller surface and enforce it with a deep-import guard rather than compatibility re-exports from deep modules.
  - Treat the first failing `max-safe` run after formatting as transient once the exact `--all-features` repro target passed and a fresh `max-safe` rerun returned a clean summary.
patterns-established:
  - "Wallet DB seam split: preserve the facade file, move homogeneous responsibilities into sibling RedB support modules, and keep test support in dedicated test-only modules."
  - "Verification closeout: when `max-safe` fails on one target, reproduce the exact command line before changing logic, then rerun the full gate for closure."
requirements-completed: [PH30-SEAMS, PH30-PROTECTED, PH30-VERIFY]
duration: multi-session
completed: 2026-04-01
---

# Phase 030 Plan 13 Summary

**Wallet DB persistence now routes through a thin `crate::db` facade over focused RedB seam modules for open, create, mutations, crypto, migrations, codecs, and debug export behavior**

## Performance

- **Duration:** multi-session
- **Started:** 2026-04-01T07:46:34Z
- **Completed:** 2026-04-01T10:14:48Z
- **Tasks:** 2
- **Files modified:** 30

## Accomplishments

- Split the oversized wallet-store root into façade-owned submodules for debug export, meta handling, discovery, session open flow, create flow, initial object creation, and mutation helpers while keeping the root caller contract stable.
- Split adjacent DB support files so `index_codecs`, `redb_wallet_crypto`, `redb_wallet_store_crypto_ops`, `redb_wallet_store_migrations`, `redb_wallet_store_objects`, and `storage_backend` each delegate isolated responsibilities to dedicated sibling files.
- Moved the huge wallet-store test body under `crates/z00z_wallets/src/db/tests/redb_wallet_store.rs` and extracted test-only helpers for storage backend, object writing, crypto, and index codec coverage.
- Closed the plan with a clean deep-import guard and a fresh clean `full_verify --max-safe-run` summary of `313 planned, 21 skipped, 0 failed`.

## Task Commits

No git commit was created in this execution pass.

The workspace contains unrelated external changes outside the Phase 030 wallet DB scope, and the repo rules require the owned Z00Z git-versioning workflow instead of ad hoc raw git commits.

## Files Created/Modified

- `crates/z00z_wallets/src/db/redb_wallet_store.rs` now acts as the stable wallet DB orchestration facade with focused sibling modules behind it.
- `crates/z00z_wallets/src/db/redb_wallet_store_open_session.rs` now owns the RedB open path, migration handoff, object validation, and session construction.
- `crates/z00z_wallets/src/db/redb_wallet_store_create.rs` and `redb_wallet_store_initial_objects.rs` now isolate create-time persistence and initial object graph generation.
- `crates/z00z_wallets/src/db/redb_wallet_store_meta.rs` now owns meta read, validation, pointer invariants, and wallet write-seq bumping.
- `crates/z00z_wallets/src/db/redb_wallet_store_debug_export.rs` and `redb_wallet_store_debug_types.rs` now isolate debug export data decoding from the wallet-store root.
- `crates/z00z_wallets/src/db/redb_wallet_crypto_aad.rs`, `redb_wallet_crypto_kdf_helpers.rs`, and `redb_wallet_crypto_models.rs` now isolate AAD framing, KDF normalization, and crypto DTOs from `redb_wallet_crypto.rs`.
- `crates/z00z_wallets/src/db/index_codecs_tx_time.rs` and `index_codecs_tests.rs` now isolate tx-time index encoding and test coverage from the codec root.
- `crates/z00z_wallets/src/db/redb_wallet_store_migrations_tables.rs` now isolates the table clearing/reset logic used by wallet DB migrations.
- `crates/z00z_wallets/src/db/tests/redb_wallet_store.rs` now holds the extracted wallet-store regression suite separately from the production facade file.
- `reports/full_verify-report-long-running-tests.txt` was regenerated by the clean `max-safe` closeout run.

## Decisions Made

- Keep the public wallet persistence surface shallow and stable: callers continue to consume `crate::db`, while deep helper modules remain internal to `src/db`.
- Prefer sibling seam modules over widening visibility through compatibility shims; only the minimum `pub(crate)` or `pub(super)` exposure needed for sibling cooperation was retained.
- Treat the first post-format `max-safe` failure on `test_bench_scan_no_tag16` as a verification problem to reproduce exactly before code changes. The exact `--all-features` reproduction passed, and the next full gate passed cleanly, so no additional product-code change was warranted.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Removed extraction corruption and repaired sibling visibility across the new wallet DB seams**

- **Found during:** Task 1 and Task 2 continuation compile/debug loop
- **Issue:** The second-level seam split introduced corrupted file prefixes, orphaned extraction fragments, and sibling-module visibility/import breakage across the new wallet DB support files.
- **Fix:** Recreated thin facade files cleanly, stripped mechanical extraction artifacts, normalized imports, and adjusted only the required `pub(crate)`/`pub(super)` boundaries for meta headers, initial objects, open helpers, and store helpers.
- **Files modified:** `crates/z00z_wallets/src/db/redb_wallet_store.rs`, `redb_wallet_store_debug.rs`, `redb_wallet_store_debug_export.rs`, `redb_wallet_store_debug_types.rs`, `redb_wallet_store_open.rs`, `redb_wallet_store_open_session.rs`, `redb_wallet_store_mutations.rs`, `redb_wallet_store_create.rs`, `redb_wallet_store_initial_objects.rs`, `redb_wallet_store_meta.rs`, `redb_wallet_store_upserts.rs`
- **Verification:** `cargo check -p z00z_wallets --tests --quiet` passed, targeted wallet tests remained green in the plan run, and the final `max-safe` gate passed cleanly.

**2. [Rule 3 - Blocking] Cleared the `max-safe` rustfmt stop and revalidated the reported failing all-features target before rerunning the workspace gate**

- **Found during:** final Task 2 verification closeout
- **Issue:** The first canonical `full_verify --max-safe-run` stopped immediately on a rustfmt diff in `crates/z00z_wallets/src/db/tests/redb_wallet_store.rs`; the next run later reported a single failing target `z00z_wallets:test:test_bench_scan_no_tag16:test`.
- **Fix:** Ran `cargo fmt` across the changed wallet DB Rust files, verified Codacy on every changed DB file, reproduced `cargo test -p z00z_wallets --release --all-features --test test_bench_scan_no_tag16 -- --test-threads 1 --nocapture` successfully, then reran the canonical `max-safe` gate to a clean summary.
- **Files modified:** wallet DB Rust files touched by formatting, including `crates/z00z_wallets/src/db/tests/redb_wallet_store.rs`
- **Verification:** deep-import guard returned `deep-import-guard: ok`; exact failing-target repro passed; `./.github/skills/z00z-full-verify-gate/scripts/full_verify.sh --max-safe-run` finished with `313 planned, 21 skipped, 0 failed`.

---

**Total deviations:** 2 auto-fixed (1 bug, 1 blocking)
**Impact on plan:** All deviations were necessary to make the seam split buildable and to close the verification gate without widening the wallet DB public surface.

## Issues Encountered

- The extracted wallet DB seam files initially contained mechanical corruption and split-boundary artifacts that had to be cleaned before the real module-boundary compile issues were visible.
- The first post-format `max-safe` run surfaced a transient failure summary for `test_bench_scan_no_tag16`, but the exact all-features command passed on direct reproduction and the subsequent fresh `max-safe` rerun closed cleanly.

## Caller Inventory Audit

```text
if rg -n 'crate::db::redb_wallet_store(::|_)|crate::db::redb_wallet_store_[a-z_]+' crates/z00z_wallets/src -g '*.rs' | grep -v '^crates/z00z_wallets/src/db/'; then
  echo 'Legacy wallet DB deep imports remain outside the shallow crate::db facade'
  exit 1
else
  echo 'deep-import-guard: ok'
fi
```

Result: `deep-import-guard: ok`

Interpretation: no remaining Rust callers outside `src/db` reach into the deep wallet DB seam files directly.

## Known Stubs

None detected in the touched Plan 13 wallet DB seam files.

## User Setup Required

None. This plan changes only internal Rust module ownership, test placement, and verification artifacts.

## Next Phase Readiness

- The wallet DB continuation is now summary-backed and verification-clean for Phase 030 follow-on planning.
- Commit/push remains intentionally deferred until the user chooses the repo-owned git-versioning flow and confirms how to handle unrelated external workspace changes.

## Verification

- `cargo check -p z00z_wallets --tests --quiet`
- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `cargo test -p z00z_wallets --release --test test_redb_wlt_open -- --nocapture`
- `cargo test -p z00z_wallets --release --test test_open_wallet_source_discovery -- --nocapture`
- `cargo test -p z00z_wallets --release --test test_tx_store_integration -- --nocapture`
- `cargo test -p z00z_wallets --release --test test_wlt_validator -- --nocapture`
- `cargo test -p z00z_wallets --release --test test_wallet_kdf_migration -- --nocapture`
- `cargo test --release --features test-fast --features wallet_debug_dump`
- `cargo test -p z00z_wallets --release --all-features --test test_bench_scan_no_tag16 -- --test-threads 1 --nocapture`
- deep-import guard over `crate::db::redb_wallet_store*` callers outside `src/db`
- `./.github/skills/z00z-full-verify-gate/scripts/full_verify.sh --max-safe-run` reached a clean summary of `313 planned, 21 skipped, 0 failed`

## Self-Check

PASSED: `030-13-SUMMARY.md` exists, and the latest `max-safe` report records `313 planned, 21 skipped, 0 failed`.

---
*Phase: 030-refactor-long-files*
*Completed: 2026-04-01*
