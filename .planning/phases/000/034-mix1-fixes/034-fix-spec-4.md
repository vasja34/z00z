# Fix Spec 4: Canonical Spend Nullifier Closure And Legacy Sender Path Retirement

**Status:** Draft for implementation  
**Date:** 2026-04-09  
**Scope:** Regular transaction spend semantics and removal of legacy sender-construction authority that conflicts with the current canonical stealth surface.

## 🎯 Objective

Define the next implementation-ready cleanup and closure phase for the current transaction stack.

This spec has four concrete goals:

1. close the remaining regular-spend alignment work around the shipped nullifier contract: the authenticated wire now carries one signed nullifier field, while deterministic derivation must stay aligned across witness and structural enforcement;
2. keep the canonical architecture vocabulary aligned with the code that actually ships today;
3. retire or isolate legacy sender-construction paths that still leak old authority through `core::tx`;
4. prevent planning and implementation drift from reintroducing stale temp-doc terminology.

This document is repository-backed. If any older markdown conflicts with live code, live code wins.

## 🔍 Verified Claim Resolution

The following claims are verified against the current repository and are the authority baseline for this spec.

### ✅ Verified claim 1: the regular public spend contract is already real

The spend path is not hypothetical. The current stack already builds and verifies a real public spend contract.

Live evidence:

```rust
pub fn verify_spend_witness_gate(
    chain_id: u32,
    recv_sec: [u8; 32],
    selected_inputs: &[AssetWire],
    outputs: &[OutputBundle],
    prev_root: CheckRoot,
) -> Result<(), String> {
    ...
    let (proof, auth) =
        build_public_spend_contract(&receiver_keys, chain_id, 1, &tx, prev_root, proof_inputs)
            .map_err(|e| format!("stage4: SpendWitness gate failed: {e}"))?;
    tx.proof = proof;
    tx.auth = auth;

    verify_tx_public_spend_contract(chain_id, 1, &tx)
        .map_err(|e| format!("stage4: SpendWitness gate failed: {e}"))
}
```

Source: `crates/z00z_wallets/src/core/tx/witness_gate.rs`

### ✅ Verified claim 2: the shipped boundary now carries a signed nullifier field, but deterministic derivation is still enforced outside the standalone public verifier

Live code now freezes a narrower and more honest statement.

```rust
/// Current-stack spend verification is real and fail closed, and the regular public spend contract now binds one deterministic nullifier semantics surface.
/// The current proof/auth seam is already live; the authenticated wire now carries the deterministic nullifier field and duplicate values reject before authorization.
/// Deterministic `chain_id || s_in` derivation is enforced in the witness bridge and structural rule layer, while the standalone public verifier authenticates the signed field and rejects malformed, duplicate, or post-signature drift.
pub fn verify_tx_public_spend_contract(
    chain_id: u32,
    tx_version: u8,
    tx: &TxWire,
) -> Result<(), SpendPublicErr> {
```

Source: `crates/z00z_wallets/src/core/tx/spend_verification.rs`

The structural spend-rule layer still carries the deterministic relation directly:

```rust
/// Current-stack spend verification is real and fail closed, and the regular public spend contract now binds one deterministic nullifier semantics surface.
/// This seam keeps the nullifier contract aligned across the witness bridge, the delivered persisted public spend contract, and the structural rule layer.
pub fn verify_spend_rules(stmt: &SpendStmt) -> Result<(), SpendRuleErr> {
```

Source: `crates/z00z_wallets/src/core/tx/spend_rules.rs`

### ✅ Verified claim 3: canonical transaction construction already exists under `core::stealth`

Current accepted-flow construction is already explicit and live.

```rust
pub fn build_tx_stealth_output_validated<'a>(
    receiver_card: &ReceiverCard,
    payment_request: Option<&PaymentRequest>,
    build_check: BuildCheck<'a>,
    sender_wallet: &mut SenderWallet,
    tx_digest: &[u8; 32],
    out_index: u32,
    amount: u64,
    asset_id: &[u8; 32],
) -> Result<TxStealthOutput, StealthError> {
    approve_req(payment_request, build_check)?;
    ...
    validate_output_self(&output, &ctx, amount)?;
    Ok(output)
}
```

