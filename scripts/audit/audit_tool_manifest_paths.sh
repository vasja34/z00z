#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "$SCRIPT_DIR/../.." && pwd)"

if ! command -v rg >/dev/null 2>&1; then
  echo "ripgrep (rg) is required" >&2
  exit 1
fi

mapfile -t OFFENDERS < <(
  rg -n \
    --glob '*.sh' \
    --glob '*.yml' \
    --glob '*.yaml' \
    --glob 'Makefile' \
    --glob 'justfile' \
    --glob '!**/audit_tool_manifest_paths.sh' \
    -- '--manifest-path[= ]tools/' \
    "$REPO_ROOT/.github" \
    "$REPO_ROOT/scripts"
)

if [[ ${#OFFENDERS[@]} -eq 0 ]]; then
  printf 'tool manifest-path audit passed\n'
  exit 0
fi

printf 'unexpected repo-root cargo --manifest-path tools/... invocations detected:\n' >&2
for offender in "${OFFENDERS[@]}"; do
  printf '  - %s\n' "$offender" >&2
done

cat >&2 <<'EOF'

Policy:
  - vendored tool builds must run from the tool root, or
  - wrappers/scripts must set CARGO_TARGET_DIR to target/tools/<name> or target/hax
  - do not call cargo from repo root with --manifest-path tools/... in tracked automation
EOF
exit 1
