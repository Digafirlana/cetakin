use std::time::Duration;

use eframe::egui::{
    self, Align, Align2, Color32, CornerRadius, FontId, Id, Layout, Margin, Order, RichText, Sense,
    Stroke, StrokeKind, UiBuilder, Vec2,
};

use crate::app::{CetakinApp, Tab, ToastKind};
use crate::layout;
use crate::theme;

enum IslandContent {
    Job {
        label: String,
        message: String,
        progress: f32,
    },
    Toast {
        title: String,
        detail: String,
        kind: ToastKind,
        auto_open: bool,
    },
}

impl IslandContent {
    fn from_app(app: &CetakinApp) -> Option<Self> {
        if let Some(job) = &app.job {
            return Some(Self::Job {
                label: job.label.clone(),
                message: job.message.clone(),
                progress: job.progress,
            });
        }
        let toast = app.toast.as_ref()?;
        Some(Self::Toast {
            title: toast.title.clone(),
            detail: toast.detail.clone(),
            kind: toast.kind,
            auto_open: toast.kind != ToastKind::Error
                && toast.created.elapsed() < Duration::from_secs(4),
        })
    }
}

// ---------------------------------------------------------------------------
// Preferences
// ---------------------------------------------------------------------------

#[derive(Clone, Copy)]
enum HeaderAction {
    Import,
    Export,
    Print,
    RunConvert,
    AddConvertFiles,
    PickInspector,
    RefreshTools,
    SavePrefs,
    ScanPick,
    ScanSave,
}

impl CetakinApp {
    // -------------------------------------------------------------- sidebar

    /// Inisial studio untuk avatar sidebar (maks. 2 huruf).
    fn initials(&self) -> String {
        let mut out = String::new();
        for word in self.studio.split_whitespace().take(2) {
            if let Some(ch) = word.chars().next() {
                out.extend(ch.to_uppercase());
            }
        }
        if out.is_empty() {
            "CT".to_string()
        } else {
            out
        }
    }

