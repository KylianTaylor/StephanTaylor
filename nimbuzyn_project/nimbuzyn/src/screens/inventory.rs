use egui::{Align, Color32, Layout, RichText, Rounding, Stroke, Vec2};
use crate::models::*;
use crate::theme::NimColors;
use crate::db::InventorySummary;

#[derive(Debug, Clone, PartialEq)]
pub enum InventoryView {
    List,
    Form,
    OutOfStock,
}

pub struct InventoryScreen {
    pub products: Vec<Product>,
    pub summary: InventorySummary,
    pub view: InventoryView,

    // Form state
    pub form: ProductForm,
    pub form_error: Option<String>,
    pub form_success: Option<String>,
    pub editing_id: Option<i64>,

    // Search
    pub search: String,
}

#[derive(Default, Clone)]
pub struct ProductForm {
    pub code: String,
    pub name: String,
    pub quantity: String,
    pub net_value: String,
    pub sale_value: String,
}

impl Default for InventoryScreen {
    fn default() -> Self {
        InventoryScreen {
            products: vec![],
            summary: InventorySummary::default(),
            view: InventoryView::List,
            form: ProductForm::default(),
            form_error: None,
            form_success: None,
            editing_id: None,
            search: String::new(),
        }
    }
}

pub enum InventoryAction {
    None,
    LoadProducts,
    SaveProduct { product: Product },
    DeleteProduct { id: i64 },
}

impl InventoryScreen {
    pub fn show(&mut self, ctx: &egui::Context, theme: &AppTheme, owner_uid: &str) -> InventoryAction {
        let c = NimColors::for_theme(theme);
        let mut action = InventoryAction::None;

        match self.view {
            InventoryView::Form => {
                action = self.show_form(ctx, &c, owner_uid);
            }
            _ => {
                action = self.show_list(ctx, &c, owner_uid);
            }
        }

        action
    }

