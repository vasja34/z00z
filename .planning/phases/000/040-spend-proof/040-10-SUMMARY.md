# 040-10 Spend Proof Canonical Suite — Phase Summary

**Status:** COMPLETE  
**Closed:** 2026-04-29  
**Plan:** `040-10-PLAN.md`

---

## ⭐ What Was Done

Six tasks converged the wallet, public verifier, and simulator runtime onto one
canonical spend-proof suite, removing every legacy, statement-bound, and
theorem-v2 semantic branch from the live seam.

| Task | Title | Result |
| ---- | ----- | ------ |
| 040-10-01 | Canonical Theorem Authority Reset | ✅ DONE |
| 040-10-02 | Canonical Spend Carrier And Backend Migration | ✅ DONE |
| 040-10-03 | Public Input, Membership, And Nullifier Closure | ✅ DONE |
| 040-10-04 | Checkpoint Pipeline Internal Continuity | ✅ DONE |
| 040-10-05 | Test Coverage Consolidation | ✅ DONE |
| 040-10-06 | Git Versioning | ✅ DONE |

---

## ⚙️ Key Invariants Locked

- **Single suite ID:** `SPEND_PROOF_SUITE = "regular_spend_theorem_bpplus"` in
  `tx_wire_types.rs`; every prover, verifier, and artifact path uses this
  constant.
- **Wire versions:** `SPEND_PROOF_WIRE_VER = 2`, `SPEND_AUTH_WIRE_VER = 1`.
- **Canonical backend:** `CanonicalSpendProofBackend` (via
  `default_spend_proof_backend()`) is the only live backend; the old
  `StatementBoundSpendProofBackend` is fully removed.
- **No legacy aliases:** `rg` over `crates/z00z_wallets/src` and
  `crates/z00z_simulator` returns zero hits for
  `regular_spend_statement_bound_v1`, `regular_spend_theorem_bpplus_v1`,
  `theorem_v2`, and `StatementBoundSpendProofBackend`.
- **Membership composition:** `prev_root` membership is composed into the
  canonical theorem relation; no external checkpoint assumption remains
  implicit.
- **Nullifier semantics:** replay-safe nullifier path is the only path; no
  witness-replay or state-only acceptance fallback is live.
- **Digest discipline:** `build_tx_package_digest` / `compute_tx_digest_from_wire`
  divergence causes closed verifier rejection — no silent drift.

---

## ✅ Test Results

All suites green on final broad validation:

```shell
cargo test --release --features test-fast --features wallet_debug_dump
```

Focused suites verified individually:

- `test_spend_proof_wire`
- `test_spend_statement`
- `test_spend_proof_backend`
- `test_spend_prover_contract`
- `test_tx_proof_verifier`
- `test_scenario1_stage_surface` (simulator)

Doc tests: 37 pass. Integration tests: 5 pass.

---

## 🚩 Open Items (Out Of Scope For This Phase)

The following items are explicitly deferred and do **not** block this phase
close:

- **Public proof-of-knowledge:** the canonical backend validates theorem
  relations deterministically; a ZK proof-of-knowledge for private witness
  values is a future cryptographic closure step.
- **Checkpoint theorem finality:** the checkpoint seam remains
  package-coupled continuity evidence; a separate checkpoint-proof theorem
  is a rollup settlement concern outside phase 040-10.
- **Full rollup settlement proof:** tracked separately.

---

## 🔑 Canonical Files Touched

```text
crates/z00z_wallets/src/core/tx/spend_proof_backend.rs
crates/z00z_wallets/src/core/tx/spend_verification.rs
crates/z00z_wallets/src/core/tx/tx_wire_types.rs
crates/z00z_wallets/tests/test_spend_proof_backend.rs
crates/z00z_wallets/tests/test_spend_statement.rs
crates/z00z_wallets/tests/test_tx_proof_verifier.rs
crates/z00z_simulator/src/scenario_1/ (stage_4, stage_6, stage_11 utils)
crates/z00z_simulator/tests/test_scenario1_stage_surface.rs
```
