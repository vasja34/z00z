use super::{
    Arc, AssetDefinition, AssetDefinitionRegistry, AssetError, BTreeMap, Cow, DefinitionRegistry,
    Logger, MetricsSink, RwLock, TimeProvider,
};

impl AssetDefinitionRegistry {
    /// Create a new empty registry.
    pub fn new(
        logger: Arc<dyn Logger>,
        metrics: Arc<dyn MetricsSink>,
        time: Arc<dyn TimeProvider>,
    ) -> Self {
        let timestamp = time.compat_unix_timestamp_millis();
        logger.info(&format!(
            "asset_registry_initialized: initial_version=0, timestamp={}",
            timestamp
        ));

        Self {
            definitions: RwLock::new(BTreeMap::new()),
            version: RwLock::new(0),
            logger,
            metrics,
            time,
        }
    }

    pub fn from_definitions(definitions: &[AssetDefinition]) -> Result<Self, AssetError> {
        let registry = Self::default();
        registry.insert_batch(definitions.to_vec())?;
        Ok(registry)
    }

    pub(super) fn defs_read(
        &self,
    ) -> Result<std::sync::RwLockReadGuard<'_, DefinitionRegistry>, AssetError> {
        self.definitions.read().map_err(|e| {
            AssetError::LockPoisoned(Cow::Owned(format!(
                "Failed to acquire read lock on registry: {}",
                e
            )))
        })
    }

    pub(super) fn defs_write(
        &self,
    ) -> Result<std::sync::RwLockWriteGuard<'_, DefinitionRegistry>, AssetError> {
        self.definitions.write().map_err(|e| {
            AssetError::LockPoisoned(Cow::Owned(format!(
                "Failed to acquire write lock on registry: {}",
                e
            )))
        })
    }

    pub(super) fn version_read(&self) -> Result<std::sync::RwLockReadGuard<'_, u64>, AssetError> {
        self.version.read().map_err(|e| {
            AssetError::LockPoisoned(Cow::Owned(format!(
                "Failed to acquire read lock on version: {}",
                e
            )))
        })
    }

    pub(super) fn version_write(&self) -> Result<std::sync::RwLockWriteGuard<'_, u64>, AssetError> {
        self.version.write().map_err(|e| {
            AssetError::LockPoisoned(Cow::Owned(format!(
                "Failed to acquire write lock on version: {}",
                e
            )))
        })
    }

    pub(super) fn bump_version(&self) -> Result<u64, AssetError> {
        let mut version = self.version_write()?;
        *version += 1;
        Ok(*version)
    }

    pub fn insert(&self, def: AssetDefinition) -> Result<Arc<AssetDefinition>, AssetError> {
        def.validate()?;
        self.insert_prechecked(def)
    }

    pub(crate) fn insert_prechecked(
        &self,
        def: AssetDefinition,
    ) -> Result<Arc<AssetDefinition>, AssetError> {
        {
            let defs = self.defs_read()?;
            if let Some(existing) = defs.get(&def.id) {
                return Ok(Arc::clone(existing));
            }
        }

        let mut defs = self.defs_write()?;
        if let Some(existing) = defs.get(&def.id) {
            return Ok(Arc::clone(existing));
        }

        self.logger.debug(&format!(
            "Inserting new AssetDefinition into registry: id={:?}, name={}",
            def.id, def.name
        ));

        let arc_def = defs
            .entry(def.id)
            .or_insert_with(|| {
                self.metrics.inc_counter("assets_registered", 1);
                Arc::new(def)
            })
            .clone();

        let new_version = self.bump_version()?;
        self.metrics.set_gauge("registry_size", defs.len() as f64);
        self.metrics
            .set_gauge("registry_version", new_version as f64);

        Ok(arc_def)
    }

    pub fn insert_batch(
        &self,
        defs: Vec<AssetDefinition>,
    ) -> Result<Vec<Arc<AssetDefinition>>, AssetError> {
        if defs.is_empty() {
            return Ok(Vec::new());
        }

        self.logger.debug(&format!(
            "Batch inserting AssetDefinitions into registry: count={}",
            defs.len()
        ));

        let mut registry = self.defs_write()?;
        let mut result = Vec::with_capacity(defs.len());
        let initial_size = registry.len();

        for def in defs {
            def.validate()?;
            let arc_def = registry
                .entry(def.id)
                .or_insert_with(|| {
                    self.logger.trace(&format!(
                        "Inserting new definition in batch: id={:?}, name={}",
                        def.id, def.name
                    ));
                    Arc::new(def)
                })
                .clone();
            result.push(arc_def);
        }

        let new_assets = registry.len() - initial_size;
        let new_version = if new_assets > 0 {
            Some(self.bump_version()?)
        } else {
            None
        };

        self.metrics
            .inc_counter("assets_registered", new_assets as u64);
        self.metrics
            .set_gauge("registry_size", registry.len() as f64);
        if let Some(new_version) = new_version {
            self.metrics
                .set_gauge("registry_version", new_version as f64);
        }

        self.logger
            .debug(&format!("Batch insert complete: inserted={}", result.len()));

        Ok(result)
    }

    pub fn get(&self, id: &[u8; 32]) -> Result<Option<Arc<AssetDefinition>>, AssetError> {
        let defs = self.defs_read()?;
        Ok(defs.get(id).map(Arc::clone))
    }

    pub fn get_version(&self) -> Result<u64, AssetError> {
        Ok(*self.version_read()?)
    }

    pub fn len(&self) -> Result<usize, AssetError> {
        let defs = self.defs_read()?;
        Ok(defs.len())
    }

    pub fn is_empty(&self) -> Result<bool, AssetError> {
        Ok(self.len()? == 0)
    }

    pub fn contains(&self, id: &[u8; 32]) -> Result<bool, AssetError> {
        let defs = self.defs_read()?;
        Ok(defs.contains_key(id))
    }

    pub fn sync_global_fallback(&self) -> Result<(), AssetError> {
        let snapshot = self.get_shared_snapshot()?;
        for definition in snapshot.values() {
            super::GLOBAL_ASSET_REGISTRY.insert_prechecked((**definition).clone())?;
        }
        Ok(())
    }
}
