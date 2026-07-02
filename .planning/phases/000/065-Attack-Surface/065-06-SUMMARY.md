---
phase: 065-Attack-Surface
plan: 065-06
status: complete
completed_at: 2026-07-01
next_plan: 065-07
summary_artifact_for: .planning/phases/065-Attack-Surface/065-06-PLAN.md
---

<!-- markdownlint-disable MD022 MD031 MD032 MD033 MD041 MD047 MD055 MD056 MD060 -->

# 065-06 Summary: Canonical Wallet Mutation And Restore Ownership

## 🎯 Outcome

`065-06` is complete.

`WS-06` now closes on one canonical mutation owner and one explicit restore
retry contract. Local asset mutation RPCs route through `LocalMutationExec`,
canonical tx identity is digest-checked at the package and broadcast seams,
restore owns a durable `.restore.json` marker across history or `.wlt` or
publish steps, and `rotate_master_key` wording now matches the persisted
rotation behavior that the code actually ships.

## 📦 Files Changed

- `.planning/phases/065-Attack-Surface/065-04-SUMMARY.md`
- `.planning/phases/065-Attack-Surface/065-06-SUMMARY.md`
- `.planning/ROADMAP.md`
- `.planning/STATE.md`
- `crates/z00z_core/src/assets/asset_crypto.rs`
- `crates/z00z_core/src/assets/test_asset.rs`
- `crates/z00z_simulator/src/scenario_1/stage_6/tx_lane_runtime_support.rs`
- `crates/z00z_simulator/tests/scenario_1/claim_pkg_crypto.rs`
- `crates/z00z_simulator/tests/scenario_1/test_claim_emit.rs`
- `crates/z00z_simulator/tests/scenario_1/test_claim_pkg_runtime.rs`
- `crates/z00z_simulator/tests/scenario_1/test_stage4_claim_gate.rs`
- `crates/z00z_wallets/src/chain/broadcast_impl.rs`
- `crates/z00z_wallets/src/chain/local_node_sim.rs`
- `crates/z00z_wallets/src/rpc/asset_rpc_impl.rs`
- `crates/z00z_wallets/src/rpc/asset_rpc_server_catalog.rs`
- `crates/z00z_wallets/src/rpc/asset_rpc_server_ops.rs`
- `crates/z00z_wallets/src/rpc/asset_rpc_support_state.rs`
- `crates/z00z_wallets/src/services/wallet_actions_backup.rs`
- `crates/z00z_wallets/src/tx/tx_digest.rs`
- `crates/z00z_wallets/tests/test_asset_rpc_mutations.rs`
- `crates/z00z_wallets/tests/test_chain_client_sim.rs`
- `crates/z00z_wallets/tests/test_tx_digest_framing.rs`
- `crates/z00z_wallets/tests/test_wallet_restore_atomic.rs`

## 🔧 Landed Changes

- Canonical mutation owner
  - `asset_rpc_support_state.rs` now exposes sealed `LocalMutationExec`.
  - `merge_assets`, `split_asset`, `stake_assets`, `swap_assets`, and
    `unstake_assets` all submit through `.local_mutation_exec(...).submit()?`.
  - the old helper-composition tail no longer invents per-RPC tx ids or local
    mutation counters outside the executor.
- Canonical tx-id and digest truth
  - local mutation outputs now derive deterministic blinding and build assets
    through `Asset::new_confidential_with_blinding(...)`.
  - `build_tx_package_digest(...)` now ignores nondeterministic output
    `range_proof` and `owner_signature` bytes while preserving the canonical
    transaction facts.
  - `LocalNodeSim` recomputes the canonical package digest from payload bytes
    and rejects mismatched `tx_digest_hex`.
  - `BroadcastImpl` now treats byte-different but canonically equivalent
    `TxPackage` payloads as one tx id through
    `canonical_tx_id_from_bytes(...)` and `same_canonical_tx(...)`.
- Explicit restore retry contract
  - `wallet_actions_backup.rs` now persists `WalletRestoreMark` with
    `RestoreMarkStage` through `Prepared`, `HistoryStep`, `WltStep`,
    `PublishStep`, and `Published`.
  - restore now owns explicit `resume_restore_mark(...)` recovery semantics
    across staged history, `.wlt` commit, rollback, and publish.
  - crash-style failpoints remain executable and now prove durable retry or
    rollback truth instead of best-effort in-memory behavior.
- Honest rotation wording
  - the live `rotate_master_key` wording and typed response contract now state
    the persisted rewrite truth instead of the retired placeholder framing.
  - source tests pin the exact persisted-rotation wording:
    `Rewrites persisted wallet encryption state under one rotation contract`.
- Broad-gate cleanup needed for full release proof
  - the last `scenario_1` claim-runtime tests now use the canonical raw-builder
    name `build_tx_output_unchecked(...)`.
  - Stage 6 shared runtime support now sends the canonical
    `{"session": session}` payload shape to `wallet.key.get_receiver_card`.
  - the stale overlength public-lane test-name mention was removed from
    `065-04-SUMMARY.md`.

