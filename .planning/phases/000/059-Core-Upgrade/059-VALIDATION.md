---
phase: 059
slug: core-upgrade
status: verified
nyquist_compliant: true
wave_0_complete: true
created: 2026-06-18
validated_at: 2026-06-18
---

# Phase 059 - Validation Strategy

Final Nyquist validation record for Phase 059. This packet reconciles the
planned Phase 059 test inventory with the exact live repository homes that
landed during `059-01` through `059-10`. Several placeholder filenames from
`059-TEST-SPEC.md` and `059-TESTS-TASKS.md` were absorbed into existing
canonical test homes instead of being created as parallel files; those routes
are treated as covered, not missing.

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust workspace `cargo test` plus repository bootstrap and final verify gates |
| **Config file** | Workspace `Cargo.toml` files plus `crates/z00z_simulator/src/scenario_1/scenario_config.yaml` |
| **Quick run command** | `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` |
| **Full suite command** | `cargo test --release` |
| **Estimated runtime** | Workspace and cache dependent; long-running exact simulator lanes are recorded in `reports/full_verify-report-long-running-tests.txt` |

## Sampling Rate

- After every Rust or test-affecting change: run
  `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` first.
- After every Phase 059 execution slice: run the slice-local release suites
  from the owning `059-0X-PLAN.md`, then `cargo test --release`.
- Before final closeout: rerun bootstrap, the targeted release packet,
  `cargo test --release`, `cargo doc --release --no-deps`, and
  `./.github/skills/z00z-full-verify-gate/scripts/full_verify.sh`.
- Run `/GSD-Review-Tasks-Execution` in manual fallback mode at least three
  times and stop only after at least two consecutive passes show no
  significant issues.

## Coverage Summary

- Automated coverage exists for all ten execution-backed Phase 059 plan slices
  `059-01` through `059-10`.
- No Wave 0 stubs or framework-install work is needed; the repository already
  contains the live core, storage, wallet, runtime, rollup, and simulator test
  homes required by `059-TEST-SPEC.md` and `059-TESTS-TASKS.md`.
- No manual-only Phase 059 behaviors remain. The phase is Nyquist-compliant on
  the current live tree.

## Final Closeout Command Packet

