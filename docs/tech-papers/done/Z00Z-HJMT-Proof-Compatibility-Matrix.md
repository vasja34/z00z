# Z00Z HJMT Proof Compatibility Matrix

Version: 2026-06-16

## 🎯 Purpose

This standalone matrix closes Appendix C row `C-04` from
[Z00Z-HJMT-Upgrade.md](Z00Z-HJMT-Upgrade.md).

It records the exact live compatibility contract among:

- `ProofBlob`
- `Vec<ProofBlob>`
- `BatchProofBlobV1`

The goal is to keep one canonical proof path hierarchy without inventing a
parallel proof layer.

## ✅ Compatibility Matrix

| Surface | Contract role | Live owner home | Accepted use | Reject and boundary rules |
| --- | --- | --- | --- | --- |
| `ProofBlob` | single-path compatibility proof envelope | `crates/z00z_storage/src/settlement/proof.rs`; `crates/z00z_storage/tests/test_hjmt_live_proof_families.rs` | one-path inclusion, deletion, and non-existence verification under the current settlement contract | verifier remains fail-closed on wrong proof-family, wrong root binding, wrong default-commitment binding, or malformed payload bytes |
| `Vec<ProofBlob>` | independent multi-path compatibility baseline | `crates/z00z_storage/src/settlement/hjmt_proof.rs`; `crates/z00z_storage/tests/test_hjmt_batch_proof.rs` | deterministic reference batch for clustered and scattered path sets under one root | remains an ordered collection of independent proofs; it does not imply shared witness reuse, shared verifier state, or a single shared parser pass |
| `BatchProofBlobV1` | additive shared multi-path proof envelope | `crates/z00z_storage/src/settlement/hjmt_batch_proof.rs`; `crates/z00z_storage/tests/test_hjmt_batch_proof.rs`; `crates/z00z_storage/tests/test_hjmt_batch_proof_negative.rs` | shared-parent and mixed-path witness reuse with one bounded parser and one shared verifier context | mixed proof families, mixed opening kinds, leaf-family mismatches, malformed references, and header-field tampering reject fail-closed |

## 🔒 Equivalence Rules

1. `ProofBlob` remains the single-path compatibility contract.
2. `Vec<ProofBlob>` remains the semantic oracle and migration baseline for
   multi-path parity checks.
3. `BatchProofBlobV1` is the only shared-proof wire contract in the live
   repository.
4. No document may treat `Vec<ProofBlob>` as if it were already a shared proof
   encoding.
5. No document may introduce a second shared-proof envelope beside
   `BatchProofBlobV1`.

## 📌 Evidence Packet

| Claim | Evidence |
| --- | --- |
| positive shared-proof compatibility is frozen | `crates/z00z_storage/tests/fixtures/hjmt_upgrade/batch_proof_v1_positive/manifest.json` with `BPB-G-001..005` |
| negative shared-proof compatibility is frozen | `crates/z00z_storage/tests/fixtures/hjmt_upgrade/batch_proof_v1_negative/manifest.json` with `BPB-T-001..008` |
| the independent baseline remains live | `crates/z00z_storage/tests/test_hjmt_batch_proof.rs` compares `Vec<ProofBlob>` and `BatchProofBlobV1` on the same seeded path sets |
| benchmark wording stays aligned to the compatibility contract | `crates/z00z_storage/benches/settlement_benches.md` and `crates/z00z_storage/tests/test_bench_lanes.rs` |

## ✅ Verification Commands

```bash
cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_batch_proof -- --nocapture
cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_batch_proof_negative -- --nocapture
cargo test -p z00z_storage --release --features test-params-fast --test test_bench_lanes -- --nocapture
```
