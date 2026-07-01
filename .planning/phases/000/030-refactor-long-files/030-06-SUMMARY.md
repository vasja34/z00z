---
phase: 030-refactor-long-files
plan: 06
subsystem: wallets-key
tags: [rust, wallets, key-management, seed, bip32, facade, seams]
requires:
  - phase: 030-05
    provides: stable include-based facade split workflow and wallet-domain review closure pattern
provides:
  - stable key facade with extracted seed, bip32, key-cache, key-state, and key-manager implementation seams
  - stable wallet facade with extracted chain-id, identity, kernel, record, and entity seams
  - structural regression guards plus release-style verification for the wallet key and identity split
affects: [030-07, 030-08, 030-09, 030-10, z00z_wallets]
tech-stack:
  added: []
  patterns: [include-based stable facade split, source-shape split guards, review-loop closeout with consecutive clean passes]
key-files:
  created:
    - crates/z00z_wallets/src/core/key/bip32_constants.rs
    - crates/z00z_wallets/src/core/key/bip32_key_deriver.rs
    - crates/z00z_wallets/src/core/key/bip32_path.rs
    - crates/z00z_wallets/src/core/key/bip32_path_validator.rs
    - crates/z00z_wallets/src/core/key/bip32_ristretto_bridge.rs
    - crates/z00z_wallets/src/core/key/key_cache.rs
    - crates/z00z_wallets/src/core/key/key_manager_impl.rs
    - crates/z00z_wallets/src/core/key/key_state.rs
    - crates/z00z_wallets/src/core/key/seed_backup_format.rs
    - crates/z00z_wallets/src/core/key/seed_cipher.rs
    - crates/z00z_wallets/src/core/key/seed_mnemonic.rs
    - crates/z00z_wallets/src/core/wallet/chain_id.rs
    - crates/z00z_wallets/src/core/wallet/wallet_entity.rs
    - crates/z00z_wallets/src/core/wallet/wallet_identity.rs
    - crates/z00z_wallets/src/core/wallet/wallet_kernel.rs
    - crates/z00z_wallets/src/core/wallet/wallet_record.rs
  modified:
    - crates/z00z_wallets/src/core/key/bip32.rs
    - crates/z00z_wallets/src/core/key/key_manager.rs
    - crates/z00z_wallets/src/core/key/seed.rs
    - crates/z00z_wallets/src/core/wallet/wallet.rs
    - crates/z00z_wallets/tests/test_bip44.rs
    - crates/z00z_wallets/tests/test_key_manager.rs
    - crates/z00z_wallets/tests/test_seed_salt_policy.rs
    - crates/z00z_wallets/tests/test_wallet_kdf_migration.rs
    - reports/full_verify-report-long-running-tests.txt
key-decisions:
  - Keep `seed.rs`, `bip32.rs`, `key_manager.rs`, and `wallet.rs` as stable include-based facades while sibling seam files own the extracted logic.
  - Treat source-shape split guards as supplemental regression coverage that protects the facade-to-seam contract without replacing behavior tests.
  - Classify the `unlock_from_storage` encrypted-seed-state gap as pre-existing after validating the same behavior in `HEAD`, and defer it instead of widening this structural wave.
patterns-established:
  - "Stable key and wallet roots: caller-visible files stay shallow and include semantically owned sibling seam files."
  - "Split closeout: structural regressions, release-style targeted anchors, max-safe verify, and consecutive clean review passes all have to agree before the wave closes."
requirements-completed: [PH30-SEAMS, PH30-FACADE, PH30-VERIFY]
duration: tracked window 1h 22m
completed: 2026-03-31
---

# Phase 030 Plan 06 Summary

📌 Stable wallet key and identity facades were preserved while seed,
derivation, cache or state, and wallet record ownership moved into coherent
seam files with green targeted tests, a green wallet release suite, and a
clean max-safe workspace gate.

## Performance

- 📌 Duration: tracked window 1h 22m
- 📌 Started: 2026-03-31T16:57:37Z
- 📌 Completed: 2026-03-31T18:19:32Z
- 📌 Tasks: 2
- 📌 Files modified: 25

## Accomplishments

- 📌 Split the wallet key stack into stable facade roots plus dedicated seed,
  BIP32, cache, state, and key-manager implementation seams without changing
  the caller-visible `core::key` surface.
- 📌 Split `wallet.rs` into chain-id, identity, kernel, record, and entity
  seam owners while keeping the outer wallet facade stable for downstream key
  and service layers.
- 📌 Closed the wave with new split-contract source guards, targeted wallet
  anchors, the release-style `z00z_wallets` suite, a clean max-safe verify,
  and two consecutive clean review passes.

## Task Commits

📌 No git commit was created in this closeout. The repository remains dirty,
and the repo rule requires explicit `z00z-git-versioning` workflow usage for
git fixation instead of ad hoc `git commit`.

## Files Created/Modified

- `crates/z00z_wallets/src/core/key/seed.rs` - Stayed as the stable seed root
  while mnemonic, cipher, and backup-format logic moved into sibling seams.
- `crates/z00z_wallets/src/core/key/bip32.rs` - Stayed as the stable derivation
  root while constants, path, validator, key-deriver, and Ristretto bridge
  logic moved into sibling seams.
- `crates/z00z_wallets/src/core/key/key_manager.rs` - Stayed as the stable
  key-manager root while cache, state, and implementation logic moved into
  sibling seams.
- `crates/z00z_wallets/src/core/wallet/wallet.rs` - Stayed as the stable wallet
  root while chain-id, identity, kernel, record, and entity ownership moved
  into sibling seams.
