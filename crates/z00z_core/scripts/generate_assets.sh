#!/bin/bash
# Z00Z Asset Generation CLI Launcher
# 
# Generates cryptographically secure confidential assets with:
# - Real Bulletproofs+ range proofs
# - Schnorr signature verification
# - Pedersen commitments
# - JSON + Bincode serialization
#
# Usage:
#   ./scripts/generate_assets.sh [options]
#   ./scripts/generate_assets.sh --help
#   ./scripts/generate_assets.sh --format bincode
#   ./scripts/generate_assets.sh --threads 16 --format json --verbose

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$(dirname "$SCRIPT_DIR")")"

# ============================================================================
# Help Function
# ============================================================================

show_help() {
    cat << 'EOF'
╔════════════════════════════════════════════════════════════════════════════╗
║                   Z00Z Asset Generation CLI Launcher                       ║
║                                                                            ║
║  Generate cryptographically secure confidential assets with:              ║
║  - Real Bulletproofs+ range proofs (0 ≤ amount < 2^64)                   ║
║  - Schnorr signature verification (s·G = R + e·P)                         ║
║  - Pedersen commitments (C = amount·G + blinding·H)                       ║
║  - JSON + Bincode serialization                                           ║
╚════════════════════════════════════════════════════════════════════════════╝

USAGE:
    generate_assets.sh [OPTIONS]

OPTIONS:
    --format FORMAT         Output format: json or bincode (default: json)
                           - json:    Human-readable JSON per asset class
                           - bincode: Compact binary, 8-9x smaller than JSON

    --config PATH          Path to YAML config file
                           (default: configs/devnet_assets_config.yaml)

    --output DIR           Output directory for assets and reports
                           (default: outputs/assets)

    --threads N            Number of parallel threads
                           (default: auto-detect CPU cores)
                           - Use --threads 1 for single-threaded
                           - Use --threads 0 to use all available cores

    --verbose, -v          Enable verbose output
                           Shows detailed per-asset and per-definition info

    --release              Build in release mode (optimized, ~30% faster)

    --help, -h            Show this help message

EXAMPLES:
    # Generate with default settings (auto-detect CPU cores, JSON output)
    ./scripts/generate_assets.sh

    # Generate with verbose output
    ./scripts/generate_assets.sh --verbose

    # Generate bincode with all CPU cores (release mode)
    ./scripts/generate_assets.sh --format bincode --release

    # Generate with limited parallelism (4 threads)
    ./scripts/generate_assets.sh --threads 4 --verbose

    # Generate with custom config and output directory
    ./scripts/generate_assets.sh --config my_assets.yaml --output /tmp/assets

CRYPTOGRAPHIC VERIFICATION:

    All generated assets are verified cryptographically:

    1. Pedersen Commitments: C = amount·G + blinding·H
       - Verification: Commitment computed from amount and blinding
       - Security: Binding (can't change amount without changing blinding)
       - Privacy: Hiding (commitment reveals nothing about amount)

    2. Bulletproofs+ Range Proofs: Proves 0 ≤ amount < 2^64
       - Verification: service.verify(proof, commitment)
       - Size: ~576 bytes per proof (~7ms generation, ~2ms verification)
       - Security: Zero-knowledge, no amount leakage

    3. Schnorr Signatures: s·G = R + e·P
       - Verification: signature.verify(public_key, message)
       - Domain: Separate hash domain per asset class
       - Security: Proves ownership without revealing private key

    4. Homomorphic Property: C1 + C2 = C(a1 + a2, b1 + b2)
       - Verification: Test for at least 2 assets of same class
       - Use case: Privacy-preserving transaction aggregation

OUTPUT FILES:

    Generated assets are organized by type:
    
    outputs/assets/
    ├── json/
    │   ├── coins_YYYYMMDD_HHMMSS.json      # All coin assets
    │   ├── tokens_YYYYMMDD_HHMMSS.json     # All token assets
    │   ├── nfts_YYYYMMDD_HHMMSS.json       # All NFT assets
    │   └── voids_YYYYMMDD_HHMMSS.json      # All void sink assets
    ├── bin/
    │   └── assets_YYYYMMDD_HHMMSS.bin      # All assets (bincode format)
    └── reports/
        └── report_YYYYMMDD_HHMMSS.txt      # Statistics and metrics

PERFORMANCE METRICS:

    Typical performance on 28-core system (release mode):
    - Generation: 2200 assets in 0.96s (2280 assets/sec)
    - Verification: All 2200 commitments, range proofs, signatures verified
    - JSON output: 17.4 MB (~50ms serialization)
    - Bincode output: 2.0 MB (~2ms serialization)
    - Compression: 8.5x (JSON vs Bincode)

CONFIGURATION:

    The asset configuration is defined in configs/devnet_assets_config.yaml:

    assets:
      - id: z00z
        class: Coin
        serials: 100           # Number of unique serial IDs
        nominal: 20000         # Value per serial
        # Each serial generates ONE asset with:
        # - Unique serial_id (0-99 for 100 serials)
        # - Random blinding factor
        # - Generated Pedersen commitment
        # - Generated Bulletproofs+ range proof
        # - Generated Schnorr signature

    Total assets generated = sum(serials) for all asset definitions
    With config above: 100 (coins) + 100 (zUSD) + 1000 (nfts) + 1000 (voids)
                     = 2200 total assets

SECURITY NOTES:

    ✓ All cryptography is from production tari_crypto library
    ✓ Range proofs: Real Bulletproofs+, not placeholders
    ✓ Signatures: Real Schnorr with domain separation
    ✓ No unsafe code, all memory-safe Rust
    ✓ DoS protection: 10KB proof size limit
    ✓ Constant-time comparison for signatures

EXIT CODES:
    0   Success
    1   General error (missing files, invalid args)
    2   Compilation failed
    3   Runtime error (generation/verification failed)

SEE ALSO:
    - configs/devnet_assets_config.yaml      Asset definitions
    - src/assets/assets.rs               Asset structure
    - src/assets/crypto.rs               Cryptographic operations
    - bin/assets_generation_cli.rs       CLI source code

EOF
}

# ============================================================================
# Parse Arguments
# ============================================================================

FORMAT="json"
CONFIG="configs/devnet_assets_config.yaml"
OUTPUT="outputs/assets"
THREADS=""
VERBOSE=""

# Default to release mode for performance (crypto-heavy workload)
RELEASE="--release"
BUILD_MODE="release"

while [[ $# -gt 0 ]]; do
    case $1 in
        --format)
            FORMAT="$2"
            shift 2
            ;;
        --config)
            CONFIG="$2"
            shift 2
            ;;
        --output)
            OUTPUT="$2"
            shift 2
            ;;
        --threads)
            THREADS="$2"
            shift 2
            ;;
        --verbose|-v)
            VERBOSE="--verbose"
            shift
            ;;
        --release)
            RELEASE="--release"
            BUILD_MODE="release"
            shift
            ;;
        --debug)
            RELEASE=""
            BUILD_MODE="debug"
            shift
            ;;
        --help|-h)
            show_help
            exit 0
            ;;
        *)
            echo -e "${RED}Error: Unknown option: $1${NC}"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

