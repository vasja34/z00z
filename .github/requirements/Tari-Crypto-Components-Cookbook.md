# 📚 Tari Crypto Components Cookbook for Z00Z

**Date:** 2026-04-05  
**Status:** Current as of `z00z_crypto` 0.22.1  
**Purpose:** Accurate reference for consuming vendored Tari functionality through the `z00z_crypto` facade

---

## 🎯 Executive Summary

`z00z_crypto` is the only supported entry point for cryptographic functionality in the Z00Z workspace.

Application and protocol code must treat the crate as a facade with a stable root and a narrow advanced lane:

1. `z00z_crypto` root: stable Z00Z-owned API for protocol operations and wrappers.
2. `z00z_crypto::expert`: advanced helper traits, encoding helpers, and concrete key types.

Do not write new application code that imports `tari_crypto::*` or `tari_utilities::*` directly.
Do not use the internal vendor subpath from workspace code.

---

## 🧭 Facade Contract

### ✅ Default Lane: Stable Root

Use the crate root for protocol-facing work that should stay backend-agnostic.

Representative root exports from `src/lib.rs`:

- `create_commitment`
- `create_range_proof`
- `create_range_proof_rng`
- `verify_range_proof`
- `batch_verify_range_proofs`
- `batch_verify_range_proofs_with`
- `derive_hash`
- `sign_kernel_signature`
- `verify_kernel_signature`
- `Hidden`
- `SecretBytes`
- `Z00ZScalar`
- `Z00ZCommitment`
- `RangeProof`
- `KernelSignature`
- `commitments::{Commitment, CommitmentOpening, BlindingFactorGenerator, generate_blinding_factor}`
- `range_proofs::{AssetRangeProof, AssetOutputProof, verify_asset_output_proofs_batch}`
- `ecdh::*`
- `aead::*`

### ✅ Advanced Lane: `expert`

Use `z00z_crypto::expert` only when you need advanced helpers that are still part of the supported facade, but are intentionally not on the default root surface.

Current expert exports from `src/expert.rs`:

- `hash_domain`
- `encoding::{from_hex, to_hex, Hex, ByteArray, SafePassword}`
- `traits::{DerivedKeyDomain, DomainSeparatedHasher, DomainSeparation, PublicKeyTrait, SecretKeyTrait}`
- `keys::{RistrettoPublicKey, RistrettoSecretKey}`

### ✅ Root-Exported Tari-Backed Contracts

Concrete Tari-backed factories and services are exposed through the root `z00z_crypto` facade when they are part of the supported public surface.

Current root re-exports include:

- `HomomorphicCommitmentFactory`
- `DiffieHellmanSharedSecret`
- `AggregatedPublicStatement`
- `ExtendedRangeProofService`
- `Statement`
- `RangeProofService`
- `BulletproofsPlusService`
- `RistrettoAggregatedPublicStatement`
- `RistrettoStatement`
- `PedersenCommitmentFactory`
- `ExtendedPedersenCommitmentFactory`
- `CompressedRistrettoComAndPubSig`
- `RistrettoComAndPubSig`
- `RistrettoComSig`
- `RistrettoSchnorr`
- `CommitmentAndPublicKeySignature`
- `CommitmentSignature`
- `SchnorrSignature`
- `RistrettoPublicKey`
- `RistrettoSecretKey`

---

## 🔐 What To Import For Real Tasks

### 💰 Confidential Amounts and Range Proofs

Use the stable root for most flows.

```rust
use z00z_crypto::{
    create_commitment, create_range_proof, verify_range_proof, Hidden, Z00ZScalar,
    MIN_VALUE_PROMISE, RANGE_PROOF_BITS_V1,
};

let blinding = Hidden::hide(Z00ZScalar::random(&mut rng));
let commitment = create_commitment(1000, blinding.reveal())?;
let proof = create_range_proof(
    1000,
    blinding.reveal(),
    RANGE_PROOF_BITS_V1,
    MIN_VALUE_PROMISE,
)?;
verify_range_proof(
    &proof,
    &commitment,
    RANGE_PROOF_BITS_V1,
    1,
    MIN_VALUE_PROMISE,
)?;
# Ok::<_, z00z_crypto::CryptoError>(())
```

