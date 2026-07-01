---
phase: 027-crypto-audit-utils
plan: "04"
subsystem: utils
tags: [rust, utils, rng, deterministic, reproducibility, genesis, simulator]
requires:
  - phase: 027-03
    provides: explicit fail-closed utility-contract baseline for downstream Phase 027 rollout waves
provides:
  - feature-gated deterministic RNG availability for approved reproducibility domains only
  - reproducibility-only deterministic RNG trait semantics without approval-sounding secure-entropy implications
  - explicit Cargo allowlist wiring for core genesis tooling and simulator Stage 4 reproducibility
  - narrowed z00z_crypto deterministic helper surface so test-oriented deterministic scalar generation no longer reads like an ambient production seam
affects: [027-05, 027-06, z00z_utils, z00z_core, z00z_crypto, z00z_simulator]
tech-stack:
  added: []
  patterns: [feature-gated-deterministic-rng, reproducibility-only-rng-contract, explicit-deterministic-allowlist]
key-files:
  created:
    - .planning/phases/027-crypto-audit-utils/027-04-SUMMARY.md
  modified:
    - crates/z00z_utils/Cargo.toml
    - crates/z00z_utils/src/lib.rs
    - crates/z00z_utils/src/rng/traits.rs
    - crates/z00z_utils/src/rng/deterministic.rs
    - crates/z00z_utils/src/rng/mod.rs
    - crates/z00z_core/Cargo.toml
    - crates/z00z_simulator/Cargo.toml
    - crates/z00z_crypto/src/types.rs
key-decisions:
  - "Treat deterministic RNG as a reproducibility-only contract and remove approval-sounding secure-entropy semantics from the public trait surface."
  - "Gate deterministic RNG exports behind explicit tests or features and opt approved genesis and simulator crates in through Cargo manifests rather than ambient exports."
  - "Narrow `z00z_crypto::types::random_deterministic` to test-oriented cfg domains so `z00z_crypto` no longer exposes an implicit broad production deterministic seam."
patterns-established:
  - "Deterministic RNG is an explicit opt-in reproducibility capability, not ambient production entropy."
  - "Approved reproducibility domains are declared in Cargo feature wiring and validated by release-style gates."
requirements-completed: [PH27-RNG]
duration: multi-session
completed: 2026-03-29
---

# Phase 027 Plan 04 Summary

📌 **Feature-gated deterministic RNG now stays reproducibility-only for approved genesis and simulator paths while `z00z_crypto` test helpers lose their implicit production seam.**

## Performance

- **Duration:** multi-session
- **Completed:** 2026-03-29T15:58:34Z
- **Tasks:** 2
- **Files modified:** 8

## Accomplishments

- ✅ Reframed deterministic RNG semantics around reproducibility rather than approval-sounding secure entropy and guarded the surface behind explicit feature or test gates.
- ✅ Added explicit `deterministic-rng` opt-in wiring for `z00z_core` and `z00z_simulator` so approved genesis tooling and simulator reproducibility remain available without broad ambient exposure.
- ✅ Closed the allowlist with release-style validation and a reproducible production-perimeter scan, then manually classified the remaining source matches that were test-only or documentation-only noise.

## Task Commits

📌 This execution closed the plan from validated working-tree state.

1. **Task 1: Replace approval-sounding deterministic trait semantics and add an explicit guardrail** - not separately committed in this execution
2. **Task 2: Opt approved genesis and simulator domains into the new deterministic guardrail** - not separately committed in this execution

**Plan metadata:** not committed in this execution; repo-owned git or versioning checkpoint remains deferred.

## Files Created/Modified

