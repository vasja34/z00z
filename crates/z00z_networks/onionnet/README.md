# OnionNet

## Role

OnionNet is the node-owned privacy overlay that sits before Phase 100 ingress.
It is not a transport alias for RPC, not a separate application service, and
not a place for sequencer or validator logic.

This crate-shaped placeholder seam exists so the repository already matches the
Phase 115 namespace and module layout before live overlay code lands.

This placeholder crate exists so future OnionNet work lands in the same
namespace and module layout already described by
`.planning/phases/115-onionnet/115-OnionSpec.md`.

## Boundary

- `z00z_networks_rpc` owns request or response RPC transport only.
- `onionnet` owns privacy-ingress transport concerns such as bridge admission,
  link protection, path privacy, replay filtering, and exit handoff into the
  canonical runtime ingress.
- Phase 120 remains the composition root for node wiring and operator control
  plane integration.
- Wallets may select OnionNet as a transport mode, but wallet code does not own
  OnionNet routing, relay, replay, or exit semantics.

## Placeholder Rule

The placeholder must stay crate-shaped and module-shaped so later
implementation fills modules in place instead of moving files or redesigning
the namespace.

Reserved modules:

- `config`
- `identity`
- `bootstrap`
- `transport_quic`
- `link_crypto`
- `packet`
- `sphinx_path`
- `session`
- `bridge_api`
- `edge`
- `relay`
- `exit`
- `telemetry`

## Naming

Use `OnionNet` for new crate and module documentation. Historical `onionet`
wallet or RPC names are a migration concern and must not define new crate
boundaries.
