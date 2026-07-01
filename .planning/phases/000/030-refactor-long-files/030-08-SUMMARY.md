---
phase: 030-refactor-long-files
plan: 08
subsystem: core-genesis
tags: [rust, core, genesis, facade, seams, alias-normalization]
requires:
  - phase: 030-02
    provides: stable asset-domain split patterns and protected-wave validation posture
provides:
  - stable genesis facade with extracted generation and validation seams
  - shallow `z00z_core::genesis::ChainType` alias normalized across current callers
  - source-shape guards for the split genesis facades
affects: [030-09, 030-10, 030-11, z00z_core, z00z_wallets]
tech-stack:
  added: []
  patterns: [include-based stable facade split, alias-first caller normalization, source-shape split guards, release-style validation closeout]
key-files:
  created:
    - crates/z00z_core/src/genesis/chain_type.rs
    - crates/z00z_core/src/genesis/genesis_accumulator.rs
    - crates/z00z_core/src/genesis/genesis_seed.rs
    - crates/z00z_core/src/genesis/genesis_derivation.rs
    - crates/z00z_core/src/genesis/genesis_output.rs
    - crates/z00z_core/src/genesis/genesis_run.rs
    - crates/z00z_core/src/genesis/genesis_tests.rs
    - crates/z00z_core/src/genesis/genesis_error.rs
    - crates/z00z_core/src/genesis/genesis_verification.rs
    - crates/z00z_core/src/genesis/genesis_config_validate.rs
    - crates/z00z_core/src/genesis/validator_tests.rs
  modified:
    - crates/z00z_core/src/genesis/genesis.rs
    - crates/z00z_core/src/genesis/validator.rs
    - crates/z00z_core/src/genesis/mod.rs
    - crates/z00z_core/benches/genesis/genesis_bench.rs
    - crates/z00z_core/tests/genesis/test_reproducibility.rs
    - crates/z00z_core/tests/genesis/test_determinism.rs
    - crates/z00z_core/tests/genesis/test_range_proofs.rs
    - crates/z00z_core/tests/genesis/test_genesis_state_verification.rs
    - crates/z00z_core/tests/genesis/test_commitment_sum.rs
    - crates/z00z_core/tests/genesis/test_cross_network_isolation.rs
    - crates/z00z_core/tests/genesis/test_batch_verification.rs
    - crates/z00z_wallets/src/services/wallet_service.rs
    - crates/z00z_wallets/src/lib.rs
    - crates/z00z_wallets/tests/test_bip44.rs
    - crates/z00z_wallets/tests/test_rpc_key_derive_e2e.rs
    - crates/z00z_wallets/tests/test_addr_rate_limit_integration.rs
    - crates/z00z_wallets/src/core/key/key_manager.rs
    - crates/z00z_wallets/src/core/key/seed.rs
    - crates/z00z_wallets/src/core/key/bip32.rs
    - crates/z00z_wallets/tests/test_key_manager.rs
    - crates/z00z_wallets/src/core/wallet/wallet.rs
    - crates/z00z_wallets/src/core/address/address_manager.rs
    - reports/full_verify-report-long-running-tests.txt
key-decisions:
  - Keep `crates/z00z_core/src/genesis/mod.rs` as the stable entrypoint while `genesis.rs` and `validator.rs` become thin include-based facades.
  - Normalize `ChainType` callers to `z00z_core::genesis::ChainType` before removing legacy deep caller paths.
  - Add split guards only as structural regression anchors and keep release-style tests as the semantic authority.
patterns-established:
  - "Genesis split closeout: caller normalization lands first, then facade extraction, then repeated review passes and a fresh max-safe gate close the wave."
  - "Core facade split: keep public roots shallow while sibling seam files own deterministic generation, verification, and config validation responsibilities."
requirements-completed: [PH30-PROTECTED, PH30-NORMALIZE, PH30-VERIFY]
completed: 2026-03-31
---

# Phase 030 Plan 08 Summary

📌 Genesis generation and validation were split behind stable facades, and the
current `ChainType` caller inventory was normalized to the shallow genesis
alias before deep-path cleanup.

## Accomplishments

- 📌 Replaced the mixed `genesis.rs` monolith with a shallow include-based
  facade and extracted ownership seams for `ChainType`, accumulation, seed
  policy, deterministic derivation, output writing, runtime orchestration, and
  unit-test ownership.
- 📌 Replaced the mixed `validator.rs` monolith with a shallow include-based
  facade and extracted ownership seams for typed errors, verification,
  config-schema validation, seed-policy checks, and validator test ownership.
- 📌 Normalized the active caller set from
  `z00z_core::genesis::genesis::ChainType` to
  `z00z_core::genesis::ChainType`, including touched wallet callers, benches,
  integration tests, and rustdoc-adjacent paths.
- 📌 Added split-shape regression guards in
  `crates/z00z_core/tests/genesis/test_reproducibility.rs` and closed the wave
  on green bootstrap, targeted genesis anchors, crate-level release checks,
  and the canonical max-safe workspace gate.

## Task Commits

📌 No git commit was created in this closeout. The repository remains dirty,
and the repo rule requires the owned Z00Z git-versioning workflow instead of
ad hoc `git commit` usage.

