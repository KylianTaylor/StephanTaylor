use egui::{Align, Color32, Layout, Rounding, RichText, Stroke, Vec2};
use crate::models::*;
use crate::theme::NimColors;
use crate::db::{Database};

#[derive(Debug, Clone, PartialEq)]
pub enum ChatTab { Friends, Acquaintances }

pub struct ChatScreen {
    pub tab: ChatTab,
    pub contacts_friends: Vec<Contact>,
    pub contacts_acquaintances: Vec<Contact>,

    // Add contact dialog
    pub show_add_dialog: bool,
    pub add_uid_input: String,
    pub add_type: ContactType,
    pub add_error: Option<String>,
    pub add_preview_user: Option<User>,

    // Active chat
    pub active_chat: Option<ActiveChat>,
}

pub struct ActiveChat {
    pub contact: Contact,
    pub chat_id: i64,
    pub messages: Vec<Message>,
    pub input_text: String,
    pub scroll_to_bottom: bool,
    pub char_count: usize,
    pub file_error: Option<String>,
}

impl Default for ChatScreen {
    fn default() -> Self {
        ChatScreen {
            tab: ChatTab::Friends,
            contacts_friends: vec![],
            contacts_acquaintances: vec![],
            show_add_dialog: false,
            add_uid_input: String::new(),
            add_type: ContactType::Friend,
            add_error: None,
            add_preview_user: None,
            active_chat: None,
        }
    }
}

pub enum ChatAction {
    None,
    LoadContacts,
    AddContact { uid: String, contact_type: ContactType },
    OpenChat { contact: Contact },
    SendMessage { chat_id: i64, content: String },
    SendFile { chat_id: i64, path: String },
    ToggleStar { contact_uid: String, contact_type: ContactType },
    RemoveContact { contact_uid: String },
    PreviewUser { uid: String },
}

impl ChatScreen {
    pub fn show(&mut self, ctx: &egui::Context, theme: &AppTheme, current_uid: &str) -> ChatAction {
        let c = NimColors::for_theme(theme);
        let mut action = ChatAction::None;

        if let Some(ref mut active) = self.active_chat {
            // ── Full screen chat window ────────────────────────────────────
            action = show_chat_window(ctx, &c, active, current_uid);
        } else {
            // ── Contacts list ──────────────────────────────────────────────
            egui::CentralPanel::default()
                .frame(egui::Frame::none().fill(c.bg_base))
                .show(ctx, |ui| {
                    // Top bar
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        ui.add_space(16.0);
                        ui.label(RichText::new("💬 Chat").size(20.0).strong().color(c.text_primary));
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            ui.add_space(16.0);
                            let add_btn = egui::Button::new(
                                RichText::new("＋ Agregar").size(13.0).color(Color32::WHITE),
                            )
                            .fill(c.primary)
                            .rounding(Rounding::same(8.0))
                            .min_size(Vec2::new(100.0, 32.0));

                            if ui.add(add_btn).clicked() {
                                self.show_add_dialog = true;
                            }
                        });
                    });
                    ui.add_space(12.0);

                    // Tab selector
                    ui.horizontal(|ui| {
                        ui.add_space(16.0);
                        for (label, tab) in [
                            ("⭐ Amigos", ChatTab::Friends),
                            ("👥 Conocidos", ChatTab::Acquaintances),
                        ] {
                            let selected = self.tab == tab;
                            let btn = egui::Button::new(
                                RichText::new(label)
                                    .size(13.0)
                                    .color(if selected { Color32::WHITE } else { c.text_secondary }),
                            )
                            .min_size(Vec2::new(130.0, 36.0))
                            .fill(if selected { c.primary } else { c.bg_card })
                            .rounding(Rounding::same(8.0));
                            if ui.add(btn).clicked() {
                                self.tab = tab;
                                action = ChatAction::LoadContacts;
                            }
                        }
                    });

                    ui.add_space(8.0);
                    ui.separator();

