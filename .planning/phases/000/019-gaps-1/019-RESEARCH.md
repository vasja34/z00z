# Phase 019: gaps-1 - Research

**Researched:** 2026-03-24
**Domain:** wallet replay protection, receiver scan taxonomy, wallet backup and restore contracts
**Confidence:** HIGH

## Summary

Phase 019 is not blocked by lack of ideas. It is blocked by three contract mismatches that are already visible in the codebase.

First, claim nullifiers are already a real protocol concept in `z00z_wallets`, but they are not part of the canonical storage root. The current implementation derives them deterministically from `(claim_id, recipient_owner, chain_id)`, reserves them before claim bundle persistence, and finalizes them only after storage apply succeeds. That is good replay protection discipline, but it is not yet storage-owned state.

Second, the receiver scan stack has the right building blocks, but the wrong public ergonomics. The canonical leaf path can report `InvalidProof`, while the runtime scanner and pack-return APIs still allow direct callers to collapse malformed input or proof failure into `NotMine` or `None`. The service layer compensates for this with a precheck, but the low-level public surface is still a footgun.

Third, the public backup API is genuinely incomplete, but the repository already contains a fuller export/import path. `create_backup` and `restore_backup` currently operate on a metadata-only encrypted container. In parallel, `export_wallet_payload` and `import_wallet_payload` already move a checksummed `WalletPersistenceState` plus seed phrase. The correct direction is to converge these two systems, not to evolve the current backup container independently.

**Primary recommendation:** Plan three implementation tracks inside Phase 019:
move claim nullifier state toward a storage-owned contract, add report-first
scan APIs with explicit invalid-input handling, and rebase the public backup
file format onto the existing full wallet export/import payload.

## User Constraints

- Answer the gaps from `todo.md` concretely and completely.
- Be prescriptive, not exploratory.
- Separate current reality, risks, and recommended direction.
- Call out architectural ambiguity and missing invariants.
- Identify items that should become separate follow-up phases.

## Phase Requirements

| ID | Description | Research Support |
| ---- | ---- | ---- |
| PH19-NULL | Explain nullifier origin, purpose, lifecycle, and relation to JMT asset presence/absence | `core/claim/nullifier.rs`, `core/claim/nullifier_store.rs`, `claim_pkg_store.rs`, `claim_pkg_consumer.rs`, `z00z_storage::assets::AssetStore` |
| PH19-SCAN | Validate scanner-layer findings and define the correct invalid-input vs not-mine vs invalid-proof contract | `core/address/leaf_scan.rs`, `core/address/stealth_scanner.rs`, `wallet_service.recv_one`, runtime parity tests, prefilter tests |
| PH19-BACKUP | Validate backup/restore claims, describe current restore reality, and define the correct complete restore contract | `core/backup/*`, `wallet_service.create_backup`, `wallet_service.restore_backup`, `wallet_service.export_wallet_payload`, `wallet_service.import_wallet_payload`, `core/wallet/snapshot.rs` |

## Project Constraints (from copilot-instructions.md)

- All code, comments, and documentation artifacts must remain in English.
- Use `z00z_utils` as the single source of truth for file I/O, codecs, time, and RNG abstractions.
- Do not introduce direct `std::fs`, raw `serde_json`/`serde_yaml`, or direct `SystemTime` usage in business logic when project abstractions already exist.
- Treat `crates/z00z_crypto/tari/` as read-only.
- Prefer provider-backed randomness and time when code needs deterministic testing or dependency injection.
- Keep changes minimal and aligned with existing crate boundaries: storage state belongs in `z00z_storage`, wallet-runtime policy belongs in `z00z_wallets`.

## Phase-Specific Answers

### 1. Nullifiers

#### Nullifier current reality

