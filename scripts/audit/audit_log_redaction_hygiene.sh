#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

python3 - <<'PY'
from pathlib import Path
import re
import sys

root = Path.cwd()
failures: list[str] = []

wasm_src = (root / "crates/z00z_networks/rpc/src/wasm_client.rs").read_text()
for banned in [
    'log.info(&format!("Connecting to worker: {}", worker_url));',
    'log.info(&format!("Connected to worker: {}", worker_url));',
    'logger.debug(&format!("RPC call: {} with params: {}", method, params));',
    'logger.debug(&format!("RPC response: {}", response));',
]:
    if banned in wasm_src:
        failures.append(f"crates/z00z_networks/rpc/src/wasm_client.rs: banned raw log `{banned}`")

for required in [
    'log.info("Connecting to worker endpoint");',
    'log.info("Connected to worker endpoint");',
    'logger.debug(&req_log(method, &params));',
    'logger.debug(&resp_log(&response));',
]:
    if required not in wasm_src:
        failures.append(f"crates/z00z_networks/rpc/src/wasm_client.rs: missing `{required}`")

logging_src = (root / "crates/z00z_wallets/src/rpc/logging_middleware.rs").read_text()
for required in ["summarize_params(method, &params", "summarize_response(method, value"]:
    if required not in logging_src:
        failures.append(
            f"crates/z00z_wallets/src/rpc/logging_middleware.rs: missing `{required}`"
        )

pattern = re.compile(
    r"logger\.(?:debug|info|warn|error)\s*\(&format!\([^)]*\b(?:params|response)\b",
    re.MULTILINE,
)

for rel_dir in ["crates/z00z_networks/rpc/src", "crates/z00z_wallets/src/rpc"]:
    for path in (root / rel_dir).rglob("*.rs"):
        rel_path = path.relative_to(root).as_posix()
        text = path.read_text()
        for match in pattern.finditer(text):
            line_no = text[: match.start()].count("\n") + 1
            if rel_path == "crates/z00z_networks/rpc/src/wasm_client.rs":
                continue
            failures.append(f"{rel_path}:{line_no}: raw params/response log pattern")

if failures:
    print("log redaction hygiene drift detected:", file=sys.stderr)
    for failure in failures:
        print(f"  {failure}", file=sys.stderr)
    raise SystemExit(1)

print("log redaction hygiene audit passed.")
PY
