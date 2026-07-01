---
phase: 031-refactor-architecture
plan: "07"
subsystem: wallet-rpc
tags: [wallets, rpc, dto, simulator, session, facade]
requires:
  - phase: 031-02
    provides: curated z00z_core root facade used by wallet and simulator callers
  - phase: 031-03
    provides: curated z00z_crypto facade and test-only boundary rules
  - phase: 031-04
    provides: z00z_networks transport/dispatch boundary clarification
  - phase: 031-05
    provides: explicit wallet service seam ownership and persisted identity rules
  - phase: 031-06
    provides: wallet identity source-of-truth and session-bound lock posture
provides:
  - wallet RPC DTO ownership stays inside adapters::rpc::types instead of widening the root wallet facade
  - wallet-core transaction planning no longer falls back to ad hoc thread_rng entropy
  - canonical live status for wallet.key.export_public_material_v2 and ReceiverCardRecordV1 is explicit in code and regression tests
  - simulator transport callers lock wallets with session tokens instead of stale wallet_id payloads
affects: [031-08, PH31-WLT-RPC, wallet-rpc, simulator-runtime]
tech-stack:
  added: []
  patterns: [edge-owned DTO imports, session-bound lock RPC, explicit compatibility-lane disposition]
key-files:
  created:
    - .planning/phases/031-refactor-architecture/031-07-SUMMARY.md
  modified:
    - crates/z00z_wallets/src/adapters/mod.rs
    - crates/z00z_wallets/src/adapters/rpc/mod.rs
    - crates/z00z_wallets/src/adapters/rpc/types/mod.rs
    - crates/z00z_wallets/src/adapters/rpc/wallet_dispatcher_wiring.rs
    - crates/z00z_wallets/src/adapters/rpc/wallet_dispatcher_wiring_register.rs
    - crates/z00z_wallets/src/adapters/rpc/methods/key.rs
    - crates/z00z_wallets/src/core/chain/receiver_card_record.rs
    - crates/z00z_wallets/src/core/tx/multi_io.rs
    - crates/z00z_wallets/src/db/redb_wallet_store.rs
    - crates/z00z_wallets/src/db/redb_wallet_store_debug_types.rs
    - crates/z00z_wallets/src/lib.rs
    - crates/z00z_wallets/src/services/app_service_tests.rs
    - crates/z00z_wallets/tests/test_receiver_card_record.rs
    - crates/z00z_wallets/tests/test_rpc_wiring_spec_a.rs
    - crates/z00z_wallets/tests/test_tx_assetpack.rs
    - crates/z00z_simulator/src/scenario_1/stage_2_utils/flow.rs
    - crates/z00z_simulator/src/scenario_1/stage_3_utils/wallet_flow.rs
    - crates/z00z_simulator/src/scenario_1/stage_4_utils/bob_flow.rs
    - crates/z00z_simulator/src/scenario_1/stage_4_utils/persistence.rs
    - crates/z00z_simulator/src/scenario_1/stage_4_utils/reports_capture.rs
    - crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_lane_runtime.rs
    - crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_lane_runtime_support.rs
    - crates/z00z_simulator/src/scenario_1/stage_5_utils/transfer_lane_runtime_support.rs
    - crates/z00z_simulator/src/scenario_1/stage_11_charlie.rs
    - crates/z00z_simulator/tests/test_stage4_bob_flow.rs
key-decisions:
  - "Kept wallet.session.lock_wallet session-bound and fixed simulator raw transport callers instead of adding a wallet_id compatibility bypass at the RPC edge."
  - "Treated wallet.key.export_public_material_v2 and ReceiverCardRecordV1 as canonical live contracts and encoded that disposition in docs plus regression tests."
  - "Aligned the WalletService source-shape test with the live named-module/crate-internal contract instead of reverting the existing service seam split."
patterns-established:
  - "Pattern: transport DTOs are imported from explicit adapters::rpc::types submodules rather than flat re-export surfaces."
  - "Pattern: simulator helpers must thread unlock-returned session tokens into wallet.session.lock_wallet transport calls."
requirements-completed: [PH31-WLT-RPC]
duration: multi-session
completed: 2026-04-04
---

# Phase 031 Plan 07: Wallet RPC Boundary Summary

**Wallet RPC transport boundaries now stay adapters-owned while simulator runtime flows use session-bound lock calls and canonical live compatibility lanes remain explicit.**

## Performance

- **Duration:** multi-session
- **Started:** 2026-04-04T15:51:27Z
- **Completed:** 2026-04-04T16:50:42Z
- **Tasks:** 4
- **Files modified:** 25

## Accomplishments

- Narrowed wallet RPC DTO/export surfaces so transport-only aliases no longer act like stable root wallet contracts.
- Replaced the wallet-core `thread_rng()` fallback with the approved RNG owner and localized debug JSON ownership to the debug seam.
- Closed the remaining session-lock drift in simulator transport helpers and acceptance tests so the release-style validation gate reaches green again.

## Task Commits

No task commits were created in this session.

