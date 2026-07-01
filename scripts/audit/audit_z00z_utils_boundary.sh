#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
targets=(
  "$repo_root/crates/z00z_core/src"
  "$repo_root/crates/z00z_wallets/src"
  "$repo_root/crates/z00z_storage/src"
  "$repo_root/crates/z00z_runtime/aggregators/src"
  "$repo_root/crates/z00z_rollup_node/src"
  "$repo_root/crates/z00z_simulator/src"
)

allowlisted_files=(
  "crates/z00z_core/src/genesis/manifest_ref_loader.rs"
  "crates/z00z_wallets/src/db/wallet_store.rs"
  "crates/z00z_wallets/src/redb_store/mutations_create_wallet.rs"
  "crates/z00z_wallets/src/redb_store/open_discovery.rs"
  "crates/z00z_wallets/src/redb_store/open_wallet.rs"
  "crates/z00z_wallets/src/redb_store/session.rs"
  "crates/z00z_simulator/src/scenario_1/stage_2/checks.rs"
)

is_allowlisted_file() {
  local rel_path="$1"
  local item
  for item in "${allowlisted_files[@]}"; do
    if [[ "$rel_path" == "$item" ]]; then
      return 0
    fi
  done
  return 1
}

failures=()
while IFS= read -r hit; do
  [[ -z "$hit" ]] && continue
  IFS=: read -r abs_path line_no code <<<"$hit"
  rel_path="${abs_path#"$repo_root"/}"
  trimmed="${code#"${code%%[![:space:]]*}"}"

  if [[ "$trimmed" == '//'*
     || "$trimmed" == '///'*
     || "$trimmed" == '//!'* ]]; then
    continue
  fi

  if [[ "$code" == *"std::fs::File"* ]]; then
    continue
  fi

  if is_allowlisted_file "$rel_path"; then
    continue
  fi

  failures+=("$rel_path:$line_no:$code")
done < <(
  rg -n --no-heading \
    'std::fs::|serde_json::|serde_yaml::|SystemTime::now\(|rand::thread_rng\(|rand::random\(' \
    "${targets[@]}" \
    -g '*.rs' \
    -g '!**/test_*.rs'
)

if ((${#failures[@]})); then
  printf 'z00z_utils boundary drift detected outside Phase 064 allowlist:\n' >&2
  printf '  %s\n' "${failures[@]}" >&2
  exit 1
fi

printf 'z00z_utils boundary audit passed for Phase 064 target crates.\n'