Source: `crates/z00z_wallets/src/core/stealth/output.rs`

Current request-aware derivation and approval checks are also already live:

```rust
fn approve_req<'a>(
    payment_request: Option<&PaymentRequest>,
    build_check: BuildCheck<'a>,
) -> Result<(), StealthError> {
    if let Some(request) = payment_request {
        let mut shadow_pins = build_check.pins.clone();
        let outcome = request
            .validate_all(&mut shadow_pins, build_check.chain_id)
            .map_err(|_| StealthError::InvalidStealthInput)?;
        if !matches!(outcome, ValidationOutcome::Approved) {
            return Err(StealthError::InvalidStealthInput);
        }
    }

    Ok(())
}
```

Source: `crates/z00z_wallets/src/core/stealth/output_build.rs`

### ✅ Verified claim 4: legacy sender-construction authority still leaks through `core::tx`

The old sender helper is still live and explicitly marks itself as legacy.

```rust
/// Create one stealth output leaf for a receiver card.
///
/// `serial_id` must be in `[1, 50_000]` per the SerialAmount spec (§21).
/// This is a legacy full-leaf path and does not participate in the H-3
/// duplicate-R retry cache used by `build_tx_stealth_output()`.
pub fn sender_create_output_for(
    card: &ReceiverCard,
    amount: u64,
    serial_id: u32,
) -> Result<AssetLeaf, WalletError> {
```

Source: `crates/z00z_wallets/src/core/tx/builder.rs`

The `core::tx` facade still re-exports this legacy constructor publicly:

```rust
pub use builder::{
    build_output_leaf, build_output_with_blind, build_output_with_rng, sender_create_output_for,
};
```

Source: `crates/z00z_wallets/src/core/tx/mod.rs`

### ✅ Verified claim 5: `output_flow.rs` is not the canonical source of truth for output construction

`output_flow.rs` still bundles Stage-4 compatibility helpers and construction-style APIs.

```rust
//! Core tx output flow helpers extracted from simulator Stage-4.

pub fn create_output_bundle(
    receiver: String,
    role: TxOutRole,
    class: AssetClass,
    card: &ReceiverCard,
    value: u64,
    serial_id: u32,
) -> Result<OutputBundle, String> {
```

Source: `crates/z00z_wallets/src/core/tx/output_flow.rs`

This file also contains helpers that are not inherently legacy, such as `bind_output_wire(...)` and `compute_tx_digest_from_wire(...)`. Therefore the correct action is not blind file deletion. The correct action is authority split, migration, and then selective retirement.

### ✅ Verified claim 6: canonical terms already differ from temp-doc language

Current canonical terms should be fixed to live code, not to old future-facing wording.

Canonical current mapping:

1. `Transaction Construction` means `build_tx_stealth_output(...)` and `build_tx_stealth_output_validated(...)`.
2. `Spend Proof` means `verify_spend_witness_gate(...)`, `build_public_spend_contract(...)`, and `verify_tx_public_spend_contract(...)`.
3. `sender cannot spend without receiver_secret` is currently a witness-gated accepted-flow property, not yet a fully closed theorem-level repository-wide proof statement.
4. `TxProof vs CheckpointProof separation` is still valid as an architecture intent and is already reflected in wire comments.
5. `Legacy full-leaf tx builder` is not a canonical concept. It is a compatibility surface.

## 🚨 Anti-Drift Rules

The following rules are mandatory while implementing this spec.

1. Do not describe `core::tx::sender_create_output_for(...)` as canonical sender construction.
2. Do not describe `output_flow.rs` as the authority layer for new transaction construction.
3. Do not describe the standalone public spend verifier as independently recomputing deterministic `chain_id || s_in` nullifiers.
4. Do not import claim-path terminology into regular tx semantics unless the code explicitly shares the same contract.
5. Do not reuse the claim nullifier helper as-is for regular tx just because it already exists.
6. Do not promise a fully shipped theorem-level TxProof closure if the implementation only binds the persisted public contract and verifier seam.
7. Do not delete the whole `output_flow.rs` file blindly; retire only the legacy construction authority and preserve helpers that remain legitimate support utilities.
8. Do not invent a new future architecture vocabulary if a live exported function already names the current concept.

