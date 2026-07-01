# Spend Proof Spec

## Final Authority Reset

`040-09-SUMMARY.md` remains the last historical implementation baseline for
Phase 040. Starting with `040-10-PLAN.md`, this specification is no longer the
authority for an honest partial closeout. It is the normative authority for the
final canonical theorem target: one suite `regular_spend_theorem_bpplus`, one
theorem contract `T(S, W)`, one canonical proof carrier, explicit membership
composition against `prev_root`, and one checkpoint/rollup closure path.
Historical baseline facts belong only in archived summary and report artifacts.

## 2.0 Document Authority

This document is the canonical Phase 040 design and requirement source for the
regular spend-proof upgrade.

Authority rules:

- this file is the only normative spend-proof design document for Phase 040;
- `040-TODO.md` is the execution backlog derived from this spec;
- older spend-proof drafts are historical inputs only and are not required to
  execute Phase 040 once their surviving requirements have been migrated here;
- the missing-code and decision points that were previously tracked separately
  are now incorporated directly into Parts 2 and 4 of this document.

Retirement decision:

- the legacy `7-Spend-Proof.md` draft is no longer required as an active
  requirement source once this document and `040-TODO.md` are kept current;
- if historical context is still desired, it should be treated as archive-only
  material and not as a second normative spec.

### 2.1 Scope

This specification defines the final canonical spend-proof target for Phase
040. It freezes one theorem suite, one theorem contract, one witness-source
table, and one end-to-end proof chain across wallet production, public
verification, checkpoint/state transition, rollup settlement, and runtime/e2e
closure. Historical implementation checkpoints remain relevant only as archived
baseline evidence; they are not the normative target of this document.

Implemented source-of-truth modules:

- `crates/z00z_core/src/assets/leaf.rs`
- `crates/z00z_core/src/assets/wire.rs`
- `crates/z00z_core/src/assets/commitment.rs`
- `crates/z00z_crypto/src/domains.rs`
- `crates/z00z_crypto/src/ecdh.rs`
- `crates/z00z_crypto/src/kdf.rs`
- `crates/z00z_wallets/src/core/address/leaf_scan.rs`
- `crates/z00z_wallets/src/core/address/stealth_scan_support.rs`
- `crates/z00z_wallets/src/core/stealth/output.rs`
- `crates/z00z_wallets/src/core/stealth/output_build.rs`
- `crates/z00z_wallets/src/core/stealth/tag.rs`
- `crates/z00z_wallets/src/core/tx/mod.rs`
- `crates/z00z_wallets/src/core/tx/spending.rs`
- `crates/z00z_wallets/src/core/tx/witness_gate.rs`
- `crates/z00z_wallets/src/core/tx/tx_verifier.rs`
- `crates/z00z_wallets/src/core/tx/state_update.rs`
- `crates/z00z_simulator/src/scenario_1/stage_4.rs`

> [!NOTE]
> The normative priority is: code first, tests and typed contracts second,
> old markdown third.

### 2.2 Status Classification

| Topic | Status | Meaning |
| --- | --- | --- |
| `AssetLeaf` wire contract | Implemented | Consensus-relevant leaf fields and asset-pack layout exist today. |
| Receiver-side ownership scan | Implemented | `receiver_scan_leaf()` and scanner adapters are live. |
| Full confidential output building | Implemented | `build_stealth_leaf()`, `build_stealth_sender_leaf()`, `build_stealth_bundle()` exist today. |
| Regular `TxPackage` wire format | Implemented | `TxInputWire`, `TxOutputWire`, `TxWire`, `TxPackage` are live. |
| Regular package verification | Implemented | `TxVerifierImpl` checks structure, digest, signatures, range proofs, fee/output invariants. |
| Spend witness gate | Implemented | `verify_spend_witness_gate()` and `build_spend_assets()` are live. |
| Pre-state resolution and checkpoint apply | Implemented | `prepare_tx_sum()` and `apply_batch_checkpoint()` are live. |
| Non-empty `TxProofWire` for regular tx | Implemented | Current `TxProofWire` carries optional `spend` proof/auth wires for accepted regular spend paths. |
| ==Concrete regular tx STARK prover/verifier== | Proposed only | No implementation exists today. |
| Concrete `CheckpointProof` object for regular tx flow | Proposed only | Current path uses typed checkpoint state update and trait hooks. |

### 2.3 Security Goals and Threat Model

#### 2.3.1 Security goals

- Ownership binding: a spender must only pass the spend gate when it can derive
  the correct owner handle, view key, `k_in`, owner tag, and `asset_id` for the
  consumed leaf secrets.
- Output correctness: receiver-owned leaves must decrypt under the intended
  stealth derivation path, with authenticated associated data and commitment
  opening consistency.
- Canonical parsing: asset-pack payloads and compact input references must be
  deterministic and reject malformed values.
- Fee and package integrity: the package digest, output nonce rules, asset-class
  checks, and fee-as-output accounting must be consistent with the live verifier.
- State-transition safety: membership witness, leaf-match, no-overlap, spent
  interval, and duplicate detection must hold before checkpoint application.

#### 2.3.2 Adversaries

- Malicious package producer sending malformed `TxPackage` bytes.
- Malicious sender trying to create outputs that decrypt incorrectly, mismatch
  commitments, or bypass range proof checks.
- Replay or substitution attacker trying to reuse a digest, output nonce, or
  state key in the wrong context.
- Malicious pre-state resolver returning the wrong leaf, the wrong serial, or a
  mismatched membership witness.
- Chain observer or integration layer mixing wallet and consensus hash domains.

#### 2.3.3 Trust boundaries

- Wallet-local secret boundary: `ReceiverSecret`, view key material, `s_out`, and
  blinding values stay outside public package bytes.
- Package-local public boundary: `TxPackage` is a compact public envelope and
  now carries optional regular-spend proof and auth wires for accepted spend
  paths, while still delegating membership and checkpoint-final validity to
  later state-boundary checks.
