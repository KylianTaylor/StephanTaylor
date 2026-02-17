use egui::{Align, Color32, Layout, RichText, Rounding, Stroke, Vec2};
use crate::models::*;
use crate::theme::NimColors;

pub struct SettingsScreen {
    // Display name edit
    pub display_name: String,
    pub display_name_edit: bool,

    // Password change
    pub old_pass: String,
    pub new_pass: String,
    pub new_pass2: String,
    pub pass_visible: bool,
    pub pass_error: Option<String>,
    pub pass_success: Option<String>,

    pub name_error: Option<String>,
    pub name_success: Option<String>,

    pub show_logout_confirm: bool,
}

pub enum SettingsAction {
    None,
    UpdateDisplayName(String),
    ChangePassword { old_pass: String, new_pass: String },
    ToggleTheme,
    Logout,
}

impl SettingsScreen {
    pub fn new(user: &User) -> Self {
        SettingsScreen {
            display_name: user.display_name.clone(),
            display_name_edit: false,
            old_pass: String::new(),
            new_pass: String::new(),
            new_pass2: String::new(),
            pass_visible: false,
            pass_error: None,
            pass_success: None,
            name_error: None,
            name_success: None,
            show_logout_confirm: false,
        }
    }

    pub fn show(
        &mut self,
        ctx: &egui::Context,
        theme: &AppTheme,
        user: &User,
    ) -> SettingsAction {
        let c = NimColors::for_theme(theme);
        let mut action = SettingsAction::None;

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(c.bg_base))
            .show(ctx, |ui| {
                ui.add_space(12.0);
                ui.horizontal(|ui| {
                    ui.add_space(16.0);
                    ui.label(
                        RichText::new("⚙  Configuración de Cuenta")
                            .size(20.0)
                            .strong()
                            .color(c.text_primary),
                    );
                });
                ui.add_space(12.0);

                egui::ScrollArea::vertical().show(ui, |ui| {
                    let card_w = (ui.available_width() - 32.0).min(500.0);
                    ui.horizontal(|ui| {
                        ui.add_space((ui.available_width() - card_w) / 2.0);
                        ui.allocate_ui_with_layout(
                            Vec2::new(card_w, 0.0),
                            Layout::top_down(Align::Min),
                            |ui| {
                                // ── Profile Card ──────────────────────────────
                                section_card(ui, &c, |ui| {
                                    ui.horizontal(|ui| {
                                        // Avatar
                                        let (rect, _) = ui.allocate_exact_size(Vec2::splat(60.0), egui::Sense::hover());
                                        ui.painter().circle_filled(rect.center(), 30.0, c.primary);
                                        ui.painter().text(
                                            rect.center(),
                                            egui::Align2::CENTER_CENTER,
                                            user.display_name.chars().next().unwrap_or('?').to_string(),
                                            egui::FontId::proportional(26.0),
                                            Color32::WHITE,
                                        );
                                        ui.add_space(12.0);
                                        ui.vertical(|ui| {
                                            ui.label(
                                                RichText::new(&user.display_name)
                                                    .size(17.0)
                                                    .strong()
                                                    .color(c.text_primary),
                                            );
                                            ui.label(
                                                RichText::new(format!("@{}", user.username))
                                                    .size(13.0)
                                                    .color(c.text_secondary),
                                            );
                                            // UID badge
                                            egui::Frame::none()
                                                .fill(c.primary.linear_multiply(0.15))
                                                .rounding(Rounding::same(6.0))
                                                .inner_margin(egui::style::Margin::symmetric(8.0, 3.0))
                                                .show(ui, |ui| {
                                                    ui.label(
                                                        RichText::new(format!("ID: {}", user.uid))
                                                            .size(11.0)
                                                            .color(c.primary)
                                                            .monospace(),
                                                    );
                                                });
                                        });
                                    });
                                });

                                ui.add_space(12.0);

                                // ── Edit Display Name ─────────────────────────
                                section_card(ui, &c, |ui| {
                                    ui.label(
                                        RichText::new("Nombre para mostrar")
                                            .size(15.0)
                                            .strong()
                                            .color(c.text_primary),
                                    );
                                    ui.add_space(8.0);
                                    ui.horizontal(|ui| {
                                        ui.add(
                                            egui::TextEdit::singleline(&mut self.display_name)
                                                .desired_width(ui.available_width() - 90.0),
                                        );
                                        let save_btn = egui::Button::new(
                                            RichText::new("Guardar").size(13.0).color(Color32::WHITE),
                                        )
                                        .fill(c.primary)
                                        .rounding(Rounding::same(8.0));
                                        if ui.add(save_btn).clicked() {
                                            let new_name = self.display_name.trim().to_string();
                                            if new_name.is_empty() {
                                                self.name_error = Some("El nombre no puede estar vacío".into());
                                            } else {
                                                action = SettingsAction::UpdateDisplayName(new_name);
                                            }
                                        }
                                    });
                                    if let Some(ref e) = self.name_error {
                                        ui.label(RichText::new(e).color(c.danger).size(12.0));
                                    }
                                    if let Some(ref s) = self.name_success {
                                        ui.label(RichText::new(s).color(c.success).size(12.0));
                                    }
                                });

                                ui.add_space(12.0);

                                // ── Change Password ───────────────────────────
                                section_card(ui, &c, |ui| {
                                    ui.label(
                                        RichText::new("Cambiar contraseña")
                                            .size(15.0)
                                            .strong()
                                            .color(c.text_primary),
                                    );
                                    ui.add_space(8.0);

                                    for (label, field) in [
                                        ("Contraseña actual", &mut self.old_pass),
                                        ("Nueva contraseña", &mut self.new_pass),
                                        ("Confirmar nueva", &mut self.new_pass2),
                                    ] {
                                        ui.label(RichText::new(label).size(12.0).color(c.text_secondary));
                                        ui.add_space(3.0);
                                        ui.add(
                                            egui::TextEdit::singleline(field)
                                                .password(!self.pass_visible)
                                                .desired_width(f32::INFINITY),
                                        );
                                        ui.add_space(6.0);
                                    }

                                    ui.horizontal(|ui| {
                                        ui.checkbox(&mut self.pass_visible, "Mostrar contraseñas");
                                    });

                                    if let Some(ref e) = self.pass_error {
                                        ui.label(RichText::new(format!("⚠ {}", e)).color(c.danger).size(12.0));
                                    }
                                    if let Some(ref s) = self.pass_success {
                                        ui.label(RichText::new(format!("✓ {}", s)).color(c.success).size(12.0));
                                    }

                                    ui.add_space(8.0);
                                    let btn = egui::Button::new(
                                        RichText::new("Cambiar contraseña").color(Color32::WHITE),
                                    )
                                    .fill(c.secondary)
                                    .rounding(Rounding::same(8.0))
                                    .min_size(Vec2::new(f32::INFINITY, 38.0));
                                    if ui.add(btn).clicked() {
                                        if self.new_pass != self.new_pass2 {
                                            self.pass_error = Some("Las contraseñas no coinciden".into());
                                        } else if self.new_pass.len() < 8 {
                                            self.pass_error = Some("Mínimo 8 caracteres".into());
                                        } else {
                                            action = SettingsAction::ChangePassword {
                                                old_pass: self.old_pass.clone(),
                                                new_pass: self.new_pass.clone(),
                                            };
                                        }
                                    }
                                });

                                ui.add_space(12.0);

                                // ── Theme ──────────────────────────────────────
                                section_card(ui, &c, |ui| {
                                    ui.label(
                                        RichText::new("Apariencia")
                                            .size(15.0)
                                            .strong()
                                            .color(c.text_primary),
                                    );
                                    ui.add_space(8.0);
                                    ui.horizontal(|ui| {
                                        let is_dark = theme == &AppTheme::Dark;
                                        ui.label(
                                            RichText::new(if is_dark { "🌙 Modo Oscuro" } else { "☀️ Modo Claro" })
                                                .size(14.0)
                                                .color(c.text_secondary),
                                        );
                                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                            let toggle_label = if is_dark { "Cambiar a Claro" } else { "Cambiar a Oscuro" };
                                            let toggle_btn = egui::Button::new(
                                                RichText::new(toggle_label).size(13.0).color(Color32::WHITE),
                                            )
                                            .fill(c.primary)
                                            .rounding(Rounding::same(8.0))
                                            .min_size(Vec2::new(150.0, 34.0));
                                            if ui.add(toggle_btn).clicked() {
                                                action = SettingsAction::ToggleTheme;
                                            }
                                        });
                                    });
                                });

                                ui.add_space(12.0);

                                // ── Logout ─────────────────────────────────────
                                section_card(ui, &c, |ui| {
                                    let logout_btn = egui::Button::new(
                                        RichText::new("🚪 Cerrar Sesión").size(15.0).color(Color32::WHITE).strong(),
                                    )
                                    .fill(c.danger)
                                    .rounding(Rounding::same(10.0))
                                    .min_size(Vec2::new(f32::INFINITY, 48.0));
                                    if ui.add(logout_btn).clicked() {
                                        self.show_logout_confirm = true;
                                    }
                                });

                                ui.add_space(40.0);
                            },
                        );
                    });
                });
            });

        // ── Logout confirmation dialog ─────────────────────────────────────
        if self.show_logout_confirm {
            egui::Window::new("¿Cerrar sesión?")
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .collapsible(false)
                .resizable(false)
                .frame(
                    egui::Frame::window(&ctx.style())
                        .fill(c.bg_card)
                        .stroke(Stroke::new(1.0, c.border))
                        .rounding(Rounding::same(14.0)),
                )
                .show(ctx, |ui| {
                    ui.label(
                        RichText::new("¿Estás seguro que deseas cerrar sesión?")
                            .color(c.text_secondary),
                    );
                    ui.add_space(16.0);
                    ui.horizontal(|ui| {
                        if ui
                            .add(
                                egui::Button::new("Cancelar")
                                    .fill(c.bg_input)
                                    .rounding(Rounding::same(8.0))
                                    .min_size(Vec2::new(120.0, 38.0)),
                            )
                            .clicked()
                        {
                            self.show_logout_confirm = false;
                        }
                        if ui
                            .add(
                                egui::Button::new(RichText::new("Cerrar sesión").color(Color32::WHITE))
                                    .fill(c.danger)
                                    .rounding(Rounding::same(8.0))
                                    .min_size(Vec2::new(120.0, 38.0)),
                            )
                            .clicked()
                        {
                            self.show_logout_confirm = false;
                            action = SettingsAction::Logout;
                        }
                    });
                });
        }

        action
    }
}

fn section_card(ui: &mut egui::Ui, c: &NimColors, add_contents: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::none()
        .fill(c.bg_card)
        .rounding(Rounding::same(14.0))
        .stroke(Stroke::new(1.0, c.border))
        .inner_margin(egui::style::Margin::same(16.0))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            add_contents(ui);
        });
}
