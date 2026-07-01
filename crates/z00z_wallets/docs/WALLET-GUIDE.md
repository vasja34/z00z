<!-- markdownlint-disable-file MD022 MD031 MD032 MD036 MD040 MD060 -->

# Z00Z Wallet Guide

Date: 2026-06-20
Scope: `crates/z00z_wallets`

This guide documents the canonical wallet model that is live through Phase 060.
It describes the normal `.wlt` reopen, save, export, restore, and typed-object
projection path only. Historical snapshot terminology is compatibility-only and
must not be treated as normal wallet authority.

## 📦 Phase 059 Object Inventory

Phase 059 extends the live wallet model with typed settlement objects:

- Assets: final spendable value.
- Vouchers: conditional claims that stay non-cash until a validator-checked
  redeem path succeeds.
- Rights: zero-value authority inventory.

The wallet therefore exposes three projections on one inventory plane:

- spendable cash assets;
- voucher claims and voucher lifecycle state;
- right authority inventory and right lifecycle state.

Unknown-policy objects remain in durable quarantine and are excluded from
spendable balance.

## 🧾 Phase 060 MVP Profile Catalog

Phase 060 publishes one repository-owned wallet profile catalog on top of the
live Phase 059 object model. The rows below are semantic contracts, not a claim
that every profile id is already a live code identifier. Unless a row says
otherwise, the profile id is a proposed Phase 060 catalog id layered on an
existing live repository anchor.

| Profile id | Phase 060 status | Family | Live anchor | `wallet.object.*` projection | `wallet.asset.*` effect | Lifecycle and spendability | MVP actions and policy surfaces | Required fail-closed rules |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `fee_credit_v1` | Proposed Phase 060 catalog id | Voucher | Existing voucher object model; product anchor: `FeeCredit` in `docs/Z00Z-Tokenomics-Incentives-Whitepaper.md` | Voucher claim row with terminal id, policy state, and backing metadata | Never appears as ordinary balance; fee capacity may be realized only through the voucher or fee-support path | `Offered -> Accepted -> Redeemable -> Redeemed/Expired/Refunded`; any non-`Available` row stays quarantined | issue, accept, transfer if policy allows, redeem for fee lane, expire, refund; backing source, sponsor scope, transferability, expiry, disclosure/retention as needed | Reject voucher-as-cash, missing backing, replay, stale root, expired use, and unknown policy |
| `service_entitlement_v1` | Proposed Phase 060 catalog id layered on a live right class | Right | Live `service_entitlement` in `crates/z00z_core/configs/devnet_rights_config.yaml` | Right inventory row with policy state, beneficiary scope, and lifecycle metadata | No cash visibility | `Granted -> Delegated/Consumed/Revoked/Expired`; only `Available` rows are usable | grant, delegate, consume, revoke, expire; disclosure policy, retention policy, provider scope, beneficiary scope | Reject right-as-value, out-of-scope use, revoked/expired right, and unknown policy |
| `data_access_v1` | Proposed Phase 060 catalog id layered on a live right class | Right | Live `data_access` in `crates/z00z_core/configs/devnet_rights_config.yaml` | Right inventory row with beneficiary-specific access state | No cash visibility | `Granted -> Challenged/Consumed/Revoked/Expired`; only `Available` rows are usable | grant, consume, challenge, revoke, expire; disclosure policy, retention policy, audit trail, challenge window | Reject expired access, challenge-window misuse, wrong beneficiary, and unknown policy |
| `agent_budget_v1` | Proposed Phase 060 catalog id layered on live right classes | Right | Live `machine_compute_capability` plus `one_time_agent_action` in `crates/z00z_core/configs/devnet_rights_config.yaml`; product anchor: `Agent spending envelope` in `docs/Z00Z-Litepaper.md` | Right inventory row describing bounded action or quota scope | No cash visibility; never becomes free balance | `Granted -> Delegated/Consumed/Revoked/Expired`; consumed or non-`Available` rows stay non-spendable | delegate, consume quota, revoke, expire; quota or amount bound, action whitelist, service scope, expiry | Reject over-budget action, unauthorized action family, consumed-right reuse, and unknown policy |
| `validator_mandate_lock_v1` | Proposed Phase 060 catalog id layered on a live right class and a wallet rule | Right plus asset-state rule | Live `validator_mandate` in `crates/z00z_core/configs/devnet_rights_config.yaml`; lock grammar source: `docs/tech-papers/TODO-Wallet-idea.md`; wallet-local profile tag: `validator_mandate_lock_v1` | Right inventory row plus lock metadata bound through `payload_commitment` | `wallet.asset.*` stays cash-only, but assets bound by an active lock are excluded from ordinary spend selection once the lock profile is present and bound | `Granted -> Locked -> Unlockable/Redelegatable/Revoked/Expired`; ordinary spend of the locked asset is forbidden until an approved unlock or redelegate transition consumes the lock | lock, unlock, redelegate, reward-claim, challenge-bounded revoke; holder, control, beneficiary, payload, time-window, transition, revocation, disclosure, and retention policy surfaces | Reject soft-lock-only behavior, ordinary spend of a locked asset, replay, stale right, and wrong-family proof |
| `transferable_claim_v1` | Proposed Phase 060 catalog id | Voucher | Existing Phase 059 voucher object model | Voucher claim row with transfer and residual state | No cash visibility until a valid redeem path finalizes | `Offered -> Accepted -> Redeemable -> Partial/Full Redeemed/Rejected/Expired`; any non-`Available` row stays quarantined | offer, accept, transfer if policy allows, partial redeem, redeem, reject, expire; backing/reserve source, accept policy, transferability, residual handling | Reject wrong-family proof, double redeem, expired use, residual mismatch, and unknown policy |

