#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
hits="$(
  rg -n --no-heading 'tari_crypto::' \
    "$repo_root/crates" \
    -g '*.rs' \
    -g '!crates/z00z_crypto/**' || true
)"

if [[ -n "$hits" ]]; then
  printf 'Direct tari_crypto imports escaped the z00z_crypto facade:\n%s\n' "$hits" >&2
  exit 1
fi

printf 'z00z_crypto facade audit passed.\n'
