use eframe::egui::{
    self, Color32, CornerRadius, FontId, Margin, RichText, Stroke, StrokeKind, Vec2,
};

// ---------------------------------------------------------------------------
// macOS Tahoe — Liquid Glass Dark
// ---------------------------------------------------------------------------
//
// Palet mengacu pada macOS 26 "Tahoe" Human Interface Guidelines:
//   • Background: lapisan Abu-abu gelap dengan nilai vibrancy (efek kaca)
//   • Sidebar    : glass putih sangat tipis di atas bg (seperti NSSidebar)
//   • Card       : "Glass fill" — putih transparan rendah + highlight atas
//   • Accent     : Biru sistem Apple  (#007AFF / macOS blue)
//   • Traffic-light: merah #FF5F57 · kuning #FFBD2E · hijau #28C840
//   • Typography : SF Pro Display / SF Pro Text — diwakili proporsional egui

// ── Mode terang / gelap ────────────────────────────────────────────────────
use std::sync::atomic::{AtomicBool, Ordering};

static DARK_MODE: AtomicBool = AtomicBool::new(true);

/// Ubah mode tampilan. Panggil `apply` setelahnya agar widget ikut berubah.
pub fn set_dark(dark: bool) {
    DARK_MODE.store(dark, Ordering::Relaxed);
}

pub fn is_dark() -> bool {
    DARK_MODE.load(Ordering::Relaxed)
}

/// Pilih warna sesuai mode aktif.
#[inline]
fn pick(dark: Color32, light: Color32) -> Color32 {
    if is_dark() {
        dark
    } else {
        light
    }
}

// ── Latar & panel ──────────────────────────────────────────────────────────
/// Latar utama: abu gelap hangat (mode gelap) atau abu terang macOS.
pub fn bg() -> Color32 {
    pick(
        Color32::from_rgb(28, 28, 30),
        Color32::from_rgb(242, 242, 247),
    )
}
/// Ujung atas gradasi latar — sedikit lebih terang (cahaya ruangan).
pub fn bg_top() -> Color32 {
    pick(
        Color32::from_rgb(40, 40, 46),
        Color32::from_rgb(252, 252, 254),
    )
}
/// Ujung bawah gradasi latar — lebih dalam agar panel kaca terasa melayang.
pub fn bg_bottom() -> Color32 {
    pick(
        Color32::from_rgb(19, 19, 22),
        Color32::from_rgb(226, 226, 233),
    )
}

/// Sidebar: putih sangat transparan — efek "liquid glass" sidebar macOS.
pub fn sidebar() -> Color32 {
    pick(
        Color32::from_rgba_premultiplied(16, 16, 16, 16),
        Color32::from_rgba_premultiplied(14, 14, 14, 14),
    )
}

/// Fill kartu (NSVisualEffectView Material Popover).
pub fn card_fill() -> Color32 {
    pick(
        Color32::from_rgba_premultiplied(20, 20, 20, 20),
        Color32::from_rgba_unmultiplied(255, 255, 255, 205),
    )
}

/// Kartu saat di-hover — sedikit lebih terang.
pub fn card_hover() -> Color32 {
    pick(
        Color32::from_rgba_premultiplied(34, 34, 34, 34),
        Color32::from_rgba_unmultiplied(255, 255, 255, 235),
    )
}

/// Input field — lebih gelap dari kartu di mode gelap, putih di mode terang.
pub fn input() -> Color32 {
    pick(
        Color32::from_rgba_premultiplied(0, 0, 0, 90),
        Color32::from_rgba_unmultiplied(255, 255, 255, 235),
    )
}

/// Latar meja kerja pratinjau — tetap kontras dengan kertas putih.
pub fn canvas() -> Color32 {
    pick(
        Color32::from_rgba_unmultiplied(0, 0, 0, 110),
        Color32::from_rgb(199, 199, 205),
    )
}

// ── Garis & pemisah ────────────────────────────────────────────────────────
pub fn border() -> Color32 {
    pick(
        Color32::from_rgba_premultiplied(30, 30, 30, 30),
        Color32::from_rgba_premultiplied(36, 36, 36, 36),
    )
}

