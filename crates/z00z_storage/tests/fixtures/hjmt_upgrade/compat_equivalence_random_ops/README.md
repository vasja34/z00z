# Compatibility Equivalence Random-Ops Manifest

Version: 2026-06-16

## 🎯 Purpose

This manifest freezes the exact deterministic replacement for the old
`compat_equivalence_random_ops` wording.

Each case:

- seeds the canonical settlement corpus;
- runs one fixed-seed mixed operation sequence;
- checks semantic parity against `OracleState`;
- reloads from durable storage and rechecks the same final root and row set.

The checked owner home is
`crates/z00z_storage/tests/test_hjmt_compat_equivalence.rs::test_manifest_matches_contract`.

Regenerate with:

```bash
Z00Z_REGEN_DUMP=1 cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_compat_equivalence test_manifest_matches_contract -- --exact --nocapture
```

This is a conformance lane, not a throughput claim. It exists to prove semantic
parity and restart safety under deterministic mixed operations.

