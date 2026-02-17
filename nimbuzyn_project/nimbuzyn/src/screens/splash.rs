use egui::{Color32, Rounding, Vec2, Rect, Stroke};
use std::time::{Duration, Instant};

/// Duration of each animation phase
const RISE_DURATION:  f32 = 0.90;   // N rises up
const GLOW_DURATION:  f32 = 0.45;   // glow pulse
const FADE_DURATION:  f32 = 0.35;   // fade out
const TOTAL_DURATION: f32 = RISE_DURATION + GLOW_DURATION + FADE_DURATION + 0.25;

#[derive(Debug, Clone, PartialEq)]
pub enum SplashState {
    Running,
    Finished,
}

pub struct SplashScreen {
    pub state: SplashState,
    start: Instant,
    particles: Vec<Particle>,
}

struct Particle {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    size: f32,
    alpha: f32,
    color: u8, // 0 = orange, 1 = yellow
}

impl SplashScreen {
    pub fn new() -> Self {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        // Pseudo-random particles using a simple deterministic generator
        let mut particles = Vec::with_capacity(18);
        for i in 0u64..18 {
            let mut h = DefaultHasher::new();
            i.hash(&mut h);
            let hash = h.finish();
            let a = ((hash >> 0)  & 0xFF) as f32 / 255.0;
            let b = ((hash >> 8)  & 0xFF) as f32 / 255.0;
            let c = ((hash >> 16) & 0xFF) as f32 / 255.0;
            let d = ((hash >> 24) & 0xFF) as f32 / 255.0;
            particles.push(Particle {
                x: 0.5 + (a - 0.5) * 0.6,
                y: 0.5 + (b - 0.5) * 0.5,
                vx: (c - 0.5) * 0.004,
                vy: -(d * 0.003 + 0.001),
                size: 2.0 + a * 5.0,
                alpha: 0.0,
                color: (i % 3) as u8,
            });
        }

        SplashScreen {
            state: SplashState::Running,
            start: Instant::now(),
            particles,
        }
    }

