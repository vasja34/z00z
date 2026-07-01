use super::{
    registry_catalog, Arc, AssetDefinitionRegistry, AssetError, Logger, MetricsSink, Path,
    TimeProvider,
};

impl AssetDefinitionRegistry {
    /// Load secondary registry-catalog definitions from a YAML file.
    pub fn load_catalog_from_yaml(
        path: &Path,
        logger: Arc<dyn Logger>,
        metrics: Arc<dyn MetricsSink>,
        time: Arc<dyn TimeProvider>,
    ) -> Result<Self, AssetError> {
        use super::super::snapshot::{RegistrySnapshot, RegistryVersion};

        let start_time = z00z_utils::time::Instant::now();
        logger.info(&format!(
            "Loading asset registry catalog from YAML: {}",
            path.display()
        ));

        let (version_num, wire_definitions) = registry_catalog::load_catalog_from_yaml(path)?;

        let mut ordered_defs = wire_definitions.clone();
        ordered_defs.sort_by_key(|wire| wire.id);
        let hash = RegistryVersion::compute_hash(version_num, &ordered_defs);

        let snapshot = RegistrySnapshot {
            version: RegistryVersion {
                version: version_num,
                hash,
                timestamp: time.compat_unix_timestamp_millis(),
            },
            definitions: wire_definitions,
        };

        let registry = Self::new(logger, metrics.clone(), time);
        registry.update_from_snapshot(snapshot)?;

        let elapsed = start_time.elapsed();
        metrics.observe_histogram("registry_load_ms", elapsed.as_millis() as f64);
        metrics.inc_counter("assets_loaded", registry.len()? as u64);

        Ok(registry)
    }
}