## ✅ Validation

Commands green on the current tree:

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `cargo fmt --all`
- `cargo test --release -p z00z_wallets --test test_asset_rpc_mutations -- --nocapture`
- `cargo test --release -p z00z_wallets --test test_wallet_restore_atomic -- --nocapture`
- `cargo test --release -p z00z_wallets --test test_chain_broadcast_retry -- --nocapture`
- `cargo test --release -p z00z_wallets --test test_tx_store_integration -- --nocapture`
- `cargo test --release -p z00z_wallets --test test_chain_client_sim -- --nocapture`
- `cargo test --release -p z00z_core test_new_confidential_with_blinding_is_deterministic -- --nocapture`
- `cargo test --release -p z00z_simulator 'scenario_1::stage_6::test_tx_lane_runtime_suite::test_tx_validation_chain_id' -- --exact --nocapture`
- `cargo test --release -p z00z_simulator 'scenario_1::stage_6::test_tx_lane_runtime_suite::test_tx_validation_nullifier_drift' -- --exact --nocapture`
- `cargo test --release -p z00z_wallets`
- `cargo test --release`

## 🔍 Manual Review Passes

The required `/GSD-Review-Tasks-Execution` loop was attempted three times, but
the current runtime still does not provide a callable review path for this
slice.

- Attempt 1
  - `bash -lc '/GSD-Review-Tasks-Execution current_spec=.planning/phases/065-Attack-Surface/065-06-PLAN.md current_task="Canonical Wallet Mutation And Restore Ownership"'`
  - Result: failed with `/GSD-Review-Tasks-Execution: No such file or directory`
- Attempt 2
  - `timeout 45s gsd -p '/GSD-Review-Tasks-Execution current_spec=.planning/phases/065-Attack-Surface/065-06-PLAN.md current_task="Canonical Wallet Mutation And Restore Ownership" --yolo'`
  - Result: exited with code `1` and reported
    `402 Prompt tokens limit exceeded: 66678 > 38936`
- Attempt 3
  - `timeout 45s gsd --no-session --extension .github -p '/GSD-Review-Tasks-Execution current_spec=.planning/phases/065-Attack-Surface/065-06-PLAN.md current_task="Canonical Wallet Mutation And Restore Ownership" --yolo'`
  - Result: exited with code `1` and reported
    `402 Prompt tokens limit exceeded: 82820 > 38936`

Equivalent workspace-first review was executed manually against the same scope.

- Pass 1
  - Re-read `065-06-PLAN.md`, `065-TODO.md`, `asset_rpc_support_state.rs`,
    `broadcast_impl.rs`, `local_node_sim.rs`, `tx_digest.rs`, and the restore
    flow in `wallet_actions_backup.rs`.
  - Result: confirmed the intended one-owner closure but found the broad
    workspace release rerun still blocked by four `scenario_1` claim-runtime
    tests importing the retired raw builder name. Fixed those tests to use the
    canonical `build_tx_output_unchecked(...)` surface.
- Pass 2
  - Re-ran the broad workspace release gate and isolated the new failure to
    Stage 6 shared-case setup, then re-read
    `tx_lane_runtime_support.rs` against the current key RPC contract.
  - Result: found one stale array payload for
    `wallet.key.get_receiver_card`. Fixed it to the canonical
    `{"session": session}` request shape and proved the repair with targeted
    release reruns of the two previously failing Stage 6 tests.
- Pass 3
  - Re-ran source-grep and touched-scope compliance checks over the mutation
    routes, restore marker contract, raw-builder names, session payload shape,
    and the stale phase-summary test-name mention.
  - Result: clean. All public local mutation RPC owners call
    `.local_mutation_exec(...)`, no touched code still uses
    `build_tx_stealth_output(...)` or `json!([session])`, and the stale
    overlength test-name mention is gone.
- Pass 4
  - Re-ran `cargo test --release`, targeted `git diff --check` on the touched
    `065-06` surface, and focused source probes for restore and rotation truth.
  - Result: clean. The full release gate is green, the touched diff surface is
    whitespace-clean, restore retry semantics stay durable and explicit, and
    the persisted rotation wording remains pinned. The untargeted
    repository-wide `git diff --check` still reports the pre-existing trailing
    whitespace issue in `.planning/GSD-Workflow.md`, outside this slice.

Passes 3 and 4 were consecutive clean manual review runs after the last
in-scope findings were fixed.

## 🧾 Closeout

`065-06` closes `WS-06` with one canonical mutation executor, canonical
digest-owned tx identity, explicit restore retry state across file and publish
boundaries, and honest persisted rotation wording. The active Phase 065 lane
moves to `065-07-PLAN.md`.