## 🗂️ Projection Grammar

The catalog above is implemented on one wallet authority plane with these
mandatory rules:

- `wallet.object.*` is the typed inventory and package-authority namespace for
  vouchers, rights, and any wallet-visible profile semantics.
- `wallet.asset.*` remains cash-only. It must not present vouchers or rights as
  spendable value and must not invent a second typed-object persistence story.
- `wallet_asset_store()` remains the only ordinary cash-persistence authority for asset rows. Non-asset profile rows remain on `WalletInventoryPayload`, while voucher/right writes stay on the explicit `OwnedNonAssetPayload` lanes.
- Any object whose policy availability is not `Available`, or that still
  requires manual review, remains in durable quarantine and is excluded from
  ordinary spendable balance.
- Unknown-policy objects remain visible only through typed inventory and stay in
  durable quarantine until an explicit known-`Available` policy verdict exists.
- Lock profiles are not UI-only suggestions. The catalog contract for
  `validator_mandate_lock_v1` requires ordinary asset selection to treat active
  locked assets as unavailable. This enforcement uses the live
  `RightClass::ValidatorMandate` leaf plus the wallet-local profile tag
  `validator_mandate_lock_v1`; it does not create a second wallet authority
  plane.
- `.wlt` and `WalletExportPack` remain the only wallet-local authority surfaces. Backup and restore must round-trip `owned_assets` and
  `owned_objects` together through the same canonical encrypted bundle plus the
  explicit JSONL tx-history sidecar.
- Debug, preview, or forensic tooling may inspect the same state, but it must
  not create a second wallet database, a second export bundle, or a second
  spendability truth path.

## 🔒 `validator_mandate_lock_v1` v1 grammar

- Profile identification:
  the wallet recognizes this proposed profile only when a right row keeps live
  `RightClass::ValidatorMandate`, policy availability `Available`, and the
  wallet-local profile tag `validator_mandate_lock_v1`.
- Mandatory leaf fields in v1:
  `holder_commitment`, `control_commitment`, `beneficiary_commitment`,
  `payload_commitment`, `valid_from`, `valid_until`, `challenge_from`,
  `challenge_until`, `use_nonce`, `transition_policy_id`,
  `revocation_policy_id`, `disclosure_policy_id`, and `retention_policy_id`.