- Pre-state boundary: membership and leaf-match live in `ResolvedInput` plus
  `MemberWit`, not in `TxInputWire`.
- Checkpoint boundary: state apply must consume the same canonical theorem
  artifact as wallet production and public verification; no second proof lane
  is allowed.
- Routing and helper boundary: `ReceiverCard`, `PaymentRequest`, and any future
  inbox helper remain sender-side or wallet-side admission surfaces and must
  not be restated as if they already produce a finished public spend theorem.

> [!CAUTION]
> This document freezes the final target, not the last completed baseline.
> Current implementation checkpoints remain archived in summary artifacts, while
> every current-authority section in this spec must describe only the required
> final theorem shape.
> [!IMPORTANT]
> The canonical closure target is one theorem path only. No semantic alias, no
> dormant compatibility carrier, and no checkpoint or rollup side-lane may
> survive in the final state.

### 2.4 Current End-to-End Architecture

The current regular transaction flow is split into five implemented layers.

1. Build confidential outputs.
2. Self-validate newly built outputs.
3. Build and verify the public `TxPackage` envelope.
4. Resolve consumed inputs into typed pre-state records and verify the spend
   witness gate.
5. Apply checkpoint state transition with membership and spent-interval checks.

This is the live shape used by the simulator Stage-4 flow.

```rust
let tx_wire = TxWire {
    tx_type: "regular_tx".to_string(),
    inputs: tx_inputs.clone(),
    outputs: tx_outputs.clone(),
    fee,
    nonce: 0,
    context: Default::default(),
    proof: Default::default(),
    auth: Default::default(),
};

let tx_digest_hex = build_tx_package_digest(
    "TxPackage",
    "regular_tx",
    1,
    chain_id,
    chain_type,
    chain_name,
    &tx_wire,
)?;

verify_spend_witness_gate(chain_id, sender_recv_sec, &selected, &outputs, prev_root)?;
verify_tx_package(&tx_bytes)?;
```

> [!IMPORTANT]
> The current `TxPackage` verifier is a local wire verifier. It is not a full
> replacement for resolved-input state validation.

### 2.5 Normative Data Structures

#### 2.5.1 Public leaf contract

`AssetLeaf` is the canonical public leaf used by wallet scan and spend-related
state handling.

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

Invariants already enforced or assumed by current code:

- `asset_id` is the public leaf identifier used in storage and package overlap
  checks.
- `serial_id` is part of `leaf_ad` and part of pre-state leaf-match.
- `r_pub`, `owner_tag`, `c_amount`, and `enc_pack` are bound together during
  decrypt and self-check flows.
- `range_proof` is verified for `Coin` and `Token` outputs and optionally for
  `Nft` and `Void` outputs when present.

#### 2.5.2 Asset-pack plaintext

`AssetPackPlain` is consensus-critical and has a fixed 72-byte layout.

```rust
pub struct AssetPackPlain {
    pub value: u64,
    pub blinding: [u8; 32],
    pub s_out: [u8; 32],
}
```

Canonical encoding:

- bytes `[0..8]`: `value.to_le_bytes()`
- bytes `[8..40]`: `blinding`
- bytes `[40..72]`: `s_out`

> [!IMPORTANT]
> `AssetPackPlain` parsing is length-sensitive and blinding-sensitive. A spec or
> implementation must not replace it with a flexible encoding.

#### 2.5.3 Public regular tx package

The current regular package is compact and intentionally does not inline
resolved-input leaves.

```rust
pub struct TxInputWire {
    pub asset_id_hex: String,
    pub serial_id: u32,
}

pub struct TxOutputWire {
    pub role: TxOutRole,
    pub asset_wire: AssetPkgWire,
}

pub struct TxProofWire {
  pub spend: Option<SpendProofWire>,
}
pub struct TxAuthWire {
  pub spend: Option<SpendAuthWire>,
}

pub struct TxWire {
    pub tx_type: String,
    pub inputs: Vec<TxInputWire>,
    pub outputs: Vec<TxOutputWire>,
    pub fee: u64,
    pub nonce: u64,
    pub context: TxContextWire,
    pub proof: TxProofWire,
    pub auth: TxAuthWire,
}

pub struct TxPackage {
    pub kind: String,
    pub package_type: String,
    pub version: u8,
    pub chain_id: u32,
    pub chain_type: String,
    pub chain_name: String,
    pub tx: TxWire,
    pub tx_digest_hex: String,
    pub status: String,
}
```

> [!WARNING]
> `TxProofWire` is no longer empty, but it still only carries the current
> optional spend proof payload. Any richer proof-bearing extension must still be
> treated as a versioned contract change with explicit migration semantics.

#### 2.5.4 Spend witness gate types

The implemented spend gate uses typed plans and witness vectors, not the
conceptual old `TxWitness` from the markdown draft.

```rust
pub struct SpendPlan {
    pub prev_root: CheckRoot,
    pub inputs: Vec<SpendInputRef>,
    pub leaf_sums: Vec<SpendInputLeaf>,
    pub outputs: Vec<AssetLeaf>,
}

pub struct SpendWitness {
    pub recv_sec: [u8; 32],
    pub s_in_vec: Vec<[u8; 32]>,
}
```

### 2.6 Implemented Cryptographic Bindings

#### 2.6.1 Receiver secret to owner/view binding

The current code already hard-locks the receiver secret derivations.

```rust
pub fn derive_view_sk(receiver_secret: &[u8; 32]) -> Result<Z00ZScalar, CryptoError> {
    hash_to_scalar_zk::<ViewKeyDomain>("", &[receiver_secret])
}

pub fn derive_owner_handle(receiver_secret: &[u8; 32]) -> [u8; 32] {
    hash_zk::<ReceiverIdDomain>("", &[receiver_secret])
}

pub fn compute_owner_tag(owner_handle: &[u8; 32], k_dh: &[u8; 32]) -> [u8; 32] {
    hash_zk::<OwnerTagDomain>("", &[owner_handle, k_dh])
}

pub fn derive_asset_id(s_out: &[u8; 32]) -> [u8; 32] {
    hash_zk::<AssetIdDomain>("", &[s_out])
}
```