pub fn border_soft() -> Color32 {
    pick(
        Color32::from_rgba_premultiplied(16, 16, 16, 16),
        Color32::from_rgba_premultiplied(20, 20, 20, 20),
    )
}

// ── Teks ────────────────────────────────────────────────────────────────────
pub fn text() -> Color32 {
    pick(
        Color32::from_rgb(245, 245, 247),
        Color32::from_rgb(28, 28, 30),
    )
}

pub fn muted() -> Color32 {
    pick(
        Color32::from_rgba_premultiplied(150, 150, 150, 150),
        Color32::from_rgba_premultiplied(150, 150, 150, 150),
    )
}

pub fn dim() -> Color32 {
    pick(
        Color32::from_rgba_premultiplied(88, 88, 88, 88),
        Color32::from_rgba_premultiplied(96, 96, 96, 96),
    )
}

// ── Aksen & warna semantik ─────────────────────────────────────────────────
/// Biru sistem Apple — accent utama macOS Tahoe.
pub fn accent() -> Color32 {
    Color32::from_rgb(10, 132, 255)
}

pub fn accent_soft() -> Color32 {
    pick(
        Color32::from_rgba_premultiplied(2, 29, 56, 56),
        Color32::from_rgba_unmultiplied(10, 132, 255, 40),
    )
}

/// Kuning sistem — ⚠ peringatan.
pub fn yellow() -> Color32 {
    pick(
        Color32::from_rgb(255, 214, 10),
        Color32::from_rgb(178, 134, 0),
    )
}
/// Oranye sistem — notifikasi ringan.
pub fn orange() -> Color32 {
    pick(
        Color32::from_rgb(255, 149, 0),
        Color32::from_rgb(200, 110, 0),
    )
}
/// Merah sistem — ✕ error.
pub fn red() -> Color32 {
    pick(
        Color32::from_rgb(255, 59, 48),
        Color32::from_rgb(215, 45, 35),
    )
}
/// Biru — info / link.
pub fn blue() -> Color32 {
    Color32::from_rgb(10, 132, 255)
}

// ── Helper: glass highlight atas (seperti specular highlight kaca Tahoe) ───
pub fn glass_top() -> Color32 {
    Color32::from_rgba_premultiplied(26, 26, 26, 26)
}
/// Kapsul hitam pekat untuk Dynamic Island.
pub fn island() -> Color32 {
    Color32::from_rgb(10, 10, 12)
}

/// Latar bergradasi vertikal — dipanggil sekali di root window.
pub fn paint_background(ui: &egui::Ui) {
    let rect = ui.max_rect();
    let mut mesh = egui::Mesh::default();
    mesh.colored_vertex(rect.left_top(), bg_top());
    mesh.colored_vertex(rect.right_top(), bg_top());
    mesh.colored_vertex(rect.left_bottom(), bg_bottom());
    mesh.colored_vertex(rect.right_bottom(), bg_bottom());
    mesh.add_triangle(0, 1, 2);
    mesh.add_triangle(1, 3, 2);
    ui.painter().add(egui::Shape::mesh(mesh));
}

