#!/usr/bin/env bash
# gen_tree.sh
#
# Generate a hierarchical tree structure of a given directory.
# Uses classic connectors ├──, └──, │ plus folder/file icons 📁 / 📄.
#
# USAGE:
#   ./gen_tree.sh <directory> [output-file]
#
# EXAMPLES:
#   ./scripts/gen_tree.sh ./crates
#   ./scripts/gen_tree.sh ./src project_tree.txt
#   ./scripts/gen_tree.sh crates/z00z_core
#
# OUTPUT:
#   - A hierarchical tree is written into the output file (default: TREE.txt).
#   - A flat list of all files/folders is written into TREE_FLAT.txt.

set -e

DIR="${1:-.}"          # directory to scan (default: current directory)
OUT="${2:-TREE.txt}"   # output file (default: TREE.txt)

# Show help if requested
if [[ "$1" == "-h" || "$1" == "--help" ]]; then
  grep "^# " "$0" | sed 's/^# //'
  exit 0
fi

# Check if directory exists
if [ ! -d "$DIR" ]; then
  echo "Error: directory '$DIR' not found."
  exit 1
fi

# Recursive function to print directory tree
print_tree() {
  local path="$1"
  local prefix="$2"

  # Collect entries (dirs + files)
  local entries=()
  for d in "$path"/*; do
    [ -e "$d" ] && entries+=("$d")
  done

  local count=${#entries[@]}
  local i=0

  for e in "${entries[@]}"; do
    i=$((i+1))
    local name=$(basename "$e")
    local connector="├──"
    local new_prefix="$prefix│   "
    if [ $i -eq $count ]; then
      connector="└──"
      new_prefix="$prefix    "
    fi

    if [ -d "$e" ]; then
      echo "${prefix}${connector} 📁 $name/"
      print_tree "$e" "$new_prefix"
    else
      echo "${prefix}${connector} 📄 $name"
    fi
  done
}

# Generate hierarchical tree into OUT
{
  echo "📁 $(basename "$DIR")/"
  print_tree "$DIR" ""
} > "$OUT"

# Generate flat list
FLAT="${OUT%.txt}_FLAT.txt"
find "$DIR" -type d -printf "📁 %P/\n" -o -type f -printf "📄 %P\n" > "$FLAT"

echo "Tree written to $OUT"
echo "Flat list written to $FLAT"

