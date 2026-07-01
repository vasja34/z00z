use std::{collections::BTreeSet, path::PathBuf};

use z00z_utils::io::{create_dir_all, read_file, write_file};

use crate::{
    error::SerializationError,
    serialization::{
        codec::{check_ver, decode_artifact, derive_artifact_id, encode_artifact},
        restore_artifact, JmtSerArtifact, JmtSerArtifactId,
    },
};

pub trait JmtSerStore {
    fn save_artifact(
        &mut self,
        artifact: &JmtSerArtifact,
    ) -> Result<JmtSerArtifactId, SerializationError>;

    fn load_artifact(
        &self,
        artifact_id: &JmtSerArtifactId,
    ) -> Result<JmtSerArtifact, SerializationError>;

    fn validate_artifact(&self, artifact: &JmtSerArtifact) -> Result<(), SerializationError>;

    fn derive_artifact_id(
        &self,
        artifact: &JmtSerArtifact,
    ) -> Result<JmtSerArtifactId, SerializationError>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JmtFsStore {
    root: PathBuf,
}

impl JmtFsStore {
    #[must_use]
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    fn artifact_dir(&self) -> PathBuf {
        self.root.join("jmt_artifact")
    }

    fn artifact_path(&self, artifact_id: &JmtSerArtifactId) -> PathBuf {
        self.artifact_dir()
            .join(format!("{}.bin", id_hex(artifact_id)))
    }
}

impl JmtSerStore for JmtFsStore {
    fn save_artifact(
        &mut self,
        artifact: &JmtSerArtifact,
    ) -> Result<JmtSerArtifactId, SerializationError> {
        self.validate_artifact(artifact)?;
        let artifact_id = self.derive_artifact_id(artifact)?;
        let bytes = encode_artifact(artifact)?;

        create_dir_all(self.artifact_dir())?;
        write_file(self.artifact_path(&artifact_id), &bytes)?;

        let reloaded = self.load_artifact(&artifact_id)?;
        let reloaded_id = self.derive_artifact_id(&reloaded)?;
        if reloaded_id != artifact_id {
            return Err(SerializationError::RebuildMix);
        }

        Ok(artifact_id)
    }

    fn load_artifact(
        &self,
        artifact_id: &JmtSerArtifactId,
    ) -> Result<JmtSerArtifact, SerializationError> {
        let bytes = read_file(self.artifact_path(artifact_id))?;
        let artifact = decode_artifact(&bytes)?;
        self.validate_artifact(&artifact)?;

        if derive_artifact_id(&artifact)? != *artifact_id {
            return Err(SerializationError::RebuildMix);
        }

        Ok(artifact)
    }

    fn validate_artifact(&self, artifact: &JmtSerArtifact) -> Result<(), SerializationError> {
        check_ver(artifact.version)?;

        if artifact.meta.node_count as usize != artifact.nodes.len() {
            return Err(SerializationError::RebuildMix);
        }
        if artifact.meta.edge_count as usize != artifact.edges.len() {
            return Err(SerializationError::RebuildMix);
        }

        let mut seen_paths = BTreeSet::new();
        for path in &artifact.meta.path_order {
            if !seen_paths.insert(*path) {
                return Err(SerializationError::RebuildMix);
            }
        }

        restore_artifact(artifact)?;

        Ok(())
    }

    fn derive_artifact_id(
        &self,
        artifact: &JmtSerArtifact,
    ) -> Result<JmtSerArtifactId, SerializationError> {
        self.validate_artifact(artifact)?;
        derive_artifact_id(artifact)
    }
}

fn id_hex(artifact_id: &JmtSerArtifactId) -> String {
    let mut out = String::with_capacity(64);
    for byte in artifact_id.as_bytes() {
        use std::fmt::Write as _;
        let _ = write!(&mut out, "{byte:02x}");
    }
    out
}
