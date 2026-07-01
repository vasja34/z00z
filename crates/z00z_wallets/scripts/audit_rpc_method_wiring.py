#!/usr/bin/env python3
"""Audit RPC method wiring across three layers.

This script scans the `crates/z00z_wallets` codebase and builds a mapping:

  JSON-RPC method string (e.g. "wallet.list")
    -> RPC trait method (e.g. `list_wallets`)
      -> RPC implementation method body (`*_impl.rs`)
        -> Wallet service calls (`self.service.<method>(...)`)

It also reports potential "dead" methods:
- RPC methods declared in traits but not registered in dispatcher wiring (strict: wallet.* and app.*)
- RPC methods whose impl does not call into `WalletService` at all (likely stubbed / not wired)
- `WalletService` methods never referenced from any RPC impl via `self.service.*`

Notes:
- This is a static scan (regex + lightweight brace matching). It intentionally avoids
  compiling the crate or using a Rust parser to keep it simple and fast.
- Output is JSON by default.

Exit codes:
- 0: No errors detected (warnings may still exist)
- 2: Errors detected

python3 [audit_rpc_method_wiring.py](http://_vscodecontentref_/3) --csv-out [mapping.csv](http://_vscodecontentref_/4) --md-out [mapping.md](http://_vscodecontentref_/5) --json-out crates/z00z_wallets/outputs/audit_rpc/mapping.json
"""

from __future__ import annotations

import argparse
import csv
import json
import os
import re
import shutil
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Dict, Iterable, List, Optional, Set, Tuple

OUT_KEEP_ENV = "Z00Z_WALLET_AUDIT_RPC_KEEP"


@dataclass(frozen=True)
class RpcTraitMethod:
    rpc_name: str
    trait_fn: str
    file: str


@dataclass(frozen=True)
class TraitFnDef:
    trait_fn: str
    trait_name: str
    file: str


@dataclass
class RpcImplInfo:
    file: str
    wallet_service_calls: List[str]
    app_service_calls: List[str]
    owner_helper_calls: List[str]


@dataclass(frozen=True)
class DispatcherRegistration:
    rpc_name: str
    wiring_fn: str
    rpc_impl_fn: str
    file: str
    guard_kind: str


@dataclass(frozen=True)
class PrivRouteSpec:
    rpc_name: str
    guard_kind: str
    file: str


LOG_METHOD_NAMES: Set[str] = {
    "log_debug",
    "log_info",
    "log_warn",
    "log_error",
    "log_trace",
}


