impl MainView {
    fn render_sidebar(&mut self, ui: &mut egui::Ui) {
        let mut clicked_entry_point: Option<String> = None;
        let mut clicked_wallet: Option<(String, String)> = None;
        let mut clicked_network: Option<&'static str> = None;

        let is_wallet_action_button = |item: &SidebarButtonSpec| {
            matches!(item.label.as_str(), "Create Wallet" | "Add Wallet")
                || matches!(item.entry_point.as_str(), "create_wallet" | "list_wallets")
        };

        let is_removed_nav_button = |item: &SidebarButtonSpec| {
            matches!(item.label.as_str(), "Node Info" | "Network")
                || matches!(item.entry_point.as_str(), "node_info" | "network_selector")
        };

        let emoji_for_sidebar_button = |item: &SidebarButtonSpec| -> &'static str {
            match item.entry_point.as_str() {
                "wallet_create_step1" => "👛",
                "wallet_list" => "➕",
                "log_viewer" => "📜",
                "config_menu" => "⚙",
                "exit" => "🚫",
                "onionnet" | "tor" => "🌀",
                _ => "📌",
            }
        };

        ui.vertical(|ui| {
            ui.heading("Wallets");

            egui::Frame::group(ui.style()).show(ui, |ui| {
                let row_height = ui.text_style_height(&egui::TextStyle::Body);
                let wallet_card_height =
                    (row_height * 3.8 + ui.spacing().item_spacing.y * 7.0).max(72.0);
                let list_max_height = wallet_card_height * 3.0;

                egui::ScrollArea::vertical()
                    .id_source("z00z_wallets_scroll")
                    .scroll_bar_visibility(
                        egui::containers::scroll_area::ScrollBarVisibility::AlwaysVisible,
                    )
                    .max_height(list_max_height)
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        let pointer_pos = ui.ctx().pointer_interact_pos();
                        let mut insertion_y: Option<f32> = None;
                        let mut last_row_bottom: Option<f32> = None;
                        self.drag_target_index = None;

                        let drag_handle_width = 24.0;
                        let drag_handle_right_padding = 10.0;

                        for (idx, wallet) in self.spec.sidebar_wallet_cards.iter().enumerate() {
                            let row = egui::Frame::none().show(ui, |ui| {
                                ui.push_id(wallet.wallet_id.as_str(), |ui| {
                                    let is_selected = self
                                        .state_machine
                                        .context()
                                        .get("ctx.current_wallet")
                                        .is_some_and(|value| {
                                            value.as_str().is_some_and(|id| id == wallet.wallet_id)
                                        });

                                    let response = ui
                                        .selectable_label(is_selected, &wallet.title)
                                        .on_hover_cursor(egui::CursorIcon::PointingHand);
                                    if response.clicked() {
                                        clicked_wallet =
                                            Some((wallet.wallet_id.clone(), wallet.title.clone()));
                                    }

                                    if wallet.fiat.is_some() || wallet.delta.is_some() {
                                        ui.horizontal(|ui| {
                                            if let Some(fiat) = &wallet.fiat {
                                                ui.label(fiat);
                                            }
                                            if let Some(delta) = &wallet.delta {
                                                ui.label(delta);
                                            }
                                        });
                                    }

                                    if let Some(toggle_label) = &wallet.toggle {
                                        let mut dummy = false;
                                        ui.add_enabled(
                                            false,
                                            egui::Checkbox::new(&mut dummy, toggle_label),
                                        );
                                    }

                                    ui.separator();
                                });
                            });

                            let row_rect = row.response.rect;
                            last_row_bottom = Some(row_rect.bottom());

                            let handle_rect = egui::Rect::from_min_size(
                                egui::pos2(
                                    row_rect.right() - drag_handle_right_padding - drag_handle_width,
                                    row_rect.top(),
                                ),
                                egui::vec2(drag_handle_width, row_height),
                            );
                            let drag_id =
                                egui::Id::new(("z00z_wallet_reorder_handle", wallet.wallet_id.as_str()));
                            let handle_resp = ui
                                .interact(handle_rect, drag_id, egui::Sense::drag())
                                .on_hover_cursor(egui::CursorIcon::Grab)
                                .on_hover_text("Drag to reorder");

                            let handle_color = self
                                .theme_colors
                                .map(|colors| colors.text_0)
                                .unwrap_or_else(|| ui.visuals().text_color());

                            let painter = ui.painter();
                            let stroke = egui::Stroke::new(2.0, handle_color);
                            let x0 = handle_rect.left() + 6.0;
                            let x1 = handle_rect.right() - 6.0;
                            let y_mid = handle_rect.center().y;
                            let dy = 4.0;
                            painter.line_segment(
                                [egui::pos2(x0, y_mid - dy), egui::pos2(x1, y_mid - dy)],
                                stroke,
                            );
                            painter.line_segment([egui::pos2(x0, y_mid), egui::pos2(x1, y_mid)], stroke);
                            painter.line_segment(
                                [egui::pos2(x0, y_mid + dy), egui::pos2(x1, y_mid + dy)],
                                stroke,
                            );

                            if handle_resp.drag_started() {
                                self.dragging_wallet_id = Some(wallet.wallet_id.clone());
                            }

                            if self.dragging_wallet_id.is_some() {
                                if let Some(pos) = pointer_pos {
                                    if row_rect.contains(pos) {
                                        let insert_before = pos.y <= row_rect.center().y;
                                        let target = if insert_before { idx } else { idx + 1 };
                                        self.drag_target_index = Some(target);
                                        insertion_y = Some(if insert_before {
                                            row_rect.top()
                                        } else {
                                            row_rect.bottom()
                                        });
                                    }
                                }
                            }
                        }

                        if self.dragging_wallet_id.is_some() {
                            if let (Some(pos), Some(bottom)) = (pointer_pos, last_row_bottom) {
                                if pos.y > bottom {
                                    self.drag_target_index = Some(self.spec.sidebar_wallet_cards.len());
                                    insertion_y = Some(bottom);
                                }
                            }
                        }

                        if let Some(y) = insertion_y {
                            let stroke_color = self
                                .theme_colors
                                .map(|colors| colors.accent_cyan)
                                .unwrap_or_else(|| ui.visuals().selection.bg_fill);
                            let stroke = egui::Stroke::new(2.0, stroke_color);
                            let x0 = ui.min_rect().left();
                            let x1 = ui.min_rect().right();
                            ui.painter()
                                .line_segment([egui::pos2(x0, y), egui::pos2(x1, y)], stroke);
                        }
                    });
            });

