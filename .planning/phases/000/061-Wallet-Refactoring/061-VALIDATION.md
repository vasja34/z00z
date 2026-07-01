---
phase: 061
slug: wallet-refactoring
status: verified
nyquist_compliant: true
wave_0_complete: true
created: 2026-06-24
validated_at: 2026-06-24
---

# Phase 061 - Validation Strategy

Final Nyquist validation record for Phase 061. This packet reconstructs the
validation map from `061-01` through `061-10`, `061-TODO.md`,
`061-CONTEXT.md`, and the executed summaries on the live one-level wallet tree.
It records the shipped canonical paths only and does not introduce a parallel
module, test, or planning lane.

## ✅ Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust workspace `cargo test` plus repository bootstrap, targeted guardrails, and structural grep/find audits |
| **Config file** | Workspace `Cargo.toml` files plus `crates/z00z_wallets/config/wallet_config.yaml`, `crates/z00z_wallets/schemas/redb-schema.yaml`, and the canonical wallet docs/assets under `crates/z00z_wallets/docs/` |
| **Quick run command** | `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` |
| **Full suite command** | `cargo test --release` |
| **Estimated runtime** | Long-running; the wallet crate, simulator, and broad workspace release lane dominate wall-clock time |

## ✅ Sampling Rate

- After every Rust or test-affecting change: run
  `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` first.
- After every Phase 061 execution slice: run the slice-local release and
  structural packet recorded in the owning `061-0X-SUMMARY.md`, then
  `cargo test --release` when the slice touches Rust, tests, or shared runtime
  behavior.
- Before final closeout: rerun bootstrap, the slice-local packet, the targeted
  guardrails when the summary calls for them, and the broad
  `cargo test --release` lane on the same live tree.
- Run `/GSD-Review-Tasks-Execution` in manual fallback mode at least three
  times and stop only after at least two consecutive passes show no
  significant issues.

## ✅ Coverage Summary

- Automated coverage exists for all ten execution-backed Phase 061 plan slices
  `061-01` through `061-10`.
- Gap analysis result: `10 covered / 0 partial / 0 missing`.
- No Wave 0 stubs or framework-install work is needed; the repository already
  contains the live wallet, storage, simulator, and documentation homes needed
  by the phase.
- No manual-only Phase 061 behaviors remain. The phase is Nyquist-compliant on
  the current live tree.

## ✅ Final Closeout Command Packet

- Bootstrap-first discipline was preserved across all ten slice summaries.
- `061-10-SUMMARY.md` records the final current-tree closeout packet as green:
  `cargo fmt --all --check`,
  `cargo check --release -p z00z_wallets --all-targets --all-features`,
  `cargo test --release -p z00z_wallets --all-targets --all-features`,
  `cargo test --release -p z00z_simulator --lib`,
  `cargo test --release`,
  `cargo test --release -p z00z_wallets --test test_rename_guards`, and
  `cargo test --release -p z00z_storage --test test_live_guardrails`.
- `061-09-SUMMARY.md` records the only bootstrap failure during execution on
  the live tree, caused by a stale `#[path]` target in
  `src/tx/asset_selector_multi.rs`; the fix landed in the same slice and the
  rerun went green before execution continued.
- `061-04-SUMMARY.md` and `061-06-SUMMARY.md` record real post-move regressions
  that were corrected in-slice and then rerun green on the same canonical
  wallet tree.

