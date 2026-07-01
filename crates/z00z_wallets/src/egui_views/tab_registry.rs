/// Tab configuration with associated Rust module file
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TabDefinition {
    /// Unique tab identifier (e.g., "wallet_assets", "settings_main")
    pub id: String,
    /// Display name shown in tab bar (e.g., "Assets", "Settings")
    pub label: String,
    /// Emoji icon for visual identification
    pub emoji: &'static str,
    /// Associated Rust module file (e.g., "wallet_tab_assets")
    pub module_name: &'static str,
    /// Section/group this tab belongs to (e.g., "wallet", "settings", "network")
    pub section: &'static str,
    /// Order within section (0 = first)
    pub order: u32,
    /// Entry points that trigger opening this tab
    pub entry_points: &'static [&'static str],
}

/// Dictionary mapping tabs to their Rust module implementations
///
/// This structure defines all available tabs in the wallet UI, their properties,
/// and the associated Rust files that implement their rendering logic.
pub struct TabRegistry {
    tabs: Vec<TabDefinition>,
}

impl TabRegistry {
    /// Create a new tab registry with all defined tabs
    pub fn new() -> Self {
        let mut tabs: Vec<TabDefinition> = Vec::new();

        let wallet_tabs = vec![
            TabDefinition {
                id: "wallet_assets".to_string(),
                label: "Assets".to_string(),
                emoji: "💰",
                module_name: "wallet_tab_assets",
                section: "wallet",
                order: 0,
                entry_points: &["list_assets"],
            },
            TabDefinition {
                id: "wallet_send".to_string(),
                label: "Send".to_string(),
                emoji: "📤",
                module_name: "wallet_tab_send",
                section: "wallet",
                order: 1,
                entry_points: &["send_asset"],
            },
            TabDefinition {
                id: "wallet_receive".to_string(),
                label: "Receive".to_string(),
                emoji: "📥",
                module_name: "wallet_tab_receive",
                section: "wallet",
                order: 2,
                entry_points: &["receive"],
            },
            TabDefinition {
                id: "wallet_import".to_string(),
                label: "Import Assets".to_string(),
                emoji: "🗄",
                module_name: "wallet_tab_import",
                section: "wallet",
                order: 3,
                entry_points: &["import_asset"],
            },
            TabDefinition {
                id: "wallet_swap".to_string(),
                label: "Swap".to_string(),
                emoji: "🔁",
                module_name: "wallet_tab_swap",
                section: "wallet",
                order: 4,
                entry_points: &["defi_swap"],
            },
            TabDefinition {
                id: "wallet_staking".to_string(),
                label: "Staking".to_string(),
                emoji: "🔑",
                module_name: "wallet_tab_staking",
                section: "wallet",
                order: 5,
                entry_points: &["defi_staking"],
            },
            TabDefinition {
                id: "wallet_history".to_string(),
                label: "History".to_string(),
                emoji: "📜",
                module_name: "wallet_tab_history",
                section: "wallet",
                order: 6,
                entry_points: &["tx_history"],
            },
            TabDefinition {
                id: "wallet_backup".to_string(),
                label: "Backup".to_string(),
                emoji: "💾",
                module_name: "wallet_tab_backup",
                section: "wallet",
                order: 7,
                entry_points: &["backup_manager"],
            },
        ];

        let create_wallet_tabs = vec![TabDefinition {
            id: "create_wallet".to_string(),
            label: "Create Wallet".to_string(),
            emoji: "👛",
            module_name: "app_create_wallet_tab",
            section: "wallet_management",
            order: 0,
            entry_points: &["wallet_create_step1"],
        }];

        let add_wallet_tabs = vec![TabDefinition {
            id: "add_wallet".to_string(),
            label: "Add Wallet".to_string(),
            emoji: "➕",
            module_name: "add_wallet_tab",
            section: "wallet_management",
            order: 0,
            entry_points: &["wallet_list"],
        }];

        let network_devnet_tabs = vec![TabDefinition {
            id: "network_devnet".to_string(),
            label: "Devnet".to_string(),
            emoji: "⚡",
            module_name: "network_devnet_tab",
            section: "network",
            order: 0,
            entry_points: &["network_devnet"],
        }];

        let network_testnet_tabs = vec![TabDefinition {
            id: "network_testnet".to_string(),
            label: "Testnet".to_string(),
            emoji: "❓",
            module_name: "network_testnet_tab",
            section: "network",
            order: 0,
            entry_points: &["network_testnet"],
        }];

        let network_mainnet_tabs = vec![TabDefinition {
            id: "network_mainnet".to_string(),
            label: "Mainnet".to_string(),
            emoji: "✅",
            module_name: "network_mainnet_tab",
            section: "network",
            order: 0,
            entry_points: &["network_mainnet"],
        }];

        let network_tor_tabs = vec![TabDefinition {
            id: "network_tor".to_string(),
            label: "Tor".to_string(),
            emoji: "🌀",
            module_name: "network_tor_tab",
            section: "network",
            order: 0,
            entry_points: &["network_tor"],
        }];

        let network_onionet_tabs = vec![TabDefinition {
            id: "network_onionet".to_string(),
            label: "Onionet".to_string(),
            emoji: "🌀",
            module_name: "network_onionnet_tab",
            section: "network",
            order: 0,
            entry_points: &["network_onionnet"],
        }];

        let network_scanner_tabs = vec![TabDefinition {
            id: "network_scanner".to_string(),
            label: "Scanner".to_string(),
            emoji: "🔎",
            module_name: "network_scanner_tab",
            section: "network",
            order: 0,
            entry_points: &["network_scanner"],
        }];

        let network_nodes_tabs = vec![TabDefinition {
            id: "network_nodes".to_string(),
            label: "Nodes".to_string(),
            emoji: "🖧",
            module_name: "network_nodes_tab",
            section: "network",
            order: 0,
            entry_points: &["network_nodes"],
        }];

        let app_settings_tabs = vec![TabDefinition {
            id: "app_settings".to_string(),
            label: "Settings".to_string(),
            emoji: "⚙",
            module_name: "app_settings_tab",
            section: "settings",
            order: 0,
            entry_points: &["config_menu"],
        }];

        let app_logs_tabs = vec![TabDefinition {
            id: "app_logs".to_string(),
            label: "Logs".to_string(),
            emoji: "📜",
            module_name: "app_logs_tab",
            section: "navigation",
            order: 0,
            entry_points: &["log_viewer"],
        }];

        let all_tab_groups = vec![
            wallet_tabs,
            create_wallet_tabs,
            add_wallet_tabs,
            network_devnet_tabs,
            network_testnet_tabs,
            network_mainnet_tabs,
            network_tor_tabs,
            network_onionet_tabs,
            network_scanner_tabs,
            network_nodes_tabs,
            app_settings_tabs,
            app_logs_tabs,
        ];

        for tab_group in all_tab_groups {
            for tab_def in tab_group {
                tabs.push(tab_def);
            }
        }

        Self { tabs }
    }

