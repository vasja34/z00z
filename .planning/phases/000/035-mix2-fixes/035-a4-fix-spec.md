# Fix Spec 1: Sender Workflow Canonicalization

**Status:** Draft for implementation  
**Date:** 2026-04-09  
**Scope:** Wallet sender workflow, legacy sender path convergence, validated card-only entrypoint, documentation split between wallet-local checks and future public-verifier semantics

## 🎯 Objective

Define an implementation-ready, repository-backed plan to eliminate sender-side legacy divergence and document the current canonical sender workflow without importing stale concepts from `.planning/temp` notes.

This document is intentionally based on the current codebase rather than on old planning signatures. Any conflict between this document and temp planning notes must be resolved in favor of the live wallet and crypto implementations.

## ✅ Verified Baseline

The following statements are verified against the current repository:

1. The canonical lightweight sender workflow already exists in `crates/z00z_wallets/src/core/stealth/output.rs` and `output_build.rs`.
2. The validated request-bound sender path already exists via `build_tx_stealth_output_validated(...)`.
3. The raw sender path still exists via `build_tx_stealth_output(...)` and intentionally relies on caller-side validation.
4. Legacy sender paths still exist in `crates/z00z_wallets/src/core/tx/builder.rs` and `crates/z00z_wallets/src/core/tx/output_flow.rs`.
5. Hedged ephemeral scalar generation and duplicate-`R` protection already exist in the canonical stealth path.
6. The dedicated validated card-only sender entrypoint now exists and must remain separated from the raw and request-bound sender surfaces.
7. Current code explicitly distinguishes wallet-local approval and self-check behavior from a future public verifier or consensus contract.

## 🔄 Execution Update - 2026-04-12

The first sender-seam implementation slice is now live in the repository:

1. `approve_card(...)` exists in `crates/z00z_wallets/src/core/stealth/output_build.rs`
    as the wallet-local strict card approval helper for the dedicated card-only
    validated lane, and it now requires a matching stored `TrustLevel::Pinned`
    receiver-card entry rather than tentative TOFU state.
2. `build_card_stealth_output_validated(...)` exists in
    `crates/z00z_wallets/src/core/stealth/output.rs` as the dedicated card-only
    validated entrypoint.
3. `build_tx_stealth_output_validated(...)` remains the request-capable
    validated constructor and was not widened into the card-approval lane.
4. `crates/z00z_wallets/tests/test_s5_misuse_gate.rs` now freezes the live
    approval split between the raw builder, the request-bound validated path,
    and the card-only validated path.

## 🔄 Execution Update - 2026-04-12 (Plan 12)

The downstream sender-regression and documentation-correction slice is now live
in the repository:

1. `crates/z00z_simulator/src/scenario_1/stage_3.rs` now keeps the live
    Stage 3 wire builder on one canonical `to_claim_wire(...)` seam that routes
    claim outputs through `build_tx_stealth_output_for(...)` with per-output
    `SenderWallet` state.
2. `crates/z00z_simulator/tests/test_stage3_nullifier_store.rs` and
    `crates/z00z_simulator/tests/test_claim_tx_pipeline.rs` now reuse that live
    `to_claim_wire(...)` helper instead of reconstructing a shadow sender path in
    test-only glue.
3. `crates/z00z_wallets/src/core/stealth/test_output.rs` now proves two
    repository-backed serial guarantees: the lower leaf-binding formulas change
    when only `serial_id` changes under fixed sender inputs, and the real public
    `build_tx_stealth_output_for(...)` wrapper produces outputs that fail closed
    under the wrong `SenderValidationCtx.serial_id`.
4. `.planning/temp/Z00Z-ECC-SPEC_part1.md` and
    `.planning/temp/Z00Z-ECC-IDEAS.md` now describe the live helper ownership
    honestly: `output_build.rs` owns the tx-output raw sender helper/formula
    seam, while compatibility leaf assembly still remains in `output.rs`.

## 🧭 Anti-Drift Rules

These rules are mandatory while implementing this spec.

1. Do not reintroduce the temp-spec idea that sender workflow must be implemented under `z00z_core::tx`.
2. Do not replace current canonical wallet-owned formulas with older temp-doc variants.
3. Do not change the meaning of current request-bound `tag16` or card-bound `tag16`.
4. Do not collapse wallet-local approval logic into a public-verifier claim.
5. Do not remove raw builders without a compatibility plan for existing tests, examples, and simulator flows.
6. Do not bypass the existing H-3 hedged `r` and duplicate-`R` protections when migrating legacy flows.

