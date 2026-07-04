---
phase: 058-HJMT-benchmarks
artifact: shared-proof-report
status: closed
updated: 2026-06-16
---

# Phase 058 Shared Proof Report

## 🎯 Purpose

This report is the exact final closeout artifact for the `Shared proof vector`
row from Upgrade `12.1`.

It closes the row without inventing a second proof authority path. The live
owner homes remain the existing storage fixtures, batch-proof tests, and bench
documentation.

## ✅ Closed Artifact Set

| Surface | Exact artifact |
| --- | --- |
| positive shared proof vectors | `crates/z00z_storage/tests/fixtures/hjmt_upgrade/batch_proof_v1_positive/manifest.json` with `BPB-G-001..005` |
| negative shared proof vectors | `crates/z00z_storage/tests/fixtures/hjmt_upgrade/batch_proof_v1_negative/manifest.json` with `BPB-T-001..008` |
| positive owner home | `crates/z00z_storage/tests/test_hjmt_batch_proof.rs::test_positive_fixtures_match_builders` |
| negative owner home | `crates/z00z_storage/tests/test_hjmt_batch_proof_negative.rs::test_negative_fixtures_reject_with_expected_error` |
| benchmark vocabulary home | `crates/z00z_storage/benches/settlement_benches.md` |
| benchmark wording guard | `crates/z00z_storage/tests/test_bench_lanes.rs` |

## 📌 Closure Statement

The shared-proof closeout is accepted because all of the following are now true:

1. the canonical shared wire contract is `BatchProofBlobV1`;
2. positive shared-proof vectors are frozen under `BPB-G-001..005`;
3. negative tamper and parser vectors are frozen under `BPB-T-001..008`;
4. the compatibility baseline against independent proofs remains live in the
   same storage-owned test home;
5. benchmark wording now points to one canonical shared-proof story instead of
   leaving the row as only `partial`.

## ✅ Verification Commands

```bash
cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_batch_proof -- --nocapture
cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_batch_proof_negative -- --nocapture
cargo test -p z00z_storage --release --features test-params-fast --test test_bench_lanes -- --nocapture
```