            ui.add_space(8.0);

            let mut button_font = ui
                .style()
                .text_styles
                .get(&egui::TextStyle::Heading)
                .cloned()
                .unwrap_or_else(|| egui::FontId::proportional(18.0));
            button_font.size *= 0.8;
            let wallet_action_width = (ui.available_width() * 0.75).max(120.0);
            let wallet_action_height = (button_font.size * 1.9).max(32.0);
            ui.scope(|ui| {
                let mut style = (*ui.style()).as_ref().clone();
                let rounding = egui::Rounding::same(12.0);
                style.visuals.widgets.inactive.rounding = rounding;
                style.visuals.widgets.hovered.rounding = rounding;
                style.visuals.widgets.active.rounding = rounding;
                style.spacing.item_spacing.y *= 1.5;
                ui.set_style(Arc::new(style));

                for item in self
                    .spec
                    .sidebar_buttons
                    .iter()
                    .filter(|button| is_wallet_action_button(button))
                {
                    let label = format!("{} {}", emoji_for_sidebar_button(item), item.label);
                    let is_selected = self
                        .selected_entry_point
                        .as_deref()
                        .is_some_and(|entry| entry == item.entry_point);

                    let text = egui::RichText::new(label).font(button_font.clone());
                    let button = if is_selected {
                        egui::Button::new(text).fill(egui::Color32::from_rgb(0, 120, 170))
                    } else {
                        egui::Button::new(text)
                    };

                    ui.horizontal(|ui| {
                        if ui
                            .add_sized([wallet_action_width, wallet_action_height], button)
                            .clicked()
                        {
                            clicked_entry_point = Some(item.entry_point.clone());
                        }
                    });
                }
            });

            ui.add_space(16.0);
            ui.heading("Navigation");
            ui.separator();
            ui.add_space(14.0);

