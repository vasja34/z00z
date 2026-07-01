#!/usr/bin/env bash
# install_nvk_llm_wiki.sh
#
# Purpose:
#   Install or refresh a stable local checkout of nvk/llm-wiki, register it as a
#   Codex marketplace, and ensure plugin wiki@llm-wiki is installed and enabled.
#
# Prerequisites:
#   - git
#   - codex
#
# Quick start:
#   ./scripts/install_nvk_llm_wiki.sh
#
# Common usage:
#   ./scripts/install_nvk_llm_wiki.sh --skip-update
#   ./scripts/install_nvk_llm_wiki.sh --checkout-dir "$HOME/src/llm-wiki"
#   ./scripts/install_nvk_llm_wiki.sh --dry-run
#
# Options: --checkout-dir, --repo-url, --skip-update, --no-verify, --dry-run
#
# Notes:
#   - Default checkout path: ~/.cache/z00z/llm-wiki
#   - To override the default path, set Z00Z_LLM_WIKI_DIR or pass --checkout-dir
#   - Run with --help for the full option list
set -euo pipefail

REPO_URL_DEFAULT="https://github.com/nvk/llm-wiki.git"
CHECKOUT_DIR_DEFAULT="${Z00Z_LLM_WIKI_DIR:-$HOME/.cache/z00z/llm-wiki}"
MARKETPLACE_NAME="llm-wiki"
PLUGIN_KEY="wiki@${MARKETPLACE_NAME}"

REPO_URL="$REPO_URL_DEFAULT"
CHECKOUT_DIR="$CHECKOUT_DIR_DEFAULT"
DRY_RUN=0
SKIP_UPDATE=0
VERIFY=1

usage() {
  cat <<EOF
Usage: $(basename "$0") [OPTIONS]

Install or refresh the local nvk/llm-wiki checkout, register it in Codex as a
local marketplace source, and ensure plugin ${PLUGIN_KEY} is installed.

Options:
  --checkout-dir <dir>  Local llm-wiki checkout path.
                        Default: ${CHECKOUT_DIR_DEFAULT}
  --repo-url <url>      Git remote to clone/update.
                        Default: ${REPO_URL_DEFAULT}
  --skip-update         Do not fetch/pull if checkout already exists.
  --no-verify           Skip final Codex verification checks.
  --dry-run             Print actions without executing them.
  -h, --help            Show this help.

Environment:
  Z00Z_LLM_WIKI_DIR     Override the default checkout path.

Examples:
  ./scripts/install_nvk_llm_wiki.sh
  ./scripts/install_nvk_llm_wiki.sh --checkout-dir "\$HOME/src/llm-wiki"
  ./scripts/install_nvk_llm_wiki.sh --dry-run --skip-update
EOF
}

log() {
  printf '[install_nvk_llm_wiki] %s\n' "$*" >&2
}

die() {
  printf '[install_nvk_llm_wiki] ERROR: %s\n' "$*" >&2
  exit 1
}

require_cmd() {
  command -v "$1" >/dev/null 2>&1 || die "Required command not found: $1"
}

run() {
  if [[ "$DRY_RUN" -eq 1 ]]; then
    printf '[dry-run] ' >&2
    printf '%q ' "$@" >&2
    printf '\n' >&2
    return 0
  fi

  "$@"
}