## 📌 Canonical Vocabulary For This Phase

Use the following terms in code, specs, comments, and tests.

| Old or drifting wording | Canonical wording for this phase |
| --- | --- |
| future transaction builder | `build_tx_stealth_output(...)` / `build_tx_stealth_output_validated(...)` |
| abstract spend proof | `verify_spend_witness_gate(...)` + `build_public_spend_contract(...)` + `verify_tx_public_spend_contract(...)` |
| sender-construction API in `core::tx` | legacy compatibility surface |
| full theorem-level sender-cannot-spend statement | accepted-flow witness-gated spend boundary |
| regular spend contract complete | regular public verifier authenticates the signed nullifier field; witness and structural layers enforce deterministic derivation |

## 🧱 Verified Baseline Snippets

### Current `TxStealthOutput` is a lightweight canonical header, not a full `AssetLeaf`

```rust
pub struct TxStealthOutput {
    pub r_pub: [u8; 32],
    pub owner_tag: [u8; 32],
    pub tag16: Option<u16>,
    pub enc_pack: ZkPackEncrypted,
    pub c_amount: [u8; 32],
}
```

Source: `crates/z00z_wallets/src/core/stealth/output.rs`

### Current legacy leaf builder is low-level useful code mixed with one legacy wrapper

```rust
pub fn build_output_leaf(
    k_dh: &[u8; 32],
    r_pub: &[u8; 32],
    owner_handle: &[u8; 32],
    value: u64,
    serial_id: u32,
    s_out: [u8; 32],
) -> Result<AssetLeaf, WalletError> {
    let mut rng = SystemRngProvider.rng();
    let blinding = Hidden::hide(Z00ZScalar::random(&mut rng));
    build_output_with_blind(
        k_dh,
        r_pub,
        owner_handle,
        value,
        serial_id,
        s_out,
        &blinding,
    )
}
```

Source: `crates/z00z_wallets/src/core/tx/builder.rs`

This helper is not itself legacy. Its owner and export path are the problem.

### Current regular public spend wire now carries an explicit nullifier field

```rust
pub struct SpendInputProofWire {
    pub serial_id: u32,
    pub nullifier_hex: String,
    pub r_pub_hex: String,
    pub owner_tag_hex: String,
    pub commitment_hex: String,
    pub leaf_ad_id_hex: String,
    pub leaf_ad_hash_hex: String,
}
```

Source: `crates/z00z_wallets/src/core/tx/tx_wire_types.rs`

### Current structural spend statement already has a natural place for nullifier semantics to land

```rust
pub struct SpendIn {
    pub r_pub_in: [u8; 32],
    pub owner_tag_in: [u8; 32],
    pub leaf_ad_id_in: [u8; 32],
    pub s_in: [u8; 32],
    pub c_in: Z00ZCommitment,
}
```

Source: `crates/z00z_wallets/src/core/tx/spend_rules.rs`

Because `s_in` is already part of the structural witness, a minimal regular-tx nullifier plan can be anchored there without inventing new hidden state.

## 🏗️ Workstream A: Add Regular Spend Nullifier Semantics

### Why legacy retirement exists

This is the only openly documented remaining semantic gap in the regular spend contract.

The goal is not to invent a new proof system. The goal is to bind one deterministic regular-spend nullifier into the current public spend contract, wire representation, and verifier seam.

### What this workstream must achieve

Final required properties:

1. every spent input in the public spend contract carries one deterministic nullifier field;
2. the nullifier is domain-separated and chain-bound;
3. the shipped path rejects malformed and duplicated nullifiers in the standalone public verifier, and rejects deterministic-contract drift in the witness bridge plus structural rule layer;
4. the structural spend-rule layer and the public spend-contract layer use the same nullifier semantics;
5. the code comments that currently describe the gap can be narrowed or removed honestly after implementation.

### What this workstream must not overclaim

