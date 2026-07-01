/// Main desktop application window.
///
/// The view is driven by `flows/main.yaml` (entry points + UI layout hints),
/// but uses the existing `StateMachine` rendering API for the content area.
pub struct MainView {
    wallet_config: Arc<WalletConfiguration>,
    state_machine: StateMachine,
    spec: MainUiSpec,
    pub tab_registry: TabRegistry,
    /// Currently open tabs (section_name -> list of tab definitions)
    current_open_tabs: Vec<TabDefinition>,
    /// Active tab index in current_open_tabs
    active_tab_index: usize,
    /// Current section being displayed
    current_section: Option<String>,
    /// Currently selected navigation entry point (for sidebar highlighting)
    selected_entry_point: Option<String>,
    dragging_wallet_id: Option<String>,
    drag_target_index: Option<usize>,
    applied_theme_key: Option<String>,
    theme_colors: Option<EguiThemeColors>,
    last_error: Option<String>,

    #[cfg(not(target_arch = "wasm32"))]
    lifecycle_hook: Option<Arc<dyn Fn(WalletLifecycleEvent) + Send + Sync>>,
    #[cfg(not(target_arch = "wasm32"))]
    last_focused: Option<bool>,
    #[cfg(not(target_arch = "wasm32"))]
    last_minimized: Option<bool>,
}

impl MainView {
    /// Build a `MainView` from a flows directory (e.g. `crates/z00z_wallets/src/flows`).
    pub fn from_flows_dir(path: impl AsRef<Path>) -> WalletResult<Self> {
        let _ = path;
        let config = Arc::new(WalletConfiguration::stub());
        Self::wallet_config_flows_dir(config, std::path::Path::new("."))
    }

    /// Build a `MainView` from an already parsed `WalletConfiguration` and a flows directory.
    ///
    /// This keeps parsing single-source-of-truth (done by the caller) while allowing the egui
    /// shell to derive state-to-state transition buttons from the YAML state files.
    fn wallet_config_flows_dir(
        config: Arc<WalletConfiguration>,
        flows_dir: impl AsRef<Path>,
    ) -> WalletResult<Self> {
        let mut view = Self::from_wallet_config(config)?;
        view.spec.state_transitions = scan_state_transitions(flows_dir.as_ref());

        let has_wallet = view
            .state_machine
            .context()
            .get("ctx.current_wallet")
            .is_some_and(|value| value.as_str().is_some_and(|id| !id.is_empty()));

        if !has_wallet {
            if let Some(first) = view.spec.sidebar_wallet_cards.first() {
                let _ = view.state_machine.context_mut().set(
                    "ctx.current_wallet",
                    YamlValue::String(first.wallet_id.clone()),
                );
                let _ = view.state_machine.context_mut().set(
                    "ctx.current_wallet_name",
                    YamlValue::String(first.title.clone()),
                );
            }
        }

        Ok(view)
    }

