---
phase: 060-Gaps-Closing
plan: 060-08
status: complete
completed_at: 2026-06-20
next_plan: 060-09
summary_artifact_for: .planning/phases/060-Gaps-Closing/060-08-PLAN.md
---

# 060-08 Summary: `validator_mandate_lock_v1` Contract And Fail-Closed Profile Coverage

## Completed Scope

`060-08` is complete for the `validator_mandate_lock_v1` lock-profile and
fail-closed coverage slice.

The repository now carries one shared `validator_mandate_lock_v1` contract on
top of the live `RightClass::ValidatorMandate` object model. The lock profile
is specified in `crates/z00z_wallets/src/wallet/WALLET-GUIDE.md` as a proposed
Phase 060 profile id layered on the existing live right class, and the guide
now states the exact v1 grammar fields:
`holder_commitment`, `control_commitment`, `beneficiary_commitment`,
`payload_commitment`, `valid_from`, `valid_until`, `challenge_from`,
`challenge_until`, `use_nonce`, `transition_policy_id`,
`revocation_policy_id`, `disclosure_policy_id`, and
`retention_policy_id`. The guide also now states the spend rule, approved
unlock or redelegate transitions, reward-claim relation, and explicit v1
non-goals.

This slice also lands one canonical payload-binding helper contract in
`crates/z00z_wallets/src/tx/spend_rules.rs`. The wallet now derives the lock
payload commitment from the bound asset id, bound amount, validity and
challenge windows, nonce, and policy ids, and reuses the same helper across
wallet filtering, test fixtures, validator tests, and simulator tests. The
profile tag stays wallet-local, but the lock semantics now sit on one coherent
code path rather than separate ad hoc checks.

Ordinary spend is now fail-closed at the wallet build and send boundary.
`WalletService::list_spendable_asset_rows(...)` filters out asset rows that are
matched by active `validator_mandate_lock_v1` rights, so `tx.build` and
`tx.send` no longer treat locked assets as ordinary spendable balance. This
behavior remains narrow: unrelated assets stay selectable, and the slice adds
explicit coverage proving that a lock does not freeze the entire wallet or
definition lane.

The validator and simulator boundaries now enforce the approved unlock grammar
more strictly. The key semantic fix is in
`crates/z00z_storage/src/settlement/object_package_contract.rs`: if an action
explicitly declares `Right` as an input family and also requires a
`RightReference(...)`, the package must actually carry a right input delta.
That closes the bypassed-unlock hole where an unlock-like asset transition
could previously present a right witness state without actually consuming or
updating the lock right. The narrow contract keeps voucher witness-only flows
intact because those actions do not declare `Right` as an input family.

Validator, watcher, and simulator coverage now agree on the same fail-closed
story:

- locked ordinary spend rejects when the lock right is missing;
- unlock after expiry is accepted only on the approved contract;
- unlock without consuming the lock right rejects as `OBJECT_MISSING_RIGHT`;
- unlock replay without replay nonce rejects as `OBJECT_REPLAY`;
- watcher severity stays `Critical` for replay and `Warn` for right expiry;
- simulator matrix rows and reject-code coverage now include the lock-unlock
  success and failure paths.

## Files Changed

- `.planning/phases/060-Gaps-Closing/060-08-SUMMARY.md`
- `.planning/ROADMAP.md`
- `.planning/STATE.md`
- `crates/z00z_storage/src/settlement/object_package_contract.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/test_tx_send_body.rs`
- `crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_receive.rs`
- `crates/z00z_wallets/src/test_support/owned_objects.rs`
- `crates/z00z_wallets/src/tx/mod.rs`
- `crates/z00z_wallets/src/tx/spend_rules.rs`
- `crates/z00z_wallets/src/wallet/WALLET-GUIDE.md`
- `crates/z00z_runtime/validators/tests/test_object_policy_verdicts.rs`
- `crates/z00z_runtime/watchers/tests/test_object_alerts.rs`
- `crates/z00z_simulator/src/scenario_1/scenario_config.yaml`
- `crates/z00z_simulator/tests/test_scenario1_object_flows.rs`

## Boundary Kept

- No new primitive leaf was introduced.
- No second wallet authority plane, no second unlock store, and no parallel
  typed-object logic layer were introduced.
