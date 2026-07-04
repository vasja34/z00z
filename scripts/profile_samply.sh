#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

readonly -a Z00Z_PACKAGES=(
  "z00z_aggregators"
  "z00z_core"
  "z00z_crypto"
  "z00z_networks_rpc"
  "z00z_rollup_node"
  "z00z_simulator"
  "z00z_storage"
  "z00z_telemetry"
  "z00z_utils"
  "z00z_validators"
  "z00z_wallets"
  "z00z_watchers"
)

usage() {
  cat <<'EOF'
Usage:
  ./scripts/profile_samply.sh list
  ./scripts/profile_samply.sh --package <z00z_pkg> [cargo-samply target args...] [-- app args...]

Examples:
  ./scripts/profile_samply.sh list
  ./scripts/profile_samply.sh --package z00z_simulator --bin scenario_1
  ./scripts/profile_samply.sh --package z00z_storage --test test_bench_lanes -- --nocapture
  ./scripts/profile_samply.sh --package z00z_wallets --bench tx_perf_bench -- --bench
EOF
}

has_package() {
  local wanted="$1"
  local pkg
  for pkg in "${Z00Z_PACKAGES[@]}"; do
    if [[ "$pkg" == "$wanted" ]]; then
      return 0
    fi
  done
  return 1
}

list_targets() {
  python3 - <<'PY'
import json
import subprocess

meta = json.loads(
    subprocess.check_output(
        ["cargo", "metadata", "--no-deps", "--format-version", "1"],
        text=True,
    )
)

for pkg in sorted(meta["packages"], key=lambda row: row["name"]):
    name = pkg["name"]
    if not name.startswith("z00z_"):
        continue
    if "/tari/" in pkg["manifest_path"]:
        continue

    print(name)
    rows = []
    for target in pkg["targets"]:
        kinds = target["kind"]
        if "bin" in kinds:
            rows.append(("bin", target["name"]))
        elif "bench" in kinds:
            rows.append(("bench", target["name"]))
        elif "example" in kinds:
            rows.append(("example", target["name"]))
        elif "test" in kinds:
            rows.append(("test", target["name"]))

    for kind, target_name in sorted(rows):
        print(f"  {kind:<7} {target_name}")
PY
}

if ! command -v cargo-samply >/dev/null 2>&1 && ! cargo samply --version >/dev/null 2>&1; then
  echo "error: cargo-samply is not installed" >&2
  echo "hint: cargo install cargo-samply --locked" >&2
  exit 127
fi

if [[ $# -eq 0 ]]; then
  usage
  exit 1
fi

if [[ "$1" == "list" ]]; then
  list_targets
  exit 0
fi

package=""
args=()
while [[ $# -gt 0 ]]; do
  case "$1" in
    -p|--package)
      if [[ $# -lt 2 ]]; then
        echo "error: missing package name after $1" >&2
        exit 2
      fi
      package="$2"
      args+=("$1" "$2")
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      args+=("$1")
      shift
      ;;
  esac
done

if [[ -z "$package" ]]; then
  echo "error: --package <z00z_pkg> is required" >&2
  echo >&2
  usage >&2
  exit 2
fi

if ! has_package "$package"; then
  echo "error: unsupported package '$package'" >&2
  echo "supported packages:" >&2
  printf '  %s\n' "${Z00Z_PACKAGES[@]}" >&2
  exit 2
fi

exec cargo samply --profile samply --no-profile-inject "${args[@]}"
