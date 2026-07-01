---
phase: 053-HJMT-Backend
status: complete
completed_at: 2026-06-05
summary_artifact_for: .planning/phases/053-HJMT-Backend/
---

<!-- markdownlint-disable MD060 -->

# Phase 053 Summary

## Result

Phase 053 is complete. The repository now runs on the live generalized HJMT
settlement backend with `SettlementStateRoot`, `SettlementPath`,
`SettlementLeaf`, `RightLeaf`, `FeeEnvelope`, deletion and non-existence proof
families, adaptive bucket policy proofs, privacy-reviewed occupancy evidence,
private cache and scheduler planes, durable reload and recovery, downstream
checkpoint, snapshot, wallet, and simulator integration, executable docs, and
legacy compatibility/simple-JMT runtime tails purged from live code.

## Completed Plans

- `053-01` through `053-05`: closed the live contract guardrails, settlement
  root generation, generalized path and leaf contracts, fee-envelope
  separation, and the HJMT store API cutover.
- `053-06` through `053-10`: closed rights-enabled core YAML/genesis inputs,
  settlement proof family v2, adaptive bucket policy proofs, occupancy privacy
  evidence, and the private forest-cache plane.
- `053-11` through `053-15`: closed the bounded async scheduler, journal and
  recovery durability, reload and historical proof coverage, downstream
  settlement integration, and Stage 13 production examples.
- `053-16` through `053-20`: closed mixed corpus and fuzz/property evidence,
  benchmarks and bounded metrics, docs and executable examples, the production
  default gate, and the final legacy-runtime purge.

## Removed Legacy Surface

- Removed live runtime or helper modules:
  - `crates/z00z_storage/src/settlement/dual_verify.rs`
  - `crates/z00z_storage/src/settlement/whitebox/*`
  - `crates/z00z_storage/src/settlement/tx_plan/tx_plan_batch.rs`
  - `crates/z00z_storage/src/settlement/tx_plan/tx_plan_batches.rs`
  - `crates/z00z_storage/src/settlement/tx_plan/tx_plan_engine.rs`
- Removed legacy-only test and bench owners that preserved the old storage
  story:
  - legacy asset-era bench lanes that previously lived under
    `crates/z00z_storage/benches/`
  - legacy nested storage test folders that previously lived under
    `crates/z00z_storage/tests/`
  - asset-era backend, reload, search, and guardrail suites superseded by the
    live `test_default_gate.rs`, `test_redb_reload.rs`, `test_store_api.rs`,
    and `test_live_guardrails.rs` owners
- Removed the superseded design alias file `docs/Z00Z-JMT-Design.md`.
- Removed live mode lanes for compatibility, forest, and dual-verify. Stale
  names now fail closed and cannot select runtime behavior.

## Final Review Evidence

The `053-20` closeout review loop reopened twice on planning-packet drift after
the runtime purge landed, then converged cleanly. Review passes 6 and 7 were
the required consecutive clean pair, so the final packet can honestly mark the
plan and the phase complete.

## Final Validation Snapshot

Phase-closeout evidence is recorded in `053-01-SUMMARY.md` through
`053-20-SUMMARY.md`. The final closeout slice reran the mandatory bootstrap
gate, targeted purge and settlement-root release tests, the full workspace
release gate, grep-backed source audits, and `git diff --check` successfully.

The resulting live surface is HJMT-only: default and explicit `hjmt` mode are
accepted, stale backend names are rejected, serialization and proof surfaces
are settlement-native, and no live production crate retains the old
compatibility/simple-JMT storage implementation as dormant runtime baggage.
