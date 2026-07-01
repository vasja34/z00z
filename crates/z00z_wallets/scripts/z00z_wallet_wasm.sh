#!/usr/bin/env bash
#
# Z00Z Wallet - WASM Browser UI Launcher
#
# Builds and launches the WASM-based wallet UI in a browser.
#
# Usage:
#   ./bin/z00z_wallet_wasm.sh           # Production build + serve
#   ./bin/z00z_wallet_wasm.sh dev       # Development build + serve
#   ./bin/z00z_wallet_wasm.sh check     # Just check compilation

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WALLET_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
PROJECT_ROOT="$(cd "$WALLET_DIR/../.." && pwd)"

cd "$WALLET_DIR"

MODE="${1:-prod}"

echo "================================================"
echo "🚀 Z00Z Wallet - WASM Browser UI"
echo "================================================"
echo ""

case "$MODE" in
    check)
        echo "🔍 Checking compilation..."
        echo ""
        
        # Check native build
        echo "  Native build..."
        cargo check
        echo ""
        
        # Check WASM build
        echo "  WASM build..."
        cargo check --target wasm32-unknown-unknown --features wasm
        echo ""
        
        echo "✅ Compilation successful!"
        echo ""
        echo "To build and run:"
        echo "  ./bin/z00z_wallet_wasm.sh"
        ;;
    
    dev)
        echo "🔨 Building WASM (development mode)..."
        echo ""
        
        # Check if build script exists
        if [ ! -f "$PROJECT_ROOT/scripts/build_wasm.sh" ]; then
            echo "❌ Build script not found: $PROJECT_ROOT/scripts/build_wasm.sh"
            exit 1
        fi
        
        # Build WASM
        "$PROJECT_ROOT/scripts/build_wasm.sh" --dev
        
        echo ""
        echo "✅ Development build complete!"
        echo ""
        echo "🌐 Starting HTTP server..."
        echo ""
        
        # Serve
        cd "$PROJECT_ROOT"
        
        # Check if serve script exists
        if [ -f "$PROJECT_ROOT/scripts/serve_wasm.sh" ]; then
            exec "$PROJECT_ROOT/scripts/serve_wasm.sh"
        else
            # Fallback to Python HTTP server
            echo "Starting Python HTTP server on port 8000..."
            cd www
            exec python3 -m http.server 8000
        fi
        ;;
    
    prod|*)
        echo "🔨 Building WASM (production mode)..."
        echo ""
        
        # Check if build script exists
        if [ ! -f "$PROJECT_ROOT/scripts/build_wasm.sh" ]; then
            echo "❌ Build script not found: $PROJECT_ROOT/scripts/build_wasm.sh"
            exit 1
        fi
        
        # Build WASM
        "$PROJECT_ROOT/scripts/build_wasm.sh"
        
        echo ""
        echo "✅ Production build complete!"
        echo ""
        echo "📊 Build statistics:"
        if [ -f "$PROJECT_ROOT/www/pkg/z00z_wallets_bg.wasm" ]; then
            WASM_SIZE=$(du -h "$PROJECT_ROOT/www/pkg/z00z_wallets_bg.wasm" 2>/dev/null | cut -f1 || echo "N/A")
            echo "   WASM size: $WASM_SIZE"
        fi
        echo ""
        echo "🌐 Starting HTTP server..."
        echo ""
        
        # Serve
        cd "$PROJECT_ROOT"
        
        # Check if serve script exists
        if [ -f "$PROJECT_ROOT/scripts/serve_wasm.sh" ]; then
            exec "$PROJECT_ROOT/scripts/serve_wasm.sh"
        else
            # Fallback to Python HTTP server
            echo "Starting Python HTTP server on port 8000..."
            cd www
            exec python3 -m http.server 8000
        fi
        ;;
esac
