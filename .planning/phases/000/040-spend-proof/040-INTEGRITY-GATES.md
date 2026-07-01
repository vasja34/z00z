# 040 Integrity Gates

## 🎯 Purpose

This ledger records the executable integrity gates for the active `040-10`
internal theorem-relation closure sweep.

The ledger is limited to three internal closure claims:

1. The canonical producer and verifier path preserves the exact
  `verify_spend_rules(...)` theorem shape under one theorem carrier at
  proof-generation time.
2. The public statement surface preserves resolved-input, output, membership,
  and scope facts with fail-closed drift rejection.
3. `build_tx_package_digest(...)` remains the only public or persisted
  proof-binding root across wallet, checkpoint, and rollup seams.

## ⚙️ 040-10 Canonical Theorem Preservation

### Theorem Scope

The canonical proof statement must preserve the current ordered theorem shape
already enforced by `verify_spend_rules(...)`.

Closure is not defined as public or trustless recovery of the full theorem from
the current public-only carrier fields. The current closure is internal:
recompute the canonical public statement from public tx facts and validate the
backend witness relation before producing the deterministic canonical artifact.

### Theorem Code Anchors

- `crates/z00z_wallets/src/core/tx/spend_rules.rs`
  - `spend_order()` freezes the rule order.
  - `spend_triplets()` freezes the witness/public/domain mapping.
  - `verify_spend_rules(...)` remains the theorem authority.
- `crates/z00z_wallets/src/core/tx/spend_verification.rs`
  - `build_public_spend_contract(...)` emits the canonical carried statement.
  - `verify_tx_public_spend_contract(...)` recomputes and rechecks the same
    statement before authorization acceptance.
  - `SpendInputProofWire::input_asset_id_hex` now binds each carried proof
    input to the exact `tx.inputs` state key at the same position, so producer
    and verifier both fail closed on mismatched proof-bundle pairing.
- `crates/z00z_wallets/src/core/tx/spend_proof_backend.rs`
  - the bounded prove seam must consume the canonical public statement and
    validate the witness-only relations without widening the public carrier.

### Public Input Inventory

The theorem backend must bind these public facts exactly, in canonical order:

1. `build_tx_package_digest(...)` output.
2. Typed `prev_root`.
3. `chain_id`, `chain_type`, `chain_name`, and `tx_ver`.
4. Ordered `SpendInputRef` vector.
5. Ordered `SpendInputLeaf` public fields, including `input_asset_id_hex`
   pairing.
6. Ordered output `AssetLeaf` public fields.
7. Fee-as-output balance data.
8. The regular-spend nullifier vector.
9. Any explicit range-proof compatibility inputs that remain outside the proof
   backend.

### Private Witness Inventory

The theorem backend must prove, but must not publish, these witness-only
relations:

1. `ReceiverSecret`.
2. Ordered `s_in[i]` for each consumed spend input.
3. The owner/view chain enforced by `verify_spend_rules(...)`, derived
  internally from `ReceiverSecret`, `r_pub`, and `s_in[i]`.

Frozen suite contract for this integrity gate:

- canonical suite id: `regular_spend_theorem_bpplus`
- canonical theorem contract: one `T(S, W)` relation on the existing proof
  carrier with no semantic aliases
- prohibited semantic drift: no version-suffixed proof branches, no dormant
  compatibility carrier, and no dual-lane verifier framing

### Capability Decision Freeze

`040-10-PLAN.md` freezes Option C as the only valid path for internal relation closure.
The approved workspace-owned spend pipeline remains the only allowed host
surface, vendor Tari code stays read-only, and authoritative membership against
`prev_root` must be lifted into the theorem sub-witness instead of remaining an
external assumption. No bridge shadow layer, fallback verifier, or parallel
proof architecture may be introduced during this migration.

### Theorem Executable Evidence

- `crates/z00z_wallets/tests/test_spend_statement.rs`
  - identical tx facts rebuild the same statement deterministically;
  - fee drift, prev-root drift, output drift, chain drift, version drift, and
    ad hoc statement fragments reject with `StatementMismatch`.
- `crates/z00z_wallets/tests/test_spend_prover_contract.rs`
  - the producer emits the canonical proof/auth carrier from live witness data;
  - mismatched witness or package facts fail closed.
- `crates/z00z_wallets/tests/test_spend_proof_backend.rs`
  - non-empty witness shape is enforced;
  - malformed payloads and statement drift reject fail closed;
  - the backend matrix must prove canonical theorem-artifact acceptance and
    legacy rejection without witness replay.
