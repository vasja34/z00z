---
phase: 033-crypto-audit-scenario-2
plan: 2
subsystem: testing
tags: [simulator, claim-pkg, wallets, semantics, derive_s_out]
requires:
  - phase: 033-01
    provides: stage-3-and-stage-4 continuity baselines for Phase 033 semantic closure
provides:
  - explicit claim-bundle discriminator and version enforcement evidence at the simulator continuity seam
  - wallet-local anti-theft wording and test naming that no longer overclaims validator-level guarantees
  - canonical derive_s_out wording across simulator and crypto documentation surfaces
affects: [033-03, ownership, privacy-routing, scenario-1]
tech-stack:
  added: []
  patterns:
    - explicit serialization-shape tests for continuity bundles
    - wallet-local security wording must stay narrower than validator-facing theorems
    - derive_s_out is the only canonical output-secret contract on active repository surfaces
key-files:
  created:
    - .planning/phases/033-crypto-audit-scenario-2/033-02-SUMMARY.md
  modified:
    - crates/z00z_simulator/src/claim_pkg_consumer.rs
    - crates/z00z_simulator/tests/test_claim_persist.rs
    - crates/z00z_wallets/src/core/tx/spend_verification.rs
    - crates/z00z_wallets/src/core/tx/witness_gate.rs
    - crates/z00z_wallets/tests/test_asset_ownership_security.rs
    - crates/z00z_simulator/README.md
    - .planning/temp/Z00Z-ECC-IDEAS.md
    - crates/z00z_crypto/.todo/Z00Z-ECC-crypto.md
    - versions.yaml
key-decisions:
  - "Treat Task 4 as an evidence-closure task: keep the existing load guard, add explicit serialization-shape tests, and prove bundle version rejection at the continuity seam."
  - "Resolve Task 5 by narrowing language to the honest wallet-local boundary instead of inventing an unsupported validator-level anti-theft theorem."
  - "Treat Task 6 as a repository-surface cleanup task and remove every active non-planning random32 output-secret reference before declaring derive_s_out canonical."
patterns-established:
  - "Claim continuity must reject implicit/defaulted bundle shapes through explicit discriminator checks and persisted artifact assertions."
  - "Security wording in wallet spend paths must name the actual proof boundary, not a broader theorem the code does not yet implement."
requirements-completed: [PH32-SEM, PH32-CLAIM-TRUST, PH32-HONEST]
duration: 69min
completed: 2026-04-06
---

# Phase 033 Plan 02 Summary

Claim continuity now proves explicit bundle discriminators, wallet-local anti-theft semantics stay honest, and derive_s_out is the only active output-secret contract across the simulator-facing repository surface.

## Performance

- **Duration:** 69 min
- **Started:** 2026-04-06T21:35:28Z
- **Completed:** 2026-04-06T22:44:16Z
- **Tasks:** 3
- **Files modified:** 9

## Accomplishments

- Added Task 4 continuity coverage that proves claim package bundles serialize explicit `kind`, `package_type`, and `version` fields and that decode rejects missing discriminator drift.
- Narrowed Task 5 wording and tests so the repository now states the honest wallet-local receiver-secret boundary instead of implying a validator-level anti-theft theorem.
- Removed competing Task 6 output-secret wording from simulator and crypto documentation surfaces so `derive_s_out(k_dh, r_pub, serial_id)` is the single active model outside historical planning artifacts.

## Task Commits

Each task was committed atomically:

1. **Task 4: Publish-Bound Claim Continuity** - `5b457afd` (feat/test)
2. **Task 5: Sender Knowledge Versus Anti-Theft** - `70397b58` (docs/test)
3. **Task 6: Canonical Output-Secret Semantics** - `1e2427c4` (docs)
4. **Task 6 follow-up: residual crate todo cleanup** - `302d443d` (docs)

## Files Created/Modified

