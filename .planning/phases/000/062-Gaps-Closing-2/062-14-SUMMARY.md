---
phase: 062-Gaps-Closing-2
plan: 062-14
status: complete
completed_at: 2026-06-26
next_plan: 062-15
summary_artifact_for: .planning/phases/062-Gaps-Closing-2/062-14-PLAN.md
---

# 062-14 Summary: Genesis Manifest Split And Referenced Loader

## Outcome

`062-14` is complete. The repository now has one canonical live
`GenesisConfig` authority path built from a root
`genesis_config_devnet.yaml` plus `manifest_refs` for `assets`, `rights`,
`policies`, and `vouchers`, and the release gate is green on the current tree.

The live loader now rehydrates the same `GenesisConfig` shape from the split
manifest layout, rejects duplicate inline-plus-ref sections, rejects reused
ref paths and parent traversal, and keeps the `actions_config.yaml` decision
explicit and fail-closed. The devnet fixture is split into one root manifest
plus four sibling section files, the genesis docs describe that layout as the
live bootstrap authority, and the compatibility `assets_config.yaml` comments
no longer read like a second production bootstrap plane.

Two real broad-gate blockers were also closed before plan completion. First,
the root-manifest rollout initially drifted the canonical devnet rights corpus:
`genesis_config_devnet_assets.yaml` was restored to the approved live
`domain_name` string `z00z.core.assets.native_coin.devnet.phase059`, and the
approved deterministic genesis-rights snapshot in
`crates/z00z_core/tests/genesis/test_genesis_rights_manifest.json` was updated
to the current live `genesis_config_devnet_small.yaml` contract. Second, the
broad rerun exposed two separate dirty-tree guardrails outside the manifest
slice: the phase-source heading in
`.planning/phases/Z00Z-IMPL-PHASES.md` and a position-sensitive asset-fixture
assumption in `crates/z00z_wallets/src/rpc/test_asset_impl.rs`. The phase
source now explicitly carries the live section `0` settlement-root/HJMT
authority wording, and the wallet tests now select seed assets by
`AssetClass` instead of relying on unstable catalog positions.

With those blockers closed, the mandatory bootstrap gate is green, the focused
genesis/storage/wallet release reruns are green, and the final broad
`cargo test --release` rerun is green. The active execution lane advances to
`062-15`.

## Files Changed

- `crates/z00z_core/src/genesis/genesis_config.rs`
- `crates/z00z_core/src/genesis/genesis_config_validate.rs`
- `crates/z00z_core/src/genesis/manifest_ref_loader.rs`
- `crates/z00z_core/src/genesis/genesis_config_devnet.yaml`
- `crates/z00z_core/src/genesis/genesis_config_devnet_assets.yaml`
- `crates/z00z_core/src/genesis/genesis_config_devnet_rights.yaml`
- `crates/z00z_core/src/genesis/genesis_config_devnet_policies.yaml`
- `crates/z00z_core/src/genesis/genesis_config_devnet_vouchers.yaml`
- `crates/z00z_core/src/genesis/README.md`
- `crates/z00z_core/src/assets/assets_config.yaml`
- `crates/z00z_core/tests/test_genesis_manifest_refs.rs`
- `crates/z00z_core/tests/test_genesis_manifest_goldens.rs`
- `crates/z00z_core/tests/genesis/test_genesis_rights_manifest.json`
- `crates/z00z_core/tests/genesis/test_genesis_state_verification.rs`
- `crates/z00z_wallets/src/rpc/test_asset_impl.rs`
- `.planning/phases/Z00Z-IMPL-PHASES.md`
- `.planning/phases/062-Gaps-Closing-2/062-14-SUMMARY.md`
- `.planning/STATE.md`
- `.planning/ROADMAP.md`

