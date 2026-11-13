//! Tab Pemindai Dokumen: pilih satu atau beberapa foto, atur 4 sudut tiap
//! dokumen, lalu simpan hasilnya sekaligus.

use std::path::{Path, PathBuf};

use eframe::egui::{
    self, Align, Align2, Color32, CornerRadius, CursorIcon, FontId, Layout, RichText, ScrollArea,
    Sense, Stroke, StrokeKind, Vec2,
};
use image::{DynamicImage, RgbaImage};

use crate::app::{CetakinApp, JobResult, JobTag, Tab};
use crate::image_utils;
use crate::model::{ImageFit, PageOrientation, SheetSettings};
use crate::pdf_export;
use crate::scan::{self, ScanPreset};
use crate::theme;
use crate::ui_common::{card_title, dashed_rect};
use crate::util::{open_folder, open_path, unique_path};

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

pub struct ScanItem {
    /// ID unik untuk nama tekstur; tidak berubah saat daftar diurutkan ulang.
    pub id: u64,
    pub path: PathBuf,
    /// Rotasi pengguna dalam derajat (0/90/180/270), di atas orientasi EXIF.
    pub rotation: u16,
    pub full_dims: (u32, u32),
    /// Pratinjau sumber (sudah diputar), maks. 1400 px.
    pub source: RgbaImage,
    pub source_tex: egui::TextureHandle,
    /// Empat sudut dokumen, ternormalisasi 0..1.
    pub corners: [[f32; 2]; 4],
    pub note: String,
    pub result_tex: Option<egui::TextureHandle>,
    pub result_dims: (u32, u32),
    pub dirty: bool,
}

pub struct ScanState {
    pub items: Vec<ScanItem>,
    pub active: usize,
    /// Preset berlaku untuk semua dokumen dalam daftar.
    pub preset: ScanPreset,
    pub last_saved: Vec<PathBuf>,
    /// Setelah menyimpan, otomatis tambahkan hasil ke Cetak Foto.
    pub add_after_save: bool,
    next_id: u64,
}

impl Default for ScanState {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            active: 0,
            preset: ScanPreset::default(),
            last_saved: Vec::new(),
            add_after_save: false,
            next_id: 1,
        }
    }
}

impl ScanState {
    pub fn active_item(&self) -> Option<&ScanItem> {
        self.items.get(self.active)
    }

    pub fn active_item_mut(&mut self) -> Option<&mut ScanItem> {
        self.items.get_mut(self.active)
    }
}

#[derive(Clone, Copy)]
enum ScanAction {
    Pick,
    PickSingle,
    Rotate,
    AutoDetect,
    Reset,
    RemoveActive,
    Select(usize),
    SaveImages,
    SavePdf,
    AddToPrint,
}

/// Data sumber yang cukup untuk memproses satu dokumen di background job.
#[derive(Clone)]
struct ScanSource {
    path: PathBuf,
    rotation: u16,
    corners: [[f32; 2]; 4],
}

struct ScanImageJob {
    source: ScanSource,
    output: PathBuf,
}

// ---------------------------------------------------------------------------
// Implementasi pada aplikasi
// ---------------------------------------------------------------------------

impl CetakinApp {
    // -------------------------------------------------------------- actions

    pub fn pick_scan_source(&mut self, ctx: &egui::Context) {
        let picked = rfd::FileDialog::new()
            .set_title("Pilih beberapa foto dokumen")
            .add_filter("Gambar", image_utils::SUPPORTED_IMAGE_EXTS)
            .pick_files();
        if let Some(paths) = picked {
            self.load_scan_sources(ctx, paths);
        }
    }

    pub fn pick_scan_source_single(&mut self, ctx: &egui::Context) {
        let picked = rfd::FileDialog::new()
            .set_title("Pilih satu foto dokumen")
            .add_filter("Gambar", image_utils::SUPPORTED_IMAGE_EXTS)
            .pick_file();
        if let Some(path) = picked {
            self.load_scan_sources(ctx, vec![path]);
        }
    }

    pub fn load_scan_source(&mut self, ctx: &egui::Context, path: PathBuf) {
        self.load_scan_sources(ctx, vec![path]);
    }

