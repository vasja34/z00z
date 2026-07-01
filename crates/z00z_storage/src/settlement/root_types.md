# Settlement Root Types And Live Boundary

This note records the storage root taxonomy after the live generalized
settlement cutover started. The public vocabulary now includes the live
settlement-generation contracts while physical tree layout remains private to
storage internals.

## Current Public Contract

`SettlementStateRoot` is the live public semantic settlement root. It
binds `RootGeneration::SettlementV1` and the 32-byte settlement-state
commitment for the canonical hierarchy:

```text
definition_id -> serial_id -> terminal_id -> SettlementLeaf
```

`SettlementLeaf` is the terminal family. It has explicit asset, right, and
voucher variants: `SettlementLeaf::Terminal(TerminalLeaf)`,
`SettlementLeaf::Right(RightLeaf)`, and
`SettlementLeaf::Voucher(VoucherLeaf)`.

Public storage paths above the backend are `SettlementPath` values and
settlement proof paths only. `AssetPath` is historical terminology and is not
part of the live authority surface.

Legacy asset-root vocabulary is removed from the live storage surface. There is
no `AssetStateRoot` adapter, alias, or old-root shim for the settlement
generation.

`CheckRoot` is checkpoint-facing evidence derived from the typed settlement-root
contract. It preserves a typed checkpoint root so checkpoint and state-
transition APIs do not accept an arbitrary 32-byte digest as storage state.

`TxDigest` is not a root. It must not be converted into `CheckRoot`.

`FeeEnvelope` is a separate processing-support object. It can bind payer or
sponsor support, budget, expiry, replay protection, and transition support, but
it does not prove ownership, right validity, or wallet control.

There is no parallel semantic root plane alongside the settlement root.

## Physical Backend Data

`backend_root` inside `ProofBlob` is physical backend data used to verify
branch proofs. It is proof-local or diagnostic only and must not be treated as
public settlement authority.

`TreeId`, namespace keys, branch ordering, path-index rows, and physical key
layout stay private to storage internals. Downstream crates must not use them
as authority.

`DefinitionRootLeaf` and `SerialRootLeaf` carry semantic child roots inside a
disclosed proof path. They do not make the physical backend root public.

## Live Hard Cutover Boundary

The live settlement cutover replaces the old design-era blockers with live settlement contracts:

- `SettlementStateRoot` carries root-generation metadata.
- `SettlementPath` and `TerminalId` widen the path vocabulary without exposing
  physical tree ids.
- `RightLeaf` is a narrow terminal object for bounded non-coin rights.
- `VoucherLeaf` is a narrow terminal object for conditional value claims with
  committed lifecycle, policy, and action-pool bindings.
- `FeeEnvelope` remains separate from right meaning.
- `AdaptiveBucket`, `BucketEpoch`, `BucketOccupancyEvidence`,
  `BucketOccupancyMetric`, `SplitProof`, `MergeProof`, and
  `PolicyTransitionProof` are storage-owned HJMT policy contracts.
- `BucketOccupancyMetric` may carry exact local counts, but verifier-visible
  proofs must stay on bounded occupancy evidence only.

The implementation must update existing storage entrypoints in place. It must
not add a parallel legacy reader, old-row adapter, alias layer, or second
public authority plane.

## Development Hard Cutover

Development and simulator flows must now start from the live
generalized corpus, not from old asset-only artifacts.

- Leave `Z00Z_SETTLEMENT_BACKEND_MODE` unset or set it to `hjmt`. Any other
  value rejects fail-closed in the live runtime.
- Use `Z00Z_SETTLEMENT_BUCKET_BITS` only to select the live fixed-bucket policy for
  testing or operational benchmarking.
- Treat `crates/z00z_core/configs/devnet_genesis_config.yaml` and its
  referenced rights, policies, and vouchers subfiles as the canonical
  regeneration inputs for dev stores.
- Treat `crates/z00z_core/configs/devnet_assets_config.yaml` as secondary
  registry/example data only.
- Treat `crates/z00z_simulator/src/scenario_1/scenario_config.yaml` as the
  canonical owner of the `stage13_hjmt_settlement_examples` artifact lane.
- Regenerate dev stores from those YAML inputs when the corpus changes. There
  is no live conversion shim from old asset-only store artifacts into the
  live generalized runtime lane.

Exact occupancy counts remain local-only diagnostics. Proof-visible adaptive
surfaces must stay on bounded `BucketOccupancyEvidence`.

## Proof Envelope

Live proof envelopes are storage-owned and settlement-first. The live
source surface carries one settlement-first proof envelope only.

The verifier binds:

- semantic root
- path context
- definition root leaf
- serial root leaf
- terminal settlement leaf
- terminal leaf hash
- backend proof bytes
- semantic/backend root binding

The live settlement contract must support verifier-validating inclusion, deletion, and
non-existence proof families. Placeholder bytes and unsupported families must
reject fail-closed.

## Hjmt Handoff

The production bucketed HJMT backend is the live runtime. Older forest or
asset-only labels are historical or superseded references only. The
ordered implementation tasks provide:

- fixed and adaptive bucket policy with verifier-visible metadata when proofs
  need it
- physical HJMT backend
- child-before-parent root publication
- HJMT commit journal
- crash-safe recovery
- deletion proof semantics
- non-existence proof semantics
- split, merge, and policy-transition proof semantics

The implementation must not create a second storage authority layer, expose
physical roots as public state roots, or route production callers through old
storage adapters.

## Source-Shape Closeout

Phase 054 also closed the storage source-shape cleanup that used to depend on
path-attribute bridges and `.inc` harness shims.

- The live settlement facade now resolves through `settlement/store.rs`.
- Checkpoint and snapshot split modules now resolve through
  `checkpoint/{artifact,build}/mod.rs`, `checkpoint/store.rs`, and `snapshot/store.rs`.
- Shared non-production helpers now live under `src/fixture_support/*`, while the
  snapshot integration tests live as flat `tests/test_snapshot_*.rs` files.

The explicit generated-data exception remains outside this crate in
`crates/z00z_simulator/src/scenario_1/runner_contract.rs`.
