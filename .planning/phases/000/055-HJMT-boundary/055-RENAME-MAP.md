# Phase 055 Family Rename Map

## Purpose

This document freezes the exact rename boundary for the storage settlement
benchmark harness family. It does not authorize a blanket rename of real
asset-domain vocabulary.

As of 2026-06-11 the live worktree already resolves the family-level targets
below. This map exists so any follow-on rename pass can distinguish:

- harness-family names that must converge on `settlement_*`;
- workload and fixture names that must stay `asset/right` because they describe
  real leaf semantics; and
- historical references that must not be rewritten inside Phase 055.

## Rename Scope

### Canonical family-level rename set

| Old surface | Canonical target | Kind | Live authority anchors | Rule |
| --- | --- | --- | --- | --- |
| `assets_hjmt` | `settlement_hjmt` | bench target and bench home | `crates/z00z_storage/Cargo.toml:49`, `crates/z00z_storage/benches/settlement_hjmt.rs:7`, `crates/z00z_storage/benches/settlement_hjmt.rs:1122`, `crates/z00z_storage/benches/settlement_benches.md:19`, `crates/z00z_storage/benches/settlement_benches.md:55` | Family-level harness name must stay `settlement_hjmt` everywhere in live code, docs, commands, and meta files. |
| `assets_proofs` | `settlement_proofs` | bench target and bench home | `crates/z00z_storage/Cargo.toml:53`, `crates/z00z_storage/benches/settlement_proofs.rs:13`, `crates/z00z_storage/benches/settlement_proofs.rs:2152`, `crates/z00z_storage/benches/settlement_benches.md:20`, `crates/z00z_storage/benches/settlement_benches.md:61` | The proof harness family is settlement-wide, not asset-only. |
| `assets_nested` | `settlement_nested` | bench target and bench home | `crates/z00z_storage/Cargo.toml:45`, `crates/z00z_storage/benches/settlement_nested.rs:6`, `crates/z00z_storage/benches/settlement_nested.rs:130`, `crates/z00z_storage/benches/settlement_benches.md:21`, `crates/z00z_storage/benches/settlement_benches.md:63` | Nested harness ownership is settlement-wide. |
| `assets_shard` | `settlement_shard` | bench target and bench home | `crates/z00z_storage/Cargo.toml:41`, `crates/z00z_storage/benches/settlement_shard.rs:6`, `crates/z00z_storage/benches/settlement_shard.rs:249`, `crates/z00z_storage/benches/settlement_benches.md:22`, `crates/z00z_storage/benches/settlement_benches.md:64` | Shard harness ownership is settlement-wide. |
| `assets_benches.md` | `settlement_benches.md` | tracked benchmark evidence home | `crates/z00z_storage/benches/settlement_benches.md:1`, `crates/z00z_storage/tests/test_bench_lanes.rs:11` | One canonical evidence index. |
| `assets_bench_support` | `settlement_bench_support` | fixture-support module | `crates/z00z_storage/src/fixture_support/mod.rs:4`, `crates/z00z_storage/benches/settlement_hjmt.rs:8`, `crates/z00z_storage/benches/settlement_proofs.rs:17` | Module family name must stay settlement-wide even when it exposes `asset_*` helpers. |
| `assets_bench_output` | `settlement_bench_output` | side-output policy module | `crates/z00z_storage/src/fixture_support/mod.rs:3`, `crates/z00z_storage/src/fixture_support/settlement_bench_output.rs:9`, `crates/z00z_storage/tests/test_bench_lanes.rs:13` | Output policy is harness infrastructure, not an asset-domain API. |
| `run_storage_assets_bench.py` | `run_storage_settlement_bench.py` | primary bench runner | `crates/z00z_storage/scripts/run_storage_settlement_bench.py:16`, `crates/z00z_storage/scripts/run_storage_settlement_bench.py:26`, `crates/z00z_storage/benches/settlement_benches.md:6`, `crates/z00z_storage/tests/test_bench_lanes.rs:12` | One canonical runner for all settlement harness homes. |
| `run_storage_assets_nested_bench.sh` | `run_storage_settlement_nested_bench.sh` | wrapper script | `crates/z00z_storage/scripts/run_storage_settlement_nested_bench.sh:1` | Companion wrapper must follow the same family rename. |
| `run_storage_assets_shard_bench.sh` | `run_storage_settlement_shard_bench.sh` | wrapper script | `crates/z00z_storage/scripts/run_storage_settlement_shard_bench.sh:1` | Companion wrapper must follow the same family rename. |
| `outputs/assets` | `outputs/settlement` | runtime output root | `crates/z00z_storage/src/fixture_support/settlement_bench_output.rs:89`, `crates/z00z_storage/src/fixture_support/settlement_bench_output.rs:136`, `crates/z00z_storage/scripts/run_storage_settlement_bench.py:16`, `crates/z00z_storage/tests/test_bench_lanes.rs:319` | Live benchmark outputs must publish under one settlement root. |
| `assets_hjmt_*` report basenames | `settlement_hjmt_*` report basenames | output note/report family | `crates/z00z_storage/benches/settlement_benches.md:55`, `crates/z00z_storage/benches/settlement_hjmt.rs:1116` | Report basenames are part of the harness identity and must follow the family rename. |
| `assets_proofs.md`, `assets_proofs_batch.md`, `assets_proof_sizes.md` | `settlement_proofs.md`, `settlement_proofs_batch.md`, `settlement_proof_sizes.md` | proof report family | `crates/z00z_storage/benches/settlement_benches.md:61`, `crates/z00z_storage/benches/settlement_benches.md:62`, `crates/z00z_storage/benches/settlement_proofs.rs:1851`, `crates/z00z_storage/benches/settlement_proofs.rs:2074`, `crates/z00z_storage/tests/test_bench_lanes.rs:620` | The proof report home is settlement-wide even though individual lanes still use `asset/right` leaf labels. |
| `assets_nested.md`, `assets_nested_reload.md` | `settlement_nested.md`, `settlement_nested_reload.md` | nested report family | `crates/z00z_storage/benches/settlement_benches.md:63`, `crates/z00z_storage/benches/settlement_nested.rs:124` | Keep nested evidence under the settlement family prefix. |
| `assets_shard.md`, `assets_shard_recovery.md` | `settlement_shard.md`, `settlement_shard_recovery.md` | shard report family | `crates/z00z_storage/benches/settlement_benches.md:64`, `crates/z00z_storage/benches/settlement_shard.rs:243` | Keep shard evidence under the settlement family prefix. |
| `Z00Z_STORAGE_ASSET_BENCH_KEEP` | `Z00Z_STORAGE_SETTLEMENT_BENCH_KEEP` | output-preservation env surface | `crates/z00z_storage/scripts/run_storage_settlement_bench.py:20`, `crates/z00z_storage/src/fixture_support/settlement_bench_output.rs:10` | Infrastructure env vars must match the settlement harness family. |
| `Z00Z_ASSET_PROOF_NOTE_SCOPE`, `Z00Z_ASSET_PROOF_NOTE_COMMAND`, `Z00Z_ASSET_PROOF_NOTE_FILTER` | `Z00Z_SETTLEMENT_PROOF_NOTE_SCOPE`, `Z00Z_SETTLEMENT_PROOF_NOTE_COMMAND`, `Z00Z_SETTLEMENT_PROOF_NOTE_FILTER` | proof note env surface | `crates/z00z_storage/scripts/run_storage_settlement_bench.py:21`, `crates/z00z_storage/src/fixture_support/settlement_bench_output.rs:11` | Batch-note scope is settlement harness infrastructure, not domain vocabulary. |
| `Z00Z_ASSET_BENCH_MODE`, `Z00Z_ASSET_ROOT_MODE`, `Z00Z_ASSET_BASELINE` | `Z00Z_SETTLEMENT_BENCH_MODE`, `Z00Z_SETTLEMENT_ROOT_MODE`, `Z00Z_SETTLEMENT_BASELINE` | helper option env surface | `crates/z00z_storage/scripts/run_storage_settlement_bench.py:159`, `crates/z00z_storage/src/fixture_support/settlement_bench_output.rs:49`, `crates/z00z_storage/src/fixture_support/settlement_bench_output.rs:50`, `crates/z00z_storage/src/fixture_support/settlement_bench_output.rs:51` | No mixed asset/settlement env plane is allowed in the live harness. |
| `storage-assets-bench-output-v1` | `storage-settlement-bench-output-v1` | managed output fingerprint | `crates/z00z_storage/src/fixture_support/settlement_bench_output.rs:9` | Managed-root identity must match the settlement output root. |

