#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

expect_fail() {
  local name="$1"
  shift
  local log="$TMP_DIR/${name}.log"
  if "$@" >"$log" 2>&1; then
    echo "expected failure but command succeeded: $*" >&2
    cat "$log" >&2
    exit 1
  fi
  if ! grep -q "MUST NOT be compiled into release-capable" "$log"; then
    echo "expected release-capable compile guard message for: $*" >&2
    cat "$log" >&2
    exit 1
  fi
}

python3 - <<'PY'
from pathlib import Path

root = Path.cwd()

checks = [
    (
        root / "crates/z00z_wallets/src/db/mod.rs",
        "pub use self::redb_store::debug_export_wallet",
        False,
        "wallet db facade must not re-export debug_export_wallet",
    ),
    (
        root / "crates/z00z_wallets/src/wallet/mod.rs",
        "pub use crate::db::debug_export_wallet",
        False,
        "wallet facade must not re-export debug_export_wallet",
    ),
    (
        root / "crates/z00z_wallets/src/lib.rs",
        "pub mod internal_debug_tools",
        True,
        "wallet crate must expose the explicit internal debug surface",
    ),
    (
        root / "crates/z00z_storage/src/settlement/hjmt_cache.rs",
        "#[cfg(debug_assertions)]\n    pub fn corrupt_forest_cache_for_test",
        True,
        "forest-cache corruption hook must stay debug-only",
    ),
    (
        root / "crates/z00z_storage/src/settlement/hjmt_cache.rs",
        "#[cfg(debug_assertions)]\n    pub fn corrupt_journal_key_for_test",
        True,
        "forest-cache journal drift hook must stay debug-only",
    ),
    (
        root / "crates/z00z_storage/src/settlement/hjmt_scheduler.rs",
        "#[cfg(debug_assertions)]\n    pub fn set_sched_limits_for_test",
        True,
        "scheduler limit hook must stay debug-only",
    ),
    (
        root / "crates/z00z_storage/src/settlement/hjmt_scheduler.rs",
        "#[cfg(debug_assertions)]\n    pub fn set_sched_cancel_for_test",
        True,
        "scheduler cancel hook must stay debug-only",
    ),
    (
        root / "crates/z00z_storage/src/settlement/hjmt_scheduler.rs",
        "#[cfg(debug_assertions)]\n    pub fn set_sched_test_skew_ms",
        True,
        "scheduler skew hook must stay debug-only",
    ),
    (
        root / "crates/z00z_storage/src/settlement/hjmt_scheduler.rs",
        "#[cfg(debug_assertions)]\n    pub fn reset_sched_for_test",
        True,
        "scheduler reset hook must stay debug-only",
    ),
    (
        root / "crates/z00z_simulator/src/scenario_1/stage_3/post_claim.rs",
        "internal_debug_tools::debug_export_wallet",
        True,
        "simulator must use the explicit internal wallet debug surface",
    ),
]

for path, needle, present, message in checks:
    text = path.read_text()
    has = needle in text
    if has != present:
        raise SystemExit(f"{message}: {path}")
PY

expect_fail wallets_test_params cargo check -p z00z_wallets --release --features test-params-fast
expect_fail wallets_debug_tools cargo check -p z00z_wallets --release --features wallet_debug_tools
expect_fail simulator_debug_tools cargo check -p z00z_simulator --release --features wallet_debug_tools
expect_fail simulator_test_params cargo check -p z00z_simulator --release --features test-params-fast
