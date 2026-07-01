# Crypto Architect Audit — `z00z_wallets` Crate

**Crate:** `crates/z00z_wallets/`  
**Scope:** All `*.rs` implementation files; `tari/` vendor excluded  
**Date:** 2026-03-26  
**Confidence:** High (examined ~60% of source files, multiple critical paths reviewed end-to-end)  
**Executive Verdict:** **Risky but Salvageable** — 1 S1, 4 S2, 2 S3, 5 S4 findings. Concrete fixes documented. Phase 1 stub density is high; production readiness requires completing stubbed transaction and chain-service paths.

---

## 1. Input Type and Scope

**Input type:** Source code implementation review (35+ `.rs` files across `core/key`, `core/domains`, `core/stealth`, `core/hashing`, `db/`, `services/`).

**Scope summary:**

| Layer | Files | Status |
|---|---|---|
| Key derivation (`core/key/`) | `key_manager.rs`, `bip32.rs`, `seed.rs`, `stealth_keys.rs` | Substantial implementation |
| Domain separation (`core/domains.rs`) | 50+ domain identifiers | Complete |
| Hashing (`core/hashing.rs`) | Domain-separated hasher aliases | Complete |
| Stealth facades (`core/stealth/`) | `facade_ecdh.rs`, `facade_kdf.rs` | Complete |
| RedB crypto (`db/redb_wallet_crypto.rs`) | AEAD envelopes, KDF params, key schedule | Substantial implementation |
| Wallet service (`services/wallet_service.rs`) | ~5700 lines | Partial stub |
| Transaction service (`services/tx_service.rs`) | All stubs | Stub |
| Backup service (`services/backup_service.rs`) | All stubs | Stub |
| Chain service (`services/chain_service.rs`) | All stubs | Stub |

---

## 2. Security Goals Assumed

1. **Confidentiality** — Master seed, receiver secrets, view SKs never appear in logs or error messages
2. **Integrity** — Wallet data at rest is authenticated (AEAD), index MACs prevent tampering
3. **Authentication** — Wallet open/unlock requires password proof under Argon2id
4. **Key separation** — Spend/view keys are in separate BIP-44 account namespaces; different chain types use distinct Ristretto domains
5. **Non-malleability** — Schnorr signatures use RFC 6979 deterministic nonces
6. **Entropy quality** — Seeds validated for catastrophic weakness before use

---

## 3. Threat Model Summary

| Actor | Assumption |
|---|---|
| Passive network observer | Can observe wallet RPC traffic; encrypted payloads only |
| Attacker with read access to wallet file (`.wlt`) | Cannot decrypt without correct password |
| Attacker with wallet file + partial password guess | Argon2id hardens brute force; 128 MiB memory, 5 iterations |
| Malicious scanning service | Receives view key; cannot spend; scan is non-interactive |
| Memory-dump attacker | `Hidden<T>` + `Zeroize` mitigate long-term secret exposure |
| Cache-poisoning attacker | Spot-check integrity validation on every 100 derivations |

**Failure model:** Crash, entropy failure, DoS on KDF (mitigated via `MAX_KDF_TIME_MS` guard), gap limit exceeded.

---

## 4. Critical and High Findings (S0/S1)

### S1 — `expect()` in Crypto Path Burns Diagnostic Information

**Component:** `services/wallet_service.rs:5453–5559`  
**Problem:** `unwrap()`, `expect()`, and `try_into().expect()` appear throughout the backup export and cryptographic object construction path:

```rust
// wallet_service.rs:5453
let aad_json = codec.serialize(aad).expect("serialize aad");
// wallet_service.rs:5488
let plaintext = codec.serialize(&payload).expect("serialize payload");
// wallet_service.rs:5489
let compressed = zstd_compress(&plaintext).expect("compress payload");
// wallet_service.rs:5495 .expect("derive key");
// wallet_service.rs:5508 .expect("encrypt payload");
// wallet_service.rs:5532
view_pk: keys.view_pk.as_bytes().try_into().expect("view pk"),
// wallet_service.rs:5533
identity_pk: keys.identity_pk.as_bytes().try_into().expect("identity pk"),
// wallet_service.rs:5550 .expect("output");
// wallet_service.rs:5553
z00z_core::genesis::asset_std::asset_from_dev_cfg("z00z", 0, amount).expect("asset");
// wallet_service.rs:5559 .expect("commitment");
```

