use std::path::{Path, PathBuf};

use eframe::egui::{
    self, Align, Color32, CornerRadius, FontId, Layout, RichText, ScrollArea, Sense, Stroke,
    StrokeKind, Vec2,
};

use crate::app::{CetakinApp, JobResult, JobTag};
use crate::convert::{self, ResizeFormat};
use crate::image_utils::{self, format_bytes};
use crate::model::{ImageFit, PageOrientation, PaperPreset};
use crate::pdf_export;
use crate::theme;
use crate::ui_common::card_title;
use crate::util::{open_folder, open_path, unique_path};

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConvertOp {
    ImagesToPdf,
    PdfToImages,
    PdfToDocx,
    DocxToPdf,
    CombinePdf,
    CompressImages,
}

impl ConvertOp {
    pub const ALL: [ConvertOp; 6] = [
        ConvertOp::ImagesToPdf,
        ConvertOp::PdfToImages,
        ConvertOp::PdfToDocx,
        ConvertOp::DocxToPdf,
        ConvertOp::CombinePdf,
        ConvertOp::CompressImages,
    ];

    pub fn title(self) -> &'static str {
        match self {
            ConvertOp::ImagesToPdf => "Gambar ke PDF",
            ConvertOp::PdfToImages => "PDF ke PNG",
            ConvertOp::PdfToDocx => "PDF ke Word",
            ConvertOp::DocxToPdf => "Word ke PDF",
            ConvertOp::CombinePdf => "Combine Menjadi PDF",
            ConvertOp::CompressImages => "Kompres Gambar",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            ConvertOp::ImagesToPdf => "Satu gambar per halaman",
            ConvertOp::PdfToImages => "Render halaman jadi PNG",
            ConvertOp::PdfToDocx => "Teks + gambar halaman",
            ConvertOp::DocxToPdf => "Via LibreOffice",
            ConvertOp::CombinePdf => "Semua file jadi satu PDF",
            ConvertOp::CompressImages => "Perkecil & ganti format",
        }
    }

    pub fn accepts(self, path: &Path) -> bool {
        match self {
            ConvertOp::ImagesToPdf | ConvertOp::CompressImages => convert::is_image_file(path),
            ConvertOp::PdfToImages | ConvertOp::PdfToDocx => convert::is_pdf_file(path),
            ConvertOp::DocxToPdf => image_utils::is_docx(path),
            ConvertOp::CombinePdf => {
                convert::is_image_file(path)
                    || convert::is_pdf_file(path)
                    || image_utils::is_docx(path)
            }
        }
    }

    pub fn hint(self) -> &'static str {
        match self {
            ConvertOp::ImagesToPdf => "JPG, PNG, WEBP, BMP, TIFF",
            ConvertOp::PdfToImages => "PDF (bisa lebih dari satu)",
            ConvertOp::PdfToDocx => "PDF (bisa lebih dari satu)",
            ConvertOp::DocxToPdf => "DOCX, DOC, ODT, RTF",
            ConvertOp::CombinePdf => "Gambar, PDF, Word — bisa campur",
            ConvertOp::CompressImages => "JPG, PNG, WEBP, BMP, TIFF",
        }
    }
}

pub struct ConvertState {
    pub op: ConvertOp,
    pub files: Vec<PathBuf>,
    pub output_dir: Option<PathBuf>,
    pub pdf_dpi: u32,
    pub include_images: bool,
    pub fit: ImageFit,
    pub margin_mm: f32,
    pub paper: PaperPreset,
    pub orientation: PageOrientation,
    pub use_page_range: bool,
    pub first_page: u32,
    pub last_page: u32,
    pub max_side: u32,
    pub quality: u8,
    pub format: ResizeFormat,
    pub results: Vec<PathBuf>,
    pub note: String,
}

impl ConvertState {
    pub fn new(output_dir: Option<PathBuf>) -> Self {
        Self {
            op: ConvertOp::PdfToDocx,
            files: Vec::new(),
            output_dir,
            pdf_dpi: 300,
            include_images: true,
            fit: ImageFit::Contain,
            margin_mm: 12.7,
            paper: PaperPreset::A4,
            orientation: PageOrientation::Auto,
            use_page_range: false,
            first_page: 1,
            last_page: 1,
            max_side: 2400,
            quality: 85,
            format: ResizeFormat::Jpeg,
            results: Vec::new(),
            note: String::new(),
        }
    }

