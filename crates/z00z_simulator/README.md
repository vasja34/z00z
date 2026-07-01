# Z00Z Simulator
<!-- markdownlint-disable MD012 -->

## 🎯 Simulator Boundary

`z00z_simulator` is the repository integration harness for end-to-end scenario execution.
It is allowed to orchestrate wallets, storage, network transport, and scenario artifacts,
but it must enter those crates through stable facades instead of deep implementation paths.

## 📌 Admission Policy

Code belongs in `z00z_simulator` when it is needed to:

- execute reproducible scenario flows across multiple crates;
- assert scenario-stage artifact contracts and release-style validation behavior;
- host harness-only helpers such as deterministic fixture wiring or sandboxed output setup.

Code does not belong in `z00z_simulator` when it would:

- create a second owner for wallet, storage, crypto, or network business rules;
- depend on deep `services`, `db`, or `store_internal` seams that a production caller would not use;
- normalize plaintext secret handling or filesystem cleanup behavior that production code must keep fail-closed.

## ⚙️ Stable Facades

Scenario code must prefer stable facades from the owning crate. If a simulator change needs a new
entrypoint, add or document the facade in the owner crate instead of reaching into private wiring.

The simulator may keep harness-specific adapters, but those adapters must stay narrow, documented,
and test-guarded so they do not become the accidental public framework for other crates.

## 🔐 Secret Artifact Policy

Plaintext wallet-secret artifacts are not part of the default public scenario contract.
If a debug-only secret artifact is retained for local troubleshooting, it must be:

- behind the `wallet_debug_tools` feature gate;
- written to a private-permission path;
- absent from the default release-style stage contract.

The remaining fix set is to keep that debug-only lane confined through explicit
wrapping and retention policy without reopening the hardened default lane.

Encrypted operational export and backup surfaces remain outside this narrower plaintext-debug-artifact claim.

## 🧪 Scenario Contract

New scenarios must reuse the existing scenario contract rather than copying `scenario_1` internals ad hoc:

- stage entrypoints live under `src/scenario_*` and are executed through the runner design contract;
- public artifacts must be described in the stage surface tests and design YAML;
- output reset logic must stay inside the approved simulator sandbox roots.

Scenario 1 output-secret semantics are also frozen at the simulator boundary:
use the deterministic `derive_s_out(k_dh, r_pub, serial_id)` contract from the
wallet stealth facade, and do not describe any competing output-secret model as
canonical simulator behavior.

When a scenario introduces a new boundary rule, add it here or in a matching architecture guard test
in the same wave so the simulator remains an integration harness instead of a megacrate.

## 📦 Phase 059 Object Lanes

`scenario_1` is the canonical executable home for the Phase 059 object model.
It was extended in place rather than replaced.

The live release packet now covers:

- asset transfer lanes;
- voucher issue/offer/accept/reject/transfer/redeem/refund/expiry lanes;
- right grant/delegate/consume/revoke/expiry/challenge lanes;
- right-gated voucher actions and fee-support boundaries;
- positive and negative Alice/Bob/Charlie handoff evidence.

The object-flow contract lives on one `object_flow_matrix` in
`src/scenario_1/scenario_config.yaml`. Public artifact anchors include:

- `asset_flow.json`
- `voucher_flow.json`
- `right_flow.json`
- `wallet_scan.json`
- `val_flow.json`
- `watch_flow.json`
- `sim_summary.md`

`asset_flow.json`, `voucher_flow.json`, and `right_flow.json` are canonical
public evidence anchors and ship on the emitted release-packet lane alongside
`wallet_scan.json`, `val_flow.json`, `watch_flow.json`, and `sim_summary.md`.

Negative simulator evidence is mandatory. The Phase 059 packet records reject
and fix surfaces for unknown policy, invalid backing, missing/expired/replayed
rights, double redeem, wrong-family proof, and forced voucher acceptance.
