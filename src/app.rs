use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};
use std::time::{Duration, Instant};

use eframe::egui::{
    self, Align, Align2, Color32, CornerRadius, FontId, Id, Layout, Margin, Order, RichText, Sense,
    Stroke, StrokeKind, Vec2,
};
use serde::{Deserialize, Serialize};

use crate::convert::ToolStatus;
use crate::image_utils;
use crate::layout::SheetLayout;
use crate::model::{normalize_rotation, ImageFit, PhotoRef, SheetMode, SheetSettings};
use crate::theme;
use crate::ui_convert::ConvertState;

// ---------------------------------------------------------------------------
// Tab & photo data
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

#[derive(Clone, Debug)]
pub struct PhotoItem {
    pub path: PathBuf,
    pub dims: (u32, u32),
    pub bytes: u64,
    /// Rotasi searah jarum jam dalam derajat: 0, 90, 180, atau 270.
    pub rotation: u16,
}

impl PhotoItem {
    pub fn name(&self) -> String {
        self.path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("foto")
            .to_string()
    }

    pub fn photo_ref(&self) -> PhotoRef {
        PhotoRef::new(self.path.clone(), self.rotation)
    }

    pub fn set_rotation(&mut self, degrees: i32) {
        self.rotation = normalize_rotation(degrees);
    }

    pub fn rotate_by(&mut self, delta: i32) {
        self.set_rotation(self.rotation as i32 + delta);
    }

    /// Dimensi setelah rotasi pengguna (90°/270° menukar lebar & tinggi).
    pub fn oriented_dims(&self) -> (u32, u32) {
        if self.rotation % 180 == 0 {
            self.dims
        } else {
            (self.dims.1, self.dims.0)
        }
    }
}

// ---------------------------------------------------------------------------
// Background jobs
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JobTag {
    Sheet,
    Print,
    Convert,
}

#[derive(Clone)]
pub struct JobCtx {
    tx: Sender<JobEvent>,
}

impl JobCtx {
    pub fn progress(&self, value: f32, message: &str) {
        let _ = self.tx.send(JobEvent::Progress(
            value.clamp(0.0, 1.0),
            message.to_string(),
        ));
    }
}

pub enum JobEvent {
    Progress(f32, String),
    Done(Result<JobResult, String>),
}

pub struct JobResult {
    pub title: String,
    pub note: String,
    pub files: Vec<PathBuf>,
}

pub struct Job {
    label: String,
    tag: JobTag,
    rx: Receiver<JobEvent>,
    progress: f32,
    message: String,
}

// ---------------------------------------------------------------------------
// Toast
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToastKind {
    Success,
    Error,
    Info,
}

pub struct Toast {
    title: String,
    detail: String,
    kind: ToastKind,
    created: Instant,
}

/// Isi Dynamic Island: job yang sedang berjalan lebih diprioritaskan
/// daripada toast notifikasi.
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

