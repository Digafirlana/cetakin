use std::path::{Path, PathBuf};

use eframe::egui::{
    self, Align, Align2, Color32, CornerRadius, FontId, Layout, Margin, RichText, ScrollArea,
    Sense, Stroke, StrokeKind, Vec2,
};

use crate::app::{
    card_title, cover_uv, drag_cm, label_value, safe_slug, temp_pdf, CetakinApp, JobResult, JobTag,
};
use crate::convert;
use crate::image_utils;
use crate::layout::{self, SheetLayout};
use crate::model::{
    CutStyle, ImageFit, MarginPreset, PageOrientation, PaperPreset, PhotoPreset, PhotoSize,
    SheetMode,
};
use crate::pdf_export;
use crate::theme;

impl CetakinApp {
    // -------------------------------------------------------------- layout

    pub fn sheet_tab(&mut self, ui: &mut egui::Ui) {
        let layout = layout::compute(&self.settings);
        egui::Panel::left("cetakin-sheet-settings")
            .exact_size(362.0)
            .resizable(false)
            .frame(egui::Frame::new().fill(theme::BG).inner_margin(Margin {
                left: 0,
                right: 16,
                top: 0,
                bottom: 0,
            }))
            .show(ui, |ui| {
                ScrollArea::vertical()
                    .id_salt("sheet-settings-scroll")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        self.settings_panel(ui, &layout);
                        ui.add_space(8.0);
                    });
            });
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(theme::BG))
            .show(ui, |ui| self.preview_panel(ui, &layout));
    }

    fn settings_panel(&mut self, ui: &mut egui::Ui, layout: &SheetLayout) {
        let before = self.settings.clone();

        theme::card().show(ui, |ui| {
            card_title(ui, theme::Icon::Page, "Kertas & Orientasi");
            ui.add_space(10.0);

            ui.horizontal(|ui| {
                ui.label(RichText::new("Kertas").size(12.0).color(theme::MUTED));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    egui::ComboBox::from_id_salt("paper-preset")
                        .width(224.0)
                        .selected_text(self.settings.paper.label())
                        .show_ui(ui, |ui| {
                            for preset in PaperPreset::ALL {
                                ui.selectable_value(
                                    &mut self.settings.paper,
                                    preset,
                                    preset.label(),
                                );
                            }
                        });
                });
            });

            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new("Orientasi").size(12.0).color(theme::MUTED));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let mut sel = match self.settings.orientation {
                        PageOrientation::Portrait => 0,
                        PageOrientation::Landscape => 1,
                        PageOrientation::Auto => 2,
                    };
                    theme::segmented_rtl(ui, &mut sel, &["Potret", "Mendatar", "Otomatis"]);
                    if sel
                        != match self.settings.orientation {
                            PageOrientation::Portrait => 0,
                            PageOrientation::Landscape => 1,
                            PageOrientation::Auto => 2,
                        }
                    {
                        self.settings.orientation = match sel {
                            1 => PageOrientation::Landscape,
                            2 => PageOrientation::Auto,
                            _ => PageOrientation::Portrait,
                        };
                    }
                });
            });

            if self.settings.paper == PaperPreset::Custom {
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Lebar").size(12.0).color(theme::MUTED));
                    drag_cm(ui, &mut self.settings.custom_paper_w_mm, 72.0);
                    ui.label(RichText::new("Tinggi").size(12.0).color(theme::MUTED));
                    drag_cm(ui, &mut self.settings.custom_paper_h_mm, 72.0);
                });
            }

            ui.add_space(10.0);
            ui.horizontal(|ui| {
                theme::badge(
                    ui,
                    &format!(
                        "{:.1} × {:.1} cm",
                        layout.page_w_mm / 10.0,
                        layout.page_h_mm / 10.0
                    ),
                    theme::ACCENT,
                );
                if self.settings.orientation == PageOrientation::Auto {
                    theme::badge(ui, "Orientasi dipilih otomatis", theme::BLUE);
                }
            });
        });

        ui.add_space(12.0);

        theme::card().show(ui, |ui| {
            card_title(ui, theme::Icon::Photo, "Ukuran Foto");
            ui.add_space(10.0);

            let mut mode = if self.settings.mode == SheetMode::Single {
                0
            } else {
                1
            };
            theme::segmented(ui, &mut mode, &["Satuan", "Paket campuran"]);
            self.settings.mode = if mode == 0 {
                SheetMode::Single
            } else {
                SheetMode::Packet
            };

            ui.add_space(10.0);
            match self.settings.mode {
                SheetMode::Single => self.single_photo_controls(ui, layout),
                SheetMode::Packet => self.packet_controls(ui, layout),
            }
        });

        ui.add_space(12.0);

        theme::card().show(ui, |ui| {
            card_title(ui, theme::Icon::Tools, "Margin & Potong");
            ui.add_space(10.0);

            ui.horizontal(|ui| {
                ui.label(RichText::new("Margin").size(12.0).color(theme::MUTED));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    egui::ComboBox::from_id_salt("margin-preset")
                        .width(220.0)
                        .selected_text(self.settings.margin_preset.label())
                        .show_ui(ui, |ui| {
                            for preset in MarginPreset::ALL {
                                ui.selectable_value(
                                    &mut self.settings.margin_preset,
                                    preset,
                                    preset.label(),
                                );
                            }
                        });
                });
            });

            if self.settings.margin_preset == MarginPreset::Custom {
                ui.add_space(8.0);
                egui::Grid::new("custom-margins")
                    .num_columns(2)
                    .spacing([10.0, 6.0])
                    .show(ui, |ui| {
                        ui.label(RichText::new("Atas").size(11.5).color(theme::MUTED));
                        drag_cm(ui, &mut self.settings.custom_margins.top_mm, 70.0);
                        ui.end_row();
                        ui.label(RichText::new("Bawah").size(11.5).color(theme::MUTED));
                        drag_cm(ui, &mut self.settings.custom_margins.bottom_mm, 70.0);
                        ui.end_row();
                        ui.label(RichText::new("Kiri").size(11.5).color(theme::MUTED));
                        drag_cm(ui, &mut self.settings.custom_margins.left_mm, 70.0);
                        ui.end_row();
                        ui.label(RichText::new("Kanan").size(11.5).color(theme::MUTED));
                        drag_cm(ui, &mut self.settings.custom_margins.right_mm, 70.0);
                        ui.end_row();
                    });
            }

            ui.add_space(8.0);
            ui.checkbox(
                &mut self.settings.auto_center,
                RichText::new("Pusatkan grid otomatis (seperti Word)").size(12.0),
            );
            ui.add_space(6.0);

            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("Jarak antar foto")
                        .size(12.0)
                        .color(theme::MUTED),
                );
            });
            ui.add(
                egui::Slider::new(&mut self.settings.gap_mm, 0.0..=15.0)
                    .suffix(" mm")
                    .fixed_decimals(1),
            );

            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new("Garis potong").size(12.0).color(theme::MUTED));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    egui::ComboBox::from_id_salt("cut-style")
                        .width(200.0)
                        .selected_text(self.settings.cut_style.label())
                        .show_ui(ui, |ui| {
                            for style in [CutStyle::None, CutStyle::Frame, CutStyle::Guillotine] {
                                ui.selectable_value(
                                    &mut self.settings.cut_style,
                                    style,
                                    style.label(),
                                );
                            }
                        });
                });
            });

            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new("Resolusi").size(12.0).color(theme::MUTED));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    egui::ComboBox::from_id_salt("dpi")
                        .width(110.0)
                        .selected_text(format!("{} DPI", self.settings.dpi))
                        .show_ui(ui, |ui| {
                            for dpi in [150u32, 300, 600] {
                                ui.selectable_value(
                                    &mut self.settings.dpi,
                                    dpi,
                                    format!("{dpi} DPI"),
                                );
                            }
                        });
                });
            });

            ui.add_space(8.0);
            ui.checkbox(
                &mut self.settings.footer_enabled,
                RichText::new("Tulis footer / nama studio").size(12.0),
            );
            if self.settings.footer_enabled {
                ui.add(
                    egui::TextEdit::singleline(&mut self.settings.footer)
                        .hint_text("Contoh: Studio Cahaya · 0812xxxx")
                        .desired_width(f32::INFINITY),
                );
            }

            ui.add_space(10.0);
            ui.label(
                RichText::new(format!("Margin efektif: {}", layout.margin_label()))
                    .size(10.5)
                    .color(theme::DIM),
            );
        });

        ui.add_space(12.0);

        theme::card().show(ui, |ui| {
            card_title(ui, theme::Icon::Check, "Ringkasan Lembar");
            ui.add_space(9.0);
            label_value(ui, "Foto di halaman", &layout.cells.len().to_string());
            label_value(
                ui,
                "Kapasitas maksimum",
                &layout.capacity.max(layout.cells.len()).to_string(),
            );
            label_value(ui, "Halaman PDF", &self.page_count().to_string());
            if layout.overflow > 0 {
                ui.add_space(4.0);
                theme::badge(
                    ui,
                    &format!("{} foto tidak muat", layout.overflow),
                    theme::RED,
                );
            }
            if let Some(dpi) = self.min_dpi(layout) {
                ui.add_space(6.0);
                let (label, level) = image_utils::dpi_quality(dpi);
                let color = match level {
                    0 => theme::ACCENT,
                    1 => theme::YELLOW,
                    2 => theme::ORANGE,
                    _ => theme::RED,
                };
                ui.horizontal(|ui| {
                    theme::badge(ui, &format!("Min. {:.0} DPI · {label}", dpi), color);
                });
            }
            ui.add_space(8.0);
            if self.photos.is_empty() {
                ui.label(
                    RichText::new("Belum ada foto. Impor foto atau jatuhkan ke jendela.")
                        .size(11.0)
                        .color(theme::MUTED),
                );
            } else if self.confirm_clear {
                ui.label(
                    RichText::new("Hapus semua foto dari daftar?")
                        .size(11.5)
                        .color(theme::RED),
                );
                ui.horizontal(|ui| {
                    if theme::danger_button(ui, "Ya, hapus").clicked() {
                        self.clear_photos();
                    }
                    if theme::secondary_button(ui, "Batal").clicked() {
                        self.confirm_clear = false;
                    }
                });
            } else {
                ui.horizontal(|ui| {
                    if theme::secondary_button(ui, "Bersihkan daftar").clicked() {
                        self.confirm_clear = true;
                    }
                    if theme::secondary_button(ui, "Impor foto").clicked() {
                        self.import_photos_dialog();
                    }
                });
            }
        });

        if self.settings != before {
            self.prefs_dirty = true;
        }
    }

    fn single_photo_controls(&mut self, ui: &mut egui::Ui, layout: &SheetLayout) {
        ui.label(RichText::new("Pilih ukuran").size(12.0).color(theme::MUTED));
        ui.add_space(5.0);
        let current = self.settings.photo.preset;
        let mut chosen = None;
        ui.horizontal_wrapped(|ui| {
            for preset in PhotoPreset::ALL {
                if preset_chip(ui, preset.short_label(), current == preset) {
                    chosen = Some(preset);
                }
            }
        });
        if let Some(preset) = chosen {
            self.settings.photo = PhotoSize::from_preset(preset);
        }

        if self.settings.photo.preset == PhotoPreset::Custom {
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new("Lebar").size(11.5).color(theme::MUTED));
                drag_cm(ui, &mut self.settings.photo.custom_w_mm, 68.0);
                ui.label(RichText::new("Tinggi").size(11.5).color(theme::MUTED));
                drag_cm(ui, &mut self.settings.photo.custom_h_mm, 68.0);
            });
        }

        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.checkbox(
                &mut self.settings.photo.landscape,
                RichText::new("Mendatar (tukar lebar × tinggi)").size(11.5),
            );
            if let Some(dpi) = self.min_dpi(layout) {
                let (label, level) = image_utils::dpi_quality(dpi);
                let color = match level {
                    0 => theme::ACCENT,
                    1 => theme::YELLOW,
                    2 => theme::ORANGE,
                    _ => theme::RED,
                };
                theme::badge(ui, &format!("{dpi:.0} DPI · {label}"), color);
            }
        });

        ui.add_space(9.0);
        let mut fill = self.settings.quantity == 0;
        if ui
            .checkbox(
                &mut fill,
                RichText::new("Penuhi halaman otomatis").size(12.0),
            )
            .changed()
        {
            self.settings.quantity = if fill { 0 } else { 4 };
        }
        if self.settings.quantity > 0 {
            ui.horizontal(|ui| {
                ui.label(RichText::new("Jumlah foto").size(11.5).color(theme::MUTED));
                ui.add(
                    egui::DragValue::new(&mut self.settings.quantity)
                        .speed(0.2)
                        .range(1..=999)
                        .suffix(" lembar"),
                );
            });
        }
        if layout.overflow > 0 {
            ui.label(
                RichText::new(format!(
                    "{} foto melebihi kapasitas halaman.",
                    layout.overflow
                ))
                .size(11.0)
                .color(theme::RED),
            );
        }

        ui.add_space(6.0);
        ui.checkbox(
            &mut self.settings.batch_per_photo,
            RichText::new("Satu halaman per foto (batch)").size(12.0),
        );
        ui.label(
            RichText::new("Cocok untuk mencetak banyak pelanggan sekaligus.")
                .size(10.5)
                .color(theme::DIM),
        );

        ui.add_space(7.0);
        ui.horizontal(|ui| {
            ui.label(RichText::new("Isi bingkai").size(12.0).color(theme::MUTED));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                egui::ComboBox::from_id_salt("fit-mode")
                    .width(170.0)
                    .selected_text(self.settings.fit.label())
                    .show_ui(ui, |ui| {
                        for fit in [ImageFit::Cover, ImageFit::Contain] {
                            ui.selectable_value(&mut self.settings.fit, fit, fit.label());
                        }
                    });
            });
        });
    }

    fn packet_controls(&mut self, ui: &mut egui::Ui, layout: &SheetLayout) {
        ui.label(
            RichText::new("Gabungkan beberapa ukuran dalam satu lembar.")
                .size(11.0)
                .color(theme::DIM),
        );
        ui.add_space(6.0);

        let photo_names: Vec<String> = self
            .photos
            .iter()
            .enumerate()
            .map(|(i, p)| format!("{}  ·  {}", i + 1, p.name()))
            .collect();
        let mut remove: Option<usize> = None;
        let count = self.settings.packet.len();
        for i in 0..count {
            egui::Frame::new()
                .fill(theme::INPUT)
                .corner_radius(CornerRadius::same(10))
                .stroke(Stroke::new(1.0, theme::BORDER_SOFT))
                .inner_margin(Margin::symmetric(10, 9))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let item = &mut self.settings.packet[i];
                        egui::ComboBox::from_id_salt(("packet-size", i))
                            .width(132.0)
                            .selected_text(item.size.preset.short_label())
                            .show_ui(ui, |ui| {
                                for preset in PhotoPreset::ALL {
                                    ui.selectable_value(
                                        &mut item.size.preset,
                                        preset,
                                        preset.label(),
                                    );
                                }
                            });
                        ui.add(
                            egui::DragValue::new(&mut item.qty)
                                .speed(0.2)
                                .range(1..=200)
                                .suffix("×"),
                        );
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if theme::icon_button(ui, theme::Icon::Trash, "Hapus ukuran").clicked()
                            {
                                remove = Some(i);
                            }
                        });
                    });
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Foto").size(10.5).color(theme::MUTED));
                        egui::ComboBox::from_id_salt(("packet-photo", i))
                            .width(ui.available_width() - 8.0)
                            .selected_text(
                                self.settings.packet[i]
                                    .photo
                                    .and_then(|p| photo_names.get(p))
                                    .cloned()
                                    .unwrap_or_else(|| "Ikuti foto aktif".to_string()),
                            )
                            .show_ui(ui, |ui| {
                                let item = &mut self.settings.packet[i];
                                ui.selectable_value(&mut item.photo, None, "Ikuti foto aktif");
                                for (pi, name) in photo_names.iter().enumerate() {
                                    ui.selectable_value(&mut item.photo, Some(pi), name);
                                }
                            });
                    });
                });
            ui.add_space(6.0);
        }

        if let Some(index) = remove {
            self.settings.packet.remove(index);
        }

        ui.horizontal(|ui| {
            if theme::secondary_button(ui, "+ Tambah ukuran").clicked() {
                self.settings.packet.push(crate::model::PacketItem {
                    size: PhotoSize::from_preset(PhotoPreset::Cm3x4),
                    qty: 2,
                    photo: None,
                });
            }
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                label_value(
                    ui,
                    "Terpasang",
                    &format!("{} / {}", layout.cells.len(), layout.capacity),
                );
            });
        });
        if layout.overflow > 0 {
            theme::badge(ui, &format!("{} tidak muat", layout.overflow), theme::RED);
        }
    }

    // -------------------------------------------------------------- preview

    fn preview_panel(&mut self, ui: &mut egui::Ui, layout: &SheetLayout) {
        theme::card().show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("Pratinjau")
                        .size(14.0)
                        .strong()
                        .color(theme::TEXT),
                );

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui
                        .add(
                            egui::Button::new(RichText::new("Fit").size(11.5))
                                .min_size(Vec2::new(34.0, 26.0))
                                .corner_radius(CornerRadius::same(7)),
                        )
                        .clicked()
                    {
                        self.preview_zoom = 1.0;
                        self.preview_pan = Vec2::ZERO;
                    }
                    if ui
                        .add(
                            egui::Button::new(RichText::new("+").size(14.0))
                                .min_size(Vec2::new(26.0, 26.0))
                                .corner_radius(CornerRadius::same(7)),
                        )
                        .clicked()
                    {
                        self.preview_zoom = (self.preview_zoom * 1.15).min(3.0);
                    }
                    ui.label(
                        RichText::new(format!("{:.0}%", self.preview_zoom * 100.0))
                            .size(11.0)
                            .color(theme::MUTED),
                    );
                    if ui
                        .add(
                            egui::Button::new(RichText::new("−").size(14.0))
                                .min_size(Vec2::new(26.0, 26.0))
                                .corner_radius(CornerRadius::same(7)),
                        )
                        .clicked()
                    {
                        self.preview_zoom = (self.preview_zoom / 1.15).max(0.6);
                        if self.preview_zoom <= 1.0 {
                            self.preview_pan = Vec2::ZERO;
                        }
                    }
                    ui.checkbox(&mut self.show_guides, RichText::new("Garis").size(11.5));
                });
            });

            ui.add_space(9.0);
            let reserved = 286.0;
            let canvas_h = (ui.available_height() - reserved).clamp(220.0, 560.0);
            let (rect, response) = ui.allocate_exact_size(
                Vec2::new(ui.available_width(), canvas_h),
                Sense::click_and_drag(),
            );
            self.draw_canvas(ui, rect, response, layout);

            ui.add_space(9.0);
            self.stats_strip(ui, layout);
            ui.add_space(8.0);
            self.photo_strip(ui);
        });
    }

    fn draw_canvas(
        &mut self,
        ui: &mut egui::Ui,
        rect: egui::Rect,
        response: egui::Response,
        layout: &SheetLayout,
    ) {
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, CornerRadius::same(14), theme::INPUT);

        // Pola titik halus sebagai latar meja kerja.
        let dot_color = Color32::from_rgba_unmultiplied(255, 255, 255, 10);
        let step = 22.0;
        let mut y = rect.top() + step;
        while y < rect.bottom() {
            let mut x = rect.left() + step;
            while x < rect.right() {
                painter.circle_filled(egui::pos2(x, y), 1.0, dot_color);
                x += step;
            }
            y += step;
        }

        if response.dragged() && self.preview_zoom > 1.01 {
            self.preview_pan += response.drag_delta();
        }
        if response.hovered() {
            let scroll = ui.input(|i| i.smooth_scroll_delta.y);
            if scroll.abs() > 0.5 {
                let old = self.preview_zoom;
                self.preview_zoom = (self.preview_zoom * (1.0 + scroll * 0.0015)).clamp(0.6, 3.0);
                if self.preview_zoom <= 1.0 {
                    self.preview_pan = Vec2::ZERO;
                } else if old <= 1.0 {
                    self.preview_pan = Vec2::ZERO;
                }
            }
        }

        let pw = layout.page_w_mm;
        let ph = layout.page_h_mm;
        let fit = ((rect.width() - 72.0) / pw)
            .min((rect.height() - 66.0) / ph)
            .max(0.3);
        let scale = fit * self.preview_zoom;
        let page_size = Vec2::new(pw * scale, ph * scale);
        let pan = if self.preview_zoom > 1.01 {
            self.preview_pan
        } else {
            Vec2::ZERO
        };
        let page = egui::Rect::from_center_size(rect.center() + pan, page_size);
        let painter = painter.with_clip_rect(rect);

        painter.rect_filled(
            page.translate(Vec2::new(0.0, 9.0)).expand(3.0),
            CornerRadius::same(3),
            Color32::from_black_alpha(110),
        );
        painter.rect_filled(page, CornerRadius::same(2), Color32::WHITE);

        // Label ukuran & halaman di atas kertas.
        let label = format!(
            "{} · {:.1} × {:.1} cm",
            self.settings
                .paper
                .label()
                .split("  ")
                .next()
                .unwrap_or("Kertas"),
            pw / 10.0,
            ph / 10.0
        );
        painter.text(
            page.left_top() + Vec2::new(0.0, -7.0),
            Align2::LEFT_BOTTOM,
            label,
            FontId::proportional(10.5),
            theme::MUTED,
        );
        if self.page_count() > 1 {
            painter.text(
                page.right_top() + Vec2::new(0.0, -7.0),
                Align2::RIGHT_BOTTOM,
                format!(
                    "Halaman {} / {} · {}",
                    self.active_photo + 1,
                    self.page_count(),
                    self.photos
                        .get(self.active_photo)
                        .map(|p| p.name())
                        .unwrap_or_default()
                ),
                FontId::proportional(10.5),
                theme::ACCENT,
            );
        }

        let mm = |x: f32, y: f32| egui::pos2(page.left() + x * scale, page.top() + y * scale);

        // Panduan margin yang bisa dicetak.
        let margins = self.settings.base_margins();
        if margins.horizontal() < pw && margins.vertical() < ph {
            let guide = egui::Rect::from_min_max(
                mm(margins.left_mm, margins.top_mm),
                mm(pw - margins.right_mm, ph - margins.bottom_mm),
            );
            crate::app::dashed_rect(
                &painter,
                guide,
                Stroke::new(1.0, Color32::from_rgba_unmultiplied(120, 128, 140, 150)),
            );
        }

        let slots = self.preview_slots(layout);
        let ctx = ui.ctx().clone();
        for (i, cell) in layout.cells.iter().enumerate() {
            let target = egui::Rect::from_min_max(
                mm(cell.x_mm, cell.y_mm),
                mm(cell.x_mm + cell.w_mm, cell.y_mm + cell.h_mm),
            )
            .shrink(0.4);
            let photo = slots.get(i).and_then(|p| p.as_ref());
            let texture = photo.and_then(|p| self.texture_for_rotated(&ctx, &p.path, p.rotation));
            match texture {
                Some(tex) => {
                    crate::app::paint_photo(&painter, target, &tex, self.settings.fit);
                }
                None => {
                    painter.rect_filled(
                        target,
                        CornerRadius::ZERO,
                        Color32::from_rgb(240, 242, 245),
                    );
                    if target.width() > 48.0 && target.height() > 42.0 {
                        let icon_rect = egui::Rect::from_center_size(
                            target.center() - Vec2::new(0.0, 9.0),
                            Vec2::splat(18.0),
                        );
                        theme::icon(
                            &painter,
                            icon_rect,
                            theme::Icon::Photo,
                            Color32::from_rgb(170, 176, 186),
                            1.4,
                        );
                        painter.text(
                            target.center() + Vec2::new(0.0, 14.0),
                            Align2::CENTER_CENTER,
                            "Belum ada foto",
                            FontId::proportional(9.0),
                            Color32::from_rgb(150, 156, 166),
                        );
                    }
                }
            }
        }

        if self.show_guides {
            let accent = Color32::from_rgba_unmultiplied(47, 227, 111, 175);
            for (x0, y0, x1, y1) in layout::cut_segments(layout, self.settings.cut_style) {
                let a = mm(x0, y0);
                let b = mm(x1, y1);
                if (x0 - x1).abs() < 0.01 {
                    painter.extend(egui::Shape::dashed_line(
                        &[a, b],
                        Stroke::new(1.0, accent),
                        4.0,
                        4.0,
                    ));
                } else {
                    painter.extend(egui::Shape::dashed_line(
                        &[a, b],
                        Stroke::new(1.0, accent),
                        4.0,
                        4.0,
                    ));
                }
            }
        }

        if self.photos.is_empty() {
            let box_rect = egui::Rect::from_center_size(page.center(), Vec2::new(250.0, 94.0));
            painter.rect_filled(
                box_rect,
                CornerRadius::same(12),
                Color32::from_rgba_unmultiplied(24, 26, 30, 235),
            );
            painter.text(
                box_rect.center() - Vec2::new(0.0, 15.0),
                Align2::CENTER_CENTER,
                "Impor foto untuk mulai",
                FontId::proportional(14.0),
                theme::TEXT,
            );
            painter.text(
                box_rect.center() + Vec2::new(0.0, 10.0),
                Align2::CENTER_CENTER,
                "atau jatuhkan file ke jendela ini",
                FontId::proportional(11.0),
                theme::MUTED,
            );
        }
    }

    fn stats_strip(&mut self, ui: &mut egui::Ui, layout: &SheetLayout) {
        egui::Frame::new()
            .fill(theme::INPUT)
            .corner_radius(CornerRadius::same(10))
            .inner_margin(Margin::symmetric(12, 8))
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    stat_text(
                        ui,
                        &format!("{} foto", layout.cells.len()),
                        theme::ACCENT,
                        true,
                    );
                    stat_sep(ui);
                    stat_text(ui, &layout.margin_label(), theme::MUTED, false);
                    stat_sep(ui);
                    stat_text(
                        ui,
                        &format!(
                            "Area {:.1} × {:.1} cm",
                            layout.used_w_mm / 10.0,
                            layout.used_h_mm / 10.0
                        ),
                        theme::MUTED,
                        false,
                    );
                    if let Some(dpi) = self.min_dpi(layout) {
                        let (label, level) = image_utils::dpi_quality(dpi);
                        let color = match level {
                            0 => theme::ACCENT,
                            1 => theme::YELLOW,
                            2 => theme::ORANGE,
                            _ => theme::RED,
                        };
                        stat_sep(ui);
                        stat_text(ui, &format!("{dpi:.0} DPI · {label}"), color, false);
                    }
                    if layout.overflow > 0 {
                        stat_sep(ui);
                        stat_text(
                            ui,
                            &format!("{} tidak muat", layout.overflow),
                            theme::RED,
                            false,
                        );
                    }
                    if self.settings.paper == PaperPreset::Custom {
                        stat_sep(ui);
                        stat_text(ui, "Kertas kustom", theme::BLUE, false);
                    }
                });
                if let Some(warning) = layout.warnings.first() {
                    ui.add_space(2.0);
                    ui.label(RichText::new(warning).size(10.5).color(theme::ORANGE));
                }
            });
    }

    fn photo_strip(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("FOTO").size(10.5).strong().color(theme::DIM));
            theme::badge(ui, &format!("{}", self.photos.len()), theme::MUTED);
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if theme::secondary_button(ui, "+ Impor").clicked() {
                    self.import_photos_dialog();
                }
                if !self.photos.is_empty() && theme::danger_button(ui, "Hapus aktif").clicked() {
                    self.remove_photo(self.active_photo);
                }
                if self.settings.batch_per_photo && self.page_count() > 1 {
                    if ui
                        .add(
                            egui::Button::new(RichText::new("›").size(16.0))
                                .min_size(Vec2::new(26.0, 26.0)),
                        )
                        .clicked()
                    {
                        self.active_photo = (self.active_photo + 1).min(self.photos.len() - 1);
                    }
                    ui.label(
                        RichText::new(format!("{} / {}", self.active_photo + 1, self.photos.len()))
                            .size(11.0)
                            .color(theme::MUTED),
                    );
                    if ui
                        .add(
                            egui::Button::new(RichText::new("‹").size(16.0))
                                .min_size(Vec2::new(26.0, 26.0)),
                        )
                        .clicked()
                    {
                        self.active_photo = self.active_photo.saturating_sub(1);
                    }
                }
            });
        });

        if self.photos.is_empty() {
            let (rect, response) =
                ui.allocate_exact_size(Vec2::new(ui.available_width(), 62.0), Sense::click());
            let painter = ui.painter();
            let stroke = Stroke::new(1.0, theme::BORDER);
            crate::app::dashed_rect(painter, rect.shrink(1.0), stroke);
            painter.text(
                rect.center(),
                Align2::CENTER_CENTER,
                "Jatuhkan foto di sini  ·  JPG, PNG, WEBP, TIFF",
                FontId::proportional(11.0),
                theme::DIM,
            );
            if response.clicked() {
                self.import_photos_dialog();
            }
            return;
        }

        ui.add_space(6.0);
        let current_rotation = self.active_rotation();
        let mut rotate_by = 0;
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("Rotasi foto aktif")
                    .size(10.5)
                    .color(theme::MUTED),
            );
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if compact_button(ui, "+90°") {
                    rotate_by = 90;
                }
                if compact_button(ui, "-90°") {
                    rotate_by = -90;
                }
                theme::badge(ui, &format!("{current_rotation}°"), theme::MUTED);
            });
        });
        if rotate_by != 0 {
            self.rotate_active(rotate_by);
        }

        let items: Vec<(PathBuf, String, u32, u32, u64, u16)> = self
            .photos
            .iter()
            .map(|p| {
                (
                    p.path.clone(),
                    p.name(),
                    p.dims.0,
                    p.dims.1,
                    p.bytes,
                    p.rotation,
                )
            })
            .collect();
        let mut select = None;
        let mut remove = None;

        ScrollArea::horizontal()
            .id_salt("photo-strip")
            .max_height(88.0)
            .auto_shrink([false, true])
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    for (i, (path, name, w, h, bytes, rotation)) in items.iter().enumerate() {
                        let tex = self.texture_for_rotated(ui.ctx(), path, *rotation);
                        let (rect, response) =
                            ui.allocate_exact_size(Vec2::splat(76.0), Sense::click());
                        let active = i == self.active_photo;
                        let border = if active {
                            Stroke::new(2.0, theme::ACCENT)
                        } else if response.hovered() {
                            Stroke::new(1.0, Color32::from_rgb(82, 88, 100))
                        } else {
                            Stroke::new(1.0, theme::BORDER)
                        };
                        ui.painter()
                            .rect_filled(rect, CornerRadius::same(8), theme::INPUT);
                        if let Some(tex) = &tex {
                            let inner = rect.shrink(2.0);
                            let uv = cover_uv(tex.size_vec2(), 1.0);
                            ui.painter().image(tex.id(), inner, uv, Color32::WHITE);
                        } else {
                            theme::icon(
                                ui.painter(),
                                rect.shrink(24.0),
                                theme::Icon::Photo,
                                theme::DIM,
                                1.3,
                            );
                        }
                        ui.painter().rect_stroke(
                            rect,
                            CornerRadius::same(8),
                            border,
                            StrokeKind::Inside,
                        );
                        if active {
                            let badge = egui::Rect::from_min_size(
                                rect.right_top() + Vec2::new(-20.0, 4.0),
                                Vec2::splat(15.0),
                            );
                            ui.painter()
                                .circle_filled(badge.center(), 7.0, theme::ACCENT);
                            theme::icon(
                                ui.painter(),
                                badge,
                                theme::Icon::Check,
                                Color32::BLACK,
                                1.6,
                            );
                        }
                        if *rotation != 0 {
                            let badge = egui::Rect::from_min_size(
                                rect.left_bottom() + Vec2::new(4.0, -20.0),
                                Vec2::new(34.0, 16.0),
                            );
                            ui.painter().rect_filled(
                                badge,
                                CornerRadius::same(4),
                                Color32::from_rgba_unmultiplied(0, 0, 0, 175),
                            );
                            ui.painter().text(
                                badge.center(),
                                Align2::CENTER_CENTER,
                                format!("{rotation}°"),
                                FontId::proportional(9.5),
                                Color32::WHITE,
                            );
                        }
                        let mut tooltip = format!(
                            "{name}\n{w} × {h} px · {}\nKlik untuk memilih · klik kanan untuk menghapus",
                            crate::image_utils::format_bytes(*bytes)
                        );
                        if *rotation != 0 {
                            tooltip.push_str(&format!("\nRotasi {}°", rotation));
                        }
                        let response = response.on_hover_text(tooltip);
                        if response.clicked() {
                            select = Some(i);
                        }
                        if response.secondary_clicked() {
                            remove = Some(i);
                        }
                        if response.hovered() && i == self.active_photo {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                        }
                    }
                    let (rect, response) =
                        ui.allocate_exact_size(Vec2::splat(76.0), Sense::click());
                    crate::app::dashed_rect(
                        ui.painter(),
                        rect.shrink(1.0),
                        Stroke::new(1.0, theme::BORDER),
                    );
                    theme::icon(
                        ui.painter(),
                        egui::Rect::from_center_size(rect.center(), Vec2::splat(18.0)),
                        theme::Icon::Add,
                        theme::MUTED,
                        1.5,
                    );
                    if response.clicked() {
                        self.import_photos_dialog();
                    }
                });
            });

        if let Some(i) = select {
            self.active_photo = i;
        }
        if let Some(i) = remove {
            self.remove_photo(i);
        }
    }

    // -------------------------------------------------------------- actions

    pub fn export_pdf_dialog(&mut self) {
        if self.photos.is_empty() {
            self.toast_error(
                "Belum ada foto",
                "Impor foto terlebih dahulu sebelum menyimpan PDF.",
            );
            return;
        }
        let layout = layout::compute(&self.settings);
        if layout.cells.is_empty() {
            self.toast_error(
                "Tidak ada yang bisa dicetak",
                "Periksa ukuran foto, margin, dan ukuran kertas.",
            );
            return;
        }
        let default_name = format!(
            "cetakin-{}-{}.pdf",
            safe_slug(
                self.settings
                    .paper
                    .label()
                    .split("  ")
                    .next()
                    .unwrap_or("kertas")
            ),
            safe_slug(&self.settings.photo.short_label().to_string())
        );
        let mut dialog = rfd::FileDialog::new()
            .set_title("Simpan lembar foto sebagai PDF")
            .set_file_name(&default_name)
            .add_filter("PDF", &["pdf"]);
        if let Some(dir) = self
            .active_path()
            .and_then(|p| p.parent().map(Path::to_path_buf))
        {
            dialog = dialog.set_directory(dir);
        }
        if let Some(path) = dialog.save_file() {
            self.export_sheet_to(path, JobTag::Sheet);
        }
    }

    pub fn print_now(&mut self) {
        if self.photos.is_empty() {
            self.toast_error("Belum ada foto", "Impor foto terlebih dahulu.");
            return;
        }
        let layout = layout::compute(&self.settings);
        if layout.cells.is_empty() {
            self.toast_error(
                "Tidak ada yang bisa dicetak",
                "Periksa ukuran foto dan margin.",
            );
            return;
        }
        let path = temp_pdf("cetak");
        let settings = self.settings.clone();
        let pages = self.build_pages(&layout);
        let tools = self.tools.clone();
        let print_path = path.clone();
        let page_count = pages.len();
        self.start_job("Menyiapkan cetak", JobTag::Print, move |ctx| {
            let mut cb = {
                let ctx = ctx.clone();
                move |p: f32, m: &str| ctx.progress(p, m)
            };
            pdf_export::export_sheet_pdf(&print_path, &settings, &layout, &pages, &mut cb)?;
            ctx.progress(0.96, "Mengirim ke printer…");
            let note = convert::print_pdf(&print_path, &tools)?;
            Ok(JobResult {
                title: "Lembar siap dicetak".to_string(),
                note: format!("{page_count} halaman. {note}"),
                files: vec![print_path],
            })
        });
    }

    fn export_sheet_to(&mut self, path: PathBuf, tag: JobTag) {
        let layout = layout::compute(&self.settings);
        let pages = self.build_pages(&layout);
        let settings = self.settings.clone();
        let page_count = pages.len();
        let output = path.clone();
        self.start_job("Menyimpan PDF", tag, move |ctx| {
            let mut cb = {
                let ctx = ctx.clone();
                move |p: f32, m: &str| ctx.progress(p, m)
            };
            pdf_export::export_sheet_pdf(&output, &settings, &layout, &pages, &mut cb)?;
            Ok(JobResult {
                title: "PDF berhasil disimpan".to_string(),
                note: format!(
                    "{page_count} halaman · {:.1} MB",
                    std::fs::metadata(&output)
                        .map(|m| m.len() as f64 / 1_048_576.0)
                        .unwrap_or(0.0)
                ),
                files: vec![output],
            })
        });
    }
}