## Validation

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `cargo test --release -p z00z_core test_genesis_manifest_phase059_fixture -- --nocapture`
- `cargo test --release -p z00z_core test_genesis_manifest_refs -- --nocapture`
- `cargo test --release -p z00z_core test_genesis_manifest_goldens -- --nocapture`
- `cargo test --release -p z00z_core --features deterministic-rng --test genesis_tests test_genesis_rights_deterministic -- --nocapture`
- `cargo test --release -p z00z_storage --test test_live_guardrails test_phase0_source_promotes_live_settlement_authority -- --nocapture`
- `cargo test --release -p z00z_wallets --lib test_asset_merge_assets_total -- --nocapture`
- `cargo test --release -p z00z_wallets --lib test_asset_split_sum_equals -- --nocapture`
- `cargo test --release -p z00z_wallets --lib test_asset_stake_echo_id -- --nocapture`
- `cargo test --release -p z00z_wallets --lib test_asset_unstake_roundtrip_surface -- --nocapture`
- `cargo test --release`
- `rg -n "manifest_refs:|\\brefs:\\b|actions_config\\.yaml|native_coin\\.devnet(\\.phase059)?|genesis_config_devnet_(assets|rights|policies|vouchers)\\.yaml" crates/z00z_core/src/genesis crates/z00z_core/src/assets crates/z00z_core/tests -S`

Result:

- `bootstrap_tests.sh` reran green after the `domain_name` fix.
- The manifest-ref release tests stayed green on the live root-manifest path.
- The deterministic genesis-rights snapshot now matches the live
  `genesis_config_devnet_small.yaml` contract.
- The phase-source guardrail is green after restoring the explicit live section
  `0` heading in `Z00Z-IMPL-PHASES.md`.
- The wallet asset RPC release regressions are green after replacing the
  position-sensitive seed-asset lookup with class-based lookup.
- The final broad `cargo test --release` rerun completed green on the current
  tree.

## Manual Review Passes

Because `/GSD-Review-Tasks-Execution` is not callable as a tool here, the
required review loop was executed manually against the same scope.

- Pass 1
  - Reviewed the initial `062-14` loader, docs, and test packet.
  - Result: found and fixed duplicate manifest-test surfaces plus the raw YAML
    consumer in `test_genesis_state_verification.rs` that bypassed the new
    loader path.
- Pass 2
  - Re-read the current devnet root-manifest split and the deterministic
    genesis-rights corpus after the first broad rerun failed.
  - Result: identified the wrong `domain_name` drift and the approved snapshot
    mismatch against the live `genesis_config_devnet_small.yaml` fixture; both
    were reconciled.
- Pass 3
  - Audited the new broad storage guardrail failure against
    `crates/z00z_storage/tests/test_live_guardrails.rs`,
    `.planning/phases/Z00Z-IMPL-PHASES.md`, and the phase-source rows in the
    Phase 062 packet.
  - Result: restored the explicit live section `0` heading and settlement-root
    wording on the phase source.
- Pass 4
  - Audited the second broad wallet failure cluster in
    `crates/z00z_wallets/src/rpc/test_asset_impl.rs`.
  - Result: identified that the live asset catalog is canonically ordered by
    `asset_id`, so the tests' `ids[1]` token assumption was invalid; rewrote
    the fixture lookup to select by `AssetClass`, then reran the four failing
    release tests green.
- Pass 5
  - Re-read the complete final diff set across genesis loader, approved
    snapshot, phase-source guardrail, and wallet asset RPC tests; reran the
    focused release validations.
  - Result: clean.
- Pass 6
  - Reran the mandatory bootstrap gate and the final full
    `cargo test --release`.
  - Result: clean.

Passes 5 and 6 were consecutive clean review runs for the actual `062-14`
scope.

## Task Status

- `TASK-076`
  - Implemented on the one canonical `GenesisConfig` authority path. The live
    devnet bootstrap fixture is now a root manifest that still rehydrates into
    the same config shape.
- `TASK-077`
  - Implemented by splitting `genesis_config_devnet.yaml` into one root
    manifest plus `assets`, `rights`, `policies`, and `vouchers` sibling
    section files.
- `TASK-078`
  - Implemented by the referenced loader and validator guards for section-key
    allowlisting, duplicate inline plus ref rejection, ref-path reuse
    rejection, and parent-traversal rejection.
- `TASK-082`
  - Implemented by the new release tests for root-manifest loading, golden
    layout, duplicate errors, path errors, and bad-section-shape failures.
- `TASK-083`
  - Implemented by the updated genesis README and compatibility
    `assets_config.yaml` comments that now distinguish the compatibility fixture
    from the live production authority path.
- `TASK-084`
  - Implemented as an explicit fail-closed decision: actions stay nested under
    policies and `actions_config.yaml` is intentionally unsupported.
