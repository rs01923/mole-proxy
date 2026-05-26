mod modules;

use eframe::egui;
use modules::config::{MoleProxyApp, ProxyConfig};
use modules::network;
use modules::ui_utils;

fn main() -> eframe::Result {
    let icon = load_icon();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 295.0])
            .with_resizable(false)
            .with_icon(icon),
        ..Default::default()
    };
    eframe::run_native(
        "Mole Proxy",
        options,
        Box::new(|cc| {
            ui_utils::configure_fonts(&cc.egui_ctx);
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Ok(Box::new(MoleProxyApp::new()))
        }),
    )
}

fn load_icon() -> egui::IconData {
    let icon_bytes = include_bytes!("../icon.png");
    let image = image::load_from_memory(icon_bytes)
        .expect("Failed to open icon.png")
        .into_rgba8();
    let (width, height) = image.dimensions();
    egui::IconData {
        rgba: image.into_raw(),
        width,
        height,
    }
}

impl eframe::App for MoleProxyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_visuals(egui::Visuals::dark());

        egui::CentralPanel::default()
            .frame(egui::Frame::central_panel(&ctx.style()).inner_margin(5.0))
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(1.0);
                    ui.heading(egui::RichText::new("Mole Proxy").size(19.0).strong());
                    ui.label(
                        egui::RichText::new("Mullvad to Minecraft proxy interface")
                            .weak()
                            .size(11.0),
                    );
                });
                ui.add_space(3.0);

                ui.add_enabled_ui(!self.is_running, |ui| {
                    ui.group(|ui| {
                        ui.set_width(ui.available_width());
                        ui.vertical_centered(|ui| {
                            egui::Grid::new("proxy_grid")
                                .num_columns(2)
                                .spacing([15.0, 6.0])
                                .min_col_width(100.0)
                                .show(ui, |ui| {
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            ui.label("Server:");
                                        },
                                    );
                                    if ui
                                        .add(
                                            egui::TextEdit::singleline(&mut self.server_addr)
                                                .desired_width(180.0)
                                                .hint_text("anticheat-test.com:25565"),
                                        )
                                        .changed()
                                    {
                                        let _ = self.save_to_file();
                                    }
                                    ui.end_row();

                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            ui.label("Listen:");
                                        },
                                    );
                                    if ui
                                        .add(
                                            egui::TextEdit::singleline(&mut self.listen_addr)
                                                .desired_width(180.0)
                                                .hint_text("0.0.0.0:25565"),
                                        )
                                        .changed()
                                    {
                                        let _ = self.save_to_file();
                                    }
                                    ui.end_row();

                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            ui.label("Location:");
                                        },
                                    );
                                    ui.vertical(|ui| {
                                        ui.horizontal(|ui| {
                                            ui.spacing_mut().item_spacing.x = 4.0;
                                            if ui
                                                .add_enabled(
                                                    self.use_custom_location,
                                                    egui::TextEdit::singleline(&mut self.location)
                                                        .desired_width(85.0)
                                                        .hint_text("e.g. us-nyc"),
                                                )
                                                .changed()
                                            {
                                                let _ = self.save_to_file();
                                            }

                                            let rand_btn = ui.add_enabled(
                                                self.use_custom_location,
                                                egui::Button::new(if self.randomize_enabled {
                                                    "🎲 Auto"
                                                } else {
                                                    "🎲 Off"
                                                })
                                                .min_size(egui::vec2(50.0, 0.0)),
                                            );
                                            if rand_btn.clicked() {
                                                self.randomize_enabled = !self.randomize_enabled;
                                                let _ = self.save_to_file();
                                            }

                                            let shuffle_btn = ui.add_enabled(
                                                self.use_custom_location,
                                                egui::Button::new("↻"),
                                            );
                                            if shuffle_btn.clicked() {
                                                match network::get_random_relay(&self.location) {
                                                    Ok((hostname, ip)) => {
                                                        self.location = hostname;
                                                        self.active_relay_ip = Some(ip);
                                                        let _ = self.save_to_file();
                                                    }
                                                    Err(e) => {
                                                        eprintln!("Failed to shuffle relay: {}", e)
                                                    }
                                                }
                                            }
                                        });

                                        ui.add_space(2.0);
                                        ui.add_enabled_ui(self.use_custom_location, |ui| {
                                            egui::ComboBox::from_id_source("country_selector")
                                                .selected_text(&self.current_country)
                                                .width(170.0)
                                                .show_ui(ui, |ui| {
                                                    for (code, name) in &self.countries {
                                                        if ui
                                                            .selectable_value(
                                                                &mut self.current_country,
                                                                name.clone(),
                                                                name,
                                                            )
                                                            .clicked()
                                                        {
                                                            self.location = code.clone();
                                                            let _ = self.save_to_file();
                                                        }
                                                    }
                                                });
                                        });
                                    });
                                    ui.end_row();

                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            ui.label("Account:");
                                        },
                                    );
                                    if ui
                                        .add(
                                            egui::TextEdit::singleline(
                                                &mut self.mullvad_account_id,
                                            )
                                            .desired_width(180.0)
                                            .password(true),
                                        )
                                        .changed()
                                    {
                                        let _ = self.save_to_file();
                                    }
                                    ui.end_row();
                                });
                        });

                        ui.add_space(2.0);
                        ui.vertical_centered(|ui| {
                            if ui
                                .checkbox(&mut self.use_custom_location, "Custom location")
                                .changed()
                            {
                                let _ = self.save_to_file();
                            }
                        });
                    });
                });

                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    let total_width = 80.0 + 10.0 + 80.0;
                    let space = (ui.available_width() - total_width) / 2.0;
                    ui.add_space(space);

                    let is_config_valid = !self.server_addr.trim().is_empty()
                        && !self.listen_addr.trim().is_empty()
                        && (!self.use_custom_location || !self.location.trim().is_empty())
                        && !self.mullvad_account_id.trim().is_empty();

                    let start_button = ui.add_enabled(
                        !self.is_running && is_config_valid,
                        egui::Button::new(egui::RichText::new("▶ Start").size(14.0))
                            .min_size(egui::vec2(80.0, 26.0)),
                    );

                    if start_button.clicked() {
                        self.normalize_addresses();
                        let _ = self.save_to_file();
                        
                        let _ = network::stop_proxy();

                        let mut final_location = if self.use_custom_location {
                            Some(self.location.clone())
                        } else {
                            None
                        };

                        if self.use_custom_location && self.randomize_enabled {
                            match network::get_random_relay(&self.location) {
                                Ok((hostname, ip)) => {
                                    println!("Randomized relay: {} ({})", hostname, ip);
                                    final_location = Some(hostname);
                                    self.active_relay_ip = Some(ip);
                                }
                                Err(e) => {
                                    eprintln!("Failed to randomize relay: {}", e);
                                    self.active_relay_ip = None;
                                }
                            }
                        } else {
                            self.active_relay_ip = None;
                        }

                        let config = ProxyConfig {
                            listen_addr: self.listen_addr.clone(),
                            target_domain: self.server_addr.clone(),
                            custom_location: final_location,
                            mullvad_account_id: self.mullvad_account_id.clone(),
                        };

                        match network::send_config(&config) {
                            Ok(res) => println!("Config sent: {}", res.status()),
                            Err(e) => eprintln!("Error sending config (timeout?): {}", e),
                        }

                        match network::start_proxy() {
                            Ok(res) => {
                                println!("Start request sent: {}", res.status());
                                self.is_running = true;
                            }
                            Err(e) => eprintln!("Error starting proxy (timeout?): {}", e),
                        }
                    }

                    ui.add_space(10.0);

                    let stop_button = ui.add_enabled(
                        self.is_running,
                        egui::Button::new(egui::RichText::new("■ Stop").size(14.0))
                            .min_size(egui::vec2(80.0, 28.0)),
                    );

                    if stop_button.clicked() {
                        match network::stop_proxy() {
                            Ok(res) => {
                                println!("Stop request sent: {}", res.status());
                                self.is_running = false;
                            }
                            Err(e) => eprintln!("Error stopping proxy (timeout?): {}", e),
                        }
                    }
                });

                ui.add_space(5.0);
                ui.separator();
                ui.add_space(3.0);

                ui.vertical_centered(|ui| {
                    let status_text = if self.is_running {
                        if let Some(ip) = &self.active_relay_ip {
                            format!("Status: Running ({})", ip)
                        } else {
                            "Status: Running".to_string()
                        }
                    } else {
                        "Status: Stopped".to_string()
                    };
                    let status_color = if self.is_running {
                        egui::Color32::from_rgb(46, 204, 113)
                    } else {
                        egui::Color32::from_rgb(231, 76, 60)
                    };

                    ui.label(
                        egui::RichText::new(status_text)
                            .color(status_color)
                            .size(14.0)
                            .strong(),
                    );
                });
            });
    }
}
