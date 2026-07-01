# Checkpoint Publication V1 Fixture Corpus

This directory is the Phase 058 authority for `CheckpointPublicationV1` golden
and tamper vectors.

- The canonical case registry lives in `manifest.json`.
- Golden rows close `CPP-G-001` through `CPP-G-005`:
  bridge publication bytes, byte-identical carry-forward for unchanged shard
  leaves, changed-one-shard successor closure, route-generation transition
  publication, and the exact `prior_public_root` chain.
- Tamper rows close `CPP-T-001` through `CPP-T-007`:
  reordered shard leaves, missing active shard leaf under the same route,
  duplicate shard leaf, route-digest drift, wrong prior public root, invalid
  root-generation tag, and one-byte mutation of a carried-forward unchanged
  shard leaf.
- Tamper rows record `source_id`, `mutation_point`, `expected_stage`, and
  `expected_error` explicitly so the checked fixture corpus preserves the Phase
  058 completion contract instead of relying on prose-only interpretation.
- Same-route successor checks now fail closed when a previously active shard
  disappears from the next publication. Route-generation migration still stays
  on the same production contract path instead of using a fixture-only rule.