## Files Created/Modified

- `crates/z00z_core/src/genesis/genesis.rs` - Stayed as the stable generation
  root while the implementation moved into sibling seam files.
- `crates/z00z_core/src/genesis/validator.rs` - Stayed as the stable
  validation root while verification and config checks moved into sibling seam
  files.
- `crates/z00z_core/src/genesis/chain_type.rs` - Took ownership of the
  canonical `ChainType` enum and its string conversion surface.
- `crates/z00z_core/src/genesis/genesis_derivation.rs` - Took ownership of
  deterministic asset derivation, RNG-seed framing, and genesis asset
  generation helpers.
- `crates/z00z_core/src/genesis/genesis_verification.rs` - Took ownership of
  consensus verification, state hashing, chain-type detection, and batch
  commitment verification.
- `crates/z00z_core/src/genesis/genesis_config_validate.rs` - Took ownership
  of config-schema validation and protected-network seed policy.
- `crates/z00z_core/tests/genesis/test_reproducibility.rs` - Added structural
  guards for the split facades and kept reproducibility coverage anchored to
  the stable genesis surface.
- `crates/z00z_core/benches/genesis/genesis_bench.rs` and touched wallet files
  - Moved active `ChainType` callers to the shallow alias so the later
  deep-path cleanup wave can proceed from one stable import contract.

## Decisions Made

- 📌 Keep `mod.rs` as the stable genesis entrypoint during the split so later
  normalization plans can change caller-visible paths in one dedicated wave.
- 📌 Use include-based shallow facades rather than new public submodules so the
  refactor changes ownership boundaries without changing external contracts.
- 📌 Treat early review warnings as hypotheses and verify them against the live
  code before widening scope; the split closed only after two consecutive clean
  review passes.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Cleaned grouped-import `ChainType` remnants after the initial alias inventory passed**

- **Found during:** Task 2 follow-up inventory after the first alias wave
- **Issue:** Direct-path grep was green, but grouped imports still pulled
  `ChainType` from deep genesis paths inside touched bench and test files.
- **Fix:** Reworked the grouped imports so `ChainType` now comes from the
  shallow `z00z_core::genesis::ChainType` alias everywhere in the touched
  caller set.
- **Files modified:** `crates/z00z_core/benches/genesis/genesis_bench.rs`,
  `crates/z00z_core/tests/genesis/test_determinism.rs`,
  `crates/z00z_core/tests/genesis/test_range_proofs.rs`,
  `crates/z00z_core/tests/genesis/test_genesis_state_verification.rs`,
  `crates/z00z_core/tests/genesis/test_commitment_sum.rs`,
  `crates/z00z_core/tests/genesis/test_cross_network_isolation.rs`,
  `crates/z00z_core/tests/genesis/test_batch_verification.rs`
- **Verification:** The post-fix grep inventory found no remaining deep
  `ChainType` path in touched Rust files, and the targeted genesis anchors plus
  final max-safe gate stayed green.
- **Committed in:** not committed in this closeout

**2. [Rule 1 - Bug] Fixed max-safe verify fallout caused by formatting drift after the split**

- **Found during:** canonical `full_verify.sh --max-safe-run`
- **Issue:** The first workspace gate failed only at `cargo fmt --check` on
  touched genesis facades, a touched bench, and one touched wallet test.
- **Fix:** Applied targeted formatting so the split closed on the same release
  gate required by the phase.
- **Files modified:** `crates/z00z_core/src/genesis/genesis.rs`,
  `crates/z00z_core/src/genesis/validator.rs`,
  `crates/z00z_core/benches/genesis/genesis_bench.rs`,
  `crates/z00z_wallets/tests/test_bip44.rs`
- **Verification:** Re-ran file-level quality analysis for the reformatted
  files and then re-ran the max-safe gate successfully.
- **Committed in:** not committed in this closeout

---

📌 Total deviations: 2 auto-fixed issues
📌 Impact on plan: Both fixes stayed inside the alias-normalization and
facade-split wave needed to close `PH30-PROTECTED`, `PH30-NORMALIZE`, and
`PH30-VERIFY`.

## Known Stubs

📌 None detected in the touched Plan 08 seams.

## User Setup Required

📌 None - no external services, keys, or manual environment preparation were
required for this plan.

## Next Phase Readiness

- 📌 Plan 09 can now split wallet tx and RPC surfaces without inheriting the
  legacy deep `ChainType` path churn.
- 📌 Later Phase 030 normalization waves now have one stable genesis alias and
  two shallow facade roots to target instead of two mixed monoliths.

## Verification

- 📌 `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- 📌 `cargo test -p z00z_core --release --test genesis_tests -- --nocapture`
- 📌 `cargo test -p z00z_core --release --features test-fast -- --nocapture`
- 📌 `./.github/skills/z00z-full-verify-gate/scripts/full_verify.sh --max-safe-run`
- 📌 Review loop: five read-only passes total, with the final two consecutive
  passes reporting no significant split-introduced issues.

## Self-Check

📌 PASSED: `030-08-SUMMARY.md` exists, `ROADMAP.md` now shows `8/12 plans
executed` with `030-08-PLAN.md` checked off, and `STATE.md` now points to
Phase 030 Plan 09 as the next active slot.