- Claim nullifiers are derived in `crates/z00z_wallets/src/core/claim/nullifier.rs` by hashing `chain_id || claim_id || owner` under the domain tag `z00z.nullifier.derive.v1.`.
- Claim package verification enforces that `tx.context.nullifier_hex` exactly equals `derive_nullifier(claim_id, recipient_owner_hex, chain_id)` in `crates/z00z_wallets/src/core/tx/claim_tx.rs`.
- Stage 3 claim package construction computes the same nullifier before package emission in `crates/z00z_simulator/src/scenario_1/stage_3.rs`.
- Replay protection is implemented by `NullifierStateStore` in `crates/z00z_wallets/src/core/claim/nullifier_store.rs`, with lifecycle states `Reserved` and `Spent`.
- The simulator claim publish flow reserves nullifiers before bundle persistence, rolls them back on write or verification failure, and marks them spent only after `AssetStore::apply_ops()` succeeds in `crates/z00z_simulator/src/claim_pkg_consumer.rs`.
- The current claim publish path builds only `StoreOp::Put` operations for claim outputs; it does not delete an existing input leaf from JMT as part of claim consumption.
- `z00z_storage` does not currently own nullifier state. Asset lookup returns `None` when a path or leaf is absent, but there is no nullifier namespace in `crates/z00z_storage/src/**`.

#### Nullifier purpose

- They are not an ownership record.
- They are a replay and double-claim prevention key for the claim domain.
- They are not evidence that every JMT spend path needs a nullifier. In the current repository they protect the claim package identity, not the generic "existing input leaf is deleted and replaced by outputs" transfer path.
- They let the system reject repeated use of the same claim identity even if an output leaf is later deleted, missing, or not yet materialized in the current JMT state.

#### Nullifier ownership and lifecycle semantics

- `NullifierClaim` is the full public identity of a claim reservation: nullifier, claim id, chain id, recipient owner binding, and tx digest.
- `NullifierLease` is only a capability handle used to finalize or roll back an already reserved nullifier.
- The nullifier is bound to the claim transaction identity, not to a mutable asset leaf.
- A nullifier starts as `Reserved` when a claim bundle is accepted for persistence.
- It transitions to `Spent` only after claim outputs are successfully published into storage.
- It must be rolled back if bundle serialization, bundle write, bundle verification, or storage apply fails.

#### Nullifier relationship to JMT empty leaf and asset presence

- JMT asset presence answers: “Is there currently a canonical asset leaf at this path?”
- JMT asset absence answers only: “There is no current leaf materialized at that path.”
- Nullifier state answers a different question: “Has this claim identity already been consumed or reserved?”
- These are different invariants and they should stay different.
- For an ordinary canonical spend path that truly consumes an existing JMT input leaf, leaf presence or absence may already be the relevant spend invariant. That is not the path being discussed in Phase 019.
- Phase 019 is about the claim path, where the verifier binds `claim_id_hex`, `claim_source_asset_id_hex`, `claim_source_commitment_hex`, and `nullifier_hex`, but the storage publish step inserts new claim outputs rather than deleting a consumed input leaf from JMT.
- An empty or missing JMT leaf cannot replace nullifier replay protection because absence does not tell whether the claim was never used, is being reserved right now, was already finalized, or was rolled back after a failed publish.

#### Nullifier architectural ambiguity

- The current nullifier contract is wallet-owned or simulator-owned process state plus optional file persistence, not storage-owned canonical state.
- Because of that, the canonical storage root does not commit to replay-protection state yet.
- The code enforces one strong invariant already: nullifier equality is deterministic and checked by the claim verifier.
- The code does not yet enforce one stronger system invariant at the storage boundary: “a finalized claim publish must advance asset state and replay-protection state atomically under one canonical root.”

#### Nullifier decision-ready conclusion

- Keep nullifiers as a separate replay-protection concept. Do not overload JMT empty leaf semantics.
- Define the contract explicitly as: `derive -> reserve -> persist bundle -> verify bundle -> apply storage -> finalize`.
- Treat `NullifierClaim` as identity data and `NullifierLease` as a transaction-scoped reservation token.
- Read this section narrowly: it explains why the current claim-domain publish flow still needs anti-replay state even though ordinary JMT leaf-consumption flows may rely on canonical input-leaf spend semantics instead.
- Phase 019 should treat nullifier ownership as its storage-integration track,
  not as a deferred architectural note. Without that, claim replay protection
  remains correct locally but not root-committed.

