# Z00Z HJMT Gaps

Version: 2026-06-16

## 🎯 Scope

This file records the closeout of the previously open HJMT gaps after Phases
055, 056, 057, and 058.

Current honest repository verdict: `integrated upgrade`.

This file now tracks closure state, not an open-gap backlog. `release-ready`
remains a stricter final ship verdict and is not asserted here.

## ✅ Already Landed

The following packets are implemented and remain canonical:

- Phase 055 closed the storage-owned `BatchProofBlobV1` boundary, fail-closed
  verifier and builder, fixture corpus, and canonical batch-proof evidence.
- Phase 056 closed the canonical `SIM-5A7S` runtime, runtime-owned planner
  truth, storage-owned semantic handoff, same-lineage failover, and
  simulator-backed observability.
- Phase 057 closed the canonical root-of-shard-roots publication lineage,
  publication-binding ownership path, and live multi-aggregator publication
  evidence.
- Phase 058 closed the final evidence ledger, benchmark vocabulary,
  archive-home wording, fixture-family ledger, and integrated-upgrade
  classification.
- Fixture families `SRT`, `SRL`, `CPP`, `FOV`, `BPB`, and `RGM` all have
  checked live anchors in the repository evidence ledger and checklist.

## 🚨 Real Remaining Work

No open items remain in this section.

The previously open rows are now closed on exact owner homes:

1. `Shared proof vector` now closes on
   `.planning/phases/058-HJMT-benchmarks/058-SHARED-PROOF-REPORT.md` plus the
   positive and negative `BatchProofBlobV1` manifests.
2. `Bucket commit fixture` now closes on
   `crates/z00z_storage/tests/fixtures/hjmt_upgrade/bucket_commit_equivalence/manifest.json`
   and `crates/z00z_storage/tests/test_hjmt_batch_commit.rs`.
3. The archive-home ambiguity is closed by retiring
   `crates/z00z_storage/outputs/assets/` and keeping
   `crates/z00z_storage/outputs/settlement/` as the one canonical measured
   report home.
4. `compat_equivalence_random_ops` now closes on
   `crates/z00z_storage/tests/fixtures/hjmt_upgrade/compat_equivalence_random_ops/manifest.json`
   and `crates/z00z_storage/tests/test_hjmt_compat_equivalence.rs` as an exact
   deterministic conformance replacement.
5. Appendix C `C-14` now closes on
   [Z00Z-HJMT-Threat-Model.md](Z00Z-HJMT-Threat-Model.md).
6. Appendix C `C-16` now closes on
   [Z00Z-HJMT-Acceptance-Thresholds.md](Z00Z-HJMT-Acceptance-Thresholds.md).

## ⚠️ Non-Blocking But Recommended Formalization

No open items remain in this section.

The previously recommended formalization items are now closed:

1. Appendix C `C-04` is promoted to the standalone
   [Z00Z-HJMT-Proof-Compatibility-Matrix.md](Z00Z-HJMT-Proof-Compatibility-Matrix.md)
   artifact.
2. Compression claims remain explicitly unsupported without a versioned
   compression lane; the benchmark and threshold packet now freeze that rule on
   the current canonical owner homes.

## 📝 Documentation Synchronization Gaps

No open items remain in this section.

The tech-paper set is synchronized as follows:

### ✅ `Z00Z-HJMT-Design.md`

- benchmark-plan wording now closes `commit_recovery_replay` and
  `compat_equivalence_random_ops` on exact deterministic owner homes;
- measured report archives now point only to
  `crates/z00z_storage/outputs/settlement/`.

### ✅ `Z00Z-HJMT-Fixture-Checklist.md`

- checklist boxes now reflect the live closed fixture packet;
- `RGM` is now present as an explicit fixture family;
- the release gate now reflects the closed repository packet.

### ✅ `Z00Z-HJMT-Key-Terms.md`

- the glossary now includes the `RGM` abbreviation.

### ✅ `Z00Z-HJMT-Upgrade.md`

- Appendix `12.1` and Appendix `C` now point to the standalone
  proof-compatibility, threat-model, acceptance-threshold, shared-proof, and
  bucket-commit owner artifacts;
- Appendix `E.5` no longer describes those artifacts as still missing.

## 🔚 Exit Condition

The HJMT gap set tracked by this file is closed. The repository now satisfies
the Phase 058 integrated-upgrade closeout packet with one canonical evidence
path and no remaining open rows in this document.