Bootstrap-first rule was honored on the final Phase 059 tree.

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` -
  passed during final closeout and passed again in this validation pass.
- `cargo test -p z00z_core --release` - passed on the final closeout tree.
- `cargo test -p z00z_storage --release` - passed on the final closeout tree.
- `cargo test -p z00z_wallets --release` - passed on the final closeout tree.
- `cargo test -p z00z_simulator --release` - passed on the final closeout
  tree.
- `cargo test -p z00z_aggregators --release` - passed on the final closeout
  tree.
- `cargo test -p z00z_validators --release` - passed on the final closeout
  tree.
- `cargo test -p z00z_watchers --release` - passed on the final closeout tree.
- `cargo test -p z00z_rollup_node --release` - passed on the final closeout
  tree.
- `cargo test --release` - passed for the workspace on the final closeout
  tree.
- `cargo doc --release --no-deps` - passed with non-failing pre-existing
  rustdoc warnings outside the Phase 059 repository verdict.
- `./.github/skills/z00z-full-verify-gate/scripts/full_verify.sh` - passed on
  the final closeout tree and refreshed
  `reports/full_verify-report-long-running-tests.txt`.

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| `059-01-01` | `059-01` | `1` | `D-01`, `D-02`, `D-03`, `D-04`, `D-38`, `D-39`, `D-40`, `D-41`, `D-42`, `D-43` | `T-059-01`, `T-059-02`, `T-059-03` | Source-audit and no-parallel-layer rules stay explicit before code changes and keep one canonical owner path. | planning audit | `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`; `rg -n "VoucherLeaf|VoucherPolicy|ActionPoolDescriptorV1|PolicyDescriptorV1|WalletOwnedObject|live|target|migration concern" .planning/phases/059-Core-Upgrade/059-SOURCE-AUDIT.md`; `rg -n "double redeem|wrong-family proof|forced voucher acceptance|value-bearing right|cargo test --release" .planning/phases/059-Core-Upgrade/059-TEST-SPEC.md .planning/phases/059-Core-Upgrade/059-TESTS-TASKS.md` | ✅ | ✅ green |
| `059-02-01` | `059-02` | `2` | `D-05` through `D-09`, `D-44` through `D-53` | `T-059-04`, `T-059-05`, `T-059-06` | Object families, policy descriptors, voucher semantics, right zero-value rules, and native cash boundaries stay deterministic and fail closed. | unit / integration | `cargo test -p z00z_core --release test_policy_descriptor -- --nocapture`; `cargo test -p z00z_core --release test_voucher_config -- --nocapture`; `cargo test -p z00z_core --release test_rights_config -- --nocapture`; `cargo test -p z00z_wallets --release --test test_rename_guards test_test_file_prefix_guard -- --nocapture` | ✅ | ✅ green |
| `059-03-01` | `059-03` | `3` | `D-10` through `D-15`, `D-42`, `D-54` through `D-58` | `T-059-07`, `T-059-08`, `T-059-09`, `T-059-10` | One genesis boundary publishes deterministic assets, rights, policies, vouchers, and one manifest without config drift or identity collisions. | unit / integration | `cargo test -p z00z_core --release --features deterministic-rng test_genesis_manifest_phase059_fixture -- --nocapture`; `cargo test -p z00z_storage --release test_ingestion_creates_rights -- --nocapture`; `cargo test -p z00z_simulator --release runner_verify -- --nocapture` | ✅ | ✅ green |
| `059-04-01` | `059-04` | `4` | `D-16`, `D-17`, `D-18`, `D-59`, `D-60`, `D-61` | `T-059-11`, `T-059-12`, `T-059-13` | Voucher leaf families extend the existing settlement root in place and cross-family proof misuse fails closed. | unit / integration | `cargo test -p z00z_storage --release --test test_settlement_leaf --test test_hjmt_live_proof_families --test test_hjmt_batch_proof --test test_hjmt_batch_proof_negative -- --nocapture` | ✅ | ✅ green |
| `059-05-01` | `059-05` | `5` | `D-18`, `D-19`, `D-20`, `D-44` through `D-48`, `D-62`, `D-63` | `T-059-14`, `T-059-15`, `T-059-16`, `T-059-17` | Typed deltas, voucher lifecycle, value conservation, fee separation, and wallet-secret exclusion stay on the canonical storage path. | unit / integration | `cargo test -p z00z_storage --release test_model -- --nocapture`; `cargo test -p z00z_storage --release test_store_api -- --nocapture`; `cargo test -p z00z_storage --release --test test_hjmt_transition_proofs -- --nocapture`; `cargo test -p z00z_storage --release --test test_fee_envelope -- --nocapture`; `cargo test -p z00z_storage --release --features test-params-fast test_hjmt_reload_preserves_object_delta_history -- --nocapture`; `cargo test -p z00z_storage --release --features test-params-fast --test test_async_scheduler -- --nocapture` | ✅ | ✅ green |
| `059-06-01` | `059-06` | `6` | `D-30`, `D-31`, `D-32`, `D-49` through `D-53` | `T-059-18`, `T-059-19`, `T-059-20` | Aggregators carry typed evidence only, validators emit precise reject classes, watchers alert on critical object failures, and rollup surfaces stay evidence-only. | integration / runtime | `cargo test -p z00z_aggregators --release -- --nocapture`; `cargo test -p z00z_validators --release -- --nocapture`; `cargo test -p z00z_watchers --release -- --nocapture`; `cargo test -p z00z_rollup_node --release -- --nocapture` | ✅ | ✅ green |
| `059-07-01` | `059-07` | `7` | `D-21`, `D-22`, `D-23`, `D-24`, `D-43`, `D-64`, `D-65` | `T-059-21`, `T-059-22`, `T-059-23` | Wallet persistence expands to typed owned objects while asset cash projection, durable quarantine, and legacy asset compatibility remain intact. | unit / integration | `cargo test -p z00z_wallets --release redb_wallet_store -- --nocapture`; `cargo test -p z00z_wallets --release test_object_inventory -- --nocapture`; `cargo test -p z00z_wallets --release test_owned_object_tags_roundtrip -- --nocapture` | ✅ | ✅ green |
| `059-08-01` | `059-08` | `8` | `D-22`, `D-23`, `D-24`, `D-25`, `D-64`, `D-66`, `D-67`, `D-68` | `T-059-24`, `T-059-25`, `T-059-26` | Wallet RPC and package building remain family-aware, asset RPC stays cash-only, and typed object backup/import remains tamper-detecting. | unit / integration | `cargo test -p z00z_wallets --release test_wallet_service -- --nocapture`; `cargo test -p z00z_wallets --release --lib test_object_rpc_lists_typed_inventory -- --nocapture`; `cargo test -p z00z_wallets --release --lib test_asset_rpc_rejects_voucher_and_right_ids -- --nocapture`; `cargo test -p z00z_wallets --release --lib test_tx_build_rejects_voucher_inventory_id -- --nocapture`; `cargo test -p z00z_wallets --release --lib test_tx_send_rejects_right_inventory_id -- --nocapture`; `cargo test -p z00z_wallets --release test_verify_backup_detects_tamper -- --nocapture` | ✅ | ✅ green |
| `059-09-01` | `059-09` | `9` | `D-26`, `D-27`, `D-28`, `D-29`, `D-69`, `D-70`, `D-71`, `D-72` | `T-059-27`, `T-059-28`, `T-059-29` | `scenario_1` stays the only executable simulator lane and proves positive and negative Asset/Voucher/Right flows with Alice/Bob/Charlie evidence. | integration / E2E | `cargo test -p z00z_simulator --release test_scenario1_object_flows -- --nocapture`; `cargo test -p z00z_simulator --release test_scenario1_object_flows_reject_codes -- --nocapture`; `cargo test -p z00z_simulator --release --test test_scenario_settlement -- --nocapture`; `cargo test -p z00z_simulator --release --test test_hjmt_e2e -- --nocapture`; `cargo test -p z00z_simulator --release --test test_scenario1_stage_surface -- --nocapture` | ✅ | ✅ green |
| `059-10-01` | `059-10` | `10` | `D-33`, `D-34`, `D-35`, `D-36`, `D-37` | `T-059-30`, `T-059-31`, `T-059-32` | Final closeout proves that all object families, interaction classes, simulator artifacts, and docs stay synchronized on one validation story. | closeout / workspace | `cargo test -p z00z_core --release`; `cargo test -p z00z_storage --release`; `cargo test -p z00z_wallets --release`; `cargo test -p z00z_simulator --release`; `cargo test -p z00z_aggregators --release`; `cargo test -p z00z_validators --release`; `cargo test -p z00z_watchers --release`; `cargo test -p z00z_rollup_node --release`; `cargo test --release`; `cargo doc --release --no-deps`; `./.github/skills/z00z-full-verify-gate/scripts/full_verify.sh` | ✅ | ✅ green |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

## Wave 0 Requirements

Existing infrastructure covers all phase requirements.

## Manual-Only Verifications

All Phase 059 behaviors have automated verification on one canonical path.

## Validation Audit Trail

| Audit Date | Gaps Found | Resolved | Escalated | Run By |
|------------|------------|----------|-----------|--------|
| 2026-06-18 | 0 | 0 | 0 | Codex `gsd-validate-phase 059` |

## Verification Evidence

- Workspace-first discovery confirmed State B for this workflow: Phase 059
  summary artifacts existed while `059-VALIDATION.md` did not.
- Cross-read of `059-01-PLAN.md` through `059-10-PLAN.md`,
  `059-01-SUMMARY.md` through `059-10-SUMMARY.md`, `059-TEST-SPEC.md`,
  `059-TESTS-TASKS.md`, `059-EVIDENCE-LEDGER.md`, and `059-SUMMARY.md`
  showed that every Phase 059 requirement group already maps to one live test,
  simulator, or final verification home.
- Filesystem scan confirmed repository-native Rust test infrastructure and the
  expected live Phase 059 test homes across `z00z_core`, `z00z_storage`,
  `z00z_wallets`, `z00z_runtime`, `z00z_rollup_node`, and `z00z_simulator`.
- Planned placeholder names such as `test_phase059_object_flows.rs`,
  `test_phase059_object_alerts.rs`, and split genesis test files were not
  treated as gaps because the executed phase closed those requirements in the
  canonical live homes `test_scenario1_object_flows.rs`,
  `test_object_alerts.rs`, `test_object_policy_verdicts.rs`, and
  `test_genesis_suite.rs`, as recorded in `059-EVIDENCE-LEDGER.md`.
- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` passed on
  the current live tree in this validation pass.
