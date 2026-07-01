---
phase: 053
slug: hjmt-backend
status: verified
threats_open: 0
asvs_level: 1
created: 2026-06-06
---

# Phase 053 - Security

> Per-phase security contract: threat register, accepted risks, and audit trail.

---

## Trust Boundaries

| Boundary | Description | Data Crossing |
|----------|-------------|---------------|
| Core/genesis -> settlement storage | Deterministic asset/right corpus enters the live settlement store. | Genesis asset rows, right rows, replay digests |
| Public settlement API -> HJMT internals | Callers request semantic paths, roots, and proofs while backend layout stays private. | `SettlementPath`, `SettlementStateRoot`, proof requests |
| Storage proof/verifier seam -> downstream consumers | Downstream crates consume only storage-owned proof blobs and semantic roots. | Proof blobs, checkpoint roots, snapshot witnesses |
| Scheduler/journal/recovery -> RedB persistence | Durable state must preserve recovery semantics and replay protections. | Journal digests, fee replay rows, policy ids, recovery checkpoints |
| Metrics/operator surfaces -> observers | Observability remains bounded diagnostics and never becomes proof authority. | Cache metrics, scheduler metrics, occupancy diagnostics |

---

## Threat Register

| Threat ID | Category | Component | Disposition | Mitigation | Status |
|-----------|----------|-----------|-------------|------------|--------|
| T-053-01 | Spoofing | guardrails | mitigate | Reject aliasing between old asset roots and settlement roots. | closed |
| T-053-02 | Tampering | public surface | mitigate | Keep physical ids private and force semantic settlement APIs. | closed |
| T-053-03 | Repudiation | docs tests | mitigate | Source-shape tests pin live vocabulary and live contract wording. | closed |
| T-053-04 | Spoofing | proof absence | mitigate | Non-existence proofs require transcript and default-commitment binding. | closed |
| T-053-05 | Tampering | proof binding | mitigate | Proof bytes bind root, family, and generation and reject rebinding. | closed |
| T-053-06 | Repudiation | deletion proofs | mitigate | Deletion proofs bind durable journal checkpoint state. | closed |
| T-053-07 | Information disclosure | occupancy payload | mitigate | Proof-visible occupancy excludes raw counts. | closed |
| T-053-08 | Linkability | occupancy timing | mitigate | Occupancy evidence stays threshold-bounded. | closed |
| T-053-09 | Tampering | occupancy policy | mitigate | Policy checks reject underreported occupancy evidence. | closed |
| T-053-10 | Tampering | scheduler ordering | mitigate | Sorted deterministic joins prevent root forks. | closed |
| T-053-11 | Denial of service | scheduler queues | mitigate | Bounded queue depth and backpressure protect resources. | closed |
| T-053-12 | Repudiation | scheduler cancellation | mitigate | Cancellation leaves no ambiguous committed journal state. | closed |
| T-053-13 | Tampering | journal recovery | mitigate | Missing or edited journal rows reject during reload and recovery. | closed |
| T-053-14 | Repudiation | crash recovery | mitigate | Recovery resolves to previous state or complete next state only. | closed |
| T-053-15 | Elevation of privilege | fee replay and policy rows | mitigate | Stale replay or policy rows reject before authorization. | closed |
| T-053-16 | Elevation of privilege | downstream wallet | mitigate | `RightLeaf` is rejected before asset ownership logic. | closed |
| T-053-17 | Tampering | downstream checkpoint | mitigate | Checkpoint code binds semantic settlement roots only. | closed |
| T-053-18 | Information disclosure | downstream layout | mitigate | Raw backend roots and tree layout stay hidden. | closed |
| T-053-19 | Tampering | corpus fixture | mitigate | Fixture loaders pin expected corpus digests to detect hand edits. | closed |
| T-053-20 | Spoofing | proof decoders | mitigate | Malformed proof bytes go through storage-owned decoders only. | closed |
| T-053-21 | Denial of service | decoder panic safety | mitigate | Malformed inputs return typed errors instead of panics. | closed |
| T-053-22 | Information disclosure | metrics | mitigate | Metrics stay bounded and never expose proof-authoritative raw counts. | closed |
| T-053-23 | Tampering | benches | mitigate | Bench lanes call live storage APIs instead of synthetic shortcuts. | closed |
| T-053-24 | Denial of service | backpressure metrics | mitigate | Queue overload stays observable through scheduler metrics. | closed |
| T-053-25 | Spoofing | alias purge | mitigate | Legacy aliases cannot present asset roots as settlement roots. | closed |
| T-053-26 | Tampering | compatibility purge | mitigate | Compatibility projection paths are removed or fail closed. | closed |
| T-053-27 | Repudiation | docs purge | mitigate | Purge tests reject stale docs that claim old runtime support. | closed |

