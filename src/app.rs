use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::time::Duration;

use eframe::egui::{self, Color32, Margin, Vec2};

use crate::convert::ToolStatus;
use crate::image_utils;
use crate::jobs::{Job, Toast};
use crate::model::SheetSettings;
use crate::prefs::Prefs;
use crate::theme;
use crate::ui_convert::ConvertState;
use crate::util::prefs_path;

pub use crate::jobs::{JobResult, JobTag, ToastKind};
pub use crate::photos::PhotoItem;

// ---------------------------------------------------------------------------
// Tab
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tab {
    Sheet,
    Convert,
    Tools,
    Settings,
}

impl Tab {
    pub fn title(self) -> &'static str {
        match self {
            Tab::Sheet => "Cetak Foto",
            Tab::Convert => "Konversi Dokumen",
            Tab::Tools => "Alat & Diagnostik",
            Tab::Settings => "Pengaturan",
        }
    }

    pub fn subtitle(self) -> &'static str {
        match self {
            Tab::Sheet => "Susun pas foto di kertas, atur margin, lalu cetak.",
            Tab::Convert => "PDF, Word, dan gambar",
            Tab::Tools => "Cek kualitas foto, kapasitas kertas, dan printer.",
            Tab::Settings => "Preferensi studio, dependensi, dan informasi aplikasi.",
        }
    }
}

// ---------------------------------------------------------------------------
// App
// ---------------------------------------------------------------------------

pub struct CetakinApp {
    pub tab: Tab,
    pub settings: SheetSettings,
    pub photos: Vec<PhotoItem>,
    pub active_photo: usize,
    pub textures: HashMap<(PathBuf, u16), egui::TextureHandle>,
    pub texture_errors: HashSet<PathBuf>,
    pub preview_zoom: f32,
    pub preview_pan: Vec2,
    pub show_guides: bool,
    pub convert: ConvertState,
    pub tools: ToolStatus,
    pub job: Option<Job>,
    pub toast: Option<Toast>,
    pub island_pinned: bool,
    pub island_hovered: bool,
    pub prefs_path: PathBuf,
    pub studio: String,
    pub printers: Vec<String>,
    pub inspector: Option<PathBuf>,
    pub last_output: Vec<PathBuf>,
    pub confirm_clear: bool,
    pub confirm_reset: bool,
    pub prefs_dirty: bool,
}

impl CetakinApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let prefs_path = prefs_path();
        let prefs: Option<Prefs> = std::fs::read_to_string(&prefs_path)
            .ok()
            .and_then(|raw| serde_json::from_str(&raw).ok());

        let (settings, studio, last_dir, flavor) = match prefs {
            Some(p) => (p.sheet, p.studio, p.last_dir, p.theme),
            None => (
                SheetSettings::default(),
                String::new(),
                None,
                theme::Flavor::default(),
            ),
        };

        // Terapkan palet Catppuccin pilihan pengguna.
        theme::set_flavor(flavor);
        theme::apply(&cc.egui_ctx);

        Self {
            tab: Tab::Sheet,
            settings,
            photos: Vec::new(),
            active_photo: 0,
            textures: HashMap::new(),
            texture_errors: HashSet::new(),
            preview_zoom: 1.0,
            preview_pan: Vec2::ZERO,
            show_guides: true,
            convert: ConvertState::new(last_dir),
            tools: ToolStatus::detect(),
            job: None,
            toast: None,
            island_pinned: false,
            island_hovered: false,
            prefs_path,
            studio,
            printers: Vec::new(),
            inspector: None,
            last_output: Vec::new(),
            confirm_clear: false,
            confirm_reset: false,
            prefs_dirty: false,
        }
    }

    // -------------------------------------------------------------- input

    fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        let (open, save, print, escape) = ctx.input(|i| {
            let cmd = i.modifiers.command;
            (
                cmd && i.key_pressed(egui::Key::O),
                cmd && i.key_pressed(egui::Key::S),
                cmd && i.key_pressed(egui::Key::P),
                i.key_pressed(egui::Key::Escape),
            )
        });
        if escape {
            self.confirm_clear = false;
            self.confirm_reset = false;
        }
        if open && self.tab != Tab::Convert {
            self.import_photos_dialog();
        }
        if save && self.tab == Tab::Sheet {
            self.export_pdf_dialog();
        }
        if print && self.tab == Tab::Sheet {
            self.print_now();
        }
    }

    fn handle_drops(&mut self, ctx: &egui::Context) {
        let files: Vec<PathBuf> = ctx.input(|i| {
            i.raw
                .dropped_files
                .iter()
                .map(|f| f.path().to_path_buf())
                .collect()
        });
        if files.is_empty() {
            return;
        }
        let images: Vec<PathBuf> = files
            .iter()
            .filter(|p| image_utils::is_supported_image(p))
            .cloned()
            .collect();
        let docs: Vec<PathBuf> = files
            .iter()
            .filter(|p| !image_utils::is_supported_image(p))
            .cloned()
            .collect();

        match self.tab {
            Tab::Sheet => {
                if !images.is_empty() {
                    self.add_photos(images);
                }
                if !docs.is_empty() {
                    self.convert.add_files(docs);
                    self.toast_info(
                        "File dokumen diterima",
                        "Buka tab Konversi untuk memprosesnya.",
                    );
                }
            }
            Tab::Convert => {
                self.convert.add_files(files);
                self.toast_info("File ditambahkan", "Pilih operasi lalu jalankan konversi.");
            }
            Tab::Tools => {
                if let Some(img) = images.first() {
                    self.inspector = Some(img.clone());
                    self.toast_info("Foto dipilih", "Lihat hasil pemeriksaan di bawah.");
                } else if !docs.is_empty() {
                    self.convert.add_files(docs);
                    self.toast_info(
                        "Dokumen ditambahkan",
                        "Buka tab Konversi untuk memprosesnya.",
                    );
                }
            }
            Tab::Settings => {
                if !images.is_empty() {
                    self.add_photos(images);
                    self.tab = Tab::Sheet;
                }
            }
        }
    }
}

impl eframe::App for CetakinApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.poll_job(&ctx);
        self.handle_shortcuts(&ctx);
        self.handle_drops(&ctx);

        theme::paint_background(ui);

        egui::Panel::left("cetakin-sidebar")
            .exact_size(260.0)
            .resizable(false)
            .frame(
                egui::Frame::new()
                    .fill(Color32::TRANSPARENT)
                    .inner_margin(Margin {
                        left: 16,
                        right: 4,
                        top: 16,
                        bottom: 16,
                    }),
            )
            .show(ui, |ui| self.sidebar(ui));

        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(Color32::TRANSPARENT)
                    .inner_margin(Margin {
                        left: 18,
                        right: 22,
                        top: 16,
                        bottom: 18,
                    }),
            )
            .show(ui, |ui| {
                self.header(ui);
                if self.tab == Tab::Sheet {
                    self.hero(ui);
                    ui.add_space(16.0);
                }
                match self.tab {
                    Tab::Sheet => self.sheet_tab(ui),
                    Tab::Convert => self.convert_tab(ui),
                    Tab::Tools => self.tools_tab(ui),
                    Tab::Settings => self.settings_tab(ui),
                }
            });

        self.island_ui(&ctx);

        if ctx.input(|i| i.viewport().close_requested()) {
            self.save_prefs();
        }
        if self.prefs_dirty {
            ctx.request_repaint_after(Duration::from_secs(10));
        }
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        theme::bg().to_normalized_gamma_f32()
    }
}
