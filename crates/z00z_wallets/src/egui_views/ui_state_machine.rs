#[derive(Debug, Clone, Default)]
struct Context {
    root: YamlValue,
}

impl Context {
    fn get(&self, path: &str) -> Option<&YamlValue> {
        let mut current = &self.root;

        for part in path.split('.').filter(|part| !part.is_empty()) {
            let YamlValue::Mapping(map) = current else {
                return None;
            };

            let key = YamlValue::String(part.to_string());
            current = map.get(&key)?;
        }

        Some(current)
    }

    fn set(&mut self, path: &str, value: YamlValue) -> WalletResult<()> {
        let parts: Vec<&str> = path.split('.').filter(|part| !part.is_empty()).collect();
        if parts.is_empty() {
            return Err(WalletError::ContextError(
                "Context path must not be empty".to_string(),
            ));
        }

        let mut current = &mut self.root;

        for (idx, part) in parts.iter().enumerate() {
            let is_last = idx + 1 == parts.len();
            let key = YamlValue::String((*part).to_string());

            let YamlValue::Mapping(map) = current else {
                return Err(WalletError::ContextError(format!(
                    "Context path segment '{}' is not a mapping",
                    part
                )));
            };

            if is_last {
                map.insert(key, value);
                return Ok(());
            }

            if !map.contains_key(&key) {
                map.insert(key.clone(), YamlValue::Mapping(Default::default()));
            }

            current = map
                .get_mut(&key)
                .ok_or_else(|| WalletError::ContextError("Failed to update context".to_string()))?;

            if !matches!(current, YamlValue::Mapping(_)) {
                *current = YamlValue::Mapping(Default::default());
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
struct StateMachine {
    current_state_id: String,
    context: Context,
}

impl StateMachine {
    fn new(initial_state_id: String) -> WalletResult<Self> {
        if initial_state_id.is_empty() {
            return Err(WalletError::InvalidConfig(
                "Initial state id must not be empty".to_string(),
            ));
        }

        Ok(Self {
            current_state_id: initial_state_id,
            context: Context {
                root: YamlValue::Mapping(Default::default()),
            },
        })
    }

    fn current_state_id(&self) -> &str {
        self.current_state_id.as_str()
    }

    fn goto(&mut self, next_state_id: String) -> WalletResult<()> {
        if next_state_id.is_empty() {
            return Err(WalletError::InvalidConfig(
                "Next state id must not be empty".to_string(),
            ));
        }

        self.current_state_id = next_state_id;
        Ok(())
    }

    fn context(&self) -> &Context {
        &self.context
    }

    fn context_mut(&mut self) -> &mut Context {
        &mut self.context
    }
}