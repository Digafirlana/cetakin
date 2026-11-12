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
    OpenSettings,
    ImportPhotos,
    ScanSingle,
    ScanMulti,
    ConvertFiles,
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
            Some(HomeAction::OpenSettings) => self.tab = Tab::Settings,
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
            let (dz_rect, dz_resp) = ui.allocate_exact_size(
                Vec2::new(ui.available_width(), dz_height),
                Sense::click(),
            );

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
                .inner_margin(Margin { left: 14, right: 14, top: 0, bottom: 14 })
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 10.0;

                        // Cetak Foto
                        quick_action_card(
                            ui,
                            Vec2::new(btn_w, btn_h),
                            theme::Icon::Photo,
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
                            theme::Icon::Scan,
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
                            theme::Icon::Convert,
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

            let tools: &[(&str, theme::Icon, Color32, HomeAction)] = &[
                ("Impor Foto", theme::Icon::Photo, theme::accent(), HomeAction::ImportPhotos),
                ("Scan 1 Foto", theme::Icon::Scan, theme::green(), HomeAction::ScanSingle),
                ("Scan Banyak", theme::Icon::Scan, theme::green(), HomeAction::ScanMulti),
                ("Konversi File", theme::Icon::Convert, theme::yellow(), HomeAction::ConvertFiles),
                ("Cetak Sekarang", theme::Icon::Print, theme::red(), HomeAction::OpenSheet),
                ("Simpan PDF", theme::Icon::Download, theme::blue(), HomeAction::OpenSheet),
                ("Alat & Cek", theme::Icon::Tools, theme::muted(), HomeAction::OpenTools),
                ("Pengaturan", theme::Icon::Settings, theme::dim(), HomeAction::OpenSettings),
            ];

            let tile_size = Vec2::new(88.0, 86.0);
            let cols = ((ui.available_width() + 10.0) / (tile_size.x + 10.0)).floor() as usize;
            let cols = cols.max(4);

            let mut col = 0;
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = Vec2::splat(10.0);
                for (label, icon, color, nav) in tools {
                    if col >= cols {
                        col = 0;
                    }
                    if tool_tile(ui, tile_size, *icon, *color, label) {
                        *action = Some(*nav);
                    }
                    col += 1;
                }
            });
        });
    }

    // ------------------------------------------------------------------ recent

    fn home_recent(&mut self, ui: &mut egui::Ui, _action: &mut Option<HomeAction>) {
        theme::card().show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("File Terbaru")
                        .size(13.5)
                        .strong()
                        .color(theme::text()),
                );
                ui.with_layout(
                    egui::Layout::right_to_left(egui::Align::Center),
                    |ui| {
                        if theme::secondary_button(ui, "Lihat Semua →").clicked() {
                            // navigasi ke tab terkait bila perlu
                        }
                    },
                );
            });
            ui.add_space(10.0);

            if self.last_output.is_empty() {
                // Placeholder kosong
                let (rect, _) = ui.allocate_exact_size(
                    Vec2::new(ui.available_width(), 60.0),
                    Sense::hover(),
                );
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
                let name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("file");
                let ext = path
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("")
                    .to_ascii_uppercase();

                let (row, resp) = ui.allocate_exact_size(
                    Vec2::new(ui.available_width(), 40.0),
                    Sense::click(),
                );
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
    icon: theme::Icon,
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

    // Ikon lingkaran berwarna
    let icon_center = egui::pos2(rect.left() + 22.0, rect.center().y);
    let icon_rect =
        egui::Rect::from_center_size(icon_center, Vec2::splat(28.0));
    painter.rect_filled(
        icon_rect,
        CornerRadius::same(9),
        Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 28),
    );
    theme::icon(&painter, icon_rect.shrink(6.0), icon, color, 1.6);

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
    icon: theme::Icon,
    color: Color32,
    label: &str,
) -> bool {
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
    let painter = ui.painter_at(rect);

    let bg = if resp.hovered() { theme::card_hover() } else { theme::input() };
    painter.rect_filled(rect, CornerRadius::same(16), bg);
    painter.rect_stroke(
        rect,
        CornerRadius::same(16),
        Stroke::new(1.0, theme::border_soft()),
        StrokeKind::Inside,
    );

    // Lingkaran ikon
    let icon_rect = egui::Rect::from_center_size(
        egui::pos2(rect.center().x, rect.top() + 30.0),
        Vec2::splat(32.0),
    );
    painter.rect_filled(
        icon_rect,
        CornerRadius::same(10),
        Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 24),
    );
    theme::icon(&painter, icon_rect.shrink(7.0), icon, color, 1.6);

    // Label
    painter.text(
        egui::pos2(rect.center().x, rect.bottom() - 16.0),
        Align2::CENTER_CENTER,
        label,
        FontId::proportional(10.0),
        theme::text(),
    );

    if resp.hovered() {
        ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
    }
    resp.clicked()
}

fn shorten(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_string();
    }
    let mut out: String = text.chars().take(max.saturating_sub(1)).collect();
    out.push('…');
    out
}
