#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "$SCRIPT_DIR/.." && pwd)"
REPO_TOOL_BIN_DIR="$REPO_ROOT/tools/formal_verification/bin"
REPO_CARGO_BIN_DIR="$REPO_ROOT/tools/formal_verification/cargo/bin"

source "$SCRIPT_DIR/target-layout.sh"

if [[ -d "$REPO_TOOL_BIN_DIR" ]]; then
  PATH="$REPO_TOOL_BIN_DIR:$PATH"
fi
if [[ -d "$REPO_CARGO_BIN_DIR" ]]; then
  PATH="$REPO_CARGO_BIN_DIR:$PATH"
fi
export PATH

usage() {
  cat <<'EOF'
Usage:
  ./scripts/run-cargo-fuzz.sh [--namespace NAME] <fuzz-dir> <cargo-fuzz-subcommand> [args...]

Examples:
  ./scripts/run-cargo-fuzz.sh crates/z00z_core/fuzz list
  ./scripts/run-cargo-fuzz.sh crates/z00z_core/fuzz build fuzz_target_asset_pack_from_bytes
  ./scripts/run-cargo-fuzz.sh --namespace custom crates/z00z_storage/fuzz run settlement_proofs -- -max_total_time=30
EOF
}

fail() {
  printf 'Error: %s\n' "$1" >&2
  exit 1
}

NAMESPACE=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --namespace)
      shift
      [[ $# -gt 0 ]] || fail "--namespace requires a value"
      NAMESPACE="$1"
      ;;
    --help|-h)
      usage
      exit 0
      ;;
    --)
      shift
      break
      ;;
    -*)
      fail "unknown option: $1"
      ;;
    *)
      break
      ;;
  esac
  shift
done

[[ $# -ge 2 ]] || {
  usage >&2
  exit 1
}

FUZZ_DIR_INPUT="$1"
shift
SUBCOMMAND="$1"
shift

FUZZ_DIR="$(cd -- "$FUZZ_DIR_INPUT" && pwd)"
[[ -f "$FUZZ_DIR/Cargo.toml" ]] || fail "missing Cargo.toml in $FUZZ_DIR"

if [[ -z "$NAMESPACE" ]]; then
  case "$FUZZ_DIR" in
    "$REPO_ROOT"/crates/*/fuzz)
      NAMESPACE="$(basename -- "$(dirname -- "$FUZZ_DIR")")"
      ;;
    */fuzz)
      NAMESPACE="$(basename -- "$(dirname -- "$FUZZ_DIR")")"
      ;;
    *)
      fail "cannot infer namespace for $FUZZ_DIR; pass --namespace"
      ;;
  esac
fi

TARGET_DIR="$(z00z_fuzz_target_dir "$REPO_ROOT" "$NAMESPACE")"
mkdir -p "$TARGET_DIR"

cd "$FUZZ_DIR"
exec cargo +nightly fuzz "$SUBCOMMAND" \
  --target-dir "$TARGET_DIR" \
  --fuzz-dir "$FUZZ_DIR" \
  "$@"