**Impact:** Panic on any serialization failure, zstd error, or type conversion error. In production, this crashes the wallet process. No error code is returned to the caller. Also: the `expect()` messages contain no error context, making post-mortem diagnosis impossible.

**Fix:** Replace all `expect()` in crypto paths with `WalletResult<T>` + `?` propagation using the existing `WalletError` enum. For `try_into().expect()`, validate sizes at construction time or use `try_into()` returning a proper error.

---

### S1 — `chain_service.rs` Discards Async Results with `.unwrap()`

**Component:** `services/chain_service.rs:267,276,285,286`  
**Problem:**

```rust
let result = service.switch_to_mainnet().await.unwrap();  // line 267
let result = service.switch_to_testnet().await.unwrap();  // line 276
service.switch_to_mainnet().await.unwrap();              // line 285
let result = service.switch_to_devnet().await.unwrap();   // line 286
```

**Impact:** Panic if any network service call fails. In a wallet handling real user funds, a network timeout or chain service error causes process termination.

**Fix:** Propagate as `WalletResult<()>` using `?` or `.await?`.

---

## 5. Medium Findings (S2)

### S2 — BIP-44 Gap Limit Uses `saturating_sub` Which Masks Logical Errors

**Component:** `core/key/key_manager.rs` — `next_external()`, `next_internal()`  
**Problem:** The gap is computed as `next_index.saturating_sub(last_used_plus1)`. If `last_used_plus1 > next_index` (which should be impossible under correct atomic ordering), `saturating_sub` returns 0 instead of panicking or returning an error. This masks invariant violations.

**Impact:** If a bug causes `last_used_plus1` to advance beyond `next_index` through a race, the gap silently appears as 0, and derivation proceeds without error. This could break the BIP-44 gap limit invariant.

**Fix:** Use checked subtraction (`checked_sub`) and return `Err(GapLimitExceeded)` if the result is `None`, since this indicates a violated invariant rather than a legitimate gap state.

---

### S2 — Cache TTL Eviction Not Enforced — TTL Field Stored But Never Checked

**Component:** `core/key/key_manager.rs` — `CachedKey` struct, `DERIVED_KEY_TTL_SECONDS = 1800`  
**Problem:** The `CachedKey` struct stores `cached_at: u64`, and `DERIVED_KEY_TTL_SECONDS` is defined as 1800 seconds, but **no code ever checks `cached_at` against the current time**. Cached entries are never evicted based on TTL. The TTL field is written but never read for eviction purposes.

**Impact:** Stale cached entries persist indefinitely (until LRU eviction at 256 entries). This is not a direct security flaw (only public keys are cached), but it defeats the TTL design intent and may cause memory growth if many keys are derived with long gaps.

**Fix:** On every cache read, check `now - cached_key.cached_at > DERIVED_KEY_TTL_SECONDS` and treat an expired entry as a cache miss, re-deriving the key.

---

### S2 — `ReceiverSecret::from_bytes` Only Checks Zero, Not Identity Point

**Component:** `core/key/stealth_keys.rs`  
**Problem:** `ReceiverSecret::from_bytes` validates that bytes ≠ all zeros (`ct_eq(&[0u8; 32])`), but does **not** validate that the scalar derived from those bytes is not the Ristretto identity point after mapping to the curve:

```rust
fn from_raw(bytes: [u8; 32]) -> Result<Self, StealthKeyError> {
    if bytes.ct_eq(&[0u8; 32]).unwrap_u8() == 1 {
        return Err(StealthKeyError::ZeroSecret);
    }
    Ok(Self(bytes))
}
```

Later, `derive_view_public_key` checks `is_identity()`:

```rust
pub fn derive_view_public_key(view_sk: &Z00ZScalar) -> Result<Z00ZRistrettoPoint, StealthKeyError> {
    let key = Z00ZRistrettoPoint::from_secret_key(view_sk);
    if key.as_bytes() == [0u8; 32] {   // ← identity check
        return Err(StealthKeyError::IdentityPointRejected);
    }
    Ok(key)
}
```

But the **receiver secret itself** could map to a view SK that is the identity point, and this is only caught later in the key derivation chain, not at `ReceiverSecret` construction.

**Impact:** A receiver secret that maps to an identity view SK is rejected at `derive_view_public_key()`, not at `ReceiverSecret::from_bytes()`. This means the secret is stored/loaded but unusable for derivation — potentially causing confusing failures at usage time rather than at wallet creation.

