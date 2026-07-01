# Deferred Items

## 2026-04-05

- Out of scope warning: [crates/z00z_storage/src/assets/store_internal/store_query.rs](/home/vadim/Projects/z00z/crates/z00z_storage/src/assets/store_internal/store_query.rs) still reports a pre-existing cyclomatic complexity issue on `keep_path` (`10 > 8`). The Phase 032 plan 03 edits only added `claim_source_contract_for_item(...)`; no new complexity warning was introduced by this task.