---

## Threat Evidence

| Threat ID | Evidence |
|-----------|----------|
| T-053-01 | `crates/z00z_storage/tests/test_live_guardrails.rs:71-79,233-277` |
| T-053-02 | `crates/z00z_storage/tests/test_live_guardrails.rs:280-313`; `crates/z00z_storage/tests/test_downstream_guardrails.rs:115-140` |
| T-053-03 | `crates/z00z_storage/tests/test_live_guardrails.rs`; `crates/z00z_storage/tests/test_readme_examples.rs` |
| T-053-04 | `crates/z00z_storage/tests/test_hjmt_live_proof_families.rs:163-247` |
| T-053-05 | `crates/z00z_storage/tests/test_hjmt_live_proof_families.rs:273-295,499-559,1059-1115` |
| T-053-06 | `crates/z00z_storage/tests/test_hjmt_live_proof_families.rs:840-981` |
| T-053-07 | `crates/z00z_storage/src/settlement/types_record.rs:581-614`; `crates/z00z_storage/tests/test_occupancy_privacy.rs:200-233` |
| T-053-08 | `crates/z00z_storage/tests/test_occupancy_privacy.rs:237-293` |
| T-053-09 | `crates/z00z_storage/tests/test_occupancy_evidence.rs:196-241,265-289` |
| T-053-10 | `crates/z00z_storage/tests/test_async_scheduler.rs:175-208,318-350` |
| T-053-11 | `crates/z00z_storage/src/settlement/hjmt_scheduler.rs`; `crates/z00z_storage/tests/test_async_scheduler.rs` |
| T-053-12 | `crates/z00z_storage/tests/test_async_scheduler.rs:267-299` |
| T-053-13 | `crates/z00z_storage/src/settlement/live_recovery_tests.rs:457-487,516-545`; `crates/z00z_storage/tests/test_redb_reload.rs:1015-1023` |
| T-053-14 | `crates/z00z_storage/src/settlement/live_recovery_tests.rs:738-770,778-818` |
| T-053-15 | `crates/z00z_storage/tests/test_fee_replay.rs:438-445,484-493,558-569`; `crates/z00z_storage/src/settlement/live_recovery_tests.rs:550-575` |
| T-053-16 | `crates/z00z_wallets/tests/jmt_wallet_scan.rs:95-103,196-200,296-300`; `crates/z00z_storage/tests/test_downstream_guardrails.rs:156-158` |
| T-053-17 | `crates/z00z_storage/tests/test_checkpoint_root_binding.rs:150-225` |
| T-053-18 | `crates/z00z_storage/tests/test_live_guardrails.rs:280-313`; `crates/z00z_storage/tests/test_downstream_guardrails.rs:115-140` |
| T-053-19 | `crates/z00z_storage/tests/test_settlement_corpus_support.inc`; `crates/z00z_core/tests/genesis/test_settlement_corpus.rs`; `crates/z00z_storage/tests/test_property_corpus.rs` |
| T-053-20 | `crates/z00z_storage/tests/test_fuzz_seeds.rs:51-96,100-117` |
| T-053-21 | `crates/z00z_storage/tests/test_property_corpus.rs:345-386`; `crates/z00z_storage/tests/test_bench_lanes.rs:190-203` |
| T-053-22 | `crates/z00z_storage/tests/test_metrics.rs:22-70,73-136`; `crates/z00z_storage/tests/test_occupancy_privacy.rs:216-233` |
| T-053-23 | `crates/z00z_storage/tests/test_bench_lanes.rs:186-238,242-276` |
| T-053-24 | `crates/z00z_storage/src/settlement/hjmt_scheduler.rs`; `crates/z00z_storage/tests/test_async_scheduler.rs` |
| T-053-25 | `crates/z00z_storage/tests/test_live_guardrails.rs`; `crates/z00z_storage/tests/test_default_gate.rs` |
| T-053-26 | `crates/z00z_storage/tests/test_live_guardrails.rs`; `crates/z00z_storage/tests/test_default_gate.rs` |
| T-053-27 | `crates/z00z_storage/tests/test_live_guardrails.rs`; `crates/z00z_storage/tests/test_readme_examples.rs` |

Unregistered flags: none.

---

## Accepted Risks Log

No accepted risks.

---

## Security Audit Trail

| Audit Date | Threats Total | Closed | Open | Run By |
|------------|---------------|--------|------|--------|
| 2026-06-06 | 27 | 27 | 0 | Codex + gsd-security-auditor |

---

## Sign-Off

- [x] All threats have a disposition (mitigate / accept / transfer)
- [x] Accepted risks documented in Accepted Risks Log
- [x] `threats_open: 0` confirmed
- [x] `status: verified` set in frontmatter

**Approval:** verified 2026-06-06
