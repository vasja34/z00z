use super::{
    Arc, AssetDefinition, AssetDefinitionRegistry, AssetError, BTreeMap, Cow, DefinitionRegistry,
    SharedRegistry,
};

impl AssetDefinitionRegistry {
    /// Create immutable snapshot from current registry.
    pub fn create_snapshot(&self) -> Result<super::super::snapshot::RegistrySnapshot, AssetError> {
        use super::super::snapshot::{RegistrySnapshot, RegistryVersion};
        use super::super::wire::DefinitionWire;

        let defs = self.defs_read()?;
        let current_version = *self.version_read()?;

        let definitions: Vec<DefinitionWire> = defs
            .values()
            .map(|arc_def| {
                arc_def.validate()?;
                Ok(DefinitionWire::from(arc_def.as_ref()))
            })
            .collect::<Result<_, AssetError>>()?;

        self.logger.debug(&format!(
            "Creating registry snapshot: definitions_count={}, version={}",
            definitions.len(),
            current_version
        ));

        self.metrics.inc_counter("registry_snapshots_created", 1);
        self.metrics
            .set_gauge("registry_version", current_version as f64);

        let hash = RegistryVersion::compute_hash(current_version, &definitions);

        Ok(RegistrySnapshot {
            version: RegistryVersion {
                version: current_version,
                hash,
                timestamp: self.time.compat_unix_timestamp_millis(),
            },
            definitions,
        })
    }

    /// Update registry from snapshot.
    pub fn update_from_snapshot(
        &self,
        snapshot: super::super::snapshot::RegistrySnapshot,
    ) -> Result<(), AssetError> {
        let super::super::snapshot::RegistrySnapshot {
            version,
            definitions,
        } = snapshot;

        self.logger.info(&format!(
            "Updating registry from snapshot: version={}, hash={:?}, definitions_count={}",
            version.version,
            version.hash,
            definitions.len()
        ));

        let current_version = *self.version_read()?;
        let mut ordered_defs = definitions;
        ordered_defs.sort_by_key(|wire| wire.id);

        if version.version == 0 && (current_version != 0 || !ordered_defs.is_empty()) {
            return Err(AssetError::InvalidAsset(Cow::Borrowed(
                "Invalid snapshot version 0",
            )));
        }
        if version.version != 0 && version.version <= current_version {
            return Err(AssetError::InvalidAsset(Cow::Owned(format!(
                "Downgrade attempt: version {} <= current {}",
                version.version, current_version
            ))));
        }

        for window in ordered_defs.windows(2) {
            if window[0].id == window[1].id {
                return Err(AssetError::InvalidAsset(Cow::Owned(format!(
                    "Duplicate snapshot definition id: {:02x?}",
                    window[0].id
                ))));
            }
        }

        let computed_hash =
            super::super::snapshot::RegistryVersion::compute_hash(version.version, &ordered_defs);
        if computed_hash != version.hash {
            return Err(AssetError::InvalidAsset(Cow::Borrowed(
                "Snapshot hash mismatch - corrupted data",
            )));
        }

        let mut new_registry: DefinitionRegistry = BTreeMap::new();
        for wire in ordered_defs.iter().cloned() {
            let def: AssetDefinition = wire.try_into()?;
            new_registry.insert(def.id, Arc::new(def));
        }

        let mut defs_guard = self.defs_write()?;
        let mut version_guard = self.version_write()?;
        let current_version = *version_guard;
        if version.version == 0 && (current_version != 0 || !ordered_defs.is_empty()) {
            return Err(AssetError::InvalidAsset(Cow::Borrowed(
                "Invalid snapshot version 0",
            )));
        }
        if version.version != 0 && version.version <= current_version {
            return Err(AssetError::InvalidAsset(Cow::Owned(format!(
                "Downgrade attempt: version {} <= current {}",
                version.version, current_version
            ))));
        }

        *defs_guard = new_registry;
        *version_guard = version.version;

        self.logger.info(&format!(
            "registry_updated: new_version={}, previous_version={}, definitions={}",
            version.version,
            current_version,
            ordered_defs.len()
        ));

        self.metrics
            .set_gauge("registry_version", version.version as f64);
        self.metrics
            .set_gauge("registry_size", ordered_defs.len() as f64);

        Ok(())
    }

    /// Return an immutable shared snapshot of the current registry.
    pub fn get_shared_snapshot(&self) -> Result<SharedRegistry, AssetError> {
        let defs = self.defs_read()?;
        Ok(Arc::new(defs.clone()))
    }

    #[cfg(test)]
    pub fn clear_for_testing(&self) -> Result<(), AssetError> {
        let mut defs = self.defs_write()?;
        let mut version = self.version_write()?;

        defs.clear();
        *version = 0;

        self.logger.debug("Registry cleared for testing");

        Ok(())
    }
}