- `crates/z00z_utils/Cargo.toml` - added the explicit `deterministic-rng` capability gate for reproducibility-only consumers.
- `crates/z00z_utils/src/rng/traits.rs` - removed approval-sounding deterministic semantics from the trait contract and clarified the reproducibility-only vocabulary.
- `crates/z00z_utils/src/rng/deterministic.rs` - aligned deterministic-provider docs and behavior with the guarded reproducibility contract.
- `crates/z00z_utils/src/rng/mod.rs` - gated deterministic exports so they are no longer ambient outside approved feature or test contexts.
- `crates/z00z_utils/src/lib.rs` - narrowed prelude-level deterministic exports to the explicit guarded surface.
- `crates/z00z_core/Cargo.toml` - opted core genesis and asset-generation tooling into the deterministic guardrail intentionally.
- `crates/z00z_simulator/Cargo.toml` - opted simulator reproducibility into the deterministic guardrail intentionally.
- `crates/z00z_crypto/src/types.rs` - restricted `random_deterministic` to test-oriented cfg domains and kept an explicit local `CryptoRng` bound where deterministic scalar generation is still needed for tests.

## Decisions Made

- 📌 Deterministic RNG stays available only as a reproducibility seam; the public trait contract must not read like a blessed production entropy source.
- 📌 Approved reproducibility domains are the Phase 027 allowlist from the plan and context: `genesis.rs`, `asset_std.rs`, `assets_generation_cli.rs`, and simulator Stage 4 tx-lane code.
- 📌 `z00z_crypto` deterministic scalar generation remains available only where tests or explicit test-like features need reproducible vectors; it is no longer left as a broad production-facing seam.

## Final Allowlist Classification

| Match class | Files | Final state | Rationale |
| --- | --- | --- | --- |
| Guarded infrastructure surface | `crates/z00z_utils/src/rng/traits.rs`, `crates/z00z_utils/src/rng/deterministic.rs`, `crates/z00z_utils/src/rng/mod.rs`, `crates/z00z_utils/src/lib.rs` | retained and feature-gated | These files define the contract and guarded export surface rather than consuming deterministic RNG as production entropy. |
| Approved production reproducibility consumers | `crates/z00z_core/bin/assets/assets_generation_cli.rs`, `crates/z00z_core/src/genesis/asset_std.rs`, `crates/z00z_core/src/genesis/genesis.rs`, `crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_lane_impl.rs` | allowlisted and kept green | These are the verified genesis and simulator reproducibility domains named in the plan. |
| Test or doc noise inside production paths | `crates/z00z_core/src/assets/assets.rs`, `crates/z00z_core/src/genesis/mod.rs` | classified as non-production | Remaining matches are inline docs or `#[cfg(test)]` helper code rather than live production consumers. |
| Previously ambiguous extra seam | `crates/z00z_crypto/src/types.rs` | narrowed to test-oriented cfg | `random_deterministic` remains for reproducible test vectors only and no longer acts like an ambient production deterministic helper. |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] The broad `z00z_utils` export surface still let deterministic RNG look ambient outside approved domains**

- **Found during:** Task 1 rollout review
- **Issue:** feature gating at the provider module was not sufficient if prelude or top-level exports still made deterministic types appear broadly available.
- **Fix:** gated deterministic exports in `crates/z00z_utils/src/rng/mod.rs` and `crates/z00z_utils/src/lib.rs` so the public surface matches the explicit capability model.
- **Files modified:** `crates/z00z_utils/src/rng/mod.rs`, `crates/z00z_utils/src/lib.rs`
- **Verification:** release-style approved-domain builds remained green and the deterministic production-perimeter scan closed with only approved or classified matches.
- **Committed in:** not committed in this execution

**2. [Rule 2 - Missing Critical] `z00z_crypto::types::random_deterministic` remained a visible non-allowlisted deterministic seam**