    pub fn load_scan_sources(&mut self, ctx: &egui::Context, paths: Vec<PathBuf>) {
        let before = self.scan.items.len();
        let mut failed = 0usize;
        let mut first_error = String::new();
        let mut duplicates = 0usize;

        for path in paths {
            if !image_utils::is_supported_image(&path) {
                failed += 1;
                if first_error.is_empty() {
                    first_error = "Gunakan JPG, PNG, WEBP, BMP, GIF, atau TIFF.".to_string();
                }
                continue;
            }
            if self.scan.items.iter().any(|item| item.path == path) {
                duplicates += 1;
                continue;
            }
            match image_utils::load_image(&path) {
                Ok(img) => {
                    let preview = img.thumbnail(1400, 1400).to_rgba8();
                    let (corners, note) = match scan::detect_quad(&preview) {
                        Some(quad) => (
                            quad,
                            "Tepi dokumen terdeteksi otomatis. Geser titik bila perlu.".to_string(),
                        ),
                        None => (
                            scan::full_frame(),
                            "Tepi tidak terdeteksi otomatis. Atur keempat titik sudut manual."
                                .to_string(),
                        ),
                    };
                    let id = self.scan.next_id;
                    self.scan.next_id += 1;
                    let texture = texture_from_rgba(ctx, &format!("scan-source-{id}"), &preview);
                    self.scan.items.push(ScanItem {
                        id,
                        path,
                        rotation: 0,
                        full_dims: (img.width(), img.height()),
                        source: preview,
                        source_tex: texture,
                        corners,
                        note,
                        result_tex: None,
                        result_dims: (0, 0),
                        dirty: true,
                    });
                }
                Err(e) => {
                    failed += 1;
                    if first_error.is_empty() {
                        first_error = e;
                    }
                }
            }
        }

        let added = self.scan.items.len() - before;
        if added > 0 {
            self.scan.active = self.scan.items.len() - 1;
            self.scan.last_saved.clear();
            self.scan.add_after_save = false;
            if added == 1 {
                self.toast_info(
                    "Foto dimuat",
                    "Periksa keempat sudut dokumen, lalu simpan hasilnya.",
                );
            } else {
                self.toast_info(
                    &format!("{added} foto dimuat"),
                    "Periksa sudut tiap dokumen, lalu simpan semua hasilnya.",
                );
            }
        } else if duplicates > 0 && failed == 0 {
            self.toast_info("Foto sudah ada", "Foto yang sama tidak dimuat dua kali.");
        }
        if failed > 0 {
            self.toast_error(
                &format!("{failed} file dilewati"),
                if first_error.is_empty() {
                    "Pastikan format gambarnya didukung."
                } else {
                    &first_error
                },
            );
        }
    }

    fn rotate_scan_source(&mut self, ctx: &egui::Context) {
        let Some(item) = self.scan.active_item_mut() else {
            return;
        };
        let rotated = image_utils::rotate_rgba(item.source.clone(), 90);
        item.rotation = (item.rotation + 90) % 360;
        item.full_dims = (item.full_dims.1, item.full_dims.0);
        item.source_tex = texture_from_rgba(ctx, &format!("scan-source-{}", item.id), &rotated);
        item.source = rotated;

        match scan::detect_quad(&item.source) {
            Some(quad) => {
                item.corners = quad;
                item.note = "Tepi dokumen terdeteksi otomatis. Geser titik bila perlu.".to_string();
            }
            None => {
                item.corners = scan::full_frame();
                item.note =
                    "Tepi tidak terdeteksi otomatis. Atur keempat titik sudut manual.".to_string();
            }
        }
        item.result_tex = None;
        item.dirty = true;
    }

    fn detect_scan_corners(&mut self, announce: bool) {
        let detected = {
            let Some(item) = self.scan.active_item_mut() else {
                return;
            };
            match scan::detect_quad(&item.source) {
                Some(quad) => {
                    item.corners = quad;
                    item.note =
                        "Tepi dokumen terdeteksi otomatis. Geser titik bila perlu.".to_string();
                    Some(true)
                }
                None => {
                    item.corners = scan::full_frame();
                    item.note = "Tepi tidak terdeteksi otomatis. Atur keempat titik sudut manual."
                        .to_string();
                    Some(false)
                }
            }
        };
        if let (true, Some(found)) = (announce, detected) {
            if let Some(item) = self.scan.active_item_mut() {
                item.dirty = true;
            }
            if found {
                self.toast_info("Tepi terdeteksi", "Sudut dokumen sudah dirapikan otomatis.");
            } else {
                self.toast_info(
                    "Tepi tidak terdeteksi",
                    "Atur keempat titik sudut dokumen secara manual.",
                );
            }
        } else if let Some(item) = self.scan.active_item_mut() {
            item.dirty = true;
        }
    }

    fn refresh_scan_preview(&mut self, ctx: &egui::Context) {
        let preset = self.scan.preset;
        let Some(item) = self.scan.active_item_mut() else {
            return;
        };
        if !item.dirty {
            return;
        }
        let result = scan::process(&item.source, &item.corners, preset, 1000);
        item.result_dims = (result.width(), result.height());
        item.result_tex = Some(texture_from_rgba(
            ctx,
            &format!("scan-result-{}", item.id),
            &result,
        ));
        item.dirty = false;
    }

    fn remove_active_scan(&mut self) {
        if self.scan.items.is_empty() {
            return;
        }
        let index = self.scan.active.min(self.scan.items.len() - 1);
        self.scan.items.remove(index);
        self.scan.active = index.min(self.scan.items.len().saturating_sub(1));
        self.scan.last_saved.clear();
        if self.scan.items.is_empty() {
            self.scan.add_after_save = false;
            self.toast_info(
                "Daftar dikosongkan",
                "Pilih foto dokumen baru untuk mulai memindai.",
            );
        }
    }

