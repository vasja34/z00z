#!/usr/bin/env bash
#
# Build WASM binaries for Z00Z Wallet
#
# Usage:
#   ./scripts/build_wasm.sh          # Production build with optimization
#   ./scripts/build_wasm.sh --dev    # Development build (no optimization)
#
# Prerequisites:
#   - cargo install wasm-pack
#   - cargo install wasm-opt
#

set -euo pipefail

# Colors for output
readonly RED='\033[0;31m'
readonly GREEN='\033[0;32m'
readonly YELLOW='\033[1;33m'
readonly NC='\033[0m' # No Color

# Default values
OPTIMIZE=true
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# Cleanup on error
cleanup_on_error() {
    echo -e "${RED}❌ Build failed, cleaning up partial artifacts${NC}"
    rm -rf "${PROJECT_ROOT}/www/pkg" "${PROJECT_ROOT}/www/worker"
}
trap cleanup_on_error ERR

# Parse command-line arguments
if [ "${1:-}" = "--dev" ]; then
    echo -e "${YELLOW}Building in development mode (no optimization)${NC}"
    OPTIMIZE=false
fi

# Change to project root
cd "${PROJECT_ROOT}"

echo "================================================"
echo "Building WASM binaries for Z00Z Wallet"
echo "================================================"
echo ""

# Step 1: Build Main UI WASM
echo "🔨 Building main UI WASM..."
cd "${PROJECT_ROOT}/crates/z00z_wallets"
if ! wasm-pack build --target web --out-dir ../../www/pkg \
    --features wasm; then
    echo -e "${RED}❌ Failed to build main UI WASM${NC}"
    exit 1
fi
cd "${PROJECT_ROOT}"
echo -e "${GREEN}✅ Main UI WASM built${NC}"
echo ""

# Step 2: Build Web Worker WASM
echo "🔨 Building Web Worker WASM..."
# Note: wasm-pack doesn't support --bin flag, so we use a workaround
# The wallet_worker module is compiled as part of the main library with wasm feature
# For now, we'll use the same build but document this needs refinement
echo -e "${YELLOW}⚠️  Using main library build (wallet_worker is included)${NC}"
echo -e "${YELLOW}    TODO: Separate worker build when wasm-pack supports binary targets${NC}"
echo ""

# Step 3: WASM Optimization
if [ "$OPTIMIZE" = true ]; then
    echo "⚙️  Optimizing WASM binaries..."
    
    if command -v wasm-opt &> /dev/null; then
        # Get original size
        original_size=$(stat -f%z "${PROJECT_ROOT}/www/pkg/z00z_wallets_bg.wasm" 2>/dev/null || stat -c%s "${PROJECT_ROOT}/www/pkg/z00z_wallets_bg.wasm" 2>/dev/null || echo "0")
        
        # Optimize main UI WASM
        echo "  Optimizing main UI WASM with -Oz --enable-bulk-memory..."
        wasm-opt -Oz --enable-bulk-memory -o www/pkg/z00z_wallets_bg.wasm.opt \
            www/pkg/z00z_wallets_bg.wasm
        mv www/pkg/z00z_wallets_bg.wasm.opt www/pkg/z00z_wallets_bg.wasm
        
        # Get optimized size
        optimized_size=$(stat -f%z "${PROJECT_ROOT}/www/pkg/z00z_wallets_bg.wasm" 2>/dev/null || stat -c%s "${PROJECT_ROOT}/www/pkg/z00z_wallets_bg.wasm" 2>/dev/null || echo "0")
        
        # Calculate reduction
        if [ "$original_size" -gt 0 ]; then
            reduction=$((100 - (optimized_size * 100 / original_size)))
            echo -e "${GREEN}  ✅ Reduced size by ${reduction}%${NC}"
        fi
        
        echo -e "${GREEN}✅ WASM optimization complete${NC}"
    else
        echo -e "${YELLOW}⚠️  wasm-opt not found, skipping optimization${NC}"
        echo -e "${YELLOW}    Install with: cargo install wasm-opt${NC}"
    fi
else
    echo -e "${YELLOW}⚠️  Skipping optimization (development mode)${NC}"
fi
echo ""

# Step 4: Size Reporting
echo "================================================"
echo "📦 Build Results"
echo "================================================"
echo ""

if [ -f "${PROJECT_ROOT}/www/pkg/z00z_wallets_bg.wasm" ]; then
    main_size=$(du -h "${PROJECT_ROOT}/www/pkg/z00z_wallets_bg.wasm" | cut -f1)
    echo "Main UI WASM: ${main_size}"
else
    echo -e "${RED}Main UI WASM: NOT FOUND${NC}"
fi

echo ""
echo "Generated files in www/pkg/:"
ls -lh "${PROJECT_ROOT}/www/pkg/" | tail -n +2 || echo "  (none)"

echo ""
echo "================================================"
echo -e "${GREEN}✅ Build complete!${NC}"
echo "================================================"
echo ""
echo "Next steps:"
echo "  1. Copy www/pkg/ to your web server"
echo "  2. Load z00z_wallets.js in your HTML"
echo "  3. Call init() before using wallet functions"
echo ""
