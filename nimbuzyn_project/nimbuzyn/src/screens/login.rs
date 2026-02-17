use egui::{Align, Align2, Color32, Layout, RichText, Rounding, Stroke, Vec2};
use crate::theme::NimColors;
use crate::models::AppTheme;

#[derive(Debug, Clone, PartialEq)]
pub enum AuthTab { Login, Register }

pub struct LoginScreen {
    pub tab: AuthTab,

    // Login fields
    pub login_user: String,
    pub login_pass: String,
    pub login_pass_visible: bool,
    pub login_error: Option<String>,
    pub login_loading: bool,

    // Register fields
    pub reg_user: String,
    pub reg_display: String,
    pub reg_pass: String,
    pub reg_pass2: String,
    pub reg_pass_visible: bool,
    pub reg_error: Option<String>,
    pub reg_success: Option<String>,
    pub reg_loading: bool,
}

impl Default for LoginScreen {
    fn default() -> Self {
        LoginScreen {
            tab: AuthTab::Login,
            login_user: String::new(),
            login_pass: String::new(),
            login_pass_visible: false,
            login_error: None,
            login_loading: false,
            reg_user: String::new(),
            reg_display: String::new(),
            reg_pass: String::new(),
            reg_pass2: String::new(),
            reg_pass_visible: false,
            reg_error: None,
            reg_success: None,
            reg_loading: false,
        }
    }
}

pub enum AuthAction {
    Login { username: String, password: String },
    Register { username: String, display_name: String, password: String },
    None,
}

impl LoginScreen {
    pub fn show(&mut self, ctx: &egui::Context, theme: &AppTheme) -> AuthAction {
        let c = NimColors::for_theme(theme);
        let mut action = AuthAction::None;

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(c.bg_base))
            .show(ctx, |ui| {
                ui.allocate_ui_with_layout(
                    ui.available_size(),
                    Layout::top_down(Align::Center),
                    |ui| {
                        ui.add_space(60.0);

                        // ── Logo / Brand ──────────────────────────────────
                        ui.vertical_centered(|ui| {
                            // Gradient-like logo badge
                            let (rect, _) = ui.allocate_exact_size(Vec2::new(80.0, 80.0), egui::Sense::hover());
                            ui.painter().rect_filled(rect, Rounding::same(20.0), c.primary);
                            ui.painter().text(
                                rect.center(),
                                Align2::CENTER_CENTER,
                                "N",
                                egui::FontId::proportional(48.0),
                                Color32::WHITE,
                            );

                            ui.add_space(16.0);
                            ui.label(
                                RichText::new("Nimbuzyn")
                                    .size(32.0)
                                    .color(c.text_primary)
                                    .strong(),
                            );
                            ui.label(
                                RichText::new("Mensajería · Inventario · Todo en uno")
                                    .size(13.0)
                                    .color(c.text_muted),
                            );
                        });

                        ui.add_space(40.0);

                        // ── Card container ────────────────────────────────
                        let card_width = (ui.available_width() - 32.0).min(420.0);

                        ui.allocate_ui_with_layout(
                            Vec2::new(card_width, 0.0),
                            Layout::top_down(Align::Center),
                            |ui| {
                                egui::Frame::none()
                                    .fill(c.bg_card)
                                    .rounding(Rounding::same(16.0))
                                    .stroke(Stroke::new(1.0, c.border))
                                    .inner_margin(egui::style::Margin::same(24.0))
                                    .show(ui, |ui| {
                                        // Tab selector
                                        ui.horizontal(|ui| {
                                            let tab_w = (ui.available_width() - 8.0) / 2.0;
                                            for (tab_label, tab_val) in
                                                [("Iniciar Sesión", AuthTab::Login),
                                                 ("Crear Cuenta",   AuthTab::Register)]
                                            {
                                                let selected = self.tab == tab_val;
                                                let btn = egui::Button::new(
                                                    RichText::new(tab_label)
                                                        .size(14.0)
                                                        .color(if selected { c.text_on_primary } else { c.text_secondary }),
                                                )
                                                .min_size(Vec2::new(tab_w, 40.0))
                                                .fill(if selected { c.primary } else { c.bg_input })
                                                .rounding(Rounding::same(8.0));

                                                if ui.add(btn).clicked() {
                                                    self.tab = tab_val;
                                                }
                                            }
                                        });

                                        ui.add_space(20.0);
                                        ui.separator();
                                        ui.add_space(16.0);

                                        match self.tab {
                                            AuthTab::Login => {
                                                action = self.show_login_form(ui, &c);
                                            }
                                            AuthTab::Register => {
                                                action = self.show_register_form(ui, &c);
                                            }
                                        }
                                    });
                            },
                        );
                    },
                );
            });