**Fix:** Add a method `ReceiverSecret::validate_usable(&self) -> Result<(), StealthKeyError>` that calls `derive_view_secret_key()` and `derive_view_public_key()` and returns an error if the identity point is produced. Call this at wallet creation and at receiver secret load-from-storage.

---

### S2 — `WalletEntropyFromRngProvider` Is Documented But Has No Constant-Time Guarantee

**Component:** `services/wallet_service.rs` — `WalletEntropyFromRngProvider`  
**Problem:** The `WalletEntropy` trait's `fill_bytes` method is documented as the randomness boundary, but the trait has no guarantee that the implementation uses a cryptographically secure RNG. A future implementor could accidentally use `rand::WeakRng` or `rand::StdRng` without realizing the security implications.

```rust
trait WalletEntropy: Send + Sync {
    fn fill_bytes(&self, dest: &mut [u8]);
}
```

**Impact:** If a caller substitutes a weak RNG, seed generation becomes guessable. The trait is `pub(crate)` and the only implementation (`WalletEntropyFromRngProvider`) uses `SecureRngProvider`, but this is not enforced by the type system.

**Fix:** Add a `CryptoRng` bound to the trait:

```rust
trait WalletEntropy: Send + Sync {
    fn fill_bytes(&self, dest: &mut [u8]);
}
```

Or rename to `CryptoWalletEntropy` and document the security contract explicitly in the doc comment.

---

### S2 — Entropy Validation Has Accept-After-Warning Path With No User Notification

**Component:** `core/key/seed.rs` — `validate_entropy()`  
**Problem:** Non-critical entropy warnings (UnusualBitCount) emit a log message but **return `Ok(())`**, meaning the wallet is created successfully with no explicit signal to the caller that a warning occurred:

```rust
// Non-critical warnings: log but accept.
let logger = TracingLogger;
for warning in &warnings {
    let msg = format!("Entropy heuristic warning: {:?}", warning);
    logger.warn(&msg);
}
Ok(())
```

**Impact:** A wallet created with marginal entropy (e.g., suspiciously few bits set) silently succeeds. The caller has no API-level signal that a warning was emitted — they would need to check logs. For user-facing wallet creation, this is a silent warning path.

**Fix:** Return warnings in the `Result` type or add a separate `ValidationResult` enum that distinguishes `Ok(())` (clean), `OkWithWarnings(Vec<EntropyWarning>)`, and `Err`. The caller can then decide whether to proceed or prompt the user.

---

## 6. Low and Info Findings (S3/S4)

### S3 — `derive_s_out` Is Exposed as Public Re-export Without Documentation

**Component:** `core/stealth/facade_kdf.rs`  
**Problem:** `derive_s_out` is imported from `super::ecdh::derive_s_out` and re-exported with no doc comment. The security properties (what it derives, how it should be used, what domain separation it relies on) are undocumented at the facade layer.

**Fix:** Add a doc comment explaining that `s_out` is the stealth output secret derived from DH key material, and that callers must ensure the input DH key and ephemeral public key are correctly bound to the transaction context.

---

### S3 — `generate_identity_keypair()` Warning Label Is Inconsistent With Security Notes

**Component:** `core/key/stealth_keys.rs` — `generate_identity_keypair()`  
**Problem:** The function has a `#[cfg(test)]` counter (`ID_GEN_COUNT`) and test-only reset functions, but the function itself is `pub` (not `#[cfg(test)]`). The comment says "Do NOT use for wallet identity keys" but the function is in the public API of the module and could be called by mistake in production.

**Impact:** A developer could mistakenly use this for non-ephemeral identity, breaking wallet recoverability. The function generates a random keypair each call — it is not deterministic from the BIP-39 seed.

**Fix:** Either mark `generate_identity_keypair()` as `#[cfg(test)]` only (export a `test_generate_identity_keypair` instead for tests), or add a deprecation warning and rename to `generate_ephemeral_identity_keypair` to signal the non-recoverable nature.

---

### S4 — `LruCache` Capacity Check Uses `unwrap()` in Constructor

**Component:** `core/key/key_manager.rs`  
**Problem:**

```rust
derived_public_keys: RwLock::new(LruCache::new(
    NonZeroUsize::new(MAX_DERIVED_PUBKEY_CACHE).unwrap(),  // MAX_DERIVED_PUBKEY_CACHE = 256
)),
```

**Impact:** If `MAX_DERIVED_PUBKEY_CACHE` is ever set to 0 (a code error), this panics at construction rather than failing gracefully. With a hardcoded constant of 256, this is extremely low risk, but the pattern is fragile.

