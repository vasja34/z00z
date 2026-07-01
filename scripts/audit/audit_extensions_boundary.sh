#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cargo_toml="$repo_root/crates/z00z_extensions/Cargo.toml"
src_dir="$repo_root/crates/z00z_extensions/src"

deps_block="$(
  awk '
    /^\[dependencies\]/ { in_deps = 1; next }
    /^\[/ && in_deps { exit }
    in_deps { print }
  ' "$cargo_toml" | sed '/^[[:space:]]*#/d;/^[[:space:]]*$/d'
)"

if [[ -n "$deps_block" ]]; then
  printf 'z00z_extensions must stay dependency-light until an explicit extension plan lands:\n%s\n' "$deps_block" >&2
  exit 1
fi

semantic_hits="$(
  rg -n --no-heading \
    'z00z_(core|wallets|storage|runtime|rollup_node|simulator|networks|telemetry)::|use z00z_(core|wallets|storage|runtime|rollup_node|simulator|networks|telemetry)' \
    "$src_dir" \
    -g '*.rs' || true
)"

if [[ -n "$semantic_hits" ]]; then
  printf 'z00z_extensions imported semantic owner crates without an explicit extension plan:\n%s\n' "$semantic_hits" >&2
  exit 1
fi

printf 'z00z_extensions boundary audit passed.\n'
