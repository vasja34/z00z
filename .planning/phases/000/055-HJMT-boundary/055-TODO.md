Ниже та же **рекомендуемая последовательность**, но уже с точным составом section-ов, которые должны входить в каждую phase из:

- `Z00Z-HJMT-Upgrade.md`
- `Z00Z-HJMT-Fixture-Checklist.md`

Сразу дам важное правило чтения:

**Есть 2 типа section-ов**

- `Primary sections`: это то, что phase реализует как свой главный deliverable.
- `Mandatory cross-read sections`: это section-ы, которые не являются главным deliverable этой phase, но их нужно держать в scope, чтобы не сломать контракт, fail-closed поведение, readiness или evidence discipline.

Именно из-за этого некоторые section-ы будут пересекаться между phase. Это нормально и по этой спецификации даже обязательно.

**Глобальные section-ы, которые должны считаться активными во всех phase**

Эти section-ы не принадлежат одной конкретной phase, а должны читаться и применяться в каждой:

Из `Z00Z-HJMT-Upgrade.md`:

- `Key Terms Used In This Paper`
- `1.1 Inherited Base Constraints`
- `1.2 Prohibited Changes`
- `1.3 Verified Current Baseline`
- `2.1 HJMT Remains The State Core`
- `2.2 Optimize Inside The Existing Paradigm`
- `2.3 Fail Closed`
- `2.4 Narrow Versioned Contracts`
- `2.5 Commitment Boundary`
- `2.6 Contract Discipline`
- `10. Correctness, Security, And Privacy Checklist`
- `10.1 Evidence Mapping Discipline`
- `13. Required Decisions And Fail-Closed Rules`
- `13.1 Fail-Closed Discipline`
- `14. Readiness Definition`
- `14.1 Completion Discipline`
- `Appendix A. Normative Upgrade Requirements`
- `Appendix E.4 Review Checklist For Implementation PRs`
- `Appendix E.5 Evidence Needed For Conformance-Safe Execution`

Из Z00Z-HJMT-Fixture-Checklist.md:

- `Completion Contract`

- `Release Gate`




## **1. Phase 1. Contract Freeze And Artifact Pack**

Логическая цель: зафиксировать нормативную рамку, ownership, boundary rules, evidence model и execution order до начала собственно build work.

`Primary sections` из Upgrade:

- `1. Purpose And Upgrade Boundary`
- `1.4 Core Architecture Decision`
- `1.5 Parallel Shard Reality Check`
- `1.6 Upgrade Boundary And Evidence Discipline`
- `1.7 Whole-System Structure View`
- `1.7.1 C4 Component View: Whole-System Roles`
- `1.8 C4 Component Reading Map`
- `9. Scorecard And Measurement Plan`
- `11. Implementation Roadmap`
- `11.1 Roadmap Dependency Discipline`
- `11.1.1 Mermaid Flow View: Upgrade Dependency Chain`
- `12. Test And Benchmark Plan`
- `12.1 Evidence Gaps`
- `Appendix B. Repository Evidence Map`
- `Appendix C. Design Artifact Requirements`
- `Appendix E.1 Suggested Module Boundaries`
- `Appendix E.2 First Slice Implementation Order`
- `Appendix E.3 Test Vector Layout`
- `Appendix E.6 Cross-Crate Module Ownership`
- `Appendix E.7 Cross-Crate Execution Order`
- `Appendix F. Discussion Coverage Matrix`
- `Appendix F.1 Traceability For Sharding And Storage Recommendations`

`Mandatory cross-read sections`:

- все глобальные section-ы выше
- особенно `13`/`14`, чтобы сразу не построить план, который нельзя честно закрыть

Из Fixture Checklist в scope этой phase:

- `Completion Contract`
- `Release Gate`

Практический смысл:

- эта phase не делает core feature code, а делает обязательный implementation contract pack:
  какие artifacts должны существовать,
  какие fixtures обязательны,
  какой порядок фаз допустим,
  кто владелец runtime/storage/public truth,
  какие claims вообще можно будет делать в конце.

**1. Phase -> expected deliverables**

1. `Phase 1. Contract Freeze And Artifact Pack` -> frozen implementation contract: phase order, dependency order, explicit non-goals, fail-closed boundaries, ownership boundaries между `z00z_rollup_node`, runtime services и `z00z_storage`; artifact/planning pack: layout diagram plan для `BatchProofBlob`, compatibility matrix plan для `ProofBlob` / `Vec<ProofBlob>` / `BatchProofBlob`, root-generation migration vector plan, conformance plan для `StorageBackend` + `JournalBackend`, multi-aggregator simulation plan, benchmark report template, fixture layout plan; cross-crate ownership map; acceptance vocabulary for `Specified contract`, `Prototype`, `Verified slice`, `Integrated upgrade`, `Release-ready`.