- `crates/z00z_wallets/tests/test_key_manager.rs` - Added a source-shape guard
  for the key-manager split contract.
- `crates/z00z_wallets/tests/test_bip44.rs` - Added a source-shape guard for
  the BIP32 split contract.
- `crates/z00z_wallets/tests/test_seed_salt_policy.rs` - Added a source-shape
  guard for the seed split contract and normalized formatting for workspace
  verify.
- `crates/z00z_wallets/tests/test_wallet_kdf_migration.rs` - Added a
  source-shape guard for the wallet split contract.

## Decisions Made

- 📌 Preserve include-based root files as the stable caller-visible contract
  for this wave instead of normalizing paths or public module shape early.
- 📌 Keep split-contract source guards lightweight and pair them with the
  existing behavior tests and release-style validation instead of replacing
  semantic coverage.
- 📌 Treat the `unlock_from_storage` encrypted-seed-state gap as a deferred
  pre-existing issue because it was confirmed in `HEAD` before the split.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Added split-contract regression guards for the new facades**

- **Found during:** Task 1 and Task 2 closeout validation
- **Issue:** The plan required stable facades, but there was no low-cost guard
  ensuring the root files still pointed at the extracted seam owners after the
  split.
- **Fix:** Added focused source-shape tests for `key_manager.rs`, `bip32.rs`,
  `seed.rs`, and `wallet.rs` so later edits cannot silently collapse the split
  contract.
- **Files modified:** `crates/z00z_wallets/tests/test_key_manager.rs`, `crates/z00z_wallets/tests/test_bip44.rs`, `crates/z00z_wallets/tests/test_seed_salt_policy.rs`, `crates/z00z_wallets/tests/test_wallet_kdf_migration.rs`
- **Verification:** Targeted wallet tests, the release-style `z00z_wallets`
  suite, and the clean max-safe workspace gate all passed.
- **Committed in:** not committed in this closeout

**2. [Rule 1 - Bug] Restored missing rustdoc on extracted public seam items**

- **Found during:** `./.github/skills/z00z-full-verify-gate/scripts/full_verify.sh --max-safe-run`
- **Issue:** New public seam items surfaced `missing_docs` warnings for
  `MAX_BIP32_INDEX`, `MasterKeyGenerator`, and `RistrettoBridge`.
- **Fix:** Added concise rustdoc to the extracted public items so the split did
  not regress public documentation quality.
- **Files modified:** `crates/z00z_wallets/src/core/key/bip32_constants.rs`, `crates/z00z_wallets/src/core/key/bip32_key_deriver.rs`, `crates/z00z_wallets/src/core/key/bip32_ristretto_bridge.rs`
- **Verification:** Re-ran the release-style `z00z_wallets` suite and the
  clean max-safe workspace gate.
- **Committed in:** not committed in this closeout

**3. [Rule 1 - Bug] Fixed formatting fallout surfaced by the workspace gate**

- **Found during:** `./.github/skills/z00z-full-verify-gate/scripts/full_verify.sh --max-safe-run`
- **Issue:** `cargo fmt --check` failed because the newly added seed split test
  needed formatting normalization.
- **Fix:** Ran workspace formatting so the new split-contract test matched the
  repository formatting contract.
- **Files modified:** `crates/z00z_wallets/tests/test_seed_salt_policy.rs`
- **Verification:** Re-ran the clean max-safe workspace gate to a green
  summary.
- **Committed in:** not committed in this closeout

---

📌 Total deviations: 3 auto-fixed issues
📌 Impact on plan: All fixes stayed inside the wallet key or identity split
closure needed for `PH30-SEAMS`, `PH30-FACADE`, and `PH30-VERIFY`.

## Issues Encountered

- 📌 Review pass 1 surfaced an `unlock_from_storage` encrypted-seed-state gap,
  but the same behavior was confirmed in `HEAD`, so it was classified as a
  pre-existing issue rather than a regression from this refactor.
- 📌 Review pass 2 challenged the include-based seam pattern and the new
  source-shape tests, but those findings were non-blocking because the phase
  explicitly preserves stable facades and the behavioral anchors remained
  green.

## User Setup Required

📌 None - no external service configuration or secrets were required for this
plan.

## Next Phase Readiness

- 📌 Later Phase 030 wallet service and caller-normalization waves can build on
  smaller key and wallet facades without reopening the seed or identity
  ownership split.
- 📌 The root-facade split contract is now guarded explicitly, which reduces
  pressure to keep these domains as monoliths in later waves.

## Verification

- 📌 `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- 📌 `cargo test -p z00z_wallets --release --test test_key_manager -- --nocapture`
- 📌 `cargo test -p z00z_wallets --release --test test_bip44 -- --nocapture`
- 📌 `cargo test -p z00z_wallets --release --test test_seed_salt_policy -- --nocapture`
- 📌 `cargo test -p z00z_wallets --release --test test_wallet_kdf_migration -- --nocapture`
- 📌 `cargo test -p z00z_wallets --release --test test_key_manager_storage_unlock -- --nocapture`
- 📌 `cargo test -p z00z_wallets --release --features test-fast --features wallet_debug_dump -- --nocapture`
- 📌 `./.github/skills/z00z-full-verify-gate/scripts/full_verify.sh --max-safe-run`
- 📌 Review-loop completed with four passes total and two consecutive clean runs
  at the end.

## Self-Check

📌 PASSED: `030-06-SUMMARY.md` exists, `ROADMAP.md` shows `6/12 plans
executed` with `030-06-PLAN.md` checked off, and `STATE.md` now points to
Phase 030 Plan 07 as the next active slot.