- `.planning/phases/033-crypto-audit-scenario-2/033-02-SUMMARY.md` - execution summary for Plan 02.
- `crates/z00z_simulator/src/claim_pkg_consumer.rs` - Task 4 explicit bundle-shape unit tests.
- `crates/z00z_simulator/tests/test_claim_persist.rs` - Task 4 persisted artifact bundle-shape assertions.
- `crates/z00z_wallets/src/core/tx/spend_verification.rs` - Task 5 public spend verifier wording narrowed to the actual proof boundary.
- `crates/z00z_wallets/src/core/tx/witness_gate.rs` - Task 5 witness-gate wording narrowed to wallet-local receiver-secret scope.
- `crates/z00z_wallets/tests/test_asset_ownership_security.rs` - Task 5 test naming and file banner aligned with wallet-local semantics.
- `crates/z00z_simulator/README.md` - Task 6 simulator boundary note frozen on `derive_s_out`.
- `.planning/temp/Z00Z-ECC-IDEAS.md` - Task 6 temp design note rewritten to the canonical derived model.
- `crates/z00z_crypto/.todo/Z00Z-ECC-crypto.md` - final residual non-planning `random32` wording removed.
- `versions.yaml` - repository version advanced from `2.21.0` to `2.21.4` across the four atomic task commits.

## Decisions Made

- Kept Task 4 focused on proof surfaces because the repository already enforced the claim bundle load boundary at runtime; the missing work was evidence, not a new loader architecture.
- Closed Task 5 by narrowing claims to the wallet-local rule because the repository still does not expose an end-to-end validator-facing anti-theft proof boundary.
- Tightened Task 6 beyond the initially edited simulator docs after a repository scan found one extra `.todo` surface still using the old wording.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Fixed rustfmt-only failure in the first Task 4 version-managed commit attempt**

- **Found during:** Task 4 (Publish-Bound Claim Continuity)
- **Issue:** The initial version-manager patch run failed in pre-commit because the new Task 4 assertions did not match repository rustfmt layout.
- **Fix:** Reflowed the affected assertions to rustfmt-compatible layout, reverted the failed-attempt version bump in `versions.yaml`, and retried the same scoped Task 4 commit.
- **Files modified:** `crates/z00z_simulator/src/claim_pkg_consumer.rs`, `crates/z00z_simulator/tests/test_claim_persist.rs`, `versions.yaml`
- **Verification:** `cargo fmt --all --check` passed before the retry, then the version-manager commit completed successfully.
- **Committed in:** `5b457afd` (part of Task 4 commit)

**2. [Rule 2 - Missing Critical] Removed one remaining non-planning `random32` output-secret reference after Task 6 initial cleanup**

- **Found during:** Task 6 (Canonical Output-Secret Semantics)
- **Issue:** A repository scan after the first Task 6 commit still found `crates/z00z_crypto/.todo/Z00Z-ECC-crypto.md` describing `s_out = random32`, which would have made the summary overstate repository consistency.
- **Fix:** Rewrote the `.todo` note to the canonical `derive_s_out(k_dh, r_pub, serial_id)` model and committed it as a Task 6 follow-up.
- **Files modified:** `crates/z00z_crypto/.todo/Z00Z-ECC-crypto.md`
- **Verification:** `grep_search` over `crates/**` with ignored files included returned no remaining `random32` matches.
- **Committed in:** `302d443d` (Task 6 follow-up)

---

**Total deviations:** 2 auto-fixed (1 blocking, 1 missing critical)
**Impact on plan:** Both fixes were necessary to keep the commit flow clean and the output-secret semantics honest. No architectural scope change was introduced.

## Issues Encountered

- A debug-profile `test_claim_persist_restart` path still exposes a pre-existing range-proof limitation outside release mode, so accepted Task 4 verification was anchored to the release-style simulator path instead of the debug-only failure lane.
- The executor surface did not expose a dedicated runner for `/.github/prompts/gsd-review-tasks-execution.prompt.md`, so the review requirement was satisfied by repeated manual prompt-guided inspection during task validation rather than by a separate prompt-execution tool.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Phase 033 can now build later ownership and routing slices on a narrower, honest semantic baseline.
- Claim continuity and output-secret wording no longer drift across the active simulator and wallet surfaces touched by this plan.
- Historical planning artifacts under `.planning/phases/**` still reference the previous audit discussion, but the active repository surfaces used by code and current docs are aligned.

## Threat Flags

None.

## Known Stubs

None.

## Self-Check

PASSED.

- FOUND: `.planning/phases/033-crypto-audit-scenario-2/033-02-SUMMARY.md`
- FOUND: `5b457afd`
- FOUND: `70397b58`
- FOUND: `1e2427c4`
- FOUND: `302d443d`

---
*Phase: 033-crypto-audit-scenario-2*
*Completed: 2026-04-06*
