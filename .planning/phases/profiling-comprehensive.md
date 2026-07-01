You are working inside the Z00Z repository.

Task: Perform a full performance profiling and deep architectural analysis of the command:

```bash
/z00z-full-verify-gate --max-safe-run
```

Primary goal: identify exactly why the full verification gate is slow, what consumes the most time, which work is repeated unnecessarily, and how to redesign the pipeline toward:

```text
memory-first / checkpoint-second architecture
```

The final result MUST be written to:

```text
.planning/phases/z00z-profiling-report.md
```

Do not produce a superficial report. The report must be evidence-based, double-checked, and actionable.

------

# 0. Required source material

Before profiling, inspect and extract all useful information from:

```text
z00z_profiling.md
```

Use it as prior context, but do not blindly trust it. Cross-check every claim against the actual scripts, Rust code, test behavior, logs, timings, and build pipeline.

Also inspect:

```text
/z00z-full-verify-gate
scripts/
Makefile
justfile
Cargo.toml
Cargo.lock
.cargo/config.toml
tests/
benches/
src/
crates/
.planning/
```

Adapt paths to the real repository layout.

------

# 1. Main questions to answer

Analyze the full execution of:

```bash
/z00z-full-verify-gate --max-safe-run
```

Answer the following:

1. Which stages take the most wall-clock time?
2. Which stages take the most CPU time?
3. Which stages are I/O-bound?
4. Which stages are blocked on locks, global resources, filesystem sync, or database access?
5. Which stages rebuild or re-run work that could be reused?
6. Which artifacts are regenerated repeatedly but could be generated once?
7. Which tests run longer than 10 seconds?
8. Which long tests are genuinely necessary and which are accidental slowdowns?
9. Which parts can be run in parallel safely?
10. Which parts can become asynchronous?
11. Which parts should remain sequential because of shared state or dependency ordering?
12. Does the simulator rebuild repeatedly instead of compiling once?
13. Are simulator stages passing data through files unnecessarily?
14. Can simulator stages pass data/artifacts in memory and only checkpoint to disk when needed?
15. Are there dead-code stages, obsolete tests, unused fixtures, or redundant verification steps that should be removed?
16. Are `--features test-fast` and `--features wallet_debug_dump` really needed in this gate?
17. Does `wallet_debug_dump` cause excessive JSON/log/artifact generation?
18. Can test settings reduce the number, size, or complexity of generated artifacts without reducing safety?
19. Can builds, tests, fixtures, mockups, assets, rights, proofs, registries, or snapshots be reused?
20. What concrete changes will give the largest speedup with the smallest risk?

------

# 2. Profiling methodology

Use a staged profiling approach.

## 2.1 Baseline timing

Run the full gate and capture:

```bash
time /z00z-full-verify-gate --max-safe-run
```

If possible, also capture:

```bash
/usr/bin/time -v /z00z-full-verify-gate --max-safe-run
```

Record:

```text
real time
user CPU time
system CPU time
max RSS
major/minor page faults
filesystem inputs/outputs
exit code
```

If the command is too heavy, profile individual stages from the script.

Do not guess. Every performance claim must be backed by one of:

```text
measured timing
script/code inspection
logs
Cargo output
test output
strace/perf/flamegraph evidence
repository evidence
```

------

## 2.2 Stage-level breakdown

Instrument or inspect the gate script to produce a table:

```text
stage_name
command
duration
CPU vs I/O indication
rebuild? yes/no
reuses artifacts? yes/no
can run parallel? yes/no
can run async? yes/no
risk of parallelization
optimization proposal
```

Pay special attention to repeated calls to:

```bash
cargo build
cargo test
cargo nextest
cargo run
cargo check
cargo clippy
cargo fmt
cargo doc
cargo metadata
cargo clean
rm -rf target
```

The simulator should be compiled once and reused. The gate must not rebuild the simulator at every stage unless there is a verified reason.

------

## 2.3 Cargo/build profiling

Check whether the gate causes unnecessary rebuilds.

Use or inspect:

```bash
cargo build --timings
cargo test --no-run
cargo nextest run --no-fail-fast
cargo metadata
```

Look for:

