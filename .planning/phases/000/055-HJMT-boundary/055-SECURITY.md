---
phase: 055
slug: 055-hjmt-boundary
status: verified
threats_open: 0
asvs_level: 1
created: 2026-06-11
register_authored_at_plan_time: true
---

# Phase 055 — Security

> Retroactive threat verification created from executed Phase 055 plan and
> summary artifacts plus the live storage and simulator code paths.

---

## Trust Boundaries

| Boundary | Description | Data Crossing |
|----------|-------------|---------------|
| Batch proof bytes -> storage verifier | Untrusted serialized `BatchProofBlobV1` enters the fail-closed decode and verification path. | Proof bytes, path/opening/witness tables, root bindings |
| Storage single-proof truth -> batch builder | The additive batch surface is derived from live `ProofBlob` / non-existence proof generation rather than a second proof engine. | Current settlement proofs, prior-context facts, witness DAG material |
| Bench helper -> canonical settlement bench home | Helper-selected note scope controls whether side outputs emit full proof notes, batch-only notes, or no note authority. | Filter selectors, note-scope env vars, bounded evidence rows |
| Stage 13 artifact pack -> runner verification | Scenario artifacts are reloaded and fail-closed against required comparison, proof-size, replay, and tamper invariants. | JSON reports, typed errors, batch comparison rows, root bindings |
| Shared fixture cache -> Stage 13 consumer tests | Expensive scenario evidence is reused only when precise fingerprints match the current binary and source scope. | Cached scenario outputs, fingerprints, stage artifacts |

---

## Threat Register

