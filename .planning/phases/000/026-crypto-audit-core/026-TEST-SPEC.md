---
phase: 026-crypto-audit-core
artifact: test-spec
status: current
source: revised-plans-summaries-verification-and-executed-phase-tests
updated: 2026-03-28
---

# Phase 026 Test Spec

## Purpose

📌 This document defines the unit, Rust integration, and end-to-end acceptance
coverage required for Phase 026.

📌 It is intended to be directly usable by another engineer or agent without
guessing scenario boundaries, proof anchors, rejection criteria, or which
existing test seam should own a given assertion.

📌 Phase 026 is a `z00z_core` crypto-hardening phase. Its end-to-end proof is
not browser automation. The required E2E signal is Rust integration coverage
that exercises canonical config, registry, genesis, wire, signature, fee,
nonce, and amount boundaries under realistic inputs and adversarial mutations.

## Workflow Status

📌 Phase 026 now has complete plan summaries and a phase-local verification
artifact.

📌 This document is the current test contract for Phase 026 and replaces the
earlier fallback-only wording.

📌 The current source of truth for this spec uses these inputs:

- `.planning/phases/026-crypto-audit-core/026-CONTEXT.md`
- `.planning/phases/026-crypto-audit-core/026-FUSION.md`
- `.planning/phases/026-crypto-audit-core/026-01-PLAN.md`
- `.planning/phases/026-crypto-audit-core/026-02-PLAN.md`
- `.planning/phases/026-crypto-audit-core/026-03-PLAN.md`
- `.planning/phases/026-crypto-audit-core/026-04-PLAN.md`
- `.planning/phases/026-crypto-audit-core/026-05-PLAN.md`
- `.planning/phases/026-crypto-audit-core/026-01-SUMMARY.md`
- `.planning/phases/026-crypto-audit-core/026-02-SUMMARY.md`
- `.planning/phases/026-crypto-audit-core/026-03-SUMMARY.md`
- `.planning/phases/026-crypto-audit-core/026-04-SUMMARY.md`
- `.planning/phases/026-crypto-audit-core/026-05-SUMMARY.md`
- `.planning/phases/026-crypto-audit-core/026-VERIFICATION.md`
- Existing test anchors in `crates/z00z_core/tests/` and current inline module
  tests under `crates/z00z_core/src/**`.

## Classification

### TDD And Integration Targets

- `crates/z00z_core/src/assets/definition.rs`
  because Phase 026 must prove one canonical definition-identity rule and must
  reject or overwrite caller-controlled identifier drift.
- `crates/z00z_core/src/assets/assets_config.rs`
  because config-driven definitions must converge on the same canonical identity
  seam used elsewhere.
- `crates/z00z_core/src/assets/snapshot.rs`
  because registry integrity must move from identifier-only hashing to full
  canonical definition-payload hashing.
- `crates/z00z_core/src/assets/registry.rs`
  because snapshot emit and snapshot apply must share the same tamper-detecting
  hash contract.
- `crates/z00z_core/src/assets/wire.rs`
  because `DefinitionWire` and `AssetWire` are authoritative domain conversion
  seams for identity, confidentiality, and state transport.
- `crates/z00z_core/src/assets/wire_pkg.rs`
  because public DTO decode must reject forbidden confidential material and must
  not silently preserve ambiguous public shapes.
- `crates/z00z_core/src/assets/test_wire.rs`
  because it is already the fastest direct regression seam for wire and DTO
  roundtrips, decode behavior, and flag transport.
- `crates/z00z_core/src/assets/assets.rs`
  because owner-signature and stealth-critical verification semantics live here.
- `crates/z00z_core/src/assets/gas.rs`
  because core-side fee validation must reject `Coin`-class assets whose
  identity is not the canonical native fee asset.
- `crates/z00z_core/src/assets/nonce.rs`
  because time-provider failures and nonce helper exports must become fail
  closed on the production path.
- `crates/z00z_core/src/assets/amount.rs`
  because `MAX_AMOUNT` must be tied to proof-width semantics instead of a bare
  literal.