    /// Build a `MainView` from an already parsed `WalletConfiguration`.
    ///
    /// This is the preferred entry point for desktop UIs: parse once at startup,
    /// then render using the immutable `wallet_config` as the only source of truth.
    fn from_wallet_config(config: Arc<WalletConfiguration>) -> WalletResult<Self> {
        let initial_state = config
            .get_entry_points()
            .first()
            .and_then(|state| state.id.clone())
            .ok_or_else(|| {
                WalletError::InvalidConfig("No entry points defined in configuration".to_string())
            })?;

        let state_machine = StateMachine::new(initial_state)?;
        let spec = load_main_ui_spec(&config)?;
        let tab_registry = TabRegistry::new();

        Ok(Self {
            wallet_config: config.clone(),
            state_machine,
            spec,
            tab_registry,
            current_open_tabs: Vec::new(),
            active_tab_index: 0,
            current_section: None,
            selected_entry_point: None,
            dragging_wallet_id: None,
            drag_target_index: None,
            applied_theme_key: None,
            theme_colors: None,
            last_error: None,

            #[cfg(not(target_arch = "wasm32"))]
            lifecycle_hook: None,
            #[cfg(not(target_arch = "wasm32"))]
            last_focused: None,
            #[cfg(not(target_arch = "wasm32"))]
            last_minimized: None,
        })
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn set_lifecycle_hook(&mut self, hook: Arc<dyn Fn(WalletLifecycleEvent) + Send + Sync>) {
        self.lifecycle_hook = Some(hook);
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn maybe_emit_lifecycle(&mut self, ctx: &egui::Context) {
        let Some(hook) = self.lifecycle_hook.as_ref() else {
            return;
        };

        let (focused, minimized) = ctx.input(|input| {
            let focused = input.viewport().focused.unwrap_or(true);
            let minimized = input.viewport().minimized.unwrap_or(false);
            (focused, minimized)
        });

        let prev_focused = self.last_focused.replace(focused);
        let prev_minimized = self.last_minimized.replace(minimized);

        let focus_lost = prev_focused.is_some_and(|prev| prev) && !focused;
        let focus_gained = prev_focused.is_some_and(|prev| !prev) && focused;
        let became_minimized = prev_minimized.is_some_and(|prev| !prev) && minimized;

        if focus_lost || became_minimized {
            hook(WalletLifecycleEvent::Backgrounded);
            return;
        }

        if focus_gained && !minimized {
            hook(WalletLifecycleEvent::Foregrounded);
        }
    }

    fn current_theme_key(&self) -> String {
        self.state_machine
            .context()
            .get("ctx.theme")
            .and_then(|value| value.as_str().map(str::to_string))
            .unwrap_or_else(|| "z00z_deep_blue".to_string())
    }

    fn apply_theme_if_needed(&mut self, ctx: &egui::Context) {
        let theme_key = self.current_theme_key();
        let needs_apply = self
            .applied_theme_key
            .as_deref()
            .is_none_or(|prev| prev != theme_key);

        if !needs_apply {
            return;
        }

        let resolved = resolve_egui_theme_colors(self.wallet_config.ui_themes(), &theme_key);
        if let Some(colors) = resolved {
            let mut visuals = egui::Visuals::dark();

            visuals.window_fill = colors.bg_0;
            visuals.panel_fill = colors.bg_1;
            visuals.extreme_bg_color = colors.bg_2;
            visuals.faint_bg_color = colors.panel_0;
            visuals.hyperlink_color = colors.accent_cyan;
            visuals.selection.bg_fill = colors.accent_cyan;
            visuals.selection.stroke.color = colors.text_0;
            visuals.window_stroke.color = colors.border_0;
            visuals.widgets.noninteractive.bg_fill = colors.panel_0;
            visuals.widgets.noninteractive.bg_stroke.color = colors.border_0;
            visuals.widgets.noninteractive.fg_stroke.color = colors.text_2;
            visuals.widgets.inactive.bg_fill = colors.panel_0;
            visuals.widgets.inactive.bg_stroke.color = colors.border_0;
            visuals.widgets.inactive.fg_stroke.color = colors.text_0;
            visuals.widgets.hovered.bg_fill = colors.panel_2;
            visuals.widgets.hovered.bg_stroke.color = colors.accent_cyan;
            visuals.widgets.hovered.fg_stroke.color = colors.text_0;
            visuals.widgets.active.bg_fill = colors.panel_2;
            visuals.widgets.active.bg_stroke.color = colors.accent_cyan;
            visuals.widgets.active.fg_stroke.color = colors.text_0;

            ctx.set_visuals(visuals);
            self.theme_colors = Some(colors);
        } else {
            self.theme_colors = None;
        }

        self.applied_theme_key = Some(theme_key);
    }

    fn goto_entry_point(&mut self, entry_point: &str) {
        let Some(state_id) = self
            .spec
            .state_id_for_entry_point(entry_point)
            .map(str::to_string)
        else {
            self.last_error = Some(format!("Unknown entry point: {entry_point}"));
            return;
        };

        if let Err(err) = self.state_machine.goto(state_id) {
            self.last_error = Some(err.to_string());
        }
    }

    fn current_wallet_label(&self) -> String {
        let wallet_name = self
            .state_machine
            .context()
            .get("ctx.current_wallet_name")
            .and_then(|value| value.as_str().map(str::to_string))
            .unwrap_or_else(|| "<no wallet>".to_string());

        let wallet_id = self
            .state_machine
            .context()
            .get("ctx.current_wallet")
            .and_then(|value| value.as_str().map(str::to_string))
            .unwrap_or_default();

        if wallet_id.is_empty() {
            wallet_name
        } else {
            format!("{wallet_name} ({wallet_id})")
        }
    }

    fn set_current_wallet(&mut self, wallet_id: &str, wallet_name: &str) {
        if let Err(err) = self.state_machine.context_mut().set(
            "ctx.current_wallet",
            YamlValue::String(wallet_id.to_string()),
        ) {
            self.last_error = Some(err.to_string());
            return;
        }

        if let Err(err) = self.state_machine.context_mut().set(
            "ctx.current_wallet_name",
            YamlValue::String(wallet_name.to_string()),
        ) {
            self.last_error = Some(err.to_string());
        }
    }

    fn active_tab(&self) -> Option<&TabDefinition> {
        self.current_open_tabs.get(self.active_tab_index)
    }

    fn set_active_tab(&mut self, index: usize) {
        if index < self.current_open_tabs.len() {
            self.active_tab_index = index;
        }
    }

    fn open_tabs_for_entry_point(&mut self, entry_point: &str) {
        if let Some(tab_def) = self.tab_registry.find_by_entry_point(entry_point) {
            self.current_open_tabs = vec![tab_def.clone()];
            self.active_tab_index = 0;
            self.current_section = Some(tab_def.section.to_string());
        }
    }
}

include!("main_view_render.rs");