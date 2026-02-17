use egui::{Align, Color32, Layout, RichText, Rounding, Vec2};
use crate::{
    db::Database,
    models::*,
    screens::{
        login::{AuthAction, LoginScreen},
        chat::{ActiveChat, ChatAction, ChatScreen},
        inventory::{InventoryAction, InventoryScreen},
        settings::{SettingsAction, SettingsScreen},
        splash::{SplashScreen, SplashState},
    },
    theme::{self, NimColors},
};

// ──────────────────────────────────────────────
// TOP-LEVEL NAVIGATION
// ──────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    Splash,
    Auth,
    Chat,
    Inventory,
    Settings,
}

// ──────────────────────────────────────────────
// APP STATE
// ──────────────────────────────────────────────

pub struct NimbuzynApp {
    pub db: Database,
    pub current_screen: Screen,
    pub current_user: Option<User>,
    pub theme: AppTheme,

    // Screen state
    pub splash_screen: SplashScreen,
    pub login_screen: LoginScreen,
    pub chat_screen: ChatScreen,
    pub inventory_screen: InventoryScreen,
    pub settings_screen: Option<SettingsScreen>,
}

impl NimbuzynApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Determine database path (platform-specific)
        let db_path = Self::db_path();
        let db = Database::open(&db_path).expect("No se pudo abrir la base de datos");

        let mut app = NimbuzynApp {
            db,
            current_screen: Screen::Splash,
            current_user: None,
            theme: AppTheme::Dark,
            splash_screen: SplashScreen::new(),
            login_screen: LoginScreen::default(),
            chat_screen: ChatScreen::default(),
            inventory_screen: InventoryScreen::default(),
            settings_screen: None,
        };

        theme::apply_theme(&cc.egui_ctx, &app.theme);
        app
    }

    fn db_path() -> String {
        #[cfg(target_os = "android")]
        {
            // On Android, use the app's files directory
            "/data/data/com.nimbuzyn.app/files/nimbuzyn.db".to_string()
        }
        #[cfg(not(target_os = "android"))]
        {
            let mut path = std::env::current_dir().unwrap_or_default();
            path.push("nimbuzyn.db");
            path.to_string_lossy().to_string()
        }
    }

    // ──────────────────────────────────────────
    // NAVIGATION
    // ──────────────────────────────────────────

    fn navigate_to(&mut self, screen: Screen, ctx: &egui::Context) {
        // Load data when navigating
        match &screen {
            Screen::Chat => {
                self.refresh_contacts();
            }
            Screen::Inventory => {
                self.refresh_products();
            }
            Screen::Settings => {
                if let Some(ref user) = self.current_user {
                    self.settings_screen = Some(SettingsScreen::new(user));
                }
            }
            _ => {}
        }
        self.current_screen = screen;
    }

    // ──────────────────────────────────────────
    // DATA REFRESH HELPERS
    // ──────────────────────────────────────────

    fn refresh_contacts(&mut self) {
        if let Some(ref user) = self.current_user {
            let uid = user.uid.clone();
            self.chat_screen.contacts_friends = self
                .db
                .get_contacts(&uid, "friend")
                .unwrap_or_default();
            self.chat_screen.contacts_acquaintances = self
                .db
                .get_contacts(&uid, "acquaintance")
                .unwrap_or_default();
        }
    }

    fn refresh_products(&mut self) {
        if let Some(ref user) = self.current_user {
            let uid = user.uid.clone();
            self.inventory_screen.products = self.db.get_products(&uid).unwrap_or_default();
            self.inventory_screen.summary = self.db.inventory_summary(&uid).unwrap_or_default();
        }
    }

    // ──────────────────────────────────────────
    // AUTH HANDLERS
    // ──────────────────────────────────────────

    fn handle_auth_action(&mut self, action: AuthAction, ctx: &egui::Context) {
        match action {
            AuthAction::Login { username, password } => {
                match self.db.login(&username, &password) {
                    Ok(user) => {
                        // Load theme preference
                        if let Ok(settings) = self.db.get_settings(&user.uid) {
                            self.theme = settings.theme;
                            theme::apply_theme(ctx, &self.theme);
                        }
                        self.current_user = Some(user);
                        self.login_screen.login_error = None;
                        self.navigate_to(Screen::Chat, ctx);
                    }
                    Err(e) => {
                        self.login_screen.login_error = Some(e.to_string());
                    }
                }
            }
            AuthAction::Register { username, display_name, password } => {
                match self.db.register_user(&username, &display_name, &password) {
                    Ok(_) => {
                        self.login_screen.reg_success =
                            Some("Cuenta creada. Ahora inicia sesión.".into());
                        self.login_screen.reg_error = None;
                        self.login_screen.tab = crate::screens::login::AuthTab::Login;
                        self.login_screen.login_user = username;
                    }
                    Err(e) => {
                        self.login_screen.reg_error = Some(e.to_string());
                    }
                }
            }
            AuthAction::None => {}
        }
    }

    // ──────────────────────────────────────────
    // CHAT HANDLERS
    // ──────────────────────────────────────────

    fn handle_chat_action(&mut self, action: ChatAction, ctx: &egui::Context) {
        let Some(ref user) = self.current_user.clone() else { return };
        let uid = user.uid.clone();

        match action {
            ChatAction::LoadContacts => self.refresh_contacts(),

            ChatAction::PreviewUser { uid: target_uid } => {
                match self.db.find_user_by_uid(&target_uid) {
                    Ok(found) => {
                        self.chat_screen.add_preview_user = Some(found);
                        self.chat_screen.add_error = None;
                    }
                    Err(e) => {
                        self.chat_screen.add_error = Some(e.to_string());
                        self.chat_screen.add_preview_user = None;
                    }
                }
            }

            ChatAction::AddContact { uid: contact_uid, contact_type } => {
                if contact_uid == uid {
                    self.chat_screen.add_error = Some("No puedes agregarte a ti mismo".into());
                    return;
                }
                match self.db.find_user_by_uid(&contact_uid) {
                    Ok(found) => {
                        let type_str = match contact_type {
                            ContactType::Friend => "friend",
                            ContactType::Acquaintance => "acquaintance",
                        };
                        let _ = self.db.add_contact(
                            &uid,
                            &found.uid,
                            &found.display_name,
                            found.avatar_color,
                            type_str,
                        );
                        self.refresh_contacts();
                        self.chat_screen.add_preview_user = None;
                        self.chat_screen.add_uid_input.clear();
                    }
                    Err(e) => {
                        self.chat_screen.add_error = Some(e.to_string());
                    }
                }
            }

            ChatAction::OpenChat { contact } => {
                if let Ok(chat) = self.db.get_or_create_chat(&uid, &contact.contact_uid) {
                    let messages = self.db.get_messages(chat.id, 100, 0).unwrap_or_default();
                    self.chat_screen.active_chat = Some(ActiveChat {
                        chat_id: chat.id,
                        contact,
                        messages,
                        input_text: String::new(),
                        scroll_to_bottom: true,
                        char_count: 0,
                        file_error: None,
                    });
                }
            }

            ChatAction::SendMessage { chat_id, content } => {
                let msg = self.db.send_message(chat_id, &uid, &content, "text", None, None);
                if let Ok(m) = msg {
                    if let Some(ref mut active) = self.chat_screen.active_chat {
                        active.messages.push(m);
                        active.scroll_to_bottom = true;
                    }
                }
            }

            ChatAction::ToggleStar { contact_uid, .. } => {
                let _ = self.db.toggle_star(&uid, &contact_uid);
                self.refresh_contacts();
            }

            ChatAction::RemoveContact { contact_uid } => {
                let _ = self.db.remove_contact(&uid, &contact_uid);
                self.refresh_contacts();
            }

            _ => {}
        }
    }

    // ──────────────────────────────────────────
    // INVENTORY HANDLERS
    // ──────────────────────────────────────────

    fn handle_inventory_action(&mut self, action: InventoryAction) {
        match action {
            InventoryAction::LoadProducts => self.refresh_products(),
            InventoryAction::SaveProduct { product } => {
                let _ = self.db.upsert_product(&product);
                self.refresh_products();
            }
            InventoryAction::DeleteProduct { id } => {
                let _ = self.db.delete_product(id);
                self.refresh_products();
            }
            InventoryAction::None => {}
        }
    }

    // ──────────────────────────────────────────
    // SETTINGS HANDLERS
    // ──────────────────────────────────────────

    fn handle_settings_action(&mut self, action: SettingsAction, ctx: &egui::Context) {
        let Some(ref user) = self.current_user.clone() else { return };

        match action {
            SettingsAction::UpdateDisplayName(name) => {
                if let Ok(()) = self.db.update_display_name(&user.uid, &name) {
                    if let Some(ref mut u) = self.current_user {
                        u.display_name = name.clone();
                    }
                    if let Some(ref mut s) = self.settings_screen {
                        s.name_success = Some("Nombre actualizado".into());
                        s.name_error = None;
                    }
                }
            }
            SettingsAction::ChangePassword { old_pass, new_pass } => {
                // Verify old password
                match self.db.login(&user.username, &old_pass) {
                    Ok(_) => {
                        if let Ok(()) = self.db.update_password(&user.uid, &new_pass) {
                            if let Some(ref mut s) = self.settings_screen {
                                s.pass_success = Some("Contraseña actualizada".into());
                                s.pass_error = None;
                                s.old_pass.clear();
                                s.new_pass.clear();
                                s.new_pass2.clear();
                            }
                        }
                    }
                    Err(_) => {
                        if let Some(ref mut s) = self.settings_screen {
                            s.pass_error = Some("Contraseña actual incorrecta".into());
                        }
                    }
                }
            }
            SettingsAction::ToggleTheme => {
                self.theme = match self.theme {
                    AppTheme::Dark => AppTheme::Light,
                    AppTheme::Light => AppTheme::Dark,
                };
                theme::apply_theme(ctx, &self.theme);
                let theme_str = match self.theme {
                    AppTheme::Dark => "dark",
                    AppTheme::Light => "light",
                };
                let _ = self.db.update_theme(&user.uid, theme_str);
            }
            SettingsAction::Logout => {
                self.current_user = None;
                self.current_screen = Screen::Auth;
                self.login_screen = LoginScreen::default();
                self.chat_screen = ChatScreen::default();
                self.inventory_screen = InventoryScreen::default();
                self.settings_screen = None;
            }
            SettingsAction::None => {}
        }
    }
}