// ---------------------------------------------------------------------------
// apply — set egui visuals ke gaya macOS Tahoe
// ---------------------------------------------------------------------------
pub fn apply(ctx: &egui::Context) {
    let dark = is_dark();
    ctx.set_theme(if dark {
        egui::ThemePreference::Dark
    } else {
        egui::ThemePreference::Light
    });
    ctx.all_styles_mut(|style| {
        style.visuals = if dark {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        };
        style.visuals.dark_mode = dark;

        // Latar utama Tahoe
        style.visuals.panel_fill = bg();
        style.visuals.window_fill = pick(
            Color32::from_rgba_unmultiplied(44, 44, 48, 244),
            Color32::from_rgba_unmultiplied(250, 250, 252, 246),
        );
        style.visuals.extreme_bg_color = input();
        style.visuals.faint_bg_color = pick(
            Color32::from_rgba_unmultiplied(255, 255, 255, 10),
            Color32::from_rgba_unmultiplied(0, 0, 0, 6),
        );
        style.visuals.code_bg_color = pick(
            Color32::from_rgb(28, 28, 30),
            Color32::from_rgb(235, 235, 240),
        );

        // Jendela & popup — sudut lebih besar, shadow kuat (gaya Tahoe)
        style.visuals.window_stroke = Stroke::new(1.0, border());
        style.visuals.window_corner_radius = CornerRadius::same(20);
        style.visuals.menu_corner_radius = CornerRadius::same(14);
        style.visuals.window_shadow = egui::epaint::Shadow {
            offset: [0, 24],
            blur: 64,
            spread: 4,
            color: pick(
                Color32::from_black_alpha(200),
                Color32::from_black_alpha(70),
            ),
        };
        style.visuals.popup_shadow = egui::epaint::Shadow {
            offset: [0, 12],
            blur: 32,
            spread: 2,
            color: pick(
                Color32::from_black_alpha(160),
                Color32::from_black_alpha(55),
            ),
        };

        // Seleksi — biru Apple
        style.visuals.selection.bg_fill = accent();
        style.visuals.selection.stroke = Stroke::new(0.0, Color32::TRANSPARENT);
        style.visuals.hyperlink_color = accent();
        style.visuals.override_text_color = Some(text());

        // Widget: noninteractive (label, separator)
        style.visuals.widgets.noninteractive.bg_fill = pick(
            Color32::from_rgba_unmultiplied(255, 255, 255, 12),
            Color32::from_rgba_unmultiplied(0, 0, 0, 8),
        );
        style.visuals.widgets.noninteractive.weak_bg_fill = Color32::TRANSPARENT;
        style.visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, border_soft());
        style.visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, muted());
        style.visuals.widgets.noninteractive.corner_radius = CornerRadius::same(12);

        // Widget: inactive (tombol diam, input kosong)
        style.visuals.widgets.inactive.bg_fill = pick(
            Color32::from_rgba_unmultiplied(255, 255, 255, 16),
            Color32::from_rgba_unmultiplied(255, 255, 255, 245),
        );
        style.visuals.widgets.inactive.weak_bg_fill = pick(
            Color32::from_rgba_unmultiplied(255, 255, 255, 8),
            Color32::from_rgba_unmultiplied(0, 0, 0, 5),
        );
        style.visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, border());
        style.visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, text());
        style.visuals.widgets.inactive.corner_radius = CornerRadius::same(12);

        // Widget: hovered — glass lebih terang
        style.visuals.widgets.hovered.bg_fill = pick(
            Color32::from_rgba_unmultiplied(255, 255, 255, 30),
            Color32::WHITE,
        );
        style.visuals.widgets.hovered.weak_bg_fill = pick(
            Color32::from_rgba_unmultiplied(255, 255, 255, 20),
            Color32::WHITE,
        );
        style.visuals.widgets.hovered.bg_stroke = Stroke::new(
            1.0,
            pick(
                Color32::from_rgba_unmultiplied(255, 255, 255, 50),
                Color32::from_rgba_unmultiplied(0, 0, 0, 55),
            ),
        );
        style.visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, text());
        style.visuals.widgets.hovered.corner_radius = CornerRadius::same(12);
        style.visuals.widgets.hovered.expansion = 0.0;

        // Widget: active (sedang diklik) — biru Apple
        style.visuals.widgets.active.bg_fill = accent_soft();
        style.visuals.widgets.active.weak_bg_fill = accent_soft();
        style.visuals.widgets.active.bg_stroke = Stroke::new(1.5, accent());
        style.visuals.widgets.active.fg_stroke = Stroke::new(1.5, accent());
        style.visuals.widgets.active.corner_radius = CornerRadius::same(12);
        style.visuals.widgets.active.expansion = 0.0;

        // Widget: open (combobox terbuka)
        style.visuals.widgets.open.bg_fill = pick(
            Color32::from_rgba_unmultiplied(255, 255, 255, 24),
            Color32::WHITE,
        );
        style.visuals.widgets.open.weak_bg_fill = pick(
            Color32::from_rgba_unmultiplied(255, 255, 255, 14),
            Color32::from_rgba_unmultiplied(255, 255, 255, 245),
        );
        style.visuals.widgets.open.bg_stroke = Stroke::new(1.0, border());
        style.visuals.widgets.open.fg_stroke = Stroke::new(1.0, text());
        style.visuals.widgets.open.corner_radius = CornerRadius::same(12);

        // Scroll bar mengambang & tipis — seperti overlay scroller macOS
        style.spacing.scroll.floating = true;
        style.spacing.scroll.bar_width = 8.0;
        style.spacing.scroll.floating_width = 3.0;
        style.spacing.scroll.handle_min_length = 36.0;
        style.spacing.scroll.bar_inner_margin = 2.0;
        style.spacing.scroll.bar_outer_margin = 2.0;
        style.spacing.scroll.dormant_handle_opacity = 0.35;
        style.spacing.scroll.active_handle_opacity = 0.55;
        style.spacing.scroll.interact_handle_opacity = 0.8;

        // Spasi — sedikit lebih longgar dari Tahoe HIG
        style.spacing.item_spacing = Vec2::new(10.0, 9.0);
        style.spacing.button_padding = Vec2::new(16.0, 8.0);
        style.spacing.interact_size = Vec2::new(36.0, 36.0);
        style.spacing.slider_width = 160.0;
        style.spacing.combo_height = 360.0;
        style.spacing.combo_width = 190.0;
        style.spacing.indent = 20.0;
        style.spacing.icon_width = 16.0;
        style.spacing.icon_width_inner = 10.0;
        style.spacing.icon_spacing = 8.0;

        // Font — SF Pro Text / Display via font proporsional egui
        style
            .text_styles
            .insert(egui::TextStyle::Body, FontId::proportional(14.0));
        style
            .text_styles
            .insert(egui::TextStyle::Button, FontId::proportional(13.5));
        style
            .text_styles
            .insert(egui::TextStyle::Small, FontId::proportional(11.5));
        style
            .text_styles
            .insert(egui::TextStyle::Heading, FontId::proportional(22.0));
        style
            .text_styles
            .insert(egui::TextStyle::Monospace, FontId::monospace(13.0));
    });
}