Current spend-rule order is also explicit and tested:

1. `OwnerHandle`
2. `ViewKey`
3. `InputKdf`
4. `OwnerTag`
5. `AssetId`
6. `Balance`
7. `Range`

#### 2.6.2 ECDH and DH key derivation

`z00z_crypto::ecdh` is the canonical crypto owner for point-based stealth ECDH.

Implemented properties:

- zero scalar is rejected;
- identity point is rejected;
- sender and receiver derive the same DH point;
- `derive_dh_key()` hashes canonical point bytes under `KdhDomain`.

#### 2.6.3 Wallet-owned `leaf_ad` and `tag16`

Wallet runtime binds leaf fields using the wallet-owned helpers below.

```rust
pub fn compute_tag16(k_dh: &[u8; 32], leaf_ad: &[u8; 32]) -> u16 {
    let hash = hash_zk::<WalletTag16HashProdDomain>("Z00Z/TAG16", &[k_dh, leaf_ad]);
    u16::from_le_bytes([hash[0], hash[1]])
}

pub fn compute_leaf_ad(
    asset_id: &[u8; 32],
    serial_id: u32,
    r_pub: &[u8; 32],
    owner_tag: &[u8; 32],
    c_amount: &[u8; 32],
) -> [u8; 32] {
    let serial = serial_id.to_le_bytes();
    hash_zk::<WalletLeafAdHashProdDomain>(
        "Z00Z/LEAFAD",
        &[asset_id, &serial, r_pub, owner_tag, c_amount],
    )
}
```

> [!CAUTION]
> This spec treats wallet `compute_leaf_ad()` and `compute_tag16()` as the
> normative runtime binding for the current receive/output flow. They must not be
> silently replaced with `z00z_crypto::derive_leaf_ad()` in the same path without
> a deliberate migration.

#### 2.6.4 Output construction binding

Full confidential outputs are currently built by the `build_stealth_*` helpers
and `bind_stealth_output_wire()` under the stealth-owned sender surface.

Binding order:

1. build commitment `c_amount` from `(value, blinding)`;
2. compute `owner_tag` from `(owner_handle, k_dh)`;
3. derive `asset_id` from `s_out`;
4. compute wallet `leaf_ad` over `(asset_id, serial_id, r_pub, owner_tag, c_amount)`;
5. encrypt `AssetPackPlain` with `ZkPack::encrypt()` using `(k_dh, leaf_ad, r_pub, asset_id, serial_id)`;
6. derive `tag16` from `(k_dh, leaf_ad)`;
7. generate a Bulletproofs+ range proof.

#### 2.6.5 Receiver-side detection binding

`receiver_scan_leaf()` delegates to `scan_owned()` and enforces the following
order on candidate owned leaves:

1. validate pack version gate;
2. decode and validate `r_pub`;
3. derive DH and `k_dh` from receiver view secret;
4. compare `owner_tag` in constant time;
5. verify `tag16` when present;
6. decrypt `enc_pack` under `leaf_ad`;
7. parse `AssetPackPlain`;
8. verify commitment opening against `c_amount`.

This is the live receive-flow cryptography boundary.

### 2.7 Verification Boundaries

#### 2.7.1 Output self-check boundary

`verify_self_decrypt()` is the sender-side sanity gate for freshly constructed
outputs. It verifies:

- `tag16` recomputation;
- self-decrypt under the current `leaf_ad`;
- plaintext/value consistency;
- `s_out` consistency;
- commitment opening;
- Bulletproofs+ range proof.

This check is in-memory only, but it is part of the real Stage-4 flow.

#### 2.7.2 Regular package verifier boundary

`TxVerifierImpl` verifies public package invariants only.

Current mandatory checks:

- package kind, type, version, chain metadata, and `tx_type`;
- `tx_digest_hex` exact recomputation with `build_tx_package_digest()`;
- asset decode for every output wire;
- output signature verification when an owner signature is present;
- output range proof verification according to asset class;
- input key uniqueness and shape (`asset_id_hex` must be 32-byte hex);
- output state-key uniqueness and no input/output overlap;
- non-zero and unique output nonces;
- asset-class amount rules (`Coin`/`Token` positive, `Nft`/`Void` zero);
- fee presence, fee-output sum, and fee estimator match.

> [!IMPORTANT]
> `TxVerifierImpl::verify_balance()` explicitly does not claim full resolved-input
> value conservation. That is deferred to a tx-proof layer or resolved pre-state
> validation path.

#### 2.7.3 Spend witness gate boundary

`verify_spend_witness_gate()` is the implemented spend-proof surrogate for the
current Stage-4 flow.

It does the following:

1. decrypt each selected input leaf with the provided receiver secret;
2. recover `s_in` from the input pack;
3. derive canonical `asset_id` from `s_in`;
4. build `SpendPlan` and `SpendWitness`;
5. call `build_spend_assets()`;
6. inside `build_spend_assets()`, enforce duplicate-input rejection,
   non-zero witness fields, leaf field presence, spend-rule verification, and
   non-empty balance vectors;
7. bind `prev_root` through the current `SpendProofApi` hook.

This boundary proves that the spender can derive the consumed leaf secret and
match the ownership equations implemented today.

It does not prove that the current public-only carrier can recover those
witness-only relations by itself. Under the current privacy boundary, theorem
closure for validator-facing verification means recomputing the canonical
public statement from public tx facts and verifying a theorem-carrying proof
that attests to the witness-only relations already enforced by
`verify_spend_rules(...)`.

#### 2.7.4 Pre-state and checkpoint boundary

`prepare_tx_sum()` and `apply_batch_checkpoint()` are the current state
transition authority for spend-related validation.

`prepare_tx_sum()`:

- parses compact input refs from `TxInputWire`;
- uses `InputResolver` to load a `ResolvedInput`;
- enforces `asset_id` and `serial_id` leaf-match;
- validates membership witness against `prev_root`.

