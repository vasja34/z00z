#!/usr/bin/env bash

# ./scripts/remove_target.sh crates
# ./scripts/remove_target.sh .
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: scripts/remove_target.sh <directory>

Runs from the given directory and removes all recursively nested directories named "target".
EOF
}

if [[ $# -ne 1 ]]; then
  echo "[ERROR] Missing directory argument"
  usage
  exit 1
fi

start_dir="$1"

if [[ ! -d "$start_dir" ]]; then
  echo "[ERROR] Directory '$start_dir' does not exist"
  exit 1
fi

start_dir="$(cd "$start_dir" && pwd)"

echo "[INFO] Searching for 'target' directories under $start_dir"

found=false
while IFS= read -r -d '' dir; do
  found=true
  echo "[INFO] Removing $dir"
  rm -rf "$dir"
done < <(find "$start_dir" -depth -type d -name target -print0)

if [[ "$found" = false ]]; then
  echo "[INFO] No 'target' directories found under $start_dir"
fi