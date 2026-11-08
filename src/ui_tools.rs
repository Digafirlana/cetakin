use std::path::{Path, PathBuf};

use eframe::egui::{
    self, Align, Align2, Color32, CornerRadius, FontId, Layout, Margin, RichText, ScrollArea,
    Sense, Stroke, Vec2,
};

use crate::app::{
    card_title, cover_uv, label_value, open_folder, open_path, CetakinApp, JobResult, JobTag,
};
use crate::convert::{self, ToolStatus};
use crate::image_utils::{self, format_bytes};
use crate::layout;
use crate::model::{
    MarginPreset, PageOrientation, PaperPreset, PhotoPreset, PhotoSize, SheetMode,
};
use crate::theme;

// Kertas & foto yang dipakai pada tabel kapasitas.
const CAPACITY_PAPERS: [PaperPreset; 6] = [
    PaperPreset::A4,
    PaperPreset::F4,
    PaperPreset::A3,
    PaperPreset::Paper4R,
    PaperPreset::Paper5R,
    PaperPreset::Paper6R,
];

const CAPACITY_PHOTOS: [PhotoPreset; 6] = [
    PhotoPreset::Cm2x3,
    PhotoPreset::Cm3x4,
    PhotoPreset::Cm4x6,
    PhotoPreset::R2,
    PhotoPreset::R3,
    PhotoPreset::R4,
];

impl CetakinApp {
    // -------------------------------------------------------------- actions

    pub fn pick_inspector(&mut self) {
        let picked = rfd::FileDialog::new()
            .set_title("Pilih gambar untuk diperiksa")
            .add_filter("Gambar", image_utils::SUPPORTED_IMAGE_EXTS)
            .pick_file();
        if let Some(path) = picked {
            if image_utils::is_supported_image(&path) {
                self.inspector = Some(path);
            } else {
                self.toast_error(
                    "Format tidak didukung",
                    "Gunakan JPG, PNG, WEBP, BMP, GIF, atau TIFF.",
                );
            }
        }
    }

    pub fn refresh_tools(&mut self) {
        self.tools = ToolStatus::detect();
        self.printers = convert::list_printers(&self.tools);
        let ready = [
            self.tools.libreoffice.is_some(),
            self.tools.pdf_renderer().is_some(),
            self.tools.lp.is_some() || cfg!(windows),
        ]
        .iter()
        .filter(|v| **v)
        .count();
        self.toast_info(
            "Diagnostik disegarkan",
            &format!(
                "{ready} dari 3 komponen siap · {} printer terdeteksi",
                self.printers.len()
            ),
        );
    }

    // -------------------------------------------------------------- tools tab

    pub fn tools_tab(&mut self, ui: &mut egui::Ui) {
        if self.inspector.is_none() {
            self.inspector = self.photos.first().map(|p| p.path.clone());
        }
        ScrollArea::vertical()
            .id_salt("tools-scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.horizontal_top(|ui| {
                    let left_w = (ui.available_width() * 0.54).max(400.0);
                    ui.vertical(|ui| {
                        ui.set_width(left_w);
                        self.inspector_card(ui);
                    });
                    ui.add_space(12.0);
                    ui.vertical(|ui| {
                        self.capacity_card(ui);
                    });
                });