normalize_checkout_dir() {
  if [[ "$CHECKOUT_DIR" != /* ]]; then
    CHECKOUT_DIR="$PWD/$CHECKOUT_DIR"
  fi
}

validate_checkout_dir() {
  case "$CHECKOUT_DIR" in
    */.codex/.tmp/marketplaces/*)
      die "Refusing checkout inside Codex tmp marketplaces: $CHECKOUT_DIR. Use a stable path such as ${CHECKOUT_DIR_DEFAULT}."
      ;;
  esac
}

validate_existing_checkout() {
  local origin_url

  [[ -d "$CHECKOUT_DIR/.git" ]] || die "Checkout exists but is not a git repository: $CHECKOUT_DIR"

  origin_url="$(git -C "$CHECKOUT_DIR" remote get-url origin 2>/dev/null || true)"
  case "$origin_url" in
    https://github.com/nvk/llm-wiki.git|https://github.com/nvk/llm-wiki|git@github.com:nvk/llm-wiki.git|ssh://git@github.com/nvk/llm-wiki.git)
      ;;
    *)
      die "Existing checkout has unexpected origin '$origin_url'. Use --checkout-dir with another path."
      ;;
  esac
}

clone_or_update_checkout() {
  if [[ ! -e "$CHECKOUT_DIR" ]]; then
    log "Cloning $REPO_URL into $CHECKOUT_DIR"
    run mkdir -p "$(dirname "$CHECKOUT_DIR")"
    run git clone "$REPO_URL" "$CHECKOUT_DIR"
    return
  fi

  validate_existing_checkout

  if [[ "$SKIP_UPDATE" -eq 1 ]]; then
    log "Skipping git update for existing checkout: $CHECKOUT_DIR"
    return
  fi

  if [[ -n "$(git -C "$CHECKOUT_DIR" status --porcelain)" ]]; then
    die "Existing checkout is dirty: $CHECKOUT_DIR. Commit/stash changes or use --skip-update."
  fi

  local branch
  branch="$(git -C "$CHECKOUT_DIR" rev-parse --abbrev-ref HEAD)"

  if [[ "$branch" == "HEAD" ]]; then
    die "Existing checkout is in detached HEAD state: $CHECKOUT_DIR. Use --skip-update or switch to a branch first."
  fi

  log "Updating existing checkout on branch '$branch'"
  run git -C "$CHECKOUT_DIR" fetch --tags --prune origin
  run git -C "$CHECKOUT_DIR" pull --ff-only origin "$branch"
}

marketplace_exists() {
  codex plugin marketplace list | awk -v name="$MARKETPLACE_NAME" 'NR > 1 && $1 == name { found = 1 } END { exit found ? 0 : 1 }'
}

marketplace_root() {
  codex plugin marketplace list | awk -v name="$MARKETPLACE_NAME" 'NR > 1 && $1 == name { print $2; exit }'
}

plugin_status_line() {
  codex plugin list | awk -v key="$PLUGIN_KEY" '$1 == key { print; exit }'
}

configure_marketplace() {
  local current_root

  if marketplace_exists; then
    current_root="$(marketplace_root)"

    if [[ "$current_root" == "$CHECKOUT_DIR" ]]; then
      log "Codex marketplace '${MARKETPLACE_NAME}' is already registered from $CHECKOUT_DIR"
      return
    fi

    die "Marketplace '${MARKETPLACE_NAME}' already exists at $current_root. Refusing to remove it automatically because Codex deletes marketplace roots on removal."
  fi

  log "Registering local Codex marketplace from $CHECKOUT_DIR"
  run codex plugin marketplace add "$CHECKOUT_DIR"
}

install_plugin() {
  local status_line

  status_line="$(plugin_status_line)"

  if [[ "$status_line" == *"installed, enabled"* ]]; then
    log "Plugin already installed and enabled: $PLUGIN_KEY"
    return
  fi

  if [[ "$status_line" == *"installed"* ]]; then
    log "Plugin is installed but not fully enabled; reinstalling $PLUGIN_KEY"
    run codex plugin remove "$PLUGIN_KEY"
  fi

  log "Installing Codex plugin $PLUGIN_KEY"
  run codex plugin add "$PLUGIN_KEY"
}

verify_installation() {
  [[ "$VERIFY" -eq 1 ]] || return 0
  [[ "$DRY_RUN" -eq 0 ]] || {
    log "Skipping verification in dry-run mode"
    return 0
  }

  local status_line
  status_line="$(plugin_status_line)"

  [[ -n "$status_line" ]] || die "Verification failed: plugin line not found for $PLUGIN_KEY"
  [[ "$status_line" == *"installed, enabled"* ]] || die "Verification failed: expected 'installed, enabled' status, got: $status_line"

  marketplace_exists || die "Verification failed: marketplace '${MARKETPLACE_NAME}' is missing"

  log "Verification passed"
  log "Checkout: $CHECKOUT_DIR"
  log "Plugin:   $status_line"
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --checkout-dir)
      [[ $# -ge 2 ]] || die "Missing value for --checkout-dir"
      CHECKOUT_DIR="$2"
      shift 2
      ;;
    --repo-url)
      [[ $# -ge 2 ]] || die "Missing value for --repo-url"
      REPO_URL="$2"
      shift 2
      ;;
    --skip-update)
      SKIP_UPDATE=1
      shift
      ;;
    --no-verify)
      VERIFY=0
      shift
      ;;
    --dry-run)
      DRY_RUN=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      die "Unknown option: $1"
      ;;
  esac
done

require_cmd git
require_cmd codex

normalize_checkout_dir
validate_checkout_dir
clone_or_update_checkout
configure_marketplace
install_plugin
verify_installation

log "Done"
log "You can now use @wiki in Codex with checkout: $CHECKOUT_DIR"
