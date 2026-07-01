#!/usr/bin/env bash
#
# Z00Z Wallet - Desktop EGUI Launcher
#
# Launches the desktop wallet UI using egui/eframe.
#
# Usage:
#   ./bin/z00z_wallet_egui.sh

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WALLET_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$WALLET_DIR"

echo "================================================"
echo "🚀 Z00Z Wallet - Desktop EGUI"
echo "================================================"
echo ""

# Check if egui feature is available
if ! cargo metadata --format-version 1 --no-deps 2>/dev/null | grep -q '"egui"'; then
    echo "⚠️  Warning: egui feature may not be properly configured"
fi

echo "🔨 Building and launching desktop wallet..."
echo ""

# Run with egui feature and auto-stop the smoke run.
EGUI_STOP_AFTER_SEC="${Z00Z_EGUI_STOP_AFTER_SEC:-5}"
HEADLESS_ENV=()

if ! ( [ -n "${DISPLAY:-}" ] && command -v xdpyinfo >/dev/null 2>&1 && xdpyinfo >/dev/null 2>&1 ); then
    echo "⚠️  No usable display detected; using headless smoke fallback"
    HEADLESS_ENV=(Z00Z_EGUI_SMOKE_HEADLESS=1)
fi

if command -v timeout >/dev/null 2>&1; then
    exec env "${HEADLESS_ENV[@]}" timeout --signal=TERM --kill-after=2s "${EGUI_STOP_AFTER_SEC}s" \
        cargo run --features egui --bin z00z_wallet_egui
fi

echo "⚠️  timeout command not found; using Python fallback auto-stop"
python3 - "$EGUI_STOP_AFTER_SEC" "${HEADLESS_ENV[@]}" <<'PY'
import os
import signal
import subprocess
import sys

stop_after = float(sys.argv[1])
extra_env = dict(arg.split("=", 1) for arg in sys.argv[2:])
env = os.environ.copy()
env.update(extra_env)
proc = subprocess.Popen(
    ["cargo", "run", "--features", "egui", "--bin", "z00z_wallet_egui"],
    start_new_session=True,
    env=env,
)

try:
    code = proc.wait(timeout=stop_after)
except subprocess.TimeoutExpired:
    os.killpg(proc.pid, signal.SIGTERM)
    try:
        proc.wait(timeout=2)
    except subprocess.TimeoutExpired:
        os.killpg(proc.pid, signal.SIGKILL)
        proc.wait()
    raise SystemExit(0)

raise SystemExit(code)
PY
