# Shard Route Table V1 Corpus

This directory is the Phase 056 route-table evidence home for `SRT-G-001`
through `SRT-G-004` and `SRT-T-001` through `SRT-T-008`.

- The canonical case registry lives in `manifest.json`.
- Every case is generated from the live runtime-owned `ShardRouteTableV1`
  encoder and decoder in `crates/z00z_runtime/aggregators/src/batch_planner.rs`.
- Golden rows freeze canonical bytes, digest, migration linkage, and re-encode
  stability.
- Tamper rows mutate one contract field at a time and prove parser, migration,
  or digest-side rejection without introducing a second planner authority path.