    pub fn scan_save_image(&mut self) {
        if self.job.is_some() {
            self.scan.add_after_save = false;
            self.toast_info(
                "Masih ada proses berjalan",
                "Tunggu proses selesai sebelum menyimpan hasil.",
            );
            return;
        }
        if self.scan.items.is_empty() {
            self.scan.add_after_save = false;
            self.toast_info("Belum ada foto", "Pilih foto dokumen terlebih dahulu.");
            return;
        }

        let preset = self.scan.preset;
        if self.scan.items.len() == 1 {
            let item = &self.scan.items[0];
            let stem = file_stem(&item.path);
            let mut dialog = rfd::FileDialog::new()
                .set_title("Simpan hasil pindai")
                .set_file_name(format!("{stem}-scan.png"))
                .add_filter("PNG", &["png"])
                .add_filter("JPEG", &["jpg", "jpeg"]);
            if let Some(dir) = item.path.parent() {
                dialog = dialog.set_directory(dir);
            }
            let Some(mut output) = dialog.save_file() else {
                self.scan.add_after_save = false;
                return;
            };
            let ext = output
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_ascii_lowercase();
            if !matches!(ext.as_str(), "png" | "jpg" | "jpeg") {
                output = output.with_extension("png");
            }
            let jobs = vec![ScanImageJob {
                source: scan_source(item),
                output,
            }];
            self.start_scan_image_job(jobs, preset);
            return;
        }

        let Some(dir) = rfd::FileDialog::new()
            .set_title("Pilih folder untuk menyimpan semua hasil pindai")
            .pick_folder()
        else {
            self.scan.add_after_save = false;
            return;
        };
        let mut jobs = Vec::with_capacity(self.scan.items.len());
        for (index, item) in self.scan.items.iter().enumerate() {
            let stem = file_stem(&item.path);
            let name = format!("{stem}-scan-{:02}.png", index + 1);
            jobs.push(ScanImageJob {
                source: scan_source(item),
                output: unique_path(dir.join(name)),
            });
        }
        self.start_scan_image_job(jobs, preset);
    }

    fn start_scan_image_job(&mut self, jobs: Vec<ScanImageJob>, preset: ScanPreset) {
        let total = jobs.len();
        self.start_job("Memindai dokumen", JobTag::Scan, move |job| {
            let mut files = Vec::with_capacity(total);
            let mut first_dims = (0, 0);
            for (index, item) in jobs.iter().enumerate() {
                let base = index as f32 / total as f32;
                let span = 1.0 / total as f32;
                job.progress(
                    base + span * 0.08,
                    &format!("Memuat foto {} dari {}…", index + 1, total),
                );
                job.progress(
                    base + span * 0.3,
                    &format!("Mengoreksi perspektif {} dari {}…", index + 1, total),
                );
                let result = process_scan_source(&item.source, preset)?;

                job.progress(base + span * 0.85, "Menyimpan gambar…");
                image_utils::save_image(
                    &DynamicImage::ImageRgba8(result.clone()),
                    &item.output,
                    92,
                )?;
                if index == 0 {
                    first_dims = (result.width(), result.height());
                }
                files.push(item.output.clone());
            }
            job.progress(1.0, "Selesai");

            let (title, note) = if total == 1 {
                (
                    "Hasil pindai disimpan".to_string(),
                    format!(
                        "{} × {} px · {}",
                        first_dims.0,
                        first_dims.1,
                        preset.label()
                    ),
                )
            } else {
                (
                    format!("{total} hasil pindai disimpan"),
                    format!("{total} dokumen · {}", preset.label()),
                )
            };
            Ok(JobResult { title, note, files })
        });
    }

    pub fn scan_save_pdf(&mut self) {
        if self.job.is_some() {
            self.scan.add_after_save = false;
            self.toast_info(
                "Masih ada proses berjalan",
                "Tunggu proses selesai sebelum menyimpan hasil.",
            );
            return;
        }
        if self.scan.items.is_empty() {
            self.scan.add_after_save = false;
            self.toast_info("Belum ada foto", "Pilih foto dokumen terlebih dahulu.");
            return;
        }

        let total = self.scan.items.len();
        let stem = file_stem(&self.scan.items[0].path);
        let default_name = if total == 1 {
            format!("{stem}-scan.pdf")
        } else {
            format!("pindai-{total}-dokumen.pdf")
        };
        let mut dialog = rfd::FileDialog::new()
            .set_title("Simpan hasil pindai sebagai PDF")
            .set_file_name(default_name)
            .add_filter("PDF", &["pdf"]);
        if let Some(dir) = self.scan.items[0].path.parent() {
            dialog = dialog.set_directory(dir);
        }
        let Some(mut output) = dialog.save_file() else {
            self.scan.add_after_save = false;
            return;
        };
        if output
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| !e.eq_ignore_ascii_case("pdf"))
            .unwrap_or(true)
        {
            output = output.with_extension("pdf");
        }

