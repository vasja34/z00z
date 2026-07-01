#!/usr/bin/env bash

set -euo pipefail

usage() {
  cat <<'EOF'
Usage: scripts/add_empty_file.sh <directory>

Adds an \`empty_file\` (one newline byte) but only in directories that are empty.
EOF
}

if [[ $# -ne 1 ]]; then
  echo "[ERROR] Directory argument required"
  usage
  exit 1
fi

start_dir="$1"

if [[ ! -d "$start_dir" ]]; then
  echo "[ERROR] Directory '$start_dir' does not exist"
  exit 1
fi

start_dir="$(cd "$start_dir" && pwd)"

echo "[INFO] Scanning directories under $start_dir"

found=false
while IFS= read -r -d '' dir; do
  # Check if the directory already contains entries other than '.' and '..'
  if find "$dir" -mindepth 1 -maxdepth 1 -print -quit | grep -q .; then
    continue
  fi

  target_file="$dir/empty_file"
  if [[ -e "$target_file" ]]; then
    continue
  fi

  found=true
  printf '\n' > "$target_file"
  echo "[INFO] Created empty_file in $dir"
done < <(find "$start_dir" -type d -print0)

if [[ "$found" = false ]]; then
  echo "[INFO] No empty directories found under $start_dir"
fi