# ============================================================================
# Validation
# ============================================================================

# Validate format
if [[ "$FORMAT" != "json" && "$FORMAT" != "bincode" ]]; then
    echo -e "${RED}Error: Invalid format '$FORMAT'. Must be 'json' or 'bincode'${NC}"
    exit 1
fi

# Check if in correct directory
if [[ ! -f "Cargo.toml" ]]; then
    echo -e "${RED}Error: Cargo.toml not found. Run from z00z_core directory${NC}"
    exit 1
fi

# Check config file
if [[ ! -f "$CONFIG" ]]; then
    echo -e "${RED}Error: Config file not found: $CONFIG${NC}"
    exit 1
fi

# ============================================================================
# Build
# ============================================================================

echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}Building asset generation CLI (${BUILD_MODE} mode)...${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"

if ! cargo build --bin assets_generation_cli $RELEASE 2>&1 | grep -E "Compiling|Finished|error"; then
    echo -e "${RED}Build failed!${NC}"
    exit 2
fi

# ============================================================================
# Prepare CLI Arguments
# ============================================================================

CLI_ARGS=(
    "--format" "$FORMAT"
    "--config" "$CONFIG"
    "--output" "$OUTPUT"
)

if [[ -n "$THREADS" ]]; then
    CLI_ARGS+=("--threads" "$THREADS")
fi

if [[ -n "$VERBOSE" ]]; then
    CLI_ARGS+=("$VERBOSE")
fi

# ============================================================================
# Run CLI
# ============================================================================

echo ""
echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}Launching Asset Generation CLI${NC}"
echo -e "${BLUE}Format: ${YELLOW}${FORMAT}${BLUE}${NC}"
if [[ -n "$THREADS" ]]; then
    echo -e "${BLUE}Threads: ${YELLOW}${THREADS}${BLUE}${NC}"
fi
echo -e "${BLUE}Build mode: ${YELLOW}${BUILD_MODE}${BLUE}${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo ""

if [[ "$RELEASE" == "--release" ]]; then
    cargo run --release --bin assets_generation_cli -- "${CLI_ARGS[@]}"
else
    cargo run --bin assets_generation_cli -- "${CLI_ARGS[@]}"
fi

EXIT_CODE=$?

# ============================================================================
# Post-execution Summary
# ============================================================================

if [[ $EXIT_CODE -eq 0 ]]; then
    echo ""
    echo -e "${GREEN}═══════════════════════════════════════════════════════════${NC}"
    echo -e "${GREEN}✓ Asset generation completed successfully!${NC}"
    echo -e "${GREEN}═══════════════════════════════════════════════════════════${NC}"
    echo ""
    
    # Count output files
    if [[ -d "$OUTPUT" ]]; then
        JSON_FILES=$(find "$OUTPUT/json" -type f 2>/dev/null | wc -l)
        BIN_FILES=$(find "$OUTPUT/bin" -type f 2>/dev/null | wc -l)
        REPORTS=$(find "$OUTPUT/reports" -type f 2>/dev/null | wc -l)
        
        echo -e "${GREEN}Output summary:${NC}"
        [[ $JSON_FILES -gt 0 ]] && echo -e "  ${GREEN}✓${NC} JSON files: $JSON_FILES"
        [[ $BIN_FILES -gt 0 ]] && echo -e "  ${GREEN}✓${NC} Binary files: $BIN_FILES"
        [[ $REPORTS -gt 0 ]] && echo -e "  ${GREEN}✓${NC} Reports: $REPORTS"
        echo ""
        echo -e "  Output directory: ${YELLOW}$OUTPUT${NC}"
    fi
else
    echo ""
    echo -e "${RED}═══════════════════════════════════════════════════════════${NC}"
    echo -e "${RED}✗ Asset generation failed (exit code: $EXIT_CODE)${NC}"
    echo -e "${RED}═══════════════════════════════════════════════════════════${NC}"
fi

exit $EXIT_CODE