// ---------------------------------------------------------------------------
// Komponen UI gaya Tahoe
// ---------------------------------------------------------------------------

/// Card "liquid glass" — seperti NSVisualEffectView .popover di dark mode
pub fn card() -> egui::Frame {
    egui::Frame::new()
        .fill(card_fill())
        .corner_radius(CornerRadius::same(20))
        .stroke(Stroke::new(1.0, border_soft()))
        .shadow(egui::epaint::Shadow {
            offset: [0, 5],
            blur: 16,
            spread: 0,
            color: pick(
                Color32::from_black_alpha(45),
                Color32::from_black_alpha(18),
            ),
        })
        .inner_margin(Margin::same(18))
}

/// Tombol utama — pill biru Apple dengan efek gloss
pub fn primary_button(ui: &mut egui::Ui, label: &str) -> egui::Response {
    ui.add(
        egui::Button::new(
            RichText::new(label)
                .size(13.5)
                .strong()
                .color(Color32::WHITE),
        )
        .fill(accent())
        .stroke(Stroke::new(
            1.0,
            Color32::from_rgba_unmultiplied(255, 255, 255, 60),
        ))
        .corner_radius(CornerRadius::same(12))
        .min_size(Vec2::new(0.0, 36.0)),
    )
}

/// Tombol sekunder — glass pill transparan
pub fn secondary_button(ui: &mut egui::Ui, label: &str) -> egui::Response {
    ui.add(
        egui::Button::new(RichText::new(label).size(13.5).color(text()))
            .fill(pick(
                Color32::from_rgba_unmultiplied(255, 255, 255, 18),
                Color32::from_rgba_unmultiplied(255, 255, 255, 245),
            ))
            .stroke(Stroke::new(1.0, border()))
            .corner_radius(CornerRadius::same(12))
            .min_size(Vec2::new(0.0, 36.0)),
    )
}