- `crates/z00z_core/src/assets/mod.rs`
  because exported production-facing nonce and fee helpers must point to the
  fail-closed surfaces.
- `crates/z00z_core/src/genesis/validator.rs`
  because protected-network anchor and seed validation must fail closed.
- `crates/z00z_core/src/genesis/genesis.rs`
  because genesis asset-definition creation must reuse the canonical definition
  identity seam and must not preserve a parallel derivation path.
- `crates/z00z_core/src/genesis/genesis_config.rs`
  because protected-network parsing and config contract wording must stay
  explicit and typed.

### E2E Browser Targets

- None.

📌 Phase 026 end-to-end behavior must be proven through Rust integration and
realistic config or snapshot or DTO roundtrips, not through browser automation.

### Skip Targets

- The planning markdown files themselves
  because they are specification inputs, not executable logic.
- Vendor code under `crates/z00z_crypto/tari/`
  because the phase scope explicitly excludes vendor modifications.
- Unrelated simulator and wallet UI flows
  unless they are later needed only as downstream consumers of the corrected
  `z00z_core` contracts.

## Existing Test Structure

📌 The repository already uses two strong integration entrypoints for this
phase:

- `crates/z00z_core/tests/assets/test_assets.rs`
- `crates/z00z_core/tests/genesis/test_genesis.rs`

📌 The project convention is focused Rust test modules under `tests/assets/` and
`tests/genesis/`, plus lightweight inline regression tests inside the source
module when the seam is tightly local.

📌 Existing high-value anchors that should be reused before creating duplicates
already exist in these files:

- `crates/z00z_core/tests/assets/test_registry_integration.rs`
- `crates/z00z_core/tests/assets/test_wire_format_snapshots.rs`
- `crates/z00z_core/tests/assets/asset_signature_domain.rs`
- `crates/z00z_core/tests/assets/test_integration_owner_signature_security.rs`
- `crates/z00z_core/tests/assets/test_amount.rs`
- `crates/z00z_core/src/assets/test_wire.rs`
- `crates/z00z_core/src/assets/nonce.rs` inline tests
- `crates/z00z_core/tests/genesis/test_config.rs`
- `crates/z00z_core/tests/genesis/test_integration.rs`
- `crates/z00z_core/tests/genesis/test_genesis_state_verification.rs`

📌 Proposed new files are acceptable only when extending the existing anchor
would make ownership ambiguous or mix too many unrelated assertions into a
legacy catch-all test file.

## Canonical Test Commands

📌 Every implementation wave should keep the same top-level validation order:

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `cargo test -p z00z_core --test assets_tests -- --nocapture`
- `cargo test -p z00z_core --test genesis_tests -- --nocapture`

📌 Targeted existing anchors that should remain runnable during Phase 026 work:

- `cargo test -p z00z_core --test assets_tests test_registry_yaml_config_loading -- --exact --nocapture`
- `cargo test -p z00z_core --test assets_tests test_definition_wire_serialization_snapshot -- --exact --nocapture`
- `cargo test -p z00z_core --test assets_tests test_sig_tamper_amount -- --exact --nocapture`
- `cargo test -p z00z_core --test assets_tests test_rejects_proof_range_tampering -- --exact --nocapture`
- `cargo test -p z00z_core --test assets_tests test_amount_encoding -- --exact --nocapture`
- `cargo test -p z00z_core --test genesis_tests test_full_genesis_generation_flow -- --exact --nocapture`
- `cargo test -p z00z_core --test genesis_tests test_genesis_three_networks_produce -- --exact --nocapture`

⚠️ The broader workspace release command from the phase plans remains useful as
a later gate, but [STATE.md](../../STATE.md) still records an external vendor
doctest blocker under `crates/z00z_crypto/tari/`. Phase-local acceptance for
this spec should therefore rely on the targeted `z00z_core` test entrypoints
above plus the bootstrap gate.

## Execution Status