1. It does not automatically create a new global nullifier-state subsystem for regular tx unless a later phase explicitly requires one.
2. It does not automatically convert the whole regular spend path into a fully authoritative theorem-level ZK proof.
3. It must not cargo-cult the claim-path nullifier helper into regular tx without checking domain and witness differences.

### Why claim nullifier code is a reference, not the implementation target

The repository already has a claim-specific nullifier helper:

```rust
pub fn derive_nullifier(claim_id: &[u8; 32], owner: &[u8; 32], chain_id: u32) -> NullifierBytes {
    let mut h = blake3::Hasher::new();
    h.update(b"z00z.nullifier.derive.v1.");
    h.update(&chain_id.to_le_bytes());
    h.update(claim_id);
    h.update(owner);
    NullifierBytes(*h.finalize().as_bytes())
}
```

Source: `crates/z00z_wallets/src/core/claim/nullifier.rs`

This is useful as a pattern for deterministic chain-bound nullifier derivation, but it is not the regular-tx contract. Regular tx currently uses consensus-domain helpers such as `AssetIdDomain`, `ReceiverIdDomain`, `ViewKeyDomain`, and `TxDigestDomain` in `z00z_crypto::domains` plus `hash_zk(...)`. Keep that style for the regular tx nullifier.

### Recommended regular-tx nullifier contract

For this phase, the minimal repository-backed nullifier contract should be:

1. witness input: `s_in`;
2. public field: `nullifier_hex`;
3. derivation input: `chain_id || s_in`;
4. derivation algorithm: `hash_zk::<SpendNullifierDomain>("", &[&chain_id_le, &s_in])`;
5. encoding: lowercase 32-byte hex string.

This choice is grounded by the current spend-rule witness shape:

```rust
const ASSET_WIT: &[&str] = &["s_in"];
const ASSET_PUB: &[&str] = &["leaf_ad_id_in"];
...
let expected_leaf_ad_id = hash_zk::<AssetIdDomain>("", &[&input.s_in]);
```

Source: `crates/z00z_wallets/src/core/tx/spend_rules.rs`

This phase should add a second public derivation from the same witness material rather than inventing a new witness dependency.

### Files to change for Workstream A

#### 1. `crates/z00z_crypto/src/domains.rs`

Add one dedicated domain declaration for the regular spend nullifier.

Recommended style:

```rust
hash_domain!(SpendNullifierDomain, "Z00Z/NULLIFIER", 1);
```

Place it in the same consensus-domain group as `AssetIdDomain`, `TxDigestDomain`, and related tx/proof domains.

#### 2. `crates/z00z_wallets/src/core/tx/tx_wire_types.rs`

Extend `SpendInputProofWire` with one new field.

Current:

```rust
pub struct SpendInputProofWire {
    pub serial_id: u32,
    pub r_pub_hex: String,
    pub owner_tag_hex: String,
    pub commitment_hex: String,
    pub leaf_ad_id_hex: String,
    pub leaf_ad_hash_hex: String,
}
```

Target:

```rust
pub struct SpendInputProofWire {
    pub serial_id: u32,
    pub r_pub_hex: String,
    pub owner_tag_hex: String,
    pub commitment_hex: String,
    pub leaf_ad_id_hex: String,
    pub leaf_ad_hash_hex: String,
    pub nullifier_hex: String,
}
```

Document it as a deterministic regular-spend nullifier and explicitly keep the existing comment that `serial_id` is not nullifier material.

#### 3. `crates/z00z_wallets/src/core/tx/spend_verification.rs`

Required changes:

1. add a small helper such as `derive_spend_nullifier(s_in: &[u8; 32], chain_id: u32) -> [u8; 32]`;
2. fill `nullifier_hex` inside the input proof construction path;
3. verify that each public nullifier decodes correctly;
4. reject duplicate nullifiers within the same spend proof;
5. verify that each `nullifier_hex` is well-formed, unique inside the proof, and still bound to the signed statement; deterministic `chain_id || s_in` derivation stays enforced in the witness bridge and structural rule layer unless a later phase exposes more public witness material.