/// Tombol bahaya — merah sistem Apple
pub fn danger_button(ui: &mut egui::Ui, label: &str) -> egui::Response {
    ui.add(
        egui::Button::new(RichText::new(label).size(13.5).color(Color32::WHITE))
            .fill(Color32::from_rgba_unmultiplied(255, 59, 48, 180))
            .stroke(Stroke::new(
                1.0,
                Color32::from_rgba_unmultiplied(255, 59, 48, 120),
            ))
            .corner_radius(CornerRadius::same(12))
            .min_size(Vec2::new(0.0, 36.0)),
    )
}

/// Badge pill — seperti label kecil macOS
pub fn badge(ui: &mut egui::Ui, text: &str, color: Color32) {
    egui::Frame::new()
        .fill(Color32::from_rgba_unmultiplied(
            color.r(),
            color.g(),
            color.b(),
            38,
        ))
        .corner_radius(CornerRadius::same(20))
        .stroke(Stroke::new(
            1.0,
            Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 70),
        ))
        .inner_margin(Margin::symmetric(10, 4))
        .show(ui, |ui| {
            ui.label(RichText::new(text).size(11.0).color(color).strong());
        });
}

/// Label section kecil uppercase — seperti NSTableView group header macOS
pub fn section_label(ui: &mut egui::Ui, text: &str) {
    ui.label(
        RichText::new(text.to_uppercase())
            .size(10.5)
            .extra_letter_spacing(0.5)
            .color(dim())
            .strong(),
    );
}

/// Titik status berwarna
pub fn dot(ui: &mut egui::Ui, color: Color32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(8.0), egui::Sense::hover());
    ui.painter().circle_filled(rect.center(), 3.5, color);
}

/// Pemisah tipis transparan gaya Tahoe
pub fn separator(ui: &mut egui::Ui) {
    let width = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, 9.0), egui::Sense::hover());
    let y = rect.center().y;
    ui.painter().line_segment(
        [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
        Stroke::new(1.0, border_soft()),
    );
}

/// Segmented control — pill capsule gaya macOS Tahoe
pub fn segmented(ui: &mut egui::Ui, selected: &mut usize, labels: &[&str]) {
    // Container pill transparan
    egui::Frame::new()
        .fill(Color32::from_rgba_unmultiplied(0, 0, 0, 60))
        .corner_radius(CornerRadius::same(11))
        .inner_margin(Margin::same(3))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 2.0;
                for (i, label) in labels.iter().enumerate() {
                    let active = *selected == i;
                    let response = ui.add(
                        egui::Button::new(RichText::new(*label).size(12.5).color(if active {
                            text()
                        } else {
                            muted()
                        }))
                        .fill(if active {
                            pick(
                                Color32::from_rgba_unmultiplied(255, 255, 255, 36),
                                Color32::WHITE,
                            )
                        } else {
                            Color32::TRANSPARENT
                        })
                        .stroke(if active {
                            Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 40))
                        } else {
                            Stroke::NONE
                        })
                        .corner_radius(CornerRadius::same(9))
                        .min_size(Vec2::new(0.0, 28.0)),
                    );
                    if response.clicked() {
                        *selected = i;
                    }
                }
            });
        });
}

/// Segmented RTL — seperti `segmented` tapi untuk layout kanan-ke-kiri
pub fn segmented_rtl(ui: &mut egui::Ui, selected: &mut usize, labels: &[&str]) {
    egui::Frame::new()
        .fill(Color32::from_rgba_unmultiplied(0, 0, 0, 60))
        .corner_radius(CornerRadius::same(11))
        .inner_margin(Margin::same(3))
        .show(ui, |ui| {
            for (i, label) in labels.iter().enumerate().rev() {
                let active = *selected == i;
                let response = ui.add(
                    egui::Button::new(RichText::new(*label).size(12.5).color(if active {
                        text()
                    } else {
                        muted()
                    }))
                    .fill(if active {
                        Color32::from_rgba_unmultiplied(255, 255, 255, 36)
                    } else {
                        Color32::TRANSPARENT
                    })
                    .stroke(if active {
                        Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 40))
                    } else {
                        Stroke::NONE
                    })
                    .corner_radius(CornerRadius::same(9))
                    .min_size(Vec2::new(0.0, 28.0)),
                );
                if response.clicked() {
                    *selected = i;
                }
            }
        });
}