                    // Contact list
                    let contacts: &Vec<Contact> = match self.tab {
                        ChatTab::Friends       => &self.contacts_friends,
                        ChatTab::Acquaintances => &self.contacts_acquaintances,
                    };

                    if contacts.is_empty() {
                        ui.add_space(60.0);
                        ui.vertical_centered(|ui| {
                            ui.label(RichText::new("😶‍🌫️").size(48.0));
                            ui.add_space(8.0);
                            ui.label(
                                RichText::new("Sin contactos todavía")
                                    .size(16.0)
                                    .color(c.text_muted),
                            );
                            ui.label(
                                RichText::new("Toca ＋ Agregar para añadir a alguien")
                                    .size(12.0)
                                    .color(c.text_muted),
                            );
                        });
                    } else {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            let contacts_clone = contacts.clone();
                            for contact in contacts_clone.iter() {
                                let row_resp = contact_row(ui, &c, contact);
                                if row_resp.chat_clicked {
                                    action = ChatAction::OpenChat { contact: contact.clone() };
                                }
                                if row_resp.star_clicked {
                                    action = ChatAction::ToggleStar {
                                        contact_uid: contact.contact_uid.clone(),
                                        contact_type: contact.contact_type.clone(),
                                    };
                                }
                                if row_resp.remove_clicked {
                                    action = ChatAction::RemoveContact {
                                        contact_uid: contact.contact_uid.clone(),
                                    };
                                }
                            }
                            ui.add_space(80.0);
                        });
                    }
                });
        }

        // ── Add Contact Dialog ─────────────────────────────────────────────
        if self.show_add_dialog {
            let dialog_action = show_add_dialog(ctx, &c, self);
            match dialog_action {
                AddDialogAction::Confirm { uid, contact_type } => {
                    self.show_add_dialog = false;
                    action = ChatAction::AddContact { uid, contact_type };
                }
                AddDialogAction::Close => {
                    self.show_add_dialog = false;
                    self.add_uid_input.clear();
                    self.add_error = None;
                    self.add_preview_user = None;
                }
                AddDialogAction::SearchUid { uid } => {
                    action = ChatAction::PreviewUser { uid };
                }
                AddDialogAction::None => {}
            }
        }

        action
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// CONTACT ROW WIDGET
// ──────────────────────────────────────────────────────────────────────────────

struct ContactRowResponse {
    chat_clicked:   bool,
    star_clicked:   bool,
    remove_clicked: bool,
}