- Mandatory payload binding in v1:
  `payload_commitment` binds the locked asset id, locked amount, validity
  window, challenge window, `use_nonce`, and the transition, revocation,
  disclosure, and retention policy ids.
- Optional or reserved in v1:
  `issuer_scope` and `provider_scope` may scope validator or pool lineage, but
  they do not replace payload binding; `challenge_policy_id` remains a
  challenge-bounded control surface and does not widen v1 into slashable bond
  logic.
- Spend rule:
  any `Granted`, `Held`, or `Delegated` row that matches the payload binding
  blocks ordinary asset selection. Passing `valid_until` makes the lock
  unlockable, not automatically cash-spendable.
- Approved transitions:
  ordinary unlock after expiry must consume or update the right through the
  approved unlock grammar; redelegate stays on the same right-family authority
  lane; reward claim remains tied to the same active lock lineage and may not
  materialize as free wallet cash without the right-gated transition.
- v1 non-goals:
  no new primitive leaf, no second wallet database, no second export contract,
  no UI-only soft lock, and no full slashable bond semantics.

## 🎯 Canonical Model

- `.wlt` is the canonical encrypted wallet database.
- `WalletProfilePayload` stores wallet profile metadata, verifier state,
  settings, and wallet lock state.
- `OwnedAssetPayload` stores wallet-owned asset state.
- `OwnedVoucherPayload` stores wallet-owned voucher state.
- `OwnedRightPayload` stores wallet-owned right state.
- `ScanStatePayload` stores the scan cursor and related receive progress.
- `KeysPayload`, `StealthMetaPayload`, and `TofuPinsPayload` store subsystem
  state that belongs inside the wallet boundary.
- Transaction history remains an explicit JSONL sidecar:
  `wallet_<stem>_tx_history.jsonl`.

## ⚙️ Normal Lifecycle

1. Create a wallet and initialize the `.wlt` file plus secret material.
2. Unlock the wallet and open a session-scoped `.wlt` handle.
3. Read `WalletProfilePayload` from `.wlt` and restore in-memory profile state.
4. Read live owned assets from `.wlt` object storage and install them into the
   service cache.
5. Keep scan state and tx history on their explicit planes instead of folding
   them into a single legacy blob.
6. Save by writing the updated wallet profile and any changed object payloads.
7. Export by building one canonical `WalletExportPack` and encrypting it.
8. Restore by validating the canonical pack, staging `.wlt`, and replaying the
   JSONL tx-history sidecar when present.

## 🔐 Persistence Boundaries

- Secrets stay in the dedicated secrets table.
- Profile, assets, scan state, and backup manifest data are encrypted object
  payloads inside `.wlt`.
- `recv_range(...)` and wallet-owned scanning remain the ownership authority.
- Export and restore use one canonical pack shape and one explicit tx-history
  sidecar plane.
- Debug helpers may inspect persisted state, but they must not reintroduce a
  second live authority model.
- Typed object inventory is additive: asset-only reopen remains valid, while
  vouchers/rights reuse the same `.wlt` object boundary instead of a second
  wallet database.

## 📦 Export And Restore Shape

The canonical `WalletExportPack` carries these live fields:

- `manifest`
- `wallet_profile`
- `owned_assets`
- `owned_objects`
- `scan_state`
- `stealth_meta`
- `tofu_pins`
- `keys`
- `tx_history_plane`
- `seed_phrase`
- `wallet_identity`

Normal wallet-state transfer must treat this explicit shape as the only live
bundle contract.

`export_wallet_payload` and `import_wallet_payload` move the canonical wallet
state pack only.

`create_backup` and `restore_backup_with_mode` are the surfaces that carry and
replay the explicit JSONL tx-history sidecar.

## 🕵️ Privacy And Disclosure Boundaries

- `tag16` prefilter and wallet-owned stealth scan detection are wallet-local
  receive primitives only. They prove receive classification, not transport
  anonymity.
