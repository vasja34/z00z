#[derive(Debug, Clone, Default, serde::Deserialize)]
struct EntryPoint {
    id: Option<String>,
}

/// Minimal UI configuration stub.
///
/// This keeps the desktop egui shell self-contained and avoids re-introducing
/// older config/state-machine types into `core/types`.
#[derive(Debug, Clone, Default)]
struct WalletConfiguration {
    entry_points: Vec<EntryPoint>,
    ui_themes: Option<YamlValue>,
}

impl WalletConfiguration {
    fn stub() -> Self {
        Self {
            entry_points: vec![EntryPoint {
                id: Some("wallet_list".to_string()),
            }],
            ui_themes: None,
        }
    }

    fn get_entry_points(&self) -> &[EntryPoint] {
        &self.entry_points
    }

    fn ui_themes(&self) -> Option<&YamlValue> {
        self.ui_themes.as_ref()
    }
}

#[derive(Debug, Clone)]
struct MainUiSpec {
    entry_points: Vec<(String, String)>,
    sidebar_buttons: Vec<SidebarButtonSpec>,
    sidebar_wallet_cards: Vec<WalletCardSpec>,
    state_transitions: Vec<(String, Vec<String>)>,
}

impl MainUiSpec {
    fn state_id_for_entry_point(&self, entry_point: &str) -> Option<&str> {
        self.entry_points
            .iter()
            .find(|(key, _)| key.as_str() == entry_point)
            .map(|(_, value)| value.as_str())
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
struct SidebarButtonSpec {
    label: String,
    entry_point: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct WalletCardSpec {
    wallet_id: String,
    title: String,
    #[serde(default)]
    fiat: Option<String>,
    #[serde(default)]
    delta: Option<String>,
    #[serde(default)]
    toggle: Option<String>,
}