    /// Get tab definition by ID
    pub fn get(&self, tab_id: &str) -> Option<&TabDefinition> {
        self.tabs.iter().find(|tab| tab.id.as_str() == tab_id)
    }

    /// Find tab by entry_point (automatically resolves entry point to tab)
    pub fn find_by_entry_point(&self, entry_point: &str) -> Option<&TabDefinition> {
        self.tabs
            .iter()
            .find(|tab| tab.entry_points.contains(&entry_point))
    }

    /// Get all tabs for a specific section
    pub fn get_section(&self, section: &str) -> Vec<&TabDefinition> {
        let mut section_tabs: Vec<_> = self
            .tabs
            .iter()
            .filter(|tab| tab.section == section)
            .collect();
        section_tabs.sort_by_key(|tab| tab.order);
        section_tabs
    }

    /// Get all tabs sorted by section and order
    pub fn get_all_sorted(&self) -> Vec<&TabDefinition> {
        let mut sorted: Vec<_> = self.tabs.iter().collect();
        sorted.sort_by(|a, b| match a.section.cmp(b.section) {
            std::cmp::Ordering::Equal => a.order.cmp(&b.order),
            other => other,
        });
        sorted
    }
}

impl Default for TabRegistry {
    fn default() -> Self {
        Self::new()
    }
}
