<!-- markdownlint-disable MD003 MD022 MD036 MD041 MD047 MD056 MD060 -->
---
phase: 033
slug: crypto-audit-scenario-2
status: complete
nyquist_compliant: true
wave_0_complete: true
created: 2026-04-09
updated: 2026-04-09
---

# Phase 033 — Validation Strategy

> Per-phase validation contract reconstructed from the executed Phase 033 plans,
> summaries, test spec, test tasks, and the final narrowed requirement state.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust `cargo test` integration/unit tests plus repository `rg`/source-shape assertion checks |
| **Config file** | [Cargo.toml](/home/vadim/Projects/z00z/Cargo.toml) |
| **Quick run command** | `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` |
| **Full suite command** | `cargo test --release --features test-fast --features wallet_debug_dump` |
| **Estimated runtime** | repository-dependent long-running release suite |

---

## Sampling Rate

- **After every task commit:** Run `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- **After every plan wave:** Run the phase-local canonical matrix from [033-TEST-SPEC.md](/home/vadim/Projects/z00z/.planning/phases/033-crypto-audit-scenario-2/033-TEST-SPEC.md) and corroborate with `cargo test --release --features test-fast --features wallet_debug_dump` when the workspace is stable enough for a broad rerun
- **Before `/gsd-verify-work`:** The targeted Phase 033 matrix must stay green, with broader release reruns used as corroborating evidence where phase summaries recorded them clean
- **Max feedback latency:** bounded by bootstrap-first gates and targeted release binaries before any broad rerun

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 033-01-01 | 01/10/11 | 1 | PH32-CLAIM-TRUST | T-033-01 / T-033-03 / T-033-28 / T-033-29 / T-033-31 | The live claim verifier rejects post-sign tuple drift, keeps reject taxonomy category-specific, and preserves the helper-owned claim-source contract without overclaiming persisted authority | integration | `cargo test -p z00z_wallets --release --lib core::tx::claim_tx::claim_tx_tests::test_claim_source_asset_id_drift_rejected -- --exact --nocapture && cargo test -p z00z_wallets --release --lib core::tx::claim_tx::claim_tx_tests::test_chain_id_drift_rejected_before_proof -- --exact --nocapture && cargo test -p z00z_wallets --release --lib core::tx::claim_tx::claim_tx_tests::test_source_root_ver_rejected_with_precise_error -- --exact --nocapture && cargo test -p z00z_wallets --release --lib core::tx::claim_tx::claim_tx_tests::test_source_proof_ver_rejected_with_precise_error -- --exact --nocapture && cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_claim_pkg_crypto_support -- --nocapture` | ✅ | ✅ green |
| 033-02-01 | 01/02/10/21 | 2 | PH32-CLAIM-TRUST | T-033-02 / T-033-04 / T-033-30 / T-033-32 / T-033-63 | Claim continuity and package shape stay honest: helper-owned continuity is explicit, implicit discriminator drift rejects, and the high-severity claim row remains frozen to persisted-membership-or-formal-narrowing language | integration + assertion | `cargo test -p z00z_storage --release --test test_claim_source_proof -- --nocapture && cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_claim_persist -- --nocapture && cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_scenario1_stage_surface -- --nocapture` | ✅ | ✅ green |
| 033-03-01 | 03/12 | 3 | PH32-HONEST | T-033-07 / T-033-08 / T-033-35 / T-033-36 | Wallet-local ownership, `leaf_ad_id` binding, and request-vs-card routing remain distinct and cannot silently flatten into a stronger repository-wide privacy theorem | integration | `cargo test -p z00z_wallets --release --features test-fast --test test_spend_witness_gate -- --nocapture && cargo test -p z00z_wallets --release --features test-fast --test test_e2e_req_flow -- --nocapture` | ✅ | ✅ green |
| 033-04-01 | 04/05/13/22 | 4 | PH32-SPEND | T-033-10 / T-033-13 / T-033-38 / T-033-39 / T-033-40 / T-033-64 | The public spend verifier stays limited to the delivered persisted public spend contract, semantic acceptance precedes state mutation, and the nullifier semantics gap remains explicit rather than silently closed | integration + assertion | `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_scenario1_spend_gate -- --nocapture && cargo test -p z00z_wallets --release --features test-fast --test test_s5_closure_gate -- --nocapture && cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_scenario1_stage_surface -- --nocapture` | ✅ | ✅ green |
| 033-05-01 | 04/05/06/13/15/18/23 | 5 | PH32-CHECKPOINT | T-033-12 / T-033-15 / T-033-16 / T-033-17 / T-033-18 / T-033-41 / T-033-42 / T-033-43 / T-033-55 / T-033-60 / T-033-65 | Checkpoint acceptance stays package-coupled, tampered and replayed persisted artifacts fail closed, and the crossed Task 65 row remains pinned to the standalone-backend gap instead of being normalized away; the remaining compatibility-looking payload-only backend closure is guarded by accepted-path tests plus wording/source-shape assertions rather than a standalone dedicated runtime file | integration + assertion | `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_checkpoint_acceptance -- --nocapture && cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_stage6_checkpoint_final_gate -- --nocapture && cargo test -p z00z_storage --release --test test_checkpoint_replay_inputs -- --nocapture && cargo test -p z00z_storage --release --test test_checkpoint_link_injective -- --nocapture && cargo test -p z00z_storage --release --test test_redb_rehydrate -- --nocapture && cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_scenario1_stage_surface -- --nocapture` | ✅ | ✅ green |
| 033-06-01 | 07/15 | 6 | PH32-SECRET | T-033-20 / T-033-21 / T-033-44 | Default Scenario 1 flow emits no plaintext wallet-secret artifacts and any debug export remains feature-gated, private, and non-default | integration + assertion | `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_stage2_secret_artifacts -- --nocapture` | ✅ | ✅ green |
| 033-06-02 | 08/15 | 6 | PH32-SECRET | T-033-22 / T-033-45 / T-033-62 | Deterministic seeded RNG remains bounded to simulator and CI reproducibility, preserves the `None == zero-seed` fallback, and does not expand into a production entropy theorem | integration + assertion | `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_transport_rng_boundaries -- --nocapture` | ✅ | ✅ green |
| 033-07-01 | 08/17/18/19/20/21/22/23 | 7 | PH32-HONEST | T-033-23 / T-033-24 / T-033-25 / T-033-26 / T-033-27 / T-033-46 / T-033-47 / T-033-48..65 | Documentation, caution rows, and crossed high-severity findings remain synchronized with the narrowed live seams and do not overclaim trustlessness, stronger checkpoint authority, or closed spend/claim semantics | assertion | `cargo test -p z00z_wallets --release --features test-fast --test test_s5_closure_gate -- --nocapture && cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_scenario1_stage_surface -- --nocapture` | ✅ | ✅ green |

*Status: ✅ green · ❌ red · ⚠️ partial*

---

## Wave 0 Requirements

Existing infrastructure covers all Phase 033 requirements.

No additional Wave 0 framework installation, shared fixture bootstrapping, or new Nyquist harness files are needed.

---

## Manual-Only Verifications

No manual-only rows remain in the executed matrix. Phase behaviors are covered
by runtime tests, source-shape wording guards, or both.

---

## Validation Audit 2026-04-09

| Metric | Count |
|--------|-------|
| Gaps found | 0 |
| Resolved | 0 |
| Escalated | 0 |

### Audit Notes

- Input state: **State B** — no existing `033-VALIDATION.md`, but all `033-*-SUMMARY.md` artifacts exist.
- A stale terminal-level concern about `cargo test --release --features test-fast --features wallet_debug_dump` exiting `101` was not reproduced during the isolated corroborating rerun started on 2026-04-08, but that rerun was not fully green end to end: after clearing the slow claim and checkpoint surfaces that triggered the recheck, it later failed in `test_scenario1_stage_surface` on the Phase 033 receiver-identity wording contract.
- The late rerun failure was a source-shape regression in [crates/z00z_wallets/src/services/wallet_service_actions_receive.rs](/home/vadim/Projects/z00z/crates/z00z_wallets/src/services/wallet_service_actions_receive.rs), where the required `repo-wide fail-closed policy` wording had been split across lines. The exact failing harness was re-run with `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_scenario1_stage_surface -- --nocapture`, and it returned `27 passed; 0 failed` after the wording fix.
- Test infrastructure is already present in the workspace and in the phase-local fallback contract defined by [033-TEST-SPEC.md](/home/vadim/Projects/z00z/.planning/phases/033-crypto-audit-scenario-2/033-TEST-SPEC.md) and [033-TESTS-TASKS.md](/home/vadim/Projects/z00z/.planning/phases/033-crypto-audit-scenario-2/033-TESTS-TASKS.md).
- The final Phase 033 requirement state in [REQUIREMENTS.md](/home/vadim/Projects/z00z/.planning/REQUIREMENTS.md) marks `PH32-CLAIM-TRUST` and `PH32-SPEND` as completed through formal narrowing, while `PH32-CHECKPOINT`, `PH32-SECRET`, and `PH32-HONEST` were already completed and remain regression-guarded by live tests.
- The two test files that `033-TEST-SPEC.md` proposed as standalone creations were absorbed by the executed reuse-first strategy:
  - the checkpoint backend boundary coverage landed on the canonical live seam in [crates/z00z_simulator/tests/test_checkpoint_acceptance.rs](/home/vadim/Projects/z00z/crates/z00z_simulator/tests/test_checkpoint_acceptance.rs) instead of a separate `test_checkpoint_backend_boundary.rs` file;
  - the RNG scope contract landed in [crates/z00z_simulator/tests/test_transport_rng_boundaries.rs](/home/vadim/Projects/z00z/crates/z00z_simulator/tests/test_transport_rng_boundaries.rs) instead of a separate `test_rng_scope_contract.rs` file.
- No new Nyquist test file was generated in this audit because the executed Phase 033 summaries already recorded the needed coverage on the live reuse-first test homes, and the remaining validation work was documentation of that final matrix rather than missing executable behavior.
- Some trust-boundary closures in the final matrix remain enforced by source-shape wording guards in addition to runtime tests. The executed Phase 033 closeout therefore proves an automated behavior-plus-wording matrix, not a runtime-only proof of every narrowed caution row.
- The latest workspace context also shows a successful bootstrap run, which matches the phase summaries' bootstrap-first contract.

---

## Validation Sign-Off

- [x] All mapped phase surfaces have automated verification commands
- [x] Sampling continuity is present through bootstrap-first gates, targeted release binaries, and summary-backed reruns
- [x] Wave 0 infrastructure already covers the phase surface
- [x] No watch-mode flags were used in the validation contract
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** approved 2026-04-09

Reason: Phase 033 has a complete automated verification matrix for the executed
narrowed phase surface, and the previously proposed standalone Nyquist files
were honestly absorbed into existing canonical test homes rather than left
missing. Some late trust-boundary rows remain guarded by source-shape assertions
in addition to runtime behavior tests, and this validation record now states
that explicitly.