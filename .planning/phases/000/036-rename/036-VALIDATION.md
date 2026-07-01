---
phase: 036
slug: rename
status: partial
nyquist_compliant: false
wave_0_complete: true
created: 2026-04-21
---

# Phase 036 - Validation Strategy

> Reconstructed Nyquist validation contract for Phase 036 from the executed plan and summary artifacts in `.planning/phases/036-rename/`. The phase remains open because `036-20` is still the separate partial shim-removal boundary, while `036-24` is summary-backed complete.

## Test Infrastructure

| Property | Value |
| -------- | ----- |
| **Framework** | Rust release-mode integration tests plus repository bootstrap checks and exact `rg` residue scans |
| **Config file** | `Cargo.toml` |
| **Quick run command** | `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` |
| **Full suite command** | `cargo test --release --features test-fast --features wallet_debug_dump` |
| **Estimated runtime** | workspace-dependent; release-style sweep |

## Sampling Rate

- After every task commit: run `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`.
- After every plan wave: run the strongest plan-local release command recorded in the relevant `036-XX-SUMMARY.md`.
- Before `/gsd-verify-work`: run the strongest available release suite for the currently active continuation and confirm the residue scan still matches the recorded boundary.
- Max feedback latency: bounded by the bootstrap gate plus the targeted release tests for the active slice.

## Evidence Snapshot

- `036-04` through `036-10` remain summary-backed complete as the fixed serial embedded-versioning chain. Their closure is supported by the bootstrap gate plus the phase-local release sweep recorded in the live summaries.
- `036-11` through `036-18` remain summary-backed complete as the legacy-removal continuation. Their closure is supported by the same bootstrap/release discipline plus the recorded blocker carry-forward and residual scans.
- `036-19` is summary-backed complete as the a4 rename matrix closeout through row 814.
- `036-20` is summary-backed partial. The focused old-name shim slice is green, but the broader constructor/storage tail still has live survivors that must not be hidden behind a false closeout.
- `036-21` is summary-backed complete as the attribute-audit continuation, with the embedded matrix and the intended NARROW/KEEP outcomes proved by bootstrap plus targeted wallet and simulator tests.
- `036-22` is summary-backed complete as the hash-domain and KDF-salt continuation, with the row-owned owner/runtime/golden sweep and residue scans completed green.
- `036-23` is summary-backed complete as the claim-contract/root-version continuation, with the claim hash regression, wallet/storage/simulator tests, and exact old-name scans completed green.
- `036-24` is summary-backed complete. The path-group rehome landed in `036-24-SUMMARY.md`, so the validation file must not regress to the planned-only state.

## Requirement Coverage Summary

