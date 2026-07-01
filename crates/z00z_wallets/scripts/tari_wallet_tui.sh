#!/bin/bash
#
# Z00Z Wallet Launcher Script
# Automatically sets up wallet paths according to wallets_config.yaml
#

set -e

# Get script directory
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
WALLETS_ROOT="$( cd "${SCRIPT_DIR}/.." && pwd )"

# Read default network from wallets_config.yaml if available
CONFIG_FILE="${WALLETS_ROOT}/src/wallets_config.yaml"
DEFAULT_NETWORK="devnet"
if [ -f "${CONFIG_FILE}" ]; then
    # Try to extract default network using grep/sed
    DEFAULT_NETWORK=$(grep -A 3 "^defaults:" "${CONFIG_FILE}" | grep "network:" | sed 's/.*network: *"\([^"]*\)".*/\1/' || echo "devnet")
fi

# Z00Z network (can be overridden by env var)
Z00Z_NETWORK="${TARI_NETWORK:-${DEFAULT_NETWORK}}"

# Map Z00Z networks to Tari networks
case "${Z00Z_NETWORK}" in
    devnet)
        TARI_NETWORK_NAME="esmeralda"
        ;;
    testnet)
        TARI_NETWORK_NAME="esmeralda"
        ;;
    mainnet)
        TARI_NETWORK_NAME="mainnet"
        ;;
    # Allow direct Tari network names
    esmeralda|nextnet|stagenet|igor|mainnet)
        TARI_NETWORK_NAME="${Z00Z_NETWORK}"
        ;;
    *)
        echo "Unknown network: ${Z00Z_NETWORK}"
        echo "Valid Z00Z networks: devnet, testnet, mainnet"
        echo "Valid Tari networks: esmeralda, nextnet, stagenet, igor, mainnet"
        exit 1
        ;;
esac

# Use Z00Z network name in base path (Tari will create subdirectory with Tari network name)
BASE_PATH="${WALLETS_ROOT}/outputs/${Z00Z_NETWORK}"
WALLET_BIN="${SCRIPT_DIR}/minotari_console_wallet"

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Create outputs directory if it doesn't exist
if [ ! -d "${BASE_PATH}" ]; then
    echo -e "${YELLOW}Creating wallet directory: ${BASE_PATH}${NC}"
    mkdir -p "${BASE_PATH}"
fi

# Check if binary exists
if [ ! -f "${WALLET_BIN}" ]; then
    echo -e "${YELLOW}Error: Wallet binary not found at ${WALLET_BIN}${NC}"
    echo "Please build the wallet first: cargo build --release -p minotari_console_wallet --features grpc"
    exit 1
fi

# Print launch info
echo -e "${GREEN}================================================${NC}"
echo -e "${GREEN}Z00Z Wallet Launcher${NC}"
echo -e "${GREEN}================================================${NC}"
echo "Z00Z Network:   ${Z00Z_NETWORK}"
echo "Tari Network:   ${TARI_NETWORK_NAME}"
echo "Base Path:      ${BASE_PATH}"
echo "Binary:         ${WALLET_BIN}"
echo -e "${GREEN}================================================${NC}"
echo

# Launch wallet
# Note: Tari will automatically create a subdirectory with the network name (e.g., esmeralda/)
export TARI_NETWORK="${TARI_NETWORK_NAME}"
"${WALLET_BIN}" --base-path "${BASE_PATH}" --network "${TARI_NETWORK_NAME}" "$@"
