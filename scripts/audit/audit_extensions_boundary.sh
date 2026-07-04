#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
namespace_root="$repo_root/crates/z00z_extensions"
root_cargo_toml="$namespace_root/Cargo.toml"
root_src_dir="$namespace_root/src"

check_semantic_hits() {
  local search_root="$1"
  local semantic_hits
  semantic_hits="$(
    rg -n --no-heading \
      'z00z_(core|wallets|storage|runtime|rollup_node|simulator|networks|telemetry)::|use z00z_(core|wallets|storage|runtime|rollup_node|simulator|networks|telemetry)' \
      "$search_root" \
      -g '*.rs' || true
  )"

  if [[ -n "$semantic_hits" ]]; then
    printf 'z00z_extensions imported semantic owner crates without an explicit extension plan:\n%s\n' "$semantic_hits" >&2
    exit 1
  fi
}

if [[ -s "$root_cargo_toml" ]]; then
  deps_block="$(
    awk '
      /^\[dependencies\]/ { in_deps = 1; next }
      /^\[/ && in_deps { exit }
      in_deps { print }
    ' "$root_cargo_toml" | sed '/^[[:space:]]*#/d;/^[[:space:]]*$/d'
  )"

  if [[ -n "$deps_block" ]]; then
    printf 'z00z_extensions must stay dependency-light until an explicit extension plan lands:\n%s\n' "$deps_block" >&2
    exit 1
  fi

  if [[ -d "$root_src_dir" ]]; then
    check_semantic_hits "$root_src_dir"
  fi
else
  live_manifests="$(
    find "$namespace_root" -mindepth 2 -maxdepth 2 -name Cargo.toml -type f -size +1c | sort
  )"
  if [[ -n "$live_manifests" ]]; then
    printf 'z00z_extensions namespace contains nested manifests that need an explicit extension workspace plan before they become live crates:\n%s\n' "$live_manifests" >&2
    exit 1
  fi

  nested_src_dirs="$(
    find "$namespace_root" -mindepth 2 -maxdepth 2 -type d -name src | sort
  )"
  if [[ -n "$nested_src_dirs" ]]; then
    while IFS= read -r src_dir; do
      [[ -n "$src_dir" ]] || continue
      check_semantic_hits "$src_dir"
    done <<< "$nested_src_dirs"
  fi
fi

printf 'z00z_extensions boundary audit passed.\n'