📌 The canonical Phase 026 validation commands were executed on `2026-03-28`.

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
  completed successfully.
- `cargo test -p z00z_core --test assets_tests -- --nocapture`
  completed successfully with
  `263 passed; 0 failed; 7 ignored; 0 measured; 0 filtered out`.
- `cargo test -p z00z_core --test genesis_tests -- --nocapture`
  completed successfully, including the long-running batch-verification anchors
  that emitted the standard `running for over 60 seconds` progress warnings
  before finishing green.

📌 The named anchors listed in this spec were also rechecked against the current
`z00z_core` tree before this artifact was refreshed.

📌 Review refresh on `2026-03-28` revalidated the wire boundary after
`AssetWire::to_asset()` was tightened to reject secret-bearing imports and to
require full `verify_complete()` rehydrate:

- `cargo test -p z00z_core --lib wire_ -- --nocapture`
- `cargo test -p z00z_core --lib verify_complete_on_roundtrip -- --nocapture`
- `cargo test -p z00z_core --test assets_tests -- --nocapture`

📌 Review checklist state on `2026-03-29`:

- [x] `PH26-WIRE` remains aligned with the spec after the boundary hardening review.
- [ ] `PH26-GENESIS` is only partial fail-closed until concrete mainnet and testnet anchor hashes are populated.

## Plan-To-Proof Coverage Map

| Plan | Must Be Proven | Primary Seams | Primary Test Ownership |
| ---- | ---- | ---- | ---- |
| `026-01` | Canonical definition identity is derived once and reused by config, genesis, and trusted wire paths | `definition.rs`, `assets_config.rs`, `genesis.rs`, `wire.rs` | assets integration + focused identity tests |
| `026-02` | Registry snapshot hash changes on any canonical payload drift and rejects tampered payloads with stable ids | `snapshot.rs`, `registry.rs`, `wire.rs` | registry integration + snapshot hash tests |
| `026-03` | Protected-network genesis anchors and seed policy fail closed | `validator.rs`, `genesis.rs`, `genesis_config.rs` | genesis integration + config validation tests |
| `026-04` | Untrusted DTO paths reject forbidden confidential material and never silently clear protocol-state flags | `wire.rs`, `wire_pkg.rs`, `test_wire.rs` | inline wire tests + assets integration tests |
| `026-05` | Owner or stealth verification binds authoritative state, native fee identity is enforced in core, and nonce or amount policy fails closed | `assets.rs`, `gas.rs`, `nonce.rs`, `amount.rs`, `mod.rs` | owner-signature tests + fee or nonce or amount tests |

## Required End-To-End Behaviors

📌 The following behaviors are mandatory and must be proven explicitly rather
than inferred from comments or local unit assertions.

