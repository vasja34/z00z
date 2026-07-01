# Z00Z Core Config Catalog

Canonical YAML files for `z00z_core` live under this directory.

## 📌 Canonical Files

- `devnet_genesis_config.yaml` is the live root manifest.
- `devnet_assets_config.yaml` owns the asset registry catalog.
- `devnet_rights_config.yaml` owns the rights bootstrap catalog.
- `devnet_policies_config.yaml` owns the action-pool and policy-template catalog.
- `devnet_vouchers_config.yaml` owns the voucher bootstrap catalog.
- `schema_genesis_config.yaml` validates the genesis manifest shape.
- `schema_assets_config.yaml` validates the asset catalog shape.
- `schema_rights_config.yaml` validates the rights catalog shape.
- `schema_policies_config.yaml` validates the policies catalog shape.
- `schema_vouchers_config.yaml` validates the vouchers catalog shape.
- Every `schema_*_config.yaml` file is standalone, self-contained, and paired to exactly one live config kind.
- No `schema_*_config.yaml` file may embed sibling catalog roots or unrelated unreachable definitions.

## ⚙️ Path Policy

- The canonical relative root is `configs/`.
- Callers should use `z00z_core::config_paths::*` instead of hardcoded paths.
- Section subfiles referenced from `devnet_genesis_config.yaml` must stay relative to this directory.