impl eframe::App for NimbuzynApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // ── Bottom navigation bar (only when logged in) ───────────────────
        if self.current_user.is_some()
            && self.current_screen != Screen::Auth
            && self.current_screen != Screen::Splash
        {
            let c = NimColors::for_theme(&self.theme);
            let current_screen = self.current_screen.clone();

            egui::TopBottomPanel::bottom("nav_bar")
                .frame(
                    egui::Frame::none()
                        .fill(c.bg_elevated)
                        .stroke(egui::Stroke::new(1.0, c.border))
                        .inner_margin(egui::style::Margin::symmetric(0.0, 4.0)),
                )
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        let btn_w = ui.available_width() / 3.0;
                        for (icon, label, screen) in [
                            ("💬", "Chat",        Screen::Chat),
                            ("📦", "Inventario",  Screen::Inventory),
                            ("⚙",  "Cuenta",      Screen::Settings),
                        ] {
                            let selected = current_screen == screen;
                            let fg = if selected { c.primary } else { c.text_muted };
                            let bg = if selected { c.primary.linear_multiply(0.12) } else { Color32::TRANSPARENT };

                            let btn = egui::Button::new(
                                RichText::new(format!("{}\n{}", icon, label))
                                    .size(11.0)
                                    .color(fg),
                            )
                            .min_size(Vec2::new(btn_w, 54.0))
                            .fill(bg)
                            .rounding(Rounding::ZERO);

                            if ui.add(btn).clicked() && !selected {
                                self.navigate_to(screen, ctx);
                            }
                        }
                    });
                });
        }

        // ── Screen routing ────────────────────────────────────────────────
        match self.current_screen.clone() {
            Screen::Splash => {
                self.splash_screen.show(ctx);
                if self.splash_screen.state == SplashState::Finished {
                    self.current_screen = Screen::Auth;
                }
                return; // No nav bar during splash
            }

            Screen::Auth => {
                let action = self.login_screen.show(ctx, &self.theme);
                self.handle_auth_action(action, ctx);
            }

            Screen::Chat => {
                let action = {
                    let uid = self
                        .current_user
                        .as_ref()
                        .map(|u| u.uid.clone())
                        .unwrap_or_default();
                    self.chat_screen.show(ctx, &self.theme, &uid)
                };
                self.handle_chat_action(action, ctx);
            }

            Screen::Inventory => {
                let uid = self
                    .current_user
                    .as_ref()
                    .map(|u| u.uid.clone())
                    .unwrap_or_default();
                let action = self.inventory_screen.show(ctx, &self.theme, &uid);
                self.handle_inventory_action(action);
            }

            Screen::Settings => {
                if let Some(ref mut settings) = self.settings_screen {
                    let user = self.current_user.as_ref().unwrap();
                    let action = settings.show(ctx, &self.theme, user);
                    self.handle_settings_action(action, ctx);
                }
            }
        }
    }
}
