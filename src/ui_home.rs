// Halaman beranda (Home) — desain dashboard modern.

use eframe::egui::{
    self, Align2, Color32, CornerRadius, CursorIcon, FontId, Margin, RichText, ScrollArea, Sense,
    Stroke, StrokeKind, Vec2,
};

use crate::app::{CetakinApp, Tab};
use crate::theme;
use crate::ui_common::dashed_rect;

// ---------------------------------------------------------------------------
// Aksi yang dipanggil dari UI
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq)]
enum HomeAction {
    OpenSheet,
    OpenScan,
    OpenConvert,
    OpenTools,
    OpenHistory,
    ImportPhotos,
    ScanSingle,
    ScanMulti,
    ConvertFiles,
}

/// Ikon berlapis khusus tile Pintasan Cepat.
#[derive(Clone, Copy)]
enum ShortcutIcon {
    ImportPhoto,
    ScanOne,
    ScanMany,
    Convert,
    Print,
    SavePdf,
    Tools,
}

// ---------------------------------------------------------------------------
// Implementasi
// ---------------------------------------------------------------------------

impl CetakinApp {
    pub fn home_tab(&mut self, ui: &mut egui::Ui) {
        let mut action: Option<HomeAction> = None;

        ScrollArea::vertical()
            .id_salt("home-scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                // --- Drop Zone + Quick Actions ---
                self.home_drop_zone(ui, &mut action);
                ui.add_space(14.0);

                // --- Grid alat ---
                self.home_tools_grid(ui, &mut action);
                ui.add_space(14.0);

                // --- File terbaru ---
                self.home_recent(ui, &mut action);
                ui.add_space(8.0);
            });

