# z00z_runtime validators

This crate is the validator-owned checkpoint, spend, and verdict surface.

## Canonical modules

- `checkpoint`: validator-side checkpoint flow marker and future checkpoint-specific coordination surface.
- `claim_verify`, `tx_verify`, `spend`, `nullifier`, and `reconcile`: canonical validation lanes for claim packages, tx packages, spend rules, nullifier checks, and batch reconciliation.
- `verdict`: final validator outputs such as `Verdict`, `VerdictKind`, and `ResolvedBatch`.
- `engine`: `ValidatorService` and `ValidatorBoundary`.
- `artifact`: validator-facing artifact decoding helpers.

## Boundaries

- `ValidatorService` validates already-resolved runtime batches; it does not own planner admission or watcher projection.
- `ValidatorBoundary` may read `ShardPlacementView` or `ShardExecTicket` already attached to a batch, but that metadata stays runtime-owned.
- Settlement roots, proof envelopes, replay semantics, and exact publication route snapshots remain storage-owned.

## Phase 059 object verdict surface

Phase 059 extends validator responsibility from asset-only admission toward
typed settlement objects.

Validators must now fail closed on:

- unknown policy descriptors;
- action not present in the declared action pool;
- missing, expired, revoked, consumed, or out-of-scope rights;
- invalid voucher backing or reserve binding;
- wrong-family proof use;
- stale-root or replay attempts;
- double redemption and invalid lifecycle transitions;
- fee-support data that tries to smuggle value or authority across object-role
  boundaries.

Validators still do not own planner authority or wallet UX semantics. They
consume typed object packages and storage-owned proof contracts and emit the
canonical verdict surface.
