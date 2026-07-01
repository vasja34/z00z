impl MainView {
    fn render_header(&self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading("Z00Z");
            ui.separator();
            ui.label(self.current_wallet_label());

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let _ = ui.button("Menu");
                let _ = ui.button("Help");
                let _ = ui.button("Copy");
            });
        });
    }

    fn render_tabs(&mut self, ui: &mut egui::Ui) {
        if self.current_open_tabs.is_empty() {
            ui.label("Select a section to view tabs");
            return;
        }

        let mut clicked_tab_index: Option<usize> = None;

        ui.horizontal(|ui| {
            for (idx, tab) in self.current_open_tabs.iter().enumerate() {
                let is_active = self.active_tab_index == idx;
                let label = format!("{} {}", tab.emoji, tab.label);
                let tab_text = egui::RichText::new(label).text_style(egui::TextStyle::Heading);

                if ui.selectable_label(is_active, tab_text).clicked() {
                    clicked_tab_index = Some(idx);
                }
            }
        });

        if let Some(idx) = clicked_tab_index {
            self.set_active_tab(idx);
        }
    }

    fn render_status_bar(&mut self, ui: &mut egui::Ui) {
        let available = self
            .state_machine
            .context()
            .get("ctx.available_total")
            .and_then(YamlValue::as_u64)
            .unwrap_or(0);
        let locked = self
            .state_machine
            .context()
            .get("ctx.locked_total")
            .and_then(YamlValue::as_u64)
            .unwrap_or(0);
        let pending_in = self
            .state_machine
            .context()
            .get("ctx.pending_in_total")
            .and_then(YamlValue::as_u64)
            .unwrap_or(0);
        let pending_out = self
            .state_machine
            .context()
            .get("ctx.pending_out_total")
            .and_then(YamlValue::as_u64)
            .unwrap_or(0);
        let sync_pct = self
            .state_machine
            .context()
            .get("ctx.sync_progress_pct")
            .and_then(YamlValue::as_f64)
            .unwrap_or(0.0)
            .clamp(0.0, 100.0);

        ui.horizontal(|ui| {
            ui.label(format!("Available: {available}"));
            ui.separator();
            ui.label(format!("Locked: {locked}"));
            ui.separator();
            ui.label(format!("Pending In: {pending_in}"));
            ui.separator();
            ui.label(format!("Pending Out: {pending_out}"));
            ui.separator();
            ui.label(format!("Scanning blockchain: {sync_pct:.1}%"));

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(format!("State: {}", self.state_machine.current_state_id()));
            });
        });

        if let Some(err) = self.last_error.as_deref() {
            ui.separator();
            ui.label(format!("Error: {err}"));
        }
    }

    fn render_content(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.render_tabs(ui);
            ui.add_space(5.0);
            ui.separator();

            if let Some(active_tab) = self.active_tab() {
                ui.vertical_centered(|ui| {
                    ui.heading(format!("{} {}", active_tab.emoji, active_tab.label));
                    ui.label(format!("Module: {}", active_tab.module_name));
                    ui.label(format!("Section: {}", active_tab.section));
                });
            } else {
                ui.vertical_centered(|ui| {
                    ui.label("Select a section from the sidebar to view tabs");
                });
            }

            /* TODO: Replace with RPC-based rendering
            if let Some(next) = state_views::render_state(ui, &state_id, &self.spec.state_transitions)
            {
                if let Err(err) = self.state_machine.goto(next) {
                    self.last_error = Some(err.to_string());
                }
            }
            */
        });
    }
}

include!("main_view_render_sidebar.rs");

impl eframe::App for MainView {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        #[cfg(not(target_arch = "wasm32"))]
        self.maybe_emit_lifecycle(ctx);

        self.apply_theme_if_needed(ctx);

        egui::TopBottomPanel::top("z00z_wallet_header")
            .exact_height(44.0)
            .show(ctx, |ui| self.render_header(ui));

        egui::SidePanel::left("z00z_wallet_sidebar")
            .exact_width(330.0)
            .show(ctx, |ui| self.render_sidebar(ui));

        egui::TopBottomPanel::bottom("z00z_wallet_status")
            .exact_height(30.0)
            .show(ctx, |ui| self.render_status_bar(ui));

        self.render_content(ctx);
    }
}