## ✅ Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| `061-01-01` | `061-01` | `0` | `061-D01`, `061-D06`, `061-D07`, `061-D09`, `061-D13`, `061-D14` | `061-01 threat model` | Live-tree drift, delete candidates, and source anchors are frozen before any structural move begins. | planning audit / diagnostics | `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`; `cargo check --release -p z00z_wallets --all-targets --all-features`; baseline TODO/tree compare audits plus `git diff --check` as recorded in `061-01-SUMMARY.md` | ✅ | ✅ green |
| `061-02-01` | `061-02` | `1` | `061-D02`, `061-D03`, `061-D04`, `061-D05`, `061-D06`, `061-D10`, `061-D14` | `061-02 threat model` | Shared `db` helpers flatten and `wallet_store_crypto*` becomes the single Rust path while persisted label space stays unchanged. | unit / integration / diagnostics | `cargo check --release -p z00z_wallets --all-targets --all-features`; `cargo test --release -p z00z_wallets --all-targets --all-features`; stale `db::redb_wallet_crypto` grep and persisted-domain-string grep from `061-02-SUMMARY.md` | ✅ | ✅ green |
| `061-03-01` | `061-03` | `2` | `061-D02`, `061-D03`, `061-D04`, `061-D05`, `061-D06`, `061-D10`, `061-D14` | `061-03 threat model` | The concrete RedB backend moves to `src/redb_store` while `db::redb_wallet_store` and its include anchors stay canonical. | integration / anchor audit | `cargo check --release -p z00z_wallets --all-targets --all-features`; `cargo test --release -p z00z_wallets --all-targets --all-features`; `find crates/z00z_wallets/src -type f -path '*/redb_wallet_store/*'`; anchor greps from `061-03-SUMMARY.md` | ✅ | ✅ green |
| `061-04-01` | `061-04` | `3` | `061-D02`, `061-D03`, `061-D06`, `061-D08`, `061-D11`, `061-D14` | `061-04 threat model` | RPC support files flatten under `src/rpc`, `adapters::rpc` remains the facade, and `wallet_config.yaml` has one canonical non-`src/` authority path. | integration / config-anchor audit | `cargo check --release -p z00z_wallets --all-targets --all-features`; `cargo test --release -p z00z_wallets --all-targets --all-features`; `find crates/z00z_wallets/src/rpc -maxdepth 1 -type f`; config-anchor grep packet from `061-04-SUMMARY.md` | ✅ | ✅ green |
| `061-05-01` | `061-05` | `4` | `061-D02`, `061-D03`, `061-D06`, `061-D07`, `061-D08`, `061-D12`, `061-D14` | `061-05 threat model` | WalletService and AppService internals flatten without creating a second wrapper layer or breaking service source-anchor tests. | integration / service-anchor audit | `cargo check --release -p z00z_wallets --all-targets --all-features`; `cargo test --release -p z00z_wallets --all-targets --all-features`; `find crates/z00z_wallets/src/services -maxdepth 1 -type f`; stale service-path grep from `061-05-SUMMARY.md` | ✅ | ✅ green |
| `061-06-01` | `061-06` | `5` | `061-D02`, `061-D03`, `061-D06`, `061-D11`, `061-D14` | `061-06 threat model` | RPC method implementations flatten to one-level `method_*` and `test_*` files while helper naming and registration entrypoints stay canonical. | integration / RPC regression | `cargo check --release -p z00z_wallets --all-targets --all-features`; `cargo test --release -p z00z_wallets --test test_rpc_logging_replay_audit -- --nocapture`; `cargo test --release -p z00z_wallets --all-targets --all-features`; stale `_rpc_` residue grep and retired-method-tree `find` from `061-06-SUMMARY.md` | ✅ | ✅ green |
| `061-07-01` | `061-07` | `6` | `061-D02`, `061-D03`, `061-D06`, `061-D08`, `061-D09`, `061-D14` | `061-07 threat model` | Receiver, persistence, and security-vault files become one-level and no duplicate storage or password-asset authority remains under `src/`. | integration / security-asset audit | `cargo check --release -p z00z_wallets --all-targets --all-features`; `cargo test --release -p z00z_wallets --all-targets --all-features`; nested receiver/persistence `find`; password-asset home `find`; stale-path grep from `061-07-SUMMARY.md` | ✅ | ✅ green |
| `061-08-01` | `061-08` | `7` | `061-D02`, `061-D03`, `061-D06`, `061-D08`, `061-D14` | `061-08 threat model` | The key tree is flat, `.inc.rs` and `#[path]` anchors still resolve, and key docs no longer depend on a nested `src/` asset layout. | unit / integration / doc-anchor audit | `cargo check --release -p z00z_wallets --all-targets --all-features`; `cargo test --release -p z00z_wallets --all-targets --all-features`; `cargo test --release -p z00z_wallets --test test_rename_guards --test test_key_manager --test test_seed_salt_policy --test test_bip44`; key-tree grep and nested-file `find` from `061-08-SUMMARY.md` | ✅ | ✅ green |
| `061-09-01` | `061-09` | `8` | `061-D02`, `061-D03`, `061-D05`, `061-D06`, `061-D08`, `061-D14` | `061-09 threat model` | Tx, claim, stealth, wallet, backup, and chain leaf domains are one-level, helper splits are canonical, and wallet semantics stay unchanged. | integration / closeout-audit | `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`; `cargo check --release -p z00z_wallets --all-targets --all-features`; `cargo test --release -p z00z_wallets --all-targets --all-features`; targeted guardrails from `061-09-SUMMARY.md` plus stale helper grep and nested-file `find` | ✅ | ✅ green |
| `061-10-01` | `061-10` | `9` | `061-D02`, `061-D06`, `061-D08`, `061-D09`, `061-D14` | `061-10 threat model` | Final cleanup proves that no nested Rust or non-Rust residue remains under `crates/z00z_wallets/src` and that the canonical closeout tree is green end to end. | workspace closeout / structural audit | `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`; `cargo fmt --all --check`; `cargo check --release -p z00z_wallets --all-targets --all-features`; `cargo test --release -p z00z_wallets --all-targets --all-features`; `cargo test --release -p z00z_simulator --lib`; `cargo test --release`; targeted guardrails and structural checks from `061-10-SUMMARY.md` | ✅ | ✅ green |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ partial*