| Behavior | Requirement | Primary Path | Pass Signal | Fail Signal |
| ---- | ---- | ---- | ---- | ---- |
| Canonical definition identity converges across construction paths | `PH26-ASSET-ID` | config load or trusted constructor -> canonical helper -> registry insert or genesis construction | same canonical payload yields one stable `id`; changed payload yields different canonical `id`; mismatched supplied `id` is rejected or overwritten explicitly | caller-chosen `id` survives unchecked, or two materially different payloads share the same accepted canonical identity |
| Registry hash is content-addressed by canonical payload | `PH26-REGISTRY` | definition payload -> snapshot create -> snapshot apply verify | payload mutation with stable ids changes hash and is rejected on apply | snapshot with same ids but changed payload still validates |
| Protected-network genesis fails closed | `PH26-GENESIS` | chain parse -> seed validation -> create_asset_definition -> state hash -> verify_genesis_consensus | unknown protected-network string, missing anchor, mismatched anchor, and weak protected-network seed all fail with explicit error | protected-network flow downgrades to devnet semantics or succeeds without anchor |
| Untrusted wire import rejects confidential or ambiguous state | `PH26-WIRE` | JSON or DTO decode -> `AssetPkgWire` -> `AssetWire` -> domain rehydrate | `secret` is rejected; plaintext-confidential policy is explicit; state flags are preserved or rejected explicitly; canonical definition validation occurs before authoritative use | secret-bearing input passes, confidential plaintext acceptance remains ambiguous, or flags are silently cleared |
| Owner and stealth-critical tampering breaks verification | `PH26-AUTH` | `to_owner_message()` or equivalent authority seam -> sign -> verify -> stealth consistency gate | tampering `r_pub`, `owner_tag`, `enc_pack`, `tag16`, or `leaf_ad_id` causes verification failure or explicit rejection | signature remains valid after stealth-critical mutation, or verifier overclaims commitment-opening proof |
| Core fee validation rejects non-native `Coin` assets | `PH26-AUTH` | `GasAsset::new_from_definition` or equivalent helper | canonical native fee asset passes, non-native `Coin` with same class fails | class-only check still accepts wrong native asset identity |
| Production nonce helpers fail closed | `PH26-NONCE-FEE` | production-facing export -> time provider -> `try_*` nonce helper | pre-epoch or time-provider failure returns typed error and does not substitute timestamp `0` | production helper silently returns a derived nonce from fallback timestamp `0` |
| Amount policy is proof-width aware | `PH26-NONCE-FEE` | amount boundary helper -> commitment or proof creation or verification | limit is derived from proof-width semantics and boundary tests prove the exact accepted and rejected edge | `MAX_AMOUNT` remains an unqualified literal with no proof-width-coupled tests |

## Critical Integration Paths

📌 Another engineer should treat these as the canonical integration paths for
Phase 026. If a new test does not anchor to one of these paths, it is probably
secondary regression coverage rather than phase-closing proof.

1. Definition identity path:
   `assets_config.rs -> canonical helper in definition.rs -> registry insert -> trusted wire or genesis reuse`
2. Registry synchronization path:
   `DefinitionWire payload -> RegistryVersion::compute_hash(...) -> create_snapshot(...) -> update_from_snapshot(...)`
3. Protected genesis path:
   `ChainType::from_str(...) -> validate_genesis_seed(...) -> create_asset_definition(...) -> compute_genesis_state_hash(...) -> verify_genesis_consensus(...)`
4. Public DTO import path:
  `decode_asset_pkg_json(...) -> parse_pkg_fields(...) -> AssetPkgWire::to_wire() -> AssetWire::to_asset() -> verify_complete()`
5. Ownership and stealth path:
   `Asset::new(...) -> to_owner_message() -> sign_owner(...) -> verify_owner_signature() -> validate_stealth_consistency()`
6. Fee and production policy path:
   `GasAsset::new_from_definition(...) -> calculate_fee(...)` and
   `mod.rs production exports -> try_get_timestamp_micros(...) -> try_derive_nonce_simple(...)`

## Scenario Oracle Rules

📌 Each scenario in this spec must have a machine-checkable pass or fail oracle.
The following rules are mandatory.

1. A scenario passes only when it proves both behavior and invariant.
2. A rejection scenario passes only when the rejection is explicit and no
  partial-success artifact remains accepted in memory or in the returned type.
3. A canonical identity scenario passes only when unchanged canonical payloads
  converge and any protected field mutation diverges or is rejected.
4. A protected-network genesis scenario passes only when unknown chain strings,
  missing anchors, mismatched anchors, and prohibited seeds fail on the
  protected path instead of degrading into optional or devnet behavior.
5. A wire-boundary scenario passes only when secret-bearing or ambiguous input
  is rejected before authoritative rehydration, or when the policy explicitly
  preserves the field without silent loss.
6. An ownership or stealth scenario passes only when mutating a signed or
  verifier-bound field breaks verification or triggers explicit typed rejection.
7. A fee scenario passes only when the same `AssetClass::Coin` is insufficient
  without the canonical native asset identity.
8. A nonce or amount scenario passes only when the production-facing path uses
  explicit error handling or proof-width-derived limits instead of implicit
  fallback or a bare literal.

## Required Examples And Fixtures

📌 The implementation phase should prepare these concrete fixtures before adding
or extending tests.

