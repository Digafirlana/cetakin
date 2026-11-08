use eframe::egui::{self, Color32, CornerRadius, FontId, Margin, RichText, Stroke, StrokeKind, Vec2};

pub const BG: Color32 = Color32::from_rgb(15, 16, 19);
pub const SIDEBAR: Color32 = Color32::from_rgb(21, 22, 26);
pub const CARD: Color32 = Color32::from_rgb(28, 30, 35);
pub const CARD_HOVER: Color32 = Color32::from_rgb(35, 38, 44);
pub const INPUT: Color32 = Color32::from_rgb(24, 26, 30);
pub const BORDER: Color32 = Color32::from_rgb(43, 46, 53);
pub const BORDER_SOFT: Color32 = Color32::from_rgb(35, 38, 44);
pub const TEXT: Color32 = Color32::from_rgb(242, 244, 247);
pub const MUTED: Color32 = Color32::from_rgb(139, 145, 158);
pub const DIM: Color32 = Color32::from_rgb(100, 106, 118);
pub const ACCENT: Color32 = Color32::from_rgb(47, 227, 111);
pub const ACCENT_SOFT: Color32 = Color32::from_rgb(25, 61, 39);
pub const YELLOW: Color32 = Color32::from_rgb(233, 211, 79);
pub const ORANGE: Color32 = Color32::from_rgb(243, 169, 74);
pub const RED: Color32 = Color32::from_rgb(240, 113, 120);
pub const BLUE: Color32 = Color32::from_rgb(100, 168, 255);

pub fn apply(ctx: &egui::Context) {
    ctx.set_theme(egui::ThemePreference::Dark);
    ctx.all_styles_mut(|style| {
        style.visuals = egui::Visuals::dark();
        style.visuals.dark_mode = true;
        style.visuals.panel_fill = BG;
        style.visuals.window_fill = CARD;
        style.visuals.extreme_bg_color = INPUT;
        style.visuals.faint_bg_color = CARD;
        style.visuals.code_bg_color = INPUT;
        style.visuals.window_stroke = Stroke::new(1.0, BORDER);
        style.visuals.window_corner_radius = CornerRadius::same(16);
        style.visuals.menu_corner_radius = CornerRadius::same(12);
        style.visuals.window_shadow = egui::epaint::Shadow {
            offset: [0, 16],
            blur: 40,
            spread: 0,
            color: Color32::from_black_alpha(160),
        };
        style.visuals.popup_shadow = egui::epaint::Shadow {
            offset: [0, 8],
            blur: 24,
            spread: 0,
            color: Color32::from_black_alpha(140),
        };
        style.visuals.selection.bg_fill = ACCENT;
        style.visuals.selection.stroke = Stroke::new(1.0, Color32::BLACK);
        style.visuals.hyperlink_color = ACCENT;
        style.visuals.override_text_color = Some(TEXT);

        style.visuals.widgets.noninteractive.bg_fill = CARD;
        style.visuals.widgets.noninteractive.weak_bg_fill = CARD;
        style.visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, BORDER);
        style.visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, MUTED);
        style.visuals.widgets.noninteractive.corner_radius = CornerRadius::same(10);

        style.visuals.widgets.inactive.bg_fill = INPUT;
        style.visuals.widgets.inactive.weak_bg_fill = INPUT;
        style.visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, BORDER);
        style.visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, TEXT);
        style.visuals.widgets.inactive.corner_radius = CornerRadius::same(10);

        style.visuals.widgets.hovered.bg_fill = CARD_HOVER;
        style.visuals.widgets.hovered.weak_bg_fill = CARD_HOVER;
        style.visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, Color32::from_rgb(72, 77, 88));
        style.visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, TEXT);
        style.visuals.widgets.hovered.corner_radius = CornerRadius::same(10);
        style.visuals.widgets.hovered.expansion = 0.0;

        style.visuals.widgets.active.bg_fill = ACCENT_SOFT;
        style.visuals.widgets.active.weak_bg_fill = ACCENT_SOFT;
        style.visuals.widgets.active.bg_stroke = Stroke::new(1.0, ACCENT);
        style.visuals.widgets.active.fg_stroke = Stroke::new(1.0, ACCENT);
        style.visuals.widgets.active.corner_radius = CornerRadius::same(10);
        style.visuals.widgets.active.expansion = 0.0;

        style.visuals.widgets.open.bg_fill = CARD_HOVER;
        style.visuals.widgets.open.weak_bg_fill = CARD_HOVER;
        style.visuals.widgets.open.bg_stroke = Stroke::new(1.0, BORDER);
        style.visuals.widgets.open.fg_stroke = Stroke::new(1.0, TEXT);
        style.visuals.widgets.open.corner_radius = CornerRadius::same(10);

        style.spacing.item_spacing = Vec2::new(10.0, 9.0);
        style.spacing.button_padding = Vec2::new(14.0, 8.0);
        style.spacing.interact_size = Vec2::new(34.0, 34.0);
        style.spacing.slider_width = 150.0;
        style.spacing.combo_height = 320.0;
        style.spacing.combo_width = 180.0;
        style.spacing.indent = 18.0;
        style.spacing.icon_width = 16.0;
        style.spacing.icon_width_inner = 10.0;
        style.spacing.icon_spacing = 7.0;

        style.text_styles.insert(
            egui::TextStyle::Body,
            FontId::proportional(14.0),
        );
        style.text_styles.insert(
            egui::TextStyle::Button,
            FontId::proportional(14.0),
        );
        style.text_styles.insert(
            egui::TextStyle::Small,
            FontId::proportional(12.0),
        );
        style.text_styles.insert(
            egui::TextStyle::Heading,
            FontId::proportional(24.0),
        );
        style.text_styles.insert(
            egui::TextStyle::Monospace,
            FontId::monospace(13.0),
        );
    });
}