```text
multiple rebuilds of the same crates
different feature sets causing rebuilds
different target dirs
different profiles
build scripts rerunning
procedural macros slowing build
cargo clean or target deletion
environment variables invalidating incremental cache
feature combinations preventing reuse
test binaries rebuilt instead of reused
simulator compiled repeatedly through cargo run
```

Specifically verify whether these features are needed:

```text
test-fast
wallet_debug_dump
```

For each feature, answer:

```text
where it is enabled
why it is enabled
which crates it affects
whether it changes compilation cache keys
whether it causes extra artifact generation
whether the gate can run safely without it
whether it should be split into a separate debug-only profile
```

------

# 3. Slow test analysis

Identify every test that runs longer than 10 seconds.

Use available test framework output or instrument manually.

For each slow test, report:

```text
test_name
crate/module
duration
reason for slowness
real bottleneck category
can be optimized?
can be moved to nightly/heavy suite?
can be made deterministic?
can be reduced in parameters?
can reuse fixture/build/artifact?
risk of changing it
recommended action
```

Investigate these likely causes:

## 3.1 Entropy / RNG

Check whether tests use:

```text
/dev/random
OsRng
thread_rng
non-seeded randomness
cryptographic randomness in loops
```

For tests, prefer deterministic seeded RNG unless the test specifically validates entropy behavior.

Look for accidental entropy exhaustion or slow randomness during:

```text
key generation
signature tests
stealth output generation
Bulletproof/Bulletproof+ proof generation
mock wallet generation
asset/right generation
```

Recommendation direction:

```text
Use deterministic seeded RNG for tests.
Reserve OsRng only for integration tests that explicitly validate production entropy behavior.
```

------

## 3.2 Redundant signature verification

Check whether the same owner signature, transaction signature, claim signature, or registry signature is verified repeatedly across simulator stages.

Look for:

```text
same signature verified in multiple stages
verification repeated after artifact already validated
verification repeated after deserialization
signature verification inside nested loops
signature verification per asset when batch verification is possible
```

Potential optimization:

```text
verification cache keyed by signature hash / message hash / public key
stage-level validated artifact marker
batch verification where cryptographically safe
```

Do not weaken security. Only propose caching if the validated input is immutable and domain-separated.

------

## 3.3 Cryptographic bottlenecks

Identify expensive repeated crypto work.

Check for repeated computation of:

```text
Bulletproof+ scalars
Pedersen commitments
range proof generators
Ristretto basepoint tables
hash-to-scalar values
JMT path hashes
Merkle/JMT proof construction
domain-separated transcript initialization
Fiat-Shamir transcript recomputation
stealth ECDH derivations
owner_tag derivation
key image / asset_id derivation
```

Look for opportunities to cache:

```text
static generators
precomputed basepoint tables
domain constants
hash domain prefixes
JMT path nodes
proof fixtures
mock proofs
asset fixtures
rights fixtures
registry snapshots
```

Do not cache secrets in unsafe ways. If caching touches secret material, explicitly analyze memory safety and zeroization implications.

------

## 3.4 Large zeroization overhead

Check whether tests allocate and zeroize large buffers repeatedly:

```text
LockedBytes
Zeroize
Zeroizing<T>
large Vec<u8>
large JSON blobs
encrypted wallet dumps
mock registry dumps
proof buffers
```

Look for:

```text
large temporary buffers zeroized repeatedly
clone-heavy secret containers
unnecessary LockedBytes for public test data
zeroization inside tight loops
```

Potential optimization:

```text
reduce buffer size
reuse buffers
avoid LockedBytes for non-secret test fixtures
separate secret-path tests from public artifact tests
```

Do not remove zeroization for actual secrets.

------

## 3.5 Busy-wait loops

Find polling loops such as:

```rust
loop {
    if condition {
        break;
    }
}
```

or repeated sleep polling:

```rust
loop {
    sleep(...)
}
```

Look for:

```text
polling state files
polling database changes
polling simulator stages
polling inbox readiness
polling artifact availability
polling async task completion
```

Recommendation direction:

```text
Use tokio::sync::Notify, channels, JoinSet, watch, broadcast, oneshot, or proper await points.
Avoid busy-wait loops.
Avoid sleep-based synchronization in tests.
```

------

## 3.6 Heavy debug logging

Check whether slow tests or simulator stages write excessive logs/artifacts.

Look for:

