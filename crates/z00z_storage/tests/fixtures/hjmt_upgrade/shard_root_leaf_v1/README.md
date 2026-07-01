# Shard Root Leaf V1 Fixture Corpus

This directory is the Phase 058 authority for `ShardRootLeafV1` golden and
tamper vectors.

- The canonical case registry lives in `manifest.json`.
- Golden rows close `SRL-G-001` through `SRL-G-004`:
  bridge bytes, same-generation changed-leaf monotonicity, one legal
  transition-state flag vector, and deterministic re-encode stability.
- Tamper rows close `SRL-T-001` through `SRL-T-006`:
  reserved-transition-bit drift, stale route binding, stale shard epoch,
  decreasing journal checkpoint, decreasing local sequence, and digest mismatch.
- Tamper rows record `source_id`, `mutation_point`, `expected_stage`, and
  `expected_error` explicitly so the checked fixture corpus preserves the Phase
  058 completion contract instead of relying on prose-only interpretation.
- Monotonicity rows use the same live publication-successor rule as the
  production `CheckpointPublicationV1` contract. No second leaf-successor
  authority path is introduced.