### 2. Scanner and Validation Layering

#### Scanner current reality

- The canonical leaf scanner in `crates/z00z_wallets/src/core/address/leaf_scan.rs` has two public surfaces.
- `receiver_scan_leaf()` returns `Ok(None)` for `DetectFail::Tag` and `DetectFail::Decrypt`.
- `receiver_scan_report()` maps the same detection failures to `ReceiveStatus::InvalidProof`.
- Parse and commitment failures remain hard errors in both canonical APIs.
- The runtime scanner in `crates/z00z_wallets/src/core/address/stealth_scanner.rs` returns `ScanResult::NotMine` for missing stealth fields and some malformed runtime assets, and returns `MaybeMine` for other invalid owned-output cases.
- The service-layer API `WalletService::recv_one()` performs `asset.validate_stealth_consistency()` first and maps malformed runtime shape to `ReceiveReject::InvalidInput` before invoking the scanner.
- Runtime parity tests confirm canonical and runtime parity for well-formed assets and proof failures, but current prefilter tests also explicitly expect `NotMine` for malformed runtime cases.

#### Scanner validation of the findings in `todo.md`

- The medium-severity finding is correct: direct callers of `StealthOutputScanner::scan_leaf()` can silently degrade malformed runtime input into `NotMine` or `MaybeMine` if they skip service-level prevalidation.
- The medium-severity finding about split API semantics is also correct: `receiver_scan_leaf()` is not a trustworthy failure-taxonomy surface, while `receiver_scan_report()` is.
- The low-severity finding about RNG provider discipline is also correct: `crates/z00z_wallets/src/core/stealth/output.rs` still uses `SystemRngProvider` directly inside the builder path instead of injected RNG. This is architecturally imperfect but not a cryptographic correctness break.

#### Scanner intended boundary

- Invalid input classification belongs at the public runtime boundary.
- Ownership and proof classification belong inside the scan authority.
- A direct caller should never need to infer taxonomy from `Option<AssetPackPlain>` or from `ScanResult` alone.

#### Scanner correct API contract shape

- The authoritative public contract should be report-first.
- For canonical leaves, `receiver_scan_report()` is the correct taxonomy surface.
- For runtime assets, the codebase is missing the equivalent authoritative report-first entry point.
- The correct runtime contract is one of these two shapes:

| Shape | Recommendation | Reason |
| ---- | ---- | ---- |
| `Result<ReceiveReport, ReceiveReject>` | Preferred | Distinguishes invalid input and runtime failure before scan, while keeping taxonomy explicit |
| `ReceiveReport` plus guaranteed prevalidation | Acceptable only for internal use | Too easy for direct callers to bypass unless the API itself performs validation |

- `ScanResult` should remain an adapter DTO for already-validated callers, not the primary public contract.
- `receiver_scan_leaf()` should remain a pack-extraction helper, not a taxonomy authority.

#### Scanner hardening location

- Service level: keep the existing `validate_stealth_consistency()` precheck. It is correct defense in depth and should not be removed.
- Scanner level: add a runtime report API that performs its own schema validation and returns explicit `InvalidInput` before any ownership classification.
- Canonical leaf level: keep `receiver_scan_report()` as the authoritative status API and treat `receiver_scan_leaf()` as a data-return helper.

#### Scanner decision-ready conclusion

- The `todo.md` findings are correct.
- The safe public contract is `ReceiveReport`-first, not `ScanResult`-first and not `Option`-first.
- Hardening must happen at both levels: service prevalidation stays, and scanner APIs need an explicit report surface so direct callers cannot silently lose failure taxonomy.
- This should be a dedicated Phase 019 plan track because existing tests and
    adapters deliberately encode the current malformed-input behavior.

### 3. Backup Flow and Wallet State Restore

#### Backup current reality