| Continuation | Status | Evidence |
| ------------ | ------ | -------- |
| `036-04..036-10` | COVERED | Summary-backed serial versioning chain with bootstrap and release-style validation. |
| `036-11..036-18` | COVERED | Summary-backed legacy-removal chain with replayed cleanup and residual scans. |
| `036-19` | COVERED | Truthful rename matrix closeout through row 814. |
| `036-20` | PARTIAL | Old-name shim slice is green, but the broader constructor/storage tail still has live survivors. |
| `036-21` | COVERED | Embedded attribute matrix closed with the intended NARROW and KEEP outcomes. |
| `036-22` | COVERED | Non-Tari hash-domain and KDF-salt normalization swept clean. |
| `036-23` | COVERED | Claim-contract and raw root-version migration swept clean. |
| `036-24` | COVERED | Path-group rehome is summary-backed complete and recorded in `036-24-SUMMARY.md`. |

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
| ------- | ---- | ---- | ----------- | ---------- | --------------- | --------- | ----------------- | ----------- | ------ |
| `036-04..036-10` | `04-10` | serial | Embedded versioning baseline | — | Fixed row-exact execution stays within the live authority chain. | bootstrap + release | Bootstrap gate plus the phase-local release sweep recorded in the summaries. | ✅ summary-backed | ✅ green |
| `036-11..036-18` | `11-18` | legacy-removal | Legacy-removal continuation | — | Legacy owners and local residue were removed without reopening the live versioning chain. | bootstrap + release + residue scan | Bootstrap gate plus the recorded legacy-removal reruns and residual scans. | ✅ summary-backed | ✅ green |
| `036-19` | `19` | rename-matrix | A4 rename matrix | — | Rename authority stays truthful through the recorded row 814 closeout. | row audit + release | The row-814 audit and the recorded release-style validation from the summary. | ✅ summary-backed | ✅ green |
| `036-20` | `20` | shim-removal | A4 shim-removal boundary | — | Old-name shim callers are gone, but the broader constructor/storage tail still requires an explicit survivor decision. | bootstrap + release + rg | `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`, `cargo test --release --features test-fast --features wallet_debug_dump`, and the exact residual scans for `recv_one(` / `decode_ristretto_pk` / `build_stealth_sender_leaf` / `build_stealth_bundle*` / `derive_address` plus `from_canonical_bytes` / `CompatRoot` / `SeqSecureRngProvider`. | ✅ summary-backed partial | ⚠️ partial |
| `036-21` | `21` | attribute-audit | Attribute audit | — | Embedded attribute rows close with the intended NARROW and KEEP outcomes only. | bootstrap + targeted release tests | Bootstrap gate plus the wallet, storage, simulator, and RPC release tests recorded in the summary. | ✅ summary-backed | ✅ green |
| `036-22` | `22` | hash-domain | Hash-domain continuation | — | Non-Tari hash domains and adjacent KDF salts use the canonical live namespace. | bootstrap + targeted release tests + rg | Bootstrap gate plus the crypto, wallet, and residue-scan commands recorded in the summary. | ✅ summary-backed | ✅ green |
| `036-23` | `23` | claim-contract | Claim-contract continuation | — | `ClaimRootVer` stays retired; `claim_contract` and raw-byte `root_version` remain canonical. | bootstrap + targeted release tests + exact rg | `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`, the claim-contract wallet/storage/simulator release tests, and the exact identifier-exact residue scans recorded in the summary. | ✅ summary-backed | ✅ green |
| `036-24` | `24` | path-group | A7 path-group continuation | T-036-24-01 / T-036-24-02 / T-036-24-03 | The remaining crypto module families are rehomed only through canonical directory-backed roots, and the summary must keep `036-20` separate. | lib test + rg + review passes | `cargo test -p z00z_crypto --release --features test-fast --lib`, the plan-local `rg` checks, and the repeated review passes recorded in `036-24-SUMMARY.md`. | ✅ summary-backed | ✅ green |

## Wave 0 Requirements

Existing infrastructure covers all completed phase groups.

No new framework installation or fixture layer was required beyond the repository bootstrap script, targeted cargo tests, and the exact residue scans recorded in the executed summaries.

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
| -------- | ----------- | ---------- | ----------------- |
| `036-20` survivor disposition for `CompatRoot` and `SeqSecureRngProvider` | `036-20` | The automated scans prove the residue is real; the closure decision itself must be made explicitly in a later wave or review. | Re-read `036-20-SUMMARY.md` and the recorded residue scans before deciding whether the remaining constructor tail is retired or kept. |

## Open Gaps And Watchpoints

- `036-24` is execution-backed and summary-backed complete, so the path-group continuation now has recorded evidence in `036-24-SUMMARY.md`.
- The workspace-wide release suite still reaches the unrelated `z00z_crypto/tests/test_h2scalar.rs::test_h2scalar_golden_vectors` failure recorded in `036-23-SUMMARY.md`; that blocker is outside the Phase 036 a7 continuation surface.
- `036-20` remains the authoritative partial shim-removal boundary and must not be collapsed into the a7 rehome wave.

## Validation Sign-Off

- [x] Existing infrastructure detected and reused
- [x] Completed phase groups have command-backed evidence
- [x] No watch-mode flags are required
- [ ] All tasks have fully automated verification
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify evidence
- [x] `036-20` is still called out as a separate partial boundary
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** partial 2026-04-21

## Reconstruction Notes

This file was reconstructed under validate-phase State B from:

- `.planning/ROADMAP.md`
- `.planning/phases/036-rename/036-CONTEXT.md`
- `.planning/phases/036-rename/036-20-SUMMARY.md`
- `.planning/phases/036-rename/036-21-SUMMARY.md`
- `.planning/phases/036-rename/036-22-SUMMARY.md`
- `.planning/phases/036-rename/036-23-SUMMARY.md`
- `.planning/phases/036-rename/036-24-PLAN.md`
- the validation-file patterns used by earlier completed phases

Gap audit result: no missing automated coverage gaps were found inside the completed live continuation chain, but the phase remains partial because `036-20` is still open.

No new test files were generated because the existing summaries already provide the needed cargo-test, bootstrap, and residue-scan evidence for the completed slices.

---
*Phase: 036-rename*
*Reconstructed: 2026-04-21*
