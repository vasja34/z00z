# Failover V1 Corpus

This directory is the live failover evidence home carried from Phase 056 into
Phase 057 for `FOV-001`, `FOV-T-001`, `FOV-T-002`, and `FOV-G-002` through
`FOV-G-004`.

- The canonical failover registry lives in `manifest.json`.
- `Failover fixture` rows cover the lawful same-lineage takeover plus the
  wrong-lineage, wrong-generation, stale-local-root, stale-restart,
  secondary-down, and split-brain reject rows.
- `Carry-forward fixture` rows cover byte-identical carried-forward shard-leaf
  bytes plus the exact public-root digest emitted by the lawful successor
  publication.
- `Crash fixture` rows cover the prior-visible versus exact-durable-successor
  public-root outcome after a durable parent-stage crash.
- `Route migration fixture` covers the crash-time route-migration reject row
  plus the requirement that no partial migration public root becomes visible.
- Every case records one explicit verdict, one shard or generation context,
  one regeneration command, one evidence pointer back to the live runtime
  tests, and exact public-root or carried-forward-leaf bytes where the Phase
  057 fixture checklist requires them.
