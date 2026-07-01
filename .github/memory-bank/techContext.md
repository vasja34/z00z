# Tech Context

## 🛠️ Core Stack

- Language: Rust
- Workspace edition baseline: 2021
- Workspace resolver: 2
- Workspace package version: 0.1.0
- Workspace rust-version: 1.90.0

## 📚 Main Workspace Members

- `crates/z00z_core`
- `crates/z00z_crypto`
- `crates/z00z_extensions`
- `crates/z00z_networks/onionnet`
- `crates/z00z_networks/rpc`
- `crates/z00z_rollup_node`
- `crates/z00z_runtime/aggregators`
- `crates/z00z_runtime/validators`
- `crates/z00z_runtime/watchers`
- `crates/z00z_simulator`
- `crates/z00z_storage`
- `crates/z00z_telemetry`
- `crates/z00z_utils`
- `crates/z00z_wallets`

## 🔐 Crypto And Dependency Notes

- Tari crypto dependencies are vendored locally through path dependencies
- `z00z_crypto` acts as the approved integration surface for Tari-backed crypto
- Common dependencies include `serde`, `thiserror`, and `anyhow`

## ⚙️ Tooling And Scripts

- Strong verification path:
  `./.github/skills/z00z-full-verify-gate/scripts/full_verify.sh`
- Fast bootstrap tests:
  `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- Supporting scripts live under `scripts/`
- Python and Bash are used for build orchestration and helper automation
- Phase planning and execution artifacts live under `.planning/`
- The repository currently uses phased plan documents and summary logs to track
  larger refactor and audit programs

## 📚 Recent Verified Phase Baseline

- Phase 025 closed the crypto audit around `claim_v2`, authoritative
  claim-source proofs, and default gating of legacy claim or experimental
  zkpack helper surfaces
- Phase 026 closed most `z00z_core` crypto-audit items around owner-signature
  binding, canonical fee identity, and fail-closed nonce helpers, but its
  validation still records one partial genesis-anchor gap because canonical
  protected-network anchor values are not yet populated
- Phase 027 closed `z00z_utils` hardening around memlock, bounded YAML,
  time-policy, deterministic RNG guardrails, logger hardening, and explicit
  JSON-compatibility policy
- Phase 028 closed `z00z_storage` checkpoint semantics, root binding, typed
  checkpoint identity, and binary `ClaimNullifier` replay state
- Phase 029 closed `z00z_wallets` crypto-contract hardening around view-key
  policy, KDF migration, backup metadata policy, validation warnings, and
  digest framing
- Phase 030 closed the long-file refactor program with a recorded zero-residue
  state for oversized files and synchronized planning closeout
- Phase 031 closed architecture-level facade narrowing, shim-retirement
  evidence, and the `z00z_utils` admission-policy note

## 🧪 Expected Validation Modes

- `cargo fmt`
- `cargo clippy --all-targets --all-features`
- `cargo test --all`
- Targeted crate-level tests for scoped changes
- Additional docs or script-specific checks when public surfaces change

## 🚧 Important Constraints

- English-only repository artifacts
- No direct modification of vendor Tari sources
- Safe file-operation discipline
- Repository-specific git versioning flow when version-managed releases are
  requested
- Dirty worktrees are common during planning-driven execution, so memory updates
  should distinguish active user changes from canonical completed repository
  state