#[derive(Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Prefs {
    pub sheet: SheetSettings,
    pub studio: String,
    pub last_dir: Option<PathBuf>,
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
        theme::apply(&cc.egui_ctx);
        let prefs_path = prefs_path();
        let prefs: Option<Prefs> = std::fs::read_to_string(&prefs_path)
            .ok()
            .and_then(|raw| serde_json::from_str(&raw).ok());

        let settings = prefs.as_ref().map(|p| p.sheet.clone()).unwrap_or_default();
        let studio = prefs.as_ref().map(|p| p.studio.clone()).unwrap_or_default();
        let last_dir = prefs.and_then(|p| p.last_dir);

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

    // -------------------------------------------------------------- photos

    pub fn import_photos_dialog(&mut self) {
        let files = rfd::FileDialog::new()
            .set_title("Pilih foto untuk dicetak")
            .add_filter("Gambar", image_utils::SUPPORTED_IMAGE_EXTS)
            .pick_files();
        if let Some(files) = files {
            self.add_photos(files);
        }
    }

    pub fn add_photos(&mut self, files: Vec<PathBuf>) {
        let before = self.photos.len();
        let mut failed = Vec::new();
        for path in files {
            if !image_utils::is_supported_image(&path) {
                failed.push(path);
                continue;
            }
            if self.photos.iter().any(|p| p.path == path) {
                continue;
            }
            match image_utils::oriented_dimensions(&path) {
                Ok(dims) => {
                    let bytes = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
                    self.photos.push(PhotoItem {
                        path,
                        dims,
                        bytes,
                        rotation: 0,
                    });
                }
                Err(e) => {
                    failed.push(PathBuf::from(e));
                }
            }
        }
        let added = self.photos.len() - before;
        if !self.photos.is_empty() && self.active_photo >= self.photos.len() {
            self.active_photo = self.photos.len() - 1;
        }
        if added > 0 {
            self.toast_success(
                &format!("{added} foto ditambahkan"),
                "Atur ukuran dan margin di panel kiri.",
            );
            if self.inspector.is_none() {
                self.inspector = self.photos.first().map(|p| p.path.clone());
            }
        }
        if !failed.is_empty() {
            self.toast_error(
                "Sebagian file dilewati",
                "Pastikan formatnya JPG, PNG, WEBP, BMP, GIF, atau TIFF.",
            );
        }
        self.prefs_dirty = true;
    }

    pub fn remove_photo(&mut self, index: usize) {
        if index < self.photos.len() {
            let removed = self.photos.remove(index);
            self.textures.retain(|(path, _), _| path != &removed.path);
            if self.active_photo >= self.photos.len() {
                self.active_photo = self.photos.len().saturating_sub(1);
            }
        }
    }

    pub fn clear_photos(&mut self) {
        self.photos.clear();
        self.textures.clear();
        self.texture_errors.clear();
        self.active_photo = 0;
        self.confirm_clear = false;
        self.toast_info(
            "Daftar foto dibersihkan",
            "Impor foto baru untuk mulai lagi.",
        );
    }

    pub fn active_path(&self) -> Option<PathBuf> {
        self.photos.get(self.active_photo).map(|p| p.path.clone())
    }

    pub fn active_ref(&self) -> Option<PhotoRef> {
        self.photos.get(self.active_photo).map(PhotoItem::photo_ref)
    }

    pub fn active_rotation(&self) -> u16 {
        self.photos
            .get(self.active_photo)
            .map(|p| p.rotation)
            .unwrap_or(0)
    }

    pub fn rotate_active(&mut self, delta: i32) {
        if let Some(photo) = self.photos.get_mut(self.active_photo) {
            photo.rotate_by(delta);
        }
    }

    pub fn texture_for(&mut self, ctx: &egui::Context, path: &Path) -> Option<egui::TextureHandle> {
        self.texture_for_rotated(ctx, path, 0)
    }

    pub fn texture_for_rotated(
        &mut self,
        ctx: &egui::Context,
        path: &Path,
        rotation: u16,
    ) -> Option<egui::TextureHandle> {
        let rotation = rotation % 360;
        let key = (path.to_path_buf(), rotation);
        if let Some(tex) = self.textures.get(&key) {
            return Some(tex.clone());
        }
        if self.texture_errors.contains(path) {
            return None;
        }
        match image_utils::thumbnail(path, 480) {
            Ok(img) => {
                let img = image_utils::rotate_rgba(img, rotation);
                let size = [img.width() as usize, img.height() as usize];
                let color = egui::ColorImage::from_rgba_unmultiplied(size, img.as_raw());
                let tex = ctx.load_texture(
                    format!("{}@{}", path.to_string_lossy(), rotation),
                    color,
                    egui::TextureOptions::LINEAR,
                );
                self.textures.insert(key, tex.clone());
                Some(tex)
            }
            Err(e) => {
                self.texture_errors.insert(path.to_path_buf());
                self.toast_error("Gagal memuat pratinjau", &e);
                None
            }
        }
    }

    /// Foto untuk setiap sel pada halaman yang sedang dipratinjau.
    pub fn preview_slots(&self, layout: &SheetLayout) -> Vec<Option<PhotoRef>> {
        match self.settings.mode {
            SheetMode::Single => {
                let active = self.active_ref();
                vec![active; layout.cells.len()]
            }
            SheetMode::Packet => layout
                .cells
                .iter()
                .map(|cell| {
                    let idx = cell.photo.or_else(|| {
                        if self.photos.is_empty() {
                            None
                        } else {
                            Some(self.active_photo.min(self.photos.len() - 1))
                        }
                    });
                    idx.and_then(|i| self.photos.get(i))
                        .map(PhotoItem::photo_ref)
                })
                .collect(),
        }
    }

    /// Semua halaman untuk ekspor PDF.
    pub fn build_pages(&self, layout: &SheetLayout) -> Vec<Vec<Option<PhotoRef>>> {
        if layout.cells.is_empty() {
            return vec![Vec::new()];
        }
        match self.settings.mode {
            SheetMode::Single => {
                if self.settings.batch_per_photo && !self.photos.is_empty() {
                    self.photos
                        .iter()
                        .map(|p| vec![Some(p.photo_ref()); layout.cells.len()])
                        .collect()
                } else {
                    let active = self.active_ref();
                    vec![vec![active; layout.cells.len()]]
                }
            }
            SheetMode::Packet => {
                let slots: Vec<Option<PhotoRef>> = layout
                    .cells
                    .iter()
                    .map(|cell| {
                        let idx = cell.photo.or_else(|| {
                            if self.photos.is_empty() {
                                None
                            } else {
                                Some(self.active_photo.min(self.photos.len() - 1))
                            }
                        });
                        idx.and_then(|i| self.photos.get(i))
                            .map(PhotoItem::photo_ref)
                    })
                    .collect();
                vec![slots]
            }
        }
    }

    pub fn page_count(&self) -> usize {
        if self.settings.mode == SheetMode::Single
            && self.settings.batch_per_photo
            && !self.photos.is_empty()
        {
            self.photos.len()
        } else {
            1
        }
    }

    pub fn min_dpi(&self, layout: &SheetLayout) -> Option<f32> {
        let slots = self.preview_slots(layout);
        slots
            .iter()
            .enumerate()
            .filter_map(|(i, photo)| {
                let photo = photo.as_ref()?;
                let cell = layout.cells.get(i)?;
                let (w, h) = self
                    .photos
                    .iter()
                    .find(|p| p.path == photo.path)
                    .map(PhotoItem::oriented_dims)?;
                Some(image_utils::estimate_dpi(
                    w,
                    h,
                    cell.w_mm,
                    cell.h_mm,
                    self.settings.fit,
                ))
            })
            .reduce(f32::min)
    }

    // -------------------------------------------------------------- jobs

    pub fn start_job<F>(&mut self, label: impl Into<String>, tag: JobTag, work: F)
    where
        F: FnOnce(JobCtx) -> Result<JobResult, String> + Send + 'static,
    {
        if self.job.is_some() {
            self.toast_info(
                "Masih ada proses berjalan",
                "Tunggu proses selesai sebelum memulai yang baru.",
            );
            return;
        }
        let (tx, rx) = channel();
        let ctx = JobCtx { tx: tx.clone() };
        let label = label.into();
        std::thread::spawn(move || {
            let result = work(ctx);
            let _ = tx.send(JobEvent::Done(result));
        });
        self.job = Some(Job {
            label,
            tag,
            rx,
            progress: 0.0,
            message: "Memulai…".to_string(),
        });
        self.island_pinned = false;
        self.island_hovered = false;
    }

    fn poll_job(&mut self, ctx: &egui::Context) {
        let mut finished: Option<Result<JobResult, String>> = None;
        if let Some(job) = &mut self.job {
            loop {
                match job.rx.try_recv() {
                    Ok(JobEvent::Progress(p, message)) => {
                        job.progress = p;
                        job.message = message;
                    }
                    Ok(JobEvent::Done(result)) => {
                        finished = Some(result);
                        break;
                    }
                    Err(TryRecvError::Empty) => break,
                    Err(TryRecvError::Disconnected) => {
                        finished = Some(Err("Proses terhenti tanpa hasil.".to_string()));
                        break;
                    }
                }
            }
        }
        if let Some(result) = finished {
            let tag = self.job.as_ref().map(|j| j.tag);
            self.job = None;
            match result {
                Ok(res) => {
                    if tag == Some(JobTag::Convert) {
                        self.convert.results = res.files.clone();
                        self.convert.note = res.note.clone();
                    }
                    if !res.files.is_empty() {
                        self.last_output = res.files.clone();
                    }
                    if tag == Some(JobTag::Print) {
                        self.toast_success("Siap dicetak", &res.note);
                    } else {
                        self.toast_success(&res.title, &res.note);
                    }
                }
                Err(e) => self.toast_error("Proses gagal", &e),
            }
            ctx.request_repaint();
        }
    }

    pub fn toast_success(&mut self, title: &str, detail: &str) {
        self.toast = Some(Toast {
            title: title.to_string(),
            detail: detail.to_string(),
            kind: ToastKind::Success,
            created: Instant::now(),
        });
        self.island_pinned = false;
    }

    pub fn toast_error(&mut self, title: &str, detail: &str) {
        self.toast = Some(Toast {
            title: title.to_string(),
            detail: detail.to_string(),
            kind: ToastKind::Error,
            created: Instant::now(),
        });
        self.island_pinned = false;
    }

    pub fn toast_info(&mut self, title: &str, detail: &str) {
        self.toast = Some(Toast {
            title: title.to_string(),
            detail: detail.to_string(),
            kind: ToastKind::Info,
            created: Instant::now(),
        });
        self.island_pinned = false;
    }

    // -------------------------------------------------------------- prefs

    pub fn save_prefs(&mut self) {
        let prefs = Prefs {
            sheet: self.settings.clone(),
            studio: self.studio.clone(),
            last_dir: self.convert.output_dir.clone(),
        };
        if let Some(dir) = self.prefs_path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        if let Ok(raw) = serde_json::to_string_pretty(&prefs) {
            if std::fs::write(&self.prefs_path, raw).is_ok() {
                self.prefs_dirty = false;
            }
        }
    }

    pub fn save_prefs_manual(&mut self) {
        self.save_prefs();
        let path = self.prefs_path.to_string_lossy().to_string();
        self.toast_success("Preferensi disimpan", &path);
    }

    pub fn reset_settings(&mut self) {
        self.settings = SheetSettings::default();
        self.studio = String::new();
        self.preview_pan = Vec2::ZERO;
        self.preview_zoom = 1.0;
        self.confirm_reset = false;
        self.prefs_dirty = true;
        self.toast_info("Pengaturan direset", "Semua pilihan kembali ke bawaan.");
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

    // -------------------------------------------------------------- chrome

    fn sidebar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let size = Vec2::splat(38.0);
            let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
            ui.painter()
                .rect_filled(rect, CornerRadius::same(11), theme::ACCENT);
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
                        .color(theme::TEXT),
                );
                ui.label(
                    RichText::new("digital printing")
                        .size(10.0)
                        .color(theme::DIM),
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
                .rect_filled(rect, CornerRadius::same(11), theme::ACCENT_SOFT);
            ui.painter().rect_stroke(
                rect,
                CornerRadius::same(11),
                Stroke::new(1.0, Color32::from_rgba_unmultiplied(10, 132, 255, 90)),
                StrokeKind::Inside,
            );
        } else if hovered {
            ui.painter()
                .rect_filled(rect, CornerRadius::same(11), theme::CARD_HOVER);
        }
        let icon_rect = egui::Rect::from_center_size(
            egui::pos2(rect.left() + 23.0, rect.center().y),
            Vec2::splat(18.0),
        );
        theme::icon(
            ui.painter(),
            icon_rect,
            icon,
            if selected { theme::TEXT } else { theme::MUTED },
            1.5,
        );
        ui.painter().text(
            egui::pos2(rect.left() + 42.0, rect.center().y),
            Align2::LEFT_CENTER,
            label,
            FontId::proportional(14.0),
            if selected { theme::TEXT } else { theme::MUTED },
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
            .fill(theme::CARD)
            .corner_radius(CornerRadius::same(14))
            .stroke(Stroke::new(1.0, theme::BORDER_SOFT))
            .inner_margin(Margin::same(14))
            .show(ui, |ui| {
                ui.label(
                    RichText::new(title)
                        .size(12.5)
                        .strong()
                        .color(theme::ACCENT),
                );
                ui.add_space(4.0);
                ui.label(RichText::new(body).size(11.5).color(theme::MUTED));
            });
    }

    fn sidebar_status(&self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if self.job.is_some() {
                ui.spinner();
                ui.label(RichText::new("Memproses…").size(11.5).color(theme::YELLOW));
            } else {
                theme::dot(ui, theme::ACCENT);
                ui.label(
                    RichText::new("Siap mencetak")
                        .size(11.5)
                        .color(theme::MUTED),
                );
            }
        });
        ui.label(RichText::new("v0.1.0").size(10.0).color(theme::DIM));
    }

    fn header(&mut self, ui: &mut egui::Ui) {
        let mut action: Option<HeaderAction> = None;
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(
                    RichText::new(self.tab.title())
                        .size(24.0)
                        .strong()
                        .color(theme::TEXT),
                );
                ui.label(
                    RichText::new(self.tab.subtitle())
                        .size(12.0)
                        .color(theme::MUTED),
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

    fn island_ui(&mut self, ctx: &egui::Context) {
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
            IslandContent::Job { .. } => (theme::ACCENT, 262.0, 124.0, false),
            IslandContent::Toast { kind, .. } => {
                let color = match kind {
                    ToastKind::Success => theme::ACCENT,
                    ToastKind::Error => theme::RED,
                    ToastKind::Info => theme::BLUE,
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
        let radius = 18.0 + 6.0 * t;

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
                        color: Color32::from_black_alpha((120.0 + 70.0 * t) as u8),
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
                    theme::RED
                } else if t > 0.5 {
                    accent
                } else {
                    theme::BORDER
                };
                ui.painter().rect(
                    rect,
                    corner,
                    theme::ISLAND,
                    Stroke::new(1.0, border),
                    StrokeKind::Inside,
                );
                // Kilau tipis di tepi atas — efek kaca kapsul Tahoe.
                let gloss = egui::Rect::from_min_max(
                    rect.left_top() + egui::vec2(corner.nw as f32, 0.0),
                    egui::pos2(rect.right() - corner.ne as f32, rect.top() + 1.5),
                );
                if gloss.width() > 0.0 {
                    ui.painter()
                        .rect_filled(gloss, CornerRadius::same(1), theme::GLASS_TOP);
                }

                let inner = rect.shrink2(egui::vec2(14.0 + 2.0 * t, 9.0 + 3.0 * t));
                let mut child = ui.new_child(
                    egui::UiBuilder::new()
                        .max_rect(inner)
                        .layout(Layout::top_down(Align::Min)),
                );
                child.set_clip_rect(inner);

                match &content {
                    IslandContent::Job {
                        label,
                        message,
                        progress,
                    } => {
                        child.horizontal(|ui| {
                            ui.spinner();
                            let text = if t < 0.62 {
                                format!("{label} · {:.0}%", progress * 100.0)
                            } else {
                                label.clone()
                            };
                            ui.add(
                                egui::Label::new(
                                    RichText::new(text).size(13.0).strong().color(theme::TEXT),
                                )
                                .truncate(),
                            );
                        });
                        if t > 0.62 {
                            child.add_space(7.0 * t);
                            child.add(
                                egui::ProgressBar::new(*progress)
                                    .fill(theme::ACCENT)
                                    .corner_radius(4)
                                    .desired_height(8.0),
                            );
                            child.add_space(3.0);
                            child.add(
                                egui::Label::new(
                                    RichText::new(message).size(11.0).color(theme::MUTED),
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
                                    RichText::new(title).size(12.5).strong().color(theme::TEXT),
                                )
                                .truncate(),
                            );
                        });
                        if t > 0.62 && !detail.is_empty() {
                            child.add_space(4.0);
                            child.add(
                                egui::Label::new(
                                    RichText::new(detail).size(11.0).color(theme::MUTED),
                                )
                                .wrap(),
                            );
                        }
                        if t > 0.9 && *kind == ToastKind::Error {
                            child.add_space(3.0);
                            child.label(
                                RichText::new("Klik untuk menutup")
                                    .size(10.0)
                                    .color(theme::DIM),
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

impl eframe::App for CetakinApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.poll_job(&ctx);
        self.handle_shortcuts(&ctx);
        self.handle_drops(&ctx);

        theme::paint_background(ui);

        egui::Panel::left("cetakin-sidebar")
            .exact_size(228.0)
            .resizable(false)
            .frame(
                egui::Frame::new()
                    .fill(theme::SIDEBAR)
                    .stroke(Stroke::new(1.0, theme::BORDER_SOFT))
                    .inner_margin(Margin {
                        left: 18,
                        right: 18,
                        top: 18,
                        bottom: 16,
                    }),
            )
            .show(ui, |ui| self.sidebar(ui));

        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(Color32::TRANSPARENT)
                    .inner_margin(Margin {
                        left: 24,
                        right: 24,
                        top: 18,
                        bottom: 18,
                    }),
            )
            .show(ui, |ui| {
                self.header(ui);
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
        theme::BG.to_normalized_gamma_f32()
    }
}

// ---------------------------------------------------------------------------
// Shared UI helpers
// ---------------------------------------------------------------------------

pub fn card_title(ui: &mut egui::Ui, icon: theme::Icon, title: &str) {
    ui.horizontal(|ui| {
        let (rect, _) = ui.allocate_exact_size(Vec2::splat(18.0), egui::Sense::hover());
        theme::icon(ui.painter(), rect, icon, theme::ACCENT, 1.5);
        ui.label(RichText::new(title).size(13.5).strong().color(theme::TEXT));
    });
}

pub fn drag_cm(ui: &mut egui::Ui, value_mm: &mut f32, width: f32) {
    let mut cm = *value_mm / 10.0;
    let response = ui.add_sized(
        [width, 28.0],
        egui::DragValue::new(&mut cm)
            .speed(0.02)
            .range(0.0..=50.0)
            .fixed_decimals(2)
            .suffix(" cm"),
    );
    if response.changed() {
        *value_mm = cm * 10.0;
    }
}

pub fn label_value(ui: &mut egui::Ui, label: &str, value: &str) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(label).size(11.5).color(theme::MUTED));
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.label(RichText::new(value).size(11.5).color(theme::TEXT));
        });
    });
}

