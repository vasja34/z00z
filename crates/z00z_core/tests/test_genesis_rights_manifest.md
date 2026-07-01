# Genesis Rights Golden Manifest

`test_genesis_rights_manifest.json` is the approved golden summary consumed by
`crates/z00z_core/tests/test_genesis_rights.rs`.

## 📌 What The Test Compares

The test:

- loads the JSON with `include_str!`
- parses it once through `canonical_manifest()`
- rebuilds live canonical corpora from the current genesis config inputs
- compares the live summary against the approved manifest

The golden summary records these fields:

- `normalized_corpus_sha256`
- `rights_sha256`
- `rights_len`
- `state_hash`
- `rights_digest`

## ✅ Canonicalization Rule

`normalized_corpus_sha256` is derived from canonical JSON, not raw serialized
corpus output. The comparison intentionally removes `range_proof` and
`owner_signature` before hashing and writes keys in stable order.

This keeps the golden focused on semantic drift instead of volatile encoding
details.

## ⚠️ When To Update The JSON

Update `test_genesis_rights_manifest.json` only when an intentional and
approved genesis-contract change modifies the canonical summary.

Typical valid reasons:

- canonical genesis YAML changed
- rights derivation or digest rules changed intentionally
- state-hash computation changed intentionally
- the approved canonical summary schema changed

Do not update the JSON when:

- the failure is caused by nondeterminism
- the drift is unexpected
- the code is simply buggy
- ignored fields changed but the canonical summary should not have changed

## 🚀 Release Verification

```bash
cargo test --release -p z00z_core --test genesis_tests genesis_rights -- --nocapture
cargo test --release -p z00z_core --test test_genesis_manifest_goldens -- --nocapture
```
