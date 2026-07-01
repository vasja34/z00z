#!/usr/bin/env python3
"""
cargo_build.py - Automated Rust Crate Builder

This script automatically discovers all Rust crates in the 'crates/' directory,
manages their configurations in 'scripts/cargo_build_config.yaml', and builds
them according to per-crate settings.

Features:
- Recursive crate discovery with unique naming for duplicates
- Automatic config updates for new/removed crates
- Per-crate build options: build, clean, mode (debug/release/profile), tests, benches
- Alphabetical sorting of crates
- Version and description extraction from Cargo.toml

Usage:
    python scripts/cargo_build.py

Config file: scripts/cargo_build_config.yaml
- Each crate has options: path, name, version, description, build, clean, mode, run_tests, run_benches

The script will update the config with new crates found and remove entries for deleted crates.
"""

import glob
import os
import re
import subprocess
import sys
import time
from collections import OrderedDict
from pathlib import Path

try:
    import tomllib
except ModuleNotFoundError:  # pragma: no cover - Python < 3.11 fallback
    import tomli as tomllib

import yaml

ROOT_PATH = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CONFIG_FILE = os.path.join(ROOT_PATH, "scripts", "cargo_build_config.yaml")
VERSIONS_FILE = os.path.join(ROOT_PATH, "versions.yaml")
WARNINGS_FILE = os.path.join(ROOT_PATH, ".temp", "todo", "warnings.md")
WORKSPACE_MANIFEST = os.path.join(ROOT_PATH, "Cargo.toml")
LIBTEST_SLOW_TEST_LINE = re.compile(
    r"^test .+ has been running for over \d+ seconds$"
)


def should_suppress_output_line(line: str) -> bool:
    normalized = line.rstrip("\r\n")
    return bool(LIBTEST_SLOW_TEST_LINE.match(normalized))


def find_all_crates():
    workspace_member_paths = load_workspace_member_paths()
    all_crates = {}
    name_counts = {}
    crates_dir = Path(ROOT_PATH) / "crates"

    for root in workspace_member_paths:
        cargo_toml = root / "Cargo.toml"
        with cargo_toml.open("r", encoding="utf-8") as f:
            content = f.read()
            if "[package]" not in content:
                continue

        rel_path = os.path.relpath(root, crates_dir)
        basename = os.path.basename(rel_path)
        if basename in name_counts:
            name_counts[basename] += 1
            unique_name = f"{basename}{name_counts[basename]}"
        else:
            name_counts[basename] = 1
            unique_name = basename
        all_crates[unique_name] = rel_path

    return all_crates


def load_workspace_member_paths():
    with open(WORKSPACE_MANIFEST, "rb") as f:
        manifest = tomllib.load(f)

    members = manifest.get("workspace", {}).get("members", [])
    member_paths = []

    for pattern in members:
        abs_pattern = os.path.join(ROOT_PATH, pattern)
        matches = sorted(glob.glob(abs_pattern))
        if not matches:
            raise FileNotFoundError(
                f"Workspace member pattern from Cargo.toml did not match any path: {pattern}"
            )

        for match in matches:
            match_path = Path(match)
            cargo_toml = match_path / "Cargo.toml"
            if cargo_toml.is_file():
                member_paths.append(match_path)

    return member_paths