pub fn card() -> egui::Frame {
    egui::Frame::new()
        .fill(CARD)
        .corner_radius(CornerRadius::same(16))
        .stroke(Stroke::new(1.0, BORDER_SOFT))
        .inner_margin(Margin::same(18))
}

pub fn primary_button(ui: &mut egui::Ui, label: &str) -> egui::Response {
    ui.add(
        egui::Button::new(
            RichText::new(label)
                .size(14.0)
                .strong()
                .color(Color32::from_rgb(8, 20, 12)),
        )
        .fill(ACCENT)
        .stroke(Stroke::NONE)
        .corner_radius(CornerRadius::same(11))
        .min_size(Vec2::new(0.0, 38.0)),
    )
}

pub fn secondary_button(ui: &mut egui::Ui, label: &str) -> egui::Response {
    ui.add(
        egui::Button::new(RichText::new(label).size(14.0).color(TEXT))
            .fill(INPUT)
            .stroke(Stroke::new(1.0, BORDER))
            .corner_radius(CornerRadius::same(11))
            .min_size(Vec2::new(0.0, 38.0)),
    )
}

pub fn danger_button(ui: &mut egui::Ui, label: &str) -> egui::Response {
    ui.add(
        egui::Button::new(RichText::new(label).size(13.0).color(RED))
            .fill(Color32::from_rgb(45, 27, 29))
            .stroke(Stroke::new(1.0, Color32::from_rgb(84, 42, 47)))
            .corner_radius(CornerRadius::same(10))
            .min_size(Vec2::new(0.0, 34.0)),
    )
}

pub fn badge(ui: &mut egui::Ui, text: &str, color: Color32) {
    egui::Frame::new()
        .fill(Color32::from_rgba_unmultiplied(
            color.r(),
            color.g(),
            color.b(),
            28,
        ))
        .corner_radius(CornerRadius::same(20))
        .inner_margin(Margin::symmetric(9, 4))
        .show(ui, |ui| {
            ui.label(RichText::new(text).size(11.5).color(color).strong());
        });
}

pub fn section_label(ui: &mut egui::Ui, text: &str) {
    ui.label(
        RichText::new(text.to_uppercase())
            .size(10.5)
            .color(DIM)
            .strong(),
    );
}

pub fn dot(ui: &mut egui::Ui, color: Color32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(8.0), egui::Sense::hover());
    ui.painter().circle_filled(rect.center(), 3.5, color);
}

pub fn separator(ui: &mut egui::Ui) {
    let width = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, 9.0), egui::Sense::hover());
    let y = rect.center().y;
    ui.painter().line_segment(
        [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
        Stroke::new(1.0, BORDER_SOFT),
    );
}

pub fn segmented(ui: &mut egui::Ui, selected: &mut usize, labels: &[&str]) {
    ui.horizontal(|ui| {
        for (i, label) in labels.iter().enumerate() {
            let active = *selected == i;
            let response = ui.add(
                egui::Button::new(
                    RichText::new(*label)
                        .size(13.0)
                        .color(if active { TEXT } else { MUTED }),
                )
                .fill(if active { CARD_HOVER } else { INPUT })
                .stroke(if active {
                    Stroke::new(1.0, Color32::from_rgb(74, 79, 90))
                } else {
                    Stroke::new(1.0, BORDER_SOFT)
                })
                .corner_radius(CornerRadius::same(9))
                .min_size(Vec2::new(0.0, 30.0)),
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
            .rect_filled(rect, CornerRadius::same(8), CARD_HOVER);
    }
    icon(
        ui.painter(),
        rect.shrink(7.0),
        kind,
        if hovered { TEXT } else { MUTED },
        1.5,
    );
    response.on_hover_text(tooltip)
}