- `WalletReveal::{Present, Redacted, Unavailable}` defines the public
  disclosure matrix for wallet-owned receive data. Public DTO, report, and log
  surfaces must keep memo plaintext, receiver secrets, blindings, output
  secrets, and private scan keys redacted.
- Backup metadata, package verify or import or export reports, and RPC logging
  summaries may expose only bounded public fields such as wallet id, package
  version, digests, counts, lifecycle, and status. They must not expose raw
  package bytes, session tokens, seed phrases, memo plaintext, receiver
  secrets, or encrypted payload internals.
- Stealth pack privacy is a wallet and crypto receive property. It is not an
  OnionNet claim, and Phase 062 does not claim live transport anonymity.

## 🛰️ Typed RPC And Package Boundary

- `wallet.asset.*` remains cash-only.
- `wallet.object.*` is the typed object namespace for inventory, preview/build,
  and voucher/right lifecycle operations.
- Wallet package building binds policy descriptors, selected action, required
  rights, typed deltas, roots, and fee-support data through one shared runtime
  object package shape.
- Wallet builders must reject voucher-as-cash, right-as-value, out-of-scope
  rights, expired rights, revoked rights, consumed rights, and unknown-policy
  spend attempts.

## ✅ Phase 062 Bounded Closeout

Phase 062 closes the bounded internal object-family lane only.

- Included live scope: `RightLeaf`, `VoucherLeaf`, `RightClass`,
  `FeeEnvelope`, the object policy registry, wallet object inventory, validator
  fail-closed checks, deterministic local voucher/right scenarios, and
  cash/object separation proofs.
- Excluded scope: external chain trust tiers, linked liability, live
  cross-chain settlement, and any claim that vouchers or rights are ordinary
  wallet cash.
- Canonical evidence anchors:
  `crates/z00z_runtime/validators/tests/test_object_policy_verdicts.rs`,
  `crates/z00z_simulator/tests/scenario_1/test_scenario1_object_flows.rs`,
  `crates/z00z_wallets/src/rpc/test_asset_impl.rs`, and
  `crates/z00z_wallets/tests/test_asset_import_security.rs`.
- Closure rule: `wallet.object.*` remains the typed object namespace, while
  `wallet.asset.*` remains the cash-only authority even when the shared
  inventory view projects asset rows beside vouchers or rights.

## 🧭 Key Runtime Surfaces

- `services/wallet_store_open_source.rs`
  unlocks and restores the profile-first live wallet state.
- `services/wallet_store_restore.rs`
  loads wallet profile bytes and owned assets from `.wlt`.
- `services/wallet_store_persistence_pack.rs`
  defines wallet file naming and the explicit JSONL history path.
- `services/wallet_store_transfer_import.rs`
  encrypts and decrypts the canonical export payload.
- `services/wallet_actions_backup.rs`
  stages and publishes canonical restore state.
- `redb_store/debug_export.rs`
  exports on-disk debug state from the actual `.wlt` tables and objects.

## 🧪 Test Anchors

The canonical path should stay anchored by tests that prove:

- profile-only reopen succeeds without any legacy snapshot dependency;
- save keeps owned assets on the explicit object path;
- export and restore operate on `WalletExportPack` plus JSONL history only;
- duplicate owned-asset payloads in a restore pack fail closed.

## 🚫 Compatibility Notes

- `WalletPersistenceState` is not the normal live save, reopen, export, or
  restore contract.
- Legacy snapshot wording is historical only.
- `claimed_assets` must not be described as a second normal authority plane for
  reopen or export.
- If compatibility decode surfaces remain anywhere, they must fail closed and
  stay outside the normal live flow.

## ⭐ Practical Guidance

- When adding new wallet state, prefer explicit `.wlt` payload objects and
  explicit indexes.
- When changing export or restore behavior, keep `WalletExportPack` and backup
  manifest validation aligned in the same patch.
- When changing tx history behavior, preserve the explicit JSONL sidecar until a
  separate planned migration lands.
- When updating docs or tests, describe only the canonical profile-first plus
  owned-asset plus scan-state plus JSONL history story.
