fn load_main_ui_spec(_wallet_config: &WalletConfiguration) -> WalletResult<MainUiSpec> {
    // STUB: Original function moved to .temp/ - will be replaced with RPC architecture

    let sidebar_buttons = vec![
        SidebarButtonSpec {
            label: "Create Wallet".to_string(),
            entry_point: "wallet_create_step1".to_string(),
        },
        SidebarButtonSpec {
            label: "Add Wallet".to_string(),
            entry_point: "wallet_list".to_string(),
        },
        SidebarButtonSpec {
            label: "Logs".to_string(),
            entry_point: "log_viewer".to_string(),
        },
        SidebarButtonSpec {
            label: "Settings".to_string(),
            entry_point: "config_menu".to_string(),
        },
        SidebarButtonSpec {
            label: "Log Out".to_string(),
            entry_point: "exit".to_string(),
        },
    ];

    Ok(MainUiSpec {
        entry_points: Vec::new(),
        sidebar_buttons,
        sidebar_wallet_cards: vec![],
        state_transitions: Vec::new(),
    })
}

fn scan_state_transitions(flows_dir: &Path) -> Vec<(String, Vec<String>)> {
    let states_dir = flows_dir.join("states");
    let mut transitions: Vec<(String, Vec<String>)> = Vec::new();

    let Ok(entries) = z00z_utils::io::read_dir(&states_dir) else {
        return transitions;
    };

    for path in entries {
        fn collect_goto_targets(value: &YamlValue, targets: &mut Vec<String>) {
            match value {
                YamlValue::Mapping(map) => {
                    let type_key = YamlValue::String("type".to_string());
                    let target_key = YamlValue::String("target".to_string());
                    let goto_key = YamlValue::String("goto".to_string());

                    if let Some(YamlValue::String(target)) = map.get(&goto_key) {
                        if !target.is_empty() {
                            targets.push(target.clone());
                        }
                    }

                    let is_goto =
                        matches!(map.get(&type_key), Some(YamlValue::String(t)) if t == "goto");
                    if is_goto {
                        if let Some(YamlValue::String(target)) = map.get(&target_key) {
                            if !target.is_empty() {
                                targets.push(target.clone());
                            }
                        }
                    }

                    for (key, nested) in map {
                        let _ = key;
                        collect_goto_targets(nested, targets);
                    }
                }
                YamlValue::Sequence(seq) => {
                    for item in seq {
                        collect_goto_targets(item, targets);
                    }
                }
                _ => {}
            }
        }

        if path.extension().and_then(|ext| ext.to_str()) != Some("yaml") {
            continue;
        }

        let Ok(content) = read_to_string(&path) else {
            continue;
        };

        let state_id = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("")
            .to_string();

        let mut next_states = Vec::new();
        let codec = YamlCodec;
        if let Ok(value) = codec.deserialize::<YamlValue>(content.as_bytes()) {
            collect_goto_targets(&value, &mut next_states);
        }

        next_states.sort();
        next_states.dedup();
        transitions.push((state_id.to_string(), next_states));
    }

    transitions
}