        // Eksekusi aksi
        match action {
            Some(HomeAction::OpenSheet) => self.tab = Tab::Sheet,
            Some(HomeAction::OpenScan) => self.tab = Tab::Scan,
            Some(HomeAction::OpenConvert) => self.tab = Tab::Convert,
            Some(HomeAction::OpenTools) => self.tab = Tab::Tools,
            Some(HomeAction::OpenHistory) => self.tab = Tab::History,
            Some(HomeAction::ImportPhotos) => {
                self.import_photos_dialog();
                self.tab = Tab::Sheet;
            }
            Some(HomeAction::ScanSingle) => {
                self.tab = Tab::Scan;
                self.pick_scan_source_single(ui.ctx());
            }
            Some(HomeAction::ScanMulti) => {
                self.tab = Tab::Scan;
                self.pick_scan_source(ui.ctx());
            }
            Some(HomeAction::ConvertFiles) => {
                self.tab = Tab::Convert;
                self.convert.pick_files_dialog();
            }
            None => {}
        }
    }

    // ------------------------------------------------------------------ drop zone

    fn home_drop_zone(&mut self, ui: &mut egui::Ui, action: &mut Option<HomeAction>) {
        theme::card().inner_margin(Margin::same(0)).show(ui, |ui| {
            // ---- Zona drag & drop ----
            let dz_height = 180.0;
            let (dz_rect, dz_resp) =
                ui.allocate_exact_size(Vec2::new(ui.available_width(), dz_height), Sense::click());

            let painter = ui.painter_at(dz_rect);
            painter.rect_filled(dz_rect, CornerRadius::same(20), theme::canvas());
            dashed_rect(
                &painter,
                dz_rect.shrink(10.0),
                Stroke::new(1.4, theme::border()),
            );

            // Ikon upload di tengah
            let icon_rect = egui::Rect::from_center_size(
                dz_rect.center() - Vec2::new(0.0, 32.0),
                Vec2::splat(44.0),
            );
            painter.rect_filled(icon_rect, CornerRadius::same(14), theme::accent_soft());
            theme::icon(
                &painter,
                icon_rect.shrink(10.0),
                theme::Icon::Download,
                theme::accent(),
                1.8,
            );

            painter.text(
                dz_rect.center() - Vec2::new(0.0, 2.0),
                Align2::CENTER_CENTER,
                "Jatuhkan file di sini, atau klik untuk impor",
                FontId::proportional(14.0),
                theme::text(),
            );
            painter.text(
                dz_rect.center() + Vec2::new(0.0, 19.0),
                Align2::CENTER_CENTER,
                "Mendukung JPG, PNG, PDF, WEBP, BMP, TIFF",
                FontId::proportional(10.5),
                theme::muted(),
            );

            if dz_resp.hovered() {
                ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
            }
            if dz_resp.clicked() {
                *action = Some(HomeAction::ImportPhotos);
            }

            // ---- 3 Tombol Quick Action ----
            ui.add_space(0.0); // sudah di dalam frame

            let avail = ui.available_width();
            let btn_w = ((avail - 32.0) / 3.0).max(100.0);
            let btn_h = 52.0;

            egui::Frame::new()
                .inner_margin(Margin {
                    left: 14,
                    right: 14,
                    top: 0,
                    bottom: 14,
                })
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 10.0;

                        // Cetak Foto
                        quick_action_card(
                            ui,
                            Vec2::new(btn_w, btn_h),
                            ShortcutIcon::Print,
                            theme::accent(),
                            "Cetak Foto",
                            "Susun & cetak pas foto",
                            action,
                            HomeAction::OpenSheet,
                        );

                        // Pemindai
                        quick_action_card(
                            ui,
                            Vec2::new(btn_w, btn_h),
                            ShortcutIcon::ScanOne,
                            theme::green(),
                            "Pemindai Dokumen",
                            "Koreksi perspektif foto",
                            action,
                            HomeAction::OpenScan,
                        );

                        // Konversi
                        quick_action_card(
                            ui,
                            Vec2::new(btn_w, btn_h),
                            ShortcutIcon::Convert,
                            theme::yellow(),
                            "Konversi Dokumen",
                            "PDF, Word, Gambar",
                            action,
                            HomeAction::OpenConvert,
                        );
                    });
                });
        });
    }

    // ------------------------------------------------------------------ tools grid

    fn home_tools_grid(&mut self, ui: &mut egui::Ui, action: &mut Option<HomeAction>) {
        theme::card().show(ui, |ui| {
            ui.label(
                RichText::new("Pintasan Cepat")
                    .size(13.5)
                    .strong()
                    .color(theme::text()),
            );
            ui.add_space(12.0);

            let tools: &[(&str, ShortcutIcon, Color32, HomeAction)] = &[
                (
                    "Impor Foto",
                    ShortcutIcon::ImportPhoto,
                    theme::accent(),
                    HomeAction::ImportPhotos,
                ),
                (
                    "Scan 1 Foto",
                    ShortcutIcon::ScanOne,
                    theme::green(),
                    HomeAction::ScanSingle,
                ),
                (
                    "Scan Banyak",
                    ShortcutIcon::ScanMany,
                    theme::green(),
                    HomeAction::ScanMulti,
                ),
                (
                    "Konversi File",
                    ShortcutIcon::Convert,
                    theme::yellow(),
                    HomeAction::ConvertFiles,
                ),
                (
                    "Cetak Sekarang",
                    ShortcutIcon::Print,
                    theme::red(),
                    HomeAction::OpenSheet,
                ),
                (
                    "Simpan PDF",
                    ShortcutIcon::SavePdf,
                    theme::blue(),
                    HomeAction::OpenSheet,
                ),
                (
                    "Alat & Cek",
                    ShortcutIcon::Tools,
                    theme::muted(),
                    HomeAction::OpenTools,
                ),
            ];

            let gap = 12.0;
            let min_tile_w = 86.0;
            let max_tile_w = 200.0;
            let avail = ui.available_width();
            let count = tools.len();
            let per_row = (((avail + gap) / (min_tile_w + gap)).floor() as usize).max(1);

            if per_row >= count {
                // Isi lebar kartu: tile ikut melebar saat jendela diperbesar.
                let tile_w = ((avail - gap * (count.saturating_sub(1)) as f32) / count as f32)
                    .clamp(min_tile_w, max_tile_w);
                let tile_h = 82.0 * (tile_w / min_tile_w).sqrt().clamp(1.0, 1.35);
                let row_w = count as f32 * tile_w + count.saturating_sub(1) as f32 * gap;
                let pad = ((avail - row_w) / 2.0).max(0.0);
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = gap;
                    ui.add_space(pad);
                    for (label, icon, color, nav) in tools {
                        if tool_tile(ui, Vec2::new(tile_w, tile_h), *icon, *color, label) {
                            *action = Some(*nav);
                        }
                    }
                });
            } else {
                let tile_size = Vec2::new(min_tile_w, 82.0);
                for chunk in tools.chunks(per_row) {
                    let row_w = chunk.len() as f32 * tile_size.x
                        + chunk.len().saturating_sub(1) as f32 * gap;
                    let pad = ((avail - row_w) / 2.0).max(0.0);
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = gap;
                        ui.add_space(pad);
                        for (label, icon, color, nav) in chunk {
                            if tool_tile(ui, tile_size, *icon, *color, label) {
                                *action = Some(*nav);
                            }
                        }
                    });
                    ui.add_space(gap);
                }
            }
        });
    }

    // ------------------------------------------------------------------ recent

    fn home_recent(&mut self, ui: &mut egui::Ui, action: &mut Option<HomeAction>) {
        theme::card().show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("File Terbaru")
                        .size(13.5)
                        .strong()
                        .color(theme::text()),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if theme::secondary_button(ui, "Lihat Semua →").clicked() {
                        *action = Some(HomeAction::OpenHistory);
                    }
                });
            });
            ui.add_space(10.0);

            if self.last_output.is_empty() {
                // Placeholder kosong
                let (rect, _) =
                    ui.allocate_exact_size(Vec2::new(ui.available_width(), 60.0), Sense::hover());
                ui.painter().text(
                    rect.center(),
                    Align2::CENTER_CENTER,
                    "Belum ada file yang diproses.",
                    FontId::proportional(12.0),
                    theme::muted(),
                );
                return;
            }

            for path in self.last_output.iter().rev().take(5) {
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("file");
                let ext = path
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("")
                    .to_ascii_uppercase();

                let (row, resp) =
                    ui.allocate_exact_size(Vec2::new(ui.available_width(), 40.0), Sense::click());
                let painter = ui.painter_at(row);
                if resp.hovered() {
                    painter.rect_filled(row, CornerRadius::same(10), theme::card_hover());
                }

                // Badge ekstensi
                let badge_rect =
                    egui::Rect::from_min_size(row.min + Vec2::new(4.0, 8.0), Vec2::new(36.0, 24.0));
                painter.rect_filled(badge_rect, CornerRadius::same(7), theme::accent_soft());
                painter.text(
                    badge_rect.center(),
                    Align2::CENTER_CENTER,
                    if ext.is_empty() { "FILE" } else { &ext },
                    FontId::proportional(9.0),
                    theme::accent(),
                );

                // Nama file
                painter.text(
                    egui::pos2(row.left() + 50.0, row.center().y - 5.0),
                    Align2::LEFT_CENTER,
                    shorten(name, 52),
                    FontId::proportional(12.5),
                    theme::text(),
                );

                // Path singkat
                let dir = path
                    .parent()
                    .and_then(|p| p.to_str())
                    .unwrap_or("")
                    .to_string();
                painter.text(
                    egui::pos2(row.left() + 50.0, row.center().y + 10.0),
                    Align2::LEFT_CENTER,
                    shorten(&dir, 60),
                    FontId::proportional(9.5),
                    theme::dim(),
                );

                if resp.clicked() {
                    crate::util::open_path(path);
                }
                if resp.hovered() {
                    ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
                }

                ui.add_space(2.0);
            }
        });
    }
}