fn contact_row(ui: &mut egui::Ui, c: &NimColors, contact: &Contact) -> ContactRowResponse {
    let mut resp = ContactRowResponse {
        chat_clicked: false,
        star_clicked: false,
        remove_clicked: false,
    };

    let row_h = 72.0;
    let (rect, row_response) =
        ui.allocate_exact_size(Vec2::new(ui.available_width(), row_h), egui::Sense::click());

    if row_response.hovered() {
        ui.painter().rect_filled(rect, Rounding::ZERO, c.bg_elevated);
    }

    if row_response.clicked() {
        resp.chat_clicked = true;
    }

    // Avatar circle
    let avatar_rect = egui::Rect::from_min_size(
        rect.min + Vec2::new(16.0, (row_h - 48.0) / 2.0),
        Vec2::splat(48.0),
    );
    let initials = contact
        .display_name
        .chars()
        .next()
        .map(|c| c.to_uppercase().to_string())
        .unwrap_or_else(|| "?".to_string());
    let av_color = Color32::from_rgba_premultiplied(
        ((contact.avatar_color >> 24) & 0xFF) as u8,
        ((contact.avatar_color >> 16) & 0xFF) as u8,
        ((contact.avatar_color >> 8)  & 0xFF) as u8,
        (contact.avatar_color & 0xFF) as u8,
    );
    ui.painter().circle_filled(avatar_rect.center(), 24.0, av_color);
    ui.painter().text(
        avatar_rect.center(),
        egui::Align2::CENTER_CENTER,
        &initials,
        egui::FontId::proportional(20.0),
        Color32::WHITE,
    );

    // Name & UID
    let name_pos = rect.min + Vec2::new(76.0, 14.0);
    ui.painter().text(
        name_pos,
        egui::Align2::LEFT_TOP,
        &contact.display_name,
        egui::FontId::proportional(15.0),
        c.text_primary,
    );
    ui.painter().text(
        name_pos + Vec2::new(0.0, 22.0),
        egui::Align2::LEFT_TOP,
        &contact.contact_uid,
        egui::FontId::proportional(12.0),
        c.text_muted,
    );

    // Star button (top-right)
    let star_center = rect.max - Vec2::new(48.0, row_h / 2.0);
    let star_rect = egui::Rect::from_center_size(star_center, Vec2::splat(32.0));
    let star_resp = ui.allocate_rect(star_rect, egui::Sense::click());
    let star_color = if contact.starred { c.star_active } else { c.star_inactive };
    ui.painter().text(
        star_center,
        egui::Align2::CENTER_CENTER,
        "★",
        egui::FontId::proportional(22.0),
        star_color,
    );
    if star_resp.clicked() {
        resp.star_clicked = true;
    }

    // Divider
    ui.painter().line_segment(
        [rect.left_bottom() + Vec2::new(16.0, 0.0), rect.right_bottom() - Vec2::new(16.0, 0.0)],
        Stroke::new(1.0, c.divider),
    );

    resp
}

// ──────────────────────────────────────────────────────────────────────────────
// ADD CONTACT DIALOG
// ──────────────────────────────────────────────────────────────────────────────

enum AddDialogAction {
    None,
    Close,
    SearchUid { uid: String },
    Confirm { uid: String, contact_type: ContactType },
}

fn show_add_dialog(ctx: &egui::Context, c: &NimColors, screen: &mut ChatScreen) -> AddDialogAction {
    let mut action = AddDialogAction::None;

    egui::Window::new("Agregar Contacto")
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .resizable(false)
        .collapsible(false)
        .frame(
            egui::Frame::window(&ctx.style())
                .fill(c.bg_card)
                .stroke(Stroke::new(1.0, c.border))
                .rounding(Rounding::same(14.0)),
        )
        .show(ctx, |ui| {
            ui.set_min_width(320.0);
            ui.set_max_width(380.0);

            ui.label(
                RichText::new("Ingresa el ID único del usuario")
                    .size(13.0)
                    .color(c.text_secondary),
            );
            ui.add_space(8.0);

            ui.horizontal(|ui| {
                ui.add(
                    egui::TextEdit::singleline(&mut screen.add_uid_input)
                        .hint_text("Ej: NIM-4F2A3B")
                        .desired_width(ui.available_width() - 80.0),
                );
                let search_btn = egui::Button::new("Buscar")
                    .fill(c.secondary)
                    .rounding(Rounding::same(8.0));
                if ui.add(search_btn).clicked() && !screen.add_uid_input.trim().is_empty() {
                    action = AddDialogAction::SearchUid {
                        uid: screen.add_uid_input.trim().to_uppercase(),
                    };
                }
            });

            ui.add_space(8.0);

            // Contact type toggle
            ui.label(RichText::new("Tipo de contacto").size(13.0).color(c.text_secondary));
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                for (label, val) in [("⭐ Amigo", ContactType::Friend), ("👥 Conocido", ContactType::Acquaintance)] {
                    let selected = screen.add_type == val;
                    let btn = egui::Button::new(RichText::new(label).size(13.0).color(
                        if selected { Color32::WHITE } else { c.text_secondary },
                    ))
                    .fill(if selected { c.primary } else { c.bg_input })
                    .rounding(Rounding::same(8.0))
                    .min_size(Vec2::new(130.0, 32.0));
                    if ui.add(btn).clicked() {
                        screen.add_type = val;
                    }
                }
            });

            // Preview found user
            if let Some(ref user) = screen.add_preview_user {
                ui.add_space(10.0);
                egui::Frame::none()
                    .fill(c.bg_input)
                    .rounding(Rounding::same(10.0))
                    .inner_margin(egui::style::Margin::same(10.0))
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new(format!("✓ {}", user.display_name))
                                .color(c.success)
                                .strong(),
                        );
                        ui.label(
                            RichText::new(format!("@{} · {}", user.username, user.uid))
                                .size(12.0)
                                .color(c.text_muted),
                        );
                    });
            }

            if let Some(ref err) = screen.add_error {
                ui.add_space(6.0);
                ui.label(RichText::new(format!("⚠ {}", err)).color(c.danger).size(13.0));
            }

            ui.add_space(16.0);

            ui.horizontal(|ui| {
                if ui
                    .add(
                        egui::Button::new(RichText::new("Cancelar").color(c.text_secondary))
                            .fill(c.bg_input)
                            .rounding(Rounding::same(8.0))
                            .min_size(Vec2::new(120.0, 38.0)),
                    )
                    .clicked()
                {
                    action = AddDialogAction::Close;
                }

                let can_confirm = screen.add_preview_user.is_some();
                let confirm_btn = egui::Button::new(
                    RichText::new("Agregar").color(Color32::WHITE).strong(),
                )
                .fill(if can_confirm { c.primary } else { c.text_muted })
                .rounding(Rounding::same(8.0))
                .min_size(Vec2::new(120.0, 38.0));

                if ui.add(confirm_btn).clicked() && can_confirm {
                    action = AddDialogAction::Confirm {
                        uid: screen.add_uid_input.trim().to_uppercase(),
                        contact_type: screen.add_type.clone(),
                    };
                }
            });
        });

    action
}