**Fix:** Add ancompile-time assertion: `const _: () = assert!(MAX_DERIVED_PUBKEY_CACHE > 0);`.

---

### S4 — `SpotCheckCache` Frequency Counter Uses `Relaxed` Ordering on Decoding

**Component:** `core/key/key_manager.rs` — `derive_count`  
**Problem:** The spot-check trigger increments `derive_count` using `Relaxed` ordering (permissive but correct for a non-synchronization counter). However, the cache spot-check itself is a correctness mechanism, not a synchronization primitive. Using `Relaxed` is technically correct here, but the relationship between `derive_count` and the spot-check trigger decision could be documented more clearly.

**Fix:** Add a comment explaining that `Relaxed` ordering is sufficient because the spot-check is advisory (it does not affect correctness if two threads see stale counts, only performance).

---

### S4 — `WalletEntropyFromRngProvider` Wraps `SecureRngProvider` But Does Not Assert It

**Component:** `services/wallet_service.rs`  
**Problem:** The struct wraps `P: SecureRngProvider` but the trait has no `Send + Sync` bound on `SecureRngProvider` itself. The `WalletEntropyFromRngProvider<P>` impl does have `Send + Sync`, but only because `P: SecureRngProvider + Send + Sync` — the `Send + Sync` is assumed from the trait, not verified.

**Impact:** Minimal — if `P: SecureRngProvider` but not `Send`, the impl won't compile due to the `impl WalletEntropy for WalletEntropyFromRngProvider<P> where P: SecureRngProvider + Send + Sync` bound.

---

### S4 — `KeyManagerImpl::unlock_from_storage` Copies Seed Bytes on Every Unlock

**Component:** `core/key/key_manager.rs`  
**Problem:** Each unlock copies the 64-byte seed from the RedB session into the `KeyManagerImpl` via `init_from_seed()`. There is no way to perform key derivation directly from the opened session without an extra copy. The seed bytes land in the BIP-44 manager's internal state.

**Impact:** The seed bytes persist in the `Bip44KeyManager` for the lifetime of the `KeyManagerImpl`. On `clear()`, they are zeroized. The copy count is bounded (1 per unlock, N unlocks over time), but the session already holds the seed — the second copy is redundant from a security perspective (though not from an API design perspective, since the key manager owns its state).

**Fix:** Accept this as a known tradeoff of the current API design. Consider adding a `KeyManager::init_from_session()` method that takes the opened session's seed reference without an additional heap copy.

---

## 7. Open Ambiguities

### Open 1 — `CipherSeedContainer::to_bytes` Inner Format Is Opaque to This Crate

**Description:** `KeyManagerImpl::state_from_encrypted_seed()` calls `encrypted_seed.to_bytes()` and stores the result as `Vec<u8>`. The inner format of the encrypted seed (how the envelope is structured) is defined in `z00z_crypto` or a lower crate and is not visible in `z00z_wallets`. If the format changes, existing persisted `KeyManagerState` records may become unreadable.

**What prevents proof of security:** No version marker on the serialized `encrypted_seed_bytes` field within `KeyManagerState`. If `CipherSeedContainer` adds a new version, old `KeyManagerState` records will silently deserialize incorrectly.

**Owner:** `z00z_crypto` / `z00z_wallets` — needs a versioned wrapper around the encrypted seed bytes in `KeyManagerState`.

---

### Open 2 — View Key Offset (`VIEW_KEY_ACCOUNT_OFFSET = 100_000`) Is a Convention, Not Cryptographic Isolation

**Description:** The BIP-44 path separation between spend and view keys uses a numeric offset (`100_000`). The documentation correctly notes this is a convention, not cryptographic enforcement. Any software with access to the master seed can derive both spend and view keys.

**What prevents proof of security:** Wallet privacy depends on all software in the ecosystem respecting the view-key-read-only policy. There is no technical mechanism preventing a compromised wallet from using the view key to spend.

**Owner:** Wallet UX / protocol design — document this as a social convention, not a cryptographic guarantee.

---

### Open 3 — `Nonce` for ChaCha20-Poly1305 Is Truncated to 12 Bytes From 32-Byte Derivation

**Component:** `core/stealth/facade_kdf.rs` — `derive_nonce()`  
**Description:** The 32-byte `pack_nonce` output is truncated to 12 bytes for use as a ChaCha20 nonce:

```rust
let mut nonce12 = [0u8; 12];
nonce12.copy_from_slice(&full[..12]);
```