// ---------------------------------------------------------------------------
// Helper widget
// ---------------------------------------------------------------------------

/// Kartu quick action dengan ikon berwarna + label + sub-label.
#[allow(clippy::too_many_arguments)]
fn quick_action_card(
    ui: &mut egui::Ui,
    size: Vec2,
    icon: ShortcutIcon,
    color: Color32,
    label: &str,
    sublabel: &str,
    action: &mut Option<HomeAction>,
    nav: HomeAction,
) {
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
    let painter = ui.painter_at(rect);

    let bg = if resp.hovered() {
        theme::card_hover()
    } else {
        theme::card_fill()
    };
    painter.rect_filled(rect, CornerRadius::same(14), bg);
    painter.rect_stroke(
        rect,
        CornerRadius::same(14),
        Stroke::new(1.0, theme::border_soft()),
        StrokeKind::Inside,
    );

    // Ikon berlapis
    let icon_rect = egui::Rect::from_center_size(
        egui::pos2(rect.left() + 24.0, rect.center().y),
        Vec2::splat(36.0),
    );
    shortcut_icon(&painter, icon_rect, icon, color);

    // Teks
    painter.text(
        egui::pos2(rect.left() + 50.0, rect.center().y - 7.0),
        Align2::LEFT_CENTER,
        label,
        FontId::proportional(12.5),
        theme::text(),
    );
    painter.text(
        egui::pos2(rect.left() + 50.0, rect.center().y + 10.0),
        Align2::LEFT_CENTER,
        sublabel,
        FontId::proportional(9.5),
        theme::dim(),
    );

    if resp.hovered() {
        ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
    }
    if resp.clicked() {
        *action = Some(nav);
    }
}

