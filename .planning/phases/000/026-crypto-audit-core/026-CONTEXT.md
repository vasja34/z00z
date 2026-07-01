# Phase 026: Crypto Audit Core - Context

<!-- markdownlint-disable MD001 MD022 MD033 -->

**Gathered:** 2026-03-28
**Status:** Ready for planning
**Source:** PRD Express Path (026-FUSION.md)

<domain>
## Phase Boundary

- This phase remediates the confirmed and near-confirmed `z00z_core` integrity gaps captured in `026-FUSION.md`.
- The phase covers protected-network genesis anchoring, canonical asset-definition identity, full-payload registry hashing, untrusted wire and DTO boundaries, ownership and stealth binding, native fee-asset enforcement, nonce fail-closed behavior, and proof-width-aware amount policy.
- The phase stays inside `crates/z00z_core/src/**` and uses existing workspace building blocks instead of adding a new crate.

</domain>

<decisions>
## Implementation Decisions

### Genesis and Network Trust

- Protected networks MUST fail closed when the expected genesis-state hash is missing or mismatched.
- Mainnet and testnet parsing MUST reject unknown chain types; no fallback-to-devnet behavior is allowed in protected flows.
- Seed validation MUST use explicit fail-closed policy checks instead of a Shannon-threshold approval gate on 32-byte samples.

### Definition and Registry Integrity

- Asset-definition identity MUST be derived in one canonical helper from framed definition payload fields and MUST NOT be trusted from config, wire, or import input.
- Registry snapshot integrity MUST hash the full ordered canonical definition payload, not only definition IDs.
- Test-only asset-id domains MUST stay confined to test-only code paths.

### Untrusted Wire and DTO Boundaries

- Untrusted wire payloads MUST NOT inject `secret` or other trusted-only confidential material.
- Public or untrusted DTO decode MUST either preserve `is_frozen` and `is_slashed` faithfully or reject payloads that cannot carry them.
- Wire-to-domain conversion MUST validate canonical definition identity before registry insertion.

### Ownership, Stealth, and Fee Authority

- `owner_signature` MUST be treated as authority over canonical asset state, not as a standalone proof of commitment-opening ownership.
- Stealth-critical fields `r_pub`, `owner_tag`, `enc_pack`, `tag16`, and `leaf_ad_id` MUST be signed or otherwise bound by verifier-checked canonical state.
- Fee validation MUST enforce the canonical native coin identity as well as `AssetClass::Coin`.

### Nonce and Amount Policy

- Production nonce helpers MUST return typed errors on time-provider failures and MUST NOT substitute timestamp `0`.
- The uniqueness and persistence contract around `NonceCounter` MUST be explicit in production-facing helpers and tests.
- `MAX_AMOUNT` MUST be derived from the supported proof-width policy instead of `u64::MAX`.

### the agent's Discretion

- Exact helper and type names, local refactor shape, and whether to introduce small internal helper structs or functions.
- Whether state-preserving DTO behavior is implemented by full parity serialization or by explicit decode rejection.
- Whether native fee-asset identity is enforced by canonical definition ID helper, registry lookup, or one sealed config constant, as long as the check is authoritative and test-covered.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Audit Source Of Truth

- `.planning/phases/026-crypto-audit-core/026-FUSION.md` — Canonical fused audit findings, blocker ordering, and target architecture.
- `.planning/phases/026-crypto-audit-core/FUSION.audit.md` — Supporting fused audit artifact for cross-checking wording and scope.

### Genesis And Network Integrity

- `crates/z00z_core/src/genesis/validator.rs` — Genesis consensus verification, chain detection, and current seed validation policy.
- `crates/z00z_core/src/genesis/genesis.rs` — `run_genesis`, `GenesisSeed`, and genesis asset-definition creation flow.
- `crates/z00z_core/src/genesis/genesis_config.rs` — Config parsing boundary for chain type and seed inputs.

### Definition And Registry Integrity

- `crates/z00z_core/src/assets/definition.rs` — Asset-definition construction and validation boundary.
- `crates/z00z_core/src/assets/snapshot.rs` — Registry snapshot hash and version contract.
- `crates/z00z_core/src/assets/registry.rs` — Snapshot creation, update, and registry verification flow.
- `crates/z00z_core/src/hashing.rs` — Canonical domain hasher aliases, including asset-id and registry domains.

### Wire And DTO Boundaries

- `crates/z00z_core/src/assets/wire.rs` — Trusted wire/domain conversion boundary and `AssetWire` / `DefinitionWire` shapes.
- `crates/z00z_core/src/assets/wire_pkg.rs` — JSON/Bincode DTO boundary and secret-field gate helpers.
- `crates/z00z_core/src/assets/test_wire.rs` — Existing wire/DTO tests that must be extended instead of bypassed.

### Ownership, Fee, Nonce, And Amount Policy

- `crates/z00z_core/src/assets/assets.rs` — Canonical owner message, owner-signature verification, and stealth-field consistency.
- `crates/z00z_core/src/assets/gas.rs` — Native fee asset validation.
- `crates/z00z_core/src/assets/nonce.rs` — Timestamp helpers, nonce derivation, and `NonceCounter` contract.
- `crates/z00z_core/src/assets/amount.rs` — `MAX_AMOUNT` source of truth.

</canonical_refs>

<specifics>
## Specific Ideas

- Every Rust or test-affecting task MUST run `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` first as the fail-fast gate.
- Every Rust or test-affecting task MUST run `cargo test --release --features test-fast --features wallet_debug_dump` when the broader validation is relevant after the bootstrap gate is green.
- Every `<task type="auto">` verify flow MUST require `/.github/prompts/gsd-review-tasks-execution.prompt.md` in YOLO mode at least three times, and it can stop only after two consecutive runs report no significant issues.
- If any plan needs a git checkpoint, it MUST use the repository-owned `z00z-git-versioning` workflow instead of ad hoc git commands.

</specifics>

<deferred>
## Deferred Ideas

- Phase-3 cleanup items from `026-FUSION.md` such as logging cleanup, dead-symbol removal, and documentation polish are deferred unless they are required to close a blocker in this phase.
- End-to-end fee-proof statement redesign outside the `z00z_core` slice is deferred unless the implementation work proves the blocker cannot be closed inside the core crate.

</deferred>

---

*Phase: 026-crypto-audit-core*
*Context gathered: 2026-03-28 via PRD Express Path*