- Two `AssetDefinition` values with byte-identical canonical payload and
  different caller-supplied `id` values.
- Two `DefinitionWire` or snapshot payloads that keep the same `id` but mutate
  `name`, `symbol`, `policy_flags`, or ordered metadata.
- Protected-network genesis configs for `mainnet` and `testnet` with missing
  expected hash, mismatched hash, unknown chain string, all-zero seed,
  sequential seed, repeating seed, and known test seed.
- Public JSON payloads that attempt to inject `secret`, partial stealth tuples,
  or a confidentiality-breaking plaintext representation on untrusted import.
- Asset values whose `is_frozen` or `is_slashed` state is set before DTO export,
  so silent rehydrate clearing can be detected.
- Signed assets whose `r_pub`, `owner_tag`, `enc_pack`, `tag16`, and
  `leaf_ad_id` are mutated one field at a time after signing.
- Two `Coin` definitions where one is the canonical native fee asset and the
  other is only class-compatible.
- Mock time providers that return pre-epoch or other explicit failure paths for
  production nonce helpers.
- Amount boundary vectors at the exact proof-width ceiling, one below it when
  relevant, and one beyond the supported width.

## Test Files To Add Or Extend

### 1. Extend `crates/z00z_core/tests/assets/test_registry_integration.rs`

📌 This file should remain the main registry lifecycle anchor for Phase 026.

Tests to add or strengthen:

1. `config_definition_id_is_canonicalized`
   Demonstrates: config-driven definitions cannot preserve caller-controlled
   identifier drift.
   Success conditions:
   - the loaded definition id matches the canonical payload rule;
   - a mismatched caller id is rejected or deterministically rewritten;
   - registry insert uses the canonicalized identity, not the untrusted input.

2. `snapshot_payload_tamper_rejected_even_if_id_stable`
   Demonstrates: snapshot integrity is keyed to full payload, not just id.
   Success conditions:
   - snapshot create succeeds on canonical payload;
   - tampering `name`, `symbol`, `policy_flags`, or metadata while keeping `id`
     constant changes the recomputed hash;
   - snapshot apply rejects the tampered payload.

3. `snapshot_create_apply_hash_roundtrip_matches`
   Demonstrates: emit and apply share one canonical hash contract.
   Success conditions:
   - create and apply compute the same hash for the same ordered payload;
   - deterministic `BTreeMap` ordering is preserved;
   - no alternate framing path exists in the apply code.

### 2. Extend `crates/z00z_core/src/assets/test_wire.rs`

📌 This file is the fastest direct wire and DTO boundary regression seam.

Tests to add or strengthen:

1. `definition_wire_rehydrate_uses_validated_identity`
   Demonstrates: authoritative domain construction no longer trusts blind
   `DefinitionWire` conversion.
   Success conditions:
   - rehydration returns `Result`, not blind struct casting;
   - mismatched identity payload is rejected before authoritative use;
   - canonical payload rehydrate succeeds.

2. `asset_pkg_rejects_secret_field_on_untrusted_decode`
   Demonstrates: public JSON import cannot inject trusted-only secret material.
   Success conditions:
   - payload with `secret` or equivalent forbidden field is rejected;
   - decode error is explicit;
   - no asset instance is produced.

3. `asset_pkg_flag_policy_is_explicit`
   Demonstrates: `is_frozen` and `is_slashed` are either preserved or rejected,
   never silently cleared.
   Success conditions:
   - flagged input roundtrip preserves flags on the supported trusted path, or
     untrusted decode rejects the shape;
   - `AssetPkgWire::to_wire()` or its replacement does not zero the flags
     silently.

4. `asset_pkg_plaintext_amount_policy_is_explicit`
   Demonstrates: confidential plaintext handling is not ambiguous.
   Success conditions:
   - the chosen policy is enforced in code;
   - confidential import either rejects plaintext amount or labels the path as
     trusted or non-confidential by design;
   - the test asserts the exact chosen policy instead of only documenting it.