This pattern matches actual workspace usage in `z00z_core`, `z00z_wallets`, `z00z_rollup_node`, and `z00z_simulator`.

### 🔏 Concrete Tari Proof Services and Factories

Use the root facade when a caller needs a supported Tari-backed service or factory type.

```rust
use z00z_crypto::{
    BulletproofsPlusService, ExtendedPedersenCommitmentFactory, RangeProofService,
};

let service = BulletproofsPlusService::init(
    z00z_crypto::RANGE_PROOF_BITS_V1,
    z00z_crypto::AGGREGATION_FACTOR,
    ExtendedPedersenCommitmentFactory::default(),
)?;
# Ok::<_, tari_crypto::errors::RangeProofError>(())
```

This matches real code in `z00z_wallets/src/core/tx/prover.rs` and wallet dispatcher wiring.

### 🔐 Sensitive Value Handling

Use `Hidden` from the stable root.

```rust
use z00z_crypto::{Hidden, Z00ZScalar};

let hidden_scalar = Hidden::hide(Z00ZScalar::random(&mut rng));
let revealed = hidden_scalar.reveal();
```

This is the current workspace default in wallet, storage, simulator, and core modules.

### 🔑 Password Handling and Encoding Helpers

Use the expert lane for `SafePassword`, `ByteArray`, and hex helpers.

```rust
use z00z_crypto::expert::encoding::{from_hex, to_hex, SafePassword};

let password = SafePassword::from("wallet-passphrase");
let hex = to_hex(b"z00z");
let bytes = from_hex(&hex)?;
# let _ = password;
# let _ = bytes;
# Ok::<_, tari_crypto::tari_utilities::hex::HexError>(())
```

This matches broad real usage throughout `z00z_wallets`.

### 🧬 Domain Separation and Key Traits

Use the expert lane for macro-based domain definitions and trait-based derivation helpers.

```rust
use z00z_crypto::expert::{hash_domain, traits::DerivedKeyDomain};
use z00z_crypto::expert::keys::RistrettoSecretKey;

hash_domain!(WalletDomain, "z00z/wallet/keys", 1);

impl DerivedKeyDomain for WalletDomain {
    type DerivedKeyType = RistrettoSecretKey;
}
```

This mirrors current workspace patterns in storage, wallet, and core domain modules.

### 🛰️ Stealth ECDH

Prefer the stable `ecdh` module over raw Tari DH helpers for stealth-address workflows.

```rust
use z00z_crypto::ecdh::{
    compute_stealth_dh_sender, derive_dh_key, generate_ephemeral_keypair,
    recover_stealth_dh_receiver,
};
```

Use `DiffieHellmanSharedSecret` from the root facade if a caller explicitly needs that Tari-backed type.

---

## 🧱 Component Map By Lane

### 🔹 Stable Root and Z00Z Wrappers

| Surface | Current status | Notes |
| --- | --- | --- |
| Commitments | ✅ Default | `create_commitment`, `commitments::Commitment`, `verify_opening` |
| Range proofs | ✅ Default | `create_range_proof`, `verify_range_proof`, `AssetRangeProof` |
| Batch proof verification | ✅ Default | `batch_verify_range_proofs`, `batch_verify_range_proofs_with` |
| ECDH stealth helpers | ✅ Default | `ecdh::*` is the canonical stealth flow |
| AEAD envelope helpers | ✅ Default | `aead::*`, `aead_transport::*` |
| Hashing and HMAC | ✅ Default | `derive_hash`, `blake2b_hash`, `poseidon2_hash`, `hmac_*` |
| KDF and wallet derivation | ✅ Default | `kdf::*`, `kdf_consensus`, `kdf_extended` |
| Sensitive wrappers | ✅ Default | `Hidden`, `SecretBytes`, `SecretBytes32` |
| Claim crypto | ✅ Default | claim v2 is part of the root surface |