## 📌 Canonical Current Workflow

The current canonical sender path is centered in `crates/z00z_wallets/src/core/stealth/`.

### Canonical public entrypoints

Current exported sender-facing entrypoints:

```rust
pub fn build_tx_stealth_output(
    receiver_card: &ReceiverCard,
    payment_request: Option<&PaymentRequest>,
    sender_wallet: &mut SenderWallet,
    tx_digest: &[u8; 32],
    out_index: u32,
    amount: u64,
    asset_id: &[u8; 32],
) -> Result<TxStealthOutput, StealthError>

pub fn build_tx_stealth_output_validated<'a>(
    receiver_card: &ReceiverCard,
    payment_request: Option<&PaymentRequest>,
    build_check: BuildCheck<'a>,
    sender_wallet: &mut SenderWallet,
    tx_digest: &[u8; 32],
    out_index: u32,
    amount: u64,
    asset_id: &[u8; 32],
) -> Result<TxStealthOutput, StealthError>

pub fn build_tx_stealth_output_for(
    receiver_card: &ReceiverCard,
    payment_request: Option<&PaymentRequest>,
    sender_wallet: &mut SenderWallet,
    tx_digest: &[u8; 32],
    out_index: u32,
    amount: u64,
    asset_id: &[u8; 32],
    serial_id: u32,
) -> Result<TxStealthOutput, StealthError>

pub fn build_card_stealth_output_validated<'a>(
    receiver_card: &ReceiverCard,
    build_check: BuildCheck<'a>,
    sender_wallet: &mut SenderWallet,
    tx_digest: &[u8; 32],
    out_index: u32,
    amount: u64,
    asset_id: &[u8; 32],
) -> Result<TxStealthOutput, StealthError>
```

Important behavior already documented in code:

```rust
/// Caller-side receiver validation is mandatory.
/// This builder does not verify `ReceiverCard` signatures and does not call
/// `ValidatePaymentRequest::validate_all()` internally.
```

```rust
/// This is the accepted-flow constructor for request-bound Scenario 1 output
/// building. It enforces explicit request approval and route matching, but it
/// does not upgrade those wallet checks into a public trustless verifier claim.
```

### Canonical derivation path

The canonical sender build path currently performs:

1. Route compatibility checks between `ReceiverCard` and optional `PaymentRequest`
2. Hedged ephemeral scalar derivation with retry on duplicate `R`
3. Sender-side DH and `k_dh` derivation
4. Request-bound or card-bound tag mode selection
5. `owner_tag` computation
6. Pedersen commitment creation for `c_amount`
7. `leaf_ad` derivation
8. `ZkPack` encryption of `AssetPackPlain`
9. Optional sender-side self-validation in the validated path

Current helper-owned build core:

```rust
fn build_output_state(...)
fn build_output_state_with_r(...)
fn build_output_state_with_r_and_rng(...)
fn build_leaf_state(...)
fn build_leaf_state_rng(...)
fn select_r(...)
fn select_r_rng(...)

    let (r, r_pub) = select_r(
        sender_wallet,
        &receiver_card.owner_handle,
        tx_digest,
        out_index,
    )?;
    let view_pk = decode_ristretto_pk(&receiver_card.view_pk)?;
    let dh = compute_dh_sender(&r, &view_pk)?;
    let k_dh = derive_stealth_key(payment_request, &dh);
    let owner_tag = compute_owner_tag(&receiver_card.owner_handle, &k_dh);
    let (blinding, c_amount) = make_amount(amount)?;
    let leaf_ad = compute_leaf_ad(asset_id, serial_id, &r_pub, &owner_tag, &c_amount);
```

### Canonical formula surfaces

Current canonical APIs:

```rust
pub fn derive_k_dh(dh: &[u8; 32]) -> [u8; 32]
pub fn derive_k_dh_with_req(dh: &[u8; 32], req_id: &[u8; 32]) -> [u8; 32]
pub fn derive_s_out(k_dh: &[u8; 32], r_pub: &[u8; 32], serial_id: u32) -> [u8; 32]
```