`apply_batch_checkpoint()`:

- requires batch non-empty;
- requires exact `prev_root` equality with the state root;
- requires non-empty inputs and outputs;
- rejects duplicate inputs and duplicate output asset ids;
- rejects malformed resolved input or membership-witness material;
- calls the pluggable `TxProofVerifier` hook;
- rejects spent-after-interval and spent-in-batch reuse;
- deletes input leaves and inserts output leaves;
- records `spent_delta` and `created_delta`.

> [!NOTE]
> In the current codebase, membership and no-double-spend are typed state-update
> checks, not a concrete standalone `CheckpointProof` artifact.

### 2.8 Canonical Serialization and Hash Rules

#### 2.8.1 Asset-pack serialization

`AssetPackPlain` serialization is fixed-width, little-endian, and must not be
changed without a versioned migration.

#### 2.8.2 Package digest

The stored package digest is:

```rust
pub fn build_tx_package_digest(
    kind: &str,
    package_type: &str,
    version: u8,
    chain_id: u32,
    chain_type: &str,
    chain_name: &str,
    tx: &TxWire,
) -> Result<String, String>
```

Behavior:

- serialize `TxWire` with `JsonCodec`;
- hash a BLAKE3 transcript prefixed by `b"z00z.tx.pkg.digest.v2."`;
- bind envelope metadata plus serialized `TxWire`;
- return lowercase hex.

#### 2.8.3 Wire digest helper

`compute_tx_digest_from_wire()` is a separate helper that:

- serializes `TxWire` with `JsonCodec`;
- frames the bytes;
- hashes with `TxDigestDomain` and label `"Z00Z/TXPKG_WIRE"`;
- returns `[u8; 32]`.

Current observed role in the codebase:

- exposed through the `core::tx` facade as a tx-level bridge helper;
- called by simulator Stage-4 as a local helper before `TxPackage` assembly;
- not stored in `TxPackage`;
- not recomputed by `TxVerifierImpl`.

#### 2.8.4 Normative digest decision

The normative transaction binding root for future non-empty `TxProofWire` is:

- `build_tx_package_digest()` for any public, persisted, verifier-checked, or
  replay-sensitive transaction identifier;
- `compute_tx_digest_from_wire()` only as an optional internal sub-statement
  helper inside a proof transcript.

Normative rules:

1. A regular transaction proof that is intended to validate the same object as
   `TxPackage` SHALL bind `build_tx_package_digest()` or SHALL bind an
   equivalent transcript that includes all of its fields:
   `kind`, `package_type`, `version`, `chain_id`, `chain_type`, `chain_name`,
   and canonical `TxWire` bytes.
2. `compute_tx_digest_from_wire()` SHALL NOT be used as the sole public binding
   root for `TxProofWire`, because it omits package-envelope metadata and is not
   the stored or verifier-checked digest in the current codebase.
3. If a future prover uses `compute_tx_digest_from_wire()` internally for
   circuit convenience, that wire digest SHALL be nested under the package
   digest or under a proof transcript that binds the exact envelope tuple above.
4. `tx_digest_hex` remains the canonical external package digest until a
   versioned migration says otherwise.

Security rationale:

- `TxVerifierImpl::verify_digest()` recomputes only
  `build_tx_package_digest()`;
- `TxPackage` persists only `tx_digest_hex` from that function;
- tests and benches construct packages with `build_tx_package_digest()`;
- `compute_tx_digest_from_wire()` is currently a Stage-4 helper over `TxWire`
  only.

> [!WARNING]
> Choosing the wire digest as the only future proof root would under-bind the
> package envelope and can create substitution or replay confusion between the
> object proved and the object actually persisted and verifier-checked.

### 2.9 Current Spend Invariants

The following invariants are already required by code and tests.

1. `AssetPackPlain` length is exactly 72 bytes.
2. `TxInputWire.asset_id_hex` is canonical 32-byte hex and does not inline the
   consumed leaf.
3. `serial_id` in `TxInputWire` must match the resolved pre-state leaf.
4. Output state keys must be unique and must not overlap consumed input ids.
5. Output nonces must be non-zero and unique inside one `TxPackage`.
6. `tx.fee` must equal the sum of `Fee` outputs and the fee estimator result.
7. `verify_spend_rules()` locks the order and domain labels of the spend-rule set.
8. `receiver_scan_leaf()` must reject tag drift, decrypt drift, parse failure,
   and commitment mismatch.
9. `apply_batch_checkpoint()` must reject `PrevRoot`, `LeafMatch`, `BadMember`,
   `SpentAfter`, `SpentBatch`, and duplicate output ids.

### 2.10 Failure Cases

Expected failure classes in the current flow:

- malformed or mixed package envelope: `TxVerifierError::InvalidStructure`;
- invalid output signature: `TxVerifierError::InvalidSignature`;
- invalid range proof: `TxVerifierError::InvalidRangeProof`;
- fee/output mismatch or duplicate keys: `TxVerifierError::VerificationFailed`;
- wrong receiver secret or undecryptable input pack: witness-gate error from
  `resolve_input_pack()` / `verify_spend_witness_gate()`;
- root mismatch, bad membership witness, or spent interval violation:
  `StateError::*` from `prepare_tx_sum()` / `apply_batch_checkpoint()`.

### 2.11 Frozen Closure Contract For Reopened Theorem Backend Gap

> [!TIP]
> The active closure target for reopened `040-09` is not to widen the current
> privacy boundary. It is to encode the already implemented spend-gate
> semantics into a versioned non-empty `TxProofWire`, then plug a
> theorem-carrying proof into `state_update::TxProofVerifier`.

Required constraints for a future regular-tx proof layer:

1. Bind exactly the current spend-rule equations already enforced by
   `verify_spend_rules()`.
2. Bind the same resolved input ids and public leaf fields used by
   `SpendInputLeaf`.
3. Bind the same output `AssetLeaf` fields already checked by
   `verify_self_decrypt()`.