    pub fn add_files(&mut self, files: Vec<PathBuf>) {
        for path in files {
            if !(convert::is_image_file(&path)
                || convert::is_pdf_file(&path)
                || image_utils::is_docx(&path))
            {
                continue;
            }
            if !self.files.iter().any(|p| p == &path) {
                if self.output_dir.is_none() {
                    self.output_dir = path.parent().map(Path::to_path_buf);
                }
                self.files.push(path);
            }
        }
    }

    pub fn clear(&mut self) {
        self.files.clear();
        self.results.clear();
        self.note.clear();
    }

    pub fn pick_files_dialog(&mut self) {
        let mut exts: Vec<&str> = image_utils::SUPPORTED_IMAGE_EXTS.to_vec();
        exts.extend_from_slice(&["pdf", "docx", "doc", "odt", "rtf"]);
        let files = rfd::FileDialog::new()
            .set_title("Pilih file untuk dikonversi")
            .add_filter("Semua file didukung", &exts)
            .add_filter("PDF", &["pdf"])
            .add_filter("Gambar", image_utils::SUPPORTED_IMAGE_EXTS)
            .add_filter("Word / ODT", &["docx", "doc", "odt", "rtf"])
            .pick_files();
        if let Some(files) = files {
            self.add_files(files);
        }
    }

    pub fn pick_output_dir(&mut self) {
        let mut dialog = rfd::FileDialog::new().set_title("Pilih folder output");
        if let Some(dir) = &self.output_dir {
            dialog = dialog.set_directory(dir);
        }
        if let Some(dir) = dialog.pick_folder() {
            self.output_dir = Some(dir);
        }
    }

    fn effective_output_dir(&self) -> PathBuf {
        self.output_dir
            .clone()
            .or_else(|| {
                self.files
                    .first()
                    .and_then(|p| p.parent().map(Path::to_path_buf))
            })
            .unwrap_or_else(std::env::temp_dir)
    }
}

// ---------------------------------------------------------------------------
// UI
// ---------------------------------------------------------------------------

impl CetakinApp {
    pub fn convert_tab(&mut self, ui: &mut egui::Ui) {
        ScrollArea::vertical()
            .id_salt("convert-scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                theme::card().show(ui, |ui| {
                    card_title(ui, theme::Icon::Convert, "Pilih Operasi");
                    ui.add_space(9.0);
                    let total = ui.available_width();
                    let card_w = ((total - 24.0) / 3.0).max(140.0);
                    for row in ConvertOp::ALL.chunks(3) {
                        ui.horizontal(|ui| {
                            for op in row {
                                if op_card(ui, *op, self.convert.op == *op, card_w) {
                                    self.convert.op = *op;
                                }
                            }
                        });
                        ui.add_space(7.0);
                    }
                    ui.label(
                        RichText::new(format!("Format diterima: {}", self.convert.op.hint()))
                            .size(10.5)
                            .color(theme::dim()),
                    );
                });

                ui.add_space(12.0);
                ui.horizontal_top(|ui| {
                    let left_w = (ui.available_width() * 0.55).max(380.0);
                    ui.vertical(|ui| {
                        ui.set_width(left_w);
                        self.convert_files_card(ui);
                    });
                    ui.add_space(12.0);
                    ui.vertical(|ui| {
                        self.convert_options_card(ui);
                        ui.add_space(12.0);
                        self.convert_output_card(ui);
                    });
                });