### 3. Extend `crates/z00z_core/tests/assets/test_wire_format_snapshots.rs`

📌 This file should guard canonical wire-shape stability after the phase lands.

Tests to add or strengthen:

1. `definition_wire_payload_snapshot_tracks_canonical_fields`
   Demonstrates: every canonical identity field is present in the wire contract
   used by snapshot hashing.
   Success conditions:
   - `id`, `class`, `name`, `symbol`, `decimals`, `serials`, `nominal`,
     `domain_name`, `version`, `crypto_version`, `policy_flags`, and metadata
     stay represented;
   - intentional contract changes require snapshot update and are visible.

2. `wire_roundtrip_does_not_mask_identity_drift`
   Demonstrates: wire roundtrip cannot normalize a forged payload into an
   accepted authoritative identity silently.
   Success conditions:
   - canonical payload roundtrip succeeds;
   - forged payload fails before authoritative acceptance.

### 4. Extend genesis config and integration anchors

`crates/z00z_core/tests/genesis/test_config.rs`,
`test_integration.rs`, and `test_genesis_state_verification.rs`

📌 These files already own the strongest genesis config and integration seams.

Tests to add or strengthen:

1. `protected_network_missing_anchor_fails_closed`
   Demonstrates: `mainnet` and `testnet` cannot succeed without configured
   expected hash.
   Success conditions:
   - missing protected-network anchor returns typed error;
   - no optional success path remains.

2. `protected_network_unknown_chain_type_fails`
   Demonstrates: unknown protected-network strings do not degrade into devnet.
   Success conditions:
   - `ChainType::from_str(...)` path fails explicitly;
   - the legacy `From<&str>` fallback cannot remain an accepted protected path.

3. `weak_protected_network_seed_rejected`
   Demonstrates: explicit weak-seed checks replace Shannon-threshold approval.
   Success conditions:
   - all-zero, sequential, repeating, and known test seeds fail on protected
     networks;
   - devnet-only relaxation, if any, is explicit and separately asserted.

4. `genesis_definition_identity_matches_plan01_canonical_rule`
   Demonstrates: genesis-created definitions reuse the same canonical identity
   seam as config and registry work.
   Success conditions:
   - genesis-generated definition ids match the canonical payload rule;
   - no parallel genesis-only derivation remains accepted.

### 5. Extend owner-signature security anchors

`crates/z00z_core/tests/assets/asset_signature_domain.rs` and
`test_integration_owner_signature_security.rs`

📌 These files already own signature-tamper coverage and should absorb Phase 026
authority hardening instead of spawning a parallel signature suite by default.

Tests to add or strengthen:

1. `sig_tamper_r_pub`
2. `sig_tamper_owner_tag`
3. `sig_tamper_enc_pack`
4. `sig_tamper_tag16`
5. `sig_tamper_leaf_ad_id`

Each test demonstrates: stealth-critical fields are verifier-bound canonical
state, not unsigned adjunct data.

Shared success conditions:

- original signed asset verifies successfully;
- mutating the single target field makes verification fail or yields explicit
  typed rejection;
- the test never claims that signature validity alone proves knowledge of the
  commitment opening.

Additional integration test to add or strengthen:

1. `verify_owner_signature_requires_authority_not_bare_signature`
   Demonstrates: a valid bare signature over incomplete semantics is not treated
   as sufficient authority if the final rule requires anchored owner state.
   Success conditions:
   - the verifier checks the declared authority rule explicitly;
   - the test distinguishes signature validity from authority semantics.

### 6. Extend amount and nonce anchors

`crates/z00z_core/tests/assets/test_amount.rs` and
`crates/z00z_core/src/assets/nonce.rs` inline tests

📌 These are the clearest existing anchors for amount bounds and nonce helper
API behavior.

Tests to add or strengthen:

1. `max_amount_is_derived_from_proof_width`
   Demonstrates: `MAX_AMOUNT` is tied to proof-width semantics, not a bare
   literal.
   Success conditions:
   - the assertion references the proof-width rule;
   - the accepted edge matches the derived boundary;
   - the test does not merely compare against `u64::MAX` by name.

