use std::{
    fs,
    path::{Path, PathBuf},
};

fn repo_root_from_manifest_dir() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .unwrap_or(&manifest_dir)
        .to_path_buf()
}

fn should_skip_file(path: &Path) -> bool {
    let path_str = path.to_string_lossy();

    if path_str.contains("/crates/z00z_crypto/tari/") {
        return true;
    }

    // Only scan production Rust sources.
    // This avoids false positives from docs, examples, and tests.
    if !path_str.contains("/src/") && !path_str.contains("/bin/") {
        return true;
    }

    // Not Rust source.
    if !path_str.ends_with(".rs") {
        return true;
    }

    // Time internals are allowed to use SystemTime + Duration directly.
    if path_str.contains("/crates/z00z_utils/src/time/") {
        return true;
    }

    false
}

fn collect_rust_files(root: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_rust_files(&path, out);
        } else if !should_skip_file(&path) {
            out.push(path);
        }
    }
}

fn find_forbidden_patterns(content: &str) -> Option<&'static str> {
    // The most common incorrect implementation: millis rounded to micros.
    // Keep this strict to avoid false positives.
    let millis_times_1000 = regex_like_contains_millis_times1000(content);
    if millis_times_1000 {
        return Some("uses unix_timestamp_millis() * 1000 (rounded micros)");
    }

    // Discourage chrono-based epoch micros timestamps in first-party code.
    // (Tari is excluded above.)
    // Require method-call syntax to avoid matching `get_timestamp_micros(`.
    if content.contains(".timestamp_micros(") {
        return Some("uses chrono timestamp_micros() directly");
    }

    // A common alternative hack: compute epoch micros from SystemTime directly.
    if content.contains("duration_since(SystemTime::UNIX_EPOCH)") && content.contains("as_micros()")
    {
        return Some("computes epoch micros from SystemTime directly");
    }

    if content.contains(".unix_timestamp(")
        || content.contains(".unix_timestamp_millis(")
        || content.contains(".unix_timestamp_micros(")
    {
        return Some("uses ambiguous unix_timestamp*() helper directly; migrate to try_* or compat_unix_timestamp*()");
    }

    None
}

fn regex_like_contains_millis_times1000(content: &str) -> bool {
    // We intentionally avoid bringing in a regex dependency into z00z_utils.
    // This is a best-effort whitespace-tolerant search.
    //
    // Matches patterns like:
    //   unix_timestamp_millis() * 1000
    //   unix_timestamp_millis()*1000

    let normalized: String = content.chars().filter(|c| !c.is_whitespace()).collect();

    normalized.contains("unix_timestamp_millis()*1000")
}

#[test]
fn test_time_policy_micros() {
    let root = repo_root_from_manifest_dir();
    let crates_dir = root.join("crates");

    let mut files = Vec::new();
    collect_rust_files(&crates_dir, &mut files);

    let mut violations: Vec<(PathBuf, &'static str)> = Vec::new();

    for file in files {
        let Ok(content) = fs::read_to_string(&file) else {
            continue;
        };

        if let Some(reason) = find_forbidden_patterns(&content) {
            violations.push((file, reason));
        }
    }

    if !violations.is_empty() {
        let mut msg = String::from("Found forbidden time-policy patterns. Use TimeProvider::try_unix_timestamp*() for security-sensitive flows or compat_unix_timestamp*() for explicit compatibility callers.\n");
        for (path, reason) in violations.iter().take(25) {
            msg.push_str(&format!("- {}: {}\n", path.display(), reason));
        }
        if violations.len() > 25 {
            msg.push_str(&format!("... and {} more\n", violations.len() - 25));
        }
        panic!("{msg}");
    }
}