- **Found during:** Task 2 allowlist review
- **Issue:** the plan already called out `crates/z00z_crypto/src/types.rs` as an extra deterministic seam outside the narrow genesis and simulator allowlist.
- **Fix:** narrowed `random_deterministic` behind `#[cfg(any(test, feature = "test-utils", feature = "test-fast"))]` so the helper remains available for reproducible vectors without reading as a general production seam.
- **Files modified:** `crates/z00z_crypto/src/types.rs`
- **Verification:** focused source read confirmed the cfg gate, and release-style approved-domain commands still passed.
- **Committed in:** not committed in this execution

**3. [Rule 3 - Blocking] The first validation batch produced incomplete evidence because expected log artifacts were missing**

- **Found during:** Task 2 closeout verification
- **Issue:** an earlier batched validation run reported success but only left the `z00z_utils` release-test log on disk, so the remaining core and simulator gates were not trustworthy as recorded evidence.
- **Fix:** re-ran the remaining required commands individually with dedicated log capture and used those rerun artifacts as the closeout evidence base.
- **Files modified:** none
- **Verification:** dedicated logs confirmed green results for core release check, genesis tests, simulator release tests, and the release `scenario_1` run.
- **Committed in:** not committed in this execution

---

**Total deviations:** 3 auto-fixed (2 missing critical, 1 blocking)
**Impact on plan:** all deviations stayed inside `PH27-RNG` closure work and were necessary to make the guarded surface truthful and the validation evidence reproducible.

## Issues Encountered

- ⚠️ The first batched verification attempt was not acceptable as closure evidence because only one of the expected log files actually existed.
- ⚠️ A broad deterministic-RNG search was too noisy because docs, tests, and bench-like material inside `src` paths still matched the raw symbol search.
- ⚠️ The final closure scan therefore required one narrower production-perimeter pass plus manual classification of the remaining `z00z_crypto` and `assets.rs` matches.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- ✅ `PH27-RNG` is now closed with explicit guardrails, allowlist wiring, and release-style evidence for approved reproducibility domains.
- ✅ Phase `027-05` can assume deterministic RNG is no longer ambient production entropy and that approved reproducibility domains are declared explicitly in Cargo configuration.
- ✅ The remaining Phase 027 work can treat logger and I/O hardening as the next highest-value closure slice without reopening the RNG contract.

## Validation Evidence

- ✅ `cargo test -p z00z_utils --release` -> existing closeout log confirmed `155 passed; 0 failed`
- ✅ `cargo check -p z00z_core --release --bin assets_generation_cli` -> passed (`Finished 'release' profile [optimized] target(s) in 4.89s`)
- ✅ `cargo test -p z00z_core --test genesis_tests -- --nocapture` -> passed (`74 passed; 0 failed`)
- ✅ `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump` -> passed with all visible simulator, wallet, claim, stage, and doc-test suites green
- ✅ `cargo run --release -p z00z_simulator --bin scenario_1 --features wallet_debug_dump` -> passed (`scenario_1.result: success`)
- ✅ Final deterministic production-perimeter scan command:

```bash
rg -n 'DeterministicRngProvider|DeterministicRng|random_deterministic' crates --glob 'crates/**/src/**/*.rs' --glob 'crates/**/bin/**/*.rs' --glob '!crates/z00z_crypto/tari/**' --glob '!**/tests/**' --glob '!**/examples/**' --glob '!**/benches/**' --glob '!**/fuzz/**'
```

- ✅ Final scan result: only the guarded `z00z_utils` infrastructure surface, the approved genesis and simulator allowlist, documentation-only matches, one `#[cfg(test)]` helper block in `crates/z00z_core/src/assets/assets.rs`, and the now test-oriented `z00z_crypto::types::random_deterministic` seam remained

## Self-Check: PASSED

- ✅ Summary artifact created at `.planning/phases/027-crypto-audit-utils/027-04-SUMMARY.md`
- ✅ Validation evidence recorded against dedicated rerun logs rather than the earlier incomplete batch-log state
- ✅ No commit hashes were claimed because git checkpointing was not performed in this execution

---

*Phase: 027-crypto-audit-utils*
*Completed: 2026-03-29*
