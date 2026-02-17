use egui::{Color32, FontId, Rounding, Stroke, Style, Visuals, FontFamily};
use crate::models::AppTheme;

/// Returns a bold, modern color scheme for Nimbuzyn
pub struct NimColors {
    pub primary: Color32,
    pub primary_hover: Color32,
    pub secondary: Color32,
    pub accent: Color32,
    pub danger: Color32,
    pub warning: Color32,
    pub success: Color32,

    pub bg_base: Color32,
    pub bg_elevated: Color32,
    pub bg_card: Color32,
    pub bg_input: Color32,

    pub text_primary: Color32,
    pub text_secondary: Color32,
    pub text_muted: Color32,
    pub text_on_primary: Color32,

    pub border: Color32,
    pub divider: Color32,

    pub star_active: Color32,
    pub star_inactive: Color32,

    pub friend_tag: Color32,
    pub acquaintance_tag: Color32,
}

impl NimColors {
    pub fn dark() -> Self {
        NimColors {
            primary:         Color32::from_rgb(0x4A, 0x9C, 0xFF),   // vivid blue
            primary_hover:   Color32::from_rgb(0x2E, 0x7FE0, 0xFF),
            secondary:       Color32::from_rgb(0x6C, 0x63, 0xFF),   // indigo
            accent:          Color32::from_rgb(0x00, 0xE5, 0xFF),   // cyan accent
            danger:          Color32::from_rgb(0xFF, 0x4D, 0x6D),
            warning:         Color32::from_rgb(0xFF, 0xA5, 0x00),
            success:         Color32::from_rgb(0x00, 0xD4, 0x8A),

            bg_base:         Color32::from_rgb(0x0D, 0x0F, 0x14),
            bg_elevated:     Color32::from_rgb(0x15, 0x18, 0x21),
            bg_card:         Color32::from_rgb(0x1C, 0x20, 0x2C),
            bg_input:        Color32::from_rgb(0x22, 0x27, 0x36),

            text_primary:    Color32::from_rgb(0xED, 0xEF, 0xF4),
            text_secondary:  Color32::from_rgb(0xA8, 0xB2, 0xD0),
            text_muted:      Color32::from_rgb(0x60, 0x6A, 0x85),
            text_on_primary: Color32::WHITE,

            border:          Color32::from_rgb(0x2C, 0x33, 0x45),
            divider:         Color32::from_rgb(0x25, 0x2C, 0x3D),

            star_active:     Color32::from_rgb(0xFF, 0xD7, 0x00),
            star_inactive:   Color32::from_rgb(0x3A, 0x40, 0x52),

            friend_tag:      Color32::from_rgb(0x4A, 0x9C, 0xFF),
            acquaintance_tag:Color32::from_rgb(0x6C, 0x63, 0xFF),
        }
    }

    pub fn light() -> Self {
        NimColors {
            primary:         Color32::from_rgb(0x00, 0x6F, 0xE6),
            primary_hover:   Color32::from_rgb(0x00, 0x56, 0xB3),
            secondary:       Color32::from_rgb(0x5A, 0x52, 0xE0),
            accent:          Color32::from_rgb(0x00, 0xAF, 0xD8),
            danger:          Color32::from_rgb(0xDC, 0x3B, 0x5A),
            warning:         Color32::from_rgb(0xE8, 0x92, 0x00),
            success:         Color32::from_rgb(0x00, 0xA8, 0x6C),

            bg_base:         Color32::from_rgb(0xF4, 0xF6, 0xFA),
            bg_elevated:     Color32::from_rgb(0xFF, 0xFF, 0xFF),
            bg_card:         Color32::from_rgb(0xFF, 0xFF, 0xFF),
            bg_input:        Color32::from_rgb(0xF0, 0xF2, 0xF7),

            text_primary:    Color32::from_rgb(0x1A, 0x1F, 0x2E),
            text_secondary:  Color32::from_rgb(0x4A, 0x52, 0x68),
            text_muted:      Color32::from_rgb(0x90, 0x98, 0xB0),
            text_on_primary: Color32::WHITE,

            border:          Color32::from_rgb(0xD8, 0xDE, 0xEA),
            divider:         Color32::from_rgb(0xE8, 0xEC, 0xF4),

            star_active:     Color32::from_rgb(0xFF, 0xB8, 0x00),
            star_inactive:   Color32::from_rgb(0xC4, 0xCC, 0xDE),

            friend_tag:      Color32::from_rgb(0x00, 0x6F, 0xE6),
            acquaintance_tag:Color32::from_rgb(0x5A, 0x52, 0xE0),
        }
    }

    pub fn for_theme(theme: &AppTheme) -> Self {
        match theme {
            AppTheme::Dark  => Self::dark(),
            AppTheme::Light => Self::light(),
        }
    }
}

/// Apply custom egui visuals based on theme.
pub fn apply_theme(ctx: &egui::Context, theme: &AppTheme) {
    let c = NimColors::for_theme(theme);

    let mut visuals = match theme {
        AppTheme::Dark  => Visuals::dark(),
        AppTheme::Light => Visuals::light(),
    };

    visuals.window_fill       = c.bg_elevated;
    visuals.panel_fill        = c.bg_base;
    visuals.faint_bg_color    = c.bg_card;
    visuals.extreme_bg_color  = c.bg_input;
    visuals.code_bg_color     = c.bg_card;

    visuals.override_text_color = Some(c.text_primary);
    visuals.hyperlink_color      = c.primary;

    visuals.window_stroke = Stroke::new(1.0, c.border);
    visuals.widgets.noninteractive.bg_fill = c.bg_card;
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, c.text_secondary);
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, c.border);
    visuals.widgets.noninteractive.rounding = Rounding::same(8.0);

    visuals.widgets.inactive.bg_fill = c.bg_input;
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, c.text_secondary);
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, c.border);
    visuals.widgets.inactive.rounding = Rounding::same(8.0);

    visuals.widgets.hovered.bg_fill = c.primary;
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, c.text_on_primary);
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.5, c.primary);
    visuals.widgets.hovered.rounding = Rounding::same(8.0);

    visuals.widgets.active.bg_fill = c.primary_hover;
    visuals.widgets.active.fg_stroke = Stroke::new(1.0, c.text_on_primary);
    visuals.widgets.active.rounding = Rounding::same(8.0);

    visuals.selection.bg_fill = c.primary.linear_multiply(0.4);
    visuals.selection.stroke = Stroke::new(1.0, c.primary);

    ctx.set_visuals(visuals);
}

/// Convenience: styled button color
pub fn primary_button_color(c: &NimColors) -> egui::Color32 {
    c.primary
}
