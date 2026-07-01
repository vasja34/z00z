#!/bin/bash

# Determine the script directory and project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"

# Change to the project root
cd "$ROOT_DIR"

# Run the Python script
python3 scripts/cargo_build.py