- The public backup API in `crates/z00z_wallets/src/services/wallet_service.rs` uses `BackupExporterImpl` and `BackupImporterImpl`.
- `BackupExporterImpl` serializes a `BackupPayloadV1` that contains only `wallet_id` and `network`.
- `BackupImporterImpl` returns `ImportedWalletData { wallet_id, network, keys: vec![], transactions: vec![], assets: vec![] }`.
- `restore_backup()` therefore restores metadata identity only. It does not restore wallet snapshot state.
- Separately, `WalletPersistenceState` in `crates/z00z_wallets/src/core/wallet/snapshot.rs` is a real checksummed snapshot contract with `password_verifier`, `address_deriver`, `settings`, `state`, `claimed_assets`, and version migration.
- Separately again, `export_wallet_payload()` and `import_wallet_payload()` already move a fuller encrypted export that contains `WalletExportPack { snapshot, seed_phrase }`.
- `restore_snapshot()` restores wallet name, wallet state, password verifier, address-deriver counters, settings, and claimed assets into memory.

#### Backup what is actually restorable today

- `restore_backup()` through the public backup API restores only metadata-level identity.
- `import_wallet_payload()` through the encrypted wallet export path can restore a real snapshot, and in the new format it can also carry the seed phrase.
- Legacy snapshot import branches can restore snapshot state, but when the seed phrase is absent they generate a new seed phrase before persisting `.wlt`, which is not equivalent to full secret restoration.

#### Backup validated claims from `todo.md`

- The claim that the public backup flow does not restore full wallet state is verified.
- The claim that the active backup container schema does not match a fuller wallet snapshot contract is verified.
- The stronger conclusion is this: the repository already has a fuller export/import mechanism, but the public backup feature has not been converged onto it.

#### Backup complete restore contract requirements

- Secret restoration material: original seed phrase or equivalent root secret material.
- Checksummed wallet snapshot state.
- Password verifier or equivalent rewrapped secret material.
- Address-derivation counters.
- Wallet settings.
- Claimed assets.
- Explicit versioning and migration.
- Integrity verification before publish.

#### Backup optional but explicitly declared restore elements

- Transaction history.
- Pending operation journals such as pending ACK or pending receive queues.
- Scan cursor and checkpoint progress.
- Tag cache or request cache.
- Diagnostic logs.

- If those items are not included, the contract must say so and the restore path must be documented as “snapshot + rescan”, not “full restore”.

#### Backup recommended direction

- Do not evolve `BackupPayloadV1` independently.
- Rebase the backup file format onto the already-existing `WalletExportPack` path.
- The cleanest design is: public backup files become a file-wrapper around the same encrypted payload used by `export_wallet_payload()` and `import_wallet_payload()`.
- Keep metadata for listing and discovery, but make the encrypted payload a full wallet export pack, not a metadata echo.

#### Backup safe migration path

| Step | Action | Why |
| ---- | ---- | ---- |
| 1 | Introduce backup format V2 carrying encrypted `WalletExportPack` | Reuses existing full-state contract instead of inventing a third one |
| 2 | Teach `restore_backup()` to try V2 first, then keep V1 read-compatible | Avoids breaking old backup files |
| 3 | Mark V1 as metadata-only legacy backup in docs and UI | Prevents false expectations |
| 4 | Optionally add rescan-based restore extensions later | Keeps initial migration minimal |

#### Backup decision-ready conclusion

- The current public backup API is incomplete by design, not by accidental bug.
- The repository already contains the correct building block for a fuller backup contract: `WalletExportPack` plus encrypted import/export payloads.
- The correct fix is convergence, not a second independent backup schema.
- This should be a dedicated Phase 019 plan track because it changes external
    wallet backup semantics and migration order matters.

### 4. Additional Validated Finding: RNG Provider Discipline

#### RNG current reality

- `crates/z00z_wallets/src/core/stealth/output.rs` uses `SystemRngProvider` inside `make_amount()` and direct randomness for sender-side output construction.
- `crates/Z00Z_DESIGN_FOUNDATION.md` explicitly sets `z00z_utils::rng::RngProvider` as the standard abstraction.