// ──────────────────────────────────────────────────────────────────────────────
// ACTIVE CHAT WINDOW
// ──────────────────────────────────────────────────────────────────────────────

fn show_chat_window(
    ctx: &egui::Context,
    c: &NimColors,
    active: &mut ActiveChat,
    current_uid: &str,
) -> ChatAction {
    let mut action = ChatAction::None;

    // Header
    egui::TopBottomPanel::top("chat_header")
        .frame(egui::Frame::none().fill(c.bg_elevated).inner_margin(egui::style::Margin::symmetric(16.0, 12.0)))
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("←").clicked() {
                    // This will be handled in app.rs by setting active_chat = None
                }
                ui.add_space(8.0);
                // Avatar
                let (rect, _) = ui.allocate_exact_size(Vec2::splat(36.0), egui::Sense::hover());
                ui.painter().circle_filled(rect.center(), 18.0, c.primary);
                ui.painter().text(
                    rect.center(), egui::Align2::CENTER_CENTER,
                    active.contact.display_name.chars().next().unwrap_or('?').to_string(),
                    egui::FontId::proportional(16.0), Color32::WHITE,
                );
                ui.add_space(8.0);
                ui.vertical(|ui| {
                    ui.label(RichText::new(&active.contact.display_name).strong().color(c.text_primary).size(15.0));
                    ui.label(RichText::new(&active.contact.contact_uid).size(11.0).color(c.text_muted));
                });
            });
        });

    // Message input at bottom
    egui::TopBottomPanel::bottom("chat_input")
        .frame(egui::Frame::none().fill(c.bg_elevated).inner_margin(egui::style::Margin::symmetric(12.0, 10.0)))
        .show(ctx, |ui| {
            let remaining = Message::MAX_TEXT_LEN.saturating_sub(active.input_text.len());
            ui.horizontal(|ui| {
                // File attach button
                let attach_btn = egui::Button::new("📎")
                    .fill(c.bg_input)
                    .rounding(Rounding::same(8.0))
                    .min_size(Vec2::splat(42.0));
                if ui.add(attach_btn).clicked() {
                    // On Android, file picker would be triggered via JNI
                    // For now show placeholder message
                    active.file_error = Some("Selector de archivos (implementar via JNI Android)".into());
                }

                let text_edit = egui::TextEdit::multiline(&mut active.input_text)
                    .hint_text("Escribe un mensaje…")
                    .desired_width(ui.available_width() - 55.0)
                    .desired_rows(1)
                    .font(egui::FontId::proportional(14.0));
                let te_resp = ui.add(text_edit);

                // Enforce max length
                if active.input_text.len() > Message::MAX_TEXT_LEN {
                    active.input_text.truncate(Message::MAX_TEXT_LEN);
                }

                let send_btn = egui::Button::new(RichText::new("➤").size(18.0).color(Color32::WHITE))
                    .fill(c.primary)
                    .rounding(Rounding::same(10.0))
                    .min_size(Vec2::splat(42.0));

                let send = ui.add(send_btn).clicked()
                    || (te_resp.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter) && !i.modifiers.shift));

                if send && !active.input_text.trim().is_empty() {
                    action = ChatAction::SendMessage {
                        chat_id: active.chat_id,
                        content: active.input_text.trim().to_string(),
                    };
                    active.input_text.clear();
                    active.scroll_to_bottom = true;
                }
            });

            // Char counter
            if active.input_text.len() > 800 {
                ui.label(
                    RichText::new(format!("{}/1000", active.input_text.len()))
                        .size(11.0)
                        .color(if remaining < 50 { c.danger } else { c.text_muted }),
                );
            }

            if let Some(ref err) = active.file_error {
                ui.label(RichText::new(err).size(11.0).color(c.warning));
            }
        });

    // Message bubbles
    egui::CentralPanel::default()
        .frame(egui::Frame::none().fill(c.bg_base))
        .show(ctx, |ui| {
            let scroll = egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .stick_to_bottom(active.scroll_to_bottom);

            scroll.show(ui, |ui| {
                ui.add_space(8.0);
                let messages = active.messages.clone();
                for msg in &messages {
                    let is_mine = msg.sender_uid == current_uid;
                    message_bubble(ui, c, msg, is_mine);
                }
                active.scroll_to_bottom = false;
                ui.add_space(8.0);
            });
        });

    action
}