/// Tile kecil dalam grid pintasan.
fn tool_tile(
    ui: &mut egui::Ui,
    size: Vec2,
    icon: ShortcutIcon,
    color: Color32,
    label: &str,
) -> bool {
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
    let painter = ui.painter_at(rect);

    let bg = if resp.hovered() {
        theme::card_hover()
    } else {
        theme::input()
    };
    painter.rect_filled(rect, CornerRadius::same(16), bg);
    painter.rect_stroke(
        rect,
        CornerRadius::same(16),
        Stroke::new(1.0, theme::border_soft()),
        StrokeKind::Inside,
    );

    // Ikon berlapis bergaya kartu
    let ratio = (size.y / 82.0).clamp(1.0, 1.35);
    let icon_side = (36.0 * ratio).min(46.0);
    let icon_rect = egui::Rect::from_center_size(
        egui::pos2(rect.center().x, rect.top() + 28.0 * ratio),
        Vec2::splat(icon_side),
    );
    shortcut_icon(&painter, icon_rect, icon, color);

    // Label
    painter.text(
        egui::pos2(rect.center().x, rect.bottom() - 15.0 * ratio),
        Align2::CENTER_CENTER,
        label,
        FontId::proportional(10.5 * ratio.min(1.15)),
        theme::text(),
    );

    if resp.hovered() {
        ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
    }
    resp.clicked()
}

// ---------------------------------------------------------------------------
// Ikon berlapis untuk Pintasan Cepat
// ---------------------------------------------------------------------------

fn shortcut_icon(painter: &egui::Painter, rect: egui::Rect, icon: ShortcutIcon, color: Color32) {
    let s = rect.width().min(rect.height());
    let c = rect.center();
    let card = s * 0.60;
    let radius = (card * 0.24).max(2.0) as u8;
    let off = s * 0.13;
    let light = mix(color, Color32::WHITE, 0.55);
    let back = egui::Rect::from_center_size(c + Vec2::new(-off, -off), Vec2::splat(card));
    let front = egui::Rect::from_center_size(c + Vec2::new(off, off), Vec2::splat(card));
    let stroke = Stroke::new((s * 0.055).max(1.0), Color32::WHITE);

    match icon {
        ShortcutIcon::ImportPhoto => {
            painter.rect_filled(back, CornerRadius::same(radius), light);
            painter.rect_filled(front, CornerRadius::same(radius), color);
            photo_glyph(painter, front, stroke);
        }
        ShortcutIcon::ScanOne => {
            // Satu kartu dengan bingkai pindai.
            let solo = egui::Rect::from_center_size(c, Vec2::splat(card * 1.05));
            painter.rect_filled(solo, CornerRadius::same(radius), color);
            scan_glyph(painter, solo, stroke);
        }
        ShortcutIcon::ScanMany => {
            painter.rect_filled(back, CornerRadius::same(radius), light);
            painter.rect_filled(front, CornerRadius::same(radius), color);
            scan_glyph(painter, front, stroke);
        }
        ShortcutIcon::Convert => {
            // Kartu Word di depan kartu PowerPoint.
            let orange = Color32::from_rgb(0xE8, 0x6A, 0x3C);
            let word_blue = Color32::from_rgb(0x4B, 0x74, 0xC9);
            painter.rect_filled(back, CornerRadius::same(radius), orange);
            painter.rect_filled(front, CornerRadius::same(radius), word_blue);
            letter_glyph(painter, back, "P", card * 0.50);
            letter_glyph(painter, front, "W", card * 0.52);
        }
        ShortcutIcon::Print => {
            painter.rect_filled(back, CornerRadius::same(radius), light);
            painter.rect_filled(front, CornerRadius::same(radius), color);
            printer_glyph(painter, front, color);
        }
        ShortcutIcon::SavePdf => {
            painter.rect_filled(back, CornerRadius::same(radius), light);
            painter.rect_filled(front, CornerRadius::same(radius), color);
            letter_glyph(painter, front, "PDF", card * 0.38);
        }
        ShortcutIcon::Tools => {
            let q = s * 0.27;
            let step = q + s * 0.055;
            let check_stroke = Stroke::new((s * 0.050).max(1.0), Color32::WHITE);
            for (i, (dx, dy)) in [(-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)]
                .into_iter()
                .enumerate()
            {
                let center = c + Vec2::new(dx * step * 0.5, dy * step * 0.5);
                let rect = egui::Rect::from_center_size(center, Vec2::splat(q));
                painter.rect_filled(
                    rect,
                    CornerRadius::same((q * 0.26) as u8),
                    if i == 3 { color } else { light },
                );
                if i == 3 {
                    check_glyph(painter, rect, check_stroke);
                } else {
                    painter.circle_filled(rect.center(), q * 0.10, color);
                }
            }
        }
    }
}