4. Preserve the distinction between decryptable output recovery, wallet-local
    ownership classification, and public spend authorization; future proof text
    may only promote claims that are actually bound by explicit witnesses and
    public inputs.
5. Use `build_tx_package_digest()` as the proof-binding root for any public or
    persisted proof statement, and treat `compute_tx_digest_from_wire()` as an
    internal helper only when it is explicitly nested under the package digest.
6. Reuse the current `prev_root` typed checkpoint pipeline instead of inventing a
   parallel membership model.
7. Expose proof versioning through a non-empty `TxProofWire`, not through ad hoc
   JSON fields.

Authoritative closure rule for the reopened `040-09` gap:

- do not define closure as recovering the full `verify_spend_rules(...)`
  theorem from the current public-only carrier fields alone;
- define closure as `canonical statement recomputation + theorem proof
  verification` over the same witness-only relations already enforced by
  `verify_spend_rules(...)`;
- keep `verify_spend_rules(...)` as theorem authority and treat any backend as
  a bounded prove/verify seam for that theorem rather than as a rewrite of the
  theorem itself.

Canonical public-input inventory for the reopened theorem backend closure:

1. `build_tx_package_digest(...)` output as the only public and persisted root.
2. Typed `prev_root` from the existing checkpoint pipeline.
3. Explicit scope tuple: `chain_id`, `chain_type`, `chain_name`, and `tx_ver`.
4. Ordered `SpendInputRef` vector.
5. Ordered `SpendInputLeaf` public fields, including the state-key binding now
  carried by `input_asset_id_hex`.
6. Ordered output `AssetLeaf` public fields already rechecked by
  `verify_self_decrypt()`.
7. Fee-as-output balance data required by the live verifier boundary.
8. The regular-spend nullifier vector.
9. Explicit range-proof mode or commitment-compatibility inputs unless those
  checks are migrated into the backend with an explicit compatibility note.

Canonical private-witness inventory for final theorem closure:

1. `ReceiverSecret`.
2. Ordered `s_in[i]` for each consumed input.
3. The current owner/view chain enforced by `verify_spend_rules(...)`,
   including `view_sk`, `k_in`, `owner_handle`, `owner_tag`, and `asset_id`,
   remains theorem-internal derivation from `ReceiverSecret` and `s_in[i]`.

Frozen closure identifiers for the final Phase 040 target:

- canonical suite id: `regular_spend_theorem_bpplus`
- canonical artifact contract: one theorem carrier on the existing
  `TxProofWire` or `SpendProofArtifact` seam with no semantic aliases
- canonical theorem contract: `T(S, W)` over the canonical spend statement,
  witness authorization, receiver binding, authoritative membership against
  `prev_root`, output commitments plus range proofs, balance equation, and
  deterministic replay-safe nullifier semantics
- no live semantic aliases or version-suffixed proof branches may remain in
  current phase authorities, code, tests, or runtime artifacts

Frozen capability decision for the active plan:

- `040-10-PLAN.md` selects Option C final canonical theorem closure on the
  existing spend pipeline.
- Workspace-owned adapter expansion is allowed when needed, but vendor Tari
  code remains read-only.
- Membership against `prev_root` must move from external assumption to an
  explicit theorem sub-witness.
- Checkpoint/state-transition and rollup settlement must consume the same
  theorem boundary.

Explicitly prohibited shortcuts:

- Do not claim STARK proof support until `TxProofWire` is non-empty and wired to
  `TxProofVerifier`.
- Do not restate `sender cannot steal because receiver_secret is required` as a
  completed public-theorem claim until the proof carrier and checkpoint-facing
  verifier actually enforce that statement.
- Do not treat a compatibility envelope or digest echo as a sufficient
  theorem-carrying proof artifact.
- Do not introduce `receiver_cards` into the regular package just because the
  old markdown used them.
- Do not smuggle receiver cards, payment requests, or inbox-helper identifiers
  into the public regular package as a shortcut for proving routing or privacy.
- Do not replace fee-as-output semantics with a separate `C_fee` contract unless
  the live verifier and tests are migrated together.
- Do not mix wallet `compute_leaf_ad()` with crypto `derive_leaf_ad()` in the
  same runtime path without a migration plan.

### 2.12 Open Questions / Missing Code Support

The following points are intentionally marked non-factual because the current
codebase does not fully implement them.

| Item | Status | Why it is not stated as current fact |
| --- | --- | --- |
| Regular `TxProofWire` carrying current public spend proof bytes | Implemented in narrowed form | The wire now carries optional `spend` proof/auth fields for accepted public spend paths. |
| Concrete regular tx prover for `TxProofDomain` | Active `040-10` implementation target | `build_public_spend_contract(...)` remains the public statement source, and the producer must emit one canonical theorem artifact on the existing wallet or Stage-4 seam. |
| Concrete regular tx verifier that consumes current `TxProofWire` spend fields | Active `040-10` implementation target | `verify_tx_public_spend_contract()` and `verify_full_tx_package()` must converge on the same theorem artifact across wallet, public verifier, and checkpoint seams. |
| Frozen theorem backend suite for final closure | Authority frozen | The only allowed suite identifier is `regular_spend_theorem_bpplus`; semantic aliases and version-suffixed proof branches are forbidden in the final state. |
| Concrete regular `CheckpointProof` object | Prohibited | Current flow must upgrade the existing checkpoint pipeline instead of introducing a parallel checkpoint proof object. |
| Unified output constructor for lightweight stealth and full confidential leaf flows | Needs implementation | Public confidential sender construction is unified under `core::stealth`, but lightweight `build_tx_stealth_output()` still remains a separate header-oriented surface. |

## Part 3 - Mapping Table