```text
wallet_debug_dump
debug JSON dumps
large registry dumps
full wallet state dumps
full asset store dumps
full proof dumps
repeated pretty_json serialization
tracing logs at debug/trace level
stdout/stderr flooding
```

Measure whether logging is the bottleneck.

Potential optimization:

```text
disable wallet_debug_dump by default
write debug dump only on failure
write compact binary snapshots instead of pretty JSON
sample logs
truncate large fields
use feature-gated debug dumps only in manual diagnostics
```

------

# 4. Lock contention analysis

Investigate whether tests wait on global locks.

Look for:

```text
RedB environment lock
database write transaction lock
global Mutex
static Lazy<Mutex<_>>
global logger lock
serial test attributes
once_cell global state
parking_lot Mutex/RwLock
std::sync::Mutex
tokio::sync::Mutex
file locks
flock
```

Specific questions:

1. Are tests serialized because they share the same RedB environment path?
2. Do tests use the same temp directory or same wallet DB path?
3. Does the logger use a global Mutex that all tests contend on?
4. Does tracing/logging block concurrent test output?
5. Are write transactions held too long?
6. Are read transactions accidentally blocking writes?
7. Are locks held across await points?
8. Are locks held while doing filesystem I/O or crypto work?
9. Are locks used for data that could be immutable after initialization?
10. Can tests use per-test isolated tempdirs and DBs?

Recommendations should distinguish:

```text
safe parallelism
unsafe parallelism
requires test isolation
requires refactor
```

------

# 5. I/O and fsync overhead

Investigate filesystem overhead.

Look for calls to:

```text
fsync
sync_all
sync_data
flush
File::create
OpenOptions
tempfile persist
rename
copy
remove_dir_all
walkdir
serde_json::to_writer_pretty
write_all large buffers
redb transactions
```

Specific questions:

1. Is fsync called unnecessarily in tests?
2. Does RedB sync every transaction?
3. Are tests creating real durable wallet databases when in-memory/temp mode would be enough?
4. Are simulator stages writing artifacts to disk only to immediately read them back?
5. Are artifacts reserialized between stages instead of passed in memory?
6. Are full registries rewritten when only one definition changed?
7. Are full wallet states dumped repeatedly?
8. Is pretty JSON used in hot paths?
9. Are large fixture directories regenerated every run?
10. Does the gate delete caches or tempdirs too aggressively?

Recommendation direction:

```text
test mode should use relaxed durability where safe
memory-first state passing between simulator stages
checkpoint only at defined boundaries
write debug artifacts only on failure or when explicitly requested
reuse generated fixtures
reuse mockups/assets/rights/proofs
```

------

# 6. Algorithmic complexity analysis

Search for accidental O(N^2) or worse behavior.

Investigate:

```text
AssetStore search API
wallet asset lookup
rights lookup
mock registry lookup
definition registry updates
JMT path updates
simulator stage artifact matching
deduplication logic
signature verification loops
proof verification loops
serialization loops
```

Look for:

```text
nested loops over assets
linear search inside per-asset loop
clone/sort/dedup repeated in loops
full registry serialization after one change
full asset scan when ID lookup exists
full wallet scan for each transaction
full JMT rebuild when incremental update is possible
recomputing all paths instead of touched paths
```

For each finding, propose:

```text
index
HashMap/BTreeMap
incremental update
memoization
precomputed fixture
batching
streaming serialization
delta encoding
```

------

# 7. Context-switch and task-spawn overhead

Investigate whether the gate or simulator creates too many short-lived processes or threads.

Look for:

```text
many cargo subprocesses
many simulator subprocesses
cargo run per stage
short-lived tokio runtimes
thread::spawn in loops
tokio::spawn for tiny tasks
blocking tasks for small CPU work
process-per-stage architecture
shell script pipeline with repeated startups
```

Questions:

1. Can multiple simulator stages run inside one process?
2. Can the simulator compile once and execute all stages via subcommands?
3. Can stages share in-memory context?
4. Can task spawning be replaced by batching?
5. Can expensive initialization be moved to once-per-run?
6. Are there too many small files causing process and I/O overhead?

Target design:

```text
single compiled simulator binary
one process per full simulation run
in-memory stage context
explicit checkpoint boundaries
artifact cache
deterministic fixture registry
failure artifact dump only
```

------

# 8. Artifact reuse analysis