**What is unclear:** Whether this truncation is safe under the ChaCha20-Poly1305 specification (RFC 7539) for the specific way `s_out` is used as a key in the stealth output encryption. The ChaCha20 block counter provides the remaining bits. This needs confirmation against the STARK circuit's expected nonce format.

**Owner:** Protocol design — confirm the ChaCha20 nonce construction matches what the OWF circuit expects.

---

### Open 4 — `ReceiverKeys::reveal_receiver_secret` Exposes the Master Secret

**Component:** `core/key/stealth_keys.rs` — `ReceiverKeys`  
**Description:** `reveal_receiver_secret(&self) -> &ReceiverSecret` returns a reference to the hidden receiver secret. Callers who misuse this (e.g., log it, store it separately) bypass the `Hidden<T>` protection. There is no boundary enforcement.

**Impact:** This is an API design risk — the `Hidden` wrapper provides semantic protection but no enforcement against `reveal()` misuse.

**Owner:** Wallet service callers — must be audited for `reveal_receiver_secret` usage.

---

## 8. Concrete Fixes

### Fix 1: Replace `expect()` in Backup/Crypto Path (S1)

In `services/wallet_service.rs`, create a helper:

```rust
fn to_result<T, E: std::fmt::Display>(r: Result<T, E>, context: &str) -> WalletResult<T> {
    r.map_err(|e| WalletError::CryptoError(format!("{}: {}", context, e)))
}
```

Replace each `expect()` with `?` + proper error mapping.

### Fix 2: TTL Check on Cache Entries (S2)

Add to `cache_read()` or a new `cache_get()` method:

```rust
pub fn get_public_key(&self, path: &Bip44Path) -> Option<RistrettoPublicKey> {
    let cache = self.cache_read().ok()?;
    let cached = cache.get(path)?;
    let now = self.time_provider.unix_timestamp();
    if now - cached.cached_at > DERIVED_KEY_TTL_SECONDS {
        return None; // treat as cache miss
    }
    Some(cached.public_key)
}
```

### Fix 3: `checked_sub` for Gap Limit (S2)

Replace `saturating_sub` with `checked_sub` and propagate a logical error:

```rust
let gap = next_index.checked_sub(last_used_plus1)
    .ok_or(KeyManagerError::StateCorrupted)?; // invariant violated
```

### Fix 4: Add `ReceiverSecret::validate_usable()` (S2)

```rust
impl ReceiverSecret {
    pub fn validate_usable(&self) -> Result<(), StealthKeyError> {
        let view_sk = derive_view_secret_key(self)?;
        let _view_pk = derive_view_public_key(&view_sk)?;
        Ok(())
    }
}
```

Call at wallet creation and after `from_encrypted()` load.

### Fix 5: Entropy Result Type (S2)

```rust
pub enum EntropyValidation {
    Valid,
    Warnings(Vec<EntropyWarning>),
    Invalid(Vec<EntropyWarning>),
}
```

Return `EntropyValidation` from `validate_entropy()` and let callers decide policy.

---

## 9. Implementation Guidance

### Positive Observations

1. **`Hidden<T>` wrappers** are correctly used for `ReceiverSecret`, `Z00ZScalar`, and seed phrases throughout — no raw secret exposure in error paths
2. **`ConstantTimeEq`** is used for all secret comparisons (`ReceiverSecret`, `KeyManagerState`, `KeyManagerMetadata`)
3. **Domain separation** is comprehensive — 50+ `hash_domain!` entries covering KDF, signing, stealth, backup, encryption, and session contexts. No domain reuse detected
4. **`Zeroizing` wrappers** on transient secret keys ensure memory is cleared immediately after use
5. **BIP-44 gap limit** enforcement uses proper atomic compare-exchange with Acquire/Release ordering — no TOCTOU race
6. **Argon2id parameters** are validated against untrusted persisted metadata before use, with hard limits on CPU/memory/time
7. **`cipher_seed.rs` entropy validation** catches catastrophic seed patterns (all-zero, uniform, repeating) as hard errors
8. **`verify_index_mac()`** uses `ct_eq` for constant-time MAC comparison — no timing oracle
9. **`Bip39Seed64`** is `!Clone` (compile-time enforced) — seed cannot be accidentally duplicated
10. **`derive_schnorr_challenge`** correctly frames all inputs with `frame_bytes` before hashing, preventing concatenation ambiguity

### Areas Requiring Immediate Attention Before Production

