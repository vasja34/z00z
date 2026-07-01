# z00z_rollup_node

This crate is the composition root for rollup-mode services and re-exports the
public settlement theorem verifier.

## Canonical modules

- `config`, `da`, `mode`, `rpc`, `runtime`, and `status`: node-local wiring, mode selection, RPC state, service attachment state, and status projection.
- `verify_settlement_theorem`: re-export of the canonical validator-owned public theorem verifier for the wallet tx package, checkpoint artifact, execution input, and link bundle.

## Boundaries

- `NodeRuntime` wires `AggregatorService`, `ValidatorService`, `WatcherService`, and `DaAdapter`.
- Status projection prefers the placement embedded in `ShardExecTicket` when both a cached placement view and an exec ticket are present.
- Planner authority and bind/publish remain in `z00z_aggregators`, while settlement roots, proofs, and recovery exports remain in `z00z_storage`.
- `verify_settlement_theorem(...)` stays one canonical public consistency check over the published bundle; `z00z_rollup_node` re-exports it and does not maintain a parallel verifier path.