    fn show_list(&mut self, ctx: &egui::Context, c: &NimColors, owner_uid: &str) -> InventoryAction {
        let mut action = InventoryAction::None;

        // ── Summary bar (fixed top) ────────────────────────────────────────
        egui::TopBottomPanel::top("inv_summary")
            .frame(egui::Frame::none().fill(c.bg_elevated).inner_margin(egui::style::Margin::symmetric(16.0, 12.0)))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("📦 Inventario").size(20.0).strong().color(c.text_primary));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let btn = egui::Button::new(
                            RichText::new("＋ Nuevo").size(13.0).color(Color32::WHITE),
                        )
                        .fill(c.primary)
                        .rounding(Rounding::same(8.0))
                        .min_size(Vec2::new(90.0, 32.0));
                        if ui.add(btn).clicked() {
                            self.form = ProductForm::default();
                            self.editing_id = None;
                            self.form_error = None;
                            self.form_success = None;
                            self.view = InventoryView::Form;
                        }
                    });
                });

                ui.add_space(10.0);

                // Stat cards
                ui.horizontal(|ui| {
                    stat_card(ui, c, "Productos", &self.summary.total_products.to_string(), c.text_primary);
                    stat_card(ui, c, "Valor Neto", &format_currency(self.summary.total_net_value), c.secondary);
                    stat_card(ui, c, "Ganancias", &format_currency(self.summary.total_profit_value), c.success);
                    if self.summary.out_of_stock_count > 0 {
                        stat_card(ui, c, "Sin Stock", &self.summary.out_of_stock_count.to_string(), c.danger);
                    }
                });
            });

        // ── Red alert: out-of-stock products (fixed bottom) ───────────────
        let out_of_stock: Vec<Product> = self.products.iter()
            .filter(|p| p.is_out_of_stock())
            .cloned()
            .collect();

        if !out_of_stock.is_empty() {
            egui::TopBottomPanel::bottom("oos_panel")
                .resizable(false)
                .min_height(120.0)
                .max_height(200.0)
                .frame(
                    egui::Frame::none()
                        .fill(Color32::from_rgb(0x2A, 0x0D, 0x11))
                        .stroke(Stroke::new(1.5, c.danger))
                        .inner_margin(egui::style::Margin::symmetric(16.0, 10.0)),
                )
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new("🔴 SIN STOCK")
                                .size(13.0)
                                .strong()
                                .color(c.danger),
                        );
                        ui.label(
                            RichText::new(format!("({})", out_of_stock.len()))
                                .size(12.0)
                                .color(c.danger),
                        );
                    });
                    ui.add_space(4.0);

                    egui::ScrollArea::vertical()
                        .id_source("oos_scroll")
                        .max_height(130.0)
                        .show(ui, |ui| {
                            for p in &out_of_stock {
                                ui.horizontal(|ui| {
                                    ui.label(
                                        RichText::new(format!("• {} [{}]", p.name, p.code))
                                            .size(13.0)
                                            .color(c.danger),
                                    );
                                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                        ui.label(
                                            RichText::new(format!("Costo: {}", format_currency(p.net_value)))
                                                .size(12.0)
                                                .color(c.text_muted),
                                        );
                                    });
                                });
                            }
                        });
                });
        }

        // ── Main scrollable list ──────────────────────────────────────────
        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(c.bg_base))
            .show(ctx, |ui| {
                // Search bar
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    ui.add_space(16.0);
                    ui.add(
                        egui::TextEdit::singleline(&mut self.search)
                            .hint_text("🔍 Buscar producto…")
                            .desired_width(ui.available_width() - 32.0),
                    );
                });
                ui.add_space(6.0);
                ui.separator();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    // Table header
                    ui.horizontal(|ui| {
                        ui.add_space(16.0);
                        table_header(ui, c, "Código",   80.0);
                        table_header(ui, c, "Nombre",   150.0);
                        table_header(ui, c, "Cant.",    55.0);
                        table_header(ui, c, "Neto",     90.0);
                        table_header(ui, c, "Venta",    90.0);
                        table_header(ui, c, "Ganancia", 90.0);
                    });
                    ui.separator();

                    let query = self.search.to_lowercase();
                    let products_clone = self.products.clone();
                    for p in products_clone.iter() {
                        // Filter by search
                        if !query.is_empty()
                            && !p.name.to_lowercase().contains(&query)
                            && !p.code.to_lowercase().contains(&query)
                        {
                            continue;
                        }

                        let row_h = 52.0;
                        let (rect, resp) = ui.allocate_exact_size(
                            Vec2::new(ui.available_width(), row_h),
                            egui::Sense::click(),
                        );

                        // Highlight out-of-stock with reddish background
                        let row_bg = if p.is_out_of_stock() {
                            Color32::from_rgba_premultiplied(80, 10, 15, 30)
                        } else if resp.hovered() {
                            c.bg_elevated
                        } else {
                            c.bg_base
                        };
                        ui.painter().rect_filled(rect, Rounding::ZERO, row_bg);

                        let x = rect.min.x + 16.0;
                        let y_center = rect.center().y;

                        let qty_color = if p.is_out_of_stock() { c.danger } else { c.text_primary };

                        // Draw columns
                        for (text, col_x, color) in [
                            (p.code.as_str(),                        x,          c.text_muted),
                            (p.name.as_str(),                        x + 86.0,   c.text_primary),
                            (&format!("{:.1}", p.quantity) as &str,  x + 240.0,  qty_color),
                            (&format_currency(p.net_value) as &str,  x + 297.0,  c.text_secondary),
                            (&format_currency(p.sale_value) as &str, x + 390.0,  c.text_secondary),
                            (&format_currency(p.profit_value) as &str, x + 483.0, c.success),
                        ] {
                            ui.painter().text(
                                egui::pos2(col_x, y_center),
                                egui::Align2::LEFT_CENTER,
                                text,
                                egui::FontId::proportional(13.0),
                                color,
                            );
                        }

                        // Edit / delete on click
                        if resp.clicked() {
                            self.form = ProductForm {
                                code: p.code.clone(),
                                name: p.name.clone(),
                                quantity: p.quantity.to_string(),
                                net_value: p.net_value.to_string(),
                                sale_value: p.sale_value.to_string(),
                            };
                            self.editing_id = Some(p.id);
                            self.form_error = None;
                            self.form_success = None;
                            self.view = InventoryView::Form;
                        }

                        // Row divider
                        ui.painter().line_segment(
                            [rect.left_bottom() + Vec2::new(16.0, 0.0),
                             rect.right_bottom() - Vec2::new(16.0, 0.0)],
                            Stroke::new(0.5, c.divider),
                        );
                    }

                    ui.add_space(100.0);
                });
            });

        action
    }

    fn show_form(&mut self, ctx: &egui::Context, c: &NimColors, owner_uid: &str) -> InventoryAction {
        let mut action = InventoryAction::None;

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(c.bg_base))
            .show(ctx, |ui| {
                ui.add_space(12.0);

                // Header
                ui.horizontal(|ui| {
                    ui.add_space(16.0);
                    if ui.button("← Volver").clicked() {
                        self.view = InventoryView::List;
                        return;
                    }
                    ui.label(
                        RichText::new(
                            if self.editing_id.is_some() { "Editar Producto" } else { "Nuevo Producto" },
                        )
                        .size(18.0)
                        .strong()
                        .color(c.text_primary),
                    );
                });

                ui.add_space(16.0);

                egui::ScrollArea::vertical().show(ui, |ui| {
                    let form_width = (ui.available_width() - 32.0).min(500.0);
                    ui.horizontal(|ui| {
                        ui.add_space((ui.available_width() - form_width) / 2.0);
                        ui.allocate_ui_with_layout(
                            Vec2::new(form_width, 0.0),
                            Layout::top_down(Align::Min),
                            |ui| {
                                egui::Frame::none()
                                    .fill(c.bg_card)
                                    .rounding(Rounding::same(14.0))
                                    .stroke(Stroke::new(1.0, c.border))
                                    .inner_margin(egui::style::Margin::same(20.0))
                                    .show(ui, |ui| {
                                        form_field(ui, c, "Código del producto", |ui| {
                                            ui.add(
                                                egui::TextEdit::singleline(&mut self.form.code)
                                                    .hint_text("Ej: PROD-001")
                                                    .desired_width(f32::INFINITY),
                                            );
                                        });
                                        ui.add_space(10.0);
                                        form_field(ui, c, "Nombre del producto", |ui| {
                                            ui.add(
                                                egui::TextEdit::singleline(&mut self.form.name)
                                                    .hint_text("Nombre descriptivo")
                                                    .desired_width(f32::INFINITY),
                                            );
                                        });
                                        ui.add_space(10.0);
                                        form_field(ui, c, "Cantidad", |ui| {
                                            ui.add(
                                                egui::TextEdit::singleline(&mut self.form.quantity)
                                                    .hint_text("0")
                                                    .desired_width(f32::INFINITY),
                                            );
                                        });
                                        ui.add_space(10.0);
                                        form_field(ui, c, "Valor Neto (costo)", |ui| {
                                            ui.horizontal(|ui| {
                                                ui.label(RichText::new("$").color(c.text_muted));
                                                ui.add(
                                                    egui::TextEdit::singleline(&mut self.form.net_value)
                                                        .hint_text("0.00")
                                                        .desired_width(f32::INFINITY),
                                                );
                                            });
                                        });
                                        ui.add_space(10.0);
                                        form_field(ui, c, "Valor Venta (precio)", |ui| {
                                            ui.horizontal(|ui| {
                                                ui.label(RichText::new("$").color(c.text_muted));
                                                ui.add(
                                                    egui::TextEdit::singleline(&mut self.form.sale_value)
                                                        .hint_text("0.00")
                                                        .desired_width(f32::INFINITY),
                                                );
                                            });
                                        });

                                        // Live profit preview
                                        if let (Ok(net), Ok(sale)) = (
                                            self.form.net_value.parse::<f64>(),
                                            self.form.sale_value.parse::<f64>(),
                                        ) {
                                            let profit = sale - net;
                                            ui.add_space(8.0);
                                            ui.label(
                                                RichText::new(format!(
                                                    "Ganancia unitaria: {}",
                                                    format_currency(profit)
                                                ))
                                                .color(if profit >= 0.0 { c.success } else { c.danger })
                                                .size(13.0),
                                            );
                                        }

                                        if let Some(ref err) = self.form_error {
                                            ui.add_space(8.0);
                                            ui.label(RichText::new(format!("⚠ {}", err)).color(c.danger).size(13.0));
                                        }
                                        if let Some(ref ok) = self.form_success {
                                            ui.add_space(8.0);
                                            ui.label(RichText::new(format!("✓ {}", ok)).color(c.success).size(13.0));
                                        }

                                        ui.add_space(20.0);
                                        ui.horizontal(|ui| {
                                            // Delete (if editing)
                                            if let Some(pid) = self.editing_id {
                                                let del_btn = egui::Button::new(
                                                    RichText::new("🗑 Eliminar").color(Color32::WHITE),
                                                )
                                                .fill(c.danger)
                                                .rounding(Rounding::same(8.0))
                                                .min_size(Vec2::new(120.0, 42.0));
                                                if ui.add(del_btn).clicked() {
                                                    action = InventoryAction::DeleteProduct { id: pid };
                                                    self.view = InventoryView::List;
                                                }
                                            }

                                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                                let save_btn = egui::Button::new(
                                                    RichText::new("💾 Guardar").color(Color32::WHITE).strong(),
                                                )
                                                .fill(c.primary)
                                                .rounding(Rounding::same(8.0))
                                                .min_size(Vec2::new(130.0, 42.0));

                                                if ui.add(save_btn).clicked() {
                                                    match self.build_product(owner_uid) {
                                                        Ok(p) => {
                                                            action = InventoryAction::SaveProduct { product: p };
                                                            self.form_success = Some("Guardado correctamente".into());
                                                            self.form_error = None;
                                                        }
                                                        Err(e) => {
                                                            self.form_error = Some(e);
                                                        }
                                                    }
                                                }
                                            });
                                        });
                                    });
                            },
                        );
                    });

                    ui.add_space(60.0);
                });
            });

        action
    }

    fn build_product(&self, owner_uid: &str) -> Result<Product, String> {
        let code = self.form.code.trim().to_string();
        let name = self.form.name.trim().to_string();
        if code.is_empty() { return Err("El código es obligatorio".into()); }
        if name.is_empty() { return Err("El nombre es obligatorio".into()); }

        let quantity = self.form.quantity.trim().parse::<f64>()
            .map_err(|_| "Cantidad inválida".to_string())?;
        let net_value = self.form.net_value.trim().parse::<f64>()
            .map_err(|_| "Valor neto inválido".to_string())?;
        let sale_value = self.form.sale_value.trim().parse::<f64>()
            .map_err(|_| "Valor venta inválido".to_string())?;

        if net_value < 0.0 || sale_value < 0.0 {
            return Err("Los valores no pueden ser negativos".into());
        }

        let profit_value = sale_value - net_value;
        let now = chrono::Utc::now().to_rfc3339();

        Ok(Product {
            id: self.editing_id.unwrap_or(0),
            owner_uid: owner_uid.to_string(),
            code,
            name,
            quantity,
            net_value,
            sale_value,
            profit_value,
            created_at: now.clone(),
            updated_at: now,
        })
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// HELPERS
// ──────────────────────────────────────────────────────────────────────────────

fn form_field(ui: &mut egui::Ui, c: &NimColors, label: &str, add_field: impl FnOnce(&mut egui::Ui)) {
    ui.label(RichText::new(label).size(13.0).color(c.text_secondary));
    ui.add_space(4.0);
    add_field(ui);
}

fn table_header(ui: &mut egui::Ui, c: &NimColors, label: &str, width: f32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, 24.0), egui::Sense::hover());
    ui.painter().text(
        rect.left_center(),
        egui::Align2::LEFT_CENTER,
        label,
        egui::FontId::proportional(12.0),
        c.text_muted,
    );
}

fn stat_card(ui: &mut egui::Ui, c: &NimColors, label: &str, value: &str, value_color: Color32) {
    let card_w = (ui.available_width() / 4.0).max(80.0);
    egui::Frame::none()
        .fill(c.bg_card)
        .rounding(Rounding::same(10.0))
        .stroke(Stroke::new(1.0, c.border))
        .inner_margin(egui::style::Margin::symmetric(10.0, 8.0))
        .show(ui, |ui| {
            ui.set_min_width(card_w);
            ui.label(RichText::new(value).size(17.0).strong().color(value_color));
            ui.label(RichText::new(label).size(11.0).color(c.text_muted));
        });
}

fn format_currency(v: f64) -> String {
    if v.abs() >= 1_000_000.0 {
        format!("${:.1}M", v / 1_000_000.0)
    } else if v.abs() >= 1_000.0 {
        format!("${:.1}K", v / 1_000.0)
    } else {
        format!("${:.2}", v)
    }
}
