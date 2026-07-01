use std::path::Path;

use serde::{Deserialize, Serialize};
use z00z_storage::settlement::{ForestCacheMetrics, ForestSchedulerMetrics};
use z00z_utils::io::save_json;

pub(crate) const STAGE13_STATUS: &str = "hjmt_examples_complete";
pub(crate) const STAGE13_MODE: &str = "generalized_settlement";
pub(crate) const STAGE13_LOG_FILE: &str = "hjmt/stage13_hjmt_examples.log";
pub(crate) const STAGE13_SCHEMA_VERSION: u32 = 1;
pub(crate) const PROOF_SURFACE_SINGLE: &str = "proof_blob_single";
pub(crate) const PROOF_SURFACE_VEC: &str = "proof_blob_vec";
pub(crate) const PROOF_SURFACE_BATCH: &str = "batch_proof_v1";
pub(crate) const PATH_SHAPE_SINGLE: &str = "single";
pub(crate) const PATH_SHAPE_CLUSTERED: &str = "clustered";
pub(crate) const PATH_SHAPE_SCATTERED: &str = "scattered";
pub(crate) const ATOMIC_VERDICT_ACCEPTED: &str = "accepted";
pub(crate) const ATOMIC_VERDICT_INDEPENDENT: &str = "independent";
pub(crate) const SHARD_CONTEXT_NONE: &str = "none";
const REDACTED_FAILURE_DETAILS: &str = "redacted failure details";
const REDACTION_TERMS: &[&str] = &[
    "private key",
    "private_key",
    "private wallet key",
    "wallet private key",
    "owner_sk",
    "seed phrase",
    "seed_phrase",
    "mnemonic",
    "proof witness",
    "witness bytes",
    "witness_bytes",
    "payload bytes",
    "payload_bytes",
    "payload contents",
    "row key",
    "row_key",
    "redb row",
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct RedactedError {
    pub(crate) class: String,
    pub(crate) message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct Stage13ArtifactMeta {
    pub(crate) example_id: String,
    pub(crate) backend_mode: String,
    pub(crate) api_surface: String,
    pub(crate) verifier_status: String,
    pub(crate) typed_error: Option<RedactedError>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct Stage13ExampleArtifact {
    pub(crate) schema_version: u32,
    pub(crate) scenario_id: u32,
    pub(crate) stage: u32,
    pub(crate) example_id: String,
    pub(crate) backend_mode: String,
    pub(crate) api_surface: String,
    pub(crate) verifier_status: String,
    pub(crate) root_generation: u8,
    pub(crate) settlement_state_root_hex: String,
    pub(crate) prior_state_root_hex: Option<String>,
    pub(crate) next_state_root_hex: Option<String>,
    pub(crate) proof_envelope_version: Option<u8>,
    pub(crate) proof_family: String,
    pub(crate) leaf_family: String,
    pub(crate) settlement_path: String,
    pub(crate) terminal_id: String,
    pub(crate) bucket_epoch: Option<u64>,
    pub(crate) bucket_policy_id: Option<String>,
    pub(crate) fee_envelope_id: Option<String>,
    pub(crate) fee_domain: Option<String>,
    pub(crate) transition_binding: Option<String>,
    pub(crate) payer_commitment: Option<String>,
    pub(crate) sponsor_commitment: Option<String>,
    pub(crate) expiry: Option<u64>,
    pub(crate) replay_status: Option<String>,
    pub(crate) artifact_names: Vec<String>,
    pub(crate) proof_size_bytes: Option<usize>,
    pub(crate) verify_time_us: Option<u64>,
    pub(crate) typed_error: Option<RedactedError>,
    pub(crate) present_key_rejection: Option<String>,
    pub(crate) proof_is_ownership: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct Stage13ExamplesReport {
    pub(crate) schema_version: u32,
    pub(crate) scenario_id: u32,
    pub(crate) stage: u32,
    pub(crate) status: String,
    pub(crate) boundary_mode: String,
    pub(crate) backend_modes: Vec<String>,
    pub(crate) root_generation: u8,
    pub(crate) artifact: Stage13ArtifactMeta,
    pub(crate) settlement_state_root_hex: String,
    pub(crate) manifest_file: String,
    pub(crate) artifact_names: Vec<String>,
    pub(crate) examples: Vec<Stage13ExampleArtifact>,
    pub(crate) comparison_rows: Vec<Stage13ProofComparisonRow>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct Stage13ProofSizeEntry {
    pub(crate) schema_version: u32,
    pub(crate) scenario_id: u32,
    pub(crate) stage: u32,
    pub(crate) example_id: String,
    pub(crate) backend_mode: String,
    pub(crate) api_surface: String,
    pub(crate) verifier_status: String,
    pub(crate) root_generation: u8,
    pub(crate) typed_error: Option<RedactedError>,
    pub(crate) proof_size_bytes: usize,
    pub(crate) verify_time_us: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct Stage13ProofSizeReport {
    pub(crate) schema_version: u32,
    pub(crate) scenario_id: u32,
    pub(crate) stage: u32,
    pub(crate) status: String,
    pub(crate) root_generation: u8,
    pub(crate) artifact: Stage13ArtifactMeta,
    pub(crate) entries: Vec<Stage13ProofSizeEntry>,
    pub(crate) comparison_rows: Vec<Stage13ProofComparisonRow>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct Stage13ProofComparisonRow {
    pub(crate) schema_version: u32,
    pub(crate) scenario_id: u32,
    pub(crate) stage: u32,
    pub(crate) row_id: String,
    pub(crate) owner_example_id: String,
    pub(crate) backend_mode: String,
    pub(crate) api_surface: String,
    pub(crate) verifier_status: String,
    pub(crate) typed_error: Option<RedactedError>,
    pub(crate) proof_surface: String,
    pub(crate) proof_family: String,
    pub(crate) leaf_family: String,
    pub(crate) path_count: u32,
    pub(crate) path_shape: String,
    pub(crate) canonical_order: bool,
    pub(crate) atomic_verdict: String,
    pub(crate) shard_context_mode: String,
    pub(crate) root_generation: u8,
    pub(crate) settlement_state_root_hex: String,
    pub(crate) settlement_paths: Vec<String>,
    pub(crate) proof_size_bytes: usize,
    pub(crate) verify_time_us: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct Stage13CacheSchedulerReport {
    pub(crate) schema_version: u32,
    pub(crate) scenario_id: u32,
    pub(crate) stage: u32,
    pub(crate) example_id: String,
    pub(crate) backend_mode: String,
    pub(crate) api_surface: String,
    pub(crate) verifier_status: String,
    pub(crate) root_generation: u8,
    pub(crate) typed_error: Option<RedactedError>,
    pub(crate) settlement_state_root_hex: String,
    pub(crate) cache_hit_count: u64,
    pub(crate) cache_miss_count: u64,
    pub(crate) invalidation_count: u64,
    pub(crate) root_reuse_ratio: f64,
    pub(crate) proof_segment_reuse_ratio: f64,
    pub(crate) scheduler_queue_depth: usize,
    pub(crate) scheduler_backpressure_count: u64,
    pub(crate) deterministic_parent_ordering: bool,
    pub(crate) cache_metrics: ForestCacheMetrics,
    pub(crate) scheduler_metrics: ForestSchedulerMetrics,
}

impl Stage13CacheSchedulerReport {
    pub(crate) fn validate_bounded(&self) -> Result<(), String> {
        if self.verifier_status != "verified" {
            return Err("stage13 cache metrics verifier_status drifted".to_string());
        }
        if self.cache_hit_count + self.cache_miss_count == 0 {
            return Err("stage13 cache metrics lost hit/miss evidence".to_string());
        }
        if !self.deterministic_parent_ordering {
            return Err("stage13 scheduler determinism evidence is missing".to_string());
        }
        if !self.root_reuse_ratio.is_finite()
            || !(0.0..=1.0).contains(&self.root_reuse_ratio)
            || !self.proof_segment_reuse_ratio.is_finite()
            || !(0.0..=1.0).contains(&self.proof_segment_reuse_ratio)
        {
            return Err("stage13 cache reuse ratios drifted out of bounds".to_string());
        }
        if self.cache_hit_count != cache_hits(&self.cache_metrics) {
            return Err("stage13 cache hit aggregation drifted".to_string());
        }
        if self.cache_miss_count != cache_misses(&self.cache_metrics) {
            return Err("stage13 cache miss aggregation drifted".to_string());
        }
        if self.invalidation_count != cache_invalidations(&self.cache_metrics) {
            return Err("stage13 cache invalidation aggregation drifted".to_string());
        }
        if self.scheduler_queue_depth != self.scheduler_metrics.max_queued {
            return Err("stage13 scheduler queue depth drifted".to_string());
        }
        if self.deterministic_parent_ordering != self.scheduler_metrics.last_ordered {
            return Err("stage13 scheduler ordering evidence drifted".to_string());
        }
        if self.scheduler_backpressure_count != self.scheduler_metrics.reject_count {
            return Err("stage13 scheduler backpressure drifted".to_string());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct Stage13ReplayEntry {
    pub(crate) schema_version: u32,
    pub(crate) scenario_id: u32,
    pub(crate) stage: u32,
    pub(crate) example_id: String,
    pub(crate) backend_mode: String,
    pub(crate) api_surface: String,
    pub(crate) verifier_status: String,
    pub(crate) root_generation: u8,
    pub(crate) typed_error: Option<RedactedError>,
    pub(crate) settlement_state_root_hex: String,
    pub(crate) reloaded_settlement_state_root_hex: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct Stage13ReplayRootsReport {
    pub(crate) schema_version: u32,
    pub(crate) scenario_id: u32,
    pub(crate) stage: u32,
    pub(crate) status: String,
    pub(crate) root_generation: u8,
    pub(crate) artifact: Stage13ArtifactMeta,
    pub(crate) store_dir: String,
    pub(crate) replay_entries: Vec<Stage13ReplayEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct Stage13TamperCase {
    pub(crate) schema_version: u32,
    pub(crate) scenario_id: u32,
    pub(crate) stage: u32,
    pub(crate) example_id: String,
    pub(crate) backend_mode: String,
    pub(crate) api_surface: String,
    pub(crate) proof_surface: String,
    pub(crate) verifier_status: String,
    pub(crate) root_generation: u8,
    pub(crate) path_count: Option<u32>,
    pub(crate) path_shape: Option<String>,
    pub(crate) case_id: String,
    pub(crate) typed_error: RedactedError,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct Stage13TamperReport {
    pub(crate) schema_version: u32,
    pub(crate) scenario_id: u32,
    pub(crate) stage: u32,
    pub(crate) status: String,
    pub(crate) root_generation: u8,
    pub(crate) artifact: Stage13ArtifactMeta,
    pub(crate) cases: Vec<Stage13TamperCase>,
}

pub(crate) fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    save_json(path, value).map_err(|e| e.to_string())
}

fn cache_hits(metrics: &ForestCacheMetrics) -> u64 {
    metrics.subtree_root.hits
        + metrics.parent_leaf.hits
        + metrics.terminal_leaf.hits
        + metrics.bucket_derivation.hits
        + metrics.proof_segment.hits
        + metrics.nonexistence.hits
        + metrics.policy_proof.hits
        + metrics.journal_digest.hits
        + metrics.path_index.hits
}

fn cache_misses(metrics: &ForestCacheMetrics) -> u64 {
    metrics.subtree_root.misses
        + metrics.parent_leaf.misses
        + metrics.terminal_leaf.misses
        + metrics.bucket_derivation.misses
        + metrics.proof_segment.misses
        + metrics.nonexistence.misses
        + metrics.policy_proof.misses
        + metrics.journal_digest.misses
        + metrics.path_index.misses
}

fn cache_invalidations(metrics: &ForestCacheMetrics) -> u64 {
    metrics.subtree_root.invalidations
        + metrics.parent_leaf.invalidations
        + metrics.terminal_leaf.invalidations
        + metrics.bucket_derivation.invalidations
        + metrics.proof_segment.invalidations
        + metrics.nonexistence.invalidations
        + metrics.policy_proof.invalidations
        + metrics.journal_digest.invalidations
        + metrics.path_index.invalidations
}

pub(crate) fn report_artifact(
    example_id: impl Into<String>,
    backend_mode: impl Into<String>,
    api_surface: impl Into<String>,
) -> Stage13ArtifactMeta {
    Stage13ArtifactMeta {
        example_id: example_id.into(),
        backend_mode: backend_mode.into(),
        api_surface: api_surface.into(),
        verifier_status: "verified".to_string(),
        typed_error: None,
    }
}

pub(crate) fn redact_error_class<E>(err: &E) -> String
where
    E: std::fmt::Debug,
{
    let raw = format!("{err:?}");
    let trimmed = raw.trim().trim_matches('"');
    let outer = ident_token(trimmed);
    let inner = trimmed
        .split_once('(')
        .map(|(_, rest)| ident_token(rest.trim_start()));
    let fallback = std::any::type_name::<E>()
        .rsplit("::")
        .next()
        .unwrap_or("Error");
    let class = match (outer, inner) {
        (Some(outer), Some(Some(inner))) if outer != inner => format!("{outer}:{inner}"),
        (Some(outer), _) => outer.to_string(),
        _ => fallback.to_string(),
    };
    if redaction_violation(&class).is_some() || class.trim().is_empty() {
        fallback.to_string()
    } else {
        class
    }
}

pub(crate) fn redact_error_message(text: &str) -> String {
    let cleaned = redact_hex_runs(text.trim());
    if redaction_violation(&cleaned).is_some() || cleaned.is_empty() {
        REDACTED_FAILURE_DETAILS.to_string()
    } else {
        cleaned
    }
}

pub(crate) fn redaction_violation(text: &str) -> Option<&'static str> {
    let lowered = text.to_ascii_lowercase();
    if REDACTION_TERMS.iter().any(|term| lowered.contains(term)) {
        return Some("forbidden-term");
    }
    if has_long_hex_run(text, 32) {
        return Some("long-hex-run");
    }
    None
}

fn ident_token(text: &str) -> Option<&str> {
    let end = text
        .char_indices()
        .find_map(|(idx, ch)| {
            (!(ch.is_ascii_alphanumeric() || ch == '_' || ch == ':')).then_some(idx)
        })
        .unwrap_or(text.len());
    let token = text.get(..end)?.trim_matches('"').trim_matches('\'');
    (!token.is_empty()).then_some(token)
}

fn has_long_hex_run(text: &str, min_len: usize) -> bool {
    let mut run_len = 0usize;
    for ch in text.chars() {
        if ch.is_ascii_hexdigit() {
            run_len += 1;
            if run_len >= min_len {
                return true;
            }
        } else {
            run_len = 0;
        }
    }
    false
}

fn redact_hex_runs(text: &str) -> String {
    fn flush(out: &mut String, run: &mut String) {
        if run.len() >= 32 {
            out.push_str("[redacted_hex]");
        } else {
            out.push_str(run);
        }
        run.clear();
    }

    let mut out = String::with_capacity(text.len());
    let mut run = String::new();
    for ch in text.chars() {
        if ch.is_ascii_hexdigit() {
            run.push(ch);
        } else {
            flush(&mut out, &mut run);
            out.push(ch);
        }
    }
    flush(&mut out, &mut run);
    out
}
