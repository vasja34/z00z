---
phase: 026
slug: crypto-audit-core
status: partial
nyquist_compliant: false
wave_0_complete: true
created: 2026-03-29
---

# Phase 026 - Validation Strategy

> 📌 Reconstructed from executed Phase 026 plan, summary, verification, and review artifacts because no prior validation file existed in this phase directory.

---

## Test Infrastructure

| Property | Value |
| -------- | ----- |
| **Framework** | Rust unit, integration, and doc tests via `cargo test` |
| **Config file** | `crates/z00z_core/Cargo.toml` |
| **Quick run command** | `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` |
| **Full suite command** | `cargo test -p z00z_core --release --features test-fast -- --nocapture` |
| **Estimated runtime** | release-mode, crate-scoped |

## Sampling Rate

- 📌 After every task commit: run `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` plus the task-specific targeted `z00z_core` test command.
- 📌 After every plan wave: run `cargo test -p z00z_core --release --features test-fast -- --nocapture`.
- 📌 Before `/gsd-verify-work`: keep the bootstrap gate plus the strongest phase-local `z00z_core` release suite green.
- 📌 Max feedback latency: bounded to one targeted `z00z_core` release validation cycle on the maintainer machine.

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
| ------- | ---- | ---- | ----------- | --------- | ----------------- | ----------- | ------ |
| 026-01-01 | 01 | 1 | PH26-ASSET-ID | integration | `cargo test -p z00z_core --test assets_tests test_registry_yaml_config_loading -- --exact --nocapture` | ✅ existing | ✅ green |
| 026-01-02 | 01 | 1 | PH26-ASSET-ID | genesis | `cargo test -p z00z_core --test genesis_tests test_asset_cross_network_id -- --exact --nocapture` | ✅ existing | ✅ green |
| 026-02-01 | 02 | 1 | PH26-REGISTRY | integration | `cargo test -p z00z_core --test assets_tests -- --nocapture` | ✅ existing | ✅ green |
| 026-03-01 | 03 | 2 | PH26-GENESIS | integration | `cargo test -p z00z_core --test genesis_tests -- --nocapture` | ✅ existing | ❌ partial |
| 026-04-01 | 04 | 3 | PH26-WIRE | unit | `cargo test -p z00z_core --lib wire_ -- --nocapture` | ✅ existing | ✅ green |
| 026-04-02 | 04 | 3 | PH26-WIRE | integration | `cargo test -p z00z_core --test assets_tests -- --nocapture` | ✅ existing | ✅ green |
| 026-05-01 | 05 | 3 | PH26-AUTH | integration | `cargo test -p z00z_core --test asset_signature_domain -- --nocapture` | ✅ existing | ✅ green |
| 026-05-02 | 05 | 3 | PH26-NONCE-FEE | unit | `cargo test -p z00z_core --lib before_epoch_fails_closed -- --nocapture` | ✅ existing | ✅ green |

📌 Status legend: ⬜ pending · ✅ green · ❌ partial/red · ⚠️ flaky

## Requirement Coverage Summary

| Requirement | Status | Evidence |
| ----------- | ------ | -------- |
| PH26-ASSET-ID | COVERED | 📌 Canonical `AssetDefinition` identity is exercised through config and genesis paths, and closure evidence is recorded in `026-01-SUMMARY.md`. |
| PH26-REGISTRY | COVERED | 📌 Registry snapshot hashing and apply semantics are covered through `assets_tests` and closure evidence in `026-02-SUMMARY.md`. |
| PH26-GENESIS | PARTIAL | 📌 Automated tests prove fail-closed missing-anchor, mismatched-anchor, weak-seed, and bad-chain rejection, but concrete protected-network anchor success cannot be verified while mainnet and testnet anchor constants remain unset. |
| PH26-WIRE | COVERED | 📌 Wire and DTO boundary behavior is exercised by inline wire tests plus `assets_tests`, including secret rejection and full rehydrate verification. |
| PH26-AUTH | COVERED | 📌 Canonical owner-message and stealth-field tamper rejection are covered by `asset_signature_domain` and supporting `assets_tests` coverage. |
| PH26-NONCE-FEE | COVERED | 📌 Fail-closed nonce helpers and canonical fee-identity checks are covered by unit tests plus `assets_tests`. |

📌 Gap analysis result: 5 requirements are `COVERED`; 1 requirement is `PARTIAL`; no `MISSING` automated requirement references were found.

## Wave 0 Requirements

📌 Existing infrastructure covers all phase requirements. No Wave 0 scaffolding is needed.

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
| -------- | ----------- | ---------- | ----------------- |
| Protected-network anchored success with concrete mainnet and testnet hashes | PH26-GENESIS | Canonical protected-network anchor values are still absent from `crates/z00z_core/src/genesis/validator.rs`, so a truthful positive anchored test cannot be generated without product-owned values. | Populate the canonical mainnet and testnet anchor hashes, then add and run a positive anchored verification case in `crates/z00z_core/tests/genesis/test_genesis_state_verification.rs` with `cargo test -p z00z_core --test genesis_tests -- --nocapture`. |

## Validation Audit 2026-03-29

| Metric | Count |
| ------ | ----- |
| Gaps found | 1 |
| Resolved | 0 |
| Escalated | 1 |

📌 `gsd-nyquist-auditor` was invoked for the single Phase 026 gap and returned `## ESCALATE`: the remaining gap is implementation or product-decision drift, not a missing test that can be filled honestly without anchor values.

## Validation Sign-Off

- [x] All tasks have `<automated>` verify coverage or existing infrastructure coverage.
- [x] Sampling continuity is preserved; no three consecutive tasks rely on missing automated verification.
- [x] Wave 0 is not required because existing infrastructure covers all phase requirements.
- [x] No watch-mode flags are part of the phase validation contract.
- [x] Feedback latency remains bounded to targeted `z00z_core` validation loops.
- [ ] `nyquist_compliant: true` is set in frontmatter.

📌 Approval: partial 2026-03-29