### 🔸 Expert Lane

| Surface | Current status | Notes |
| --- | --- | --- |
| Hex helpers | ✅ Available | `from_hex`, `to_hex`, `Hex` |
| Byte encoding trait | ✅ Available | `ByteArray` |
| Safe password wrapper | ✅ Available | `SafePassword` |
| Concrete Tari key types | ✅ Available | `RistrettoPublicKey`, `RistrettoSecretKey` |
| Domain separation traits | ✅ Available | `DerivedKeyDomain`, `DomainSeparatedHasher`, `DomainSeparation` |
| Key traits | ✅ Available | `PublicKeyTrait`, `SecretKeyTrait` |

### 🔺 Root-Exported Tari Contracts

| Surface | Current status | Notes |
| --- | --- | --- |
| Pedersen factories | ✅ Available | `PedersenCommitmentFactory`, `ExtendedPedersenCommitmentFactory` |
| Bulletproofs+ service | ✅ Available | `BulletproofsPlusService` |
| Extended range proof contracts | ✅ Available | `ExtendedRangeProofService`, `Statement`, `AggregatedPublicStatement` |
| Tari DH type | ✅ Available | `DiffieHellmanSharedSecret` |
| Schnorr and commitment signatures | ✅ Available | `RistrettoSchnorr`, `RistrettoComSig`, `CommitmentSignature`, others |
| Concrete Tari key types | ✅ Available | Re-exported through the root facade |

---

## 🚫 What Is Not Part Of The Current Public Surface

The following items are not part of the current `z00z_crypto` facade and must not be documented as active integration points:

- `EmojiId`
- `tari_common_types::*`
- `encrypt_bytes_integral_nonce`
- `decrypt_bytes_integral_nonce`
- Any direct `tari_crypto::*` import path for application code
- Any suggestion that the crate root is a public Tari namespace

If future work adds these features, document them only after they are exported from `z00z_crypto` and used by real workspace code.

---

## 🧪 Real Usage Anchors In The Workspace

Use the codebase as the source of truth when updating this document.

Current representative usage locations:

- Stable root commitments and proofs:
  - `crates/z00z_core/src/assets/asset_crypto.rs`
  - `crates/z00z_wallets/src/core/tx/builder.rs`
  - `crates/z00z_rollup_node/src/lib.rs`
  - `crates/z00z_simulator/src/scenario_1/stage_3.rs`
- `Hidden` root wrapper:
  - `crates/z00z_wallets/src/core/storage/file_key_store.rs`
  - `crates/z00z_wallets/src/services/seed_phrase.rs`
  - `crates/z00z_simulator/src/actors.rs`
- Expert `SafePassword` and encoding helpers:
  - multiple modules under `crates/z00z_wallets/src/`
- Root-facade imports of Tari-backed contracts:
  - `crates/z00z_wallets/src/core/tx/prover.rs`
  - `crates/z00z_wallets/src/adapters/rpc/wallet_dispatcher_wiring.rs`
  - `crates/z00z_core/src/assets/test_wire.rs`

---

## ✅ Documentation Rules For Future Updates

When updating this cookbook:

1. Verify exports against `src/lib.rs`, `src/expert.rs`, and `src/vendor.rs`.
2. Prefer examples that mirror real imports already present in workspace crates.
3. Do not mark something as integrated unless it is exported today.
4. Do not suggest direct imports from vendored Tari crates for application code.
5. Treat `crates/z00z_crypto/tari/` as read-only vendor code.
6. Keep workspace-facing imports on the root facade; do not teach or depend on the internal vendor subpath.

---

## 🔒 Vendor Boundary Reminder

`crates/z00z_crypto/tari/` is vendored source and must not be modified.

To expose new functionality:

1. Add the export in `z00z_crypto`.
2. Add or update tests in `z00z_crypto` or consuming crates.
3. Update this cookbook only after the export exists and is validated.