                if !self.convert.results.is_empty() {
                    ui.add_space(12.0);
                    self.convert_results_card(ui);
                }
                ui.add_space(8.0);
            });
    }

    fn convert_files_card(&mut self, ui: &mut egui::Ui) {
        theme::card().show(ui, |ui| {
            ui.horizontal(|ui| {
                card_title(ui, theme::Icon::Folder, "File Sumber");
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if !self.convert.files.is_empty()
                        && theme::icon_button(ui, theme::Icon::Trash, "Kosongkan daftar").clicked()
                    {
                        self.convert.clear();
                    }
                    theme::badge(
                        ui,
                        &format!("{} file", self.convert.files.len()),
                        theme::muted(),
                    );
                });
            });
            ui.add_space(8.0);

            if self.convert.files.is_empty() {
                let (rect, response) =
                    ui.allocate_exact_size(Vec2::new(ui.available_width(), 184.0), Sense::click());
                let painter = ui.painter();
                crate::ui_common::dashed_rect(
                    painter,
                    rect.shrink(1.0),
                    Stroke::new(1.0, theme::border()),
                );
                let icon_rect = egui::Rect::from_center_size(
                    rect.center() - Vec2::new(0.0, 22.0),
                    Vec2::splat(30.0),
                );
                theme::icon(
                    painter,
                    icon_rect,
                    theme::Icon::Download,
                    theme::accent(),
                    1.6,
                );
                painter.text(
                    rect.center() + Vec2::new(0.0, 12.0),
                    egui::Align2::CENTER_CENTER,
                    "Jatuhkan file ke jendela",
                    FontId::proportional(13.0),
                    theme::text(),
                );
                painter.text(
                    rect.center() + Vec2::new(0.0, 30.0),
                    egui::Align2::CENTER_CENTER,
                    self.convert.op.hint(),
                    FontId::proportional(10.5),
                    theme::dim(),
                );
                if response.clicked() {
                    self.convert.pick_files_dialog();
                }
                ui.add_space(8.0);
                if theme::primary_button(ui, "+ Tambah File").clicked() {
                    self.convert.pick_files_dialog();
                }
                return;
            }

            let items: Vec<(PathBuf, String, u64, bool)> = self
                .convert
                .files
                .iter()
                .map(|p| {
                    (
                        p.clone(),
                        p.file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("file")
                            .to_string(),
                        std::fs::metadata(p).map(|m| m.len()).unwrap_or(0),
                        self.convert.op.accepts(p),
                    )
                })
                .collect();

            let mut remove: Option<usize> = None;
            let mut move_up: Option<usize> = None;
            let mut move_down: Option<usize> = None;
            ScrollArea::vertical()
                .id_salt("convert-file-list")
                .max_height(300.0)
                .auto_shrink([false, true])
                .show(ui, |ui| {
                    for (i, (path, name, size, accepted)) in items.iter().enumerate() {
                        let (rect, response) = ui.allocate_exact_size(
                            Vec2::new(ui.available_width(), 46.0),
                            Sense::click(),
                        );
                        if response.hovered() {
                            ui.painter().rect_filled(
                                rect,
                                CornerRadius::same(9),
                                theme::card_hover(),
                            );
                        }
                        let icon = if convert::is_pdf_file(path) {
                            theme::Icon::Page
                        } else if convert::is_image_file(path) {
                            theme::Icon::Photo
                        } else {
                            theme::Icon::Folder
                        };
                        theme::icon(
                            ui.painter(),
                            egui::Rect::from_center_size(
                                egui::pos2(rect.left() + 17.0, rect.center().y),
                                Vec2::splat(16.0),
                            ),
                            icon,
                            if *accepted {
                                theme::accent()
                            } else {
                                theme::red()
                            },
                            1.4,
                        );
                        ui.painter().text(
                            egui::pos2(rect.left() + 34.0, rect.center().y - 7.0),
                            egui::Align2::LEFT_CENTER,
                            &shorten(name, 34),
                            FontId::proportional(12.0),
                            theme::text(),
                        );
                        ui.painter().text(
                            egui::pos2(rect.left() + 34.0, rect.center().y + 8.0),
                            egui::Align2::LEFT_CENTER,
                            if *accepted {
                                format_bytes(*size)
                            } else {
                                format!("{} · tidak cocok dengan operasi", format_bytes(*size))
                            },
                            FontId::proportional(9.5),
                            if *accepted {
                                theme::dim()
                            } else {
                                theme::red()
                            },
                        );

                        let controls = egui::Rect::from_min_max(
                            egui::pos2(rect.right() - 108.0, rect.top()),
                            rect.right_bottom(),
                        );
                        let mut child = ui.new_child(
                            egui::UiBuilder::new()
                                .max_rect(controls)
                                .layout(Layout::right_to_left(Align::Center)),
                        );
                        if theme::icon_button(&mut child, theme::Icon::Close, "Hapus").clicked() {
                            remove = Some(i);
                        }
                        if self.convert.op == ConvertOp::CombinePdf {
                            if theme::icon_button(&mut child, theme::Icon::Down, "Turun").clicked()
                            {
                                move_down = Some(i);
                            }
                            if theme::icon_button(&mut child, theme::Icon::Up, "Naik").clicked() {
                                move_up = Some(i);
                            }
                        }
                        if !remove.is_some() {
                            ui.add_space(1.0);
                        }
                    }
                });

            if let Some(i) = remove {
                if i < self.convert.files.len() {
                    self.convert.files.remove(i);
                }
            }
            if let Some(i) = move_up {
                if i > 0 {
                    self.convert.files.swap(i, i - 1);
                }
            }
            if let Some(i) = move_down {
                if i + 1 < self.convert.files.len() {
                    self.convert.files.swap(i, i + 1);
                }
            }

            ui.add_space(6.0);
            ui.horizontal(|ui| {
                if theme::secondary_button(ui, "+ Tambah").clicked() {
                    self.convert.pick_files_dialog();
                }
                ui.label(
                    RichText::new("Gunakan tombol panah untuk mengatur urutan hasil combine.")
                        .size(10.0)
                        .color(theme::dim()),
                );
            });
        });
    }

    fn convert_options_card(&mut self, ui: &mut egui::Ui) {
        theme::card().show(ui, |ui| {
            card_title(ui, theme::Icon::Tools, "Opsi Konversi");
            ui.add_space(10.0);
            match self.convert.op {
                ConvertOp::ImagesToPdf => {
                    self.paper_orientation_controls(ui);
                    ui.add_space(8.0);
                    ui.label(
                        RichText::new("Margin halaman")
                            .size(11.5)
                            .color(theme::muted()),
                    );
                    ui.add(
                        egui::Slider::new(&mut self.convert.margin_mm, 0.0..=40.0)
                            .suffix(" mm")
                            .fixed_decimals(1),
                    );
                    self.fit_dpi_controls(ui);
                }
                ConvertOp::PdfToImages => {
                    self.dpi_segments(ui);
                    ui.add_space(8.0);
                    ui.checkbox(
                        &mut self.convert.use_page_range,
                        RichText::new("Batasi rentang halaman").size(12.0),
                    );
                    if self.convert.use_page_range {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Dari").size(11.0).color(theme::muted()));
                            ui.add(
                                egui::DragValue::new(&mut self.convert.first_page)
                                    .range(1..=9999)
                                    .speed(0.2),
                            );
                            ui.label(RichText::new("sampai").size(11.0).color(theme::muted()));
                            ui.add(
                                egui::DragValue::new(&mut self.convert.last_page)
                                    .range(1..=9999)
                                    .speed(0.2),
                            );
                        });
                    }
                    self.renderer_note(ui);
                }
                ConvertOp::PdfToDocx => {
                    ui.checkbox(
                        &mut self.convert.include_images,
                        RichText::new("Sertakan gambar halaman (visual setia)").size(12.0),
                    );
                    ui.label(
                        RichText::new(
                            "Teks tetap diekstrak sebagai paragraf yang bisa diedit. Gambar halaman membuat hasil lebih mirip aslinya.",
                        )
                        .size(10.5)
                        .color(theme::dim()),
                    );
                    ui.add_space(8.0);
                    self.dpi_segments(ui);
                    self.renderer_note(ui);
                }
                ConvertOp::DocxToPdf => {
                    if self.tools.libreoffice.is_some() {
                        ui.horizontal(|ui| {
                            theme::dot(ui, theme::accent());
                            ui.label(
                                RichText::new("LibreOffice terdeteksi — format asli dipertahankan.")
                                    .size(11.5)
                                    .color(theme::text()),
                            );
                        });
                    } else {
                        ui.horizontal(|ui| {
                            theme::dot(ui, theme::yellow());
                            ui.label(
                                RichText::new("LibreOffice tidak ada — DOCX akan dirender sebagai teks.")
                                    .size(11.5)
                                    .color(theme::yellow()),
                            );
                        });
                        ui.label(
                            RichText::new(
                                "Pasang LibreOffice untuk mempertahankan tabel, gambar, dan layout Word.",
                            )
                            .size(10.5)
                            .color(theme::dim()),
                        );
                    }
                    ui.add_space(8.0);
                    ui.label(
                        RichText::new("PDF ditulis ke folder output dengan nama file yang sama.")
                            .size(10.5)
                            .color(theme::muted()),
                    );
                }
                ConvertOp::CombinePdf => {
                    ui.label(
                        RichText::new(
                            "Semua file pada daftar digabung menjadi satu PDF. Urutan daftar menentukan urutan halaman.",
                        )
                        .size(11.5)
                        .color(theme::muted()),
                    );
                    ui.add_space(6.0);
                    ui.label(
                        RichText::new(
                            "Gambar akan diletakkan utuh di halaman kertas yang dipilih, PDF dipertahankan apa adanya, dan Word/ODT dikonversi lebih dulu.",
                        )
                        .size(10.5)
                        .color(theme::dim()),
                    );
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        theme::badge(
                            ui,
                            &format!("{} file siap", self.convert.files.len()),
                            if self.convert.files.is_empty() {
                                theme::yellow()
                            } else {
                                theme::accent()
                            },
                        );
                    });
                }
                ConvertOp::CompressImages => {
                    ui.label(
                        RichText::new("Sisi terpanjang (px) — 0 = pertahankan ukuran")
                            .size(11.0)
                            .color(theme::muted()),
                    );
                    ui.add(
                        egui::Slider::new(&mut self.convert.max_side, 0..=6000)
                            .suffix(" px"),
                    );
                    ui.add_space(5.0);
                    ui.label(
                        RichText::new("Kualitas JPEG/WebP")
                            .size(11.0)
                            .color(theme::muted()),
                    );
                    ui.add(egui::Slider::new(&mut self.convert.quality, 40..=100).suffix("%"));
                    ui.add_space(6.0);
                    ui.label(RichText::new("Format").size(11.0).color(theme::muted()));
                    let mut fmt = match self.convert.format {
                        ResizeFormat::Jpeg => 0,
                        ResizeFormat::Png => 1,
                        ResizeFormat::WebP => 2,
                    };
                    theme::segmented(ui, &mut fmt, &["JPEG", "PNG", "WebP"]);
                    self.convert.format = match fmt {
                        1 => ResizeFormat::Png,
                        2 => ResizeFormat::WebP,
                        _ => ResizeFormat::Jpeg,
                    };
                }
            }
        });
    }

    fn paper_orientation_controls(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("Kertas").size(11.5).color(theme::muted()));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                egui::ComboBox::from_id_salt("conv-paper")
                    .width(190.0)
                    .selected_text(self.convert.paper.label())
                    .show_ui(ui, |ui| {
                        for preset in PaperPreset::ALL {
                            ui.selectable_value(&mut self.convert.paper, preset, preset.label());
                        }
                    });
            });
        });
        ui.add_space(5.0);
        ui.horizontal(|ui| {
            ui.label(RichText::new("Orientasi").size(11.5).color(theme::muted()));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let mut sel = match self.convert.orientation {
                    PageOrientation::Portrait => 0,
                    PageOrientation::Landscape => 1,
                    PageOrientation::Auto => 2,
                };
                theme::segmented_rtl(ui, &mut sel, &["Potret", "Mendatar", "Auto"]);
                self.convert.orientation = match sel {
                    0 => PageOrientation::Portrait,
                    1 => PageOrientation::Landscape,
                    _ => PageOrientation::Auto,
                };
            });
        });
    }

    fn fit_dpi_controls(&mut self, ui: &mut egui::Ui) {
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("Isi halaman")
                    .size(11.5)
                    .color(theme::muted()),
            );
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                egui::ComboBox::from_id_salt("conv-fit")
                    .width(170.0)
                    .selected_text(self.convert.fit.label())
                    .show_ui(ui, |ui| {
                        for fit in [ImageFit::Cover, ImageFit::Contain] {
                            ui.selectable_value(&mut self.convert.fit, fit, fit.label());
                        }
                    });
            });
        });
        ui.add_space(6.0);
        self.dpi_segments(ui);
    }

    fn dpi_segments(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("Resolusi").size(11.5).color(theme::muted()));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let mut sel = match self.convert.pdf_dpi {
                    0..=180 => 0,
                    181..=400 => 1,
                    _ => 2,
                };
                theme::segmented_rtl(ui, &mut sel, &["150", "300", "600"]);
                self.convert.pdf_dpi = [150, 300, 600][sel];
            });
        });
    }

    fn renderer_note(&self, ui: &mut egui::Ui) {
        ui.add_space(6.0);
        if self.tools.pdf_renderer().is_some() {
            ui.horizontal(|ui| {
                theme::dot(ui, theme::accent());
                ui.label(
                    RichText::new("Renderer PDF siap (pdftoppm/mutool).")
                        .size(10.5)
                        .color(theme::muted()),
                );
            });
        } else {
            ui.horizontal(|ui| {
                theme::dot(ui, theme::yellow());
                ui.label(
                    RichText::new(
                        "Renderer PDF belum ada. Pasang Poppler (pdftoppm) atau MuPDF (mutool).",
                    )
                    .size(10.5)
                    .color(theme::yellow()),
                );
            });
        }
    }

    fn convert_output_card(&mut self, ui: &mut egui::Ui) {
        theme::card().show(ui, |ui| {
            card_title(ui, theme::Icon::Download, "Output");
            ui.add_space(9.0);

            ui.horizontal(|ui| {
                ui.label(RichText::new("Folder").size(11.0).color(theme::muted()));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui.small_button("Ubah").clicked() {
                        self.convert.pick_output_dir();
                    }
                    if ui.small_button("Buka").clicked() {
                        open_folder(&self.convert.effective_output_dir());
                    }
                });
            });
            ui.label(
                RichText::new(shorten(
                    &self.convert.effective_output_dir().to_string_lossy(),
                    52,
                ))
                .size(10.5)
                .color(theme::text()),
            );

            ui.add_space(8.0);
            ui.label(
                RichText::new(match self.convert.op {
                    ConvertOp::ImagesToPdf | ConvertOp::CombinePdf => {
                        "Anda akan diminta memilih nama file PDF saat menjalankan."
                    }
                    ConvertOp::PdfToDocx => "Nama DOCX mengikuti nama PDF sumber.",
                    ConvertOp::PdfToImages => "PNG ditulis dengan akhiran -hal-01, -hal-02, dst.",
                    ConvertOp::DocxToPdf => "PDF ditulis dengan nama file Word sumber.",
                    ConvertOp::CompressImages => "Hasil diberi akhiran -kompres.",
                })
                .size(10.0)
                .color(theme::dim()),
            );

            ui.add_space(10.0);
            let enabled = !self.convert.files.is_empty() && self.job.is_none();
            if ui
                .add_enabled(enabled, {
                    egui::Button::new(
                        RichText::new(format!("Jalankan {}", self.convert.op.title()))
                            .size(13.5)
                            .strong()
                            .color(theme::on_accent()),
                    )
                    .fill(theme::accent())
                    .stroke(Stroke::new(
                        1.0,
                        Color32::from_rgba_unmultiplied(255, 255, 255, 60),
                    ))
                    .corner_radius(CornerRadius::same(12))
                    .min_size(Vec2::new(ui.available_width(), 40.0))
                })
                .clicked()
            {
                self.run_convert();
            }

            if self.convert.files.is_empty() {
                ui.add_space(4.0);
                ui.label(
                    RichText::new("Tambahkan file sumber terlebih dahulu.")
                        .size(10.0)
                        .color(theme::dim()),
                );
            }
        });
    }

    fn convert_results_card(&mut self, ui: &mut egui::Ui) {
        theme::card().show(ui, |ui| {
            ui.horizontal(|ui| {
                card_title(ui, theme::Icon::Check, "Hasil Terakhir");
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui.small_button("Buka folder").clicked() {
                        if let Some(first) = self.convert.results.first() {
                            open_folder(first);
                        }
                    }
                });
            });
            if !self.convert.note.is_empty() {
                ui.label(
                    RichText::new(&self.convert.note)
                        .size(10.5)
                        .color(theme::muted()),
                );
            }
            ui.add_space(6.0);
            for path in self.convert.results.clone() {
                ui.horizontal(|ui| {
                    theme::icon(
                        ui.painter(),
                        egui::Rect::from_min_size(ui.cursor().min, Vec2::splat(14.0)),
                        if convert::is_image_file(&path) {
                            theme::Icon::Photo
                        } else {
                            theme::Icon::Page
                        },
                        theme::accent(),
                        1.3,
                    );
                    ui.add_space(18.0);
                    ui.label(
                        RichText::new(path.file_name().and_then(|n| n.to_str()).unwrap_or("hasil"))
                            .size(11.5)
                            .color(theme::text()),
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
        });
    }

    // -------------------------------------------------------------- run

    pub fn run_convert(&mut self) {
        if self.job.is_some() {
            self.toast_info("Sedang sibuk", "Tunggu proses yang berjalan selesai.");
            return;
        }
        let files = self.convert.files.clone();
        if files.is_empty() {
            self.toast_error("Belum ada file", "Tambahkan file sumber terlebih dahulu.");
            return;
        }

        let invalid: Vec<String> = files
            .iter()
            .filter(|p| !self.convert.op.accepts(p))
            .map(|p| {
                p.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("file")
                    .to_string()
            })
            .collect();
        if !invalid.is_empty() {
            self.toast_error(
                "File tidak cocok",
                &format!(
                    "Operasi {} tidak menerima: {}",
                    self.convert.op.title(),
                    invalid.join(", ")
                ),
            );
            return;
        }
        let outdir = self.convert.effective_output_dir();
        let op = self.convert.op;
        let dpi = self.convert.pdf_dpi;
        let tools = self.tools.clone();

        match op {
            ConvertOp::ImagesToPdf => {
                let Some(path) = save_pdf_dialog("cetakin-gambar.pdf") else {
                    return;
                };
                let mut settings = self.settings.clone();
                settings.paper = self.convert.paper;
                settings.orientation = self.convert.orientation;
                settings.dpi = dpi;
                let fit = self.convert.fit;
                let margin = self.convert.margin_mm;
                let output = path.clone();
                let count = files.len();
                self.start_job("Membuat PDF", JobTag::Convert, move |ctx| {
                    let mut cb = {
                        let ctx = ctx.clone();
                        move |p: f32, m: &str| ctx.progress(p, m)
                    };
                    pdf_export::images_to_pdf(
                        &output, &files, &settings, fit, margin, dpi, &mut cb,
                    )?;
                    Ok(JobResult {
                        title: "PDF berhasil dibuat".to_string(),
                        note: format!("{count} gambar · {dpi} DPI"),
                        files: vec![output],
                    })
                });
            }
            ConvertOp::PdfToImages => {
                let first = self
                    .convert
                    .use_page_range
                    .then_some(self.convert.first_page);
                let last = self
                    .convert
                    .use_page_range
                    .then_some(self.convert.last_page);
                let output = outdir.clone();
                let count = files.len();
                self.start_job("Merender PDF", JobTag::Convert, move |ctx| {
                    let mut all = Vec::new();
                    let mut notes = Vec::new();
                    for (i, input) in files.iter().enumerate() {
                        let base = i as f32 / count.max(1) as f32;
                        let span = 1.0 / count.max(1) as f32;
                        let mut cb = {
                            let ctx = ctx.clone();
                            move |p: f32, m: &str| ctx.progress(base + p * span, m)
                        };
                        let outcome = convert::pdf_to_images(
                            input, &output, dpi, first, last, &tools, &mut cb,
                        )?;
                        notes.push(outcome.note);
                        all.extend(outcome.files);
                    }
                    Ok(JobResult {
                        title: "PDF berhasil dirender".to_string(),
                        note: format!(
                            "{} file PNG. {}",
                            all.len(),
                            notes.first().cloned().unwrap_or_default()
                        ),
                        files: all,
                    })
                });
            }
            ConvertOp::PdfToDocx => {
                let output = outdir.clone();
                let include_images = self.convert.include_images;
                let count = files.len();
                self.start_job("Mengubah PDF ke Word", JobTag::Convert, move |ctx| {
                    let mut all = Vec::new();
                    let mut notes = Vec::new();
                    for (i, input) in files.iter().enumerate() {
                        let base = i as f32 / count.max(1) as f32;
                        let span = 1.0 / count.max(1) as f32;
                        let mut cb = {
                            let ctx = ctx.clone();
                            move |p: f32, m: &str| ctx.progress(base + p * span, m)
                        };
                        let stem = input
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("dokumen");
                        let dest = unique_path(output.join(format!("{stem}-dari-pdf.docx")));
                        let outcome = convert::pdf_to_docx(
                            input,
                            &dest,
                            include_images,
                            dpi,
                            &tools,
                            &mut cb,
                        )?;
                        notes.push(outcome.note);
                        all.extend(outcome.files);
                    }
                    Ok(JobResult {
                        title: "Word berhasil dibuat".to_string(),
                        note: notes.join(" "),
                        files: all,
                    })
                });
            }
            ConvertOp::DocxToPdf => {
                let output = outdir.clone();
                let count = files.len();
                self.start_job("Mengubah Word ke PDF", JobTag::Convert, move |ctx| {
                    let mut all = Vec::new();
                    let mut notes = Vec::new();
                    let mut errors = Vec::new();
                    for (i, input) in files.iter().enumerate() {
                        let base = i as f32 / count.max(1) as f32;
                        let span = 1.0 / count.max(1) as f32;
                        let mut cb = {
                            let ctx = ctx.clone();
                            move |p: f32, m: &str| ctx.progress(base + p * span, m)
                        };
                        match convert::docx_to_pdf(input, &output, &tools, &mut cb) {
                            Ok(outcome) => {
                                notes.push(outcome.note);
                                all.extend(outcome.files);
                            }
                            Err(e) => errors.push(e),
                        }
                    }
                    if all.is_empty() {
                        return Err(errors.join("\n"));
                    }
                    if !errors.is_empty() {
                        notes.push(format!("{} file gagal.", errors.len()));
                    }
                    Ok(JobResult {
                        title: "Konversi selesai".to_string(),
                        note: notes.join(" "),
                        files: all,
                    })
                });
            }
            ConvertOp::CombinePdf => {
                let Some(path) = save_pdf_dialog("cetakin-combine.pdf") else {
                    return;
                };
                let output = path.clone();
                let settings = self.settings.clone();
                let count = files.len();
                self.start_job("Combine ke PDF", JobTag::Convert, move |ctx| {
                    let mut cb = {
                        let ctx = ctx.clone();
                        move |p: f32, m: &str| ctx.progress(p, m)
                    };
                    let outcome =
                        convert::combine_to_pdf(&files, &output, &settings, dpi, &tools, &mut cb)?;
                    Ok(JobResult {
                        title: "PDF berhasil dibuat".to_string(),
                        note: format!("{count} file · {}", outcome.note),
                        files: outcome.files,
                    })
                });
            }
            ConvertOp::CompressImages => {
                let output = outdir.clone();
                let quality = self.convert.quality;
                let max_side = self.convert.max_side;
                let format = self.convert.format;
                self.start_job("Memproses gambar", JobTag::Convert, move |ctx| {
                    let mut cb = {
                        let ctx = ctx.clone();
                        move |p: f32, m: &str| ctx.progress(p, m)
                    };
                    let outcome = convert::compress_images(
                        &files, &output, max_side, quality, format, &mut cb,
                    )?;
                    Ok(JobResult {
                        title: "Gambar selesai diproses".to_string(),
                        note: outcome.note,
                        files: outcome.files,
                    })
                });
            }
        }
    }
}

fn save_pdf_dialog(default_name: &str) -> Option<PathBuf> {
    rfd::FileDialog::new()
        .set_title("Simpan PDF")
        .set_file_name(default_name)
        .add_filter("PDF", &["pdf"])
        .save_file()
        .map(|p| {
            if p.extension().is_none() {
                p.with_extension("pdf")
            } else {
                p
            }
        })
}

fn shorten(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        text.to_string()
    } else {
        let mut out: String = text.chars().take(max.saturating_sub(1)).collect();
        out.push('…');
        out
    }
}