| SPEC section | Related code module / type / function | Status | Notes |
| --- | --- | --- | --- |
| 2.4 Current End-to-End Architecture | `crates/z00z_simulator/src/scenario_1/stage_4.rs`, `TxPackage`, `verify_spend_witness_gate()` | Matches current code | Stage-4 orchestrates the current spend flow. |
| 2.5.1 Public leaf contract | `z00z_core::assets::AssetLeaf` | Matches current code | Public confidential leaf shape is implemented. |
| 2.5.2 Asset-pack plaintext | `z00z_core::assets::AssetPackPlain` | Matches current code | Fixed 72-byte layout and checked decode path. |
| 2.5.3 Public regular tx package | `TxInputWire`, `TxOutputWire`, `TxWire`, `TxPackage`, `TxProofWire` | Matches current code | `TxProofWire` and `TxAuthWire` now carry optional `spend` proof/auth fields for accepted spend paths. |
| 2.5.4 Spend witness gate types | `SpendPlan`, `SpendWitness`, `SpendInputRef`, `SpendInputLeaf` | Matches current code | These are the live spend-binding types. |
| 2.6.1 Receiver secret binding | `derive_view_sk()`, `derive_owner_handle()`, `compute_owner_tag()`, `derive_asset_id()` | Matches current code | Implemented in `z00z_crypto::kdf`. |
| 2.6.2 ECDH and DH key derivation | `z00z_crypto::ecdh::*` | Matches current code | Identity and zero-scalar checks are implemented. |
| 2.6.3 Wallet-owned `leaf_ad` / `tag16` | `z00z_wallets::core::stealth::tag::*` | Matches current code | Wallet runtime currently owns these bindings. |
| 2.6.4 Output construction binding | `build_stealth_leaf()`, `build_stealth_leaf_with_blind()`, `build_stealth_sender_leaf()`, `build_stealth_bundle()`, `bind_stealth_output_wire()` | Matches current code | Full confidential leaf and sender bundle builders now live under `core::stealth`; `core::tx` keeps only bridge-style compatibility utilities. |
| 2.6.5 Receiver-side detection binding | `receiver_scan_leaf()`, `scan_owned()` | Matches current code | Decrypt + commitment-open path exists today. |
| 2.7.2 Regular package verifier | `TxVerifierImpl` | Matches current code | Public wire checks and fee/output checks only. |
| 2.7.3 Spend witness gate | `verify_spend_witness_gate()`, `build_spend_assets()`, `verify_spend_rules()` | Matches current code | Current spend-proof surrogate. |
| 2.7.4 Pre-state and checkpoint boundary | `prepare_tx_sum()`, `apply_batch_checkpoint()`, `ResolvedInput`, `MemberWit` | Matches current code | Membership and state-apply path is live. |
| 2.8.2 Package digest | `build_tx_package_digest()` | Matches current code | Stored in `tx_digest_hex`. |
| 2.8.3 Wire digest helper | `compute_tx_digest_from_wire()` | Matches current code | Helper exists but is not the stored package digest. |
| 2.8.4 Normative digest decision | `TxVerifierImpl::verify_digest()`, `TxPackage.tx_digest_hex`, Stage-4 `core_tx_digest()` call site | Matches current code | Public proof binding must follow the persisted/verifier-checked package digest. |
| 2.11 Frozen theorem-backend closure contract | `TxProofDomain`, `TxProofWire`, `state_update::TxProofVerifier` | Active `040-10` authority | The planning target is explicit: one canonical theorem verifier must replace all semantic aliases and converge wallet, checkpoint, and rollup seams on the same contract. |
| 2.12 Unified proof-bearing checkpoint model | old `CheckpointProof` narrative only | Proposed only | No concrete regular-tx checkpoint proof object today. |

## 🎯 Part 4 - Mandatory Implementation Backlog

### 🚫 4.1 Legacy Requirement Migration Verdict

The old Phase 040 spend-proof document mixes live implementation facts,
partially delivered current-stack constraints, and stale conceptual structures.
Only the rows marked `Implement` below should drive new code.

| Legacy topic from the retired spend-proof draft | Current status in repo | Required action |
| --- | --- | --- |
| Ownership equation chain (`owner_handle`, `view_sk`, `k_in`, `owner_tag`, `asset_id`) | Already implemented as live rule kernel in `verify_spend_rules()`, `build_spend_assets()`, and the current public spend contract | Reuse as canonical theorem source. Do not redesign. |
| Current public spend proof or auth boundary for regular tx | Already implemented in narrowed form by `build_public_spend_contract()` and `verify_tx_public_spend_contract()` | Keep honest wording. Extend, do not replace. |
| `z00z_core::tx` module with conceptual `TxBuilder`, `SpendableAsset`, and standalone `TxProof` tree | Stale relative to current code ownership | Do not implement this shape. Use the existing `z00z_wallets::core::tx` split. |
| Full STARK circuit and gadget narrative in Section 7.3 | Proposed only | Do not claim as delivered. Implement only after a versioned non-empty proof carrier exists. |
| `receiver_cards` as regular on-chain tx payload | Stale | Do not add receiver cards to the persisted regular package. Keep them only in the current auth boundary or equivalent admission-only context. |
| Standalone regular-tx `CheckpointProof` object as the primary state-validation surface | Overstated / stale | Reuse `prepare_tx_sum()`, `apply_batch_checkpoint()`, and `TxProofVerifier` instead of inventing a parallel checkpoint artifact. |
| Unified output-construction path for all stealth and confidential output surfaces | Partially open | Keep as follow-up only. It is not the first blocker for Phase 040 proof closure. |

### ⚖️ 4.2 Architectural Decisions With Pros And Cons

#### 🔑 4.2.1 Proof carrier shape

**Recommended path:** extend the existing `TxProofWire` and `SpendProofWire`
surface into a non-empty, versioned regular-tx proof carrier.

**Pros:**

- keeps backward-compatible ownership of regular-tx proof material in the same
  wire boundary already persisted by Stage 4;
- reuses `build_public_spend_contract()`, `verify_tx_public_spend_contract()`,
  `TxProofVerifier`, and the current tx/package admission model;
- avoids reviving the stale conceptual `z00z_core::tx` module tree;
- keeps the file split aligned with the real `z00z_wallets::core::tx` facade.

**Cons:**