| Threat ID | Category | Component | Disposition | Mitigation | Status |
|-----------|----------|-----------|-------------|------------|--------|
| T-055-01 | Tampering | `BatchProofBlobV1` wire contract | mitigate | Deterministic family/domain tags, explicit limits, and exact decode/re-encode contract in `crates/z00z_storage/src/settlement/proof_batch.rs`; contract tests in `crates/z00z_storage/tests/test_hjmt_batch_proof.rs`. | closed |
| T-055-02 | Spoofing | proof-format ownership | mitigate | Canonical additive export stays storage-owned via `SettlementStore::{settlement_inclusion_batch_v1, settlement_nonexistence_batch_v1, settlement_deletion_batch_v1}` in `crates/z00z_storage/src/settlement/hjmt_batch_proof.rs`; guardrails reject public compat/shadow exports in `crates/z00z_storage/tests/test_live_guardrails.rs`. | closed |
| T-055-03 | Elevation of privilege | future shard/route context | mitigate | Live V1 verifier rejects partial or non-live shard context and shard/global witness domains in `crates/z00z_storage/src/settlement/proof_batch_verify.rs`; enforced by `test_batch_proof_v1_rejects_partial_shard_context` and `test_batch_proof_v1_rejects_shard_witness_domain_without_sharding`. | closed |
| T-055-04 | Tampering | batch verifier parser/header/opening path | mitigate | `check_batch_contract_v1` validates header, ordering, exact table usage, opening kinds, witness structure, transcript, and root folding in `crates/z00z_storage/src/settlement/proof_batch_verify.rs`; reject coverage lives in `crates/z00z_storage/tests/test_hjmt_batch_proof.rs` and `crates/z00z_storage/tests/test_hjmt_batch_proof_negative.rs`. | closed |
| T-055-05 | Repudiation | atomic verification semantics | mitigate | No partial-acceptance surface exists: verification is `Result<(), ProofChkErr>` and Stage 13 requires `atomic_verdict == accepted` for every batch row in `crates/z00z_simulator/src/scenario_1/runner_verify.rs`; drift tests live in `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs`. | closed |
| T-055-06 | Denial of service | oversized / malformed batch payloads | mitigate | Byte ceilings and table-count bounds are enforced by `BatchProofLimits` and `decode_with_limits`, then index and exact-usage checks fail early in `crates/z00z_storage/src/settlement/proof_batch.rs` and `crates/z00z_storage/src/settlement/proof_batch_verify.rs`; oversized and out-of-bounds cases are tested in `crates/z00z_storage/tests/test_hjmt_batch_proof.rs`. | closed |
| T-055-07 | Spoofing | second proof engine drift | mitigate | Batch builder derives every row from live `settlement_proof_blob` / `settlement_nonexistence_proof_blob` and validates each source blob before lowering in `crates/z00z_storage/src/settlement/hjmt_batch_proof.rs`; summaries and guardrails keep `ProofBlob`, `Vec<ProofBlob>`, and `BatchProofBlobV1` visible together. | closed |
| T-055-08 | Tampering | checked-in fixture provenance | mitigate | Positive and negative manifests carry `regen_command` and `evidence_pointer`, positive bytes are decoded from checked-in authorities, and negative mutations must reject with exact typed errors in `crates/z00z_storage/tests/test_hjmt_batch_proof.rs` and `crates/z00z_storage/tests/test_hjmt_batch_proof_negative.rs`; shared Stage 13 builds are fingerprinted and contract-tested by `crates/z00z_simulator/src/test_support/fixture_cache.rs` and `crates/z00z_simulator/tests/test_fixture_cache_contract.rs`. | closed |
| T-055-09 | Downgrade | baseline replacement | mitigate | Stage 13 and bench evidence require all three surfaces `proof_blob_single`, `proof_blob_vec`, and `batch_proof_v1` rather than letting batch proofs replace the baseline; checks live in `crates/z00z_simulator/src/scenario_1/runner_verify.rs`, `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs`, and `crates/z00z_storage/benches/settlement_benches.md`. | closed |
| T-055-10 | Repudiation | evidence-free phase closeout | mitigate | One canonical shared Stage 13 fixture is built through the live runner in `crates/z00z_simulator/src/test_support/stage13_shared_cases.rs`, `crates/z00z_simulator/src/scenario_1/runner_verify.rs` fails closed if comparison, proof-size, replay, or tamper evidence drifts, and Phase 055 closeout summaries keep that artifact pack authoritative in `.planning/phases/055-HJMT-boundary/055-04-SUMMARY.md` and `.planning/phases/055-HJMT-boundary/055-SUMMARY.md`. | closed |
| T-055-11 | Information disclosure | benchmark metrics and scenario errors | mitigate | Batch note scope records bounded semantics only in `crates/z00z_storage/benches/settlement_proofs.rs` and `crates/z00z_storage/src/fixture_support/settlement_bench_output.rs`; Stage 13 typed errors are redacted and bounded by `verify_redacted_error` / `metrics.validate_bounded()` in `crates/z00z_simulator/src/scenario_1/runner_verify.rs`. | closed |
| T-055-12 | Tampering | benchmark / scenario claim drift | mitigate | Canonical batch-only note scope, representative live counts `{2,8,32}`, and verify-only skip policy are enforced by `crates/z00z_storage/benches/settlement_proofs.rs`, `crates/z00z_storage/tests/test_bench_lanes.rs`, and `crates/z00z_storage/benches/settlement_benches.md`; Stage 13 also fails if batch shapes, counts, families, rows, or tamper cases drift. | closed |

*Status: open · closed*
*Disposition: mitigate (implementation required) · accept (documented risk) · transfer (third-party)*

---

## Accepted Risks Log

No accepted risks.

---

## Security Audit Trail

| Audit Date | Threats Total | Closed | Open | Run By |
|------------|---------------|--------|------|--------|
| 2026-06-11 | 12 | 12 | 0 | Codex |

---

## Sign-Off

- [x] All threats have a disposition (mitigate / accept / transfer)
- [x] Accepted risks documented in Accepted Risks Log
- [x] `threats_open: 0` confirmed
- [x] `status: verified` set in frontmatter

**Approval:** verified 2026-06-11
