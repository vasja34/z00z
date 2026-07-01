---
phase: 051-HJMT-Facade
status: complete
completed_at: 2026-05-28
summary_artifact_for: .planning/phases/051-HJMT-Facade/
---

<!-- markdownlint-disable MD060 -->

# Phase 051 Summary: HJMT Facade

## Result

Phase 051 is complete. The repository now has one storage-owned semantic
facade for asset-tree behavior, an explicit compatibility backend over the
current shared namespaced JMT implementation, locked root vocabulary, locked
compatibility proof envelope, downstream authority guardrails, a green
compatibility golden corpus, and a green reload/checkpoint/proof reject corpus.

Phase 052 may begin as full HJMT backend implementation behind the existing
facade. It must not reopen facade ownership, root taxonomy, proof envelope, or
downstream authority decisions.

## Completed Plans

- `051-01`: established `AssetTreeBackend` and `CompatibilityBackend`.
- `051-02`: locked root taxonomy and compatibility proof envelope v1.
- `051-03`: closed public API and downstream semantic-authority leakage.
- `051-04`: landed golden compatibility, proof reject, reload, checkpoint, and
  path-index coverage.
- `051-05`: synchronized storage docs and future forest handoff.
- `051-06`: proved Phase 052 readiness with source-backed exit-gate evidence.

## Shipped Evidence

- `crates/z00z_storage/tests/assets/test_backend_facade_contract.rs`
- `crates/z00z_storage/tests/test_phase051_golden_corpus.rs`
- `crates/z00z_storage/tests/test_phase051_guardrails.rs`
- storage proof, root, RedB reload, checkpoint, serialization, and search
  tests extended through existing anchors
- wallet tamper/spend-proof anchors validated against storage-owned proof
  authority
- simulator Stage 7 and unified Scenario 1 anchors validated as proof-first
  consumers
- simulator Stage 13 no longer exports or verifies `flat_root*`; physical root
  diagnostics remain storage-owned whitebox state

## Deferred To Phase 052

- physical forest backend
- fixed bucket policy and verifier-visible bucket metadata
- independent physical bucket commits
- child-before-parent publication
- forest commit journal
- crash-safe forest recovery
- dual-backend equivalence mode
- configuration-gated backend enablement
- deletion proof semantics
- non-existence proof semantics
- `RightLeaf`, `FeeEnvelope`, and live generalized settlement root exports

## Validation

All Rust validation evidence used release mode.

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` passed as
  the mandatory fail-fast gate.
- Focused storage, wallet, simulator, checkpoint, reload, search, golden
  corpus, and guardrail release tests passed.
- `cargo test -p z00z_storage --release --features test-fast --features wallet_debug_dump`
  passed.
- `cargo test --release --features test-fast --features wallet_debug_dump`
  passed for the workspace.
- `cargo fmt --check` exited 0 with only the repository's existing
  stable-channel rustfmt warnings for nightly-only options.
- `git diff --check` exited 0.
- Final source-backed scans found no downstream physical-layout authority leak,
  no simulator `flat_root*` export, no duplicate validator checkpoint verifier
  shape, no fake forest backend, and no stale suffixed claim-source proof
  spelling.
