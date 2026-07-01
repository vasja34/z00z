# Bucket Commit Equivalence Manifest

Version: 2026-06-16

## 🎯 Purpose

This manifest is the exact checked artifact for the HJMT bucket-commit
equivalence seam.

- `BCM-G-001` proves that one same-bucket insert batch lands on the exact
  expected root and survives reload.
- `BCM-G-002` proves that delete-plus-replace under the same bucket lineage
  lands on the exact expected root and survives reload.

The manifest freezes:

- the old root;
- the exact operation batch;
- the touched bucket set;
- the expected new root;
- the reload-roundtrip requirement.

Regenerate with:

```bash
cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_batch_commit print_bucket_commit_manifest_json -- --ignored --nocapture
```

The live evidence owner is
`crates/z00z_storage/tests/test_hjmt_batch_commit.rs::test_bucket_commit_manifest_matches_live_contract`.