- requires careful versioning because the current `TxProofWire` only carries a
  narrowed optional spend payload rather than a full proof carrier;
- forces honest migration work at every call site that still treats tx proof as
  placeholder bytes.

**Rejected alternative:** create a new parallel proof object outside
`TxProofWire`.

**Why rejected:** it would duplicate the current public contract, widen the
admission surface, and drift away from the live tx wire.

#### 🔒 4.2.2 Checkpoint verification boundary

**Recommended path:** keep `verify_tx_public_spend_contract()` as the regular
package-admission boundary, use `verify_full_tx_package()` as the canonical
package-admission wrapper, and use `state_update::TxProofVerifier` for the
resolved-input and checkpoint-facing proof check.

**Pros:**

- preserves the current separation of concerns between package admission and
  state application;
- avoids bloating `TxPkgSum` with fields that are only needed at the
  package-authentication boundary;
- matches the current Stage 4 and Stage 7 split already exercised in simulator
  code.

**Cons:**

- proof-related failures may still surface at two distinct verification seams;
- future non-empty proof-carrier work still needs checkpoint-facing verifier
  upgrades beyond the current admission wrapper.

**Alternative:** expand `TxPkgSum` so checkpoint apply carries the full tx auth,
context, and package-admission statement.

**Why not first:** this is a larger refactor, increases duplication, and is not
required to close the current regular-spend gap.

#### 🧷 4.2.3 Nullifier semantics placement

**Recommended path:** add deterministic regular-spend nullifier derivation in a
dedicated spend-nullifier module, commit those nullifiers in the non-empty
proof carrier, and enforce replay/uniqueness at the checkpoint-state boundary.

**Pros:**

- closes the exact still-open element called out by `verify_spend_rules()` and
  the existing tests;
- keeps replay semantics tied to resolved inputs and state transition instead of
  pretending a local package verifier alone can close global uniqueness;
- gives one explicit seam for future proof binding, storage replay state, and
  simulator checks.

**Cons:**

- requires a new canonical derivation contract and storage-facing replay tests;
- needs careful chain or root scoping to avoid cross-context reuse.

**Alternative:** keep nullifier semantics only as a public-wire or local verifier
feature.

**Why rejected:** replay safety is global state, so it cannot be honestly closed
only inside the local package verifier.

#### 🏗️ 4.2.4 Proof producer placement

**Recommended path:** keep regular-tx proof production on the wallet or
Stage-4 admission side, and make it emit the same versioned proof carrier that
the checkpoint-facing verifier later consumes.

**Pros:**

- matches the current Stage-4 flow, where spend-related public proof material
  is already built before package persistence;
- preserves offline-friendly transaction construction instead of moving witness
  requirements into checkpoint execution;
- keeps secret witness material out of checkpoint-state code and out of any
  future storage-facing verifier boundary;
- aligns with the missing-code analysis already captured in Part 2, where the
  missing prover is treated as a regular-tx concern rather than a reason to
  introduce a separate checkpoint artifact.

**Cons:**

- requires a shared statement contract so producer and verifier cannot drift;
- introduces one more explicit module boundary to maintain.

**Rejected alternative:** make the concrete proof producer a checkpoint-only or
aggregator-only responsibility.

**Why rejected:** the current architecture and the old spend-proof intent both
assume local transaction validity can be prepared before checkpoint execution,
while checkpoint apply should consume proof evidence rather than reconstruct
private witness material.

### 🧩 4.3 Mandatory Phase 040 Implementation Tasks

#### Task 1. Introduce a versioned non-empty regular-spend proof carrier

**Objective:** replace the placeholder-only regular `TxProofWire` surface with a
non-empty versioned carrier that can hold proof bytes and the minimum public
binding material required by the checkpoint verifier.

**Mandatory steps:**

1. Extend the current `SpendProofWire` / `TxProofWire` contract instead of
    creating a second proof container.
2. Keep the proof carrier versioned and migration-aware.
3. Make the carrier explicit about which fields are public statement material,
    which field identifies the proof suite or parameters, and which fields are
    opaque proof payload.
4. Include a canonical public statement shape that binds chain or root scope,
    the canonical package digest, and deterministic commitments to the consumed
    input list and produced output list, or a documented equivalent that is no
    weaker.
5. Preserve the current Stage 4 persisted tx package format as the canonical
    entry point.

**Target files:**

- `crates/z00z_wallets/src/core/tx/tx_wire_types.rs`
- `crates/z00z_wallets/src/core/tx/spend_verification.rs`
- `crates/z00z_wallets/src/core/tx/mod.rs`

#### Task 2. Centralize canonical spend-proof statement encoding in one auditable home

**Objective:** freeze one auditable canonical statement home for the current
Phase 040 execution path. The approved baseline keeps that home inside
`spend_verification.rs` unless a later proof-backend split is explicitly
re-specified.

**Mandatory steps:**

1. Keep the canonical regular-spend proof statement builder centralized in one
  auditable implementation home for both producer and verifier paths.
2. Bind the statement to the same spend-rule theorem already enforced today,
    including canonical input refs, public leaf fields, output leaf fields, and
    package digest.
3. Preserve the currently live range-proof semantics: the future proof flow
    must either keep verifier-side output range-proof checks or migrate them into
    the proof system with an explicit compatibility decision.
4. Keep `compute_tx_digest_from_wire()` as an internal helper only when nested
    under the package digest.
5. Do not introduce ad hoc JSON fragments or unversioned proof transcripts.

**Recommended file split:**

- keep the canonical statement builder in
  `crates/z00z_wallets/src/core/tx/spend_verification.rs`
- add a bounded theorem prove/verify seam in
  `crates/z00z_wallets/src/core/tx/spend_proof_backend.rs`

#### Task 3. Implement a real regular-tx prover/verifier backend behind `TxProofVerifier`

**Objective:** make the wallet or Stage-4 flow emit a real regular-tx proof
carrier, and make checkpoint apply verify more than opaque `tx_proof` byte
equality.

**Mandatory steps:**