fn op_card(ui: &mut egui::Ui, op: ConvertOp, selected: bool, width: f32) -> bool {
    let (rect, response) = ui.allocate_exact_size(Vec2::new(width, 68.0), Sense::click());
    let fill = if selected {
        theme::accent_soft()
    } else if response.hovered() {
        theme::card_hover()
    } else {
        theme::input()
    };
    ui.painter().rect_filled(rect, CornerRadius::same(11), fill);
    ui.painter().rect_stroke(
        rect,
        CornerRadius::same(11),
        Stroke::new(
            if selected { 1.4 } else { 1.0 },
            if selected {
                theme::accent()
            } else {
                theme::border_soft()
            },
        ),
        StrokeKind::Inside,
    );
    let icon_rect =
        egui::Rect::from_min_size(rect.left_top() + Vec2::new(12.0, 13.0), Vec2::splat(16.0));
    theme::icon(
        ui.painter(),
        icon_rect,
        match op {
            ConvertOp::ImagesToPdf => theme::Icon::Photo,
            ConvertOp::PdfToImages => theme::Icon::Download,
            ConvertOp::PdfToDocx => theme::Icon::Page,
            ConvertOp::DocxToPdf => theme::Icon::Convert,
            ConvertOp::CombinePdf => theme::Icon::Stack,
            ConvertOp::CompressImages => theme::Icon::Tools,
        },
        if selected {
            theme::accent()
        } else {
            theme::muted()
        },
        1.4,
    );
    ui.painter().text(
        rect.left_top() + Vec2::new(36.0, 18.0),
        egui::Align2::LEFT_CENTER,
        op.title(),
        FontId::proportional(12.0),
        if selected {
            theme::text()
        } else {
            theme::text()
        },
    );
    ui.painter().text(
        rect.left_top() + Vec2::new(12.0, 42.0),
        egui::Align2::LEFT_CENTER,
        shorten(op.description(), 24),
        FontId::proportional(9.5),
        if selected {
            theme::muted()
        } else {
            theme::dim()
        },
    );
    if selected {
        ui.painter().circle_filled(
            rect.right_top() + Vec2::new(-13.0, 13.0),
            3.5,
            theme::accent(),
        );
    }
    response.clicked()
}