2. `amount_above_supported_width_rejected`
   Demonstrates: values beyond the supported proof-width contract do not pass as
   valid production amounts.
   Success conditions:
   - the exact out-of-range case is rejected or fails proof creation or
     verification predictably;
   - zero-value exceptions for NFT or Void remain explicit and separate.

3. `production_nonce_helper_before_epoch_fails_closed`
   Demonstrates: exported production helpers no longer substitute timestamp `0`.
   Success conditions:
   - pre-epoch mock time returns typed error on the production-facing helper;
   - any legacy lightweight helper remains clearly scoped as non-production if
     retained.

4. `nonce_counter_contract_is_explicit`
   Demonstrates: persistence or uniqueness requirements are observable and not
   left as comments only.
   Success conditions:
   - the test names the production contract explicitly;
   - rollback or reuse behavior is asserted where the implementation exposes it.

### 7. Proposed New Files If Existing Anchors Become Too Diffuse

📌 These files are proposed targets, not confirmed existing files. Use them only
if extending the current anchors would make ownership unclear.

- `crates/z00z_core/tests/assets/test_definition_identity.rs`
  for a dedicated identity-convergence suite across config, trusted wire, and
  registry seams.
- `crates/z00z_core/tests/assets/test_registry_snapshot_integrity.rs`
  for focused snapshot-tamper rejection coverage.
- `crates/z00z_core/tests/assets/test_wire_boundary_rejection.rs`
  for secret-field rejection, plaintext amount policy, and flag-transport rules.
- `crates/z00z_core/tests/genesis/test_fail_closed_consensus.rs`
  for protected-network missing-anchor, mismatched-anchor, and weak-seed cases.
- `crates/z00z_core/tests/assets/test_core_policy_fail_closed.rs`
  for fee-identity, nonce, and amount fail-closed semantics when grouping them
  into one focused policy file is clearer than scattering them further.

## Minimal Scenario Set Required To Close The Phase

📌 The final implementation must prove at least these ten scenarios end to end.

1. Same canonical definition payload from two construction paths yields the same
  accepted identity.
2. Same `id` with changed canonical payload is rejected or rederived before
  authoritative acceptance.
3. Registry snapshot with changed payload and stable ids is rejected on apply.
4. `mainnet` or `testnet` with missing expected hash fails closed.
5. Unknown protected-network chain string does not degrade into devnet.
6. Untrusted DTO payload with `secret` is rejected.
7. DTO rehydration cannot silently clear `is_frozen` or `is_slashed`.
8. Mutating each stealth-critical field breaks authoritative verification.
9. `Coin`-class asset with wrong native identity is rejected as fee asset.
10. Production nonce or amount boundary violation fails closed with explicit
   error or rejection.

## Measurable Completion Conditions

📌 Phase 026 test coverage is sufficient only when all of the following are
true.

1. Each of the five phase plans has at least one positive and one negative
  scenario anchored to executable tests.
2. Every rejection scenario asserts the exact rejection effect and confirms that
  no silent fallback path produced an accepted object.
3. Every cryptographic mutation test changes one field at a time so the broken
  invariant is unambiguous.
4. At least one integration test proves each of these full paths:
  definition identity, registry snapshot, protected genesis, public DTO import,
  owner or stealth authority, and nonce or amount fail-closed behavior.
5. The targeted `z00z_core` test entrypoints stay runnable independently of the
  known workspace-wide vendor doctest blocker.

## Outbound Notes For The Future `gsd-add-tests` Pass

📌 When Phase 026 eventually has summary and verification artifacts, the
completed-phase `gsd-add-tests` pass should treat this file as the default phase
test contract unless the later implementation artifacts prove a narrower and
better-aligned ownership split.

📌 If implementation spreads across too many legacy `test_integration_assets_*`
files, prefer creating one focused proposed file per new invariant rather than
burying the phase logic in unrelated historical buckets.
