#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

python3 - <<'PY'
from pathlib import Path
import re
import sys

root = Path.cwd()


def derive_traits(text: str, type_name: str) -> set[str]:
    pattern = re.compile(
        rf"#\[derive\(([^]]+)\)\]\s*pub (?:struct|enum)\s+{re.escape(type_name)}\b",
        re.MULTILINE,
    )
    match = pattern.search(text)
    if not match:
        return set()
    return {part.strip() for part in match.group(1).split(",")}


checks = [
    ("crates/z00z_crypto/src/secret.rs", "SecretBytes"),
    ("crates/z00z_crypto/src/kdf/secret_bytes.rs", "SecretBytes32"),
    ("crates/z00z_wallets/src/key/receiver_keys_secret.rs", "ReceiverSecret"),
    ("crates/z00z_wallets/src/key/seed_cipher_types.rs", "CipherSeedContainer"),
    ("crates/z00z_wallets/src/key/seed_cipher_types.rs", "SeedBytes"),
    ("crates/z00z_wallets/src/key/seed_mnemonic.rs", "SeedWords"),
    ("crates/z00z_wallets/src/key/seed_backup_format_phrase.rs", "SeedPhrase24"),
]

failures: list[str] = []
for rel_path, type_name in checks:
    text = (root / rel_path).read_text()
    banned = derive_traits(text, type_name) & {"Debug", "Serialize", "Deserialize"}
    if banned:
        failures.append(
            f"{rel_path}: {type_name} must not derive {', '.join(sorted(banned))}"
        )

session_src = (root / "crates/z00z_wallets/src/rpc/security_types.rs").read_text()
session_debug = re.search(
    r"#\[derive\([^]]*\bDebug\b[^]]*\)\]\s*pub struct SessionToken\b",
    session_src,
    re.MULTILINE,
)
if session_debug:
    failures.append(
        "crates/z00z_wallets/src/rpc/security_types.rs: SessionToken must not derive Debug"
    )
if "impl fmt::Debug for SessionToken" not in session_src:
    failures.append(
        "crates/z00z_wallets/src/rpc/security_types.rs: SessionToken must provide manual Debug"
    )
if "<redacted>" not in session_src:
    failures.append(
        "crates/z00z_wallets/src/rpc/security_types.rs: redacted debug marker is missing"
    )

if failures:
    print("secret type hygiene drift detected:", file=sys.stderr)
    for failure in failures:
        print(f"  {failure}", file=sys.stderr)
    raise SystemExit(1)

print("secret type hygiene audit passed.")
PY