// ---------------------------------------------------------------------------
// Ikon garis sederhana (tanpa font ikon tambahan)
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Icon {
    Photo,
    Convert,
    Tools,
    Settings,
    Print,
    Download,
    Folder,
    Add,
    Trash,
    Check,
    Up,
    Down,
    Warning,
    Page,
    Close,
}

pub fn icon(painter: &egui::Painter, rect: egui::Rect, kind: Icon, color: Color32, width: f32) {
    let s = Stroke::new(width, color);
    let r = rect.shrink(rect.width() * 0.12);
    let c = r.center();
    let w = r.width();
    let h = r.height();
    let p = |x: f32, y: f32| egui::pos2(r.left() + x * w, r.top() + y * h);
    match kind {
        Icon::Photo => {
            painter.rect_stroke(
                egui::Rect::from_min_max(p(0.02, 0.08), p(0.98, 0.92)),
                CornerRadius::same(3),
                s,
                StrokeKind::Middle,
            );
            painter.circle_stroke(p(0.26, 0.3), w * 0.08, s);
            painter.line_segment([p(0.08, 0.86), p(0.42, 0.5)], s);
            painter.line_segment([p(0.42, 0.5), p(0.62, 0.7)], s);
            painter.line_segment([p(0.52, 0.62), p(0.72, 0.42)], s);
            painter.line_segment([p(0.72, 0.42), p(0.94, 0.74)], s);
        }
        Icon::Convert => {
            painter.line_segment([p(0.08, 0.3), p(0.9, 0.3)], s);
            painter.line_segment([p(0.9, 0.3), p(0.72, 0.14)], s);
            painter.line_segment([p(0.9, 0.3), p(0.72, 0.46)], s);
            painter.line_segment([p(0.92, 0.7), p(0.1, 0.7)], s);
            painter.line_segment([p(0.1, 0.7), p(0.28, 0.54)], s);
            painter.line_segment([p(0.1, 0.7), p(0.28, 0.86)], s);
        }
        Icon::Tools => {
            for (y, x) in [(0.25, 0.32), (0.5, 0.68), (0.75, 0.44)] {
                painter.line_segment([p(0.08, y), p(0.92, y)], s);
                painter.circle_filled(p(x, y), w * 0.09, color);
            }
        }
        Icon::Settings => {
            painter.circle_stroke(c, w * 0.24, s);
            for i in 0..8 {
                let a = i as f32 * std::f32::consts::TAU / 8.0;
                let inner = egui::vec2(a.cos(), a.sin()) * w * 0.34;
                let outer = egui::vec2(a.cos(), a.sin()) * w * 0.46;
                painter.line_segment([c + inner, c + outer], s);
            }
        }
        Icon::Print => {
            painter.rect_stroke(
                egui::Rect::from_min_max(p(0.12, 0.32), p(0.88, 0.78)),
                CornerRadius::same(3),
                s,
                StrokeKind::Middle,
            );
            painter.rect_stroke(
                egui::Rect::from_min_max(p(0.26, 0.08), p(0.74, 0.32)),
                CornerRadius::same(2),
                s,
                StrokeKind::Middle,
            );
            painter.rect_stroke(
                egui::Rect::from_min_max(p(0.26, 0.55), p(0.74, 0.94)),
                CornerRadius::same(2),
                s,
                StrokeKind::Middle,
            );
            painter.circle_filled(p(0.78, 0.44), w * 0.035, color);
        }
        Icon::Download => {
            painter.line_segment([p(0.5, 0.08), p(0.5, 0.62)], s);
            painter.line_segment([p(0.5, 0.62), p(0.28, 0.42)], s);
            painter.line_segment([p(0.5, 0.62), p(0.72, 0.42)], s);
            painter.line_segment([p(0.12, 0.78), p(0.12, 0.92)], s);
            painter.line_segment([p(0.12, 0.92), p(0.88, 0.92)], s);
            painter.line_segment([p(0.88, 0.92), p(0.88, 0.78)], s);
        }
        Icon::Folder => {
            painter.line_segment([p(0.06, 0.24), p(0.4, 0.24)], s);
            painter.line_segment([p(0.4, 0.24), p(0.5, 0.36)], s);
            painter.line_segment([p(0.5, 0.36), p(0.94, 0.36)], s);
            painter.line_segment([p(0.94, 0.36), p(0.94, 0.86)], s);
            painter.line_segment([p(0.94, 0.86), p(0.06, 0.86)], s);
            painter.line_segment([p(0.06, 0.86), p(0.06, 0.24)], s);
        }
        Icon::Add => {
            painter.line_segment([p(0.5, 0.12), p(0.5, 0.88)], s);
            painter.line_segment([p(0.12, 0.5), p(0.88, 0.5)], s);
        }
        Icon::Trash => {
            painter.line_segment([p(0.18, 0.24), p(0.82, 0.24)], s);
            painter.line_segment([p(0.38, 0.24), p(0.38, 0.12)], s);
            painter.line_segment([p(0.38, 0.12), p(0.62, 0.12)], s);
            painter.line_segment([p(0.62, 0.12), p(0.62, 0.24)], s);
            painter.line_segment([p(0.26, 0.24), p(0.3, 0.9)], s);
            painter.line_segment([p(0.74, 0.24), p(0.7, 0.9)], s);
            painter.line_segment([p(0.3, 0.9), p(0.7, 0.9)], s);
            painter.line_segment([p(0.44, 0.4), p(0.45, 0.76)], s);
            painter.line_segment([p(0.56, 0.4), p(0.55, 0.76)], s);
        }
        Icon::Check => {
            painter.line_segment([p(0.12, 0.52), p(0.4, 0.82)], s);
            painter.line_segment([p(0.4, 0.82), p(0.9, 0.2)], s);
        }
        Icon::Up => {
            painter.line_segment([p(0.5, 0.88), p(0.5, 0.14)], s);
            painter.line_segment([p(0.5, 0.14), p(0.22, 0.42)], s);
            painter.line_segment([p(0.5, 0.14), p(0.78, 0.42)], s);
        }
        Icon::Down => {
            painter.line_segment([p(0.5, 0.12), p(0.5, 0.86)], s);
            painter.line_segment([p(0.5, 0.86), p(0.22, 0.58)], s);
            painter.line_segment([p(0.5, 0.86), p(0.78, 0.58)], s);
        }
        Icon::Warning => {
            painter.line_segment([p(0.5, 0.08), p(0.94, 0.88)], s);
            painter.line_segment([p(0.94, 0.88), p(0.06, 0.88)], s);
            painter.line_segment([p(0.06, 0.88), p(0.5, 0.08)], s);
            painter.line_segment([p(0.5, 0.38), p(0.5, 0.64)], s);
            painter.circle_filled(p(0.5, 0.76), w * 0.035, color);
        }
        Icon::Page => {
            painter.rect_stroke(
                egui::Rect::from_min_max(p(0.18, 0.06), p(0.82, 0.94)),
                CornerRadius::same(2),
                s,
                StrokeKind::Middle,
            );
            painter.line_segment([p(0.32, 0.3), p(0.68, 0.3)], s);
            painter.line_segment([p(0.32, 0.48), p(0.68, 0.48)], s);
            painter.line_segment([p(0.32, 0.66), p(0.56, 0.66)], s);
        }
        Icon::Close => {
            painter.line_segment([p(0.18, 0.18), p(0.82, 0.82)], s);
            painter.line_segment([p(0.82, 0.18), p(0.18, 0.82)], s);
        }
    }
}

pub fn icon_button(ui: &mut egui::Ui, kind: Icon, tooltip: &str) -> egui::Response {
    let size = Vec2::splat(30.0);
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());
    let hovered = response.hovered();
    if hovered {
        ui.painter()
            .rect_filled(rect, CornerRadius::same(8), card_hover());
    }
    icon(
        ui.painter(),
        rect.shrink(7.0),
        kind,
        if hovered { text() } else { muted() },
        1.5,
    );
    response.on_hover_text(tooltip)
}