Find all generated artifacts and classify them.

Artifacts may include:

```text
fixtures
mockups
assets
rights
wallet dumps
registry snapshots
proofs
keys
simulator state
test databases
JMT snapshots
checkpoint proofs
transaction bundles
logs
debug JSON
```

For each artifact type, determine:

```text
generated where
consumed where
regenerated how often
deterministic or random
safe to cache?
cache key
invalidated by what?
can be prebuilt?
can be embedded?
can be shared across tests?
must be unique per test?
contains secrets?
requires zeroization?
```

Propose a reuse strategy:

```text
pre-generate deterministic public fixtures
cache immutable crypto parameters
reuse compiled simulator
reuse test binaries
reuse registry snapshots
reuse mock assets/rights where safe
checkpoint stage output once
avoid reserializing full state
```

------

# 9. Dead code and obsolete pipeline checks

Identify slow code that is not actually needed by the verification gate.

Look for:

```text
unused scripts
obsolete simulator stages
duplicated fixture generators
old wallet_debug_dump paths
legacy asset/right formats
unused feature combinations
tests for deprecated APIs
dead integration tests
bench-only code running during tests
```

Do not delete code automatically unless explicitly allowed.

In the report, classify each candidate:

```text
safe to delete
probably obsolete but requires confirmation
keep but move out of max-safe-run
keep and optimize
```

------

# 10. Parallelism plan

Produce a concrete parallelization plan.

Classify stages into:

```text
parallel-safe now
parallel-safe after per-test tempdir isolation
parallel-safe after DB isolation
parallel-safe after removing global logger lock
must remain sequential
```

For tests, check whether they can run with:

```bash
cargo nextest run --jobs N
cargo test -- --test-threads=N
```

But do not assume parallelism is safe. Verify shared resources first.

Possible blockers:

```text
shared RedB path
shared fixture directory
global logger
global RNG
static mutable state
fixed ports
fixed filenames
shared environment variables
serial_test attributes
filesystem lock files
```

------

# 11. Async architecture plan

Analyze whether the simulator and gate can benefit from async execution.

Look for operations that can become async:

```text
independent artifact generation
independent proof fixture generation
independent wallet setup
independent registry validation
independent JMT proof checks
log collection
non-dependent stage preparation
```

But separate CPU-bound crypto from async I/O.

Recommendations should specify:

```text
use async for I/O orchestration
use rayon or controlled worker pool for CPU-bound crypto
avoid spawning tiny tasks
use JoinSet only for meaningful jobs
limit concurrency
do not hold locks across await
```

------

# 12. Memory-first / checkpoint-second redesign

Propose a concrete redesign from current file-based state passing to hybrid memory-first/checkpoint-second.

Required design target:

```text
SimulatorRunContext {
    config
    deterministic_rng
    fixture_cache
    build_artifact_paths
    registry_snapshot
    wallet_state
    asset_store
    rights_store
    jmt_state
    proof_cache
    stage_outputs
    timing_trace
}
```

Stages should pass structured data in memory.

Disk checkpoints should happen only:

```text
at phase boundaries
on failure
when explicitly requested by debug flag
when required for reproducibility
when producing final artifacts
```

Report expected benefits:

```text
less JSON serialization
less fsync
less file polling
less artifact regeneration
less process startup
less lock contention
better test determinism
better profiling visibility
```

Also list risks:

```text
more memory usage
harder post-mortem debugging if no failure dump
cache invalidation bugs
secret material lifetime
need careful zeroization policy
```

------

# 13. Required report structure

Write the final report to:

```text
.planning/phases/z00z-profiling-report.md
```

The report MUST contain these sections:

```markdown
# Z00Z Full Verify Gate Profiling Report

## 1. Executive Summary
- Top 5 confirmed bottlenecks
- Estimated speedup potential
- Highest-impact low-risk fixes

## 2. Baseline Measurements
- Command used
- Environment
- Total runtime
- CPU/user/system time
- Memory
- I/O stats
- Notes and limitations

## 3. Stage-Level Timing Table
| Stage | Command | Time | CPU/I/O/Lock/Build | Rebuild? | Reuses artifacts? | Parallel-safe? | Recommendation |

## 4. Slow Tests > 10 Seconds
| Test | Crate/Module | Time | Root Cause | Optimization | Risk |

## 5. Build and Cargo Reuse Analysis
- Rebuild causes
- Feature-set issues
- Simulator compile reuse
- Test binary reuse
- Cargo cache invalidation

## 6. Feature Flags Analysis
### test-fast
### wallet_debug_dump
For each:
- Where enabled
- Why enabled
- Cost
- Safety impact
- Recommendation

## 7. Lock Contention Findings
- RedB locks
- Global Mutex / logger locks
- Shared tempdirs
- Serial tests
- Locks across await
- Recommendations

## 8. I/O and fsync Findings
- fsync/sync_all/flush usage
- RedB persistence behavior
- JSON/debug dump overhead
- File-based state passing
- Recommendations

## 9. Cryptographic Bottlenecks
- Bulletproof/Bulletproof+ work
- JMT/hash work
- Signature verification
- Stealth/ECDH/tag derivation
- Cacheable vs non-cacheable work

## 10. Algorithmic Complexity Findings
- O(N^2) or repeated full scans
- Registry serialization
- AssetStore search
- JMT path recomputation
- Recommendations

## 11. Context Switch and Task Spawn Overhead
- cargo/process spawning
- tokio/thread spawning
- simulator subprocesses
- recommendations

## 12. Artifact Reuse Plan
| Artifact | Generated By | Consumed By | Cacheable? | Cache Key | Risk | Recommendation |

## 13. Dead Code / Obsolete Stage Candidates
| Candidate | Evidence | Cost | Recommendation | Risk |

## 14. Parallelization Plan
| Stage/Test Group | Current Blocker | Required Change | Safe Jobs | Expected Gain |

## 15. Async / Worker Pool Plan
- What should be async
- What should use rayon/worker pool
- What should remain sync
- Lock safety notes

## 16. Memory-First / Checkpoint-Second Architecture Proposal
- Current model
- Proposed model
- Data flow
- Checkpoint boundaries
- Failure dump policy
- Secret/zeroization policy

## 17. Prioritized Fix Plan
### P0: Immediate low-risk fixes
### P1: Medium refactors
### P2: Larger architecture changes
### P3: Optional heavy optimizations

## 18. Double-Check Log
- Claims verified
- Claims not fully verified
- Assumptions
- Open questions

## 19. Final Recommendation
- What to change first
- What not to change yet
- Expected before/after behavior
```

------

# 14. Important rules

1. Do not guess. Mark uncertain findings as uncertain.
2. Do not weaken cryptographic safety for speed.
3. Do not remove zeroization for real secrets.
4. Do not cache secret-bearing artifacts unless the report explicitly analyzes risk.
5. Do not recommend disabling security tests unless they are moved to a heavier suite.
6. Distinguish:
   - production safety
   - test safety
   - debug-only behavior
   - CI behavior
   - local developer behavior
7. Any recommended parallelism must include shared-state risk analysis.
8. Any recommended artifact reuse must include cache invalidation strategy.
9. Any recommendation to remove code must include evidence that it is dead/obsolete.
10. Before writing the final report, double-check all major findings against code/scripts/logs.

------

# 15. Extra latency hints to investigate

Also investigate these non-obvious causes of slowness:

```text
- cargo clean or target deletion inside gate
- different RUSTFLAGS between stages
- different feature sets causing duplicate builds
- build.rs scripts rerunning due to generated files
- proc-macro crates dominating compile time
- tests sharing one RedB database path
- tests sharing one fixture/output directory
- serial_test forcing global serialization
- global logger/tracing initialization lock
- debug logs blocking stdout/stderr
- pretty JSON serialization in hot paths
- fsync/sync_all in test mode
- tempdir creation/removal overhead
- remove_dir_all on large artifact trees
- full registry rewritten for one changed definition
- full wallet dump written per stage
- full AssetStore scan instead of indexed lookup
- O(N^2) nested asset/right matching
- repeated proof generation instead of deterministic fixtures
- repeated verification of same signature/proof
- repeated JMT path hashing
- repeated domain separator/hash initialization
- large LockedBytes buffers allocated and zeroized repeatedly
- OsRng or /dev/random in tests instead of seeded RNG
- sleep-based polling
- busy-wait loops
- tokio runtime created repeatedly
- thread::spawn/tokio::spawn for tiny tasks
- subprocess-per-stage simulator architecture
- cargo run used repeatedly instead of prebuilt binary
- test binaries rebuilt instead of cargo test --no-run reuse
- wallet_debug_dump writing large JSON artifacts
- test-fast feature not actually reducing work
- debug assertions or tracing enabled in heavy path
- bench-like tests accidentally running in normal gate
- integration tests doing production-size parameters
- fixture generators using production crypto parameters when reduced test params are safe
- missing once_cell/Lazy caches for static public crypto parameters
- locks held while doing crypto or filesystem I/O
- locks held across await points
- fixed TCP ports or fixed filenames causing retries/waits
- hidden retries/timeouts masking real failures
- artificial sleeps used for ordering
```