```rust
pub fn compute_tag16(k_dh: &[u8; 32], leaf_ad: &[u8; 32]) -> u16
pub fn compute_tag16_with_req(k_dh: &[u8; 32], req_id: &[u8; 32]) -> u16
pub fn compute_leaf_ad(
    asset_id: &[u8; 32],
    serial_id: u32,
    r_pub: &[u8; 32],
    owner_tag: &[u8; 32],
    c_amount: &[u8; 32],
) -> [u8; 32]
```

```rust
pub fn compute_owner_tag(owner_handle: &[u8; 32], k_dh: &[u8; 32]) -> [u8; 32]
```

### Canonical `ZkPack` binding

Current `ZkPack` already binds more than a minimal `leaf_ad`-only statement suggests:

```rust
let pack_key = derive_pack_key(k_dh, asset_id, serial_id);
let nonce12 = derive_nonce(leaf_ad, r_pub, asset_id, serial_id);
let aad = make_aad(leaf_ad, r_pub, asset_id, serial_id);
```

This means implementation and documentation must describe the live binding truth instead of simplifying it back down to an older planning abstraction.

## 🚨 Verified Gaps To Fix

### Gap 1: Legacy sender path divergence

The legacy sender path in `crates/z00z_wallets/src/core/tx/builder.rs` historically bypassed the canonical stealth builder and owned a shadow construction lane.

Current note in code:

```rust
/// This is a legacy full-leaf path and does not participate in the H-3
/// duplicate-R retry cache used by `build_tx_stealth_output()`.
pub fn sender_create_output_for(
    card: &ReceiverCard,
    amount: u64,
    serial_id: u32,
) -> Result<AssetLeaf, WalletError> {
```

This was the real gap because it preserved old behavior while the stealth path had already hardened around:

1. Hedged `r`
2. Duplicate-`R` retry policy
3. Shared route checks
4. Shared sender self-check semantics

### Gap 2: Replayable output bundle path divergence

The replayable builder path in `crates/z00z_wallets/src/core/tx/output_flow.rs` historically used an older random-scalar path:

```rust
pub fn create_output_bundle_with_rng<R: rand::CryptoRng + rand::RngCore>(
    receiver: String,
    role: TxOutRole,
    class: AssetClass,
    card: &ReceiverCard,
    value: u64,
    serial_id: u32,
    rng: &mut R,
) -> Result<OutputBundle, String> {
    let r = Z00ZScalar::random(rng);
```

This path is useful for replayable tests and simulator flows. The remaining requirement is to keep it on canonical helper/formula ownership and document it as an explicit replayable/stateless compatibility lane rather than letting it silently masquerade as the wallet-owned hardened sender path.

### Gap 3: Dedicated validated card-only entrypoint closure

The request-aware validated constructor still enforces request approval when a request is present. The dedicated card-only sender API now exists, and the remaining requirement is to keep its approval boundary explicit instead of collapsing it back into the raw or request-bound lanes:

"I am using a bare `ReceiverCard`, and I want the strictest card-only validations the wallet can provide before building the output."

Raw callers can still bypass that stricter lane, so export surfaces, tests, and docs must keep the approval split explicit.

### Gap 4: Documentation mixes approval levels

Current code already distinguishes:

1. wallet-local approval and request/card acceptance
2. lightweight sender self-check
3. future public-verifier or consensus semantics

This distinction must be made explicit in the new documentation and in public comments where legacy language still suggests stronger guarantees than current code actually provides.

## 🏗️ Target Architecture

### Design goal

There should be one canonical sender construction engine reused across:

1. lightweight stealth output builders
2. full `AssetLeaf` sender builders
3. replayable output-bundle helpers
4. validated request-bound flow
5. validated card-only flow

### Target structure

#### 1. Keep `core/stealth/output_build.rs` as the source of truth

Do not move canonical sender derivation into `z00z_core` or back into temp-spec architecture.

Instead, extend the current stealth builder so that legacy paths become thin adapters over it.

#### 2. Keep the dedicated validated card-only entrypoint explicit

Keep the dedicated public API in `crates/z00z_wallets/src/core/stealth/output.rs`
and preserve its approval boundary.

Recommended name:

```rust
pub fn build_card_stealth_output_validated<'a>(
    receiver_card: &ReceiverCard,
    build_check: BuildCheck<'a>,
    sender_wallet: &mut SenderWallet,
    tx_digest: &[u8; 32],
    out_index: u32,
    amount: u64,
    asset_id: &[u8; 32],
) -> Result<TxStealthOutput, StealthError>
```