#### RNG conclusion

- This is a real architecture gap.
- It is not part of the three primary blockers for Phase 019.
- Do not mix it into the scan or backup phases unless you intentionally scope a small refactor follow-up.

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
| ---- | ---- | ---- | ---- |
| `z00z_wallets::core::claim::{derive_nullifier, nullifier_store}` | workspace current | Claim-domain nullifier derivation and replay lifecycle | Already canonical for reserve/finalize semantics |
| `z00z_wallets::core::address::{leaf_scan, stealth_scanner}` | workspace current | Canonical leaf scan plus runtime asset adapter | This is the existing scan architecture that planning must harden, not replace |
| `z00z_wallets::services::wallet_service` | workspace current | Public boundary, validation precheck, persistence route | Service layer already owns invalid-input classification |
| `z00z_wallets::core::wallet::snapshot` | workspace current | Checksummed wallet snapshot contract | This is the real restoreable state contract |
| `z00z_storage::assets::AssetStore` | workspace current | Canonical asset presence and apply ownership | Current source of truth for leaf existence, not nullifier replay state |

### Supporting

| Library | Version | Purpose | When to Use |
| ---- | ---- | ---- | ---- |
| `z00z_utils::io` | workspace current | File I/O abstraction | All backup, claim bundle, and research follow-up persistence work |
| `z00z_utils::codec::{JsonCodec, BincodeCodec}` | workspace current | Canonical serialization | Bundle, snapshot, and backup payload serialization |
| `z00z_utils::rng::RngProvider` | workspace current | RNG abstraction | Provider-backed randomness for testable crypto-adjacent code |
| `WalletBackupCrypto` | workspace current | Argon2id + XChaCha20-Poly1305 + checksum glue | Existing backup container crypto plumbing |
| `WalletEncryption` and `EncryptedWalletContainer` | workspace current | Full encrypted wallet export/import payload | Existing path closest to complete restore |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
| ---- | ---- | ---- |
| Inference from JMT absence | Storage-owned nullifier namespace | Correct long-term direction, but requires a dedicated storage integration phase |
| `ScanResult` as public contract | Report-first runtime receive API | Stronger taxonomy, safer for direct callers |
| Evolving backup V1 payload | Reusing `WalletExportPack` | Reuse wins because the fuller contract already exists |

## Architecture Patterns

### Recommended Project Structure

```text
crates/z00z_wallets/src/
├── core/claim/            # Nullifier derivation and replay store
├── core/address/          # Canonical leaf scan and runtime adapter
├── core/wallet/           # Snapshot contract
├── core/backup/           # Legacy public backup container
└── services/              # Runtime boundary, prevalidation, persistence

crates/z00z_simulator/src/
├── claim_pkg_store.rs     # Reserve/load/finalize claim nullifiers
└── claim_pkg_consumer.rs  # Verify bundles, publish into storage, finalize nullifiers
```

### Pattern 1: Reserve Then Finalize

#### Pattern 1 what

Replay-protection state is reserved before any bundle write, then finalized only after storage apply succeeds.

#### Pattern 1 when to use

Any claim or spend flow where replay state must fail closed across partial writes.

#### Pattern 1 example

```rust
let claims = claim_nulls(&packages)?;
let leases = reserve_nulls(&claims)?;
write_file(&claim_pkg_path, &bundle_bytes)?;
let ops = build_claim_store_ops(&packages)?;
store.apply_ops(ops)?;
finalize_nulls(&leases, &claims)?;
```

### Pattern 2: Canonical Authority Plus Runtime Adapter

#### Pattern 2 what

`leaf_scan` owns canonical leaf semantics; `stealth_scanner` adapts runtime `Asset` values for wallet use.

#### Pattern 2 when to use

Whenever the same cryptographic ownership logic exists in both canonical leaf and runtime asset forms.

#### Pattern 2 example

