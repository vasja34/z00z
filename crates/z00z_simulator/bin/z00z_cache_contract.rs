use std::{env, io, path::PathBuf, process};

use z00z_simulator::scenario_1::support::cleanup_contract;

fn main() {
    if let Err(err) = run() {
        eprintln!("z00z_cache_contract: {err}");
        process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    match args.next().as_deref() {
        None | Some("--help") | Some("-h") => {
            print_usage();
            Ok(())
        }
        Some("emit-repo-cache-paths") => emit_repo_cache_paths(args),
        Some(cmd) => Err(format!("unknown command: {cmd}")),
    }
}

fn emit_repo_cache_paths(mut args: impl Iterator<Item = String>) -> Result<(), String> {
    let flag = args
        .next()
        .ok_or_else(|| "missing --repo-root argument".to_string())?;
    if flag != "--repo-root" {
        return Err(format!("unknown argument: {flag}"));
    }

    let repo_root = args
        .next()
        .ok_or_else(|| "missing value after --repo-root".to_string())?;
    if let Some(extra) = args.next() {
        return Err(format!("unexpected trailing argument: {extra}"));
    }

    cleanup_contract::emit_repo_cache_paths(&PathBuf::from(repo_root), io::stdout().lock())
}

fn print_usage() {
    println!(
        "Usage:\n  cargo run -p z00z_simulator --bin z00z_cache_contract -- \\\n    emit-repo-cache-paths --repo-root /abs/path/to/repo"
    );
}