------

# 16. Expected final output quality

The final `.planning/phases/z00z-profiling-report.md` must be specific enough that another developer can start implementing fixes immediately.

Bad output:

```text
Tests are slow because of I/O. Optimize I/O.
```

Good output:

```text
Stage X writes a 24MB pretty JSON wallet_debug_dump three times.
It is enabled through feature wallet_debug_dump in command Y.
The file is consumed only on failure, but currently generated on every successful run.
Recommendation: generate dump only on failure or when Z00Z_DEBUG_DUMP=1.
Expected gain: remove approximately N seconds and M MB writes per run.
Risk: lower post-mortem visibility on successful runs; acceptable.
```

End the report with a short prioritized implementation checklist.

---

## Bottlenecks

### 🔴 CRITICAL: Redundant Cargo Rebuilds

Current scripts often call `cargo run` or `cargo test` with varying feature flags (`--features test-fast --features wallet_debug_dump`). This invalidates the incremental compilation cache, forcing a partial recompile of the `z00z_wallets` and `z00z_simulator` crates.
**Solution:** Standardize on a single feature set for the verify gate.

### 🟠 HIGH: Serial Stage Execution in Scenario 1

Stages 1 through 12 in `scenario_1` run strictly sequentially. While some stages depend on previous outputs (e.g., Stage 4 needs Stage 3's claims), many verification steps (like checking backup integrity or log sanitization) can be parallelized.
**Solution:** Implement a Directed Acyclic Graph (DAG) for stage execution or use `tokio` for async staging.

### 🟠 HIGH: Large JSON Snapshot I/O

The project relies heavily on `stage_N_snapshot.json` files. As the number of assets grows, `serde_json` serialization/deserialization becomes a significant portion of the runtime.
**Solution:** Use the already implemented `BincodeCodec` for internal simulator handoffs and keep JSON only for final audit reports.

### 🟡 MEDIUM: Redundant Genesis Generation

Every `max-safe-run` regenerates the genesis state. Since the devnet genesis inputs are deterministic and already pinned by the typed genesis manifest plus referenced subfiles, this is wasted CPU time.
**Solution:** Pre-generate a "Golden Genesis Artifact" and reuse it across all tests.

## 3. Optimization Roadmap

### Phase A: Build System & Features

1. **Consolidate Features:** Evaluate if `test-fast` and `wallet_debug_dump` can be merged into a single `dev-verify` feature. `test-fast` is mandatory for cryptographic performance in CI/Verify gates as it reduces the complexity of Bulletproofs+ and JMT hashing.
2. **Single Binary:** Compile the `scenario_1` binary once in `--release` mode and pass stage numbers as arguments, rather than running through `cargo` every time.

### Phase B: Artifact & State Reuse

1. **Fixture Pre-generation:** 
   - Genesis Assets (Stage 1).
   - Initial Wallet Identifiers (Stage 2).
2. **In-Memory Handoff:** Modify the `SimContext` to allow passing `Arc<AssetStore>` between stages in memory when running in a single process, skipping the disk write/read cycle.

### Phase C: Test Suite Optimization (>10s)

Tests exceeding 10 seconds (like `test_reads_during_snapshot_updates` or `test_stress_concurrent_registry`) should be reviewed for:

- **Iteration Counts:** Reduce the number of loops in verify-gate mode while keeping them high for overnight stress tests.
- **Sleep Calls:** Replace `Duration::from_secs(5)` loops with condition-based `WaitGroup` or `channels`.
- **Crypto Complexity:** Ensure `test-fast` is active to use smaller curve groups or reduced proof dimensions where possible.

## 