**2. Phase -> required tests/benchmarks**

`Phase 1` -> runnable coverage не обязательна как acceptance gate, но обязателен inventory definition с owner homes для tests: `test_hjmt_batch_proof.rs`, `test_hjmt_batch_proof_negative.rs`, `test_hjmt_batch_commit.rs`, `test_hjmt_batch_recovery.rs`, `test_hjmt_storage_boundary.rs`, `test_hjmt_backend_conformance.rs`, `test_hjmt_shard_routing.rs`, `test_hjmt_failover_same_lineage.rs`, `test_hjmt_split_brain_fencing.rs`, `test_hjmt_multi_aggregator_sim.rs`, `test_hjmt_root_generation.rs`, `test_hjmt_historical_proofs.rs`, `test_hjmt_transition_proofs.rs`, `test_hjmt_privacy_regression.rs`; benchmark homes: `hjmt_batch_proof_bytes.rs`, `hjmt_batch_verify.rs`, `hjmt_bucket_delta_commit.rs`, `hjmt_backend_boundary.rs`, `hjmt_shard_parallel_commit.rs`, `hjmt_root_of_roots_publish.rs`, `hjmt_transition_locality.rs`.



**3. Phase -> exact fixture IDs from checklist**

1. `Phase 1` -> `Completion Contract`, `Release Gate`.



## **2. Phase 2. Batch Proof Contract And Shared Multiproof**

Логическая цель: построить и зацементировать byte-level и verifier-level shared proof contract, не ломая текущий `ProofBlob`.

`Primary sections` из Upgrade:

- `3. Upgrade 1: Shared Hierarchical Multiproof`
- `3.1 Required Format`
- `3.1.1 Exact Codec Contract For BatchProofBlobV1`
- `3.1.2 Exact Codec Contract For Nested Batch Tables`
- `3.2 Canonical Ordering`
- `3.3 Verification Algorithm`
- `3.4 Witness Reuse Rules`
- `3.5 Acceptance Evidence`
- `3.6 Verifier Safety Requirements`
- `3.7 Implementation Guidance`
- `3.8 C4 Component View: Batch Proof Contract`
- `Appendix D.1 Batch Proof Envelope Skeleton`
- `Appendix D.2 Fail-Closed Batch Verifier Skeleton`

`Mandatory cross-read sections`:

- `9.1 Benchmark Matrix`
- `9.2 Claim Gate`
- `9.3 Score Claim Discipline`
- `12. Test And Benchmark Plan`
- `12.1 Evidence Gaps`
- `Appendix E.2 First Slice Implementation Order`
- `Appendix E.3 Test Vector Layout`

Из Fixture Checklist в этой phase `primary ownership`:

- `8. BatchProofBlobV1 Golden Vectors`
- `9. BatchProofBlobV1 Tamper Vectors`

Из Fixture Checklist как cross-read:

- `Completion Contract`
- `Release Gate`

Практический смысл:

- после этой phase должен существовать уже не просто “идея batch proof”, а канонический формат, deterministic codec, fail-closed verifier, golden/tamper vectors и baseline benchmarks против `Vec<ProofBlob>`.

**1. Phase -> expected deliverables**

1. `Phase 2. Batch Proof Contract And Shared Multiproof` -> `BatchProofBlobV1` canonical format; exact codec contract для batch header, path table, witness DAG, opening table, reference table; frozen family-specific opening payload contracts; deterministic canonical ordering rules; fail-closed parser limits; atomic verifier без partial acceptance; compatibility preservation: `ProofBlob` unchanged, `BatchProofBlobV1` as separate envelope; first shared witness-reuse implementation; baseline comparison against `Vec<ProofBlob>`.

**2. Phase -> required tests/benchmarks**

1. `Phase 2` -> required tests: `test_hjmt_batch_proof.rs`, `test_hjmt_batch_proof_negative.rs`; required benchmarks: `hjmt_batch_proof_bytes.rs`, `hjmt_batch_verify.rs`; required scenario coverage inside tests: deterministic encoding, parser size limits, canonical ordering reject, duplicate-path reject, mixed proof-family reject, opening family mismatch reject, witness reference bound checks.

**3. Phase -> exact fixture IDs from checklist**

1. `Phase 2` -> golden fixtures `BPB-G-001`, `BPB-G-002`, `BPB-G-003`, `BPB-G-004`, `BPB-G-005`; tamper fixtures `BPB-T-001`, `BPB-T-002`, `BPB-T-003`, `BPB-T-004`, `BPB-T-005`, `BPB-T-006`, `BPB-T-007`, `BPB-T-008`.
