use eframe::egui::{
    self, Color32, CornerRadius, FontId, Margin, RichText, Stroke, StrokeKind, Vec2,
};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicUsize, Ordering};

// ---------------------------------------------------------------------------
// Palet tema
// ---------------------------------------------------------------------------
//
// Aplikasi hanya menyediakan dua mode tampilan, keduanya bergaya macOS Tahoe:
//   • Tahoe Terang — latar putih/abu lembut, aksen biru sistem
//   • Tahoe Gelap  — latar arang pekat, aksen biru sistem (bawaan)
//
// Semua warna permukaan sengaja opak (bukan transparan) agar tidak ada
// artefak blending saat tema berganti.

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Flavor {
    /// Mode terang.
    #[serde(alias = "light", alias = "tahoe_light")]
    TahoeLight,
    /// Mode gelap (bawaan). Alias menjaga preferensi lama tetap terbaca.
    #[default]
    #[serde(
        alias = "dark",
        alias = "tahoe",
        alias = "latte",
        alias = "frappe",
        alias = "macchiato",
        alias = "mocha"
    )]
    TahoeDark,
}

impl Flavor {
    pub const ALL: [Flavor; 2] = [Flavor::TahoeLight, Flavor::TahoeDark];

    pub fn label(self) -> &'static str {
        match self {
            Flavor::TahoeLight => "Tahoe Terang",
            Flavor::TahoeDark => "Tahoe Gelap",
        }
    }

    pub fn short_label(self) -> &'static str {
        match self {
            Flavor::TahoeLight => "Terang",
            Flavor::TahoeDark => "Gelap",
        }
    }

    pub fn tagline(self) -> &'static str {
        match self {
            Flavor::TahoeLight => "Putih bersih",
            Flavor::TahoeDark => "Arang pekat",
        }
    }

    fn index(self) -> usize {
        match self {
            Flavor::TahoeLight => 0,
            Flavor::TahoeDark => 1,
        }
    }

    fn from_index(i: usize) -> Flavor {
        Self::ALL.get(i).copied().unwrap_or(Flavor::TahoeDark)
    }
}

/// Indeks `Flavor::TahoeDark` di dalam `Flavor::ALL`.
static FLAVOR: AtomicUsize = AtomicUsize::new(1);

pub fn flavor() -> Flavor {
    Flavor::from_index(FLAVOR.load(Ordering::Relaxed))
}

pub fn set_flavor(f: Flavor) {
    FLAVOR.store(f.index(), Ordering::Relaxed);
}

pub fn is_dark() -> bool {
    palette().dark
}

#[inline]
const fn rgb(hex: u32) -> Color32 {
    Color32::from_rgb(
        ((hex >> 16) & 0xff) as u8,
        ((hex >> 8) & 0xff) as u8,
        (hex & 0xff) as u8,
    )
}

#[inline]
fn alpha(color: Color32, a: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), a)
}

/// Warna transparan dalam format *premultiplied* — aman dipakai di `const`.
#[inline]
const fn tint(r: u8, g: u8, b: u8, a: u8) -> Color32 {
    Color32::from_rgba_premultiplied(
        ((r as u16 * a as u16) / 255) as u8,
        ((g as u16 * a as u16) / 255) as u8,
        ((b as u16 * a as u16) / 255) as u8,
        a,
    )
}

/// Seluruh warna yang dipakai UI. Dihitung ringan setiap pemanggilan.
#[derive(Clone, Copy)]
pub struct Palette {
    pub dark: bool,
    pub bg: Color32,
    pub bg_top: Color32,
    pub bg_bottom: Color32,
    pub glow: Color32,
    pub sidebar: Color32,
    pub card: Color32,
    pub card_hover: Color32,
    pub input: Color32,
    pub canvas: Color32,
    pub border: Color32,
    pub border_soft: Color32,
    pub shadow: Color32,
    pub text: Color32,
    pub muted: Color32,
    pub dim: Color32,
    pub accent: Color32,
    pub accent_soft: Color32,
    pub on_accent: Color32,
    pub yellow: Color32,
    pub orange: Color32,
    pub red: Color32,
    pub blue: Color32,
    pub green: Color32,
    pub nav_active_bg: Color32,
    pub nav_active_fg: Color32,
    pub island: Color32,
    pub island_border: Color32,
    pub island_text: Color32,
    pub island_muted: Color32,
    pub island_dim: Color32,
    pub island_field: Color32,
}

