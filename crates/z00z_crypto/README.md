# z00z_crypto

Cryptographic primitives and integrations for Z00Z.

This crate exposes secure hashing, key derivation, commitments, range proofs, and helper abstractions used by workspace crates.

## Public Surface

- Stable root `z00z_crypto`: the only approved facade for workspace code, including Tari-backed factories, proof services, signatures, hashes, KDFs, AEAD entrypoints, wrappers, and canonical domain modules.
- Expert lane `z00z_crypto::expert`: advanced helper traits, encoding utilities, and specialist key helpers that remain part of the facade.

Do not import the internal vendor subpath from workspace code. If a Tari-backed contract is part of the supported API, consume it from the root `z00z_crypto` facade.

## Test-Only AEAD

Caller-supplied nonce helpers are intentionally non-production. Use `z00z_crypto::aead::test_only::seal_with_nonce_TEST_ONLY` only from `cfg(test)`, `test-params-fast`, or `test-utils` paths.

## Cryptographic Safety Checklist

- Constant-time checks use `subtle::ConstantTimeEq` where required (AEAD/auth tags, ownership checks).
- Zero rejection is enforced for security-critical scalars and blindings.
- Identity-point rejection is enforced for untrusted ECC inputs.
- Domain separation is mandatory across KDF/hash operations.
- AEAD nonces are generated via secure RNG in canonical envelope flows.
- Production crypto paths avoid `unwrap()` and use typed `Result<T, E>` errors.
- Sensitive buffers use zeroization-capable wrappers and avoid debug leakage.

## Verification Commands

```bash
cargo check --package z00z_crypto --release --features test-params-fast
cargo clippy --package z00z_crypto --release --features test-params-fast -- -D warnings
cargo clippy --package z00z_crypto --release --features test-params-fast --target wasm32-unknown-unknown -- -D warnings
cargo test --package z00z_crypto --release --features test-params-fast
```

## Security Policy

Responsible disclosure policy is documented in [security.md](security.md).
