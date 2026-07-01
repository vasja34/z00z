# Phase 031 Retirement Record

## Purpose

This file records the Phase 031 retirement decisions for shims, suffixes, provisional exports, and remaining migration-only exceptions.
Every retired seam below is tied back to the Wave 0 inventory so closeout stays caller-proof-backed instead of assumption-driven.

## Retired Or Demoted Seams

| Inventory Row | Change | Evidence | Caller-Proof Basis | Rollback Condition |
| --- | --- | --- | --- | --- |
| `INV-CORE-001` | Removed the wildcard root export `pub use assets::*` from `z00z_core` and replaced it with curated root exports. | [031-02-SUMMARY.md](/home/vadim/Projects/z00z/.planning/phases/031-refactor-architecture/031-02-SUMMARY.md) | `031-INVENTORY.md` recorded broad root callers and scoped the Wave 1 cleanup to in-tree migrated imports. | Roll back if root callers still require omitted domain exports or if a new wildcard export is reintroduced to fix regressions. |
| `INV-CRYPTO-001` | Demoted Tari concrete leakage out of the stable `z00z_crypto` root into an explicit vendor lane. | [031-03-SUMMARY.md](/home/vadim/Projects/z00z/.planning/phases/031-refactor-architecture/031-03-SUMMARY.md) | `031-INVENTORY.md` and `031-IMPORT-GRAPH.md` identified Tari leakage as a root-facade ownership issue, not a required stable contract. | Roll back if downstream callers require vendor concrete types through the stable root to preserve non-vendor API behavior. |
| `INV-CRYPTO-004` | Kept nonce-controlled AEAD helpers off the non-test stable facade and behind explicit test-only gating. | [031-03-SUMMARY.md](/home/vadim/Projects/z00z/.planning/phases/031-refactor-architecture/031-03-SUMMARY.md) | Wave 0 marked the seam migration-only and Phase 031 treated non-test reachability as a blocker. | Roll back if the gating change breaks test-only callers that cannot be migrated within the test profile. |
| `INV-WLT-001` | Retired the `include!`-assembled wallet service root in favor of explicit service-module ownership. | [031-05-SUMMARY.md](/home/vadim/Projects/z00z/.planning/phases/031-refactor-architecture/031-05-SUMMARY.md) | Wave 0 flagged the flat assembled service surface as non-canonical and provisional. | Roll back if explicit module composition cannot preserve the existing stable wallet service facade. |
| `INV-WLT-004` | Demoted transport-facing wallet DTO ownership away from the stable wallet root and kept it adapters-owned. | [031-07-SUMMARY.md](/home/vadim/Projects/z00z/.planning/phases/031-refactor-architecture/031-07-SUMMARY.md) | Wave 0 caller proof showed DTOs and dispatcher wiring were edge contracts, not stable root-wallet contracts. | Roll back if wallet root callers still require those DTO exports as stable public imports. |
| `INV-SIM-003` | Removed the default-public Stage 2 plaintext secret artifact lane and kept any retained artifact debug-only, private-path-only, and private-permission-only. | [031-09-SUMMARY.md](/home/vadim/Projects/z00z/.planning/phases/031-refactor-architecture/031-09-SUMMARY.md) | Wave 0 marked `wlt_secrets_debug.md` as a provisional unsafe output, and Wave 3 closed it with explicit gated tests. | Roll back only if the gated debug lane proves insufficient for required local troubleshooting and a safer replacement cannot be added. |
| `INV-SIM-004` | Retired the unsandboxed `reset_outputs_dir()` behavior by making sandbox validation a fail-closed precondition. | [031-09-SUMMARY.md](/home/vadim/Projects/z00z/.planning/phases/031-refactor-architecture/031-09-SUMMARY.md) | Wave 0 flagged recursive delete outside an approved output root as a hardening blocker. | Roll back only if the sandbox allowlist is incomplete and blocks legitimate simulator-owned output roots. |

## Remaining Explicit Exceptions

| Inventory Row | Owning File | Status | Why It Remains | Why It Is Not A Default Stable-Facade Violation |
| --- | --- | --- | --- | --- |
| `INV-CRYPTO-003` | `crates/z00z_crypto/src/ecdh_stealth.rs` | migration-only compatibility lane remains explicit | The stealth compatibility helpers remain documented as compatibility-only and were not retired in this phase. | The lane is explicitly named as compatibility-oriented in Wave 0 and is tracked as a bounded exception rather than an unlabelled stable root surface. |
| `INV-WLT-005` | `crates/z00z_wallets/src/adapters/rpc/methods/key.rs` and related dispatcher wiring | canonical live contract | Wave 031-07 disposition made `wallet.key.export_public_material_v2` an explicit live RPC contract. | It remains named and documented intentionally at the adapters edge, not as an accidental default-public stable wallet root alias. |
| `INV-WLT-006` | `crates/z00z_wallets/src/core/chain/receiver_card_record.rs` | canonical live contract | Wave 031-07 disposition kept `ReceiverCardRecordV1` as an explicit live compatibility-named type used by edge conversion helpers. | Its continued use is explicit and documented as a live contract rather than an unreviewed leftover. |
| `D-25` boundary note | `crates/z00z_utils/src/time/traits.rs` | explicit compatibility helper family remains | `compat_unix_timestamp*()` stays as the documented compatibility-only time surface. | The helpers are already marked compatibility-only in code and are not hidden as default security-sensitive APIs. |

## Fresh Wave 4 Grep Outcome

- [031-10-pub-surface-grep.log](/home/vadim/Projects/z00z/.planning/phases/031-refactor-architecture/logs/031-10-pub-surface-grep.log) did not surface any uncovered default-public `compat`, `legacy`, or `shim` re-export lane.
- [031-10-compat-grep.log](/home/vadim/Projects/z00z/.planning/phases/031-refactor-architecture/logs/031-10-compat-grep.log) still reports explicit compatibility helpers and versioned protocol constants across the workspace, but the surviving default-public items relevant to Phase 031 are the bounded exceptions listed above.
- The fresh max-safe and broad workspace release reruns both completed green, so none of the retained exceptions require a rollback classification for this closeout.

## Closeout Rule

No additional shim or suffix retirement is claimed beyond the summary-backed Wave 1 through Wave 3 changes listed above.
If a future grep gate surfaces a default-public compat, legacy, shim, or version-suffixed stable facade that is not covered by the explicit exceptions table, Phase 031 must be reopened or the later phase must stay open until that surface is either retired or reclassified with caller-proof-backed evidence.