const TAHOE_DARK: Palette = Palette {
    dark: true,
    bg: rgb(0x1c1c1e),
    bg_top: rgb(0x242427),
    bg_bottom: rgb(0x141416),
    glow: rgb(0x0a84ff),
    sidebar: rgb(0x232326),
    card: rgb(0x2c2c2e),
    card_hover: rgb(0x3a3a3c),
    input: rgb(0x1a1a1c),
    canvas: rgb(0x141416),
    border: tint(255, 255, 255, 30),
    border_soft: tint(255, 255, 255, 16),
    shadow: Color32::from_black_alpha(110),
    text: rgb(0xf5f5f7),
    muted: rgb(0xaeaeb2),
    dim: rgb(0x8e8e93),
    accent: rgb(0x0a84ff),
    accent_soft: tint(10, 132, 255, 46),
    on_accent: rgb(0xffffff),
    yellow: rgb(0xffd60a),
    orange: rgb(0xff9f0a),
    red: rgb(0xff453a),
    blue: rgb(0x0a84ff),
    green: rgb(0x30d158),
    nav_active_bg: rgb(0x0a84ff),
    nav_active_fg: rgb(0xffffff),
    island: rgb(0x0b0b0d),
    island_border: tint(255, 255, 255, 36),
    island_text: rgb(0xf5f5f7),
    island_muted: rgb(0xaeaeb2),
    island_dim: rgb(0x8e8e93),
    island_field: rgb(0x2c2c2e),
};

// Mode terang dibuat "kertas lembut", bukan putih murni: permukaan kartu dan
// panel ditahan di kisaran abu hangat agar tidak menyilaukan saat dipakai lama.
const TAHOE_LIGHT: Palette = Palette {
    dark: false,
    bg: rgb(0xe8e8ec),
    bg_top: rgb(0xefeff2),
    bg_bottom: rgb(0xdcdce2),
    glow: rgb(0x007aff),
    sidebar: rgb(0xf3f3f6),
    card: rgb(0xf7f7fa),
    card_hover: rgb(0xeaeaf0),
    input: rgb(0xe9e9ee),
    canvas: rgb(0xd7d7de),
    border: tint(0, 0, 0, 34),
    border_soft: tint(0, 0, 0, 20),
    shadow: Color32::from_black_alpha(36),
    text: rgb(0x1c1c1e),
    muted: rgb(0x5f5f66),
    dim: rgb(0x77777e),
    accent: rgb(0x007aff),
    accent_soft: tint(0, 122, 255, 30),
    on_accent: rgb(0xffffff),
    yellow: rgb(0xd9a400),
    orange: rgb(0xf08c00),
    red: rgb(0xff3b30),
    blue: rgb(0x007aff),
    green: rgb(0x2a9d4a),
    nav_active_bg: rgb(0x007aff),
    nav_active_fg: rgb(0xffffff),
    island: rgb(0xf5f5f8),
    island_border: tint(0, 0, 0, 28),
    island_text: rgb(0x1c1c1e),
    island_muted: rgb(0x5f5f66),
    island_dim: rgb(0x77777e),
    island_field: rgb(0xe9e9ee),
};

fn palette_for(f: Flavor) -> Palette {
    match f {
        Flavor::TahoeLight => TAHOE_LIGHT,
        Flavor::TahoeDark => TAHOE_DARK,
    }
}

#[inline]
pub fn palette() -> Palette {
    palette_for(flavor())
}

/// Tiga warna contoh untuk chip pemilih tema.
pub fn swatches(f: Flavor) -> [Color32; 3] {
    let p = palette_for(f);
    [p.bg_top, p.card_hover, p.accent]
}

// ── Latar & panel ──────────────────────────────────────────────────────────
pub fn bg() -> Color32 {
    palette().bg
}
pub fn bg_top() -> Color32 {
    palette().bg_top
}
pub fn bg_bottom() -> Color32 {
    palette().bg_bottom
}

/// Sidebar: panel melayang bernuansa sesuai tema.
pub fn sidebar() -> Color32 {
    palette().sidebar
}

/// Fill kartu.
pub fn card_fill() -> Color32 {
    palette().card
}

pub fn card_hover() -> Color32 {
    palette().card_hover
}

/// Input field.
pub fn input() -> Color32 {
    palette().input
}

