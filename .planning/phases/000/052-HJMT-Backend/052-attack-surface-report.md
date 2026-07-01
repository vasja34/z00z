# Attack Surface Scan

## Metadata

- Generated At: 2026-05-29T12:10:04Z
- Scope: `docs/Z00Z-JMT-Design.md`, `.planning/phases/052-HJMT-Backend/052-TODO.md`
- Requested Report: `.planning/phases/052-HJMT-Backend/052-attack-surface-report.md`
- Requested DB: `.planning/phases/052-HJMT-Backend/052-attack-surface-db.jsonl`
- Max Variants: 20

## Boundary Slice Map

- external input and parser slice: present via proof-blob decode and env parsing
- cryptographic and proof-verification slice: present via forest proof envelope and root-bind verification
- replay, nonce, uniqueness, and state-consumption slice: present via claim-source and checkpoint-bound asset-state handling
- configuration, feature-flag, and deployment-default slice: present via backend-mode and bucket-policy env surface
- operator, admin, and debug-only surface slice: present via wallet commit-audit diagnostics

## Scan Result

No candidate passed the pro-con audit and uniqueness gate.

### Rejection Summary

- `backend-root-diagnostic-export`: `ProofScanOut` and the wallet asset-class audit intentionally export proof-local `backend_root`, which is a real misuse seam, but the live sink re-checks semantic root, `root_bind`, and leaf hash before accepting the data, and the repository keeps explicit guardrails that this value is diagnostic only rather than state authority. This stayed below the acceptance threshold for a distinct admitted vulnerability.
- `env-selectable-backend-mode`: `Z00Z_ASSET_BACKEND_MODE` and `Z00Z_ASSET_BUCKET_BITS` are operator-controlled deployment seams, but the code defaults to compatibility mode, validates unsupported modes fail-closed, and bounds bucket-policy construction. The remaining risk is operational misconfiguration rather than an attacker-controlled exploit path that crosses the Phase 052 trust boundary.
- `unsupported-proof-family-gap`: deletion and non-existence proofs remain intentionally unsupported in the live forest family, but the implementation rejects those proof families fail-closed instead of attempting partial verification or fallback semantics. That is a deferred feature boundary, not an exploitable verification bypass in the current tree.
- `bucket-policy-metadata-visibility`: forest proof envelopes do carry verifier-visible bucket metadata, but the code binds the committed bucket policy and bucket root leaf into proof verification and rejects tampering. The scan did not find a realistic path where that metadata becomes a new authorization, integrity, or privacy break beyond the already explicit proof contract.

### Live Evidence Reviewed

- `crates/z00z_storage/src/assets/proof.rs:39` - compatibility and forest proof families reject unsupported deletion and non-existence semantics
- `crates/z00z_storage/src/assets/proof.rs:419` - root binding is re-checked against `AssetStateRoot` and proof-local backend root
- `crates/z00z_storage/src/assets/proof.rs:664` - forest verification binds bucket policy, bucket root leaf, bucket proof, and asset proof in order
- `crates/z00z_storage/src/assets/store_internal/forest_config.rs:7` - backend mode and bucket bits are env-driven but parsed through fail-closed validation
- `crates/z00z_storage/src/assets/types_identity.rs:151` - bucket policy construction enforces bounded bucket parameters and stable policy ids
- `crates/z00z_wallets/src/tx/commit_audit.rs:307` - wallet audit captures `backend_root` and `root_bind`
- `crates/z00z_wallets/src/tx/commit_audit.rs:414` - wallet audit rejects mismatched backend-root or root-bind data and re-checks both `check_root_bind()` and `check_leaf_hash()`
- `crates/z00z_storage/tests/test_phase052_guardrails.rs:331` - guardrail coverage keeps wallet-facing backend-root usage explicitly diagnostic
- `crates/z00z_storage/tests/test_phase052_forest_proofs.rs:321` - deletion and absence proof families are still rejected as unsupported

### Notes

- The requested scope was documentation-first, but live code and tests were inspected because doc-only evidence is not enough for admitted attack-surface findings.
- No accepted finding was strong enough to justify a new append-only JSONL record.
- `.planning/phases/052-HJMT-Backend/052-attack-surface-db.jsonl` was therefore not created or modified in this run.
