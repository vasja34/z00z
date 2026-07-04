#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "$SCRIPT_DIR/../.." && pwd)"

if ! command -v rg >/dev/null 2>&1; then
  echo "ripgrep (rg) is required" >&2
  exit 1
fi

cd "$REPO_ROOT"

SEARCH_ROOTS=(
  .planning
  crates
  wiki
  docs
  scripts
)

RG_BASE=(
  rg
  -l
  -F
  --hidden
  --glob
  !target/**
  --glob
  !.git/**
  --glob
  !.planning/graphs/**
  --glob
  !**/000/**
  --glob
  !wiki/.vitepress/public/**
  --glob
  !wiki/llms-full.txt
  --glob
  !crates/.understand-anything/**
  --glob
  !crates/z00z_core/tests/test_live_guardrails.rs
  --glob
  !scripts/audit/audit_narrowed_wording.sh
)

offenders=()

check_absent() {
  local pattern="$1"
  local -a hits=()
  mapfile -t hits < <("${RG_BASE[@]}" "$pattern" "${SEARCH_ROOTS[@]}" || true)
  if [[ ${#hits[@]} -eq 0 ]]; then
    return
  fi

  offenders+=("forbidden pattern: $pattern")
  for hit in "${hits[@]}"; do
    offenders+=("  $hit")
  done
}

check_allowed() {
  local pattern="$1"
  shift
  local -a allowed=("$@")
  local -a hits=()
  mapfile -t hits < <("${RG_BASE[@]}" "$pattern" "${SEARCH_ROOTS[@]}" || true)
  if [[ ${#hits[@]} -eq 0 ]]; then
    return
  fi

  local hit
  for hit in "${hits[@]}"; do
    local allow_hit=0
    local allowed_path
    for allowed_path in "${allowed[@]}"; do
      if [[ "$hit" == "$allowed_path" ]]; then
        allow_hit=1
        break
      fi
    done

    if [[ $allow_hit -eq 0 ]]; then
      offenders+=("unexpected narrowed-claim hit: $pattern :: $hit")
    fi
  done
}

check_absent "crates/z00z_core/src/assets/assets_config.yaml"
check_absent "canonical regeneration inputs for dev stores"
check_absent 'frozen in `assets_config.yaml`'

check_allowed \
  "still carries no nullifier semantics" \
  ".planning/phases/065-Attack-Surface/065-TODO.md" \
  "crates/z00z_wallets/tests/test_scenario1_semantics.rs"

check_allowed \
  "V2 memo unsupported" \
  ".planning/phases/065-Attack-Surface/065-TODO.md" \
  ".planning/phases/065-Attack-Surface/065-CONTEXT.md" \
  ".planning/phases/065-Attack-Surface/065-09-PLAN.md"

check_allowed \
  "invalid-signature downgrade" \
  ".planning/phases/065-Attack-Surface/065-09-PLAN.md"

check_allowed \
  "invalid-owner-signature downgrade" \
  ".planning/phases/065-Attack-Surface/065-TODO.md" \
  ".planning/phases/065-Attack-Surface/065-CONTEXT.md"

check_allowed \
  'guarded `claim_v1`' \
  ".planning/phases/065-Attack-Surface/065-TODO.md" \
  ".planning/phases/065-Attack-Surface/065-CONTEXT.md" \
  ".planning/phases/065-Attack-Surface/065-09-PLAN.md"

check_allowed \
  "in-memory placeholder only" \
  ".planning/phases/065-Attack-Surface/065-TODO.md"

check_allowed \
  "placeholder-password finding" \
  ".planning/phases/065-Attack-Surface/065-TODO.md"

if [[ ${#offenders[@]} -eq 0 ]]; then
  printf 'phase 065 narrowed wording audit passed\n'
  exit 0
fi

printf 'phase 065 narrowed wording audit failed:\n' >&2
for offender in "${offenders[@]}"; do
  printf '  - %s\n' "$offender" >&2
done
exit 1