pub fn dashed_rect(painter: &egui::Painter, rect: egui::Rect, stroke: Stroke) {
    let points = [
        rect.left_top(),
        rect.right_top(),
        rect.right_bottom(),
        rect.left_bottom(),
        rect.left_top(),
    ];
    painter.extend(egui::Shape::dashed_line(&points, stroke, 5.0, 4.0));
}

pub fn cover_uv(texture_size: Vec2, target_aspect: f32) -> egui::Rect {
    let tex_aspect = (texture_size.x / texture_size.y.max(1.0)).max(0.01);
    if tex_aspect > target_aspect {
        let u = (target_aspect / tex_aspect).clamp(0.0, 1.0);
        egui::Rect::from_min_max(
            egui::pos2((1.0 - u) / 2.0, 0.0),
            egui::pos2((1.0 + u) / 2.0, 1.0),
        )
    } else {
        let v = (tex_aspect / target_aspect.max(0.01)).clamp(0.0, 1.0);
        egui::Rect::from_min_max(
            egui::pos2(0.0, (1.0 - v) / 2.0),
            egui::pos2(1.0, (1.0 + v) / 2.0),
        )
    }
}

pub fn paint_photo(
    painter: &egui::Painter,
    rect: egui::Rect,
    tex: &egui::TextureHandle,
    fit: ImageFit,
) {
    match fit {
        ImageFit::Cover => {
            let uv = cover_uv(tex.size_vec2(), rect.width() / rect.height().max(1.0));
            painter.image(tex.id(), rect, uv, Color32::WHITE);
        }
        ImageFit::Contain => {
            painter.rect_filled(rect, CornerRadius::ZERO, Color32::WHITE);
            let tex_size = tex.size_vec2();
            let scale =
                (rect.width() / tex_size.x.max(1.0)).min(rect.height() / tex_size.y.max(1.0));
            let size = tex_size * scale;
            let target = egui::Rect::from_center_size(rect.center(), size);
            painter.image(
                tex.id(),
                target,
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                Color32::WHITE,
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Filesystem helpers
// ---------------------------------------------------------------------------

fn prefs_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("cetakin")
        .join("prefs.json")
}

pub fn unique_path(path: PathBuf) -> PathBuf {
    if !path.exists() {
        return path;
    }
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("hasil");
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_string();
    for n in 2..10_000 {
        let candidate = path.with_file_name(if ext.is_empty() {
            format!("{stem}-{n}")
        } else {
            format!("{stem}-{n}.{ext}")
        });
        if !candidate.exists() {
            return candidate;
        }
    }
    path
}

pub fn safe_slug(text: &str) -> String {
    let mut out = String::new();
    for c in text.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
        } else if (c == ' ' || c == '-' || c == '_') && !out.ends_with('-') {
            out.push('-');
        }
    }
    out.trim_matches('-').to_string()
}

pub fn temp_pdf(marker: &str) -> PathBuf {
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    std::env::temp_dir().join(format!("cetakin-{marker}-{stamp}.pdf"))
}

pub fn open_path(path: &Path) {
    let _ = open::that_detached(path);
}

pub fn open_folder(path: &Path) {
    let folder = if path.is_dir() {
        path.to_path_buf()
    } else {
        path.parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| path.to_path_buf())
    };
    let _ = open::that_detached(folder);
}