```rust
let canon = receiver_scan_report(&keys, &leaf)?;
asset.validate_stealth_consistency().map_err(|_| ReceiveReject::InvalidInput)?;
let runtime = scanner.scan_leaf(&asset);
assert_eq!(canon, runtime.recv_report());
```

### Pattern 3: Snapshot Contract Separate From Transport Envelope

#### Pattern 3 what

The snapshot defines restorable state, while the export or backup container defines encryption and transport.

#### Pattern 3 when to use

Any restoreable wallet export path.

#### Pattern 3 example

```rust
let snapshot = self.create_snapshot(wallet_id).await?;
let export_pack = WalletExportPack { snapshot, seed_phrase };
let container = WalletEncryption::encrypt_wallet(password, &aad, &bytes)?;
```

### Anti-Patterns to Avoid

- **Using JMT absence as replay protection:** absence is not a spent marker.
- **Using `receiver_scan_leaf()` as taxonomy authority:** `Option` drops invalid-proof classification.
- **Using `scan_leaf()` directly on unvalidated runtime assets:** malformed runtime input can degrade to `NotMine` or `MaybeMine`.
- **Maintaining separate full-restore and backup schemas:** the repo already has one better full-state path.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
| ---- | ---- | ---- | ---- |
| Claim replay protection | Ad hoc “missing leaf means unused” logic | Nullifier lifecycle store with reserve/finalize semantics | Handles partial writes, rollback, and replays correctly |
| Receive failure taxonomy | Caller-side inference from `Option` or `ScanResult` | Explicit `ReceiveReport` contract | Preserves invalid-input and invalid-proof boundaries |
| Full wallet restore | New custom backup schema from scratch | Existing `WalletExportPack` plus encrypted export payload | Reuses checksummed snapshot and seed-phrase transport |
| Backup crypto | Custom AEAD and KDF glue | `WalletBackupCrypto` or `WalletEncryption` | Reduces security and compatibility risk |

### Key insight

The codebase already contains the right primitives. The problem is contract convergence and ownership boundaries, not missing algorithms.

## Common Pitfalls

### Pitfall 1: Confusing Absence With Spend State

#### Pitfall 1 what goes wrong

A missing asset leaf is treated as proof that the corresponding claim was never used.

#### Pitfall 1 why it happens

Asset presence and replay protection are different state machines.

#### Pitfall 1 how to avoid

Keep nullifier lifecycle separate and explicit.

#### Pitfall 1 warning signs

Logic asks only `get_item() == None` before accepting a claim.

### Pitfall 2: Using Pack-Return APIs As If They Were Report APIs

#### Pitfall 2 what goes wrong

`None` is interpreted as “foreign” when it may actually mean invalid proof.

#### Pitfall 2 why it happens

`receiver_scan_leaf()` intentionally optimizes for pack extraction, not taxonomy.

#### Pitfall 2 how to avoid

Use `receiver_scan_report()` or a new runtime report API whenever caller behavior depends on rejection class.

#### Pitfall 2 warning signs

Business logic branches on `Option<AssetPackPlain>` without consulting `ReceiveReport`.

### Pitfall 3: Assuming Public Backup And Wallet Export Are The Same Feature

#### Pitfall 3 what goes wrong

The product promises “restore backup” while the file only contains metadata.

#### Pitfall 3 why it happens

Two parallel transport formats exist today.

#### Pitfall 3 how to avoid

Converge the public backup format onto the full encrypted wallet export payload.

#### Pitfall 3 warning signs

Backup code and wallet export code evolve independently.

### Pitfall 4: Fixing Taxonomy Only In Tests Or Only In Services

#### Pitfall 4 what goes wrong

One entry point becomes safe, but direct callers still silently misclassify failures.

#### Pitfall 4 why it happens

The codebase has multiple scan surfaces with different semantics.

#### Pitfall 4 how to avoid

Keep service prevalidation and add scanner-level report APIs.

#### Pitfall 4 warning signs

New code calls `scan_leaf()` directly outside service wrappers.

## Code Examples

