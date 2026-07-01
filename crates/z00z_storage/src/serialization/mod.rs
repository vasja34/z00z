//! Storage-owned JMT serialization contracts.
//!
//! This module defines the typed artifact boundary for deterministic JMT
//! serialization and later inspection-oriented restore and rendering helpers.
//! It intentionally avoids exposing raw `jmt` node, batch, or proof types.

mod artifact;
mod build;
mod codec;
mod restore;
mod store;
mod temp_tree;
mod view;

pub use self::{
    artifact::{
        JmtSerArtifact, JmtSerArtifactId, JmtSerEdge, JmtSerMeta, JmtSerNode, JmtSerNodeKind,
        JmtSerRoots, JmtSerTreeId, JmtSerTreeRoot, JmtSerVersion,
    },
    codec::{decode_artifact, derive_artifact_id, encode_artifact},
    restore::{restore_artifact, JmtRestore, JmtTreeState},
    store::{JmtFsStore, JmtSerStore},
    view::{render_jmt_view, JmtView, JmtViewFmt},
};

// Test-fast serialization suites build live settlement artifacts directly; this
// helper is intentional test surface, not a legacy compatibility export.
pub use self::build::build_artifact;