        let preset = self.scan.preset;
        let sources: Vec<ScanSource> = self.scan.items.iter().map(scan_source).collect();
        self.start_job("Membuat PDF pindaian", JobTag::Scan, move |job| {
            let mut temps = Vec::with_capacity(total);
            let build = (|| -> Result<(), String> {
                for (index, source) in sources.iter().enumerate() {
                    let base = index as f32 / total as f32;
                    let span = 1.0 / total as f32;
                    job.progress(
                        base + span * 0.05,
                        &format!("Memuat foto {} dari {}…", index + 1, total),
                    );
                    job.progress(
                        base + span * 0.25,
                        &format!("Mengoreksi perspektif {} dari {}…", index + 1, total),
                    );
                    let result = process_scan_source(source, preset)?;

                    job.progress(
                        base + span * 0.65,
                        &format!("Menyiapkan halaman {}…", index + 1),
                    );
                    let temp = temp_scan_png(index);
                    image_utils::save_image(&DynamicImage::ImageRgba8(result), &temp, 95)?;
                    temps.push(temp);
                }

                job.progress(0.85, "Menyusun PDF…");
                let settings = SheetSettings {
                    orientation: PageOrientation::Auto,
                    fit: ImageFit::Contain,
                    ..Default::default()
                };
                let mut progress = {
                    let job = job.clone();
                    move |p: f32, m: &str| job.progress(0.85 + p * 0.14, m)
                };
                pdf_export::images_to_pdf(
                    &output,
                    &temps,
                    &settings,
                    ImageFit::Contain,
                    10.0,
                    300,
                    &mut progress,
                )
            })();
            for temp in &temps {
                let _ = std::fs::remove_file(temp);
            }
            build?;
            job.progress(1.0, "Selesai");

            Ok(JobResult {
                title: "PDF pindaian dibuat".to_string(),
                note: if total == 1 {
                    format!("1 halaman · {}", preset.label())
                } else {
                    format!("{total} halaman · {}", preset.label())
                },
                files: vec![output],
            })
        });
    }

    pub fn scan_add_to_print(&mut self) {
        if !self.scan.items.is_empty() {
            let images: Vec<PathBuf> = self
                .scan
                .last_saved
                .iter()
                .filter(|path| path.exists() && image_utils::is_supported_image(path))
                .cloned()
                .collect();
            // Hanya gunakan hasil tersimpan bila lengkap; kalau tidak, simpan
            // ulang semua dokumen (mis. hasil terakhir berupa PDF).
            if images.len() == self.scan.items.len() {
                self.add_photos(images);
                self.tab = Tab::Sheet;
                return;
            }
        }
        if self.scan.items.is_empty() {
            self.toast_info("Belum ada foto", "Pilih foto dokumen terlebih dahulu.");
            return;
        }
        // Simpan dulu, lalu otomatis masukkan ke Cetak Foto setelah selesai.
        self.scan.add_after_save = true;
        self.scan_save_image();
    }

    // -------------------------------------------------------------- tab UI

    pub fn scan_tab(&mut self, ui: &mut egui::Ui) {
        let mut action: Option<ScanAction> = None;
        ScrollArea::vertical()
            .id_salt("scan-scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                self.scan_intro_card(ui, &mut action);
                ui.add_space(12.0);
                if self.scan.items.is_empty() {
                    self.scan_empty_card(ui, &mut action);
                    ui.add_space(12.0);
                } else {
                    if self.scan.items.len() > 1 {
                        self.scan_list_card(ui, &mut action);
                        ui.add_space(12.0);
                    }
                    ui.horizontal_top(|ui| {
                        let left_w = (ui.available_width() * 0.52).max(420.0);
                        ui.vertical(|ui| {
                            ui.set_width(left_w);
                            self.scan_source_card(ui, &mut action);
                        });
                        ui.add_space(12.0);
                        ui.vertical(|ui| self.scan_result_card(ui, &mut action));
                    });
                    ui.add_space(12.0);
                }
                self.scan_tips_card(ui);
                ui.add_space(8.0);
            });

        match action {
            Some(ScanAction::Pick) => self.pick_scan_source(ui.ctx()),
            Some(ScanAction::PickSingle) => self.pick_scan_source_single(ui.ctx()),
            Some(ScanAction::Rotate) => self.rotate_scan_source(ui.ctx()),
            Some(ScanAction::AutoDetect) => self.detect_scan_corners(true),
            Some(ScanAction::Reset) => {
                if let Some(item) = self.scan.active_item_mut() {
                    item.corners = scan::full_frame();
                    item.note = "Sudut direset ke seluruh bingkai.".to_string();
                    item.result_tex = None;
                    item.dirty = true;
                }
            }
            Some(ScanAction::RemoveActive) => self.remove_active_scan(),
            Some(ScanAction::Select(index)) => {
                if index < self.scan.items.len() {
                    self.scan.active = index;
                }
            }
            Some(ScanAction::SaveImages) => self.scan_save_image(),
            Some(ScanAction::SavePdf) => self.scan_save_pdf(),
            Some(ScanAction::AddToPrint) => self.scan_add_to_print(),
            None => {}
        }

        // Perbarui pratinjau hanya setelah interaksi selesai (hemat CPU saat drag).
        let pointer_down = ui.input(|i| i.pointer.any_down());
        let dirty = self.scan.active_item().map(|i| i.dirty).unwrap_or(false);
        if dirty && !pointer_down {
            self.refresh_scan_preview(ui.ctx());
        }
    }

    fn scan_intro_card(&mut self, ui: &mut egui::Ui, action: &mut Option<ScanAction>) {
        theme::card().show(ui, |ui| {
            ui.horizontal(|ui| {
                let (rect, _) = ui.allocate_exact_size(Vec2::splat(46.0), Sense::hover());
                ui.painter()
                    .rect_filled(rect, CornerRadius::same(13), theme::accent());
                theme::icon(
                    ui.painter(),
                    rect.shrink(12.0),
                    theme::Icon::Scan,
                    theme::on_accent(),
                    1.8,
                );
                ui.add_space(4.0);
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new("Pemindai Dokumen")
                            .size(19.0)
                            .strong()
                            .color(theme::text()),
                    );
                    ui.label(
                        RichText::new(
                            "Foto miring jadi scan rapi: koreksi perspektif, putihkan kertas, \
                             dan proses banyak dokumen sekaligus.",
                        )
                        .size(11.0)
                        .color(theme::muted()),
                    );
                });
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let is_empty = self.scan.items.is_empty();
                    let multi_label = if is_empty {
                        "Beberapa Foto"
                    } else {
                        "Tambah Beberapa"
                    };
                    let single_label = if is_empty { "1 Foto" } else { "Tambah 1 Foto" };
                    if theme::primary_button(ui, multi_label).clicked() {
                        *action = Some(ScanAction::Pick);
                    }
                    if theme::secondary_button(ui, single_label).clicked() {
                        *action = Some(ScanAction::PickSingle);
                    }
                });
            });
        });
    }

    fn scan_empty_card(&mut self, ui: &mut egui::Ui, action: &mut Option<ScanAction>) {
        theme::card().show(ui, |ui| {
            let (rect, _) =
                ui.allocate_exact_size(Vec2::new(ui.available_width(), 250.0), Sense::hover());
            let painter = ui.painter_at(rect);
            painter.rect_filled(rect, CornerRadius::same(14), theme::input());
            dashed_rect(
                &painter,
                rect.shrink(8.0),
                Stroke::new(1.2, theme::border()),
            );

            let icon_rect = egui::Rect::from_center_size(
                rect.center() - Vec2::new(0.0, 52.0),
                Vec2::splat(46.0),
            );
            painter.rect_filled(icon_rect, CornerRadius::same(13), theme::accent_soft());
            theme::icon(
                &painter,
                icon_rect.shrink(11.0),
                theme::Icon::Scan,
                theme::accent(),
                1.7,
            );
            painter.text(
                rect.center() - Vec2::new(0.0, 6.0),
                Align2::CENTER_CENTER,
                "Belum ada foto dokumen",
                FontId::proportional(15.0),
                theme::text(),
            );
            painter.text(
                rect.center() + Vec2::new(0.0, 16.0),
                Align2::CENTER_CENTER,
                "Pilih 1 foto atau beberapa foto sekaligus.",
                FontId::proportional(11.0),
                theme::muted(),
            );

            let button_rect = egui::Rect::from_center_size(
                rect.center() + Vec2::new(-82.0, 62.0),
                Vec2::new(140.0, 36.0),
            );
            if ui
                .put(
                    button_rect,
                    egui::Button::new("1 Foto").min_size(button_rect.size()),
                )
                .clicked()
            {
                *action = Some(ScanAction::PickSingle);
            }
            let button_rect2 = egui::Rect::from_center_size(
                rect.center() + Vec2::new(82.0, 62.0),
                Vec2::new(140.0, 36.0),
            );
            if ui
                .put(
                    button_rect2,
                    egui::Button::new("Beberapa Foto").min_size(button_rect2.size()),
                )
                .clicked()
            {
                *action = Some(ScanAction::Pick);
            }
        });
    }

    fn scan_list_card(&mut self, ui: &mut egui::Ui, action: &mut Option<ScanAction>) {
        theme::card().show(ui, |ui| {
            ui.horizontal(|ui| {
                card_title(ui, theme::Icon::Photo, "Daftar Dokumen");
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(
                        RichText::new(format!(
                            "{} foto · aktif {}",
                            self.scan.items.len(),
                            self.scan.active + 1
                        ))
                        .size(10.5)
                        .color(theme::dim()),
                    );
                });
            });
            ui.add_space(8.0);

            let active = self.scan.active;
            ScrollArea::horizontal()
                .id_salt("scan-thumbs")
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        for (index, item) in self.scan.items.iter().enumerate() {
                            let is_active = index == active;
                            let name = item
                                .path
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("dokumen");
                            ui.vertical(|ui| {
                                let size = Vec2::new(112.0, 82.0);
                                let (rect, response) = ui.allocate_exact_size(size, Sense::click());
                                let painter = ui.painter_at(rect);
                                painter.rect_filled(rect, CornerRadius::same(10), theme::canvas());

                                let tex_size = item.source_tex.size_vec2();
                                let scale = ((rect.width() - 8.0) / tex_size.x.max(1.0))
                                    .min((rect.height() - 8.0) / tex_size.y.max(1.0));
                                let image_rect =
                                    egui::Rect::from_center_size(rect.center(), tex_size * scale);
                                painter.image(
                                    item.source_tex.id(),
                                    image_rect,
                                    egui::Rect::from_min_max(
                                        egui::pos2(0.0, 0.0),
                                        egui::pos2(1.0, 1.0),
                                    ),
                                    Color32::WHITE,
                                );

                                painter.rect_stroke(
                                    rect,
                                    CornerRadius::same(10),
                                    if is_active {
                                        Stroke::new(2.0, theme::accent())
                                    } else {
                                        Stroke::new(1.0, theme::border())
                                    },
                                    StrokeKind::Inside,
                                );
                                let badge = egui::Rect::from_min_size(
                                    rect.left_top() + Vec2::new(5.0, 5.0),
                                    Vec2::new(20.0, 16.0),
                                );
                                painter.rect_filled(
                                    badge,
                                    CornerRadius::same(5),
                                    Color32::from_black_alpha(125),
                                );
                                painter.text(
                                    badge.center(),
                                    Align2::CENTER_CENTER,
                                    format!("{}", index + 1),
                                    FontId::proportional(10.0),
                                    Color32::WHITE,
                                );

                                if response.clicked() {
                                    *action = Some(ScanAction::Select(index));
                                }
                                if response.hovered() {
                                    ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
                                }
                                ui.label(RichText::new(truncate(name, 16)).size(9.0).color(
                                    if is_active {
                                        theme::accent()
                                    } else {
                                        theme::dim()
                                    },
                                ));
                            });
                        }
                    });
                });

            ui.add_space(7.0);
            ui.horizontal_wrapped(|ui| {
                if theme::secondary_button(ui, "Tambah Foto").clicked() {
                    *action = Some(ScanAction::Pick);
                }
                if theme::secondary_button(ui, "Hapus Foto Aktif").clicked() {
                    *action = Some(ScanAction::RemoveActive);
                }
            });
        });
    }

    fn scan_source_card(&mut self, ui: &mut egui::Ui, action: &mut Option<ScanAction>) {
        let Some(index) = self
            .scan
            .items
            .get(self.scan.active)
            .map(|_| self.scan.active)
        else {
            return;
        };
        let total = self.scan.items.len();
        let item_id = self.scan.items[index].id;
        theme::card().show(ui, |ui| {
            ui.horizontal(|ui| {
                card_title(ui, theme::Icon::Photo, "Sumber & Sudut");
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let name = self.scan.items[index]
                        .path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("dokumen");
                    ui.label(
                        RichText::new(format!("{} · {}/{}", truncate(name, 26), index + 1, total))
                            .size(10.5)
                            .color(theme::dim()),
                    );
                });
            });
            ui.add_space(9.0);

            let height = 372.0;
            let (rect, _) =
                ui.allocate_exact_size(Vec2::new(ui.available_width(), height), Sense::hover());
            let painter = ui.painter_at(rect);
            painter.rect_filled(rect, CornerRadius::same(14), theme::canvas());

            let mut img_rect = None;
            {
                let item = &self.scan.items[index];
                let size = item.source_tex.size_vec2();
                let scale = ((rect.width() - 24.0) / size.x.max(1.0))
                    .min((rect.height() - 24.0) / size.y.max(1.0));
                let image_rect = egui::Rect::from_center_size(rect.center(), size * scale);
                painter.image(
                    item.source_tex.id(),
                    image_rect,
                    egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                    Color32::WHITE,
                );
                img_rect = Some(image_rect);
            }

            if let Some(image_rect) = img_rect {
                let mut corners = self.scan.items[index].corners;
                let points: Vec<egui::Pos2> = corners
                    .iter()
                    .map(|c| {
                        egui::pos2(
                            image_rect.left() + c[0] * image_rect.width(),
                            image_rect.top() + c[1] * image_rect.height(),
                        )
                    })
                    .collect();

                let fill = Color32::from_rgba_unmultiplied(
                    theme::accent().r(),
                    theme::accent().g(),
                    theme::accent().b(),
                    20,
                );
                painter.add(egui::Shape::convex_polygon(
                    points.clone(),
                    fill,
                    Stroke::NONE,
                ));
                painter.add(egui::Shape::closed_line(
                    points.clone(),
                    Stroke::new(1.6, theme::accent()),
                ));

                let mut changed = false;
                for (i, point) in points.iter().enumerate() {
                    let hit = egui::Rect::from_center_size(*point, Vec2::splat(30.0));
                    let response = ui.interact(
                        hit,
                        ui.scope_id().with(("scan-corner", item_id, i)),
                        Sense::drag(),
                    );
                    if response.dragged() {
                        if let Some(pos) = ui.ctx().pointer_interact_pos() {
                            corners[i] = [
                                ((pos.x - image_rect.left()) / image_rect.width()).clamp(0.0, 1.0),
                                ((pos.y - image_rect.top()) / image_rect.height()).clamp(0.0, 1.0),
                            ];
                            changed = true;
                        }
                    }
                    if response.dragged() {
                        ui.ctx().set_cursor_icon(CursorIcon::Grabbing);
                    } else if response.hovered() {
                        ui.ctx().set_cursor_icon(CursorIcon::Grab);
                    }

                    let active = response.dragged() || response.hovered();
                    painter.circle_filled(
                        *point,
                        if active { 8.0 } else { 6.5 },
                        if active {
                            theme::accent()
                        } else {
                            Color32::WHITE
                        },
                    );
                    painter.circle_stroke(
                        *point,
                        if active { 8.0 } else { 6.5 },
                        Stroke::new(2.0, theme::accent()),
                    );
                    painter.circle_filled(*point, 2.2, theme::on_accent());
                }

                if changed {
                    let item = &mut self.scan.items[index];
                    item.corners = corners;
                    item.result_tex = None;
                    item.dirty = true;
                }
            }

            ui.add_space(9.0);
            ui.horizontal_wrapped(|ui| {
                if theme::secondary_button(ui, "Deteksi Otomatis").clicked() {
                    *action = Some(ScanAction::AutoDetect);
                }
                if theme::secondary_button(ui, "Reset Sudut").clicked() {
                    *action = Some(ScanAction::Reset);
                }
                if theme::secondary_button(ui, "Putar 90°").clicked() {
                    *action = Some(ScanAction::Rotate);
                }
                if theme::secondary_button(ui, "Hapus Foto").clicked() {
                    *action = Some(ScanAction::RemoveActive);
                }
            });
            ui.add_space(6.0);
            let item = &self.scan.items[index];
            ui.label(RichText::new(&item.note).size(10.0).color(theme::dim()));
            ui.label(
                RichText::new(format!(
                    "Sumber {} × {} px · sudut bisa digeser langsung pada gambar",
                    item.full_dims.0, item.full_dims.1
                ))
                .size(10.0)
                .color(theme::dim()),
            );
        });
    }

    fn scan_result_card(&mut self, ui: &mut egui::Ui, action: &mut Option<ScanAction>) {
        let total = self.scan.items.len();
        theme::card().show(ui, |ui| {
            ui.horizontal(|ui| {
                card_title(ui, theme::Icon::Page, "Hasil Pindai");
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if total > 1 {
                        theme::badge(ui, &format!("{total} dokumen"), theme::green());
                    }
                    theme::badge(ui, self.scan.preset.label(), theme::accent());
                });
            });
            ui.add_space(8.0);

            let mut selected = self.scan.preset.index();
            theme::segmented(ui, &mut selected, &["Warna", "Dokumen", "Hitam Putih"]);
            if selected != self.scan.preset.index() {
                self.scan.preset = ScanPreset::from_index(selected);
                for item in &mut self.scan.items {
                    item.result_tex = None;
                    item.dirty = true;
                }
            }
            ui.add_space(4.0);
            ui.label(
                RichText::new(self.scan.preset.hint())
                    .size(10.0)
                    .color(theme::dim()),
            );
            ui.add_space(8.0);

            let active = self.scan.active_item();
            let height = 320.0;
            let (rect, _) =
                ui.allocate_exact_size(Vec2::new(ui.available_width(), height), Sense::hover());
            let painter = ui.painter_at(rect);
            painter.rect_filled(rect, CornerRadius::same(14), theme::canvas());
            if let Some(tex) = active.and_then(|item| item.result_tex.clone()) {
                let size = tex.size_vec2();
                let scale = ((rect.width() - 24.0) / size.x.max(1.0))
                    .min((rect.height() - 24.0) / size.y.max(1.0));
                let image_rect = egui::Rect::from_center_size(rect.center(), size * scale);
                painter.rect_filled(
                    image_rect.expand(2.0),
                    CornerRadius::same(2),
                    Color32::from_black_alpha(if theme::is_dark() { 70 } else { 26 }),
                );
                painter.image(
                    tex.id(),
                    image_rect,
                    egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                    Color32::WHITE,
                );
            } else {
                painter.text(
                    rect.center(),
                    Align2::CENTER_CENTER,
                    "Menyiapkan pratinjau…",
                    FontId::proportional(12.0),
                    theme::muted(),
                );
            }

            ui.add_space(8.0);
            ui.horizontal(|ui| {
                let dims = active.map(|item| item.result_dims).unwrap_or((0, 0));
                ui.label(
                    RichText::new(format!("Pratinjau {} × {} px", dims.0, dims.1))
                        .size(10.5)
                        .color(theme::muted()),
                );
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let rotation = active.map(|item| item.rotation).unwrap_or(0);
                    ui.label(
                        RichText::new(format!("Rotasi {rotation}°"))
                            .size(10.0)
                            .color(theme::dim()),
                    );
                });
            });

            ui.add_space(9.0);
            ui.horizontal_wrapped(|ui| {
                let save_label = if total > 1 {
                    "Simpan Semua Gambar"
                } else {
                    "Simpan Gambar"
                };
                if theme::primary_button(ui, save_label).clicked() {
                    *action = Some(ScanAction::SaveImages);
                }
                if theme::secondary_button(ui, "Simpan PDF").clicked() {
                    *action = Some(ScanAction::SavePdf);
                }
                if theme::secondary_button(ui, "Tambahkan ke Cetak").clicked() {
                    *action = Some(ScanAction::AddToPrint);
                }
            });
            if total > 1 {
                ui.add_space(4.0);
                ui.label(
                    RichText::new("Simpan semua gambar akan meminta satu folder tujuan (PNG).")
                        .size(9.5)
                        .color(theme::dim()),
                );
            }

            if !self.scan.last_saved.is_empty() {
                ui.add_space(8.0);
                theme::separator(ui);
                ui.add_space(4.0);
                let paths = self.scan.last_saved.clone();
                let first = &paths[0];
                ui.horizontal(|ui| {
                    let label = if paths.len() == 1 {
                        let name = first
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("hasil");
                        format!("Tersimpan: {}", truncate(name, 34))
                    } else {
                        format!("{} hasil tersimpan", paths.len())
                    };
                    ui.label(RichText::new(label).size(10.0).color(theme::green()));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui.small_button("Folder").clicked() {
                            if let Some(dir) = first.parent() {
                                open_folder(Path::new(dir));
                            }
                        }
                        if ui.small_button("Buka").clicked() {
                            open_path(first);
                        }
                    });
                });
            }

            ui.add_space(6.0);
            ui.label(
                RichText::new(
                    "Tip: gunakan latar gelap saat memotret agar tepi kertas mudah dideteksi.",
                )
                .size(10.0)
                .color(theme::dim()),
            );
        });
    }

    fn scan_tips_card(&mut self, ui: &mut egui::Ui) {
        theme::card().show(ui, |ui| {
            card_title(ui, theme::Icon::Tools, "Cara Pakai");
            ui.add_space(8.0);
            for tip in [
                "1. Pilih beberapa foto sekaligus; sudut kertas tiap dokumen dideteksi otomatis.",
                "2. Klik thumbnail untuk berpindah dokumen, lalu geser 4 titik bila tepi belum tepat.",
                "3. Pilih preset Warna, Dokumen, atau Hitam Putih untuk semua dokumen.",
                "4. Simpan semua gambar (PNG) atau PDF multi-halaman, lalu tambahkan ke Cetak Foto.",
            ] {
                ui.label(RichText::new(tip).size(11.0).color(theme::muted()));
                ui.add_space(3.0);
            }
        });
    }
}

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

fn scan_source(item: &ScanItem) -> ScanSource {
    ScanSource {
        path: item.path.clone(),
        rotation: item.rotation,
        corners: item.corners,
    }
}

fn process_scan_source(source: &ScanSource, preset: ScanPreset) -> Result<RgbaImage, String> {
    let full = image_utils::load_image(&source.path)?;
    let full = image_utils::rotate_dynamic(&full, source.rotation);
    Ok(scan::process(
        &full.to_rgba8(),
        &source.corners,
        preset,
        4000,
    ))
}

fn file_stem(path: &Path) -> &str {
    path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("dokumen")
}

fn texture_from_rgba(ctx: &egui::Context, name: &str, img: &RgbaImage) -> egui::TextureHandle {
    let color = egui::ColorImage::from_rgba_unmultiplied(
        [img.width() as usize, img.height() as usize],
        img.as_raw(),
    );
    ctx.load_texture(name, color, egui::TextureOptions::LINEAR)
}

fn temp_scan_png(index: usize) -> PathBuf {
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    std::env::temp_dir().join(format!("cetakin-scan-{stamp}-{index}.png"))
}

fn truncate(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_string();
    }
    let mut out: String = text.chars().take(max.saturating_sub(1)).collect();
    out.push('…');
    out
}