fn stat_text(ui: &mut egui::Ui, text: &str, color: Color32, strong: bool) {
    let mut rich = RichText::new(text).size(11.0).color(color);
    if strong {
        rich = rich.strong();
    }
    ui.label(rich);
}

fn stat_sep(ui: &mut egui::Ui) {
    ui.label(RichText::new("·").size(11.0).color(theme::DIM));
}

fn compact_button(ui: &mut egui::Ui, label: &str) -> bool {
    ui.add(
        egui::Button::new(RichText::new(label).size(11.0).color(theme::TEXT))
            .fill(theme::INPUT)
            .stroke(Stroke::new(1.0, theme::BORDER))
            .corner_radius(CornerRadius::same(8))
            .min_size(Vec2::new(0.0, 26.0)),
    )
    .clicked()
}

fn preset_chip(ui: &mut egui::Ui, label: &str, selected: bool) -> bool {
    ui.add(
        egui::Button::new(RichText::new(label).size(11.5).color(if selected {
            Color32::BLACK
        } else {
            theme::TEXT
        }))
        .fill(if selected {
            theme::ACCENT
        } else {
            theme::INPUT
        })
        .stroke(if selected {
            Stroke::new(1.0, theme::ACCENT)
        } else {
            Stroke::new(1.0, theme::BORDER_SOFT)
        })
        .corner_radius(CornerRadius::same(8))
        .min_size(Vec2::new(57.0, 30.0)),
    )
    .clicked()
}