## ✅ Wave 0 Requirements

Existing infrastructure covers all phase requirements.

## ✅ Manual-Only Verifications

All Phase 061 behaviors have automated verification on one canonical path.

The manual fallback for `/GSD-Review-Tasks-Execution` does not create a Nyquist
gap because each execution summary already records at least three review passes
and two consecutive clean passes before the phase advanced.

## ✅ Validation Audit Trail

| Audit Date | Gaps Found | Resolved | Escalated | Run By |
|------------|------------|----------|-----------|--------|
| 2026-06-24 | 0 | 0 | 0 | Codex `gsd-validate-phase 061` |

## ✅ Verification Evidence

- Workflow state was reconstructed as State B: the Phase 061 execution
  summaries already existed while `061-VALIDATION.md` did not.
- Cross-read of `061-01-PLAN.md` through `061-10-PLAN.md`,
  `061-01-SUMMARY.md` through `061-10-SUMMARY.md`, `061-TODO.md`,
  `061-CONTEXT.md`, `061-SECURITY.md`, `.planning/STATE.md`, and
  `.planning/ROADMAP.md` showed that every plan-owned requirement group is
  reflected in an executed summary and in a live validation command packet.
- Filesystem scan confirmed repository-native Rust test infrastructure. The
  incidental `.temp/sovereign-sdk-dev/typescript/vitest.config.ts` result is
  outside Phase 061 scope and is not part of the live validation path.
- `061-01` closed the planner drift before code motion: the missing-old-path
  audit and the nested-path-uncovered audit both resolved to `0` after the
  canonical TODO corrections recorded in `061-01-SUMMARY.md`.
- `061-04` closed the stale `src/wallet_config.yaml` anchor by switching the
  affected env-setup helper to
  `crate::wallet_config_support::default_wallet_config_path()`, then reran the
  bootstrap and full wallet release packet green.
- `061-06` corrected the RPC audit-script path and reran the targeted
  `test_rpc_logging_replay_audit` plus the full wallet release packet green on
  the same canonical tree.
- `061-09` hit a real stale `#[path]` target in
  `src/tx/asset_selector_multi.rs`, fixed it in-slice, reran
  `cargo test --release -p z00z_wallets --lib --tests --no-run`, and then
  reran the bootstrap gate green before continuing.
- `061-10` recorded the broad live-tree closeout packet as green, including
  the full workspace `cargo test --release`, targeted rename/live guardrails,
  and the final no-nested-Rust / no-non-Rust-under-`src` structural proof.
- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` was
  rerun in this validation pass on `2026-06-24` and finished green with
  `skip z00z_wallets examples: no example targets` followed by
  `=== BOOTSTRAP COMPLETE ===`.
- Gap analysis found no `MISSING` or `PARTIAL` plan-owned behaviors, so no
  separate `gsd-nyquist-auditor` spawn was required.

## ✅ Review Loop

Manual fallback for `/.github/prompts/gsd-review-tasks-execution.prompt.md`
was used throughout Phase 061 because the slash prompt is not a callable tool
in this runtime.

- `061-01` through `061-07` each record at least three review passes and
  explicit consecutive clean closure after real issue fixes.
- `061-08`, `061-09`, and `061-10` explicitly record three manual review
  passes each, with passes 2 and 3 consecutive and clean.
- This validation pass found no newly uncovered behavior outside the already
  summary-backed closeout packet, so no new review escalation loop was needed.

## ✅ Validation Sign-Off

- [x] All ten Phase 061 execution tasks have automated verification recorded
- [x] All slice-level verify packets are reflected in the summaries or in this
  validation artifact
- [x] No Wave 0 work is required
- [x] No plan-owned manual-only behaviors remain
- [x] Bootstrap-first and release-mode discipline are reflected across the
  phase validation contract
- [x] The final closeout release suites and structural guards are recorded on
  one canonical live tree
- [x] `nyquist_compliant: true` is set in frontmatter

**Approval:** verified 2026-06-24
