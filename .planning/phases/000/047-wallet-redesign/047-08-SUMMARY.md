---
phase: 047-wallet-redesign
plan: 8
status: complete
completed_at: 2026-05-20
next_plan: complete
---

# Phase 047-08 Summary

## Completed Scope

`047-08` is complete for the simulator, docs, existing-test migration, and
final honesty-wave closure. Stage 13 now proves the live `wallet.tx.*`
lifecycle over persisted wallet `OwnedAssetPayload` objects plus canonical
tx-history JSONL with real backup-restore, tamper, and reopen execution, the
phase-local Phase 046 spec copy now tells the same storage story, and the final
validation matrix closed green without leaving Snapshot framed as the target
live authority.

## Files Changed

- `crates/z00z_simulator/src/scenario_1/stage_13_wallet_tx/flow.rs`
- `crates/z00z_simulator/src/scenario_1/stage_13_wallet_tx/storage.rs`
- `crates/z00z_simulator/src/scenario_1/scenario_design.yaml`
- `crates/z00z_simulator/src/scenario_1/runner_contract_table.in`
- `crates/z00z_simulator/src/scenario_1/runner_verify.rs`
- `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs`
- `.planning/phases/047-wallet-redesign/047-wallet-addon-spec.md`
- `crates/z00z_wallets/src/adapters/rpc/methods/key_impl/tests.rs`

## Landed Changes

- Rewrote the Stage 13 storage authority text, contract table wording, design
  YAML description, verifier constants, and stage-surface assertions so the
  simulator now names encrypted `.wlt` `OwnedAssetPayload` objects plus
  canonical tx-history JSONL instead of Snapshot payloads.
- Upgraded Stage 13 from report-only truth to executed proof: the flow now
  creates a real backup, restores it through
  `ForensicImportMode::WalletPlusHistory`, compares restored owned-asset sets
  and exact tx-history bytes against the live sender wallet, proves tampered
  import does not mutate receiver owned assets, and proves sender reopen
  reloads the same owned-asset set after session lock.
- Tightened `runner_verify.rs` so Stage 13 fails closed unless the persisted log
  rows contain the new owned-asset authority wording plus explicit
  `WalletPlusHistory`, tamper, and reopen markers.
- Updated the Phase 047-local Phase 046 spec copy so its decisions, rationale,
  EARS requirements, and doublecheck register now treat Snapshot as a
  compatibility bridge only, keep `wallet.asset.send_asset` in the
  compatibility/UX lane, and align the docs with the post-cutover runtime.
- The full broad rerun exposed one unrelated-but-real test-boundary regression:
  `key_impl` test helpers could inherit a dead `Z00Z_WALLET_CONFIG_PATH` from
  neighboring suites. The helper now serializes env access with the existing
  wallet-config env lock, captures/restores the config env, and clears ambient
  wallet-config overrides before constructing test services.

## Boundary Kept Intact

- `047-08` does not move canonical tx history into `.wlt`; the live history
  plane remains `wallet_<stem>_tx_history.jsonl`.
- `047-08` does not reintroduce Snapshot as a second live authority plane and
  does not add simulator-only persistence seams.
- `047-08` does not resume paused Phase 046 or auto-select the next roadmap
  phase after Phase 047 closure.

## Review Passes

- Pass 1: Rechecked the simulator truth surface, stale authority phrases, and
  Stage 13 verifier markers across `storage.rs`, `runner_verify.rs`, and the
  stage-surface guards. No significant issues found.
- Pass 2: Rechecked the execution helpers in `flow.rs` and the phase-local spec
  copy in `047-wallet-addon-spec.md`; confirmed that backup restore, tx-history
  comparison, tamper no-mutation, and reopen proof all execute for real and
  that the doc copy stays aligned to one owned-asset authority model. No
  significant issues found.
- Pass 3: Rechecked the final tree against the green bootstrap gate, the green
  broad workspace `cargo test` gate, and the final stale-string sweep. The only
  hygiene issue left in `git diff --check` was pre-existing trailing whitespace
  in `.planning/GSD-Workflow.md`, outside the `047-08` write-set. No
  significant issues found.

Two consecutive clean passes were achieved on passes 2 and 3.

## Validation

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` passed on
  the final tree after the env-boundary test fix.
- `cargo test --release --features test-fast --features wallet_debug_dump -p z00z_wallets test_session_expired_rpc_code -- --nocapture` passed.
- `cargo test --release --features test-fast --features wallet_debug_dump -p z00z_wallets test_derive_session_expired_code -- --nocapture` passed.
- `cargo test --release --features test-fast --features wallet_debug_dump`
  passed on the final tree.
