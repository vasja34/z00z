---
phase: 031
slug: refactor-architecture
status: audited
nyquist_compliant: true
wave_0_complete: true
created: 2026-04-05
---

# Phase 031 — Validation Strategy

> Reconstructed Nyquist validation contract for Phase 031 from completed plan, summary, and verification artifacts.

---

## Test Infrastructure

| Property | Value |
| -------- | ----- |
| **Framework** | Cargo integration tests plus `rg` guard commands and repository bootstrap checks |
| **Config file** | Workspace Cargo configuration in [Cargo.toml](/home/vadim/Projects/z00z/Cargo.toml) |
| **Quick run command** | `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` |
| **Full suite command** | `./.github/skills/z00z-full-verify-gate/scripts/full_verify.sh --max-safe-run` |
| **Estimated runtime** | ~600+ seconds for the full release-style sweep, workspace-dependent |

---

## Sampling Rate

- **After every task commit:** Run `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- **After every plan wave:** Run the plan-scoped release commands recorded in each `031-XX-PLAN.md`
- **Before `/gsd-verify-work`:** `./.github/skills/z00z-full-verify-gate/scripts/full_verify.sh --max-safe-run` must be green
- **Max feedback latency:** bounded by the quick bootstrap plus targeted crate tests for the active plan

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
| ------- | ---- | ---- | ----------- | ---------- | --------------- | --------- | ----------------- | ----------- | ------ |
| 031-01-01 | 01 | 0 | PH31-INV | G-00 | Retirement-sensitive seams are inventoried before any narrowing starts | bootstrap + grep | bootstrap script plus root-surface inventory guards | ✅ | ✅ green |
| 031-01-02 | 01 | 0 | PH31-INV | G-00 | Import graph proves caller ownership and wave ordering before refactor execution | integration + grep | core assets suite plus crypto hash policy plus import-graph guards | ✅ | ✅ green |
| 031-02-01 | 02 | 1 | PH31-CORE | G-01 | `z00z_core` root is curated and free of wildcard stable exports | integration + grep | bootstrap script plus core wildcard-export guard plus assets and genesis suites | ✅ | ✅ green |
| 031-02-02 | 02 | 1 | PH31-CORE | D-36 | Asset JSON import stays fail-closed and asset-owned with explicit size caps | integration + example | bootstrap script plus bounded JSON guards plus core examples and wire-pkg bounds test | ✅ | ✅ green |
| 031-03-01 | 03 | 1 | PH31-CRYPTO | G-02 | Stable crypto facade stays Z00Z-owned while vendor contracts move to explicit non-default lanes | integration + grep | bootstrap script plus vendor-leak guard plus crypto public-surface suites | ✅ | ✅ green |
| 031-03-02 | 03 | 1 | PH31-CRYPTO | D-10 | Test-only AEAD nonce helpers are unreachable from ship-ready production profiles | integration + grep | bootstrap script plus AEAD gating guard plus exact public-surface and edge-case tests | ✅ | ✅ green |
| 031-04-01 | 04 | 1 | PH31-NET | D-11 to D-14 | `z00z_networks_rpc` remains transport-only and names auth or lifecycle seams as external | crate test + grep | bootstrap script plus RPC boundary guard plus RPC crate release suite | ✅ | ✅ green |
| 031-04-02 | 04 | 1 | PH31-NET | Phase 115 placeholder boundary | `onionnet` is a documented node-owned overlay placeholder aligned with the reserved module map | crate test + grep | bootstrap script plus OnionNet ownership guard plus onionnet release suite | ✅ | ✅ green |
| 031-05-01 | 05 | 2 | PH31-WLT-SEAMS | G-03 / D-28 | Wallet service root is explicit and no longer `include!`-assembled | release test + grep | bootstrap script plus include-removal guard plus service split and wallet error suites | ✅ | ✅ green |
| 031-05-02 | 05 | 2 | PH31-WLT-SEAMS | root-facade caller proof | Service-shape regressions validate the named seam map instead of the legacy monolith | release test | bootstrap script plus phase30 split suite plus wallet service error suite plus wallet release suite | ✅ | ✅ green |
| 031-05-03 | 05 | 2 | PH31-WLT-SEAMS | measurement obligation | Review-surface and compile-friction measurement remains documented rather than implied | doc guard + release test | bootstrap script plus summary measurement guard plus service split suite | ✅ | ✅ green |
| 031-06-01 | 06 | 2 | PH31-WLT-ID | G-04 / D-40 | Persisted wallet identity remains authoritative for path and bytes reopen lanes | exact integration tests | bootstrap script plus exact wallet open and persisted-chain anchors | ✅ | ✅ green |
| 031-06-02 | 06 | 2 | PH31-WLT-ID | G-05 | `lock_wallet` rejects unauthenticated transport callers and stays session-bound | exact integration test + grep | bootstrap script plus exact RPC auth denial anchor plus lock-wallet auth guard | ✅ | ✅ green |
| 031-07-01 | 07 | 2 | PH31-WLT-RPC | D-31 | Wallet DTOs and dispatcher wiring remain edge-owned inside adapters or RPC types | release test | bootstrap script plus RPC type serialization and dispatcher roundtrip suites | ✅ | ✅ green |
| 031-07-02 | 07 | 2 | PH31-WLT-RPC | root export inventory | Wallet root no longer widens transport aliases into the stable facade | release test + grep | bootstrap script plus wallet root export guard plus RPC persistence suite | ✅ | ✅ green |
| 031-07-03 | 07 | 2 | PH31-WLT-RPC | D-41 / M-35 | Wallet-core transaction assembly no longer owns ad hoc RNG or generic JSON representation drift | release test + negative grep | bootstrap script plus RNG and JSON negative guards plus persistence and wallet release suites | ✅ | ✅ green |
| 031-07-04 | 07 | 2 | PH31-WLT-RPC | D-32 / M-31 | Named compatibility lanes are explicitly dispositioned before Wave 4 retirement | release test + grep | bootstrap script plus compatibility-lane guard plus receiver-card and RPC wiring suites | ✅ | ✅ green |
| 031-08-01 | 08 | 3 | PH31-STORAGE | G-06 / D-37 | Canonical checkpoint seal path is statement-bound and rejects missing replay evidence | release test + grep | bootstrap script plus checkpoint proof guard plus store, finalization, and draft-final suites | ✅ | ✅ green |
| 031-08-02 | 08 | 3 | PH31-STORAGE | replay and rehydrate map | Reload and rehydrate stay storage-owned and reject attested proof drift without wallet-shaped leakage | release test + negative grep | bootstrap script plus rehydrate guard plus exact proof-drift anchor plus storage fast suite | ✅ | ✅ green |
| 031-08-03 | 08 | 3 | PH31-STORAGE | D-38 | Validation map is assigned to storage-owned files while `ClaimNullifier` remains a bounded non-finding | release test + cross-crate grep | bootstrap script plus validation-map guard plus store, root-binding, replay-bound, and rehydrate suites | ✅ | ✅ green |
| 031-09-01 | 09 | 3 | PH31-SIM | G-08 | Default Stage 2 contract does not expose plaintext wallet secret artifacts | release test + runtime | bootstrap script plus exact secret-output anchors plus simulator runtime release suite | ✅ | ✅ green |
| 031-09-02 | 09 | 3 | PH31-SIM | G-07 / G-09 | Simulator uses stable facades only and rejects recursive cleanup outside the sandbox root | release test + grep | bootstrap script plus simulator boundary guards plus architecture and sandbox-reset suites | ✅ | ✅ green |
| 031-10-01 | 10 | 4 | PH31-UTILS | G-10 | `z00z_utils` ends the phase with an explicit README-level admission policy | release test + doc grep | bootstrap script plus utils boundary-note guard plus utils release suite | ✅ | ✅ green |
| 031-10-02 | 10 | 4 | PH31-CLOSEOUT | G-10 | Retirement and planning truth close only on caller-proof-backed green validation | release sweep + grep | bootstrap script plus wallet and simulator release suites plus full-verify and retirement guards | ✅ | ✅ green |

Status legend: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky

---

## Wave 0 Requirements

Existing infrastructure covers all phase requirements.

No new framework installation, fixture layer, or test harness bootstrap was required beyond the repository-standard bootstrap script, targeted cargo tests, grep guards, and the canonical full-verify gate.

---

## Manual-Only Verifications

All phase behaviors have automated verification.

The original `/GSD-Review-Tasks-Execution` prompt-runner was unavailable in this executor environment, but each affected summary records the substitution path and the release-style command evidence used instead.

---

## Validation Sign-Off

- [x] All tasks have automated verify coverage or an equivalent command-backed proof path
- [x] Sampling continuity: no 3 consecutive tasks without automated verify evidence
- [x] Wave 0 inventory and import-graph proof cover the prerequisite seam map
- [x] No watch-mode flags are required for phase verification
- [x] Feedback latency stays bounded by bootstrap plus targeted release tests for the active plan slice
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** approved 2026-04-05

---

## Reconstruction Notes

This file was reconstructed under validate-phase State B from:

- `031-01-PLAN.md` through `031-10-PLAN.md`
- `031-01-SUMMARY.md` through `031-10-SUMMARY.md`
- `031-VERIFICATION.md`
- `.planning/ROADMAP.md`
- `.planning/REQUIREMENTS.md`

Gap audit result: **0 missing automated coverage gaps** detected for Phase 031.

No new test files were generated because each requirement already maps to existing cargo-test, grep-guard, bootstrap, or full-verify evidence recorded in the phase artifacts.
