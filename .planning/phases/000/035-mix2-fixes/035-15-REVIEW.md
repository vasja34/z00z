---
phase: 035-mix2-fixes
reviewed: 2026-04-13T03:20:00+03:00
depth: deep
files_reviewed: 8
files_reviewed_list:
  - .planning/phases/035-mix2-fixes/035-15-PLAN.md
  - .planning/phases/035-mix2-fixes/035-TODO.md
  - .planning/phases/035-mix2-fixes/035-5-fix-spec.md
  - crates/z00z_wallets/tests/test_stealth_kdf_vectors.rs
  - crates/z00z_wallets/tests/fixtures/stealth_kdf_vectors.yaml
  - crates/z00z_core/src/assets/leaf.rs
  - crates/z00z_core/src/assets/leaf_tests.rs
  - crates/z00z_core/src/assets/mod.rs
findings:
  critical: 0
  warning: 0
  info: 0
  total: 0
status: clean
---

# Phase 035 Plan 15 Code Review Report

**Reviewed:** 2026-04-13T03:20:00+03:00
**Depth:** deep
**Files Reviewed:** 8
**Status:** clean

## Summary

Reviewed the Plan 15 derivation-vector and V2 memo boundary delta against the
Phase 035 tasks `035-35`, `035-36`, and `035-37`, focusing only on issues that
were introduced, left unresolved, or newly exposed by the current repository
changes.

The mandatory review loop ran five times on the live delta.

Earlier passes surfaced two real issues that were fixed before closure:

1. a serial-based decode shortcut wrongly inferred the asset-pack format from
   `serial_id`, which would have coupled a version boundary to unrelated serial
   semantics;
2. the new V2 memo boundary types were publicly Serde-deserializable, which
   allowed callers to bypass the bounded `memo <= 512` decode contract.

The final corrected delta keeps version-aware decode explicit through
`decode_asset_pack(bytes, version)`, preserves V1 behavior, exposes a concrete
`DecodedAssetPack::V2Memo(...)` branch, freezes the request-bound and card-bound
derivation vectors on the wallet test seam, and leaves wallet receive support
for V2 intentionally fail-closed until the later receive-path plan.

Final pass 4 was clean and final pass 5 was clean, which satisfies the closeout
rule of at least three review executions with at least two consecutive clean
passes before closure.

---

_Reviewed: 2026-04-13T03:20:00+03:00_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: deep_