### Keep: workload and domain vocabulary that must not be renamed in this wave

These names describe real workload semantics, real fixture shapes, or real
asset-domain objects. They are not family-level harness markers.

| Surface to keep | Why it stays | Anchors |
| --- | --- | --- |
| `inclusion_asset`, `inclusion_right`, `deletion_right` | They label the exact proof family or leaf workload under measurement. | `crates/z00z_storage/benches/settlement_proofs.rs:804`, `crates/z00z_storage/benches/settlement_proofs.rs:813`, `crates/z00z_storage/benches/settlement_proofs.rs:822` |
| `mixed_asset_right` | It names a mixed terminal/right nested workload, not the harness family. | `crates/z00z_storage/benches/settlement_nested.rs:39`, `crates/z00z_storage/benches/settlement_benches.md:82` |
| `asset_batch`, `right_batch` | They describe the actual batch contents under test. | `crates/z00z_storage/benches/settlement_shard.rs:100`, `crates/z00z_storage/benches/settlement_shard.rs:133`, `crates/z00z_storage/benches/settlement_benches.md:74` |
| `single_asset`, `single_right_fee`, `single_asset_delete` | They describe exact CRUD workloads. | `crates/z00z_storage/benches/settlement_hjmt.rs:354`, `crates/z00z_storage/benches/settlement_hjmt.rs:367`, `crates/z00z_storage/benches/settlement_hjmt.rs:503` |
| `asset_seed`, `asset_item`, `asset_path`, `hot_assets`, `hot_bucket_assets` | These helpers construct real terminal-asset fixtures and paths. | `crates/z00z_storage/src/fixture_support/settlement_bench_support.rs:83`, `crates/z00z_storage/src/fixture_support/settlement_bench_support.rs:149`, `crates/z00z_storage/benches/settlement_shard.rs:15` |
| `AssetSeed`, `fixture.assets`, `z00z_core::assets::*` | These are real asset-domain types or fixture fields, not bench-family wrappers. | `crates/z00z_storage/tests/test_bench_lanes.rs:17`, `crates/z00z_storage/benches/settlement_proofs.rs:10`, `crates/z00z_storage/benches/settlement_hjmt.rs:171` |
| `asset_impl`, wallet asset surfaces, simulator asset generation, and core asset registry names | They belong to the product domain and sit outside the storage harness-family rename boundary. | `crates/z00z_wallets/src/adapters/rpc/methods/asset_impl.rs`, `crates/z00z_simulator/src/scenario_1/stage_1.rs:168`, `crates/z00z_core` asset modules |

### Defer: historical or non-live references that Phase 055 must not rewrite

| Surface to defer | Reason |
| --- | --- |
| Historical planning and summary references under `.planning/phases/000/**` that still mention `assets_*` bench homes | They are historical evidence of prior phase state and should not be rewritten as if history always used the new names. |
| Older completed rows in `.planning/STATE.md` and `.planning/ROADMAP.md` that quote `assets_*` commands or artifacts | They describe what was true when those phases ran. Rewriting them would falsify provenance. |
| Historical tech-paper or benchmark narratives that cite old `assets_*` artifact names | They should be updated only in a dedicated documentation-history pass, not piggybacked onto Phase 055 live boundary work. |
| Any future domain-level rename of product asset terminology | That is a separate semantic refactor and must not be smuggled into this harness-family rename. |

## Execution Rule

Any follow-on rename pass must satisfy all three rules:

1. Rename only the family-level harness surfaces in the canonical scope above.
2. Leave the keep-list vocabulary unchanged unless there is a separate approved
   asset-domain refactor with its own audit map.
3. Treat any remaining `assets_*` hits outside the live harness as historical
   or domain-level until proven otherwise.
