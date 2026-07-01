use std::{path::PathBuf, sync::OnceLock};

use z00z_utils::{
    config::{ConfigSource, EnvConfig},
    io::{hash_root_inputs, reset_managed_root_once, write_file},
    time::{SystemTimeProvider, TimeProvider},
};

const BENCH_OUTPUT_HASH_SCHEMA: &str = "storage-settlement-bench-output-v2";
const BENCH_OUTPUT_KEEP_ENV: &str = "Z00Z_STORAGE_SETTLEMENT_BENCH_KEEP";
const PROOF_NOTE_SCOPE_ENV: &str = "Z00Z_SETTLEMENT_PROOF_NOTE_SCOPE";
const PROOF_NOTE_COMMAND_ENV: &str = "Z00Z_SETTLEMENT_PROOF_NOTE_COMMAND";
const PROOF_NOTE_FILTER_ENV: &str = "Z00Z_SETTLEMENT_PROOF_NOTE_FILTER";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProofNoteScope {
    Full,
    BatchOnly,
    Skip,
}

impl ProofNoteScope {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Full => "full",
            Self::BatchOnly => "batch_only",
            Self::Skip => "skip",
        }
    }
}

pub struct BenchMeta<'a> {
    bench: &'a str,
    backend_mode: Option<String>,
    bucket_bits: Option<String>,
    bench_mode: Option<String>,
    root_mode: Option<String>,
    baseline: Option<String>,
    proof_note_scope: Option<String>,
    helper: &'a str,
}

impl<'a> BenchMeta<'a> {
    pub fn new(bench: &'a str, helper: &'a str) -> Self {
        Self {
            bench,
            backend_mode: env_opt("Z00Z_SETTLEMENT_BACKEND_MODE"),
            bucket_bits: env_opt("Z00Z_SETTLEMENT_BUCKET_BITS"),
            bench_mode: env_opt("Z00Z_SETTLEMENT_BENCH_MODE"),
            root_mode: env_opt("Z00Z_SETTLEMENT_ROOT_MODE"),
            baseline: env_opt("Z00Z_SETTLEMENT_BASELINE"),
            proof_note_scope: env_opt(PROOF_NOTE_SCOPE_ENV),
            helper,
        }
    }
}

fn env_opt(key: &str) -> Option<String> {
    EnvConfig.get(key).ok().flatten()
}

pub fn should_emit_side_outputs() -> bool {
    !std::env::args().any(|arg| matches!(arg.as_str(), "-h" | "--help" | "--list" | "--version"))
}

pub fn proof_note_scope() -> ProofNoteScope {
    match env_opt(PROOF_NOTE_SCOPE_ENV).as_deref() {
        Some("batch_only") => ProofNoteScope::BatchOnly,
        Some("skip") => ProofNoteScope::Skip,
        _ => ProofNoteScope::Full,
    }
}

pub fn proof_note_command() -> Option<String> {
    env_opt(PROOF_NOTE_COMMAND_ENV)
}

pub fn proof_note_filter() -> Option<String> {
    env_opt(PROOF_NOTE_FILTER_ENV)
}

pub fn write_meta(meta: BenchMeta<'_>) {
    let dir = prepared_out_dir();
    let path = dir.join(meta_name(&meta));
    let body = render_meta(&meta);
    write_file(path, body.as_bytes()).expect("bench meta file");
}

pub fn out_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("outputs/settlement")
}

pub fn write_note(name: &str, body: &str) {
    let dir = prepared_out_dir();
    write_file(dir.join(name), body.as_bytes()).expect("bench note file");
}

fn prepared_out_dir() -> PathBuf {
    let dir = out_dir();
    reset_managed_root_once(
        &dir,
        &bench_output_fingerprint(),
        &[],
        Some(BENCH_OUTPUT_KEEP_ENV),
    )
    .expect("reset bench output dir");
    dir
}

fn bench_output_fingerprint() -> String {
    static VALUE: OnceLock<String> = OnceLock::new();
    VALUE
        .get_or_init(|| {
            let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
            hash_root_inputs(
                BENCH_OUTPUT_HASH_SCHEMA,
                &[
                    root.join("Cargo.toml"),
                    root.join("Cargo.lock"),
                    root.join(".cargo/config.toml"),
                    root.join("crates/z00z_core/Cargo.toml"),
                    root.join("crates/z00z_crypto/Cargo.toml"),
                    root.join("crates/z00z_simulator/Cargo.toml"),
                    root.join("crates/z00z_storage/Cargo.toml"),
                    root.join("crates/z00z_utils/Cargo.toml"),
                    root.join("crates/z00z_wallets/Cargo.toml"),
                ],
                &[
                    root.join("crates/z00z_core/src"),
                    root.join("crates/z00z_crypto/src"),
                    root.join("crates/z00z_simulator/src"),
                    root.join("crates/z00z_storage/benches"),
                    root.join("crates/z00z_storage/scripts"),
                    root.join("crates/z00z_storage/src"),
                    root.join("crates/z00z_utils/src"),
                    root.join("crates/z00z_wallets/src"),
                ],
            )
            .expect("hash settlement bench outputs")
        })
        .clone()
}

fn meta_name(meta: &BenchMeta<'_>) -> String {
    let mut parts = vec![meta.bench.to_string()];
    if let Some(value) = &meta.backend_mode {
        parts.push(value.clone());
    }
    if let Some(value) = &meta.bucket_bits {
        parts.push(format!("bucket_bits_{value}"));
    }
    if let Some(value) = &meta.bench_mode {
        parts.push(value.clone());
    }
    if let Some(value) = &meta.root_mode {
        parts.push(value.clone());
    }
    if let Some(value) = &meta.baseline {
        parts.push(value.clone());
    }
    format!("{}.meta.md", parts.join("_"))
}

fn render_meta(meta: &BenchMeta<'_>) -> String {
    let stamp = SystemTimeProvider.compat_unix_timestamp();
    let mut body = String::new();
    body.push_str("# Storage Settlement Bench Meta\n\n");
    body.push_str(&format!("- bench: `{}`\n", meta.bench));
    body.push_str(&format!("- unix_ts: `{stamp}`\n"));
    body.push_str("- output_dir: `crates/z00z_storage/outputs/settlement`\n");
    if let Some(value) = &meta.backend_mode {
        body.push_str(&format!("- backend_mode: `{value}`\n"));
    }
    if let Some(value) = &meta.bucket_bits {
        body.push_str(&format!("- bucket_bits: `{value}`\n"));
    }
    if let Some(value) = &meta.bench_mode {
        body.push_str(&format!("- bench_mode: `{value}`\n"));
    }
    if let Some(value) = &meta.root_mode {
        body.push_str(&format!("- root_mode: `{value}`\n"));
    }
    if let Some(value) = &meta.baseline {
        body.push_str(&format!("- baseline: `{value}`\n"));
    }
    if let Some(value) = &meta.proof_note_scope {
        body.push_str(&format!("- proof_note_scope: `{value}`\n"));
    }
    body.push_str(&format!("- helper: `{}`\n", meta.helper));
    body
}
