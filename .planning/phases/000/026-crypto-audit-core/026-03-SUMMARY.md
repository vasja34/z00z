---
phase: 026-crypto-audit-core
plan: "03"
subsystem: core
tags: [rust, genesis, chain-type, seed-policy, consensus, asset-identity]
requires:
  - phase: 026-crypto-audit-core
    provides: canonical asset-definition identity and validated definition payloads
provides:
  - fail-closed protected-network genesis anchor verification
  - explicit weak-seed rejection without Shannon-threshold approval heuristics
  - canonical asset-definition identity reuse across runtime genesis construction
affects: [026-04, 026-05, z00z_core-genesis, z00z_core-assets]
tech-stack:
  added: []
  patterns: [fail-closed-chain-parse, protected-anchor-contract, canonical-definition-rebuild]
key-files:
  created:
    - .planning/phases/026-crypto-audit-core/026-03-SUMMARY.md
  modified:
    - crates/z00z_core/src/genesis/validator.rs
    - crates/z00z_core/src/genesis/genesis.rs
    - crates/z00z_core/src/genesis/genesis_config.rs
    - crates/z00z_core/src/genesis/asset_std.rs
    - crates/z00z_core/tests/genesis/test_config.rs
    - crates/z00z_core/tests/genesis/test_cross_network_isolation.rs
    - crates/z00z_core/tests/genesis/test_genesis_state_verification.rs
key-decisions:
  - "Parse ChainType once through FromStr and route both seed validation and consensus-anchor checks through the typed value."
  - "Mainnet and testnet treat missing expected genesis anchors as typed failures instead of optional success paths."
  - "Genesis asset-definition construction now reuses canonical AssetDefinition identity, so tests must rebuild definitions when identity-bearing fields change."
patterns-established:
  - "Protected-network genesis validation fails closed on both configuration parsing and consensus-anchor lookup."
  - "Weak-seed policy is explicit and structural: reject zero, ones, sequential, repeating, and known test seeds instead of relying on a Shannon threshold gate."
requirements-completed: [PH26-GENESIS]
duration: multi-session
completed: 2026-03-28
---

# Phase 026 Plan 03 Summary

📌 **Protected-network genesis now fails closed on unknown chain parsing, missing anchors, mismatched anchors, and explicit weak-seed patterns while runtime genesis reuses the canonical asset-definition identity seam**

## Performance

- **Duration:** multi-session
- **Completed:** 2026-03-28
- **Tasks:** 2
- **Files modified:** 8

## Accomplishments

- ✅ Replaced fail-open protected-network consensus verification with typed `ChainType` parsing and missing-anchor rejection for `Mainnet` and `Testnet`.
- ✅ Removed the production Shannon-threshold approval rule from genesis seed validation and replaced it with explicit weak-seed rejection for zero, ones, sequential, repeating, and known test-only seeds.
- ✅ Routed genesis asset-definition construction through canonical `AssetDefinition::new(...)` so runtime genesis no longer preserves a parallel identity derivation rule.
- ✅ Updated integration coverage so protected-network parsing, anchor failures, seed rejection, and cross-network identity semantics are all tested against the new contract.

## Task Commits

📌 This execution closed the plan from validated working-tree state.

1. **Task 1: Make protected-network genesis anchors fail closed** - not separately committed in this execution
2. **Task 2: Replace weak seed heuristics and unsafe network fallback policy** - not separately committed in this execution

**Plan metadata:** not committed in this execution; repo-owned git/versioning checkpoint remains deferred.

## Files Created/Modified