fn message_bubble(ui: &mut egui::Ui, c: &NimColors, msg: &Message, is_mine: bool) {
    let bubble_max_w = ui.available_width() * 0.72;
    let layout = if is_mine {
        Layout::right_to_left(Align::Min)
    } else {
        Layout::left_to_right(Align::Min)
    };

    ui.with_layout(Layout::top_down(if is_mine { Align::Max } else { Align::Min }), |ui| {
        ui.add_space(4.0);
        let bg = if is_mine { c.primary } else { c.bg_card };
        let fg = if is_mine { Color32::WHITE } else { c.text_primary };

        let content = match &msg.msg_type {
            MessageType::Text => msg.content.clone(),
            other => format!(
                "{} {}",
                other.icon(),
                msg.file_name.as_deref().unwrap_or("archivo")
            ),
        };

        egui::Frame::none()
            .fill(bg)
            .rounding(Rounding {
                nw: if is_mine { 14.0 } else { 4.0 },
                ne: if is_mine { 4.0 } else { 14.0 },
                sw: 14.0,
                se: 14.0,
            })
            .inner_margin(egui::style::Margin::symmetric(12.0, 8.0))
            .show(ui, |ui| {
                ui.set_max_width(bubble_max_w);
                ui.label(RichText::new(&content).size(14.0).color(fg));

                // Timestamp
                let time_str = msg.sent_at.get(11..16).unwrap_or("");
                ui.label(
                    RichText::new(time_str)
                        .size(10.0)
                        .color(if is_mine { Color32::from_white_alpha(150) } else { c.text_muted }),
                );
            });

        ui.add_space(2.0);
    });
}
