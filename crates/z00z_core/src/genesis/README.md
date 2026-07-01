# z00z_core genesis

This module is the single genesis orchestration boundary for the live Z00Z
object model.

`GenesisConfig` is the canonical typed bootstrap manifest for the live
repository. Other YAML surfaces may supply registry data, examples, fixtures,
or compatibility inputs, but they do not replace or compete with the
`z00z_core::genesis` authority contract.

## Phase 059 live contract

Genesis now coordinates four typed object lanes under one deterministic export
path:

- Assets: finite-supply bootstrap value objects.
- Rights: zero-value authority objects.
- Policies: content-addressed descriptor records used by wallets, validators,
  storage witnesses, and simulator fixtures.
- Vouchers: conditional-value bootstrap exceptions backed by explicit
  reserve/backing data.

Phase 059 did not introduce a second genesis authority. Object-specific
generation happens through typed helpers under the same `z00z_core::genesis`
surface.

## Typed config sections

The live `GenesisConfig` may include these typed sections:

- `assets`
- `rights`
- `policies`
- `vouchers`

The sections are additive under one manifest and one seed-handling boundary.
They do not imply that all object classes share the same birth semantics.
They do mean that bootstrap authority stays on one manifest instead of being
split across parallel YAML owner surfaces.

For production-sized configs, the repository uses one root manifest plus
referenced subfiles for `assets`, `rights`, `policies`, and `vouchers`. The
live devnet root is `configs/devnet_genesis_config.yaml` with sibling
`configs/devnet_{assets,rights,policies,vouchers}_config.yaml` files. The
loader must rehydrate those refs back into the same `GenesisConfig` shape
before validation or generation. Splitting files is allowed; splitting
authority is not.

## Canonical semantics matrix

Use [`../docs/OBJECT_FAMILY_SEMANTICS.md`](../docs/OBJECT_FAMILY_SEMANTICS.md)
as the single live semantics matrix for assets, rights, policies, and
vouchers.

- `z00z_core::ObjectFamily` is the canonical public family vocabulary.
  `z00z_core::assets::ObjectFamily` remains a compatibility facade only.
- `VoucherBootstrapEntryV1` is a bootstrap-only input that materializes
  voucher config; the runtime voucher object is `VoucherLeaf`.
- `mintable` stays an asset-definition/catalog flag on the current tree. The
  public runtime object RPC does not expose a generic post-genesis asset-mint
  selector.

## Object-specific semantics

### Assets

- Asset generation remains finite-supply and class-aware.
- Native `Z00Z` cash stays semantically clean and does not accept arbitrary
  action-pool behavior.
- The current public runtime does not document a generic post-genesis asset
  mint API. `mintable` remains a bootstrap/catalog flag until a separate live
  runtime issuance lane is implemented and tested.

### Rights

- Rights are authority instances, not value containers.
- Genesis rights bind scope, holder/control fixtures, quotas or nonce,
  validity windows, policy ids, and metadata commitments.
- Rights never mint, reserve, or transport value.

### Policies

- Policy descriptors are deterministic and content-addressed.
- Exported descriptor hashes are the canonical cross-crate bind point for
  wallet builders, validator checks, storage records, and simulator fixtures.

### Vouchers

- Genesis vouchers are bootstrap exceptions, not the ordinary issuance lane.
- `VoucherBootstrapEntryV1` is the bootstrap manifest shape; runtime vouchers
  persist as `VoucherLeaf` after materialization and validator checks.
- Every genesis voucher must bind to explicit backing/reserve evidence or a
  genesis-reserve source.
- Runtime issuance remains a validator-checked action outside genesis.

## Exported artifacts

Stage 1 and direct genesis exports now bind one manifest plus typed artifacts:

- `genesis_rights.json`
- `genesis_policies.json`
- `genesis_vouchers.json`
- `genesis_settlement_manifest.json`

The manifest remains the single summary packet. Typed artifacts do not replace
it or create parallel publication truth.

## Determinism rules

Genesis derivation is deterministic and domain-separated by:

- network
- chain id
- object class
- object id
- per-object index
- root generation
- policy/action descriptor hash where applicable

Voucher derivations must not reuse right derivation labels.

## Operator notes

- Use the shared genesis boundary for all object classes.
- Regenerate simulator/dev fixtures from the typed genesis config rather than
  inventing wallet-local or storage-local bootstrap paths.
- Use `z00z_core::rights` as the owner path for rights-config types and
  loaders; the old `assets::right_config` shim path is removed.
- Treat `configs/devnet_assets_config.yaml` as registry/example/compatibility data,
  not as equal bootstrap authority with `GenesisConfig`.
- Treat `configs/devnet_genesis_config.yaml` as the canonical root
  manifest when the live config is split into referenced subfiles.
- Keep actions nested inside policy entries. `actions_config.yaml` is
  intentionally absent from the live bootstrap path and must not become a
  second authority file.
- Treat `configs/devnet_genesis_config.yaml` as the compact Phase 059
  object-family fixture for release-mode simulator checks.

## Validation anchors

The live genesis surface is pinned by release-mode tests including:

- `cargo test -p z00z_core --release --test genesis_tests test_genesis_plan_full_bootstrap_matches_legacy_run -- --nocapture`
- `cargo test -p z00z_core --release --test test_genesis_manifest_refs -- --nocapture`
- `cargo test -p z00z_core --release --test test_genesis_manifest_goldens -- --nocapture`
- `cargo test -p z00z_core --release --lib test_policy_descriptor -- --nocapture`
- `cargo test -p z00z_core --release --lib test_voucher_config -- --nocapture`
- `cargo test -p z00z_core --release --test assets_tests rights_config -- --nocapture`

See `059-03-SUMMARY.md` and `059-EVIDENCE-LEDGER.md` for the full closure map.