def build_crate(crate_name, config):
    path = config.get("path")
    real_crate_name = config.get("name", crate_name)
    build_flag = config.get("build", False)
    clean = config.get("clean", False)
    mode = config.get("mode", "release")
    run_tests = config.get("run_tests", False)
    run_benches = config.get("run_benches", False)
    dev_only = bool(config.get("dev_only", False))

    if not build_flag:
        print(f"Skipping {crate_name} (build disabled)")
        return

    if dev_only and os.environ.get("Z00Z_BUILD_DEV_ONLY") != "1":
        print(
            f"Error: {crate_name} is marked dev_only but Z00Z_BUILD_DEV_ONLY=1 is not set"
        )
        sys.exit(1)

    print(f"Building {crate_name} ({real_crate_name}) at {path} in {mode} mode")
    original_dir = os.getcwd()
    os.chdir(path)

    def format_elapsed(seconds):
        minutes, secs = divmod(int(seconds), 60)
        hours, minutes = divmod(minutes, 60)
        if hours:
            return f"{hours}h {minutes}m {secs}s"
        if minutes:
            return f"{minutes}m {secs}s"
        return f"{secs}s"

    def run_command(cmd, phase):
        phase_start = time.time()
        print(f"[cargo-build] START crate={crate_name} phase={phase} cmd={cmd}", flush=True)
        proc = subprocess.Popen(
            cmd,
            shell=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
            bufsize=1,
        )
        assert proc.stdout is not None
        for line in proc.stdout:
            if should_suppress_output_line(line):
                continue
            print(line, end="", flush=True)
        proc.wait()
        if proc.returncode != 0:
            elapsed = format_elapsed(time.time() - phase_start)
            print(
                f"[cargo-build] FAIL crate={crate_name} phase={phase} elapsed={elapsed}",
                flush=True,
            )
            print(f"Error during {phase} for {crate_name}")
            sys.exit(1)
        elapsed = format_elapsed(time.time() - phase_start)
        print(
            f"[cargo-build] DONE crate={crate_name} phase={phase} elapsed={elapsed}",
            flush=True,
        )

    if clean:
        print(f"Cleaning {crate_name}")
        run_command("cargo clean", "clean")

    build_cmd = "cargo build"
    if mode == "release":
        build_cmd += " --release"
    elif mode == "debug":
        pass  # default
    else:
        build_cmd += f" --profile {mode}"

    run_command(build_cmd, "build")

    if run_tests:
        print(f"Running tests for {crate_name}")
        test_cmd = "cargo test"
        if mode == "release":
            test_cmd += " --release"
        run_command(test_cmd, "test")

    if run_benches:
        print(f"Running benches for {crate_name}")
        bench_cmd = "cargo bench --no-run"
        run_command(bench_cmd, "bench")

    os.chdir(original_dir)