/// Latar meja kerja pratinjau.
pub fn canvas() -> Color32 {
    palette().canvas
}

// ── Garis & pemisah ────────────────────────────────────────────────────────
pub fn border() -> Color32 {
    palette().border
}

pub fn border_soft() -> Color32 {
    palette().border_soft
}

pub fn shadow() -> Color32 {
    palette().shadow
}

// ── Teks ────────────────────────────────────────────────────────────────────
pub fn text() -> Color32 {
    palette().text
}

pub fn muted() -> Color32 {
    palette().muted
}

pub fn dim() -> Color32 {
    palette().dim
}

// ── Aksen & warna semantik ─────────────────────────────────────────────────
pub fn accent() -> Color32 {
    palette().accent
}

pub fn accent_soft() -> Color32 {
    palette().accent_soft
}

pub fn on_accent() -> Color32 {
    palette().on_accent
}

pub fn yellow() -> Color32 {
    palette().yellow
}
pub fn orange() -> Color32 {
    palette().orange
}
pub fn red() -> Color32 {
    palette().red
}
pub fn blue() -> Color32 {
    palette().blue
}
pub fn green() -> Color32 {
    palette().green
}

// ── Navigasi ───────────────────────────────────────────────────────────────
pub fn nav_active_bg() -> Color32 {
    palette().nav_active_bg
}
pub fn nav_active_fg() -> Color32 {
    palette().nav_active_fg
}

// ── Dynamic Island ────────────────────────────────────────────────────────
pub fn island() -> Color32 {
    palette().island
}
pub fn island_border() -> Color32 {
    palette().island_border
}
pub fn island_text() -> Color32 {
    palette().island_text
}
pub fn island_muted() -> Color32 {
    palette().island_muted
}
pub fn island_dim() -> Color32 {
    palette().island_dim
}
pub fn island_accent() -> Color32 {
    palette().accent
}
pub fn island_field() -> Color32 {
    palette().island_field
}

/// Latar bergradasi vertikal + glow dekoratif di sudut.
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

    // Glow lembut agar latar tidak terasa datar.
    let accent = palette().glow;
    let strength = if is_dark() { 26 } else { 10 };
    soft_glow(
        ui.painter(),
        egui::pos2(rect.right() - 80.0, rect.top() + 60.0),
        300.0,
        alpha(accent, strength),
    );
    soft_glow(
        ui.painter(),
        egui::pos2(rect.left() + 40.0, rect.bottom() - 40.0),
        260.0,
        alpha(accent, strength / 2),
    );
}

/// Lingkaran bercahaya lembut dengan gradasi radial (mesh, tanpa garis).
fn soft_glow(painter: &egui::Painter, center: egui::Pos2, radius: f32, color: Color32) {
    if radius <= 0.0 {
        return;
    }
    let segments = 48u32;
    let mut mesh = egui::Mesh::default();
    mesh.colored_vertex(center, color);
    for i in 0..=segments {
        let a = i as f32 * std::f32::consts::TAU / segments as f32;
        let pos = center + egui::vec2(a.cos(), a.sin()) * radius;
        mesh.colored_vertex(pos, Color32::TRANSPARENT);
    }
    for i in 0..segments {
        mesh.add_triangle(0, i + 1, i + 2);
    }
    painter.add(egui::Shape::mesh(mesh));
}

