use std::time::Duration;

use eframe::egui::{
    self, Align, Align2, Color32, CornerRadius, FontId, Id, Layout, Margin, Order, RichText, Sense,
    Stroke, StrokeKind, Vec2,
};

use crate::app::{CetakinApp, Tab, ToastKind};
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
}

impl CetakinApp {
    // -------------------------------------------------------------- chrome

    pub(crate) fn sidebar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let size = Vec2::splat(38.0);
            let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
            ui.painter()
                .rect_filled(rect, CornerRadius::same(11), theme::accent());
            ui.painter().rect_stroke(
                rect,
                CornerRadius::same(11),
                Stroke::new(1.0, Color32::from_white_alpha(70)),
                StrokeKind::Inside,
            );
            theme::icon(
                ui.painter(),
                rect.shrink(8.0),
                theme::Icon::Print,
                Color32::WHITE,
                1.8,
            );
            ui.add_space(2.0);
            ui.vertical(|ui| {
                ui.label(
                    RichText::new("cetakin.")
                        .size(21.0)
                        .strong()
                        .color(theme::text()),
                );
                ui.label(
                    RichText::new("digital printing")
                        .size(10.0)
                        .color(theme::dim()),
                );
            });
        });
        ui.add_space(30.0);
        theme::section_label(ui, "Workspace");
        ui.add_space(9.0);

        let items = [
            (Tab::Sheet, theme::Icon::Photo, "Cetak Foto"),
            (Tab::Convert, theme::Icon::Convert, "Konversi"),
            (Tab::Tools, theme::Icon::Tools, "Alat"),
            (Tab::Settings, theme::Icon::Settings, "Pengaturan"),
        ];
        for (tab, icon, label) in items {
            if self.nav_item(ui, icon, label, self.tab == tab) {
                self.tab = tab;
            }
            ui.add_space(2.0);
        }

        ui.add_space((ui.available_height() - 168.0).max(0.0));
        self.sidebar_tip(ui);
        ui.add_space(10.0);
        self.sidebar_status(ui);
    }

    fn nav_item(&self, ui: &mut egui::Ui, icon: theme::Icon, label: &str, selected: bool) -> bool {
        let height = 44.0;
        let (rect, response) = ui.allocate_exact_size(
            Vec2::new(ui.available_width(), height),
            egui::Sense::click(),
        );
        let hovered = response.hovered();
        if selected {
            // Baris terpilih: kaca biru lembut khas sidebar Tahoe.
            ui.painter()
                .rect_filled(rect, CornerRadius::same(11), theme::accent_soft());
            ui.painter().rect_stroke(
                rect,
                CornerRadius::same(11),
                Stroke::new(1.0, Color32::from_rgba_unmultiplied(10, 132, 255, 90)),
                StrokeKind::Inside,
            );
        } else if hovered {
            ui.painter()
                .rect_filled(rect, CornerRadius::same(11), theme::card_hover());
        }
        let icon_rect = egui::Rect::from_center_size(
            egui::pos2(rect.left() + 23.0, rect.center().y),
            Vec2::splat(18.0),
        );
        theme::icon(
            ui.painter(),
            icon_rect,
            icon,
            if selected {
                theme::text()
            } else {
                theme::muted()
            },
            1.5,
        );
        ui.painter().text(
            egui::pos2(rect.left() + 42.0, rect.center().y),
            Align2::LEFT_CENTER,
            label,
            FontId::proportional(14.0),
            if selected {
                theme::text()
            } else {
                theme::muted()
            },
        );
        response.clicked()
    }

    fn sidebar_tip(&self, ui: &mut egui::Ui) {
        let (title, body) = match self.tab {
            Tab::Sheet => (
                "Tips margin",
                "Margin Normal 2,54 cm cocok untuk printer rumahan. Untuk hasil padat, pilih Sempit.",
            ),
            Tab::Convert => (
                "Tips konversi",
                "Untuk PDF ke Word yang rapi, sertakan gambar halaman. Teks tetap bisa diedit.",
            ),
            Tab::Tools => (
                "Tips kualitas",
                "Pas foto 3×4 sebaiknya minimal 300 DPI. Di bawah 150 DPI akan terlihat pecah.",
            ),
            Tab::Settings => (
                "Tips studio",
                "Isi nama studio, lalu aktifkan footer agar setiap lembar punya identitas.",
            ),
        };
        egui::Frame::new()
            .fill(theme::card_fill())
            .corner_radius(CornerRadius::same(14))
            .stroke(Stroke::new(1.0, theme::border_soft()))
            .inner_margin(Margin::same(14))
            .show(ui, |ui| {
                ui.label(
                    RichText::new(title)
                        .size(12.5)
                        .strong()
                        .color(theme::accent()),
                );
                ui.add_space(4.0);
                ui.label(RichText::new(body).size(11.5).color(theme::muted()));
            });
    }

    fn sidebar_status(&self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if self.job.is_some() {
                ui.spinner();
                ui.label(
                    RichText::new("Memproses…")
                        .size(11.5)
                        .color(theme::yellow()),
                );
            } else {
                theme::dot(ui, theme::accent());
                ui.label(
                    RichText::new("Siap mencetak")
                        .size(11.5)
                        .color(theme::muted()),
                );
            }
        });
        ui.label(RichText::new("v0.1.0").size(10.0).color(theme::dim()));
    }

    pub(crate) fn header(&mut self, ui: &mut egui::Ui) {
        let mut action: Option<HeaderAction> = None;
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(
                    RichText::new(self.tab.title())
                        .size(24.0)
                        .strong()
                        .color(theme::text()),
                );
                ui.label(
                    RichText::new(self.tab.subtitle())
                        .size(12.0)
                        .color(theme::muted()),
                );
            });
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| match self.tab {
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
            });
        });
        ui.add_space(14.0);

        match action {
            Some(HeaderAction::Import) => self.import_photos_dialog(),
            Some(HeaderAction::Export) => self.export_pdf_dialog(),
            Some(HeaderAction::Print) => self.print_now(),
            Some(HeaderAction::RunConvert) => self.run_convert(),
            Some(HeaderAction::AddConvertFiles) => self.convert.pick_files_dialog(),
            Some(HeaderAction::PickInspector) => self.pick_inspector(),
            Some(HeaderAction::RefreshTools) => self.refresh_tools(),
            Some(HeaderAction::SavePrefs) => self.save_prefs_manual(),
            None => {}
        }
    }

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
                    ToastKind::Success => theme::catppuccin::GREEN,
                    ToastKind::Error => theme::catppuccin::RED,
                    ToastKind::Info => theme::catppuccin::BLUE,
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
                        color: Color32::from_rgba_unmultiplied(
                            0x11,
                            0x11,
                            0x1b,
                            (150.0 + 45.0 * t) as u8,
                        ),
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
                    theme::catppuccin::RED
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
                    egui::UiBuilder::new()
                        .max_rect(inner)
                        .layout(Layout::top_down(Align::Min)),
                );
                child.set_clip_rect(inner);

                // Samakan warna widget bawaan (spinner, progress) dengan Catppuccin.
                let mut visuals = child.visuals().clone();
                visuals.override_text_color = Some(theme::island_text());
                visuals.extreme_bg_color = theme::catppuccin::SURFACE0;
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