- `crates/z00z_core/src/genesis/validator.rs` - fail-closed anchor verification, typed chain parsing, and explicit weak-seed policy
- `crates/z00z_core/src/genesis/genesis.rs` - direct protected-network verification path and canonical asset-definition construction
- `crates/z00z_core/src/genesis/genesis_config.rs` - documented fail-closed chain and seed contract
- `crates/z00z_core/src/genesis/asset_std.rs` - fail-closed chain parsing in the dev-config helper seam
- `crates/z00z_core/tests/genesis/test_config.rs` - negative coverage for weak seeds and unknown chain types
- `crates/z00z_core/tests/genesis/test_cross_network_isolation.rs` - canonical identity expectations across network-varying payloads
- `crates/z00z_core/tests/genesis/test_genesis_state_verification.rs` - reduced-serial integration coverage rebuilt through canonical ids
- `.planning/phases/026-crypto-audit-core/026-03-SUMMARY.md` - phase execution summary and verification record

## Decisions Made

- 📌 `GenesisSeed::from_config(...)` now parses `ChainType` through `FromStr` and feeds one typed value into seed validation instead of using devnet-like fallback semantics.
- 📌 `verify_genesis_consensus(...)` now treats absent protected-network anchors as a typed error and no longer allows runtime genesis to skip protected verification behind an optional detection gate.
- 📌 Genesis definition construction now uses the same canonical identity seam as plan 01, so any test that changes identity-bearing fields such as `serials` must rebuild the definition instead of mutating fields after id derivation.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Legacy integration test mutated `serials` after canonical id derivation**

- **Found during:** Final `genesis_tests` verification
- **Issue:** `test_genesis_three_networks_produce` reduced `serials` by mutating an already-built `AssetDefinition`, which broke the new canonical `id` integrity contract.
- **Fix:** Rebuilt the reduced-size test definitions through `AssetDefinition::new(...)` so `serials` and `id` stay coherent.
- **Files modified:** `crates/z00z_core/tests/genesis/test_genesis_state_verification.rs`
- **Verification:** `cargo test -p z00z_core --test genesis_tests genesis::genesis_state_verification::test_genesis_three_networks_produce -- --nocapture`; `cargo test -p z00z_core --test genesis_tests -- --nocapture`
- **Committed in:** not committed in this execution

**2. [Rule 1 - Bug] Repeating-byte devnet seed expectation still reflected the removed heuristic policy**

- **Found during:** Library validation after the first plan-03 pass
- **Issue:** Existing tests expected `[42; 32]` to remain acceptable on devnet even after the explicit repeating-byte rejection policy was introduced.
- **Fix:** Updated test expectations so the new structural fail-closed seed policy is authoritative.
- **Files modified:** `crates/z00z_core/src/genesis/genesis.rs`, `crates/z00z_core/tests/genesis/test_config.rs`
- **Verification:** `cargo test -p z00z_core --lib -- --nocapture`
- **Committed in:** not committed in this execution

---

**Total deviations:** 2 auto-fixed bug issues
**Impact on plan:** Both deviations were required to align existing tests with the new canonical genesis and asset-identity contract. Scope remained within the plan-03 genesis boundary.

## Issues Encountered

- ⚠️ The broader workspace release gate with `cargo test --release --features test-fast --features wallet_debug_dump` remains historically blocked outside this plan by read-only vendor doctest failures under `crates/z00z_crypto/tari/crypto/`.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- ✅ `PH26-GENESIS` is now closed with fail-closed protected-network parsing, anchor verification, and weak-seed rejection.
- ✅ `026-04` can reuse the same canonical asset-definition contract when hardening untrusted wire and DTO decode paths.
- ✅ `026-05` inherits stricter genesis and native-asset assumptions for ownership, nonce, and fee policy checks.

## Validation Evidence

- ✅ `cargo test -p z00z_core --lib -- --nocapture` -> `219 passed; 0 failed`
- ✅ `cargo test -p z00z_core --test genesis_tests -- --nocapture` -> `74 passed; 0 failed`
- ✅ `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` -> `BOOTSTRAP COMPLETE`

## Self-Check: PASSED

- ✅ Summary artifact created at `.planning/phases/026-crypto-audit-core/026-03-SUMMARY.md`
- ✅ `PH26-GENESIS` closure evidence recorded against the final tested working tree
- ✅ No commit hashes were claimed because git checkpointing was not performed in this execution

---

*Phase: 026-crypto-audit-core*
*Completed: 2026-03-28*