                ui.add_space(12.0);
                ui.horizontal_top(|ui| {
                    let left_w = (ui.available_width() * 0.54).max(400.0);
                    ui.vertical(|ui| {
                        ui.set_width(left_w);
                        self.printers_card(ui);
                    });
                    ui.add_space(12.0);
                    ui.vertical(|ui| {
                        self.dependencies_card(ui);
                    });
                });
                ui.add_space(8.0);
            });
    }

    fn inspector_card(&mut self, ui: &mut egui::Ui) {
        theme::card().show(ui, |ui| {
            ui.horizontal(|ui| {
                card_title(ui, theme::Icon::Photo, "Inspektur Kualitas Foto");
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if theme::secondary_button(ui, "Pilih Gambar").clicked() {
                        self.pick_inspector();
                    }
                });
            });
            ui.add_space(10.0);

            let Some(path) = self.inspector.clone() else {
                let (rect, response) = ui.allocate_exact_size(
                    Vec2::new(ui.available_width(), 210.0),
                    Sense::click(),
                );
                crate::app::dashed_rect(
                    ui.painter(),
                    rect.shrink(1.0),
                    Stroke::new(1.0, theme::BORDER),
                );
                let icon_rect = egui::Rect::from_center_size(
                    rect.center() - Vec2::new(0.0, 24.0),
                    Vec2::splat(34.0),
                );
                theme::icon(
                    ui.painter(),
                    icon_rect,
                    theme::Icon::Photo,
                    theme::ACCENT,
                    1.6,
                );
                ui.painter().text(
                    rect.center() + Vec2::new(0.0, 14.0),
                    Align2::CENTER_CENTER,
                    "Belum ada gambar diperiksa",
                    FontId::proportional(13.0),
                    theme::TEXT,
                );
                ui.painter().text(
                    rect.center() + Vec2::new(0.0, 33.0),
                    Align2::CENTER_CENTER,
                    "Klik untuk memilih, atau jatuhkan gambar ke jendela.",
                    FontId::proportional(10.5),
                    theme::DIM,
                );
                if response.clicked() {
                    self.pick_inspector();
                }
                return;
            };

            if !image_utils::is_supported_image(&path) {
                ui.label(
                    RichText::new("File ini bukan gambar yang didukung.")
                        .size(12.0)
                        .color(theme::RED),
                );
                return;
            }

            let texture = self.texture_for(ui.ctx(), &path);
            let dims = image_utils::oriented_dimensions(&path)
                .unwrap_or((0, 0));
            let bytes = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);

            ui.horizontal(|ui| {
                let (rect, _) = ui.allocate_exact_size(Vec2::splat(178.0), Sense::hover());
                ui.painter()
                    .rect_filled(rect, CornerRadius::same(10), theme::INPUT);
                if let Some(tex) = &texture {
                    let inner = rect.shrink(2.0);
                    let uv = cover_uv(tex.size_vec2(), inner.width() / inner.height().max(1.0));
                    ui.painter().image(tex.id(), inner, uv, Color32::WHITE);
                } else {
                    theme::icon(
                        ui.painter(),
                        egui::Rect::from_center_size(rect.center(), Vec2::splat(30.0)),
                        theme::Icon::Photo,
                        theme::DIM,
                        1.4,
                    );
                }

                ui.add_space(4.0);
                ui.vertical(|ui| {
                    ui.set_width(ui.available_width());
                    ui.label(
                        RichText::new(
                            path.file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("gambar"),
                        )
                        .size(13.5)
                        .strong()
                        .color(theme::TEXT),
                    );
                    ui.label(
                        RichText::new(
                            path.parent()
                                .map(|p| p.to_string_lossy().to_string())
                                .unwrap_or_default(),
                        )
                        .size(10.0)
                        .color(theme::DIM),
                    );
                    ui.add_space(7.0);
                    theme::separator(ui);
                    ui.add_space(3.0);
                    label_value(ui, "Dimensi", &format!("{dims:?} px"));
                    label_value(ui, "Ukuran file", &format_bytes(bytes));

                    ui.add_space(7.0);
                    ui.label(
                        RichText::new("PERKIRAAN KETEBALAN CETAK")
                            .size(9.5)
                            .strong()
                            .color(theme::DIM),
                    );
                    ui.add_space(3.0);
                    for (label, w, h) in [
                        ("2×3 cm", 20.0f32, 30.0f32),
                        ("3×4 cm", 30.0, 40.0),
                        ("4×6 cm", 40.0, 60.0),
                    ] {
                        let dpi = image_utils::estimate_dpi(
                            dims.0,
                            dims.1,
                            w,
                            h,
                            crate::model::ImageFit::Cover,
                        );
                        let (quality, level) = image_utils::dpi_quality(dpi);
                        let color = match level {
                            0 => theme::ACCENT,
                            1 => theme::YELLOW,
                            2 => theme::ORANGE,
                            _ => theme::RED,
                        };
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(label).size(10.5).color(theme::MUTED),
                            );
                            ui.add_space(4.0);
                            theme::badge(
                                ui,
                                &format!("{dpi:.0} DPI · {quality}"),
                                color,
                            );
                        });
                    }
                });
            });

            ui.add_space(10.0);
            ui.horizontal(|ui| {
                if theme::primary_button(ui, "Jadikan Foto Aktif").clicked() {
                    if !self.photos.iter().any(|p| p.path == path) {
                        self.add_photos(vec![path.clone()]);
                    }
                    if let Some(index) =
                        self.photos.iter().position(|p| p.path == path)
                    {
                        self.active_photo = index;
                        self.tab = crate::app::Tab::Sheet;
                        self.toast_success(
                            "Foto aktif diperbarui",
                            "Kembali ke lembar cetak untuk mengatur layout.",
                        );
                    }
                }
                if theme::secondary_button(ui, "Buka File").clicked() {
                    open_path(&path);
                }
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let supported = self.photos.iter().any(|p| p.path == path);
                    theme::badge(
                        ui,
                        if supported {
                            "Sudah di daftar cetak"
                        } else {
                            "Belum di daftar cetak"
                        },
                        if supported { theme::ACCENT } else { theme::MUTED },
                    );
                });
            });
        });
    }

    fn capacity_card(&mut self, ui: &mut egui::Ui) {
        theme::card().show(ui, |ui| {
            ui.horizontal(|ui| {
                card_title(ui, theme::Icon::Page, "Kalkulator Kapasitas Kertas");
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if theme::secondary_button(ui, "Terapkan ke Lembar").clicked() {
                        let state = capacity_state(ui);
                        let paper = CAPACITY_PAPERS[state.0.min(CAPACITY_PAPERS.len() - 1)];
                        let photo = CAPACITY_PHOTOS[state.1.min(CAPACITY_PHOTOS.len() - 1)];
                        self.settings.paper = paper;
                        self.settings.photo = PhotoSize::from_preset(photo);
                        self.settings.mode = SheetMode::Single;
                        self.tab = crate::app::Tab::Sheet;
                        self.prefs_dirty = true;
                        self.toast_success(
                            "Pengaturan diterapkan",
                            &format!("{} · {}", paper.label(), photo.label()),
                        );
                    }
                });
            });
            ui.add_space(10.0);

            let mut state = capacity_state(ui);
            ui.horizontal(|ui| {
                ui.label(RichText::new("Kertas").size(11.5).color(theme::MUTED));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    egui::ComboBox::from_id_salt("cap-paper")
                        .width(190.0)
                        .selected_text(CAPACITY_PAPERS[state.0].label())
                        .show_ui(ui, |ui| {
                            for (i, preset) in CAPACITY_PAPERS.iter().enumerate() {
                                ui.selectable_value(&mut state.0, i, preset.label());
                            }
                        });
                });
            });
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new("Foto").size(11.5).color(theme::MUTED));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    egui::ComboBox::from_id_salt("cap-photo")
                        .width(190.0)
                        .selected_text(CAPACITY_PHOTOS[state.1].label())
                        .show_ui(ui, |ui| {
                            for (i, preset) in CAPACITY_PHOTOS.iter().enumerate() {
                                ui.selectable_value(&mut state.1, i, preset.label());
                            }
                        });
                });
            });
            store_capacity_state(ui, state);

            let mut scratch = self.settings.clone();
            scratch.mode = SheetMode::Single;
            scratch.paper = CAPACITY_PAPERS[state.0];
            scratch.photo = PhotoSize::from_preset(CAPACITY_PHOTOS[state.1]);
            scratch.quantity = 0;
            scratch.batch_per_photo = false;
            scratch.auto_center = true;
            let result = layout::compute(&scratch);

            ui.add_space(10.0);
            egui::Frame::new()
                .fill(theme::INPUT)
                .corner_radius(CornerRadius::same(12))
                .stroke(Stroke::new(1.0, theme::BORDER_SOFT))
                .inner_margin(Margin::same(12))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.label(
                                RichText::new(result.capacity.to_string())
                                    .size(30.0)
                                    .strong()
                                    .color(theme::ACCENT),
                            );
                            ui.label(
                                RichText::new("foto per lembar")
                                    .size(10.5)
                                    .color(theme::MUTED),
                            );
                        });
                        ui.add_space(14.0);
                        ui.vertical(|ui| {
                            label_value(
                                ui,
                                "Grid",
                                &format!("{} kolom × {} baris", result.cols, result.rows),
                            );
                            label_value(
                                ui,
                                "Area cetak",
                                &format!(
                                    "{:.1} × {:.1} cm",
                                    (result.page_w_mm - result.margins.horizontal()) / 10.0,
                                    (result.page_h_mm - result.margins.vertical()) / 10.0
                                ),
                            );
                            label_value(
                                ui,
                                "Margin",
                                &scratch.margin_preset.label(),
                            );
                        });
                    });
                });

            ui.add_space(10.0);
            ui.label(
                RichText::new("PERBANDINGAN CEPAT (MARGIN SAAT INI)")
                    .size(9.5)
                    .strong()
                    .color(theme::DIM),
            );
            ui.add_space(4.0);
            let headers: Vec<&str> = CAPACITY_PAPERS
                .iter()
                .map(|p| p.label().split("  ").next().unwrap_or("?"))
                .collect();
            egui::Grid::new("capacity-grid")
                .num_columns(CAPACITY_PAPERS.len() + 1)
                .spacing([8.0, 5.0])
                .show(ui, |ui| {
                    ui.label("");
                    for header in &headers {
                        ui.label(
                            RichText::new(*header)
                                .size(9.5)
                                .strong()
                                .color(theme::MUTED),
                        );
                    }
                    ui.end_row();
                    for photo in CAPACITY_PHOTOS {
                        ui.label(
                            RichText::new(photo.short_label())
                                .size(9.5)
                                .color(theme::MUTED),
                        );
                        let mut best = 0usize;
                        let mut counts = Vec::new();
                        for paper in CAPACITY_PAPERS {
                            let mut probe = scratch.clone();
                            probe.paper = paper;
                            probe.photo = PhotoSize::from_preset(photo);
                            let count = layout::compute(&probe).capacity;
                            best = best.max(count);
                            counts.push(count);
                        }
                        for count in counts {
                            let color = if count == 0 {
                                theme::DIM
                            } else if count == best {
                                theme::ACCENT
                            } else {
                                theme::TEXT
                            };
                            ui.label(
                                RichText::new(count.to_string())
                                    .size(10.5)
                                    .strong()
                                    .color(color),
                            );
                        }
                        ui.end_row();
                    }
                });
        });
    }

    fn printers_card(&mut self, ui: &mut egui::Ui) {
        theme::card().show(ui, |ui| {
            ui.horizontal(|ui| {
                card_title(ui, theme::Icon::Print, "Printer");
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if theme::secondary_button(ui, "Segarkan").clicked() {
                        self.refresh_tools();
                    }
                });
            });
            ui.add_space(9.0);

            if self.printers.is_empty() {
                ui.horizontal(|ui| {
                    theme::dot(ui, theme::YELLOW);
                    ui.label(
                        RichText::new("Belum ada printer terdeteksi.")
                            .size(12.0)
                            .color(theme::TEXT),
                    );
                });
                ui.label(
                    RichText::new(if cfg!(windows) {
                        "Di Windows, pastikan printer terpasang di Settings → Printer & scanner. Lalu tekan Segarkan."
                    } else {
                        "Di Linux, pastikan CUPS aktif (systemctl start cups) dan printer ditambahkan. Lalu tekan Segarkan."
                    })
                    .size(10.5)
                    .color(theme::DIM),
                );
                ui.add_space(8.0);
                ui.label(
                    RichText::new("Cetak tetap bisa dilakukan dengan menyimpan PDF lalu membukanya di penampil PDF.")
                        .size(10.5)
                        .color(theme::MUTED),
                );
            } else {
                for name in self.printers.clone() {
                    egui::Frame::new()
                        .fill(theme::INPUT)
                        .corner_radius(CornerRadius::same(10))
                        .inner_margin(Margin::symmetric(11, 8))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                theme::dot(ui, theme::ACCENT);
                                ui.label(
                                    RichText::new(&name)
                                        .size(12.0)
                                        .color(theme::TEXT),
                                );
                                ui.with_layout(
                                    Layout::right_to_left(Align::Center),
                                    |ui| {
                                        theme::badge(ui, "Siap", theme::ACCENT);
                                    },
                                );
                            });
                        });
                    ui.add_space(5.0);
                }
            }

            ui.add_space(8.0);
            let printable = self
                .last_output
                .iter()
                .find(|p| convert::is_pdf_file(p))
                .cloned();
            ui.horizontal(|ui| {
                let enabled = printable.is_some() && self.job.is_none();
                if ui
                    .add_enabled(
                        enabled,
                        egui::Button::new(
                            RichText::new("Cetak PDF Terakhir")
                                .size(12.0)
                                .color(if enabled { theme::TEXT } else { theme::DIM }),
                        )
                        .fill(theme::INPUT)
                        .stroke(Stroke::new(1.0, theme::BORDER))
                        .corner_radius(CornerRadius::same(10))
                        .min_size(Vec2::new(0.0, 34.0)),
                    )
                    .clicked()
                {
                    if let Some(pdf) = printable.clone() {
                        self.print_last_pdf(pdf);
                    }
                }
                if let Some(pdf) = &printable {
                    ui.label(
                        RichText::new(
                            pdf.file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("dokumen.pdf"),
                        )
                        .size(10.5)
                        .color(theme::MUTED),
                    );
                    if ui.small_button("Buka").clicked() {
                        open_path(pdf);
                    }
                } else {
                    ui.label(
                        RichText::new("Belum ada PDF hasil ekspor.")
                            .size(10.5)
                            .color(theme::DIM),
                    );
                }
            });
        });
    }

    fn print_last_pdf(&mut self, pdf: PathBuf) {
        let tools = self.tools.clone();
        self.start_job("Mengirim ke printer", JobTag::Print, move |ctx| {
            ctx.progress(0.4, "Menghubungi printer…");
            let note = convert::print_pdf(&pdf, &tools)?;
            Ok(JobResult {
                title: "Perintah cetak dikirim".to_string(),
                note,
                files: vec![pdf],
            })
        });
    }

    fn dependencies_card(&mut self, ui: &mut egui::Ui) {
        theme::card().show(ui, |ui| {
            ui.horizontal(|ui| {
                card_title(ui, theme::Icon::Tools, "Komponen Pendukung");
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if theme::secondary_button(ui, "Deteksi Ulang").clicked() {
                        self.refresh_tools();
                    }
                });
            });
            ui.add_space(9.0);

            let rows: [(&str, Option<PathBuf>, &str); 6] = [
                (
                    "LibreOffice",
                    self.tools.libreoffice.clone(),
                    "Word → PDF dengan layout asli",
                ),
                (
                    "Poppler · pdftoppm",
                    self.tools.pdftoppm.clone(),
                    "Render PDF menjadi PNG",
                ),
                (
                    "Poppler · pdftotext",
                    self.tools.pdftotext.clone(),
                    "Ekstraksi teks PDF → Word",
                ),
                (
                    "MuPDF · mutool",
                    self.tools.mutool.clone(),
                    "Alternatif render PDF",
                ),
                (
                    "ImageMagick",
                    self.tools.magick.clone(),
                    "Cadangan konversi gambar",
                ),
                (
                    "CUPS · lp",
                    self.tools.lp.clone(),
                    "Kirim dokumen ke printer",
                ),
            ];

            for (name, found, purpose) in rows {
                let (ready, path_text) = match &found {
                    Some(path) => (true, path.to_string_lossy().to_string()),
                    None => (false, "Tidak ditemukan".to_string()),
                };
                ui.horizontal(|ui| {
                    theme::dot(
                        ui,
                        if ready { theme::ACCENT } else { theme::YELLOW },
                    );
                    ui.vertical(|ui| {
                        ui.label(
                            RichText::new(name)
                                .size(12.0)
                                .strong()
                                .color(theme::TEXT),
                        );
                        ui.label(
                            RichText::new(purpose)
                                .size(9.5)
                                .color(theme::DIM),
                        );
                        ui.label(
                            RichText::new(short_path(&path_text, 46))
                                .size(9.5)
                                .color(if ready { theme::MUTED } else { theme::YELLOW }),
                        );
                    });
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        theme::badge(
                            ui,
                            if ready { "Siap" } else { "Opsional" },
                            if ready { theme::ACCENT } else { theme::YELLOW },
                        );
                    });
                });
                ui.add_space(4.0);
            }

            ui.add_space(7.0);
            ui.label(
                RichText::new(
                    "Tanpa komponen di atas, Cetakin tetap bisa menyusun foto, membuat PDF, dan mengekspor gambar. Konversi Word akan memakai renderer teks bawaan.",
                )
                .size(10.0)
                .color(theme::DIM),
            );
        });
    }

    // -------------------------------------------------------------- settings tab

    pub fn settings_tab(&mut self, ui: &mut egui::Ui) {
        ScrollArea::vertical()
            .id_salt("settings-scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.horizontal_top(|ui| {
                    let left_w = (ui.available_width() * 0.54).max(400.0);
                    ui.vertical(|ui| {
                        ui.set_width(left_w);
                        self.studio_card(ui);
                        ui.add_space(12.0);
                        self.defaults_card(ui);
                    });
                    ui.add_space(12.0);
                    ui.vertical(|ui| {
                        self.config_card(ui);
                        ui.add_space(12.0);
                        self.about_card(ui);
                    });
                });
                ui.add_space(12.0);
                self.help_card(ui);
                ui.add_space(8.0);
            });
    }

    fn studio_card(&mut self, ui: &mut egui::Ui) {
        theme::card().show(ui, |ui| {
            card_title(ui, theme::Icon::Photo, "Identitas Studio");
            ui.add_space(9.0);
            ui.label(
                RichText::new("Nama studio / keterangan footer")
                    .size(11.5)
                    .color(theme::MUTED),
            );
            ui.add(
                egui::TextEdit::singleline(&mut self.studio)
                    .hint_text("Contoh: Studio Cahaya · 0812-3456-7890")
                    .desired_width(f32::INFINITY),
            );
            if self.studio != self.settings.footer || self.settings.footer_enabled {
                self.prefs_dirty = true;
            }
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if theme::secondary_button(ui, "Pakai di Footer").clicked() {
                    self.settings.footer = self.studio.clone();
                    self.settings.footer_enabled = !self.studio.trim().is_empty();
                    self.prefs_dirty = true;
                    self.toast_info(
                        "Footer disiapkan",
                        "Nama studio akan dicetak di setiap lembar.",
                    );
                }
                ui.checkbox(
                    &mut self.settings.footer_enabled,
                    RichText::new("Tampilkan footer").size(12.0),
                );
            });
            ui.add_space(6.0);
            if self.settings.footer_enabled && self.settings.footer.trim().is_empty() {
                ui.label(
                    RichText::new("Footer aktif tetapi teksnya masih kosong.")
                        .size(10.5)
                        .color(theme::YELLOW),
                );
            }
        });
    }

    fn defaults_card(&mut self, ui: &mut egui::Ui) {
        let before = self.settings.clone();
        theme::card().show(ui, |ui| {
            card_title(ui, theme::Icon::Settings, "Bawaan Lembar Cetak");
            ui.add_space(9.0);

            settings_combo(ui, "set-paper", "Kertas", self.settings.paper.label(), |ui| {
                for preset in PaperPreset::ALL {
                    ui.selectable_value(
                        &mut self.settings.paper,
                        preset,
                        preset.label(),
                    );
                }
            });
            settings_combo(
                ui,
                "set-orientation",
                "Orientasi",
                self.settings.orientation.label(),
                |ui| {
                    for preset in PageOrientation::ALL {
                        ui.selectable_value(
                            &mut self.settings.orientation,
                            preset,
                            preset.label(),
                        );
                    }
                },
            );
            settings_combo(
                ui,
                "set-photo",
                "Ukuran foto",
                self.settings.photo.preset.label(),
                |ui| {
                    for preset in PhotoPreset::ALL {
                        ui.selectable_value(
                            &mut self.settings.photo,
                            PhotoSize::from_preset(preset),
                            preset.label(),
                        );
                    }
                },
            );
            settings_combo(
                ui,
                "set-margin",
                "Margin",
                self.settings.margin_preset.label(),
                |ui| {
                    for preset in MarginPreset::ALL {
                        ui.selectable_value(
                            &mut self.settings.margin_preset,
                            preset,
                            preset.label(),
                        );
                    }
                },
            );
            settings_combo(
                ui,
                "set-dpi",
                "Resolusi ekspor",
                &format!("{} DPI", self.settings.dpi),
                |ui| {
                    for dpi in [150u32, 300, 600] {
                        ui.selectable_value(
                            &mut self.settings.dpi,
                            dpi,
                            format!("{dpi} DPI"),
                        );
                    }
                },
            );

            ui.add_space(8.0);
            ui.checkbox(
                &mut self.settings.auto_center,
                RichText::new("Pusatkan grid otomatis").size(12.0),
            );
            ui.checkbox(
                &mut self.settings.batch_per_photo,
                RichText::new("Aktifkan batch satu halaman per foto").size(12.0),
            );

            ui.add_space(10.0);
            ui.horizontal(|ui| {
                if theme::primary_button(ui, "Simpan Preferensi").clicked() {
                    self.save_prefs_manual();
                }
                if !self.confirm_reset {
                    if theme::secondary_button(ui, "Reset ke Bawaan").clicked() {
                        self.confirm_reset = true;
                    }
                }
            });
            if self.confirm_reset {
                ui.add_space(6.0);
                ui.label(
                    RichText::new("Semua pengaturan akan kembali ke bawaan. Lanjutkan?")
                        .size(11.0)
                        .color(theme::RED),
                );
                ui.horizontal(|ui| {
                    if theme::danger_button(ui, "Ya, reset").clicked() {
                        self.reset_settings();
                    }
                    if theme::secondary_button(ui, "Batal").clicked() {
                        self.confirm_reset = false;
                    }
                });
            }
        });
        if self.settings != before {
            self.prefs_dirty = true;
        }
    }

    fn config_card(&mut self, ui: &mut egui::Ui) {
        theme::card().show(ui, |ui| {
            ui.horizontal(|ui| {
                card_title(ui, theme::Icon::Folder, "Penyimpanan");
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if theme::secondary_button(ui, "Buka Folder").clicked() {
                        open_folder(&self.prefs_path);
                    }
                });
            });
            ui.add_space(9.0);
            label_value(
                ui,
                "Status",
                if self.prefs_dirty {
                    "Ada perubahan belum disimpan"
                } else {
                    "Tersimpan"
                },
            );
            ui.label(
                RichText::new(short_path(&self.prefs_path.to_string_lossy(), 52))
                    .size(10.0)
                    .color(theme::MUTED),
            );
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if theme::secondary_button(ui, "Simpan Sekarang").clicked() {
                    self.save_prefs_manual();
                }
                if theme::secondary_button(ui, "Buka File").clicked() {
                    open_path(&self.prefs_path);
                }
            });

            if !self.last_output.is_empty() {
                ui.add_space(10.0);
                theme::separator(ui);
                ui.add_space(5.0);
                ui.label(
                    RichText::new("HASIL TERAKHIR")
                        .size(9.5)
                        .strong()
                        .color(theme::DIM),
                );
                for path in self.last_output.clone().into_iter().take(4) {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(
                                path.file_name()
                                    .and_then(|n| n.to_str())
                                    .unwrap_or("hasil"),
                            )
                            .size(10.5)
                            .color(theme::TEXT),
                        );
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if ui.small_button("Buka").clicked() {
                                open_path(&path);
                            }
                            if ui.small_button("Folder").clicked() {
                                open_folder(&path);
                            }
                        });
                    });
                }
            }
        });
    }

    fn about_card(&mut self, ui: &mut egui::Ui) {
        theme::card().show(ui, |ui| {
            ui.horizontal(|ui| {
                let (rect, _) =
                    ui.allocate_exact_size(Vec2::splat(42.0), Sense::hover());
                ui.painter()
                    .rect_filled(rect, CornerRadius::same(12), theme::ACCENT);
                theme::icon(
                    ui.painter(),
                    rect.shrink(10.0),
                    theme::Icon::Print,
                    Color32::from_rgb(8, 24, 14),
                    1.8,
                );
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new("cetakin.")
                            .size(19.0)
                            .strong()
                            .color(theme::TEXT),
                    );
                    ui.label(
                        RichText::new(format!(
                            "Versi {} · Rust + egui",
                            env!("CARGO_PKG_VERSION")
                        ))
                        .size(10.5)
                        .color(theme::MUTED),
                    );
                });
            });
            ui.add_space(10.0);
            ui.label(
                RichText::new(
                    "Aplikasi desktop untuk menyusun pas foto di kertas, mengekspor PDF siap cetak, dan mengonversi dokumen digital printing.",
                )
                .size(11.5)
                .color(theme::MUTED),
            );
            ui.add_space(8.0);
            label_value(ui, "Lisensi", "MIT");
            label_value(ui, "Platform", "Windows · Linux");
            label_value(ui, "Renderer", "glow / OpenGL");
        });
    }

    fn help_card(&mut self, ui: &mut egui::Ui) {
        theme::card().show(ui, |ui| {
            card_title(ui, theme::Icon::Warning, "Panduan Dependensi");
            ui.add_space(9.0);
            ui.horizontal_wrapped(|ui| {
                theme::badge(ui, "Wajib: tidak ada tambahan", theme::ACCENT);
                theme::badge(ui, "Opsional: LibreOffice", theme::BLUE);
                theme::badge(ui, "Opsional: Poppler", theme::BLUE);
                theme::badge(ui, "Opsional: ImageMagick", theme::BLUE);
            });
            ui.add_space(8.0);
            ui.label(
                RichText::new("Linux (Debian/Ubuntu)")
                    .size(12.0)
                    .strong()
                    .color(theme::TEXT),
            );
            ui.label(
                RichText::new(
                    "sudo apt install libgtk-3-dev poppler-utils libreoffice imagemagick",
                )
                .size(10.5)
                .color(theme::MUTED)
                .monospace(),
            );
            ui.add_space(6.0);
            ui.label(
                RichText::new("Windows")
                    .size(12.0)
                    .strong()
                    .color(theme::TEXT),
            );
            ui.label(
                RichText::new(
                    "Pasang LibreOffice, Poppler for Windows, dan ImageMagick agar konversi memakai mesin asli. Tanpa itu, Cetakin memakai renderer bawaan.",
                )
                .size(10.5)
                .color(theme::MUTED),
            );
            ui.add_space(9.0);
            ui.label(
                RichText::new("SHORTCUT")
                    .size(9.5)
                    .strong()
                    .color(theme::DIM),
            );
            for (keys, action) in [
                ("Ctrl + O", "Impor foto"),
                ("Ctrl + S", "Simpan PDF"),
                ("Ctrl + P", "Cetak"),
                ("Esc", "Batalkan dialog konfirmasi"),
            ] {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(keys)
                            .size(10.5)
                            .monospace()
                            .color(theme::ACCENT),
                    );
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label(
                            RichText::new(action)
                                .size(10.5)
                                .color(theme::MUTED),
                        );
                    });
                });
            }
        });
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn capacity_state(ui: &mut egui::Ui) -> (usize, usize) {
    ui.data(|d| {
        d.get_temp::<(usize, usize)>(egui::Id::unique("cetakin-capacity-state"))
    })
    .map(|(paper, photo)| {
        (
            paper.min(CAPACITY_PAPERS.len() - 1),
            photo.min(CAPACITY_PHOTOS.len() - 1),
        )
    })
    .unwrap_or((0, 1))
}

fn store_capacity_state(ui: &mut egui::Ui, state: (usize, usize)) {
    ui.data_mut(|d| {
        d.insert_temp(egui::Id::unique("cetakin-capacity-state"), state);
    });
}

fn settings_combo(
    ui: &mut egui::Ui,
    id: &str,
    label: &str,
    selected: &str,
    add_contents: impl FnOnce(&mut egui::Ui),
) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(label).size(11.5).color(theme::MUTED));
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            egui::ComboBox::from_id_salt(id)
                .width(210.0)
                .selected_text(selected)
                .show_ui(ui, add_contents);
        });
    });
    ui.add_space(4.0);
}

fn short_path(path: &str, max: usize) -> String {
    if path.chars().count() <= max {
        return path.to_string();
    }
    let tail: String = path
        .chars()
        .rev()
        .take(max.saturating_sub(3))
        .collect::<String>()
        .chars()
        .rev()
        .collect();
    format!("…{tail}")
}

// Keep the import if a future revision needs path references in helpers.
#[allow(dead_code)]
fn _path_hint(path: &Path) -> &Path {
    path
}
