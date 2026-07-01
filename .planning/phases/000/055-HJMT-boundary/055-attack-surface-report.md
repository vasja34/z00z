# Phase 055 Attack Surface Report

**Rechecked:** 2026-06-11
**Doublecheck Mode:** phase-packet plus live owner-code skeptical verification
**Result:** no admitted candidate after boundary-slice review of the batch-proof, Stage 13, and settlement-bench evidence paths

## ✅ Scan Result

No candidate passed the pro-con audit and verification gate.

### 🔍 Reviewed Boundary Slices

- **External input and parser slice:** `BatchProofBlobV1::decode_with_limits(...)` enforces byte ceilings, bounded table counts, and trailing-byte rejection before acceptance, and then immediately hands the decoded blob into the fail-closed verifier at `crates/z00z_storage/src/settlement/proof_batch.rs:944` and `crates/z00z_storage/src/settlement/proof_batch.rs:1005`.
- **Cryptographic and proof-verification slice:** `check_batch_contract_v1(...)` verifies transcript domain, root generation, root bind, checkpoint bind, canonical ordering, family consistency, exact table usage, witness-domain restrictions, and atomic root reconstruction at `crates/z00z_storage/src/settlement/proof_batch_verify.rs:63`, `crates/z00z_storage/src/settlement/proof_batch_verify.rs:143`, `crates/z00z_storage/src/settlement/proof_batch_verify.rs:248`, and `crates/z00z_storage/src/settlement/proof_batch_verify.rs:318`.
- **Replay, uniqueness, and state-consumption slice:** duplicate paths, duplicate witness indexes, and unused tables fail closed in `crates/z00z_storage/src/settlement/proof_batch_verify.rs:154`, `crates/z00z_storage/src/settlement/proof_batch_verify.rs:276`, and `crates/z00z_storage/src/settlement/proof_batch_verify.rs:302`, while Stage 13 requires live `atomic_verdict == accepted` semantics through the runner verification contract in `crates/z00z_simulator/src/scenario_1/runner_verify.rs:178`.
- **Operator, admin, and debug-only surface slice:** the canonical settlement bench helper narrows proof-note authority with explicit `full` / `batch_only` / `skip` scope in `crates/z00z_storage/scripts/run_storage_settlement_bench.py:134`, and the scenario lane returns non-zero when Stage 13 artifacts were not refreshed in the current run at `crates/z00z_storage/scripts/run_storage_settlement_bench.py:465` and `crates/z00z_storage/scripts/run_storage_settlement_bench.py:685`.
- **Secret handling, storage, and logging slice:** Stage 3 debug dumps redact `secrets[]` at write time in `crates/z00z_simulator/src/scenario_1/stage_3_utils/post_claim.rs:268`, and Stage 13 re-verifies that claim-lane and post-claim dumps contain no `seed_phrase`, no `plaintext_b64`, and `secrets_redacted=true` in `crates/z00z_simulator/src/scenario_1/runner_verify.rs:1694` and `crates/z00z_simulator/src/scenario_1/runner_verify.rs:1710`.

### 🚫 Rejection Summary

- The strongest initial candidate was the canonical `scenario_1` helper path hardcoding `--features wallet_debug_tools` in `crates/z00z_storage/scripts/run_storage_settlement_bench.py:76`, but that path no longer survives admission because the live Stage 3 dump writer redacts `secrets[]` at source and `runner_verify.rs` rejects any claim-lane or post-claim debug dump that still contains wallet-secret fields.
- The helper freshness logic in `run_storage_settlement_bench.py` remains an evidence-discipline concern rather than a product-security break. It snapshots artifact `mtime` and size, reports freshness, and returns non-zero on stale Stage 13 artifacts, but it does not bypass proof verification, authorization, or secret redaction on its own.
- The batch benchmark timing ambiguity around `hjmt_batch_proof_bytes` is a correctness and measurement-authority issue, not an attacker-controlled proof-verification bypass or secret-handling leak. It therefore does not clear the security admission threshold for this scan.
- No parser, verifier, deletion-prior-context, or mixed-family gap survived the cryptographic skeptical pass. The live verifier validates deletion prior context down to definition/serial/bucket/terminal proof linkage at `crates/z00z_storage/src/settlement/proof_batch_verify.rs:492`, which closes the only plausible proof-binding shortcut found during review.

### 🧾 Verification Notes

- Scope authority reviewed: `.planning/phases/055-HJMT-boundary/055-TODO.md`, `.planning/phases/055-HJMT-boundary/055-CONTEXT.md`, `.planning/phases/055-HJMT-boundary/055-SECURITY.md`, and `.planning/phases/055-HJMT-boundary/055-TEST-SPEC.md`.
- Live owner-code reviewed: `proof_batch.rs`, `proof_batch_verify.rs`, `hjmt_batch_proof.rs`, `runner_verify.rs`, `stage_3_utils/post_claim.rs`, `fixture_cache.rs`, and `run_storage_settlement_bench.py`.
- Database action: no append performed because no finding was admitted.