// ---------------------------------------------------------------------------
// apply — set egui visuals mengikuti palet aktif
// ---------------------------------------------------------------------------
pub fn apply(ctx: &egui::Context) {
    let p = palette();
    ctx.set_theme(if p.dark {
        egui::ThemePreference::Dark
    } else {
        egui::ThemePreference::Light
    });
    ctx.all_styles_mut(|style| {
        style.visuals = if p.dark {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        };
        style.visuals.dark_mode = p.dark;

        style.visuals.panel_fill = p.bg;
        style.visuals.window_fill = p.card;
        style.visuals.extreme_bg_color = p.input;
        style.visuals.faint_bg_color = if p.dark {
            Color32::from_rgba_unmultiplied(255, 255, 255, 10)
        } else {
            Color32::from_rgba_unmultiplied(0, 0, 0, 6)
        };
        style.visuals.code_bg_color = p.input;

        // Jendela & popup — sudut besar, bayangan lembut.
        style.visuals.window_stroke = Stroke::new(1.0, p.border);
        style.visuals.window_corner_radius = CornerRadius::same(20);
        style.visuals.menu_corner_radius = CornerRadius::same(14);
        style.visuals.window_shadow = egui::epaint::Shadow {
            offset: [0, 22],
            blur: 56,
            spread: 4,
            color: if p.dark {
                Color32::from_black_alpha(170)
            } else {
                Color32::from_black_alpha(48)
            },
        };
        style.visuals.popup_shadow = egui::epaint::Shadow {
            offset: [0, 12],
            blur: 30,
            spread: 2,
            color: if p.dark {
                Color32::from_black_alpha(150)
            } else {
                Color32::from_black_alpha(40)
            },
        };

        // Seleksi — aksen tema.
        style.visuals.selection.bg_fill = p.accent;
        style.visuals.selection.stroke = Stroke::new(0.0, p.on_accent);
        style.visuals.hyperlink_color = p.accent;
        // Catatan: `override_text_color` sengaja tidak dipakai agar teks
        // nonaktif tetap terlihat redup dan status disabled tidak hilang.

        // Widget: noninteractive (label, separator)
        style.visuals.widgets.noninteractive.bg_fill = if p.dark {
            Color32::from_rgba_unmultiplied(255, 255, 255, 12)
        } else {
            Color32::from_rgba_unmultiplied(0, 0, 0, 6)
        };
        style.visuals.widgets.noninteractive.weak_bg_fill = Color32::TRANSPARENT;
        style.visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, p.border_soft);
        style.visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, p.muted);
        style.visuals.widgets.noninteractive.corner_radius = CornerRadius::same(10);

        // Widget: inactive (tombol diam, input kosong)
        style.visuals.widgets.inactive.bg_fill = if p.dark {
            Color32::from_rgba_unmultiplied(255, 255, 255, 14)
        } else {
            p.card
        };
        style.visuals.widgets.inactive.weak_bg_fill = if p.dark {
            Color32::from_rgba_unmultiplied(255, 255, 255, 8)
        } else {
            Color32::from_rgba_unmultiplied(0, 0, 0, 4)
        };
        style.visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, p.border);
        style.visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, p.text);
        style.visuals.widgets.inactive.corner_radius = CornerRadius::same(10);

        // Widget: hovered
        style.visuals.widgets.hovered.bg_fill = p.card_hover;
        style.visuals.widgets.hovered.weak_bg_fill = style.visuals.widgets.hovered.bg_fill;
        style.visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, p.border);
        style.visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, p.text);
        style.visuals.widgets.hovered.corner_radius = CornerRadius::same(10);
        style.visuals.widgets.hovered.expansion = 0.0;

        // Widget: active (sedang diklik)
        style.visuals.widgets.active.bg_fill = p.accent_soft;
        style.visuals.widgets.active.weak_bg_fill = p.accent_soft;
        style.visuals.widgets.active.bg_stroke = Stroke::new(1.2, p.accent);
        style.visuals.widgets.active.fg_stroke = Stroke::new(1.2, p.accent);
        style.visuals.widgets.active.corner_radius = CornerRadius::same(10);
        style.visuals.widgets.active.expansion = 0.0;

        // Widget: open (combobox terbuka)
        style.visuals.widgets.open.bg_fill = p.card;
        style.visuals.widgets.open.weak_bg_fill = p.card_hover;
        style.visuals.widgets.open.bg_stroke = Stroke::new(1.0, p.border);
        style.visuals.widgets.open.fg_stroke = Stroke::new(1.0, p.text);
        style.visuals.widgets.open.corner_radius = CornerRadius::same(10);

        // Scrollbar mengambang & tipis.
        style.spacing.scroll.floating = true;
        style.spacing.scroll.bar_width = 8.0;
        style.spacing.scroll.floating_width = 3.0;
        style.spacing.scroll.handle_min_length = 36.0;
        style.spacing.scroll.bar_inner_margin = 2.0;
        style.spacing.scroll.bar_outer_margin = 2.0;
        style.spacing.scroll.dormant_handle_opacity = 0.35;
        style.spacing.scroll.active_handle_opacity = 0.55;
        style.spacing.scroll.interact_handle_opacity = 0.8;

        // Spasi
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

        // Font
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
// Komponen UI
// ---------------------------------------------------------------------------

