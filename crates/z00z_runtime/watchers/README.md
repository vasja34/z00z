# z00z_runtime watchers

This crate is the observation and alert surface over publication, verdict, and
DA-provider health signals.

## Canonical modules

- `engine`: `WatcherInput`, `WatcherService`, and `WatcherBoundary`.
- `publication`, `status`, `alerts`, `evidence_export`, `censorship`, `provider`, and `da_health`: operational observation, export, and alerting surfaces.

## Boundaries

- `WatcherService` consumes already-published runtime state and emits `ObservationSnapshot`.
- `WatcherBoundary` prefers the placement carried by `ShardExecTicket` when exec metadata exists; placement remains observational data, not semantic truth.
- Publication route acceptance stays storage-owned: watcher publication checks may project `PublicationBinding`, but exact route-table coverage must come from the committed `PublicationRouteSnapshotV1` carried with the published batch.
- This crate does not own planner authority, validator verdict rules, or settlement semantics.

## Phase 059 alert classes

Phase 059 extends watcher evidence from publication-only health toward typed
object verdict observation.

The live watcher alert surface must expose at least these reject families:

- unknown policy usage;
- invalid voucher backing;
- wrong-family proof attempts;
- replay and duplicate redemption;
- expired object use;
- acceptance/refund boundary violations;
- rights used as value or otherwise crossing the object-role boundary.

Watcher output is evidence and alerting only. It does not create a second
semantic authority beside validators and storage.