- The final closeout packet already records green release-mode evidence for the
  targeted package reruns, full `cargo test --release`,
  `cargo doc --release --no-deps`, and
  `./.github/skills/z00z-full-verify-gate/scripts/full_verify.sh`.

## Review Loop

Manual fallback for `/.github/prompts/gsd-review-tasks-execution.prompt.md`
was used during Phase 059 closeout because the slash prompt is not a callable
tool in this environment.

- Pass 1 found real closeout issues in the canonical verify gate:
  rustfmt drift in `stage4_support.rs` and a clippy needless-borrow reject in
  `test_scenario1_stage_surface.rs`.
- Pass 2 reran `full_verify.sh`, re-audited the phase packet, and found no
  significant issues.
- Pass 3 repeated the same audit after summary and planning-state sync and
  found no significant issues.

Two consecutive clean review passes were achieved on passes 2 and 3.

## Validation Sign-Off

- [x] All ten Phase 059 execution tasks have automated verification recorded
- [x] All plan-owned release suites are reflected in the slice summaries or in
  this phase validation packet
- [x] No manual-only Phase 059 behaviors remain
- [x] Bootstrap-first rule was honored on the final closeout tree
- [x] The final closeout release suites are green
- [x] `cargo test --release` passed on the final closeout tree
- [x] `cargo doc --release --no-deps` passed on the final closeout tree with
  only non-failing pre-existing warnings outside verdict scope
- [x] `nyquist_compliant: true` is set in frontmatter

**Approval:** verified 2026-06-18
