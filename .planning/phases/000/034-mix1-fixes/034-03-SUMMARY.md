---
phase: 034-mix1-fixes
plan: 03
subsystem: sender-authority-migration
tags: [stealth, tx-facade, simulator, wallet, fail-closed, api-break]
requires:
  - phase: 034-02
    provides: stable spend-nullifier contract and verified package-loading baseline
provides:
  - Stealth-owned full-leaf and bundle construction surface
  - Narrowed tx facade with no public sender-construction authority
  - Migrated wallet, simulator, and example callers on the stealth binding path
  - Updated Phase 040 wording for live proof-wire and construction-owner truth
affects: [PH34-SENDER-AUTHORITY, Phase-040-spend-proof]
tech-stack:
  added: []
  patterns: [stealth-owned sender authority, fail-closed legacy retirement, public-facade narrowing]
key-files:
  created:
    - /home/vadim/Projects/z00z/.planning/phases/034-mix1-fixes/034-03-SUMMARY.md
  modified:
    - /home/vadim/Projects/z00z/crates/z00z_wallets/src/core/stealth/output.rs
    - /home/vadim/Projects/z00z/crates/z00z_wallets/src/core/stealth/mod.rs
    - /home/vadim/Projects/z00z/crates/z00z_wallets/src/core/tx/mod.rs
    - /home/vadim/Projects/z00z/crates/z00z_wallets/src/core/tx/builder.rs
    - /home/vadim/Projects/z00z/crates/z00z_wallets/src/core/tx/output_flow.rs
    - /home/vadim/Projects/z00z/crates/z00z_wallets/src/core/tx/witness_gate.rs
    - /home/vadim/Projects/z00z/crates/z00z_wallets/src/core/tx/lifecycle.rs
    - /home/vadim/Projects/z00z/crates/z00z_wallets/src/core/tx/spend_verification.rs
    - /home/vadim/Projects/z00z/crates/z00z_wallets/src/services/wallet_service_tests.rs
    - /home/vadim/Projects/z00z/crates/z00z_wallets/tests/test_spend_witness_gate.rs
    - /home/vadim/Projects/z00z/crates/z00z_wallets/tests/test_s5_spec6_bridge.rs
    - /home/vadim/Projects/z00z/crates/z00z_wallets/tests/support/test_s5_sender_examples_support.rs
    - /home/vadim/Projects/z00z/crates/z00z_wallets/examples/wallet_reload.rs
    - /home/vadim/Projects/z00z/crates/z00z_wallets/examples/ALL-wallet-flows.md
    - /home/vadim/Projects/z00z/crates/z00z_simulator/src/scenario_1/stage_4_utils/mod.rs
    - /home/vadim/Projects/z00z/crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_validation_gates.rs
    - /home/vadim/Projects/z00z/crates/z00z_simulator/src/scenario_1/stage_6_utils/bridge_output_router.rs
    - /home/vadim/Projects/z00z/crates/z00z_simulator/src/scenario_1/stage_6_utils/bundle_lane_impl.rs
    - /home/vadim/Projects/z00z/crates/z00z_simulator/examples/simulator_interop/support.rs
    - /home/vadim/Projects/z00z/crates/z00z_simulator/tests/test_stage5_receive_bridge.rs
    - /home/vadim/Projects/z00z/crates/z00z_simulator/tests/test_stage6_checkpoint_final_gate.rs
    - /home/vadim/Projects/z00z/crates/z00z_simulator/tests/test_stage6_checkpoint_storage_bridge.rs
    - /home/vadim/Projects/z00z/.planning/phases/040-spend-proof/040-Spend-Proof-Spec.md
key-decisions:
  - "Make `core::stealth` the only public sender-construction owner and remove tx-facade construction exports after live callers are migrated."
  - "Keep tx-level bridge helpers only as internal support utilities; external binder consumers move to the stealth surface or crate-root stealth re-exports."
  - "Treat the retirement of historical `core::tx::builder::*` and `core::tx::output_flow::*` construction paths as an intentional public API break rather than a soft runtime shim."
patterns-established:
  - "Public sender-output construction now enters through crate-root stealth re-exports or `core::stealth`, while `core::tx` stays focused on tx assembly and verification."
requirements-completed: [PH34-SENDER-AUTHORITY]
completed: 2026-04-10
reviewed: 2026-04-10T00:00:00Z
---

# Phase 034 Plan 03 Summary

## Outcome

Plan 03 is complete. Sender-construction authority moved to the stealth-owned
surface, `core::tx` no longer exports sender-construction entrypoints as public
APIs, and the remaining wallet, simulator, and example consumers now build and
bind confidential outputs through the stealth-owned path.

## Accomplishments

- Added and exported canonical stealth-owned builders for full leaf and bundle
  construction, including explicit serial-range enforcement on public builder
  entrypoints.
- Migrated live wallet, simulator, and example callers away from
  `sender_create_output_for(...)`, `create_output_bundle(...)`, and
  `create_output_bundle_with_rng(...)` as public construction authority.
- Narrowed `core::tx` so `builder` and `output_flow` are no longer public
  construction surfaces; tx binding helpers remain internal support utilities.
- Moved public binder consumers onto `bind_stealth_output_wire()` and aligned
  Stage 6 bridge rebuilds with recomputed nonce semantics.
- Updated touched Phase 040 spec sections so the live proof-wire, digest
  prefix, chain-aware witness-gate call shape, and stealth-owned construction
  owner are described truthfully.

## Verification

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` passed
  earlier in the execution wave before broader narrowing work continued.
- `cargo test -p z00z_wallets --release --features test-fast --features wallet_debug_dump --test test_s5_spec6_bridge --test test_spend_witness_gate --test test_s7_examples` passed after the final public-binder migration.
- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_stage6_checkpoint_final_gate --test test_stage6_checkpoint_storage_bridge` passed after the final Stage 6 nonce and binder-path fixes.
- `cargo test -p z00z_wallets --release --features test-fast --features wallet_debug_dump test_recv_range_restart -- --nocapture` passed after correcting the receive helper serial regression.

## Breaking Change Note

- Historical public construction imports under `core::tx::builder::*` and
  `core::tx::output_flow::*` are intentionally retired in this plan.
- Public caller migration target is the crate-root stealth re-export surface or
  `core::stealth`, depending on whether the caller is public-story code or
  internal module code.

## Issues Encountered

- Stronger fail-closed verification moved several simulator failures earlier in
  the pipeline, so Stage 5 and Stage 6 tests had to accept earlier package
  rejection points instead of late-stage shape failures.
- Review passes found residual serial-range, nonce-rebuild, and spec-drift
  issues after the main migration landed; each was corrected before closeout.
- A receive helper in `wallet_service_tests.rs` still used `serial_id = 0`
  after the new serial contract became universal; that regression was fixed and
  revalidated with the exact receive-range test.

## Next Phase Readiness

- `034-04` can now start from one public sender-construction owner and a
  narrowed tx facade.
- Phase 040 planning can use the updated proof-wire and digest wording as the
  current baseline instead of the pre-migration tx-owned construction story.

## Known Stubs

- The regular tx proof backend is still not a full authoritative checkpoint
  proof system; current proof and auth wires remain the narrowed public-spend
  carrier only.

## Threat Flags

None for the Plan 03 sender-authority migration scope after the final
review-driven fixes.

## Self-Check

PASSED.

---
*Phase: 034-mix1-fixes*
*Completed: 2026-04-10*