- `crates/z00z_wallets/tests/test_tx_proof_verifier.rs`
  - the public verifier rejects malformed or drifted carried fields;
  - the verifier matrix must prove there is no fallback acceptance path outside
    the canonical theorem contract.

### Theorem Integrity Verdict

The shipped producer and verifier now close the input-binding pairing gap:
forged receiver-bound inputs reject before signing, the carried statement
remains ordered and deterministic, and mismatched `tx.inputs` /
`proof.inputs` pairings reject with `InputRefMismatch`.

`040-10` internal relation scope is complete only when the producer and runtime
wallet seams consume the same canonical theorem artifact and the legacy proof
surface is gone. Public-witness expansion remains bounded to the frozen witness
table: `receiver_secret + ordered s_in[i]` plus the explicit membership
sub-witness required for authoritative `prev_root` composition. Public/trustless
proof-of-knowledge, checkpoint theorem finality, and rollup settlement closure
remain open until an actual verifier-side proof path lands.

## ✅ 040-10 Public Input Surface Preservation

### Public Input Scope

The canonical public statement must preserve the same resolved input ids,
public input leaf fields, output leaf fields, and explicit scope facts already
used by the live spend path.

### Public Input Code Anchors

- `crates/z00z_wallets/src/core/tx/spend_verification.rs`
  - `SpendInputRef` and `SpendInputLeaf` define the carried public input shape.
  - `SpendInputProofWire::input_asset_id_hex` binds the carried proof inputs
    to the exact consumed tx input state keys.
  - output leaf-ad, owner-tag, commitment, and range-proof checks are
    recomputed from tx facts during public spend verification.
  - chain id, tx version, chain type, chain name, and `prev_root` stay inside
    the canonical statement scope.

### Public Input Executable Evidence

- `crates/z00z_wallets/tests/test_spend_statement.rs`
  - statement drift rejects on fee, root, output-role, chain, and version
    changes;
  - digest recomputation ignores auth-only spend fields, so public inputs stay
    tied to tx facts rather than mutable carrier extras.
- `crates/z00z_wallets/tests/test_tx_proof_verifier.rs`
  - input-count drift rejects;
  - mismatched input state-key binding rejects;
  - malformed input nullifier or prev-root encodings reject;
  - missing output `leaf_ad_id`, `r_pub`, and `owner_tag` reject;
  - output `leaf_ad_id` drift rejects as canonical statement drift;
  - missing or tampered output range proof rejects;
  - input/output `leaf_ad_id` overlap rejects.

### Public Input Integrity Verdict

The public input surface is explicit and fail closed: carried statement fields
must match recomputed tx facts, and drift in input, output, state-key binding,
or scope data is a hard verifier failure. Output `leaf_ad_id` drift closes
through canonical `StatementMismatch` rather than through a separate
`BadOutputLeafAd` branch.

## ✅ 040-11 Digest-Root Discipline

### Digest Scope

`build_tx_package_digest(...)` is the only public or persisted proof-binding
root. `compute_tx_digest_from_wire(...)` remains an internal helper and must
never become the sole public proof root.

### Digest Code Anchors

- `crates/z00z_wallets/src/core/tx/tx_verifier.rs`
  - `verify_full_tx_package(...)` is the canonical composed admission entry
    point.
  - local package verification remains insufficient without the public spend
    contract.
- `crates/z00z_wallets/src/core/tx/spend_verification.rs`
  - public spend verification recomputes the statement under the package
    digest-bound contract.
- `crates/z00z_wallets/src/core/tx/output_flow.rs`
  - `compute_tx_digest_from_wire(...)` stays helper-only.

### Digest Executable Evidence

- `crates/z00z_wallets/tests/test_spend_statement.rs`
  - bare wire digest used as the carried statement root rejects with
    `StatementMismatch`;
  - package digest remains stable when spend auth-only fields are tampered;
  - non-canonical spend proof hex is rejected during package digest rebuild.
- `crates/z00z_wallets/src/core/tx/test_tx_verifier_suite.rs`
  - a structurally valid package without spend proof fails the canonical full
    verifier;
  - the public spend boundary still rejects packages that pass local wire
    checks but lack the canonical public spend contract.

### Digest Integrity Verdict

The package digest is the only authoritative public root for the regular spend
path. Any proof path that tries to bind only the wire digest remains invalid.

## Review Discipline

Closeout review for `040-09` through `040-11` must stay bounded to these integrity claims. Do
not reopen the retired superseded draft, do not invent a separate checkpoint proof
layer, and do not restate wallet-local or witness-local behavior as a broader
public theorem than the current code actually binds.
