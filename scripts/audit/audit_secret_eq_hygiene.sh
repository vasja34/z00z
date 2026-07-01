#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

python3 - <<'PY'
from pathlib import Path
import sys

root = Path.cwd()

checks = [
    (
        "crates/z00z_wallets/src/services/wallet_session_manager.rs",
        "session.token_hex != token.token",
        "direct session token compare must not remain",
        "ct_eq_token(&session.token_hex, &token.token)",
    ),
    (
        "crates/z00z_wallets/src/tx/spend_verification.rs",
        "receiver_secret.as_bytes() != receiver_keys.reveal_receiver_secret().as_bytes()",
        "direct receiver secret byte compare must not remain",
        ".receiver_secret\n        .ct_eq(receiver_keys.reveal_receiver_secret())",
    ),
    (
        "crates/z00z_wallets/src/redb_store/debug_export.rs",
        "revealed == master_key",
        "direct master key compare must not remain",
        "revealed.ct_eq(master_key).unwrap_u8() == 1",
    ),
]

failures: list[str] = []
for rel_path, banned, banned_msg, required in checks:
    text = (root / rel_path).read_text()
    if banned in text:
        failures.append(f"{rel_path}: {banned_msg}")
    if required not in text:
        failures.append(f"{rel_path}: required constant-time helper missing")

if failures:
    print("secret equality hygiene drift detected:", file=sys.stderr)
    for failure in failures:
        print(f"  {failure}", file=sys.stderr)
    raise SystemExit(1)

print("secret equality hygiene audit passed.")
PY