fn letter_glyph(painter: &egui::Painter, card: egui::Rect, text: &str, size: f32) {
    painter.text(
        card.center(),
        Align2::CENTER_CENTER,
        text,
        FontId::proportional(size),
        Color32::WHITE,
    );
}

fn photo_glyph(painter: &egui::Painter, card: egui::Rect, stroke: Stroke) {
    let r = card.shrink(card.width() * 0.22);
    painter.rect_stroke(
        r,
        CornerRadius::same((r.width() * 0.12).max(1.0) as u8),
        stroke,
        StrokeKind::Middle,
    );
    let p = |x: f32, y: f32| egui::pos2(r.left() + r.width() * x, r.top() + r.height() * y);
    painter.line_segment([p(0.10, 0.84), p(0.40, 0.52)], stroke);
    painter.line_segment([p(0.40, 0.52), p(0.62, 0.72)], stroke);
    painter.line_segment([p(0.54, 0.64), p(0.82, 0.38)], stroke);
    painter.circle_filled(p(0.28, 0.28), r.width() * 0.09, Color32::WHITE);
}

fn scan_glyph(painter: &egui::Painter, card: egui::Rect, stroke: Stroke) {
    let r = card.shrink(card.width() * 0.25);
    let l = r.width() * 0.36;
    for (p, dx, dy) in [
        (r.left_top(), 1.0, 1.0),
        (r.right_top(), -1.0, 1.0),
        (r.left_bottom(), 1.0, -1.0),
        (r.right_bottom(), -1.0, -1.0),
    ] {
        painter.line_segment([p, p + Vec2::new(dx * l, 0.0)], stroke);
        painter.line_segment([p, p + Vec2::new(0.0, dy * l)], stroke);
    }
    painter.line_segment(
        [
            egui::pos2(r.left(), r.center().y),
            egui::pos2(r.right(), r.center().y),
        ],
        stroke,
    );
}

fn printer_glyph(painter: &egui::Painter, card: egui::Rect, slot_color: Color32) {
    let w = card.width() * 0.66;
    let h = card.height() * 0.34;
    let body =
        egui::Rect::from_center_size(card.center() + Vec2::new(0.0, h * 0.10), Vec2::new(w, h));
    let paper_top = egui::Rect::from_center_size(
        egui::pos2(card.center().x, body.top() - h * 0.12),
        Vec2::new(w * 0.52, h * 0.62),
    );
    let paper_bottom = egui::Rect::from_center_size(
        egui::pos2(card.center().x, body.bottom() + h * 0.10),
        Vec2::new(w * 0.52, h * 0.55),
    );
    let radius = CornerRadius::same((h * 0.18).max(1.0) as u8);
    painter.rect_filled(paper_top, radius, Color32::WHITE);
    painter.rect_filled(body, radius, Color32::WHITE);
    painter.rect_filled(paper_bottom, radius, Color32::WHITE);
    painter.rect_filled(
        egui::Rect::from_center_size(
            egui::pos2(body.center().x, body.center().y - h * 0.12),
            Vec2::new(w * 0.50, h * 0.16),
        ),
        CornerRadius::same((h * 0.05).max(1.0) as u8),
        slot_color,
    );
}

fn check_glyph(painter: &egui::Painter, card: egui::Rect, stroke: Stroke) {
    let r = card.shrink(card.width() * 0.28);
    let bottom = egui::pos2(r.left() + r.width() * 0.40, r.bottom());
    painter.line_segment([egui::pos2(r.left(), r.center().y), bottom], stroke);
    painter.line_segment([bottom, egui::pos2(r.right(), r.top())], stroke);
}

fn mix(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    let blend = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t).round() as u8;
    Color32::from_rgb(
        blend(a.r(), b.r()),
        blend(a.g(), b.g()),
        blend(a.b(), b.b()),
    )
}

fn shorten(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_string();
    }
    let mut out: String = text.chars().take(max.saturating_sub(1)).collect();
    out.push('…');
    out
}
