#!/usr/bin/env bash

z00z_target_root() {
  local root="${1:?repository root is required}"
  root="${root%/}"
  printf '%s/target\n' "$root"
}

z00z_workspace_target_dir() {
  local root="${1:?repository root is required}"
  printf '%s/workspace\n' "$(z00z_target_root "$root")"
}

z00z_tool_target_dir() {
  local root="${1:?repository root is required}"
  local name="${2:?tool name is required}"
  printf '%s/tools/%s\n' "$(z00z_target_root "$root")" "$name"
}

z00z_hax_target_dir() {
  local root="${1:?repository root is required}"
  printf '%s/hax\n' "$(z00z_target_root "$root")"
}

z00z_fuzz_target_dir() {
  local root="${1:?repository root is required}"
  local name="${2:?fuzz namespace is required}"
  printf '%s/fuzz/%s\n' "$(z00z_target_root "$root")" "$name"
}