Verified patterns from the codebase:

### Claim Nullifier Derivation and Enforcement

```rust
let nullifier = derive_nullifier(claim_id_bytes, &recipient_owner, chain_id);
let nullifier_hex = nullifier.to_hex();

let expected = derive_nullifier(&claim_id, &owner, chain_id).to_hex();
if tx.context.nullifier_hex != expected {
    return Err(ClaimTxError::NullifierMismatch(...));
}
```

### Fail-Closed Claim Publish Sequence

```rust
let claims = claim_nulls(&packages)?;
let leases = reserve_nulls(&claims)?;
write_file(&claim_pkg_path, &bundle_bytes)?;
let ops = build_claim_store_ops(&packages)?;
store.apply_ops(ops)?;
finalize_nulls(&leases, &claims)?;
```

### Runtime Boundary With Explicit Invalid-Input Gate

```rust
asset
    .validate_stealth_consistency()
    .map_err(|_| ReceiveReject::InvalidInput)?;
let recv_keys = self.receiver_keys(wallet_id).await?;
let scanner = StealthOutputScanner::from_keys(&recv_keys);
let result = scanner.scan_leaf(asset);
```

### Full-State Wallet Export Path

```rust
let snapshot = self.create_snapshot(wallet_id).await?;
let export_pack = WalletExportPack {
    snapshot,
    seed_phrase,
};
let plaintext = codec.serialize(&export_pack)?;
let container = WalletEncryption::encrypt_wallet(password, &aad, &plaintext)?;
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
| ---- | ---- | ---- | ---- |
| Replay inferred from asset presence | Explicit reserve/finalize nullifier lifecycle | Current codebase | Correct fail-closed replay handling, but not yet storage-owned |
| Pack-return receive API | Report-first taxonomy API | Current codebase already partial | Correct classification requires explicit report APIs |
| Metadata-only backup container | Checksummed full snapshot plus encrypted export pack | Current codebase already contains this path | Backup feature should converge onto the fuller path |

### Deprecated or outdated assumptions

- “The backup API is the canonical restore surface.” This is false in current code.
- “A missing leaf is enough to prove non-replay.” This is false in current code.
- “Runtime scan API preserves all failure taxonomy by itself.” This is false in current code.

## Resolved During Context Capture

1. **Nullifier target direction is now locked.**
     - Locked decision: Phase 019 targets a storage-owned nullifier contract.
     - Locked invariant: finalized claim publish must advance asset state and
         nullifier state atomically.
     - Remaining planner choice: exact placement under the existing
         `z00z_storage` write boundary.

2. **Public restore direction is now locked.**
     - Locked decision: the public backup contract should converge on
         `WalletExportPack` and target full restore semantics.
     - Remaining planner choice: whether pending journals and scan cursor are
         included in the first converged contract or explicitly deferred under a
         documented rescan extension.

## Rollout Hazards

- Direct `receiver_scan_leaf()` and `scan_leaf()` consumers already exist in
    simulator tests, wallet examples, RPC method tests, and helper flows. Any
    taxonomy hardening plan must either migrate those callers together or label
    compatibility helpers explicitly.
- The public backup RPC surface already exposes create and restore semantics via
    `wallet_service.rs` and `backup_impl.rs`. Backup convergence must preserve
    read compatibility and avoid promising full restore before the V2 payload is
    actually wired.
- Reachability stubs still exist in `core/wallet/wallet.rs` for export, import,
    and backup-adjacent methods. Planning must avoid treating those placeholders
    as the canonical runtime boundary.

## Environment Availability

No external dependencies were identified for this research phase. This is a code and contract analysis phase, so environment auditing is skipped.

## Validation Architecture

### Test Framework

| Property | Value |
| ---- | ---- |
| Framework | Rust built-in test harness via `cargo test` |
| Config file | Workspace `Cargo.toml` |
| Quick run command | `cargo test -p z00z_wallets --test test_e2e_runtime_parity -- --nocapture` |
| Full suite command | `cargo test -p z00z_wallets && cargo test -p z00z_simulator` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
| ---- | ---- | ---- | ---- | ---- |
| PH19-NULL | Reserve, rollback, reload, and finalize nullifier state correctly | integration | `cargo test -p z00z_simulator --test test_stage3_nullifier_store -- --nocapture` | ✅ |
| PH19-SCAN | Canonical leaf report and runtime scan stay in parity for valid and invalid-proof cases | integration | `cargo test -p z00z_wallets --test test_e2e_runtime_parity -- --nocapture` | ✅ |
| PH19-SCAN | Malformed runtime assets should not silently lose taxonomy once hardening lands | integration | `cargo test -p z00z_wallets --test test_stealth_scanner_prefilter -- --nocapture` | ✅ |
| PH19-BACKUP | Full wallet export and import round-trip snapshot state | unit | `cargo test -p z00z_wallets test_export_import_wallet_payload -- --nocapture` | ✅ |
| PH19-BACKUP | Public backup API semantics stay explicit during migration | integration | `cargo test -p z00z_wallets backup_impl::tests::test_backup_create_list_restore -- --nocapture` | ✅ |

### Sampling Rate

- **Per task commit:** targeted `cargo test` for the touched contract.
- **Per wave merge:** `cargo test -p z00z_wallets` plus relevant simulator tests.
- **Phase gate:** wallet and simulator tests covering nullifier, scan parity, and backup/import semantics must be green.

### Wave 0 Gaps

- `recv_one` has no focused test asserting `ReceiveReject::InvalidInput` on malformed runtime assets.
- There is no direct test proving that public `restore_backup()` restores less state than `import_wallet_payload()`.
- There is no test yet for a converged backup V2 container built on `WalletExportPack`.
- There is no storage-level test binding claim nullifier finalization to a canonical root or checkpoint artifact.

## Sources

### Primary (HIGH confidence)

- `crates/z00z_wallets/src/core/claim/nullifier.rs` - nullifier derivation and domain separation
- `crates/z00z_wallets/src/core/claim/nullifier_store.rs` - reservation, rollback, finalize lifecycle
- `crates/z00z_wallets/src/core/tx/claim_tx.rs` - verifier-enforced nullifier equality
- `crates/z00z_simulator/src/claim_pkg_store.rs` - simulator-side reservation and finalize orchestration
- `crates/z00z_simulator/src/claim_pkg_consumer.rs` - storage publish plus finalize order
- `crates/z00z_storage/src/assets/store.rs` - asset presence or absence semantics
- `crates/z00z_wallets/src/core/address/leaf_scan.rs` - canonical leaf pack and report surfaces
- `crates/z00z_wallets/src/core/address/stealth_scanner.rs` - runtime scan surface and current malformed-input behavior
- `crates/z00z_wallets/src/core/address/stealth_scanner/types.rs` - public status and reject taxonomy
- `crates/z00z_wallets/src/services/wallet_service.rs` - runtime receive precheck, backup API, export/import payloads, snapshot restore
- `crates/z00z_wallets/src/core/backup/backup_exporter_impl.rs` - public backup file contents
- `crates/z00z_wallets/src/core/backup/backup_importer_impl.rs` - public backup restore payload reality
- `crates/z00z_wallets/src/core/wallet/snapshot.rs` - actual snapshot contract and checksum rules

### Secondary (MEDIUM confidence)

- `crates/Z00Z_DESIGN_FOUNDATION.md` - RNG abstraction and snapshot provider patterns
- `docs/Z00Z Messenger.md` - product-language references to pending ACK and wallet FSM concepts

### Tertiary (LOW confidence)

- None. The critical conclusions in this document are based on direct code evidence.

## Metadata

### Confidence breakdown

- Standard stack: HIGH - derived from active source files, not assumptions
- Architecture: HIGH - verified by end-to-end claim publish and scan code paths
- Pitfalls: HIGH - corroborated by direct API shape and existing tests

**Research date:** 2026-03-24
**Valid until:** 2026-04-07