def _read_text(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def _keep_prefixes(env_key: str) -> List[Path]:
    raw = os.environ.get(env_key, "")
    prefixes: List[Path] = []
    for chunk in raw.replace("\n", ";").split(";"):
        text = chunk.strip()
        if not text:
            continue
        path = Path(text)
        if path.is_absolute() or any(part == ".." for part in path.parts):
            raise SystemExit(f"{env_key} must contain only relative output prefixes: {text}")
        normalized = Path(*[part for part in path.parts if part not in ("", ".")])
        if not normalized.parts:
            raise SystemExit(f"{env_key} must not contain empty prefixes")
        prefixes.append(normalized)
    return prefixes


def _starts_with(path: Path, prefix: Path) -> bool:
    try:
        path.relative_to(prefix)
        return True
    except ValueError:
        return False


def _clear_dir_contents(root: Path, rel_dir: Path, keep: List[Path]) -> None:
    for entry in root.iterdir():
        rel = Path(entry.name) if rel_dir == Path(".") else rel_dir / entry.name
        if any(_starts_with(rel, prefix) for prefix in keep):
            continue
        if entry.is_dir() and any(_starts_with(prefix, rel) for prefix in keep):
            _clear_dir_contents(entry, rel, keep)
            if not any(entry.iterdir()):
                entry.rmdir()
            continue
        if entry.is_dir():
            shutil.rmtree(entry)
        else:
            entry.unlink()


def _prepare_default_out_dir(default_out_dir: Path, outputs: List[Path]) -> None:
    default_root = default_out_dir.resolve()
    if not any(_starts_with(path, default_root) for path in outputs):
        return
    default_root.mkdir(parents=True, exist_ok=True)
    _clear_dir_contents(default_root, Path("."), _keep_prefixes(OUT_KEEP_ENV))


def _find_workspace_root(start: Path) -> Path:
    """Find the workspace root by walking up until we hit the repo root.

    Heuristic: a workspace root must contain `Cargo.toml` and `crates/`.
    """

    current = start.resolve()
    for candidate in [current, *current.parents]:
        if (candidate / "Cargo.toml").exists() and (candidate / "crates").is_dir():
            return candidate

    # Fallback for expected layout: `<root>/crates/z00z_wallets/scripts/<this file>`
    # This keeps the script working even if the heuristic above fails.
    try:
        return start.resolve().parents[3]
    except Exception:
        return start.resolve().parent


def parse_rpc_traits(methods_dir: Path) -> List[RpcTraitMethod]:
    trait_methods: List[RpcTraitMethod] = []

    method_attr = re.compile(r"^\s*#\[\s*method\s*\(\s*name\s*=\s*\"([^\"]+)\"\s*\)\s*\]\s*$")
    fn_sig = re.compile(r"^\s*(?:async\s+)?fn\s+([a-zA-Z0-9_]+)\b")

    for path in sorted(methods_dir.glob("*.rs")):
        if path.name in {"mod.rs"}:
            continue
        if path.name.endswith("_impl.rs"):
            continue

        pending_rpc_name: Optional[str] = None
        for line in _read_text(path).splitlines():
            m = method_attr.match(line)
            if m:
                pending_rpc_name = m.group(1)
                continue

            if pending_rpc_name is not None:
                fnm = fn_sig.match(line)
                if fnm:
                    trait_methods.append(
                        RpcTraitMethod(
                            rpc_name=pending_rpc_name,
                            trait_fn=fnm.group(1),
                            file=str(path),
                        )
                    )
                    pending_rpc_name = None

        # If we ended the file with a dangling #[method], keep it visible in output.
        if pending_rpc_name is not None:
            trait_methods.append(
                RpcTraitMethod(
                    rpc_name=pending_rpc_name,
                    trait_fn="<missing fn signature>",
                    file=str(path),
                )
            )

    return trait_methods


def parse_trait_fn_defs(methods_dir: Path) -> List[TraitFnDef]:
    """Parse all RPC trait function names, even if they lack #[method] attributes.

    Some traits (e.g. app/crypto/scan/storage) are written as plain Rust traits
    without jsonrpsee #[method(name = "...")] attributes. The dispatcher wiring
    still binds JSON-RPC method strings to these trait methods, so we need this
    to build a complete audit.
    """

    trait_defs: List[TraitFnDef] = []
    trait_decl = re.compile(r"\bpub\s+trait\s+([a-zA-Z0-9_]+)\b")
    fn_sig = re.compile(r"\b(?:async\s+)?fn\s+([a-zA-Z0-9_]+)\b")

    for path in sorted(methods_dir.glob("*.rs")):
        if path.name == "mod.rs":
            continue
        if path.name.endswith("_impl.rs"):
            continue

        text = _read_text(path)

        for m in trait_decl.finditer(text):
            trait_name = m.group(1)
            block = _extract_fn_block(text, m.start())
            if not block:
                continue

            body = text[block[0] : block[1]]
            for fnm in fn_sig.finditer(body):
                trait_defs.append(
                    TraitFnDef(
                        trait_fn=fnm.group(1),
                        trait_name=trait_name,
                        file=str(path),
                    )
                )

    return trait_defs


def parse_dispatcher_wiring(path: Path) -> Dict[str, List[DispatcherRegistration]]:
    """Return rpc_name -> list of dispatcher registration details.

    We capture:
    - the rpc method string
    - the wiring function containing the registration (e.g. register_asset_methods)
    - the RPC impl function called inside the handler closure (e.g. send_asset)
    - the wiring file

    This is a static scan using simple regex + brace matching.
    """

    text = _read_text(path)

    fn_def = re.compile(r"\bpub\s+fn\s+([a-zA-Z0-9_]+)\b")
    reg = re.compile(
        r"dispatcher\.(?:register_method|register_typed)\(\s*\"([^\"]+)\"\s*,",
        re.MULTILINE,
    )
    call = re.compile(r"\b([a-zA-Z_][a-zA-Z0-9_]*)\s*\.\s*([a-zA-Z0-9_]+)\s*\(")

    mapping: Dict[str, List[DispatcherRegistration]] = {}

    for fm in fn_def.finditer(text):
        wiring_fn = fm.group(1)
        block = _extract_fn_block(text, fm.start())
        if not block:
            continue

        body = text[block[0] : block[1]]
        matches = list(reg.finditer(body))
        for idx, m in enumerate(matches):
            rpc_name = m.group(1)
            block_end = matches[idx + 1].start() if idx + 1 < len(matches) else len(body)
            reg_block = body[m.start() : block_end]
            guard_kind = ""
            if "typed_handler_cap(" in reg_block:
                if "verify_no_touch_cap(session)" in reg_block or "verify_rotate_cap(session)" in reg_block:
                    guard_kind = "no_touch"
                elif "verify_touch_cap(session)" in reg_block:
                    guard_kind = "touch"
                else:
                    guard_kind = "cap"
            reg_calls = [
                (cm.group(1), cm.group(2))
                for cm in call.finditer(reg_block)
                if cm.group(1) == "rpc" or cm.group(1).endswith("_rpc")
            ]
            if reg_calls:
                _, fn_name = reg_calls[-1]
                mapping.setdefault(rpc_name, []).append(
                    DispatcherRegistration(
                        rpc_name=rpc_name,
                        wiring_fn=wiring_fn,
                        rpc_impl_fn=fn_name,
                        file=str(path),
                        guard_kind=guard_kind,
                    )
                )

    return mapping


def parse_priv_route_specs(path: Path) -> Dict[str, PrivRouteSpec]:
    if not path.exists():
        return {}

    text = _read_text(path)
    spec = re.compile(
        r'PrivRouteSpec\s*\{\s*rpc:\s*"([^"]+)",\s*guard:\s*PrivRouteGuard::([A-Za-z]+)\s*,?\s*\}',
        re.MULTILINE,
    )

    mapping: Dict[str, PrivRouteSpec] = {}
    for match in spec.finditer(text):
        rpc_name = match.group(1)
        guard_kind = match.group(2)
        if guard_kind == "NoTouch":
            guard_kind = "no_touch"
        else:
            guard_kind = guard_kind.lower()
        mapping[rpc_name] = PrivRouteSpec(
            rpc_name=rpc_name,
            guard_kind=guard_kind,
            file=str(path),
        )

    return mapping


def parse_dispatcher_wiring_files(paths: Iterable[Path]) -> Dict[str, List[DispatcherRegistration]]:
    """Parse dispatcher registrations from multiple wiring files.

    The RPC surface is split across several modules (wallet/app/etc). We merge them.
    """

    merged: Dict[str, List[DispatcherRegistration]] = {}
    for path in paths:
        if not path.exists():
            continue
        part = parse_dispatcher_wiring(path)
        for rpc_name, regs in part.items():
            merged.setdefault(rpc_name, []).extend(regs)
    return merged


def _z00z_wallet_path(wallets_crate: Path, raw_path: Optional[str]) -> str:
    """Convert an absolute/relative repo path to a canonical workspace-relative path."""

    if not raw_path:
        return ""

    try:
        workspace = wallets_crate.resolve().parents[1]
        rel = Path(raw_path).resolve().relative_to(workspace)
        return rel.as_posix()
    except Exception:
        return Path(raw_path).as_posix()


def _join(items: Iterable[str]) -> str:
    # Use a semicolon separator to avoid naive CSV parsers treating the cell as multiple columns.
    return "; ".join([i for i in items if i])


def _choose_primary_service_call(
    calls: List[str],
    preferred_name: Optional[str],
    service_to_core_calls: Dict[str, Set[str]],
) -> Optional[str]:
    if not calls:
        return None

    # 1) Best-effort: prefer the call that matches the RPC impl fn name.
    if preferred_name is not None:
        for c in calls:
            if c == preferred_name:
                return c

    # 2) Prefer the first call that reaches core.
    for c in calls:
        if service_to_core_calls.get(c):
            return c

    # 3) Fallback: preserve prior behavior (last call).
    return calls[-1]


def find_dispatcher_wiring_files(wallets_crate: Path) -> List[Path]:
    """Return existing dispatcher wiring module paths.

    New layout: split wiring across wallet/app modules.
    Older layout: single `dispatcher_wiring.rs`.
    """

    candidates = [
        wallets_crate / "src" / "rpc" / "wallet_dispatcher_wiring.rs",
        wallets_crate / "src" / "rpc" / "wallet_dispatcher_routes.rs",
        wallets_crate / "src" / "rpc" / "app_dispatcher_wiring.rs",
        wallets_crate / "src" / "rpc" / "wallet_methods_dispatcher_wiring.rs",
        wallets_crate / "src" / "rpc" / "dispatcher_wiring.rs",
        wallets_crate / "src" / "adapters" / "rpc" / "wallet_dispatcher_wiring.rs",
        wallets_crate / "src" / "adapters" / "rpc" / "wallet_dispatcher_routes.rs",
        wallets_crate / "src" / "adapters" / "rpc" / "app_dispatcher_wiring.rs",
        wallets_crate / "src" / "adapters" / "rpc" / "wallet_methods_dispatcher_wiring.rs",
        wallets_crate / "src" / "adapters" / "rpc" / "dispatcher_wiring.rs",
    ]
    return [p for p in candidates if p.exists()]


def _extract_fn_block(source: str, fn_start: int) -> Optional[Tuple[int, int]]:
    """Return (block_start, block_end) (inclusive-exclusive) for the function body."""

    brace_open = source.find("{", fn_start)
    if brace_open == -1:
        return None

    depth = 0
    i = brace_open
    while i < len(source):
        ch = source[i]
        if ch == "{":
            depth += 1
        elif ch == "}":
            depth -= 1
            if depth == 0:
                return (brace_open, i + 1)
        i += 1

    return None


def parse_rpc_impls(impl_files: Iterable[Path], trait_fn_names: Set[str]) -> Dict[str, RpcImplInfo]:
    """Return trait_fn -> impl info (file + service calls) for methods found."""

    # Match function definitions. We don't try to ensure it's inside an `impl ...` block;
    # we only keep names that are present in the RPC trait method set.
    fn_def = re.compile(r"\basync\s+fn\s+([a-zA-Z0-9_]+)\b|\bfn\s+([a-zA-Z0-9_]+)\b")
    # Determine the type behind common service fields so we can classify calls.
    # Example patterns:
    #   service: Arc<WalletService>,
    #   service: Arc<AppService>,
    #   app_service: Arc<AppService>,
    field_arc_type = re.compile(
        r"\b([a-zA-Z0-9_]+)\s*:\s*Arc\s*<\s*([a-zA-Z0-9_:]+)\s*>"
    )

    def _field_type_by_name(text: str) -> Dict[str, str]:
        mapping: Dict[str, str] = {}
        for m in field_arc_type.finditer(text):
            field = m.group(1)
            typ = m.group(2).split("::")[-1]
            mapping[field] = typ
        return mapping

    def _classify_service_calls(
        body: str,
        field_types: Dict[str, str],
    ) -> Tuple[List[str], List[str], List[str]]:
        # Generic pattern: self.<field>.<method>
        call = re.compile(r"\bself\s*\.\s*([a-zA-Z0-9_]+)\s*\.\s*([a-zA-Z0-9_]+)\b")
        helper_call = re.compile(r"\bself\s*\.\s*([a-zA-Z0-9_]+)\s*\(")
        wallet_calls_in_order: List[str] = []
        app_calls_in_order: List[str] = []
        owner_helper_calls_in_order: List[str] = []

        for m in call.finditer(body):
            field = m.group(1)
            method = m.group(2)

            # These are intentionally not treated as “wiring” calls.
            if method in LOG_METHOD_NAMES:
                continue

            typ = field_types.get(field)
            if typ == "WalletService":
                wallet_calls_in_order.append(method)
            elif typ == "AppService":
                app_calls_in_order.append(method)
            else:
                # Back-compat: if the field is literally named `service`, treat it as wallet.
                if field == "service":
                    wallet_calls_in_order.append(method)

        for m in helper_call.finditer(body):
            method = m.group(1)
            if method in LOG_METHOD_NAMES:
                continue
            if method.endswith(("_impl", "_checked")) or method.startswith("verify_"):
                owner_helper_calls_in_order.append(method)

        # Preserve first-seen order, remove duplicates.
        def _ordered_unique(items: List[str]) -> List[str]:
            seen: Set[str] = set()
            out: List[str] = []
            for item in items:
                if item in seen:
                    continue
                seen.add(item)
                out.append(item)
            return out

        return (
            _ordered_unique(wallet_calls_in_order),
            _ordered_unique(app_calls_in_order),
            _ordered_unique(owner_helper_calls_in_order),
        )

    result: Dict[str, RpcImplInfo] = {}

    for path in sorted(impl_files):
        text = _read_text(path)
        field_types = _field_type_by_name(text)
        for m in fn_def.finditer(text):
            name = m.group(1) or m.group(2)
            if name not in trait_fn_names:
                continue

            block = _extract_fn_block(text, m.start())
            if not block:
                continue

            body = text[block[0] : block[1]]
            wallet_calls, app_calls, owner_helper_calls = _classify_service_calls(body, field_types)

            # Keep first occurrence; duplicate names across impl files should not happen.
            if name not in result:
                result[name] = RpcImplInfo(
                    file=str(path),
                    wallet_service_calls=wallet_calls,
                    app_service_calls=app_calls,
                    owner_helper_calls=owner_helper_calls,
                )

    return result


def parse_wallet_service_methods(wallet_service_path: Path) -> Set[str]:
    text = _read_text(wallet_service_path)

    # Capture public and pub(crate) methods (async or sync)
    meth = re.compile(
        r"\bpub\s*(?:\(crate\))?\s*(?:async\s+)?fn\s+([a-zA-Z0-9_]+)\b"
    )

    return {m.group(1) for m in meth.finditer(text)}


def parse_app_service_methods(app_service_path: Path) -> Set[str]:
    text = _read_text(app_service_path)
    meth = re.compile(r"\bpub\s*(?:async\s+)?fn\s+([a-zA-Z0-9_]+)\b")
    return {m.group(1) for m in meth.finditer(text)}


def parse_core_app_methods(core_app_path: Path) -> Set[str]:
    text = _read_text(core_app_path)
    meth = re.compile(r"\bpub\s*fn\s+([a-zA-Z0-9_]+)\b")
    return {m.group(1) for m in meth.finditer(text)}


def parse_app_calls(app_service_path: Path, core_app_methods: Set[str]) -> Dict[str, Set[str]]:
    if not core_app_methods:
        return {}

    text = _read_text(app_service_path)
    fn_def = re.compile(r"\bpub\s*(?:async\s+)?fn\s+([a-zA-Z0-9_]+)\b")
    method_call = re.compile(r"\bself\s*\.\s*core_app\s*\.\s*([a-zA-Z0-9_]+)\b")

    mapping: Dict[str, Set[str]] = {}
    for m in fn_def.finditer(text):
        service_method = m.group(1)
        block = _extract_fn_block(text, m.start())
        if not block:
            continue

        body = text[block[0] : block[1]]
        calls = {c.group(1) for c in method_call.finditer(body) if c.group(1) in core_app_methods}
        if calls:
            mapping[service_method] = calls

    return mapping


def parse_core_wallet_methods(core_wallet_path: Path) -> Set[str]:
    """Parse public methods declared on the core wallet `Z00ZWallet` type."""

    text = _read_text(core_wallet_path)
    meth = re.compile(r"\bpub\s+(?:async\s+)?fn\s+([a-zA-Z0-9_]+)\b")
    return {m.group(1) for m in meth.finditer(text)}


def parse_wallet_calls(
    wallet_service_path: Path,
    core_wallet_methods: Set[str],
) -> Dict[str, Set[str]]:
    """Return WalletService method -> referenced `Z00ZWallet` method names.

    This uses a conservative heuristic:
    - Extract each `pub fn` / `pub async fn` body in `wallet_service.rs`
    - Find method-call tokens like `.foo(`
    - Keep only those where `foo` is a known `Z00ZWallet` public method
    """

    if not core_wallet_methods:
        return {}

    text = _read_text(wallet_service_path)
    fn_def = re.compile(r"\bpub\s*(?:\(crate\))?\s*(?:async\s+)?fn\s+([a-zA-Z0-9_]+)\b")
    method_call = re.compile(r"\.\s*([a-zA-Z0-9_]+)\s*\(")

    mapping: Dict[str, Set[str]] = {}
    for m in fn_def.finditer(text):
        service_method = m.group(1)
        block = _extract_fn_block(text, m.start())
        if not block:
            continue

        body = text[block[0] : block[1]]
        calls = {
            c.group(1)
            for c in method_call.finditer(body)
            if c.group(1) in core_wallet_methods
        }
        if calls:
            mapping[service_method] = calls

    return mapping


def main() -> int:
    parser = argparse.ArgumentParser(description="Audit RPC wiring across rpc/service/wallet layers")
    parser.add_argument(
        "--workspace",
        default=None,
        help="Workspace root (default: auto-detect from this script location)",
    )
    parser.add_argument(
        "--csv-out",
        default=None,
        help=(
            "Path to write a CSV report (default: ../outputs/audit_rpc/audit_rpc_methods.csv)"
        ),
    )
    parser.add_argument(
        "--md-out",
        default=None,
        help=(
            "Path to write a Markdown report (default: ../outputs/audit_rpc/audit_rpc_methods.md)"
        ),
    )
    parser.add_argument(
        "--json-out",
        default="",
        help=(
            "Path to write the full JSON report (default: ../outputs/audit_rpc/audit_rpc_methods.json)."
        ),
    )
    args = parser.parse_args()

    workspace = (
        Path(args.workspace).resolve()
        if args.workspace
        else _find_workspace_root(Path(__file__))
    )
    wallets_crate = workspace / "crates" / "z00z_wallets"

    methods_dir = wallets_crate / "src" / "rpc"
    dispatcher_wiring_files = find_dispatcher_wiring_files(wallets_crate)
    # Keep a single representative path for backward-compatible reporting.
    dispatcher_wiring = (
        dispatcher_wiring_files[0]
        if dispatcher_wiring_files
        else wallets_crate / "src" / "rpc" / "wallet_dispatcher_wiring.rs"
    )
    wallet_service_path = wallets_crate / "src" / "services" / "wallet_service.rs"
    app_service_path = wallets_crate / "src" / "services" / "app_service.rs"
    core_wallet_path = wallets_crate / "src" / "wallet" / "wallet.rs"
    core_app_path = wallets_crate / "src" / "app" / "mod.rs"

    trait_methods = parse_rpc_traits(methods_dir)
    trait_by_rpc: Dict[str, RpcTraitMethod] = {
        m.rpc_name: m for m in trait_methods if m.trait_fn != "<missing fn signature>"
    }

    trait_fn_defs = parse_trait_fn_defs(methods_dir)
    # IMPORTANT:
    # `parse_rpc_traits()` only captures #[method(name=...)] traits (wallet.*). Other modules
    # (app/crypto/scan/storage) use plain Rust traits without #[method] attributes.
    # We must include those function names too, otherwise their *_impl.rs methods won't be found.
    trait_fn_names: Set[str] = {
        *{m.trait_fn for m in trait_methods if m.trait_fn != "<missing fn signature>"},
        *{d.trait_fn for d in trait_fn_defs},
    }

    fn_def_by_name: Dict[str, TraitFnDef] = {}
    for d in trait_fn_defs:
        # Keep first occurrence; duplicate names across traits should be rare.
        fn_def_by_name.setdefault(d.trait_fn, d)

    dispatcher_map = parse_dispatcher_wiring_files(dispatcher_wiring_files)
    priv_route_specs = parse_priv_route_specs(wallets_crate / "src" / "rpc" / "wallet_dispatcher_routes.rs")

    impl_files = sorted(
        {
            *methods_dir.glob("*_impl.rs"),
            *methods_dir.glob("*_server.rs"),
        }
    )
    impl_info = parse_rpc_impls(impl_files, trait_fn_names)

    wallet_service_methods = parse_wallet_service_methods(wallet_service_path) if wallet_service_path.exists() else set()
    app_service_methods = parse_app_service_methods(app_service_path) if app_service_path.exists() else set()

    core_wallet_methods = parse_core_wallet_methods(core_wallet_path) if core_wallet_path.exists() else set()
    core_app_methods = parse_core_app_methods(core_app_path) if core_app_path.exists() else set()
    service_to_core_calls = (
        parse_wallet_calls(wallet_service_path, core_wallet_methods)
        if wallet_service_path.exists()
        else {}
    )

    app_service_to_core_calls = (
        parse_app_calls(app_service_path, core_app_methods)
        if app_service_path.exists()
        else {}
    )

    rpc_names: List[str] = sorted(set(dispatcher_map.keys()) | set(trait_by_rpc.keys()))

    rows: List[dict] = []
    for rpc_name in rpc_names:
        trait_method = trait_by_rpc.get(rpc_name)
        dispatcher_regs = dispatcher_map.get(rpc_name, [])

        # Prefer explicit #[method(name=...)] mapping when present.
        inferred_trait_fn: Optional[str] = None
        inferred_trait_file: Optional[str] = None
        if trait_method is not None:
            inferred_trait_fn = trait_method.trait_fn
            inferred_trait_file = trait_method.file
        else:
            # Otherwise infer from dispatcher call: rpc.<fn>(...) -> locate fn in any trait file.
            if len(dispatcher_regs) == 1 and dispatcher_regs[0].rpc_impl_fn in fn_def_by_name:
                d = fn_def_by_name[dispatcher_regs[0].rpc_impl_fn]
                inferred_trait_fn = d.trait_fn
                inferred_trait_file = d.file

        impl = impl_info.get(inferred_trait_fn) if inferred_trait_fn else None

        # Derive a "primary" service call for the table.
        # Prefer the call that matches the RPC impl fn, otherwise prefer a call that
        # reaches core, and finally fall back to the last call.
        primary_wallet_service_call: Optional[str] = None
        primary_app_service_call: Optional[str] = None
        if impl is not None:
            primary_wallet_service_call = _choose_primary_service_call(
                impl.wallet_service_calls,
                inferred_trait_fn,
                service_to_core_calls,
            )
            primary_app_service_call = _choose_primary_service_call(
                impl.app_service_calls,
                inferred_trait_fn,
                app_service_to_core_calls,
            )

        core_wallet_primary: List[str] = []
        core_app_primary: List[str] = []
        if primary_wallet_service_call is not None:
            core_wallet_primary = sorted(service_to_core_calls.get(primary_wallet_service_call, set()))
        if primary_app_service_call is not None:
            core_app_primary = sorted(app_service_to_core_calls.get(primary_app_service_call, set()))

        core_wallet_calls: Set[str] = set()
        core_app_calls: Set[str] = set()
        if impl is not None:
            for ws_method in impl.wallet_service_calls:
                core_wallet_calls.update(service_to_core_calls.get(ws_method, set()))
            for app_method in impl.app_service_calls:
                core_app_calls.update(app_service_to_core_calls.get(app_method, set()))

        dispatcher_registered_once = len(dispatcher_regs) == 1
        dispatcher_calls_fn = dispatcher_regs[0].rpc_impl_fn if dispatcher_registered_once else ""
        dispatcher_file = dispatcher_regs[0].file if dispatcher_registered_once else ""
        dispatcher_fn = dispatcher_regs[0].wiring_fn if dispatcher_registered_once else ""

        rows.append(
            {
                "rpc": rpc_name,

                "dispatcher_file": _z00z_wallet_path(wallets_crate, dispatcher_file) if dispatcher_registered_once else "",
                "dispatcher_fn": dispatcher_fn if dispatcher_registered_once else "",

                "rpc_impl_file": _z00z_wallet_path(wallets_crate, impl.file) if impl else "",
                "rpc_impl_fn": inferred_trait_fn or "",

                "service_file": _z00z_wallet_path(wallets_crate, str(wallet_service_path))
                if primary_wallet_service_call is not None
                else (
                    _z00z_wallet_path(wallets_crate, str(app_service_path))
                    if primary_app_service_call is not None
                    else ""
                ),
                "service_fn": primary_wallet_service_call or primary_app_service_call or "",

                "core_file": _z00z_wallet_path(wallets_crate, str(core_wallet_path))
                if primary_wallet_service_call is not None
                else (
                    _z00z_wallet_path(wallets_crate, str(core_app_path))
                    if primary_app_service_call is not None
                    else ""
                ),
                "core_fn": _join(core_wallet_primary) if primary_wallet_service_call is not None else _join(core_app_primary),

                # Extra fields kept for JSON consumers / debugging.
                "rpc_trait_fn": inferred_trait_fn or "<missing trait fn>",
                "trait_file": _z00z_wallet_path(wallets_crate, inferred_trait_file) if inferred_trait_file else "",
                "dispatcher_registered": dispatcher_registered_once,
                "dispatcher_calls_fn": dispatcher_calls_fn,
                "dispatcher_registrations": [
                    {
                        "file": _z00z_wallet_path(wallets_crate, r.file),
                        "wiring_fn": r.wiring_fn,
                        "rpc_impl_fn": r.rpc_impl_fn,
                        "guard_kind": r.guard_kind,
                    }
                    for r in dispatcher_regs
                ],
                "guard_kind": dispatcher_regs[0].guard_kind if dispatcher_registered_once else "",
                "wallet_service_calls": impl.wallet_service_calls if impl else [],
                "app_service_calls": impl.app_service_calls if impl else [],
                "owner_helper_calls": impl.owner_helper_calls if impl else [],
                "core_wallet_methods_reached": sorted(core_wallet_calls),
                "core_app_methods_reached": sorted(core_app_calls),
            }
        )

    # Errors and warnings
    errors: List[str] = []
    warnings: List[str] = []
    notes: List[str] = []

    # 1) Strict registration: wallet.* and app.* methods declared via #[method(name = ...)]
    # must appear exactly once in dispatcher wiring.
    strict_namespaces = ("wallet.", "app.")
    for tm in trait_methods:
        if not tm.rpc_name.startswith(strict_namespaces):
            continue
        regs = dispatcher_map.get(tm.rpc_name, [])
        if len(regs) == 0:
            errors.append(
                f"dispatcher missing registration for {tm.rpc_name} ({tm.trait_fn})"
            )
        elif len(regs) > 1:
            details = "; ".join(
                f"{Path(r.file).name}:{r.wiring_fn}->{r.rpc_impl_fn}" for r in regs
            )
            errors.append(
                f"dispatcher has duplicate registrations for {tm.rpc_name} ({tm.trait_fn}): {details}"
            )

    # 2) Ensure dispatcher calls the expected function for the given rpc name.
    # If a method is registered multiple times, we treat that as an error above,
    # but still validate each registration here to make mismatches visible.
    for rpc_name, regs in sorted(dispatcher_map.items()):
        for reg in regs:
            called_fn = reg.rpc_impl_fn
            trait = trait_by_rpc.get(rpc_name)
            if trait and trait.trait_fn != called_fn:
                if reg.guard_kind and called_fn.endswith("_checked"):
                    continue
                errors.append(
                    f"dispatcher mismatch for {rpc_name}: dispatcher calls {called_fn}, trait defines {trait.trait_fn}"
                )

            # If the trait file does not have an explicit #[method] mapping, at least ensure
            # the called function exists in some RPC trait.
            if trait is None and called_fn not in fn_def_by_name:
                errors.append(
                    f"dispatcher calls unknown rpc fn for {rpc_name}: {called_fn} (no matching trait fn found)"
                )

    for rpc_name, spec in sorted(priv_route_specs.items()):
        regs = dispatcher_map.get(rpc_name, [])
        if len(regs) != 1:
            continue
        guard_kind = regs[0].guard_kind
        if guard_kind != spec.guard_kind:
            errors.append(
                f"privileged route {rpc_name} must use {spec.guard_kind} capability guard, got: {guard_kind or 'missing'}"
            )

    direct_owner_rows = sorted(
        row["rpc"]
        for row in rows
        if row.get("owner_helper_calls")
        and not row.get("wallet_service_calls")
        and not row.get("app_service_calls")
    )
    if direct_owner_rows:
        notes.append(
            f"Direct owner RPC methods resolved without a service layer: {len(direct_owner_rows)}"
        )

    # 3) Warn only on RPC methods that have no service call and no direct owner helper path.
    for row in rows:
        trait_fn = row.get("rpc_trait_fn")
        if not trait_fn or trait_fn == "<missing trait fn>":
            continue

        impl = impl_info.get(str(trait_fn))
        if not impl:
            continue
        if (
            len(impl.wallet_service_calls) == 0
            and len(impl.app_service_calls) == 0
            and len(impl.owner_helper_calls) == 0
        ):
            warnings.append(
                f"rpc method {row.get('rpc')} ({trait_fn}) does not call a service (stub/unwired)"
            )

    # 4) Track referenced service methods.
    # We distinguish between:
    # - all calls (including helper/precheck calls)
    # - "primary" calls (heuristic: last service call in the RPC impl method)
    referenced_wallet_service_methods_all: Set[str] = set()
    referenced_wallet_service_methods_primary: Set[str] = set()
    referenced_app_service_methods_all: Set[str] = set()
    referenced_app_service_methods_primary: Set[str] = set()

    for info in impl_info.values():
        if info.wallet_service_calls:
            referenced_wallet_service_methods_all.update(info.wallet_service_calls)
            referenced_wallet_service_methods_primary.add(info.wallet_service_calls[-1])
        if info.app_service_calls:
            referenced_app_service_methods_all.update(info.app_service_calls)
            referenced_app_service_methods_primary.add(info.app_service_calls[-1])

    unreferenced_wallet_service = sorted(wallet_service_methods - referenced_wallet_service_methods_all)
    unreferenced_app_service = sorted(app_service_methods - referenced_app_service_methods_all)
    if unreferenced_app_service:
        notes.append(
            f"AppService methods not referenced from RPC impls: {len(unreferenced_app_service)}"
        )
    if unreferenced_wallet_service:
        notes.append(
            f"WalletService methods not referenced from RPC impls: {len(unreferenced_wallet_service)}"
        )

    # 5) Warn on primary service calls that do not reach core.
    # Helper/precheck calls frequently appear before the real action call and are not
    # expected to call into core, so we only consider the "primary" call per RPC method.
    referenced_service_without_core = sorted(
        m
        for m in referenced_wallet_service_methods_primary
        if m not in service_to_core_calls
    )
    referenced_app_service_without_core = sorted(
        m
        for m in referenced_app_service_methods_primary
        if m not in app_service_to_core_calls
    )
    if referenced_app_service_without_core:
        notes.append(
            f"AppService primary methods referenced from RPC but not calling core app: {len(referenced_app_service_without_core)}"
        )

    if referenced_service_without_core:
        notes.append(
            f"WalletService primary methods referenced from RPC but not calling core wallet: {len(referenced_service_without_core)}"
        )

    # 7) Warn on AppService calls that look like an alias for a different core method name.
    # Example: an RPC impl calls AppService::switch_to_onionet(), but that service method
    # only calls Z00ZApp::configure_onionet(). This is a likely place for naming mismatches.
    alias_like_app_service_calls: List[str] = []
    for method_name in sorted(referenced_app_service_methods_primary):
        core_calls = sorted(app_service_to_core_calls.get(method_name, set()))
        if len(core_calls) != 1:
            continue
        core_method = core_calls[0]
        if core_method == "rpc_stub_called":
            continue
        if method_name != core_method:
            alias_like_app_service_calls.append(f"{method_name} -> {core_method}")

    if alias_like_app_service_calls:
        warnings.append(
            f"AppService methods called from RPC that map to a different core method name: {len(alias_like_app_service_calls)}"
        )

    # 6) Coverage note on core wallet methods not referenced from WalletService
    referenced_core_wallet_methods: Set[str] = set()
    for calls in service_to_core_calls.values():
        referenced_core_wallet_methods.update(calls)
    unreferenced_core_wallet_methods = sorted(core_wallet_methods - referenced_core_wallet_methods)
    if unreferenced_core_wallet_methods:
        notes.append(
            f"Core wallet (Z00ZWallet) public methods not referenced from WalletService: {len(unreferenced_core_wallet_methods)}"
        )

    out = {
        "workspace": str(workspace),
        "scanned": {
            "rpc_trait_files_dir": _relpath(workspace, methods_dir),
            "rpc_impl_files": [_relpath(workspace, p) for p in sorted(impl_files)],
            "dispatcher_wiring": _relpath(workspace, dispatcher_wiring),
            "dispatcher_wiring_files": [
                _relpath(workspace, p) for p in dispatcher_wiring_files
            ],
            "wallet_service": _relpath(workspace, wallet_service_path),
            "app_service": _relpath(workspace, app_service_path),
            "core_wallet": _relpath(workspace, core_wallet_path),
            "core_app": _relpath(workspace, core_app_path),
        },
        "summary": {
            "rpc_methods": len(rows),
            "dispatcher_registrations": len(dispatcher_map),
            "rpc_impl_methods_found": len(impl_info),
            "wallet_service_methods": len(wallet_service_methods),
            "referenced_wallet_service_methods": len(referenced_wallet_service_methods_all),
            "unreferenced_wallet_service_methods": len(unreferenced_wallet_service),
            "app_service_methods": len(app_service_methods),
            "referenced_app_service_methods": len(referenced_app_service_methods_all),
            "unreferenced_app_service_methods": len(unreferenced_app_service),
            "core_wallet_methods": len(core_wallet_methods),
            "referenced_core_wallet_methods": len(referenced_core_wallet_methods),
            "unreferenced_core_wallet_methods": len(unreferenced_core_wallet_methods),
            "direct_owner_rows": len(direct_owner_rows),
            "referenced_service_without_core": len(referenced_service_without_core),
            "core_app_methods": len(core_app_methods),
            "referenced_app_service_without_core": len(referenced_app_service_without_core),
            "errors": len(errors),
            "warnings": len(warnings),
            "notes": len(notes),
        },
        "rows": rows,
        "errors": errors,
        "warnings": warnings,
        "notes": notes,
        "direct_owner_rows": direct_owner_rows,
        "unreferenced_wallet_service_methods": unreferenced_wallet_service,
        "referenced_service_without_core": referenced_service_without_core,
        "unreferenced_app_service_methods": unreferenced_app_service,
        "referenced_app_service_without_core": referenced_app_service_without_core,
        "alias_like_app_service_calls": alias_like_app_service_calls,
        "unreferenced_core_wallet_methods": unreferenced_core_wallet_methods,
        "core_app_methods": sorted(core_app_methods),
    }

    default_out_dir = wallets_crate / "outputs" / "audit_rpc"
    default_csv = default_out_dir / "audit_rpc_methods.csv"
    default_md = default_out_dir / "audit_rpc_methods.md"
    default_json = default_out_dir / "audit_rpc_methods.json"

    csv_out = Path(args.csv_out).resolve() if args.csv_out else default_csv
    md_out = Path(args.md_out).resolve() if args.md_out else default_md
    json_out = Path(args.json_out).resolve() if args.json_out else default_json

    _prepare_default_out_dir(default_out_dir, [csv_out, md_out, json_out])

    _write_csv(csv_out, rows)
    _write_markdown_report(md_out, out)

    json_out.parent.mkdir(parents=True, exist_ok=True)
    json_out.write_text(json.dumps(out, indent=2, sort_keys=True) + "\n", encoding="utf-8")

    generated_paths: List[Path] = [csv_out, md_out, json_out]

    # Print ONLY generated file paths.
    for p in generated_paths:
        sys.stdout.write(_display_path(workspace, p) + "\n")

    return 2 if errors else 0


def _relpath(workspace: Path, path: Path) -> str:
    try:
        return str(path.resolve().relative_to(workspace.resolve()))
    except Exception:
        return str(path)


def _display_path(workspace: Path, path: Path) -> str:
    """Prefer workspace-relative paths in stdout for readability."""

    try:
        return path.resolve().relative_to(workspace.resolve()).as_posix()
    except Exception:
        return str(path)


def _write_csv(path: Path, rows: List[dict]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)

    def _csv_cell(value: object) -> str:
        # Produce CSV-safe cells without relying on quote wrapping.
        # This keeps the output easy to inspect and avoids "extra columns" in naive parsers.
        text = "" if value is None else str(value)
        text = text.replace("\r\n", " ").replace("\n", " ").replace("\r", " ")
        text = text.replace(",", ";")
        return text

    with path.open("w", encoding="utf-8", newline="") as f:
        writer = csv.DictWriter(
            f,
            fieldnames=[
                "rpc",
                "dispatcher_file",
                "dispatcher_fn",
                "rpc_impl_file",
                "rpc_impl_fn",
                "service_file",
                "service_fn",
                "core_file",
                "core_fn",
            ],
            quoting=csv.QUOTE_NONE,
            escapechar="\\",
        )
        writer.writeheader()
        for row in rows:
            writer.writerow(
                {
                    "rpc": _csv_cell(row.get("rpc")),
                    "dispatcher_file": _csv_cell(row.get("dispatcher_file", "")),
                    "dispatcher_fn": _csv_cell(row.get("dispatcher_fn", "")),
                    "rpc_impl_file": _csv_cell(row.get("rpc_impl_file", "")),
                    "rpc_impl_fn": _csv_cell(row.get("rpc_impl_fn", "")),
                    "service_file": _csv_cell(row.get("service_file", "")),
                    "service_fn": _csv_cell(row.get("service_fn", "")),
                    "core_file": _csv_cell(row.get("core_file", "")),
                    "core_fn": _csv_cell(row.get("core_fn", "")),
                }
            )


def _write_markdown_report(path: Path, report: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)

    summary = report.get("summary", {})
    rows = report.get("rows", [])
    errors = report.get("errors", [])
    warnings = report.get("warnings", [])
    notes = report.get("notes", [])

    lines: List[str] = []
    lines.append("# 📌 RPC Method Wiring Audit")
    lines.append("")
    lines.append("📌 This document is generated from a static code scan.")
    lines.append("")
    lines.append(
        "🎯 Goal: map JSON-RPC methods → Dispatcher → RPC impl → Service or direct owner helper → Core (wallet/app)."
    )
    lines.append("")
    lines.append(
        "⚠️ Note: some live RPC flows intentionally stay on direct owner helpers instead of routing through `WalletService`; those lanes are not treated as stubs."
    )
    lines.append("")

    lines.append("## 📌 Summary")
    lines.append("")
    lines.append(f"📌 RPC methods: {summary.get('rpc_methods')}")
    lines.append(f"📌 Dispatcher registrations: {summary.get('dispatcher_registrations')}")
    lines.append(f"📌 WalletService methods: {summary.get('wallet_service_methods')}")
    lines.append(f"📌 Core wallet (Z00ZWallet) public methods: {summary.get('core_wallet_methods')}")
    lines.append(f"📌 Direct owner RPC rows: {summary.get('direct_owner_rows')}")
    lines.append(f"📌 Errors: {summary.get('errors')}")
    lines.append(f"📌 Warnings: {summary.get('warnings')}")
    lines.append("")

    if errors:
        lines.append("## ❌ Errors")
        lines.append("")
        for e in errors:
            lines.append(f"- {e}")
        lines.append("")

    if warnings:
        lines.append("## ⚠️ Warnings")
        lines.append("")
        for w in warnings:
            lines.append(f"- {w}")
        lines.append("")

    if notes:
        lines.append("## 📌 Notes")
        lines.append("")
        for n in notes:
            lines.append(f"- {n}")
        lines.append("")

    lines.append("## 📌 Mapping Table")
    lines.append("")
    lines.append(
        "| RPC method | Dispatcher file | Dispatcher fn | RPC impl file | RPC impl fn | Service file | Service fn | Core file | Core fn |"
    )
    lines.append("|---|---|---|---|---|---|---|---|---|")

    for row in rows:
        lines.append(
            "| "
            + " | ".join(
                [
                    str(row.get("rpc", "")),
                    str(row.get("dispatcher_file", "")),
                    str(row.get("dispatcher_fn", "")),
                    str(row.get("rpc_impl_file", "")),
                    str(row.get("rpc_impl_fn", "")),
                    str(row.get("service_file", "")),
                    str(row.get("service_fn", "")),
                    str(row.get("core_file", "")),
                    str(row.get("core_fn", "")),
                ]
            )
            + " |"
        )

    content = "\n".join(lines) + "\n"
    path.write_text(content, encoding="utf-8")


if __name__ == "__main__":
    raise SystemExit(main())