            ui.scope(|ui| {
                let mut style = (*ui.style()).as_ref().clone();
                let rounding = egui::Rounding::same(12.0);
                style.visuals.widgets.inactive.rounding = rounding;
                style.visuals.widgets.hovered.rounding = rounding;
                style.visuals.widgets.active.rounding = rounding;
                style.spacing.item_spacing.y *= 1.5;
                ui.set_style(Arc::new(style));

                let mut networks_font = button_font.clone();
                networks_font.size *= 1.2;

                egui::CollapsingHeader::new(
                    egui::RichText::new("🌐 Networks").font(networks_font),
                )
                .default_open(true)
                .show(ui, |ui| {
                    ui.indent("z00z_networks_indent", |ui| {
                        let items: [(&'static str, &'static str); 7] = [
                            ("⚡", "Devnet"),
                            ("❓", "Testnet"),
                            ("✅", "Mainnet"),
                            ("🌀", "Tor"),
                            ("🌀", "OnionNet"),
                            ("🔎", "Scanner"),
                            ("🖧", "Nodes"),
                        ];

                        for (emoji, name) in items {
                            let is_selected = self.selected_entry_point.as_deref().is_some_and(|ep| {
                                let ep_name = match ep {
                                    "network_devnet" => "Devnet",
                                    "network_testnet" => "Testnet",
                                    "network_mainnet" => "Mainnet",
                                    "network_tor" => "Tor",
                                    "network_onionnet" => "OnionNet",
                                    "network_scanner" => "Scanner",
                                    "network_nodes" => "Nodes",
                                    _ => "",
                                };
                                ep_name == name
                            });

                            let text = egui::RichText::new(format!("{emoji} {name}"))
                                .font(button_font.clone());
                            let button = if is_selected {
                                egui::Button::new(text).fill(egui::Color32::from_rgb(0, 120, 170))
                            } else {
                                egui::Button::new(text)
                            };

                            ui.horizontal(|ui| {
                                if ui
                                    .add_sized([wallet_action_width, wallet_action_height], button)
                                    .clicked()
                                {
                                    clicked_network = Some(name);
                                }
                            });
                        }
                    });
                });

                ui.add_space(22.0);

                let mut bottom_buttons: Vec<&SidebarButtonSpec> = self
                    .spec
                    .sidebar_buttons
                    .iter()
                    .filter(|button| {
                        !is_wallet_action_button(button) && !is_removed_nav_button(button)
                    })
                    .collect();

                bottom_buttons.sort_by_key(|button| match button.entry_point.as_str() {
                    "exit" => 0,
                    "config_menu" => 1,
                    "log_viewer" => 2,
                    _ => 0,
                });

                ui.allocate_space(egui::Vec2::new(0.0, 50.0));

                for item in bottom_buttons.into_iter().rev() {
                    let label = format!("{} {}", emoji_for_sidebar_button(item), item.label);
                    let is_selected = self
                        .selected_entry_point
                        .as_deref()
                        .is_some_and(|entry| entry == item.entry_point);

                    let text = egui::RichText::new(label).font(button_font.clone());
                    let button = if is_selected {
                        egui::Button::new(text).fill(egui::Color32::from_rgb(0, 120, 170))
                    } else {
                        egui::Button::new(text)
                    };

                    ui.horizontal(|ui| {
                        if ui
                            .add_sized([wallet_action_width, wallet_action_height], button)
                            .clicked()
                        {
                            clicked_entry_point = Some(item.entry_point.clone());
                        }
                    });
                }
            });
        });

        if let Some((wallet_id, wallet_name)) = clicked_wallet {
            self.set_current_wallet(&wallet_id, &wallet_name);
        }

        let should_drop = ui.ctx().input(|input| input.pointer.any_released());
        if should_drop {
            if let (Some(dragged_id), Some(target_index)) =
                (self.dragging_wallet_id.clone(), self.drag_target_index)
            {
                if let Some(from_index) = self
                    .spec
                    .sidebar_wallet_cards
                    .iter()
                    .position(|wallet| wallet.wallet_id == dragged_id)
                {
                    let mut to_index = target_index;
                    if from_index < to_index {
                        to_index = to_index.saturating_sub(1);
                    }

                    if from_index != to_index {
                        let moved = self.spec.sidebar_wallet_cards.remove(from_index);
                        let to_index = to_index.min(self.spec.sidebar_wallet_cards.len());
                        self.spec.sidebar_wallet_cards.insert(to_index, moved);
                    }
                }
            }

            self.dragging_wallet_id = None;
            self.drag_target_index = None;
        }

        if let Some(entry_point) = clicked_entry_point {
            self.selected_entry_point = Some(entry_point.clone());
            self.open_tabs_for_entry_point(&entry_point);
            self.goto_entry_point(&entry_point);
        }

        if let Some(network) = clicked_network {
            let _ = self.state_machine.context_mut().set(
                "ctx.selected_network",
                YamlValue::String(network.to_lowercase()),
            );

            let entry_point = match network.to_lowercase().as_str() {
                "devnet" => "network_devnet",
                "testnet" => "network_testnet",
                "mainnet" => "network_mainnet",
                "tor" => "network_tor",
                "onionnet" => "network_onionnet",
                "scanner" => "network_scanner",
                "nodes" => "network_nodes",
                _ => "network_devnet",
            };

            self.selected_entry_point = Some(entry_point.to_string());
            self.open_tabs_for_entry_point(entry_point);
        }
    }
}