This should be implemented adjacent to the existing input-proof construction and verification code, not as an unrelated new module with disconnected semantics.

#### 4. `crates/z00z_wallets/src/core/tx/witness_gate.rs`

`prepare_spend_public_inputs(...)` is the current bridge from selected inputs to public spend-proof material.

That means it is the correct place to ensure the bridge emits proof inputs that are sufficient for nullifier binding.

Do not add new speculative state dependencies here. Keep it limited to deterministic witness-to-public-input projection.

#### 5. `crates/z00z_wallets/src/core/tx/spend_rules.rs`

Update the structural spend-rule vocabulary so that the rule inventory matches the public spend contract.

Recommended minimum change:

1. add `SpendRule::Nullifier`;
2. add `nullifier_in: [u8; 32]` to `SpendIn`;
3. verify `nullifier_in == H_zk(chain_id || s_in)`;
4. keep the old `AssetId` rule separate from the new `Nullifier` rule.

This matters because `leaf_ad_id_in` and `nullifier_in` serve different semantics and must not collapse into one field.

#### 6. Tests that must be updated

At minimum:

1. `crates/z00z_wallets/tests/test_spend_witness_gate.rs`
2. `crates/z00z_wallets/src/core/tx/spend_rules.rs` internal tests
3. `crates/z00z_wallets/src/core/tx/spend_verification.rs` internal tests
4. `crates/z00z_wallets/tests/test_scenario1_semantics.rs`
5. `crates/z00z_simulator/tests/test_scenario1_spend_gate.rs`

New test cases required:

1. accepts valid deterministic nullifier;
2. rejects malformed hex;
3. rejects signed nullifier drift on the public seam and deterministic mismatch on the structural seam;
4. rejects duplicate nullifier inside one tx;
5. keeps wording pinned to the shipped signed-field boundary and removes obsolete open-gap wording only after implementation is complete.

### Workstream A execution order

1. add domain;
2. add wire field;
3. add deterministic helper;
4. emit nullifier in input-proof construction;
5. verify nullifier in public contract;
6. mirror rule in `spend_rules.rs`;
7. update tests and closure wording.

## 🧹 Workstream B: Retire Legacy Sender Construction Authority

### Why this workstream exists

The repository currently exposes two different authority stories:

1. canonical accepted-flow construction under `core::stealth`;
2. legacy full-leaf and Stage-4 helper construction under `core::tx`.

As long as both remain public and co-equal, specs and future implementations can drift back into the wrong source of truth.

### Current live leak points

#### Legacy wrapper in `builder.rs`

```rust
pub fn sender_create_output_for(
    card: &ReceiverCard,
    amount: u64,
    serial_id: u32,
) -> Result<AssetLeaf, WalletError> {
```

#### Compatibility-style construction helpers in `output_flow.rs`

```rust
pub fn create_output_bundle(
    receiver: String,
    role: TxOutRole,
    class: AssetClass,
    card: &ReceiverCard,
    value: u64,
    serial_id: u32,
) -> Result<OutputBundle, String> {
```

#### Public exports from `core::tx`

```rust
pub use builder::{
    build_output_leaf, build_output_with_blind, build_output_with_rng, sender_create_output_for,
};

pub use output_flow::{
    bind_output_wire, compute_tx_digest_from_wire, create_output_bundle,
    create_output_bundle_with_rng, decode_output_pack, derive_balance_commitment,
    derive_fee_commitment, derive_tx_output_nonce, verify_commitment_balance_gate,
    verify_fee_commitment_opening, verify_fee_opening_eq, verify_plaintext_balance_with_fee,
    verify_self_decrypt, OutputBundle,
};
```

Source: `crates/z00z_wallets/src/core/tx/mod.rs`

### Why blind deletion is wrong

The old layer is mixed. Some code is legacy authority. Some code is just low-level utility.

Examples of legitimate low-level helpers that should likely survive, but under the right owner:

1. `build_output_leaf(...)`
2. `build_output_with_blind(...)`
3. `build_output_with_rng(...)`
4. `bind_output_wire(...)`
5. `compute_tx_digest_from_wire(...)`

