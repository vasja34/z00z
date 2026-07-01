# Key Derivation Call Graphs

This note compares the two code-backed derivation paths that matter most in the
current crate:

- the live RPC path behind `wallet.key.derive_receiver`
- the direct key-derivation path used by seed-centric tests

It is explanation-only and should stay aligned with current code.

---

## 🎯 Scope

- "RPC path" means the in-process JSON-RPC flow exercised by
  `crates/z00z_wallets/tests/test_rpc_key_derive_e2e.rs`.
- "Direct path" means seed-to-key derivation without wallet sessions or RPC.
- "Canonical path" means a `Bip44Path` string such as
  `m/44'/1337'/0'/0/0`.

---

## 🧪 RPC path

The current high-level RPC call graph is:

1. test transport calls `wallet.key.derive_receiver`
2. `rpc/key_rpc_server_derive.rs::derive_receiver_impl()` parses the path and
   resolves the wallet id from the session
3. `WalletService::verify_session(...)` validates session state
4. `WalletService::key_derive_rate_limit_precheck(...)` enforces the current
   derive limit
5. `WalletService::derive_public_key_for_path(...)` enters the service
   derivation path
6. `WalletService::get_create_wallet_receiver_deriver(...)` resolves or creates
   the per-wallet receiver-deriver handle
7. `WalletService::create_receiver_deriver_state(...)` initializes
   `KeyManagerImpl` from the wallet seed when needed
8. `ReceiverManagerImpl::derive_spend_key(...)` requests the public key for the
   path
9. `KeyManagerImpl::derive_key(...)` derives the public key through the key
   stack
10. `Bip44KeyManager::derive_address_key_for_path(...)` derives the leaf BIP-32
    private key
11. `RistrettoBridge::to_ristretto_key(...)` maps leaf material into the Z00Z
    Ristretto domain
12. the RPC layer returns
    `RuntimeDeriveReceiverResponse { public_key, path }`

The important output rule is unchanged: the RPC path returns public material and
the canonical path string, not a long-lived secret key.

---

## 🧪 Direct path

The direct path is shorter and skips wallet/session machinery:

1. parse mnemonic and derive the BIP-39 seed
2. build `Bip39Seed64`
3. create `Bip44KeyManager::new(seed, 1337, chain)`
4. derive a canonical path leaf
5. map the leaf into a Ristretto key

Current code-backed anchors for this path live in:

- `crates/z00z_wallets/tests/test_bip44.rs`
- `crates/z00z_wallets/src/key/test_bip44_manager_suite.rs`
- `crates/z00z_wallets/src/key/test_bip44_manager_entropy_suite.rs`

---

## 🔁 Shared invariants

Both paths converge on the same cryptographic rules:

- BIP-39 seed material is the root input
- BIP-44 path policy is enforced before derivation
- chain-separated Ristretto mapping happens after leaf derivation
- the same seed plus path stays deterministic
- different chains produce different outputs

The difference is system responsibility, not cryptographic core:

- the RPC path adds session validation and rate limiting
- the direct path is useful for unit/integration tests and low-level reasoning

---

## 📚 Canonical file pointers

- `crates/z00z_wallets/tests/test_rpc_key_derive_e2e.rs`
- `crates/z00z_wallets/tests/test_bip44.rs`
- `crates/z00z_wallets/src/rpc/key_rpc_server_derive.rs`
- `crates/z00z_wallets/src/services/wallet_session_derivation.rs`
- `crates/z00z_wallets/src/receiver/receiver_manager_impl_trait_impl.rs`
- `crates/z00z_wallets/src/key/manager_core.rs`
- `crates/z00z_wallets/src/key/bip32.rs`
- `crates/z00z_wallets/src/domains/hashing.rs`
- `crates/z00z_wallets/src/tx/signer.rs`