The repository already contained a large unrelated dirty worktree, so changes were left uncommitted to avoid mixing this plan's wallet-RPC work with parallel edits. If commit packaging is needed later, it should use the repository `z00z-git-versioning` workflow after the worktree is separated.

## Files Created/Modified

- `.planning/phases/031-refactor-architecture/031-07-SUMMARY.md` - plan closeout record
- `crates/z00z_wallets/src/adapters/rpc/types/mod.rs` - removed flat DTO re-exports so callers use owning submodules
- `crates/z00z_wallets/src/adapters/mod.rs` and `crates/z00z_wallets/src/adapters/rpc/mod.rs` - stopped widening transport-facing convenience exports through broader facades
- `crates/z00z_wallets/src/core/tx/multi_io.rs` - replaced ad hoc fallback randomness with `SystemRngProvider`
- `crates/z00z_wallets/src/db/redb_wallet_store*.rs` - moved debug JSON ownership into the debug seam
- `crates/z00z_wallets/src/adapters/rpc/methods/key.rs` and `crates/z00z_wallets/src/core/chain/receiver_card_record.rs` - documented canonical live compatibility lanes
- `crates/z00z_simulator/src/scenario_1/**` - fixed raw transport lock calls to reuse unlock-returned session tokens
- `crates/z00z_wallets/src/services/app_service_tests.rs` - updated source-shape assertions to match the live named-module/crate-internal service contract

## Decisions Made

- Kept `wallet.session.lock_wallet` strict at the RPC edge because wallet tests explicitly require unauthenticated transport calls to fail.
- Fixed simulator callers instead of weakening the wallet RPC contract with a `wallet_id` backdoor.
- Treated the existing WalletService named-module split as canonical and updated the stale source-shape test instead of reverting the seam split.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Fixed stale asset-pack test import blocking broader validation**

- **Found during:** release-style verification after wallet RPC boundary changes
- **Issue:** `crates/z00z_wallets/tests/test_tx_assetpack.rs` still imported `z00z_core::leaf::PackErr`, which no longer exists at the crate root.
- **Fix:** Updated the import to `z00z_core::assets::{leaf::PackErr, AssetPackPlain}`.
- **Files modified:** `crates/z00z_wallets/tests/test_tx_assetpack.rs`
- **Verification:** `cargo test -p z00z_wallets --release --features test-fast --test test_tx_assetpack -- --nocapture`

**2. [Rule 1 - Bug] Fixed simulator runtime/session-lock drift against the live wallet RPC contract**

- **Found during:** `cargo test --release --features test-fast --features wallet_debug_dump`
- **Issue:** multiple simulator helpers sent `{"wallet_id": ...}` to `wallet.session.lock_wallet`, but the live RPC contract requires a session token payload.
- **Fix:** Captured the unlock response session and threaded it into subsequent lock calls across stage 2, stage 3, stage 4, stage 5, and stage 11 runtime helpers plus the Bob flow test.
- **Files modified:** simulator stage helper files and `crates/z00z_simulator/tests/test_stage4_bob_flow.rs`
- **Verification:** `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_claim_acceptance -- --nocapture`; `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_stage4_bob_flow -- --nocapture`

**3. [Rule 1 - Bug] Aligned wallet-service source-shape test with the live seam contract**

- **Found during:** final release-style validation pass after simulator fixes
- **Issue:** `services::app_service::tests::test_wallet_create_no_bypass` still expected a retired flat `include!("wallet_service_store.rs")` shape and `pub(super)` visibility, while the live code uses a named internal module and crate-internal orchestrator access.
- **Fix:** Updated the test to assert the named-module contract and `pub(crate)` visibility while keeping the method non-public.
- **Files modified:** `crates/z00z_wallets/src/services/app_service_tests.rs`
- **Verification:** `cargo test -p z00z_wallets --release --lib test_wallet_create_no_bypass -- --nocapture`

---

**Total deviations:** 3 auto-fixed (1 blocking, 2 bugs)
**Impact on plan:** All deviations were required to complete the requested release-style verification. They tightened live contract alignment without widening the wallet RPC facade.

## Issues Encountered

- The environment did not expose a dedicated prompt-runner tool for `/.github/prompts/gsd-review-tasks-execution.prompt.md`, so that review loop could not be executed literally from this agent session.
- The repository worktree contains extensive unrelated edits, so commit packaging was intentionally deferred to avoid mixing this plan with parallel change sets.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- `PH31-WLT-RPC` is now satisfied and the broader release-style gate is green again.
- Phase state can advance to `031-08` once planning metadata is updated.
- Commit packaging remains pending and should be done only after separating this plan's files from the unrelated dirty worktree.

## Self-Check: PASSED

- Summary file created at `.planning/phases/031-refactor-architecture/031-07-SUMMARY.md`
- Mandatory bootstrap gate rerun passed
- Full `cargo test --release --features test-fast --features wallet_debug_dump` passed after the final fixes
- No remaining deferred blocker from this plan is left in `.planning/phases/031-refactor-architecture/deferred-items.md`

---
*Phase: 031-refactor-architecture*
*Completed: 2026-04-04*