1. Introduce a dedicated regular-spend proof backend trait or implementation
    split that covers both proof production and proof verification over the same
    statement contract.
2. Add a concrete wallet or Stage-4 producer path that emits the non-empty
    versioned proof carrier from the canonical statement builder and the current
    spend witness material.
3. Verify the proof against resolved inputs, public output leaves, and the bound
    package digest.
4. Reject suite mismatch, statement mismatch, malformed proof payload, and any
    divergence between carried public inputs and recomputed transaction facts.
5. Do not attempt theorem closure by reconstructing witness-only secrets from
  the current public-only carrier; the backend must prove those relations
  directly while keeping the public carrier stable.
6. Wire that verifier through `state_update::apply_batch_checkpoint()` via the
    existing `TxProofVerifier` seam.
7. Keep the current package-coupled simulator verifier only as an interim or
    adapter path, not as the final proof backend.

**Target files:**

- new `crates/z00z_wallets/src/core/tx/spend_prover.rs`
- existing `crates/z00z_wallets/src/core/tx/spend_proof_backend.rs`
- `crates/z00z_wallets/src/core/tx/state_update.rs`
- `crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_lane_runtime_flow.rs`
- `crates/z00z_simulator/src/scenario_1/stage_6_utils/bundle_lane_impl.rs`

#### Task 4. Add explicit regular-spend nullifier semantics

**Objective:** close the exact still-open public-contract gap for `PH32-SPEND`.

**Mandatory steps:**

1. Define deterministic regular-spend nullifier derivation from consumed input
    witness data plus explicit chain or root scope.
2. Carry the committed nullifier vector in the non-empty proof or statement
    surface.
3. Enforce replay and uniqueness against checkpoint or storage state, not only
    against local package bytes.
4. Keep claim nullifier semantics and regular spend nullifier semantics
    explicitly separated.

**Recommended file split:**

- new `crates/z00z_wallets/src/core/tx/spend_nullifiers.rs`
- update `crates/z00z_wallets/src/core/tx/spend_rules.rs`
- update `crates/z00z_wallets/src/core/tx/state_update.rs`

#### Task 5. Add one canonical full regular-tx verification entry point

**Objective:** prevent callers from running the local package verifier while
silently skipping the public spend contract.

**Mandatory steps:**

1. Add a facade entry point that composes package structure, digest,
    signature/range checks, and `verify_tx_public_spend_contract()`.
2. Keep `TxVerifierImpl` honest about what it proves if it remains local-only.
3. Use the new full verifier in simulator and any future publish or admission
    path.
4. Do not overstate `verify_balance()` as resolved-input value conservation.

**Target files:**

- `crates/z00z_wallets/src/core/tx/tx_verifier.rs`
- `crates/z00z_wallets/src/core/tx/mod.rs`
- `crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_lane_runtime_flow.rs`

#### Task 6. Keep output-constructor unification as a bounded follow-up

**Objective:** reduce duplicated output-construction surfaces only after the
proof carrier and verifier seams are stable.

**Mandatory steps:**

1. Do not block proof closure on total output-builder unification.
2. After the proof carrier lands, decide whether
    `build_tx_stealth_output()` should delegate to the confidential leaf builder
    path or remain a specialized helper.
3. Preserve exact `leaf_ad`, `tag16`, commitment, and range-proof semantics
    during any unification.

**Target files:**

- `crates/z00z_wallets/src/core/stealth/output.rs`
- `crates/z00z_wallets/src/core/stealth/output_build.rs`
- `crates/z00z_wallets/src/core/tx/mod.rs`

### 🧪 4.4 Mandatory Test Surfaces (`create-tests` aligned)

The following tests are required when the above tasks are implemented.

| Test objective | Test file target |
| --- | --- |
| prove non-empty proof-wire encode/decode and version rejection | new `crates/z00z_wallets/tests/test_spend_proof_wire.rs` |
| prove canonical spend statement binding to package digest, inputs, and outputs | new `crates/z00z_wallets/tests/test_spend_statement.rs` |
| prove proof-carrier suite id and public-statement mismatch are fail-closed | extend `crates/z00z_wallets/tests/test_tx_proof_verifier.rs` |
| prove Stage-4 or wallet proof producer emits a carrier that the checkpoint verifier re-accepts without statement drift | new `crates/z00z_wallets/tests/test_spend_prover_contract.rs` |
| prove regular nullifier derivation, uniqueness, and chain or root scoping | new `crates/z00z_wallets/tests/test_spend_nullifier_semantics.rs` |
| prove `TxProofVerifier` rejects malformed proof bytes, wrong resolved input, wrong root, and mismatched outputs | new `crates/z00z_wallets/tests/test_tx_proof_verifier.rs` |
| prove range-proof path stays explicit during proof-backend integration, whether kept verifier-side or migrated in-proof | extend `crates/z00z_simulator/tests/test_scenario1_spend_gate.rs` |
| prove the full regular package verifier rejects packages that pass local structure checks but fail the public spend contract | extend `crates/z00z_simulator/tests/test_scenario1_spend_gate.rs` |
| prove Stage 4 to Stage 6 roundtrip keeps proof carrier, statement binding, and checkpoint admission coherent | new `crates/z00z_simulator/tests/test_scenario1_tx_proof_roundtrip.rs` |
| keep honest wording about current public spend scope and unresolved nullifier closure | extend `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs` |

### ✅ 4.5 Phase 040 Completion Rule

Phase 040 is not complete until all of the following are true:

1. the regular tx path uses a non-empty, versioned proof carrier;
2. the proof carrier is bound to the canonical package digest and current
    spend-rule theorem;
3. wallet or Stage-4 code emits proof evidence that the checkpoint path can
    consume without private-witness reconstruction;
4. checkpoint apply verifies real regular-tx proof semantics through
    `TxProofVerifier` rather than placeholder byte equality;
5. regular-spend nullifier semantics are defined and replay-safe;
6. the simulator and wallet tests prove the honest current boundary without
    overstating full proof closure.