Why this name:

1. It stays within the identifier-length rule.
2. It follows the existing `build_*_validated` pattern.
3. It states exactly that the validated mode is card-only and not request-bound.

#### 3. Introduce reusable sender approval helpers

The approval layer should be explicit and split into small helpers.

Recommended internal helpers:

```rust
fn approve_card<'a>(
    receiver_card: &ReceiverCard,
    build_check: BuildCheck<'a>,
) -> Result<(), StealthError>

fn approve_request<'a>(
    payment_request: &PaymentRequest,
    receiver_card: &ReceiverCard,
    build_check: BuildCheck<'a>,
    amount: u64,
) -> Result<(), StealthError>
```

Rules:

1. `approve_card` is wallet-local policy, not consensus proof.
2. `approve_request` remains stricter and includes route and request checks.
3. The raw builder remains raw and must keep its current warning semantics.

#### 4. Migrate legacy paths into stealth adapters

`sender_create_output_for(...)` and `create_output_bundle_with_rng(...)` should stop owning sender derivation logic directly.

They should instead:

1. call a canonical stealth build helper
2. convert the resulting `TxStealthOutput` into `AssetLeaf` or `OutputBundle`
3. preserve replay/test ergonomics through explicit extension points rather than through duplicated sender logic

## ⚙️ File-By-File Change Plan

### 1. `crates/z00z_wallets/src/core/stealth/output.rs`

Add:

1. `build_card_stealth_output_validated(...)`
2. comments that clearly separate raw builder semantics from validated builder semantics
3. public documentation that names current guarantees precisely

Keep:

1. `build_tx_stealth_output(...)` as the raw low-level sender entrypoint
2. `build_tx_stealth_output_validated(...)` as the request-capable validated sender entrypoint

Implementation direction:

```rust
pub fn build_card_stealth_output_validated<'a>(
    receiver_card: &ReceiverCard,
    build_check: BuildCheck<'a>,
    sender_wallet: &mut SenderWallet,
    tx_digest: &[u8; 32],
    out_index: u32,
    amount: u64,
    asset_id: &[u8; 32],
) -> Result<TxStealthOutput, StealthError> {
    approve_card(receiver_card, build_check)?;

    let (output, ctx) = build_output_ctx(
        receiver_card,
        None,
        sender_wallet,
        tx_digest,
        out_index,
        amount,
        asset_id,
    )?;
    validate_output_self(&output, &ctx, amount)?;
    Ok(output)
}
```

This intentionally mirrors the existing validated request-bound constructor and avoids introducing a second derivation engine.

### 2. `crates/z00z_wallets/src/core/stealth/output_build.rs`

Add:

1. `approve_card(...)`
2. stronger internal separation between route approval and build derivation
3. optional adapter support for legacy callers that need deterministic or injected scalar selection

Recommended constraints:

1. The helper-owned `build_output_state*` and `build_leaf_state*` path remains the canonical build material seam.
2. `select_r(...)` remains the default wallet-owned scalar policy.
3. If legacy flows need injected randomness for replayability, provide an internal injected-scalar variant instead of duplicating sender derivation.

Recommended internal addition:

```rust
fn build_output_ctx_with_r(
    receiver_card: &ReceiverCard,
    payment_request: Option<&PaymentRequest>,
    r: Z00ZScalar,
    amount: u64,
    asset_id: &[u8; 32],
    serial_id: u32,
) -> Result<(TxStealthOutput, SenderValidationCtx), StealthError>
```

Use this only if replayable flows cannot be migrated cleanly through existing retry-based selection.

Reason:

1. It keeps formulas centralized.
2. It avoids forcing test/replay helpers to own their own sender derivation.
3. It avoids copying old code from `builder.rs` and `output_flow.rs`.

### 3. `crates/z00z_wallets/src/core/tx/builder.rs`

Refactor `sender_create_output_for(...)` so it becomes an adapter over the stealth canonical path.

Current function:

```rust
pub fn sender_create_output_for(
    card: &ReceiverCard,
    amount: u64,
    serial_id: u32,
) -> Result<AssetLeaf, WalletError>
```

Target behavior:

1. Keep the public signature for compatibility unless downstream users are known to be absent.
2. Internally use the canonical stealth builder path.
3. Convert `TxStealthOutput` to `AssetLeaf` in one place.
4. Preserve `serial_id` constraints and full-leaf requirements.
5. Document whether this adapter stays legacy-compatible or becomes a soft-deprecated wrapper.

Important note:

The current lightweight stealth builder fixes `serial_id` to `LIGHT_SERIAL_ID`. That means this refactor cannot blindly reuse it for arbitrary full-leaf semantics unless the canonical build path is first generalized.

Therefore, implementation must first decide one of two repository-backed options:

1. Generalize the canonical stealth build path so `serial_id` is a parameter.
2. Add a canonical full-leaf stealth build helper that still lives under `core/stealth/` and shares the same derivation primitives.

Option 1 is preferred because it reduces duplicated semantics.

### 4. `crates/z00z_wallets/src/core/tx/output_flow.rs`

Refactor `create_output_bundle_with_rng(...)` so replayable output construction no longer reimplements sender derivation with a random `r` in place.

Target behavior:

1. Replayable flows may still inject deterministic or caller-provided randomness.
2. They must do so through a canonical stealth build helper.
3. Conversion to `OutputBundle` must happen after canonical sender derivation, not instead of it.

Preferred approach:

1. Add a stealth-layer helper that accepts an injected `r` or injected scalar source.
2. Keep `output_flow.rs` as an adapter that converts the result to `OutputBundle`.
3. Preserve simulator and test ergonomics.

### 5. `crates/z00z_wallets/src/core/stealth/mod.rs`

Update exports to include the new validated card-only entrypoint.

### 6. `crates/z00z_wallets/src/core/stealth/test_output.rs`

Add tests for:

1. successful validated card-only construction
2. rejected invalid signed card
3. rejected identity or view key parse failure
4. sender self-check success for the new card-only validated path

### 7. `crates/z00z_wallets/src/core/stealth/test_output_extra.rs`

Add negative tests for:

1. card validation failure paths
2. request/card approval separation
3. no regression in current request-bound validated behavior

### 8. `crates/z00z_wallets/src/core/tx/` tests and adapters

Update any tests relying on the old sender derivation path so they assert the same visible output behavior while using the canonical stealth path under the hood.

### 9. Documentation targets

Update the relevant planning and code comments so they clearly separate:

1. raw sender construction
2. wallet-local validated sender construction
3. sender self-check
4. future public-verifier semantics

Likely documentation files:

1. `.planning/temp/Z00Z-ECC-SPEC_part1.md`
2. `.planning/temp/Z00Z-ECC-IDEAS.md`
3. any wallet-facing stealth module docs that currently overstate guarantees

## 🧪 Testing Plan

### Required unit coverage

1. `build_card_stealth_output_validated(...)` succeeds for a valid signed `ReceiverCard`.
2. It fails when `ReceiverCard::verify()` would fail.
3. It fails on invalid point encoding through strict untrusted parse paths where applicable.
4. Existing request-bound validated tests remain green.
5. Legacy adapters produce equivalent visible outputs after migration.
6. Replayable `output_flow.rs` tests preserve deterministic behavior where required.

### Required regression coverage

1. no regression in hedged `r` path
2. no regression in duplicate-`R` cache behavior
3. no regression in request-bound `tag16`
4. no regression in card-bound `tag16`
5. no regression in sender self-validation

### Suggested verification commands

Run at minimum:

```bash
cargo test -p z00z_wallets --release --features test-fast --features wallet_debug_dump
```

Then run the workspace bootstrap gate already used in this repository:

```bash
./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh
```

If `output_flow.rs` or simulator callers are touched, run the release-style simulator command expected in this workspace.

## 🔐 Security and Correctness Requirements

1. All validated entrypoints must fail closed.
2. Raw entrypoints must keep explicit documentation that validation is caller-owned.
3. No path may silently downgrade from hedged `r` to plain random `r` without an explicit replay/test contract.
4. No path may reintroduce older formulas for `owner_tag`, `tag16`, `leaf_ad`, or `enc_pack` binding.
5. No documentation may describe wallet-local approval as if it were a consensus-level proof guarantee.

## 🚦 Migration Sequence

### Phase A: Canonical helper extension

1. Add `approve_card(...)`.
2. Add validated card-only entrypoint.
3. Add any needed canonical helper for injected-`r` or variable `serial_id` support.
4. Prove no request-bound regression.