Examples of authority that should be retired or isolated:

1. `sender_create_output_for(...)`
2. `create_output_bundle(...)`
3. `create_output_bundle_with_rng(...)` as a public transaction-construction entrypoint
4. public re-export of these helpers from `core::tx`

### Verified call sites that block immediate deletion

Current repository users of the legacy sender or output-flow surface include:

1. `crates/z00z_simulator/src/scenario_1/stage_6_utils/bridge_output_router.rs`
2. `crates/z00z_simulator/src/scenario_1/stage_4_utils/*`
3. `crates/z00z_wallets/tests/test_phase15_regress.rs`
4. `crates/z00z_wallets/tests/test_spend_witness_gate.rs`
5. `crates/z00z_wallets/tests/test_s5_sender_examples.rs`
6. `crates/z00z_wallets/tests/test_phase14_pipeline.rs`
7. `crates/z00z_wallets/src/core/tx/lifecycle.rs`
8. `crates/z00z_wallets/examples/wallet_reload.rs`

Therefore the correct action is phased migration, then removal.

### Target end state for Workstream B

Final target state:

1. `core::stealth` is the only public owner of sender-side output construction semantics;
2. low-level leaf builders live under the same canonical owner as the rest of the stealth derivation path;
3. `core::tx` keeps only tx-level assembly, verification, and wire helpers;
4. legacy Stage-4 bundle construction is either internal-only or deleted after migration;
5. no public spec text tells future work to start in `builder.rs` or `output_flow.rs` for sender construction.

### Required implementation steps for Workstream B

#### Phase B1. Re-home low-level leaf builders under the stealth owner

Move or re-home the generic leaf-building helpers:

1. `build_output_leaf(...)`
2. `build_output_with_blind(...)`
3. `build_output_with_rng(...)`

These helpers are not legacy. They are low-level stealth construction helpers and belong under `crates/z00z_wallets/src/core/stealth/`.

Recommended options:

1. move them into `core/stealth/output.rs` if the file stays readable;
2. or create a sibling such as `core/stealth/output_leaf.rs` and re-export from `core::stealth`.

The second option is preferred if file size or responsibility boundaries become unclear.

#### Phase B2. Introduce one canonical full-leaf adapter under `core::stealth`

Because `TxStealthOutput` is lightweight and `AssetLeaf` is full-leaf, the migration cannot be a one-line replacement.

Current lightweight header:

```rust
pub struct TxStealthOutput {
    pub r_pub: [u8; 32],
    pub owner_tag: [u8; 32],
    pub tag16: Option<u16>,
    pub enc_pack: ZkPackEncrypted,
    pub c_amount: [u8; 32],
}
```

Source: `crates/z00z_wallets/src/core/stealth/output.rs`

Current full leaf:

```rust
pub struct AssetLeaf {
    pub asset_id: [u8; 32],
    pub serial_id: u32,
    pub r_pub: [u8; 32],
    pub owner_tag: [u8; 32],
    pub c_amount: [u8; 32],
    pub enc_pack: ZkPackEncrypted,
    pub range_proof: Vec<u8>,
    pub tag16: u16,
}
```

Source: `crates/z00z_core/src/assets/leaf.rs`

Therefore add one canonical full-leaf adapter in `core::stealth` rather than keeping the old `sender_create_output_for(...)` as the de facto bridge forever.

Recommended target shape:

```rust
pub fn build_stealth_leaf_for_card(
    card: &ReceiverCard,
    amount: u64,
    serial_id: u32,
) -> Result<AssetLeaf, WalletError>
```

This name is a proposal for this phase. If a shorter or more consistent house-style name is preferred, choose it once and then update all call sites. Do not keep both names public.

The implementation should be assembled from the same canonical derivation steps already used by `core::stealth` plus the moved low-level leaf helper. Do not call back into the legacy wrapper.

#### Phase B3. Migrate callers

Migration order:

1. simulator Stage-4 and Stage-6 compatibility callers;
2. wallet examples and internal lifecycle helpers;
3. wallet integration tests;
4. public facade exports.

Concrete files to update first:

1. `crates/z00z_simulator/src/scenario_1/stage_6_utils/bridge_output_router.rs`
2. `crates/z00z_simulator/src/scenario_1/stage_4_utils/*`
3. `crates/z00z_wallets/src/core/tx/lifecycle.rs`
4. `crates/z00z_wallets/examples/wallet_reload.rs`
5. `crates/z00z_wallets/tests/test_spend_witness_gate.rs`
6. `crates/z00z_wallets/tests/test_phase14_pipeline.rs`
7. `crates/z00z_wallets/tests/test_phase15_regress.rs`
8. `crates/z00z_wallets/tests/test_s5_sender_examples.rs`

#### Phase B4. Narrow `output_flow.rs`

After caller migration:

1. keep `bind_output_wire(...)`, `compute_tx_digest_from_wire(...)`, and balance helpers only if they still have live callers and belong to tx-level bridge logic;
2. make construction helpers such as `create_output_bundle(...)` and `create_output_bundle_with_rng(...)` internal-only, test-only, or remove them entirely;
3. remove the legacy alias `derive_fee_commitment(...)` if callers are migrated to `derive_balance_commitment(...)` and the alias no longer adds value.

#### Phase B5. Remove public legacy exports from `core::tx`

After migration is green:

1. stop re-exporting `sender_create_output_for(...)` from `core::tx`;
2. stop re-exporting construction-style `create_output_bundle(...)` from `core::tx`;
3. re-export only tx-level helpers that genuinely belong to tx assembly or tx verification.

#### Phase B6. Delete or hard-block the old wrappers

Only after all callers are migrated:

1. delete `sender_create_output_for(...)`, or
2. keep a temporary blocked shim that returns a migration error, then delete it in the next cleanup phase.

The same rule applies to any obsolete `output_flow.rs` construction entrypoint.

## 🧪 Required Verification Matrix

Implementation is not complete until all of the following are true.

### Workstream A verification

1. a valid regular public spend contract with populated nullifiers verifies successfully;
2. malformed `nullifier_hex` fails closed;
3. duplicate nullifiers inside one tx fail closed;
4. signed nullifier drift or structural deterministic mismatch fails closed;
5. comments and tests no longer describe nullifier semantics as an open gap after code lands.

### Workstream B verification

1. there is exactly one public canonical sender construction authority;
2. simulator Stage-4 and Stage-6 continue to pass through the migrated canonical path;
3. wallet examples and tests compile without importing the retired legacy API;
4. `core::tx` no longer publicly teaches the old construction path;
5. no remaining public doc points future work at `builder.rs` or `output_flow.rs` as the canonical sender layer.

## 📚 Documentation Updates Required

After code migration, update the repository docs that describe this area.

At minimum review and update:

1. `.planning/phases/040-spend-proof/040-Spend-Proof-Spec.md`
2. `.planning/temp/Z00Z-ECC-IDEAS.md`
3. `.planning/temp/Z00Z-ECC-SPEC_part1.md`
4. any phase context or requirement file that still describes the shipped spend boundary as an open gap or overstates standalone public-verifier behavior

Required wording rules after closure:

1. describe the shipped regular spend nullifier boundary as implemented only after tests and verifier land, while keeping the standalone public-verifier statement narrow;
2. describe `core::stealth` as the canonical sender authority;
3. describe old `core::tx` sender construction APIs as retired compatibility surfaces or remove them from docs entirely;
4. keep the theorem-level claim narrow unless a later phase truly ships a stronger proof backend.

## ✅ Summary

This phase is deliberately narrow and repository-backed.

It does **not** ask for a new speculative proof architecture.

It does ask for two concrete cleanups that the current codebase is already pointing toward:

1. finish the regular spend closure by keeping the signed public nullifier field and deterministic witness plus structural derivation on one canonical contract;
2. make `core::stealth` the only public authority for sender-side transaction construction and retire the old `core::tx` construction surfaces.

If this spec is executed as written, the result will be:

1. a more honest and more complete regular spend contract;
2. a smaller public surface for legacy construction drift;
3. planning text that aligns with the actual code instead of the old temp-doc architecture.