- `validator_mandate_lock_v1` remains a proposed Phase 060 profile id layered
  on live `validator_mandate`; it was not promoted into a fake new protocol
  class.
- The new storage-side guard is narrow to consumptive right-input actions and
  does not widen voucher witness-only flows into a second semantic model.
- The slice does not widen v1 into slashable bond logic; it stays
  challenge-bounded and non-slashable as required by the phase packet.

## Review Loop

Manual fallback for `.github/prompts/gsd-review-tasks-execution.prompt.md` was
used because the slash prompt is not a callable tool in this environment.

- Pass 1 audited the diff, string anchors, and field-grammar contract against
  `060-08-PLAN.md`, `060-TODO.md`, and the guide. It confirmed the required
  grammar strings, the lock rows in the simulator matrix, and a clean
  `git diff --check` result.
- Pass 2 audited the semantic contract across wallet, storage, validator, and
  simulator boundaries. It found one real fail-closed gap: an unlock-like
  action could present `RightReference(Present)` without an actual right input
  delta. The slice closed that gap in
  `object_package_contract.rs` and added direct validator and simulator
  regression coverage for the bypassed-unlock reject path.
- Pass 3 reran the targeted release validation, reran the broad workspace
  `cargo test --release`, reran scoped `git diff --check`, and rechecked the
  lock anchors after the final fix. No significant issues remained.

Two consecutive clean review passes were achieved on passes 2 and 3 after the
semantic fix landed.

## Validation

- Mandatory bootstrap gate passed on the slice:
  `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- Lock grammar and source-anchor grep passed:
  `rg -n "validator_mandate_lock_v1|validator_mandate|holder_commitment|control_commitment|beneficiary_commitment|payload_commitment|valid_from|valid_until|challenge_from|challenge_until|use_nonce|transition_policy_id|revocation_policy_id|disclosure_policy_id|retention_policy_id|unlock|redelegate|reward" crates/z00z_wallets/src/wallet/WALLET-GUIDE.md crates/z00z_core/src/assets/assets_config.yaml crates/z00z_core/src/genesis/genesis_rights.rs docs/tech-papers/TODO-Wallet-idea.md crates/z00z_simulator/src/scenario_1/scenario_config.yaml crates/z00z_runtime/validators/tests/test_object_policy_verdicts.rs crates/z00z_runtime/watchers/tests/test_object_alerts.rs`
- Targeted wallet release validation passed:
  `cargo test -p z00z_wallets --release --lib test_tx_build_rejects_locks -- --nocapture`
  `cargo test -p z00z_wallets --release --lib test_tx_send_rejects_locks -- --nocapture`
  `cargo test -p z00z_wallets --release --lib test_tx_build_keeps_assets -- --nocapture`
  `cargo test -p z00z_wallets --release --lib validator_mandate_lock -- --nocapture`
- Targeted validator, watcher, and simulator release validation passed:
  `cargo test -p z00z_validators --release --test test_object_policy_verdicts -- --nocapture`
  `cargo test -p z00z_watchers --release --test test_object_alerts -- --nocapture`
  `cargo test -p z00z_simulator --release --test scenario_1 test_scenario1_object_flows -- --nocapture`
  `cargo test -p z00z_simulator --release --test scenario_1 test_scenario1_stage_surface -- --nocapture`
- Broad workspace release validation passed:
  `cargo test --release`
- Final scoped whitespace check is clean:
  `git diff --check -- crates/z00z_storage/src/settlement/object_package_contract.rs crates/z00z_wallets/src/tx/spend_rules.rs crates/z00z_wallets/src/tx/mod.rs crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_receive.rs crates/z00z_wallets/src/test_support/owned_objects.rs crates/z00z_wallets/src/adapters/rpc/methods/test_tx_send_body.rs crates/z00z_wallets/src/wallet/WALLET-GUIDE.md crates/z00z_runtime/validators/tests/test_object_policy_verdicts.rs crates/z00z_runtime/watchers/tests/test_object_alerts.rs crates/z00z_simulator/src/scenario_1/scenario_config.yaml crates/z00z_simulator/tests/test_scenario1_object_flows.rs`

## Result

`060-08` is complete. Phase 060 advances to `060-09-PLAN.md` for the
adversarial high-finding closure slice.