### Phase B: Legacy adapter convergence

1. Refactor `sender_create_output_for(...)` to use canonical stealth helpers.
2. Refactor `create_output_bundle_with_rng(...)` to use canonical stealth helpers.
3. Keep old public signatures where possible.
4. Update tests and examples.

### Phase C: Documentation correction

1. Rewrite stale temp planning text to describe current architecture honestly.
2. Remove or mark stale signatures as historical or superseded.
3. Document the approval-level split explicitly.

## ✅ Acceptance Criteria

This spec is complete only when all of the following are true:

1. There is a dedicated validated card-only sender entrypoint.
2. Legacy sender derivation in `builder.rs` is removed or reduced to an adapter over canonical stealth helpers.
3. Legacy sender derivation in `output_flow.rs` is removed or reduced to an adapter over canonical stealth helpers.
4. Current request-bound validated behavior remains unchanged at the observable API level unless an intentional change is documented and tested.
5. Documentation no longer claims the sender workflow is merely conceptual.
6. Documentation explicitly separates wallet-local approval, sender self-check, and future public-verifier semantics.
7. No formula-level concept drift is introduced during implementation.

## 📚 Reference Snippets

### Current request validation boundary

```rust
// signature, chain, expiry, and TOFU or pinning rules, but it is still
// a wallet-local approval boundary rather than a public verifier claim.
```

### Current sender self-check boundary

```rust
/// This is a lightweight accepted-flow sender self-check. It is not the final
/// public spend verifier contract for Scenario 1.
```

### Current receiver card export and rotation support

```rust
pub fn rotate_view(&mut self) -> Result<ReceiverCard, StealthKeyError>
pub fn export_receiver_card(&self) -> Result<ReceiverCard, StealthKeyError>
```

These existing surfaces matter because card-only validation and legacy-path convergence must work with the wallet's real signed-card lifecycle instead of reintroducing a conceptual card model.

## 🔗 TODO One-To-One Mapping

| 035-4 section | Task coverage | Mapping note |
| --- | --- | --- |
| `Objective` | `035-22`; `035-23` | freezes sender canonicalization as a real implementation target |
| `Verified Baseline` | `035-22`; `035-23`; `035-30` | preserves the repository-backed starting point before edits land |
| `Anti-Drift Rules` | `035-22`; `035-29`; `035-30`; `035-31` | guards against terminology drift and fake closure |
| `Canonical Current Workflow` | `035-22`; `035-23`; `035-24`; `035-25`; `035-26`; `035-27`; `035-30` | maps current entrypoints, derivation path, formulas, and binding semantics into execution work |
| `Canonical public entrypoints` | `035-22`; `035-24`; `035-27`; `035-29` | covers raw, serial-aware raw, request-bound validated, and card-only entrypoint truth plus doc corrections |
| `Canonical derivation path` | `035-23`; `035-25`; `035-26`; `035-30` | drives helper extension and legacy-adapter convergence |
| `Canonical formula surfaces` | `035-23`; `035-27`; `035-30` | freezes formula-level correctness and regression checks |
| `Canonical \`ZkPack\` binding` | `035-23`; `035-27`; `035-30` | keeps the binding path canonical while coverage expands |
| `Verified Gaps To Fix` | `035-23`; `035-24`; `035-25`; `035-26`; `035-29` | each verified gap becomes a concrete execution wave |
| `Target Architecture` | `035-23`; `035-24`; `035-25`; `035-26`; `035-29` | converts the desired end-state structure into implementation order |
| `File-By-File Change Plan` | `035-23`; `035-24`; `035-25`; `035-26`; `035-27`; `035-28`; `035-29` | aligns concrete files to the planned task waves |
| `Testing Plan` | `035-27`; `035-28`; `035-30` | maps unit, regression, and verification commands to validation tasks |
| `Security and Correctness Requirements` | `035-24`; `035-25`; `035-26`; `035-30`; `035-31` | keeps validated behavior and fail-closed rules explicit |
| `Migration Sequence` | `035-23`; `035-25`; `035-26`; `035-29` | preserves the helper-first, adapter-second, docs-third order |
| `Acceptance Criteria` | `035-30`; `035-31` | closes the sender lane only on tested architectural convergence |
| `Reference Snippets` | `035-22`; `035-24`; `035-29`; `035-30` | anchors the backlog to the current repo evidence and wording boundaries |
