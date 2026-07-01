# Z00Z Utils

## 🎯 Boundary

`z00z_utils` is the repository cross-cutting policy crate.
It hosts infrastructure primitives that must be shared across crates to keep one audited implementation of generic concerns such as codecs, file I/O, logging, metrics, time, RNG, and best-effort OS hardening.

It is not a product-domain crate, a fallback staging area, or a place to hide ownership uncertainty.
If a capability carries wallet, storage, network, runtime, or simulator semantics, the owning crate must keep that behavior and depend on `z00z_utils` only for the narrow primitive it needs.

## 📌 Admission Policy

Add code to `z00z_utils` only when all of the following stay true:

- the concern is genuinely cross-cutting across multiple crates;
- the implementation is infrastructure-level rather than product-domain behavior;
- centralizing the code reduces security, correctness, or maintenance drift;
- the interface can stay generic without importing wallet, storage, simulator, or transport policy.

Do not add code to `z00z_utils` when it would:

- encode business rules for wallets, storage, claims, transport, or scenario flows;
- accumulate crate-specific compatibility shims as a convenience dumping ground;
- hide ownership problems that should instead be solved by narrowing a public facade in the owning crate;
- require domain-specific configuration, file naming, serialization shape, or rollback semantics that belong elsewhere.

## 🔑 Why These Modules Are Admitted

### JSON Compatibility

JSON support belongs here because encoding and decoding are shared infrastructure concerns.
`z00z_utils::codec` owns generic codec abstractions and the narrow compatibility surface for JSON values and helpers.

It does not own domain wire contracts.
If a wallet, storage, or RPC flow needs a special JSON migration rule, that rule must stay with the owning crate and consume the codec boundary rather than widening `z00z_utils` into a domain migration registry.

### Compression

Compression stays here only for generic bounded compression and decompression primitives.
The crate may provide Zstd or LZ4 helpers because those are reusable infrastructure building blocks with shared DoS and size-bound policy.

It does not own backup formats, checkpoint containers, or wallet-specific persistence semantics.
Those formats must remain in their owning crates and call the shared bounded primitives from here.

### OS Hardening

`os_hardening` stays here because process-level hardening and memory-lock helpers are repository-wide safety primitives.
Keeping one audited implementation avoids per-crate drift around `mlock`, `prctl`, or equivalent best-effort behavior.

It does not own secret lifecycle policy for a specific product flow.
Wallets, storage, and other crates decide when hardening is required; `z00z_utils` only provides the reusable primitive.

## 🚫 What Stays Outside

The following concerns must stay out of `z00z_utils`:

- wallet session rules, key-derivation policy, backup semantics, or RPC transport behavior;
- storage checkpoint semantics, proof binding, artifact retention, or database-specific compatibility policy;
- simulator artifact contracts, test harness rules, or output-sandbox ownership;
- network retry, peer identity, streaming, or overlay lifecycle logic;
- runtime or application orchestration that happens to call multiple infrastructure helpers.

## 🧪 Evaluation Rule For Future Additions

Before adding a new module, answer these questions explicitly:

1. Is the concern infrastructure-level and reused by more than one owning crate?
2. Would duplication create security or correctness drift if each crate implemented it separately?
3. Can the API remain generic without importing domain-specific policy types?
4. Is there a clearer owner crate that should keep the behavior instead?

If any answer points to domain ownership, the code does not belong in `z00z_utils`.

## ✅ Working Rule

Treat `z00z_utils` as the place for shared primitives, not shared product behavior.
When in doubt, keep ownership with the domain crate and depend on the smallest reusable utility surface from here.