    pub fn show(&mut self, ctx: &egui::Context) {
        let elapsed = self.start.elapsed().as_secs_f32();

        if elapsed >= TOTAL_DURATION {
            self.state = SplashState::Finished;
            return;
        }

        // Request a repaint every frame during animation
        ctx.request_repaint();

        // Compute overall alpha (for fade-out)
        let fade_start = RISE_DURATION + GLOW_DURATION + 0.25;
        let global_alpha = if elapsed > fade_start {
            let t = (elapsed - fade_start) / FADE_DURATION;
            1.0 - t.min(1.0)
        } else {
            1.0
        };

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(Self::lerp_color(
                Color32::from_rgb(11, 14, 22),
                Color32::from_rgb(11, 14, 22),
                global_alpha,
            )))
            .show(ctx, |ui| {
                let rect = ui.max_rect();
                let painter = ui.painter();
                let cx = rect.center().x;
                let cy = rect.center().y;

                // ── Background ─────────────────────────────────────────────
                painter.rect_filled(rect, Rounding::ZERO, Color32::from_rgb(11, 14, 22));

                // ── Ambient glow ring ──────────────────────────────────────
                if elapsed > RISE_DURATION * 0.6 {
                    let glow_t = ((elapsed - RISE_DURATION * 0.6)
                        / (GLOW_DURATION + 0.5))
                        .min(1.0);
                    let glow_r = ease_out_cubic(glow_t);
                    for ring in 0..8 {
                        let r_frac = ring as f32 / 7.0;
                        let ring_radius = 70.0 + r_frac * 140.0;
                        let ring_alpha = (1.0 - r_frac) * glow_r * 0.18 * global_alpha;
                        let ring_color = Color32::from_rgba_unmultiplied(
                            255,
                            130 + (r_frac * 40.0) as u8,
                            0,
                            (ring_alpha * 255.0) as u8,
                        );
                        painter.circle_stroke(
                            rect.center(),
                            ring_radius,
                            Stroke::new(1.5, ring_color),
                        );
                    }
                }

                // ── Floating particles ─────────────────────────────────────
                if elapsed > 0.4 {
                    let pt = ((elapsed - 0.4) / 1.0).min(1.0);
                    for p in &mut self.particles {
                        p.x += p.vx * 0.016;
                        p.y += p.vy * 0.016;
                        p.alpha = (pt * (1.0 - (p.y - 0.0).abs().max(0.0))).min(1.0)
                            * global_alpha;

                        let px = rect.min.x + p.x * rect.width();
                        let py = rect.min.y + p.y * rect.height();
                        let pc = match p.color {
                            0 => Color32::from_rgba_unmultiplied(255, 140, 20, (p.alpha * 180.0) as u8),
                            1 => Color32::from_rgba_unmultiplied(255, 200, 60, (p.alpha * 140.0) as u8),
                            _ => Color32::from_rgba_unmultiplied(200, 80, 0, (p.alpha * 120.0) as u8),
                        };
                        painter.circle_filled(egui::pos2(px, py), p.size * global_alpha, pc);
                    }
                }

                // ── 3D N letter animation ──────────────────────────────────
                let rise_t = (elapsed / RISE_DURATION).min(1.0);
                let rise = ease_out_bounce(rise_t);          // bounce ease
                let squash = 1.0 + (1.0 - rise) * 0.4;      // squash when low
                let stretch = 1.0 + ease_out_elastic(rise_t) * 0.08;  // stretch when rising

                // Base icon size
                let icon_size = 160.0;
                let icon_w = icon_size * (1.0 / squash).max(0.6);
                let icon_h = icon_size * stretch;

                // Y position: starts 250px below center, rises to center
                let start_y = cy + 250.0;
                let end_y   = cy - 10.0;
                let pos_y   = start_y + (end_y - start_y) * rise;

                let icon_rect = Rect::from_center_size(
                    egui::pos2(cx, pos_y),
                    Vec2::new(icon_w, icon_h),
                );

                // Draw the 3D N
                let n_alpha = (rise_t * 3.0).min(1.0) * global_alpha;
                draw_3d_N(painter, icon_rect, n_alpha, elapsed);

                // ── "Nimbuzyn" text appears ────────────────────────────────
                if elapsed > RISE_DURATION * 0.8 {
                    let text_t = ((elapsed - RISE_DURATION * 0.8) / 0.5).min(1.0);
                    let text_alpha = ease_out_cubic(text_t) * global_alpha;
                    let text_y = pos_y + icon_h * 0.5 + 24.0;

                    painter.text(
                        egui::pos2(cx, text_y),
                        egui::Align2::CENTER_TOP,
                        "Nimbuzyn",
                        egui::FontId::proportional(32.0),
                        Color32::from_rgba_unmultiplied(
                            237, 239, 244,
                            (text_alpha * 255.0) as u8,
                        ),
                    );

                    // Tagline
                    if text_t > 0.5 {
                        let tag_t = ((text_t - 0.5) / 0.5).min(1.0);
                        painter.text(
                            egui::pos2(cx, text_y + 44.0),
                            egui::Align2::CENTER_TOP,
                            "Mensajería · Inventario",
                            egui::FontId::proportional(14.0),
                            Color32::from_rgba_unmultiplied(
                                130, 145, 170,
                                (tag_t * global_alpha * 200.0) as u8,
                            ),
                        );
                    }
                }

                // ── Loading dots ───────────────────────────────────────────
                if elapsed > 1.2 && elapsed < RISE_DURATION + GLOW_DURATION + 0.25 {
                    let dot_spacing = 14.0;
                    let dot_y = cy + 200.0;
                    for i in 0i32..3 {
                        let phase = elapsed * 3.0 + i as f32 * 1.0;
                        let pulse = (phase.sin() * 0.5 + 0.5) * global_alpha;
                        painter.circle_filled(
                            egui::pos2(cx + (i - 1) as f32 * dot_spacing, dot_y),
                            4.0 * pulse,
                            Color32::from_rgba_unmultiplied(255, 140, 20, (pulse * 200.0) as u8),
                        );
                    }
                }
            });
    }

    fn lerp_color(a: Color32, b: Color32, t: f32) -> Color32 {
        Color32::from_rgba_unmultiplied(
            (a.r() as f32 + (b.r() as f32 - a.r() as f32) * t) as u8,
            (a.g() as f32 + (b.g() as f32 - a.g() as f32) * t) as u8,
            (a.b() as f32 + (b.b() as f32 - a.b() as f32) * t) as u8,
            255,
        )
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// 3D N PAINTER
// ──────────────────────────────────────────────────────────────────────────────

fn draw_3d_N(painter: &egui::Painter, rect: Rect, alpha: f32, elapsed: f32) {
    let a = |base: u8| -> u8 { (base as f32 * alpha) as u8 };

    let w = rect.width();
    let h = rect.height();
    let lx = rect.min.x;
    let ly = rect.min.y;

    let stroke = w * 0.21;
    let depth  = w * 0.07;
    let dx     = depth;
    let dy     = depth;

    // Colors with pulsing brightness
    let pulse = (elapsed * 2.5).sin() * 0.08 + 0.92;
    let orange_top  = Color32::from_rgba_unmultiplied((255.0 * pulse) as u8, a(145), a(20),  a(255));
    let orange_mid  = Color32::from_rgba_unmultiplied(a(230), a(110), a(5),   a(255));
    let orange_dark = Color32::from_rgba_unmultiplied(a(160), a(65),  a(0),   a(230));
    let orange_deep = Color32::from_rgba_unmultiplied(a(100), a(40),  a(0),   a(200));
    let highlight   = Color32::from_rgba_unmultiplied(a(255), a(210), a(90),  a(200));

    // ── Background rounded square ──────────────────────────────────────────
    painter.rect_filled(rect, Rounding::same(rect.width() * 0.2), Color32::from_rgba_unmultiplied(20, 24, 36, a(255)));

    // Subtle border glow
    painter.rect_stroke(
        rect.shrink(2.0),
        Rounding::same(rect.width() * 0.2),
        Stroke::new(1.5, Color32::from_rgba_unmultiplied(255, 140, 20, a(60))),
    );

    // ── 3D extrusion (bottom-right offset polygons) ────────────────────────
    // Left bar right side
    painter.add(egui::Shape::convex_polygon(
        vec![
            egui::pos2(lx + stroke,      ly + dy),
            egui::pos2(lx + stroke + dx, ly),
            egui::pos2(lx + stroke + dx, ly + h),
            egui::pos2(lx + stroke,      ly + h + dy),
        ],
        orange_dark,
        Stroke::NONE,
    ));
    // Left bar bottom
    painter.add(egui::Shape::convex_polygon(
        vec![
            egui::pos2(lx,          ly + h + dy),
            egui::pos2(lx + dx,     ly + h),
            egui::pos2(lx + stroke + dx, ly + h),
            egui::pos2(lx + stroke, ly + h + dy),
        ],
        orange_deep,
        Stroke::NONE,
    ));

    // Right bar extrusion
    painter.add(egui::Shape::convex_polygon(
        vec![
            egui::pos2(lx + w - stroke, ly + dy),
            egui::pos2(lx + w - stroke + dx, ly),
            egui::pos2(lx + w + dx, ly),
            egui::pos2(lx + w, ly + dy),
        ],
        orange_dark,
        Stroke::NONE,
    ));
    painter.add(egui::Shape::convex_polygon(
        vec![
            egui::pos2(lx + w - stroke, ly + dy),
            egui::pos2(lx + w, ly + dy),
            egui::pos2(lx + w, ly + h + dy),
            egui::pos2(lx + w - stroke, ly + h + dy),
        ],
        orange_dark,
        Stroke::NONE,
    ));
    painter.add(egui::Shape::convex_polygon(
        vec![
            egui::pos2(lx + w - stroke, ly + h + dy),
            egui::pos2(lx + w,          ly + h + dy),
            egui::pos2(lx + w + dx,     ly + h),
            egui::pos2(lx + w - stroke + dx, ly + h),
        ],
        orange_deep,
        Stroke::NONE,
    ));

    // ── Front face of the N ────────────────────────────────────────────────
    // Left vertical bar
    painter.rect_filled(
        Rect::from_min_max(egui::pos2(lx, ly), egui::pos2(lx + stroke, ly + h)),
        Rounding::ZERO,
        orange_mid,
    );

    // Right vertical bar
    painter.rect_filled(
        Rect::from_min_max(egui::pos2(lx + w - stroke, ly), egui::pos2(lx + w, ly + h)),
        Rounding::ZERO,
        orange_mid,
    );

    // Diagonal from top-left to bottom-right
    let d0x = lx + stroke;
    let d0y = ly + stroke * 0.4;
    let d1x = lx + w - stroke;
    let d1y = ly + h - stroke * 0.4;
    let ds  = stroke * 1.1;

    painter.add(egui::Shape::convex_polygon(
        vec![
            egui::pos2(d0x, d0y),
            egui::pos2(d0x + ds, d0y),
            egui::pos2(d1x, d1y),
            egui::pos2(d1x - ds, d1y),
        ],
        orange_mid,
        Stroke::NONE,
    ));

    // ── Highlights ─────────────────────────────────────────────────────────
    let hl_w = (stroke * 0.15).max(2.0);
    // Left edge specular
    painter.rect_filled(
        Rect::from_min_max(egui::pos2(lx, ly), egui::pos2(lx + hl_w, ly + h)),
        Rounding::ZERO,
        highlight,
    );
    // Top edges
    painter.rect_filled(
        Rect::from_min_max(egui::pos2(lx, ly), egui::pos2(lx + stroke, ly + hl_w)),
        Rounding::ZERO,
        highlight,
    );
    painter.rect_filled(
        Rect::from_min_max(egui::pos2(lx + w - stroke, ly), egui::pos2(lx + w, ly + hl_w)),
        Rounding::ZERO,
        Color32::from_rgba_unmultiplied(a(255), a(200), a(80), a(160)),
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// EASING FUNCTIONS
// ──────────────────────────────────────────────────────────────────────────────

fn ease_out_cubic(t: f32) -> f32 {
    1.0 - (1.0 - t.min(1.0)).powi(3)
}

fn ease_out_bounce(t: f32) -> f32 {
    let t = t.min(1.0);
    if t < 1.0 / 2.75 {
        7.5625 * t * t
    } else if t < 2.0 / 2.75 {
        let t2 = t - 1.5 / 2.75;
        7.5625 * t2 * t2 + 0.75
    } else if t < 2.5 / 2.75 {
        let t2 = t - 2.25 / 2.75;
        7.5625 * t2 * t2 + 0.9375
    } else {
        let t2 = t - 2.625 / 2.75;
        7.5625 * t2 * t2 + 0.984375
    }
}

fn ease_out_elastic(t: f32) -> f32 {
    if t <= 0.0 { return 0.0; }
    if t >= 1.0 { return 1.0; }
    let c4 = (2.0 * std::f32::consts::PI) / 3.0;
    2.0f32.powf(-10.0 * t) * ((t * 10.0 - 10.75) * c4).sin() + 1.0
}
