//! Tab Riwayat: daftar aktivitas cetak, simpan, konversi, dan pindai.

use std::time::Instant;

use eframe::egui::{self, Align, CornerRadius, Layout, RichText, ScrollArea, Sense, Vec2};

use crate::app::{CetakinApp, Tab};
use crate::theme;
use crate::util::{open_folder, open_path};

impl CetakinApp {
    pub fn history_tab(&mut self, ui: &mut egui::Ui) {
        let mut clear = false;
        let mut back_home = false;

        ScrollArea::vertical()
            .id_salt("history-scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                if self.history.is_empty() {
                    theme::card().show(ui, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.add_space(34.0);
                            let (rect, _) =
                                ui.allocate_exact_size(Vec2::splat(56.0), Sense::hover());
                            let painter = ui.painter();
                            painter.rect_filled(
                                rect,
                                CornerRadius::same(16),
                                theme::accent_soft(),
                            );
                            theme::icon(
                                painter,
                                rect.shrink(14.0),
                                theme::Icon::History,
                                theme::accent(),
                                1.6,
                            );
                            ui.add_space(14.0);
                            ui.label(
                                RichText::new("Belum ada riwayat")
                                    .size(14.0)
                                    .strong()
                                    .color(theme::text()),
                            );
                            ui.add_space(4.0);
                            ui.label(
                                RichText::new(
                                    "Hasil cetak, simpan PDF, konversi, dan pindai akan muncul di sini.",
                                )
                                .size(11.0)
                                .color(theme::muted()),
                            );
                            ui.add_space(14.0);
                            if theme::primary_button(ui, "Kembali ke Beranda").clicked() {
                                back_home = true;
                            }
                            ui.add_space(34.0);
                        });
                    });
                    return;
                }

                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("Riwayat Sesi")
                            .size(13.5)
                            .strong()
                            .color(theme::text()),
                    );
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if theme::secondary_button(ui, "Bersihkan").clicked() {
                            clear = true;
                        }
                        theme::badge(
                            ui,
                            &format!("{} aktivitas", self.history.len()),
                            theme::accent(),
                        );
                    });
                });
                ui.add_space(12.0);

                for entry in self.history.iter().rev() {
                    theme::card().show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(&entry.title)
                                    .size(13.0)
                                    .strong()
                                    .color(theme::text()),
                            );
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                ui.label(
                                    RichText::new(time_ago(entry.created))
                                        .size(10.0)
                                        .color(theme::dim()),
                                );
                            });
                        });
                        ui.label(
                            RichText::new(&entry.note)
                                .size(10.5)
                                .color(theme::muted()),
                        );
                        ui.add_space(6.0);
                        theme::separator(ui);

                        for path in &entry.files {
                            let name = path
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("hasil");
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new(shorten(name, 46))
                                        .size(11.5)
                                        .color(theme::text()),
                                );
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    if ui.small_button("Folder").clicked() {
                                        open_folder(path);
                                    }
                                    if ui.small_button("Buka").clicked() {
                                        open_path(path);
                                    }
                                });
                            });
                        }
                    });
                    ui.add_space(12.0);
                }
            });

        if clear {
            self.history.clear();
        }
        if back_home {
            self.tab = Tab::Home;
        }
    }
}

fn time_ago(created: Instant) -> String {
    let secs = created.elapsed().as_secs();
    if secs < 5 {
        "baru saja".to_string()
    } else if secs < 60 {
        format!("{secs} detik lalu")
    } else if secs < 3_600 {
        format!("{} menit lalu", secs / 60)
    } else {
        format!("{} jam lalu", secs / 3_600)
    }
}

fn shorten(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_string();
    }
    let mut out: String = text.chars().take(max.saturating_sub(1)).collect();
    out.push('…');
    out
}