/// Kartu utama: permukaan terang, sudut membulat, bayangan lembut.
pub fn card() -> egui::Frame {
    egui::Frame::new()
        .fill(card_fill())
        .corner_radius(CornerRadius::same(20))
        .stroke(Stroke::new(1.0, border_soft()))
        .shadow(egui::epaint::Shadow {
            offset: [0, 8],
            blur: 24,
            spread: 0,
            color: shadow(),
        })
        .inner_margin(Margin::same(18))
}

/// Tombol utama — pill aksen tema.
pub fn primary_button(ui: &mut egui::Ui, label: &str) -> egui::Response {
    ui.add(
        egui::Button::new(RichText::new(label).size(13.0).strong().color(on_accent()))
            .fill(accent())
            .stroke(Stroke::new(1.0, alpha(accent(), 160)))
            .corner_radius(CornerRadius::same(10))
            .min_size(Vec2::new(0.0, 34.0)),
    )
}

/// Tombol sekunder — permukaan kartu dengan garis tepi.
pub fn secondary_button(ui: &mut egui::Ui, label: &str) -> egui::Response {
    ui.add(
        egui::Button::new(RichText::new(label).size(13.0).color(text()))
            .fill(card_fill())
            .stroke(Stroke::new(1.0, border()))
            .corner_radius(CornerRadius::same(10))
            .min_size(Vec2::new(0.0, 34.0)),
    )
}

/// Tombol bahaya.
pub fn danger_button(ui: &mut egui::Ui, label: &str) -> egui::Response {
    ui.add(
        egui::Button::new(RichText::new(label).size(13.0).color(Color32::WHITE))
            .fill(alpha(red(), 205))
            .stroke(Stroke::new(1.0, alpha(red(), 150)))
            .corner_radius(CornerRadius::same(10))
            .min_size(Vec2::new(0.0, 34.0)),
    )
}

/// Badge pill kecil.
pub fn badge(ui: &mut egui::Ui, text: &str, color: Color32) {
    egui::Frame::new()
        .fill(alpha(color, 26))
        .corner_radius(CornerRadius::same(20))
        .stroke(Stroke::new(1.0, alpha(color, 60)))
        .inner_margin(Margin::symmetric(9, 3))
        .show(ui, |ui| {
            ui.label(RichText::new(text).size(10.5).color(color).strong());
        });
}

/// Label section kecil uppercase.
pub fn section_label(ui: &mut egui::Ui, text: &str) {
    ui.label(
        RichText::new(text.to_uppercase())
            .size(10.0)
            .extra_letter_spacing(0.7)
            .color(dim())
            .strong(),
    );
}

/// Titik status berwarna.
pub fn dot(ui: &mut egui::Ui, color: Color32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(8.0), egui::Sense::hover());
    ui.painter().circle_filled(rect.center(), 3.5, color);
}

/// Pemisah tipis.
pub fn separator(ui: &mut egui::Ui) {
    let width = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, 9.0), egui::Sense::hover());
    let y = rect.center().y;
    ui.painter().line_segment(
        [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
        Stroke::new(1.0, border_soft()),
    );
}

/// Segmented control — kapsul dengan indikator aktif.
pub fn segmented(ui: &mut egui::Ui, selected: &mut usize, labels: &[&str]) {
    egui::Frame::new()
        .fill(input())
        .corner_radius(CornerRadius::same(10))
        .stroke(Stroke::new(1.0, border_soft()))
        .inner_margin(Margin::same(3))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 2.0;
                for (i, label) in labels.iter().enumerate() {
                    let active = *selected == i;
                    let response = ui.add(
                        egui::Button::new(RichText::new(*label).size(12.0).color(if active {
                            text()
                        } else {
                            muted()
                        }))
                        .fill(if active {
                            card_fill()
                        } else {
                            Color32::TRANSPARENT
                        })
                        .stroke(if active {
                            Stroke::new(1.0, border_soft())
                        } else {
                            Stroke::NONE
                        })
                        .corner_radius(CornerRadius::same(8))
                        .min_size(Vec2::new(0.0, 26.0)),
                    );
                    if response.clicked() {
                        *selected = i;
                    }
                }
            });
        });
}

