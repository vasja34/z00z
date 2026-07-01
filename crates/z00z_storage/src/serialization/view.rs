use std::fmt::Write as _;

use crate::{
    error::SerializationError,
    serialization::{restore_artifact, JmtRestore, JmtSerArtifact, JmtSerNodeKind, JmtSerTreeId},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JmtViewFmt {
    Dot,
    Text,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JmtView {
    fmt: JmtViewFmt,
    body: String,
}

impl JmtView {
    fn new(fmt: JmtViewFmt, body: String) -> Self {
        Self { fmt, body }
    }

    #[must_use]
    pub const fn fmt(&self) -> JmtViewFmt {
        self.fmt
    }

    #[must_use]
    pub fn body(&self) -> &str {
        &self.body
    }
}

pub fn render_jmt_view(
    artifact: &JmtSerArtifact,
    fmt: JmtViewFmt,
) -> Result<JmtView, SerializationError> {
    let restored = restore_artifact(artifact)?;
    let body = match fmt {
        JmtViewFmt::Dot => render_dot(&restored),
        JmtViewFmt::Text => render_text(&restored),
    };

    Ok(JmtView::new(fmt, body))
}

fn render_dot(restored: &JmtRestore) -> String {
    let artifact = restored.artifact();
    let mut out = String::from("digraph jmt_artifact {\n  rankdir=LR;\n");

    for tree in restored.trees() {
        let label = namespace_label(&tree.tree_id);
        let _ = writeln!(
            out,
            "  subgraph \"cluster_{}\" {{",
            cluster_label(&tree.tree_id)
        );
        let _ = writeln!(
            out,
            "    label=\"tree:{} tree_root={} jmt_root={}\";",
            label,
            hex32(&tree.root),
            hex32(&tree.jmt_root),
        );

        for node in artifact
            .nodes
            .iter()
            .filter(|node| node.tree_id == tree.tree_id)
        {
            let _ = writeln!(
                out,
                "    \"{}\" [label=\"{}\\nkind={}\\nhash={}\"];",
                hex32(&node.id),
                if node.key.is_empty() {
                    "root".to_string()
                } else {
                    hex_bytes(&node.key)
                },
                kind_label(node.kind),
                hex32(&node.node_hash),
            );
        }

        for edge in artifact
            .edges
            .iter()
            .filter(|edge| edge.tree_id == tree.tree_id)
        {
            let _ = writeln!(
                out,
                "    \"{}\" -> \"{}\" [label=\"slot={}\"];",
                hex32(&edge.parent),
                hex32(&edge.child),
                edge.slot,
            );
        }

        out.push_str("  }\n");
    }

    out.push_str("}\n");
    out
}

fn render_text(restored: &JmtRestore) -> String {
    let artifact = restored.artifact();
    let mut out = String::new();
    let _ = writeln!(
        out,
        "settlement_root={}",
        hex32(artifact.roots.sem_root.as_bytes())
    );
    let _ = writeln!(out, "settlement_path_order:");
    for path in &artifact.meta.path_order {
        let _ = writeln!(
            out,
            "- def={} serial={} terminal={}",
            hex32(path.definition_id.as_bytes()),
            path.serial_id.get(),
            hex32(path.terminal_id.as_bytes()),
        );
    }

    for tree in restored.trees() {
        let _ = writeln!(
            out,
            "tree {} tree_root={} jmt_root={} nodes={} edges={} bound={}",
            namespace_label(&tree.tree_id),
            hex32(&tree.root),
            hex32(&tree.jmt_root),
            tree.node_ids.len(),
            tree.edge_ids.len(),
            tree.is_root_bound,
        );

        for node in artifact
            .nodes
            .iter()
            .filter(|node| node.tree_id == tree.tree_id)
        {
            let _ = writeln!(
                out,
                "  node {} kind={} key={} key_hash={} value_hash={}",
                hex32(&node.id),
                kind_label(node.kind),
                if node.key.is_empty() {
                    String::from("<root>")
                } else {
                    hex_bytes(&node.key)
                },
                node.key_hash
                    .map(|bytes| hex32(&bytes))
                    .unwrap_or_else(|| String::from("-")),
                node.value_hash
                    .map(|bytes| hex32(&bytes))
                    .unwrap_or_else(|| String::from("-")),
            );
        }

        for edge in artifact
            .edges
            .iter()
            .filter(|edge| edge.tree_id == tree.tree_id)
        {
            let _ = writeln!(
                out,
                "  edge {} -> {} slot={}",
                hex32(&edge.parent),
                hex32(&edge.child),
                edge.slot,
            );
        }
    }

    out
}

fn namespace_label(tree_id: &JmtSerTreeId) -> String {
    match tree_id {
        JmtSerTreeId::Definition => String::from("definition"),
        JmtSerTreeId::Serial(definition_id) => {
            format!("serial:{}", hex32(definition_id.as_bytes()))
        }
        JmtSerTreeId::Bucket {
            definition_id,
            serial_id,
        } => format!(
            "bucket:{}:{}",
            hex32(definition_id.as_bytes()),
            serial_id.get()
        ),
        JmtSerTreeId::Terminal {
            definition_id,
            serial_id,
            bucket_id,
        } => format!(
            "terminal:{}:{}:{}",
            hex32(definition_id.as_bytes()),
            serial_id.get(),
            hex32(bucket_id.as_bytes())
        ),
        JmtSerTreeId::PathIndex => String::from("settlement_path_index"),
    }
}

fn cluster_label(tree_id: &JmtSerTreeId) -> String {
    namespace_label(tree_id).replace(':', "_")
}

fn kind_label(kind: JmtSerNodeKind) -> &'static str {
    match kind {
        JmtSerNodeKind::Internal => "internal",
        JmtSerNodeKind::Leaf => "leaf",
        JmtSerNodeKind::Null => "null",
    }
}

fn hex32(bytes: &[u8; 32]) -> String {
    hex_bytes(bytes)
}

fn hex_bytes(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let _ = write!(out, "{byte:02x}");
    }
    out
}