1. All `expect()` calls in `services/wallet_service.rs:5453–5560` must be converted to `WalletResult` propagation
2. All `.unwrap()` in `chain_service.rs` must be replaced with `?`
3. Cache TTL enforcement must be implemented (currently dead code)
4. `ReceiverSecret::validate_usable()` should be added and called at all creation/load points
5. The transaction service (`tx_service.rs`) is entirely stubbed — production wallet needs full transaction construction, signing, and broadcast flow

---

## 10. Test Plan

### Positive Tests (should pass after fixes)

| Domain | Test Case |
|---|---|
| Key derivation | BIP-44 path round-trip: `m/44'/1337'/0'/0/0` → bytes → parse → same path |
| Key derivation | Spend/view key separation: verify different public keys for spend vs view path |
| Stealth keys | Receiver secret → view SK → view PK → identity point check rejects zero scalar |
| Entropy | Uniform seed (all 0x00) → `Err(UniformBytes)` |
| Entropy | 64-byte repeating pattern → `Err(HeuristicWarnings)` |
| Entropy | Valid random seed → `Ok(())` |
| Cache | Public key cached, retrieved, TTL expired → cache miss |
| Gap limit | Derive 20 + 1 addresses → `Err(GapLimitExceeded)` |
| Domain separation | Same input to two different domain hashers → different outputs |
| Schnorr | Sign message M with key at path P → verify → success |
| Schnorr | Sign with wrong message → verify fails |
| AEAD envelope | Encrypt + decrypt round-trip with XChaCha20-Poly1305 |
| Argon2id | Untrusted params with ops_limit=1_000_000 → `Err(InvalidParameters)` |

### Negative Tests (should fail/panic on current code)

| Domain | Test Case | Current Behavior |
|---|---|---|
| Crypto path | Serialize fails → `expect()` panics | Panics (S1) |
| Chain service | Network call fails → `.unwrap()` panics | Panics (S1) |
| Cache | TTL expired → still returns cached key | Wrong (S2) |
| Receiver secret | Identity-point-producing bytes → accepted at creation | Late error (S2) |
| Gap limit | Race causes `last_used > next` → `saturating_sub` → 0 | Silent (S2) |

### Fuzz Targets

- `Bip44Path::from_str` — malformed path strings
- `CipherSeedContainer::from_encrypted` — attacker-controlled encrypted blob
- `AeadEnvelope::decrypt` — tampered envelope bytes
- `ReceiverSecret::from_encrypted` — invalid version byte, wrong password
- `KdfParams::validate_untrusted_persisted` — pathological memory/CPU parameters

---

## 11. Confidence Assessment

| Claim | Confidence | Evidence That Would Change It |
|---|---|---|
| No raw secret in error messages | High | Grep for `format!` on secret types |
| Domain separation is complete | High | Two identical domain strings found |
| BIP-44 derivation is BIP-32 compliant | High | Code review of `bip32.rs` CKD function |
| Argon2id params are OWASP-aligned | High | OWASP table updated; retest with new params |
| Cache spot-check is effective | Medium | TTL field is written but never read for eviction |
| View key convention is documented | High | Code review of `VIEW_KEY_ACCOUNT_OFFSET` usage |
| ChaCha20 nonce truncation is safe | Medium | RFC 7539 review + OWF circuit confirmation needed |
| No forward secrecy | High (by design) | Document confirms no ephemeral key rotation |
|entropy validation catches catastrophic seeds | High | Unit tests for all `EntropyError` variants |

---

## 12. Final Decision

**`Blocked: S1 findings must be resolved before production deployment.`**

The S1 findings (`.unwrap()`/`.expect()` in crypto and chain-service paths) can cause silent process death and data loss in a wallet holding real funds. The fixes are straightforward (replace with `?` + proper error types). Once those are resolved, the crate is in a **safe enough** state for Phase 2 development with the documented S2/S3 findings as backlog items.

The architectural foundation is **solid**: comprehensive domain separation, correct use of `Hidden`/`Zeroizing`, constant-time comparisons, BIP-44 with gap limits, Argon2id with DoS guards, and entropy validation are all correctly implemented. The primary risk is incomplete error handling in the service layer, not in the cryptographic core.

**Recommended order:**
1. Fix S1: replace all `expect()`/`unwrap()` in `wallet_service.rs` and `chain_service.rs`
2. Fix S2 items 1-3 (TTL, gap `checked_sub`, `ReceiverSecret::validate_usable`)
3. Complete `tx_service.rs` stub with real transaction construction
4. Re-run full verification gate
5. Proceed to integration testing with testnet