    pub(crate) fn sidebar(&mut self, ui: &mut egui::Ui) {
        let available_h = ui.available_height();
        egui::Frame::new()
            .fill(theme::sidebar())
            .corner_radius(CornerRadius::same(24))
            .stroke(Stroke::new(1.0, theme::border_soft()))
            .shadow(egui::epaint::Shadow {
                offset: [0, 12],
                blur: 32,
                spread: 0,
                color: theme::shadow(),
            })
            .inner_margin(Margin::same(14))
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                ui.set_min_height((available_h - 28.0).max(0.0));
                self.sidebar_brand(ui);
                ui.add_space(20.0);

                let main_items = [
                    (Tab::Home, theme::Icon::Stack, "Beranda"),
                    (Tab::History, theme::Icon::History, "Riwayat"),
                    (Tab::Settings, theme::Icon::Settings, "Pengaturan"),
                ];
                for (tab, icon, label) in main_items {
                    if self.nav_item(ui, icon, label, self.tab == tab) {
                        self.tab = tab;
                    }
                    ui.add_space(2.0);
                }

                // Bagi ruang kosong secara merata: setengah di atas kartu ringkasan
                // dan setengah di bawahnya, supaya atas-bawah sidebar seimbang.
                let free = (ui.available_height() - 150.0).max(20.0);
                ui.add_space(free * 0.5);
                self.sidebar_summary(ui);
                ui.add_space(free * 0.5);
                self.sidebar_status(ui);
            });
    }

    fn sidebar_brand(&self, ui: &mut egui::Ui) {
        let (rect, response) =
            ui.allocate_exact_size(Vec2::new(ui.available_width(), 46.0), egui::Sense::click());
        if response.hovered() {
            ui.painter()
                .rect_filled(rect, CornerRadius::same(12), theme::card_hover());
        }
        let avatar = egui::Rect::from_min_size(
            egui::pos2(rect.left() + 4.0, rect.center().y - 17.0),
            Vec2::splat(34.0),
        );
        ui.painter()
            .rect_filled(avatar, CornerRadius::same(10), theme::accent_soft());
        ui.painter().text(
            avatar.center(),
            Align2::CENTER_CENTER,
            self.initials(),
            FontId::proportional(12.5),
            theme::accent(),
        );
        let display_name = if self.studio.trim().is_empty() {
            "Cetakin Studio".to_string()
        } else {
            shorten(self.studio.trim(), 20)
        };
        ui.painter().text(
            egui::pos2(avatar.right() + 10.0, rect.center().y - 7.0),
            Align2::LEFT_CENTER,
            display_name,
            FontId::proportional(13.0),
            theme::text(),
        );
        ui.painter().text(
            egui::pos2(avatar.right() + 10.0, rect.center().y + 9.0),
            Align2::LEFT_CENTER,
            "Workspace cetak",
            FontId::proportional(10.0),
            theme::dim(),
        );
        theme::icon(
            ui.painter(),
            egui::Rect::from_center_size(
                egui::pos2(rect.right() - 9.0, rect.center().y),
                Vec2::splat(14.0),
            ),
            theme::Icon::ChevronDown,
            theme::dim(),
            1.4,
        );
    }

    fn nav_item(&self, ui: &mut egui::Ui, icon: theme::Icon, label: &str, selected: bool) -> bool {
        let height = 40.0;
        let (rect, response) = ui.allocate_exact_size(
            Vec2::new(ui.available_width(), height),
            egui::Sense::click(),
        );
        let hovered = response.hovered();
        if selected {
            ui.painter()
                .rect_filled(rect, CornerRadius::same(11), theme::nav_active_bg());
        } else if hovered {
            ui.painter()
                .rect_filled(rect, CornerRadius::same(11), theme::card_hover());
        }
        let fg = if selected {
            theme::nav_active_fg()
        } else {
            theme::muted()
        };
        let icon_rect = egui::Rect::from_center_size(
            egui::pos2(rect.left() + 20.0, rect.center().y),
            Vec2::splat(17.0),
        );
        theme::icon(ui.painter(), icon_rect, icon, fg, 1.5);
        ui.painter().text(
            egui::pos2(rect.left() + 40.0, rect.center().y),
            Align2::LEFT_CENTER,
            label,
            FontId::proportional(13.5),
            if selected { fg } else { theme::text() },
        );
        if selected {
            theme::icon(
                ui.painter(),
                egui::Rect::from_center_size(
                    egui::pos2(rect.right() - 16.0, rect.center().y),
                    Vec2::splat(13.0),
                ),
                theme::Icon::ChevronRight,
                fg,
                1.4,
            );
        }
        response.clicked()
    }

    fn sidebar_summary(&self, ui: &mut egui::Ui) {
        let sheet = layout::compute(&self.settings);
        let used = self.photos.len().min(sheet.capacity.max(1));
        let capacity = sheet.capacity.max(1);
        let ratio = (used as f32 / capacity as f32).clamp(0.0, 1.0);

        egui::Frame::new()
            .fill(theme::input())
            .corner_radius(CornerRadius::same(15))
            .stroke(Stroke::new(1.0, theme::border_soft()))
            .inner_margin(Margin {
                left: 13,
                right: 13,
                top: 11,
                bottom: 12,
            })
            .show(ui, |ui| {
                ui.label(
                    RichText::new("KAPASITAS LEMBAR")
                        .size(9.0)
                        .strong()
                        .extra_letter_spacing(0.6)
                        .color(theme::dim()),
                );
                ui.add_space(3.0);
                ui.label(
                    RichText::new(format!("{used} / {capacity} foto"))
                        .size(12.5)
                        .strong()
                        .color(theme::text()),
                );
                ui.add_space(7.0);

                let (track, _) = ui.allocate_exact_size(
                    Vec2::new(ui.available_width(), 6.0),
                    egui::Sense::hover(),
                );
                ui.painter()
                    .rect_filled(track, CornerRadius::same(3), theme::border_soft());
                if ratio > 0.0 {
                    let fill = egui::Rect::from_min_size(
                        track.min,
                        Vec2::new((track.width() * ratio).max(3.0), track.height()),
                    );
                    ui.painter()
                        .rect_filled(fill, CornerRadius::same(3), theme::accent());
                }
                ui.add_space(6.0);
                ui.label(
                    RichText::new(format!(
                        "{} · {} foto",
                        short_paper(self.settings.paper.label()),
                        self.photos.len()
                    ))
                    .size(9.5)
                    .color(theme::dim()),
                );
            });
    }

    fn sidebar_status(&self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if self.job.is_some() {
                ui.spinner();
                ui.label(
                    RichText::new("Memproses…")
                        .size(11.0)
                        .color(theme::yellow()),
                );
            } else {
                theme::dot(ui, theme::green());
                ui.label(
                    RichText::new("Siap mencetak")
                        .size(11.0)
                        .color(theme::muted()),
                );
            }
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label(RichText::new("v0.1.0").size(9.5).color(theme::dim()));
            });
        });
    }

    // -------------------------------------------------------------- header

    pub(crate) fn header(&mut self, ui: &mut egui::Ui) {
        let mut action: Option<HeaderAction> = None;
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(
                    RichText::new(self.tab.title())
                        .size(22.0)
                        .strong()
                        .color(theme::text()),
                );
                ui.label(
                    RichText::new(self.tab.subtitle())
                        .size(11.5)
                        .color(theme::muted()),
                );
            });
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                // Avatar studio kecil di ujung kanan.
                let (rect, _) = ui.allocate_exact_size(Vec2::splat(34.0), Sense::hover());
                ui.painter()
                    .rect_filled(rect, CornerRadius::same(10), theme::accent_soft());
                ui.painter().text(
                    rect.center(),
                    Align2::CENTER_CENTER,
                    self.initials(),
                    FontId::proportional(11.5),
                    theme::accent(),
                );
                ui.add_space(6.0);
                match self.tab {
                    Tab::Home | Tab::History => {}
                    Tab::Sheet => {
                        if theme::primary_button(ui, "Impor Foto").clicked() {
                            action = Some(HeaderAction::Import);
                        }
                        if theme::secondary_button(ui, "Simpan PDF").clicked() {
                            action = Some(HeaderAction::Export);
                        }
                        if theme::secondary_button(ui, "Cetak").clicked() {
                            action = Some(HeaderAction::Print);
                        }
                    }
                    Tab::Convert => {
                        if theme::primary_button(ui, "Jalankan").clicked() {
                            action = Some(HeaderAction::RunConvert);
                        }
                        if theme::secondary_button(ui, "Tambah File").clicked() {
                            action = Some(HeaderAction::AddConvertFiles);
                        }
                    }
                    Tab::Scan => {
                        if theme::primary_button(ui, "Pilih Foto").clicked() {
                            action = Some(HeaderAction::ScanPick);
                        }
                        if theme::secondary_button(ui, "Simpan Hasil").clicked() {
                            action = Some(HeaderAction::ScanSave);
                        }
                    }
                    Tab::Tools => {
                        if theme::primary_button(ui, "Pilih Gambar").clicked() {
                            action = Some(HeaderAction::PickInspector);
                        }
                        if theme::secondary_button(ui, "Segarkan Alat").clicked() {
                            action = Some(HeaderAction::RefreshTools);
                        }
                    }
                    Tab::Settings => {
                        if theme::primary_button(ui, "Simpan Preferensi").clicked() {
                            action = Some(HeaderAction::SavePrefs);
                        }
                    }
                }
            });
        });
        ui.add_space(12.0);

        match action {
            Some(HeaderAction::Import) => self.import_photos_dialog(),
            Some(HeaderAction::Export) => self.export_pdf_dialog(),
            Some(HeaderAction::Print) => self.print_now(),
            Some(HeaderAction::RunConvert) => self.run_convert(),
            Some(HeaderAction::AddConvertFiles) => self.convert.pick_files_dialog(),
            Some(HeaderAction::PickInspector) => self.pick_inspector(),
            Some(HeaderAction::RefreshTools) => self.refresh_tools(),
            Some(HeaderAction::SavePrefs) => self.save_prefs_manual(),
            Some(HeaderAction::ScanPick) => self.pick_scan_source(ui.ctx()),
            Some(HeaderAction::ScanSave) => self.scan_save_image(),
            None => {}
        }
    }

    // -------------------------------------------------------------- hero

    /// Bar ringkas di atas tab Cetak Foto: statistik pekerjaan + tombol impor.
    pub(crate) fn hero(&mut self, ui: &mut egui::Ui) {
        let sheet = layout::compute(&self.settings);
        let dpi = self
            .min_dpi(&sheet)
            .map(|d| format!("{d:.0} DPI"))
            .unwrap_or_else(|| format!("{} DPI", self.settings.dpi));

        let mut import = false;
        egui::Frame::new()
            .fill(theme::card_fill())
            .corner_radius(CornerRadius::same(16))
            .stroke(Stroke::new(1.0, theme::border_soft()))
            .inner_margin(Margin::symmetric(16, 10))
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                ui.horizontal(|ui| {
                    hero_stat(ui, "Foto diimpor", &format!("{} foto", self.photos.len()));
                    hero_divider(ui);
                    hero_stat(
                        ui,
                        "Kertas",
                        &format!(
                            "{} · {}",
                            short_paper(self.settings.paper.label()),
                            self.settings.orientation.label()
                        ),
                    );
                    hero_divider(ui);
                    hero_stat(ui, "Resolusi", &dpi);
                    hero_divider(ui);
                    hero_stat(ui, "Kapasitas", &format!("{} slot", sheet.capacity));

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if theme::primary_button(ui, "Impor Foto").clicked() {
                            import = true;
                        }
                    });
                });
            });
        if import {
            self.import_photos_dialog();
        }
    }

    // -------------------------------------------------------------- island

    pub(crate) fn island_ui(&mut self, ctx: &egui::Context) {
        if let Some(toast) = &self.toast {
            let expired = toast.kind != ToastKind::Error
                && toast.created.elapsed() > Duration::from_secs(6)
                && !self.island_hovered
                && !self.island_pinned;
            if expired {
                self.toast = None;
            }
        }

        let Some(content) = IslandContent::from_app(self) else {
            self.island_hovered = false;
            self.island_pinned = false;
            return;
        };

        let (accent, compact_width, expanded_height, force_open) = match &content {
            IslandContent::Job { .. } => (theme::island_accent(), 262.0, 124.0, false),
            IslandContent::Toast { kind, .. } => {
                let color = match kind {
                    ToastKind::Success => theme::green(),
                    ToastKind::Error => theme::red(),
                    ToastKind::Info => theme::blue(),
                };
                let force_open = *kind == ToastKind::Error;
                (
                    color,
                    250.0,
                    if force_open { 122.0 } else { 112.0 },
                    force_open,
                )
            }
        };
        let auto_open = match &content {
            IslandContent::Toast { auto_open, .. } => *auto_open,
            IslandContent::Job { .. } => false,
        };
        let open_target = self.island_pinned || self.island_hovered || force_open || auto_open;
        let t = ctx.animate_bool_with_time(Id::unique("cetakin-island-open"), open_target, 0.22);
        let width = compact_width + (400.0 - compact_width) * t;
        let height = 42.0 + (expanded_height - 42.0) * t;
        // Kapsul penuh saat ringkas (r = tinggi/2), sudut lembut saat membesar.
        let radius = (height * 0.5).min(28.0);

        egui::Area::new(Id::unique("cetakin-island"))
            .anchor(Align2::CENTER_TOP, [0.0, 10.0])
            .order(Order::Foreground)
            .interactable(true)
            .show(ctx, |ui| {
                let (rect, response) =
                    ui.allocate_exact_size(egui::vec2(width, height), Sense::click());
                let corner = CornerRadius::same(radius.round() as u8);
                ui.painter().add(
                    egui::epaint::Shadow {
                        offset: [0, 10],
                        blur: (20.0 + 16.0 * t) as u8,
                        spread: 0,
                        color: Color32::from_black_alpha(if theme::is_dark() {
                            (120.0 + 45.0 * t) as u8
                        } else {
                            (32.0 + 26.0 * t) as u8
                        }),
                    }
                    .as_shape(rect, corner),
                );
                let border = if matches!(
                    &content,
                    IslandContent::Toast {
                        kind: ToastKind::Error,
                        ..
                    }
                ) {
                    theme::red()
                } else if t > 0.5 {
                    accent
                } else {
                    theme::island_border()
                };
                ui.painter().rect(
                    rect,
                    corner,
                    theme::island(),
                    Stroke::new(1.0, border),
                    StrokeKind::Inside,
                );

                let inner = rect.shrink2(egui::vec2(14.0 + 2.0 * t, 9.0 + 3.0 * t));
                let mut child = ui.new_child(
                    UiBuilder::new()
                        .max_rect(inner)
                        .layout(Layout::top_down(Align::Min)),
                );
                child.set_clip_rect(inner);

                // Samakan warna widget bawaan (spinner, progress) dengan tema.
                let mut visuals = child.visuals().clone();
                visuals.override_text_color = Some(theme::island_text());
                visuals.extreme_bg_color = theme::island_field();
                visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, theme::island_muted());
                visuals.widgets.active.fg_stroke = Stroke::new(1.0, accent);
                visuals.selection.bg_fill = accent;
                child.set_visuals(visuals);

                match &content {
                    IslandContent::Job {
                        label,
                        message,
                        progress,
                    } => {
                        child.horizontal(|ui| {
                            ui.add(egui::Spinner::new().color(accent));
                            let text = if t < 0.62 {
                                format!("{label} · {:.0}%", progress * 100.0)
                            } else {
                                label.clone()
                            };
                            ui.add(
                                egui::Label::new(
                                    RichText::new(text)
                                        .size(13.0)
                                        .strong()
                                        .color(theme::island_text()),
                                )
                                .truncate(),
                            );
                        });
                        if t > 0.62 {
                            child.add_space(7.0 * t);
                            child.add(
                                egui::ProgressBar::new(*progress)
                                    .fill(accent)
                                    .corner_radius(4)
                                    .desired_height(8.0),
                            );
                            child.add_space(3.0);
                            child.add(
                                egui::Label::new(
                                    RichText::new(message)
                                        .size(11.0)
                                        .color(theme::island_muted()),
                                )
                                .wrap(),
                            );
                        }
                    }
                    IslandContent::Toast {
                        title,
                        detail,
                        kind,
                        ..
                    } => {
                        child.horizontal(|ui| {
                            theme::dot(ui, accent);
                            ui.add(
                                egui::Label::new(
                                    RichText::new(title)
                                        .size(12.5)
                                        .strong()
                                        .color(theme::island_text()),
                                )
                                .truncate(),
                            );
                        });
                        if t > 0.62 && !detail.is_empty() {
                            child.add_space(4.0);
                            child.add(
                                egui::Label::new(
                                    RichText::new(detail)
                                        .size(11.0)
                                        .color(theme::island_muted()),
                                )
                                .wrap(),
                            );
                        }
                        if t > 0.9 && *kind == ToastKind::Error {
                            child.add_space(3.0);
                            child.label(
                                RichText::new("Klik untuk menutup")
                                    .size(10.0)
                                    .color(theme::island_dim()),
                            );
                        }
                    }
                }

                self.island_hovered = response.hovered();
                if response.clicked() {
                    if matches!(
                        &content,
                        IslandContent::Toast {
                            kind: ToastKind::Error,
                            ..
                        }
                    ) {
                        self.toast = None;
                        self.island_pinned = false;
                        self.island_hovered = false;
                    } else {
                        self.island_pinned = !self.island_pinned;
                    }
                    ctx.request_repaint();
                }
            });

        if t > 0.001 && t < 0.999 {
            ctx.request_repaint_after(Duration::from_millis(16));
        } else if self.job.is_some() {
            ctx.request_repaint_after(Duration::from_millis(120));
        }
        if self.toast.is_some() {
            ctx.request_repaint_after(Duration::from_millis(250));
        }
    }
}

/// Satu sel statistik di kartu hero.
fn hero_stat(ui: &mut egui::Ui, label: &str, value: &str) {
    let width = 112.0;
    ui.allocate_ui_with_layout(Vec2::new(width, 46.0), Layout::top_down(Align::Min), |ui| {
        ui.label(RichText::new(label).size(9.5).color(theme::muted()));
        ui.add_space(3.0);
        ui.add(
            egui::Label::new(
                RichText::new(value)
                    .size(13.5)
                    .strong()
                    .color(theme::text()),
            )
            .truncate(),
        );
    });
}

fn hero_divider(ui: &mut egui::Ui) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(1.0, 30.0), Sense::hover());
    ui.painter().line_segment(
        [rect.center_top(), rect.center_bottom()],
        Stroke::new(1.0, theme::border_soft()),
    );
}

fn shorten(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_string();
    }
    let mut out: String = text.chars().take(max.saturating_sub(1)).collect();
    out.push('…');
    out
}

/// "A4  (21 × 29,7 cm)" → "A4"
fn short_paper(label: &str) -> &str {
    label.split("  (").next().unwrap_or(label).trim()
}
