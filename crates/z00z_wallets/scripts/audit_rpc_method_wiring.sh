#!/usr/bin/env bash
set -euo pipefail

# Wrapper for the RPC wiring audit.
# Generates:
# - ../outputs/audit_rpc/audit_rpc_methods.csv
# - ../outputs/audit_rpc/audit_rpc_methods.md
# RUN: ./crates/z00z_wallets/scripts/audit_rpc_method_wiring.sh

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

WORKSPACE_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"

OUT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)/outputs/audit_rpc"

want_csv_out=true
want_md_out=true
want_workspace=true

for arg in "$@"; do
	case "$arg" in
		--workspace|--workspace=*)
			want_workspace=false
			;;
		--csv-out|--csv-out=*)
			want_csv_out=false
			;;
		--md-out|--md-out=*)
			want_md_out=false
			;;
	esac
done

extra_args=()
if [[ "$want_workspace" == "true" ]]; then
	extra_args+=(--workspace "$WORKSPACE_ROOT")
fi
if [[ "$want_csv_out" == "true" ]]; then
	extra_args+=(--csv-out "$OUT_DIR/audit_rpc_methods.csv")
fi
if [[ "$want_md_out" == "true" ]]; then
	extra_args+=(--md-out "$OUT_DIR/audit_rpc_methods.md")
fi

exec python3 "$SCRIPT_DIR/audit_rpc_method_wiring.py" "${extra_args[@]}" "$@"
