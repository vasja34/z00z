<!-- markdownlint-disable MD003 MD022 MD036 MD041 MD047 MD056 MD060 -->
---
phase: 032
slug: crypto-audit-scenario-1
status: partial
nyquist_compliant: false
wave_0_complete: true
created: 2026-04-05
updated: 2026-04-05
---

# Phase 032 — Validation Strategy

> Per-phase validation contract reconstructed from the executed Phase 032 plans,
> summaries, verification evidence, and the current reopened requirement state.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust `cargo test` integration/unit tests plus repository `rg` assertion checks |
| **Config file** | [Cargo.toml](/home/vadim/Projects/z00z/Cargo.toml) |
| **Quick run command** | `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` |
| **Full suite command** | `cargo test --release --features test-fast --features wallet_debug_dump` |
| **Estimated runtime** | repository-dependent long-running release suite |

---

## Sampling Rate

- **After every task commit:** Run `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- **After every plan wave:** Run the phase-local canonical matrix from [032-TEST-SPEC.md](/home/vadim/Projects/z00z/.planning/phases/032-crypto-audit-scenario-1/032-TEST-SPEC.md) plus `cargo test --release --features test-fast --features wallet_debug_dump`
- **Before `/gsd-verify-work`:** The targeted Phase 032 matrix and the broader release suite must stay green
- **Max feedback latency:** bounded by targeted release binaries first, then broader release reruns

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 032-01-01 | 01 | 0 | PH32-SEM | T-032-01 / T-032-02 / T-032-03 | Scenario 1 semantic freeze keeps `leaf_ad_id`, `s_out`, request/card binding, and trust language stable and test-backed | integration | `cargo test -p z00z_wallets --release --features test-fast --test test_scenario1_semantics -- --nocapture` | ✅ | ✅ green |
| 032-02-01 | 02 | 1 | PH32-CLAIM-BIND | T-032-04 / T-032-05 / T-032-06 | The authority signature binds the full authenticated claim tuple including authoritative source-root fields | unit + integration | `cargo test -p z00z_crypto --release --test test_claim_v2_contract -- --nocapture` | ✅ | ✅ green |
| 032-03-01 | 03 | 2 | PH32-CLAIM-TRUST | T-032-07 / T-032-08 / T-032-09 | Claim production and consumption share one canonical helper-owned root/proof contract and reject forged or stale packages, but persisted storage-backed continuity for the original requirement remains open | integration | `cargo test -p z00z_storage --release --test test_claim_source_proof -- --nocapture && cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_claim_pkg_crypto_support -- --nocapture` | ✅ | ⚠️ partial |
| 032-04-01 | 04 | 3 | PH32-SPEND | T-032-10 / T-032-11 / T-032-12 | The current-stack spend boundary rejects structural-only acceptance and enforces the persisted spend proof/auth contract, but does not yet prove the original nullifier-semantics portion of the requirement | integration | `cargo test -p z00z_wallets --release --features test-fast --test test_spend_witness_gate -- --nocapture && cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_scenario1_spend_gate -- --nocapture` | ✅ | ⚠️ partial |
| 032-05-01 | 05 | 4 | PH32-CHECKPOINT | T-032-13 / T-032-14 / T-032-15 | Checkpoint apply rejects tampered package proof continuity and no longer accepts placeholder proof/spent-set success lanes | integration | `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_checkpoint_acceptance -- --nocapture && cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_stage6_checkpoint_final_gate -- --nocapture` | ✅ | ✅ green |
| 032-06-01 | 06 | 5 | PH32-SECRET | T-032-16 / T-032-17 / T-032-18 | Default Scenario 1 flow emits no plaintext wallet secrets and keeps seeded RNG bounded to simulator-only fixture behavior | integration | `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_stage2_secret_artifacts -- --nocapture && cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_transport_rng_boundaries -- --nocapture` | ✅ | ✅ green |
| 032-07-01 | 07 | 6 | PH32-HONEST | T-032-19 / T-032-20 / T-032-21 | Status docs and closeout language do not overclaim STARK/FRI, trustless verification, stronger checkpoint authority, or closed spend and claim-trust semantics | assertion | `rg -n "(bootstrap_tests|test-fast|wallet_debug_dump|GSD-Review-Tasks-Execution|2 consecutive clean runs|does not prove|out of scope|PH32-SPEND|PH32-CLAIM-TRUST)" .planning/phases/032-crypto-audit-scenario-1/032-HONEST-CLOSEOUT.md docs/code-review/032-scenario-1-crypto-status.md .planning/phases/032-crypto-audit-scenario-1/032-VERIFICATION.md` | ✅ | ✅ green |

*Status: ✅ green · ❌ red · ⚠️ partial*

---

## Wave 0 Requirements

Existing infrastructure covers all Phase 032 requirements.

No additional Wave 0 framework installation or harness bootstrapping is needed.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Persisted storage-backed claim membership continuity for accepted Scenario 1 claim packages | PH32-CLAIM-TRUST | Current code reuses one canonical helper seam, but that helper still builds a synthetic one-item store contract rather than proving continuity against persisted storage-backed state | Do not mark Phase 032 Nyquist-compliant until the claim-source helper derives root/proof data from persisted store-backed membership state or the requirement is formally narrowed and re-approved |
| Full original spend requirement closure, including nullifier semantics inside the regular-spend public contract | PH32-SPEND | The current tree explicitly keeps this portion of the requirement open in [REQUIREMENTS.md](/home/vadim/Projects/z00z/.planning/REQUIREMENTS.md) and [032-VERIFICATION.md](/home/vadim/Projects/z00z/.planning/phases/032-crypto-audit-scenario-1/032-VERIFICATION.md); this is an implementation gap, not a missing test harness | Do not mark Phase 032 Nyquist-compliant until the regular-spend verifier carries and validates nullifier semantics or the requirement is formally narrowed and re-approved |

---

## Validation Audit 2026-04-05

| Metric | Count |
|--------|-------|
| Gaps found | 2 |
| Resolved | 0 |
| Escalated | 2 |

### Audit Notes

- Input state: **State B** — no existing `032-VALIDATION.md`, but all seven `032-*-SUMMARY.md` artifacts exist.
- Test infrastructure is already present in the workspace and in the phase-local fallback contract defined by [032-TEST-SPEC.md](/home/vadim/Projects/z00z/.planning/phases/032-crypto-audit-scenario-1/032-TEST-SPEC.md).
- The phase has broad automated coverage for `PH32-SEM`, `PH32-CLAIM-BIND`, `PH32-CHECKPOINT`, `PH32-SECRET`, and `PH32-HONEST`.
- `PH32-CLAIM-TRUST` remains partially open because the current canonical helper still re-derives claim membership from a synthetic one-item store contract instead of persisted storage-backed continuity.
- `PH32-SPEND` remains intentionally open because the current-stack verifier hardening did not deliver the original nullifier-semantics portion of the requirement.
- No new Nyquist test file was generated in this audit because the unresolved gaps are implementation-level requirement drift, not uncovered behavior that can be honestly closed by adding tests alone.

---

## Validation Sign-Off

- [x] All plans have `<automated>` verify commands or an equivalent phase-local release matrix
- [x] Sampling continuity is present through bootstrap-first gates, targeted release binaries, and broader reruns
- [x] Wave 0 infrastructure already covers the phase surface
- [x] No watch-mode flags were used in the validation contract
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** partial 2026-04-05

Reason: Phase 032 has strong automated coverage, but the reopened `PH32-CLAIM-TRUST`
and `PH32-SPEND` gaps prevent honest Nyquist-compliant sign-off.
