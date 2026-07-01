# Project Brief

## 🎯 Purpose

Z00Z is a privacy-focused Rust workspace for a stateless blockchain architecture
with confidential transaction flows, off-chain ownership logic, and optional
traceable operating modes for regulated or enterprise scenarios.

## 📦 Scope

- Core protocol logic in `crates/z00z_core`
- Cryptographic wrappers and Tari-backed primitives in `crates/z00z_crypto`
- Shared abstractions in `crates/z00z_utils`
- Storage, simulator, wallets, runtime, networking, telemetry, and rollup-node
  crates in the workspace
- Supporting scripts, docs, configs, deployment assets, and website materials

## ✅ Primary Outcomes

- Preserve privacy-oriented protocol behavior without breaking auditability
  options where explicitly enabled
- Keep business logic aligned with the `z00z_utils` one-source-of-truth
  abstractions for I/O, config, serialization, time, logging, metrics, and RNG
- Maintain safe, typed, well-documented Rust APIs with strong verification gates
- Keep vendor Tari sources isolated and read-only

## 🧭 Core Constraints

- Repository artifacts must be written in English
- `crates/z00z_crypto/tari/` is protected vendor code and must not be modified
- New work must follow the Z00Z Design Foundation and repository instructions
- Quality gates matter: formatting, clippy, tests, and targeted verification are
  part of normal delivery

## 📈 Success Signals

- Workspace changes compile and pass the relevant verification path
- Architectural boundaries remain intact
- Documentation stays synchronized with behavior and public APIs
- The memory bank remains current enough for a future session to resume work
  without rediscovery