/// Segmented RTL — seperti `segmented` tapi untuk layout kanan-ke-kiri.
pub fn segmented_rtl(ui: &mut egui::Ui, selected: &mut usize, labels: &[&str]) {
    egui::Frame::new()
        .fill(input())
        .corner_radius(CornerRadius::same(10))
        .stroke(Stroke::new(1.0, border_soft()))
        .inner_margin(Margin::same(3))
        .show(ui, |ui| {
            for (i, label) in labels.iter().enumerate().rev() {
                let active = *selected == i;
                let response = ui.add(
                    egui::Button::new(RichText::new(*label).size(12.0).color(if active {
                        text()
                    } else {
                        muted()
                    }))
                    .fill(if active {
                        card_fill()
                    } else {
                        Color32::TRANSPARENT
                    })
                    .stroke(if active {
                        Stroke::new(1.0, border_soft())
                    } else {
                        Stroke::NONE
                    })
                    .corner_radius(CornerRadius::same(8))
                    .min_size(Vec2::new(0.0, 26.0)),
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
    Scan,
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
    Stack,
    ChevronDown,
    ChevronRight,
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
        Icon::Scan => {
            // Sudut pemindai + garis pindai.
            for (x, y, dx, dy) in [
                (0.12, 0.12, 1.0, 1.0),
                (0.88, 0.12, -1.0, 1.0),
                (0.12, 0.88, 1.0, -1.0),
                (0.88, 0.88, -1.0, -1.0),
            ] {
                painter.line_segment([p(x, y), p(x + dx * 0.22, y)], s);
                painter.line_segment([p(x, y), p(x, y + dy * 0.22)], s);
            }
            painter.line_segment([p(0.14, 0.56), p(0.86, 0.56)], s);
            painter.line_segment([p(0.28, 0.72), p(0.72, 0.72)], s);
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
        Icon::Stack => {
            // Tiga lembar bertumpuk — simbol combine / kumpulan file.
            for (dx, dy) in [(0.0, 0.0), (0.12, 0.12), (0.24, 0.24)] {
                painter.rect_stroke(
                    egui::Rect::from_min_max(p(0.04 + dx, 0.04 + dy), p(0.66 + dx, 0.66 + dy)),
                    CornerRadius::same(2),
                    s,
                    StrokeKind::Middle,
                );
            }
        }
        Icon::ChevronDown => {
            painter.line_segment([p(0.2, 0.36), p(0.5, 0.68)], s);
            painter.line_segment([p(0.5, 0.68), p(0.8, 0.36)], s);
        }
        Icon::ChevronRight => {
            painter.line_segment([p(0.36, 0.2), p(0.68, 0.5)], s);
            painter.line_segment([p(0.68, 0.5), p(0.36, 0.8)], s);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_flavor_is_dark_tahoe() {
        assert_eq!(Flavor::default(), Flavor::TahoeDark);
    }

    #[test]
    fn legacy_theme_prefs_load_as_dark_tahoe() {
        for old in ["latte", "frappe", "macchiato", "mocha", "tahoe", "dark"] {
            let raw = format!("\"{old}\"");
            let flavor: Flavor = serde_json::from_str(&raw).unwrap();
            assert_eq!(flavor, Flavor::TahoeDark, "pref lama {old}");
        }
    }

    #[test]
    fn light_theme_pref_roundtrips() {
        let raw = serde_json::to_string(&Flavor::TahoeLight).unwrap();
        assert_eq!(raw, "\"tahoe_light\"");
        let flavor: Flavor = serde_json::from_str(&raw).unwrap();
        assert_eq!(flavor, Flavor::TahoeLight);
    }

    #[test]
    fn palette_indices_stay_in_sync() {
        for (i, flavor) in Flavor::ALL.iter().copied().enumerate() {
            assert_eq!(flavor.index(), i);
            assert_eq!(Flavor::from_index(i), flavor);
        }
        assert_eq!(Flavor::from_index(usize::MAX), Flavor::TahoeDark);
    }

    #[test]
    fn palettes_are_opaque_and_contrasting() {
        for flavor in Flavor::ALL {
            let p = palette_for(flavor);
            for (name, color) in [
                ("bg", p.bg),
                ("sidebar", p.sidebar),
                ("card", p.card),
                ("input", p.input),
                ("canvas", p.canvas),
                ("text", p.text),
                ("muted", p.muted),
                ("hero_text", p.hero_text),
                ("island", p.island),
                ("island_text", p.island_text),
            ] {
                assert_eq!(color.a(), 255, "{name} harus opak pada {flavor:?}");
            }
            assert_eq!(p.dark, flavor == Flavor::TahoeDark);
        }
    }
}