def main():
    start_time = time.time()

    # Ensure warnings directory exists
    os.makedirs(os.path.dirname(WARNINGS_FILE), exist_ok=True)

    # Clear warnings file
    with open(WARNINGS_FILE, "w") as f:
        f.write("# Warnings from cargo build\n\n")

    if not os.path.isfile(CONFIG_FILE):
        print(f"Error: Config file {CONFIG_FILE} not found")
        sys.exit(1)

    with open(CONFIG_FILE, "r") as f:
        original_config = yaml.safe_load(f)

    # Find all crates in crates/ directory
    all_crates = find_all_crates()

    def default_config_for(rel_path: str) -> dict:
        # Default behavior should be safe: new Z00Z crates are built/tested,
        # while newly discovered vendored Tari crates are treated as dev-only
        # and are opt-in via config + Z00Z_BUILD_DEV_ONLY=1.
        if rel_path.startswith("z00z_crypto/tari/"):
            return {
                "build": False,
                "clean": False,
                "mode": "release",
                "run_tests": False,
                "run_benches": False,
                "dev_only": True,
            }

        return {
            "build": True,
            "clean": True,
            "mode": "release",
            "run_tests": True,
            "run_benches": True,
        }

    crates = original_config.get("crates", {}).copy()
    new_crates_added = False
    removed_crates = []

    # Add new crates with default config
    for crate_name, rel_path in all_crates.items():
        if crate_name not in crates:
            path = f"crates/{rel_path}"
            cargo_toml = os.path.join(ROOT_PATH, path, "Cargo.toml")
            real_crate_name = crate_name  # default
            if os.path.exists(cargo_toml):
                with open(cargo_toml, "r") as f:
                    content = f.read()
                    match = re.search(r'^name\s*=\s*"([^"]+)"', content, re.MULTILINE)
                    if match:
                        real_crate_name = match.group(1)
            print(f"Adding new crate: {crate_name}")
            defaults = default_config_for(rel_path)
            crates[crate_name] = {
                "path": path,
                "name": real_crate_name,
                "version": "0.1.0",  # Will be updated below
                "description": f"{crate_name} crate",
                **defaults,
            }
            new_crates_added = True

    # Remove crates that no longer exist
    crates_to_remove = []
    for crate_name in crates:
        if crate_name not in all_crates:
            crates_to_remove.append(crate_name)
            removed_crates.append(crate_name)
    for crate_name in crates_to_remove:
        del crates[crate_name]
        print(f"Removing crate: {crate_name} (no longer found in crates/)")

    # Update versions and descriptions from Cargo.toml files for all crates
    for crate_name, crate_config in crates.items():
        path = crate_config.get("path")
        if path:
            cargo_toml = os.path.join(ROOT_PATH, path, "Cargo.toml")
            if os.path.isfile(cargo_toml):
                with open(cargo_toml, "r") as f:
                    content = f.read()
                    # Update version only for new crates or if not set
                    if crate_name in [
                        c
                        for c in all_crates
                        if c not in original_config.get("crates", {})
                    ] or not crate_config.get("version"):
                        match = re.search(
                            r'^version\s*=\s*"([^"]+)"', content, re.MULTILINE
                        )
                        if match:
                            crate_config["version"] = match.group(1)
                    # Update description only for new crates or if not set
                    if crate_name in [
                        c
                        for c in all_crates
                        if c not in original_config.get("crates", {})
                    ] or not crate_config.get("description"):
                        desc_match = re.search(
                            r'^description\s*=\s*"([^"]+)"', content, re.MULTILINE
                        )
                        if desc_match:
                            crate_config["description"] = desc_match.group(1)
                    # Update crate_name
                    name_match = re.search(
                        r'^name\s*=\s*"([^"]+)"', content, re.MULTILINE
                    )
                    if name_match:
                        crate_config["name"] = name_match.group(1)

    # Sort crates alphabetically
    sorted_crates = OrderedDict(sorted(crates.items()))

    # Update config
    new_config = original_config.copy()
    new_config["crates"] = dict(sorted_crates)

    # Save updated config only if structural changes
    if new_crates_added or removed_crates:
        with open(CONFIG_FILE, "w") as f:
            yaml.dump(new_config, f, default_flow_style=False, sort_keys=False)
        print("Config updated and saved.")
    else:
        print("No structural changes to config.")

    # Update versions.yaml
    if os.path.isfile(VERSIONS_FILE):
        with open(VERSIONS_FILE, "r") as f:
            versions_data = yaml.safe_load(f) or {}
    else:
        versions_data = {}

    # Preserve total_version
    total_version = versions_data.get(
        "total_version", {"version": "1.0.0", "description": "my description"}
    )

    # Update crates section
    new_crates_versions = {}
    for crate_name, crate_config in sorted_crates.items():
        new_crates_versions[crate_name] = {
            "path": crate_config["path"],
            "name": crate_config["name"],
            "version": crate_config["version"],
        }

    versions_data = {"total_version": total_version, "crates": new_crates_versions}

    with open(VERSIONS_FILE, "w") as f:
        yaml.dump(versions_data, f, default_flow_style=False, sort_keys=False)
    print("Versions file updated.")

    if new_crates_added:
        print(
            "New crates added to config. Please review and adjust configurations if needed."
        )

    if removed_crates:
        print(f"Removed crates from config: {', '.join(removed_crates)}")

    for crate_name, crate_config in sorted_crates.items():
        build_crate(crate_name, crate_config)

    end_time = time.time()
    elapsed = end_time - start_time
    print(f"Build process completed in {elapsed:.2f} seconds")
    # Ensure we are back in the project root
    os.chdir(ROOT_PATH)


if __name__ == "__main__":
    main()
