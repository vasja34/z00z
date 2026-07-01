# Tari Crypto Integration for Z00Z

**Status:** ✅ Active and current  
**Facade crate:** `z00z_crypto` 0.22.1  
**Last Updated:** 2026-04-05

---

## 🎯 What This Means Today

Z00Z does not expose Tari as a default public namespace.

Instead, Tari is integrated behind the `z00z_crypto` facade with two supported import lanes:

1. `z00z_crypto` root for stable protocol operations and wrappers.
2. `z00z_crypto::expert` for advanced helpers.

This is the current production contract used across the workspace.

---

## 🚀 Current Integration Surface

### ✅ Stable Root

Use the root for day-to-day application and protocol code.

Representative capabilities:

- Pedersen commitment creation and verification
- Range proof creation and verification
- Batch range proof verification
- Z00Z hash, HMAC, KDF, and domain helpers
- Stealth-address ECDH helpers in `ecdh`
- AEAD envelope helpers in `aead`
- Sensitive wrappers such as `Hidden` and `SecretBytes`
- Kernel signature helpers

Example:

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

### ✅ Expert Lane

Use `expert` for helper traits and encoding/password utilities.

Current expert surface:

- `hash_domain`
- `encoding::{from_hex, to_hex, Hex, ByteArray, SafePassword}`
- `traits::{DerivedKeyDomain, DomainSeparatedHasher, DomainSeparation, PublicKeyTrait, SecretKeyTrait}`
- `keys::{RistrettoPublicKey, RistrettoSecretKey}`

Example:

```rust
use z00z_crypto::expert::encoding::{SafePassword, to_hex};

let password = SafePassword::from("wallet-passphrase");
let encoded = to_hex(b"z00z");
# let _ = password;
# let _ = encoded;
```

### ✅ Root-Exported Tari Contracts

Use the root facade when the caller truly needs a supported Tari-backed contract.

Current concrete exports include:

- `BulletproofsPlusService`
- `PedersenCommitmentFactory`
- `ExtendedPedersenCommitmentFactory`
- `RangeProofService`
- `ExtendedRangeProofService`
- `Statement`
- `AggregatedPublicStatement`
- `DiffieHellmanSharedSecret`
- `RistrettoSchnorr`
- `RistrettoComSig`
- `RistrettoComAndPubSig`
- `CommitmentSignature`
- `CommitmentAndPublicKeySignature`
- `SchnorrSignature`

Example:

```rust
use z00z_crypto::{
    BulletproofsPlusService, ExtendedPedersenCommitmentFactory, RangeProofService,
};

let service = BulletproofsPlusService::init(
    z00z_crypto::RANGE_PROOF_BITS_V1,
    z00z_crypto::AGGREGATION_FACTOR,
    ExtendedPedersenCommitmentFactory::default(),
)?;
# let _ = service;
# Ok::<_, tari_crypto::errors::RangeProofError>(())
```

---

## 🔐 Real Workspace Usage

The live codebase already follows this split:

- Stable root commitment and proof helpers are used in:
  - `crates/z00z_core/src/assets/asset_crypto.rs`
  - `crates/z00z_wallets/src/core/tx/builder.rs`
  - `crates/z00z_rollup_node/src/lib.rs`
  - `crates/z00z_simulator/src/scenario_1/stage_3.rs`
- `Hidden` from the stable root is used broadly in wallet, storage, simulator, and core flows.
- `SafePassword` is consumed from `z00z_crypto::expert::encoding` across wallet modules.
- Concrete Tari-backed services and factories are imported from the root `z00z_crypto` facade in wallet and core tests.

This is the implementation reality the documentation must reflect.

---

## 🧭 Import Rules

### ✅ Use These Imports

For stable application code:

```rust
use z00z_crypto::{create_commitment, create_range_proof, Hidden, Z00ZScalar};
```

For advanced helper traits and utilities:

```rust
use z00z_crypto::expert::encoding::{ByteArray, SafePassword, from_hex, to_hex};
use z00z_crypto::expert::traits::{DerivedKeyDomain, DomainSeparation};
```

For supported Tari-backed contracts:

```rust
use z00z_crypto::{
    BulletproofsPlusService,
    ExtendedPedersenCommitmentFactory,
    RistrettoPublicKey,
    RistrettoSecretKey,
};
```

### ❌ Do Not Use These Imports In Application Code

```rust
use tari_crypto::...;
use tari_crypto::tari_utilities::...;
use tari_utilities::...;
```

Those paths bypass the facade and are legacy from older documentation.

---

## 🛰️ Stealth ECDH Guidance

For stealth-address and shared-secret workflows, prefer the Z00Z-owned `ecdh` module instead of raw Tari DH types.

```rust
use z00z_crypto::ecdh::{
    compute_stealth_dh_sender, derive_dh_key, generate_ephemeral_keypair,
    recover_stealth_dh_receiver,
};
```

Use `DiffieHellmanSharedSecret` from the root facade only when a caller explicitly needs that Tari-backed type.

---

## 🚫 Not Integrated Into The Public Facade

The following are not current `z00z_crypto` public-surface features and should not be documented as active integration points:

- `EmojiId`
- `tari_common_types::*`
- Tari encryption helpers from `common_types`
- Any direct public dependency on `tari_common_types`

If these become part of the facade later, document them only after the exports and tests exist in the repository.

---

## 📖 Reference Documents

- [Tari-Crypto-Components-Cookbook.md](./Tari-Crypto-Components-Cookbook.md): complete lane-by-lane reference for current exports.
- [src/lib.rs](../../crates/z00z_crypto/src/lib.rs): authoritative root facade surface.
- [src/expert.rs](../../crates/z00z_crypto/src/expert.rs): authoritative advanced helper surface.
- [src/vendor.rs](../../crates/z00z_crypto/src/vendor.rs): authoritative explicit Tari passthrough surface.

---

## 🔒 Critical Boundary Rule

`crates/z00z_crypto/tari/` is read-only vendor code.

Do not modify vendored Tari sources. Expose or adapt functionality only through `z00z_crypto`.
