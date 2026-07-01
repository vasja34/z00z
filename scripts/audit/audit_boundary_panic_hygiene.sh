#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

python3 - <<'PY'
from pathlib import Path
import re
import sys

root = Path.cwd()
targets = [
    "crates/z00z_storage/src/settlement/store.rs",
    "crates/z00z_wallets/src/services/wallet_session_manager.rs",
    "crates/z00z_wallets/src/services/wallet_actions_backup.rs",
    "crates/z00z_wallets/src/key/receiver_keys_secret.rs",
    "crates/z00z_wallets/src/tx/spend_verification.rs",
    "crates/z00z_networks/rpc/src/wasm_client.rs",
]

pattern = re.compile(r"\b(?:unwrap|expect)\s*\(|panic!\s*\(")


def trim_prod(text: str) -> str:
    marker = "\n#[cfg(test)]"
    if marker in text:
        text = text.split(marker, 1)[0]
    lines = []
    for line in text.splitlines():
        stripped = line.lstrip()
        if stripped.startswith("//"):
            continue
        lines.append(line)
    return "\n".join(lines)


failures: list[str] = []
for rel_path in targets:
    text = trim_prod((root / rel_path).read_text())
    for match in pattern.finditer(text):
        line_no = text[: match.start()].count("\n") + 1
        failures.append(f"{rel_path}:{line_no}:{match.group(0)}")

if failures:
    print("boundary panic hygiene drift detected:", file=sys.stderr)
    for failure in failures:
        print(f"  {failure}", file=sys.stderr)
    raise SystemExit(1)

print("boundary panic hygiene audit passed.")
PY
