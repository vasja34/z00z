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
    root / "crates/z00z_core/src",
    root / "crates/z00z_wallets/src",
    root / "crates/z00z_storage/src",
    root / "crates/z00z_runtime",
    root / "crates/z00z_rollup_node/src",
    root / "crates/z00z_networks/rpc/src",
]

allowlisted = {
    "crates/z00z_wallets/src/stealth/output.rs",
    "crates/z00z_wallets/src/rpc/tx_rpc_server_lifecycle.rs",
    "crates/z00z_wallets/src/tx/multi_io.rs",
}

patterns = [
    re.compile(r"\brand::thread_rng\s*\("),
    re.compile(r"\brand::random\s*\("),
    re.compile(r"\bStdRng::(?:seed_from_u64|from_seed)\s*\("),
    re.compile(r"\bChaCha(?:8|12|20)Rng::from_seed\s*\("),
    re.compile(r"\bSmallRng\b"),
]


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
for target in targets:
    for path in target.rglob("*.rs"):
        rel_path = path.relative_to(root).as_posix()
        if "/tari/" in rel_path:
            continue
        if path.name.startswith("test_"):
            continue
        text = trim_prod(path.read_text())
        if rel_path in allowlisted:
            continue
        for pattern in patterns:
            match = pattern.search(text)
            if match:
                line_no = text[: match.start()].count("\n") + 1
                failures.append(f"{rel_path}:{line_no}:{match.group(0)}")
                break

if failures:
    print("crypto RNG hygiene drift detected outside the Phase 065 allowlist:", file=sys.stderr)
    for failure in failures:
        print(f"  {failure}", file=sys.stderr)
    raise SystemExit(1)

print("crypto RNG hygiene audit passed.")
PY
