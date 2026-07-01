#!/usr/bin/env bash
#
# Simple HTTP server for testing Z00Z Wallet WASM
#
# Usage:
#   ./scripts/serve_wasm.sh [port]
#
# Default port: 8000
#

set -euo pipefail

PORT="${1:-8000}"
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

cd "${PROJECT_ROOT}/www"

echo "================================================"
echo "Starting Z00Z Wallet Test Server"
echo "================================================"
echo ""
echo "Server: http://localhost:${PORT}"
echo "Debug:  http://localhost:${PORT}?debug"
echo ""
echo "Press Ctrl+C to stop"
echo "================================================"
echo ""

# Check if Python 3 is available
if command -v python3 &> /dev/null; then
    python3 -m http.server "${PORT}"
elif command -v python &> /dev/null; then
    python -m SimpleHTTPServer "${PORT}"
else
    echo "❌ Python not found. Please install Python 3 to run the test server."
    exit 1
fi