        action
    }

    fn show_login_form(&mut self, ui: &mut egui::Ui, c: &NimColors) -> AuthAction {
        let mut action = AuthAction::None;

        // Username
        ui.label(RichText::new("Usuario").size(13.0).color(c.text_secondary));
        ui.add_space(4.0);
        let user_resp = ui.add(
            egui::TextEdit::singleline(&mut self.login_user)
                .hint_text("Tu nombre de usuario")
                .desired_width(f32::INFINITY)
                .font(egui::FontId::proportional(15.0)),
        );
        ui.add_space(12.0);

        // Password
        ui.label(RichText::new("Contraseña").size(13.0).color(c.text_secondary));
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.add(
                egui::TextEdit::singleline(&mut self.login_pass)
                    .hint_text("••••••••")
                    .password(!self.login_pass_visible)
                    .desired_width(ui.available_width() - 50.0)
                    .font(egui::FontId::proportional(15.0)),
            );
            let eye_label = if self.login_pass_visible { "🙈" } else { "👁" };
            if ui.small_button(eye_label).clicked() {
                self.login_pass_visible = !self.login_pass_visible;
            }
        });
        ui.add_space(20.0);

        // Error
        if let Some(err) = &self.login_error {
            ui.label(
                RichText::new(format!("⚠ {}", err))
                    .size(13.0)
                    .color(c.danger),
            );
            ui.add_space(8.0);
        }

        // Login button
        let btn = egui::Button::new(
            RichText::new(if self.login_loading { "Iniciando…" } else { "Iniciar Sesión" })
                .size(15.0)
                .color(Color32::WHITE)
                .strong(),
        )
        .min_size(Vec2::new(f32::INFINITY, 48.0))
        .fill(c.primary)
        .rounding(Rounding::same(10.0));

        let enter = user_resp.lost_focus()
            && ui.input(|i| i.key_pressed(egui::Key::Enter));

        if (ui.add(btn).clicked() || enter) && !self.login_loading {
            if self.login_user.trim().is_empty() || self.login_pass.is_empty() {
                self.login_error = Some("Completa todos los campos".to_string());
            } else {
                self.login_error = None;
                action = AuthAction::Login {
                    username: self.login_user.trim().to_string(),
                    password: self.login_pass.clone(),
                };
            }
        }

        action
    }

    fn show_register_form(&mut self, ui: &mut egui::Ui, c: &NimColors) -> AuthAction {
        let mut action = AuthAction::None;

        labeled_field(ui, c, "Nombre de usuario", |ui| {
            ui.add(
                egui::TextEdit::singleline(&mut self.reg_user)
                    .hint_text("Sin espacios, único")
                    .desired_width(f32::INFINITY),
            );
        });
        ui.add_space(10.0);

        labeled_field(ui, c, "Nombre para mostrar", |ui| {
            ui.add(
                egui::TextEdit::singleline(&mut self.reg_display)
                    .hint_text("Como quieres que te vean")
                    .desired_width(f32::INFINITY),
            );
        });
        ui.add_space(10.0);

        labeled_field(ui, c, "Contraseña", |ui| {
            ui.horizontal(|ui| {
                ui.add(
                    egui::TextEdit::singleline(&mut self.reg_pass)
                        .hint_text("Mínimo 8 caracteres")
                        .password(!self.reg_pass_visible)
                        .desired_width(ui.available_width() - 50.0),
                );
                if ui.small_button(if self.reg_pass_visible { "🙈" } else { "👁" }).clicked() {
                    self.reg_pass_visible = !self.reg_pass_visible;
                }
            });
        });
        ui.add_space(10.0);

        labeled_field(ui, c, "Confirmar contraseña", |ui| {
            ui.add(
                egui::TextEdit::singleline(&mut self.reg_pass2)
                    .hint_text("Repite la contraseña")
                    .password(!self.reg_pass_visible)
                    .desired_width(f32::INFINITY),
            );
        });
        ui.add_space(20.0);

        if let Some(err) = &self.reg_error {
            ui.label(RichText::new(format!("⚠ {}", err)).size(13.0).color(c.danger));
            ui.add_space(6.0);
        }
        if let Some(ok) = &self.reg_success {
            ui.label(RichText::new(format!("✓ {}", ok)).size(13.0).color(c.success));
            ui.add_space(6.0);
        }

        let btn = egui::Button::new(
            RichText::new("Crear Cuenta").size(15.0).color(Color32::WHITE).strong(),
        )
        .min_size(Vec2::new(f32::INFINITY, 48.0))
        .fill(c.secondary)
        .rounding(Rounding::same(10.0));

        if ui.add(btn).clicked() {
            self.validate_register(&mut action);
        }

        action
    }

    fn validate_register(&mut self, action: &mut AuthAction) {
        self.reg_error = None;
        self.reg_success = None;

        let user = self.reg_user.trim().to_string();
        let display = self.reg_display.trim().to_string();

        if user.is_empty() || display.is_empty() {
            self.reg_error = Some("Todos los campos son requeridos".into());
            return;
        }
        if user.contains(' ') {
            self.reg_error = Some("El usuario no puede contener espacios".into());
            return;
        }
        if user.len() < 3 {
            self.reg_error = Some("El usuario debe tener al menos 3 caracteres".into());
            return;
        }
        if self.reg_pass.len() < 8 {
            self.reg_error = Some("La contraseña debe tener al menos 8 caracteres".into());
            return;
        }
        if self.reg_pass != self.reg_pass2 {
            self.reg_error = Some("Las contraseñas no coinciden".into());
            return;
        }

        *action = AuthAction::Register {
            username: user,
            display_name: display,
            password: self.reg_pass.clone(),
        };
    }
}

fn labeled_field(ui: &mut egui::Ui, c: &NimColors, label: &str, add_field: impl FnOnce(&mut egui::Ui)) {
    ui.label(RichText::new(label).size(13.0).color(c.text_secondary));
    ui.add_space(4.0);
    add_field(ui);
}
