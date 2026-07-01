#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "$SCRIPT_DIR/../.." && pwd)"

is_allowed_target_dir() {
  local rel="$1"

  if [[ "$rel" =~ ^reports(/[^/]+)+/target$ ]]; then
    return 0
  fi

  case "$rel" in
    tools/formal_verification/creusot/target)
      return 0
      ;;
    tools/formal_verification/cargo/registry/src/*/tests/testsuite/*/target)
      return 0
      ;;
    tools/formal_verification/cargo/registry/src/*/cc-*/src/target)
      return 0
      ;;
    tools/formal_verification/cargo/git/checkouts/*/target)
      return 0
      ;;
    tools/formal_verification/rustup/toolchains/*/lib/rustlib/src/*/target)
      return 0
      ;;
  esac

  return 1
}

mapfile -d '' FOUND_TARGET_DIRS < <(
  find "$REPO_ROOT" \
    \( -path "$REPO_ROOT/.git" -o -path "$REPO_ROOT/target" \) -prune \
    -o -type d -name target -print0 | sort -z
)

mapfile -d '' FOUND_ROOT_TARGET_CACHE_ENTRIES < <(
  find "$REPO_ROOT/target" -mindepth 1 -maxdepth 1 \
    \( -type d \( -name debug -o -name release -o -name doc -o -name package -o -name 'flycheck*' \) \
    -o -type f \( -name .rustc_info.json -o -name .future-incompat-report.json \) \) \
    -print0 2>/dev/null | sort -z
)

OFFENDERS=()
for path in "${FOUND_TARGET_DIRS[@]}"; do
  rel="${path#"$REPO_ROOT"/}"
  if is_allowed_target_dir "$rel"; then
    continue
  fi
  OFFENDERS+=("$rel")
done

for path in "${FOUND_ROOT_TARGET_CACHE_ENTRIES[@]}"; do
  OFFENDERS+=("${path#"$REPO_ROOT"/}")
done

if [[ ${#OFFENDERS[@]} -eq 0 ]]; then
  printf 'target layout audit passed\n'
  exit 0
fi

printf 'unexpected build target directories detected outside canonical roots:\n' >&2
for rel in "${OFFENDERS[@]}"; do
  printf '  - %s\n' "$rel" >&2
done
cat >&2 <<'EOF'

Policy:
  - workspace Cargo builds go to target/workspace
  - repo-owned tool builds go to target/tools/<name> or target/hax
  - fuzz builds go to target/fuzz/<name>
  - verifier run reports may use reports/.../target
  - flat root caches like target/debug or target/release are not allowed
  - vendored upstream source fixtures must stay on the allowlist
EOF
exit 1
