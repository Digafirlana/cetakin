//! Data foto dan pengelolaan daftar foto (impor, rotasi, thumbnail,
//! serta pemetaan foto ke sel layout).

use std::path::{Path, PathBuf};

use eframe::egui;

use crate::app::CetakinApp;
use crate::image_utils;
use crate::layout::SheetLayout;
use crate::model::{normalize_rotation, PhotoRef, SheetMode};

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

impl CetakinApp {
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
}
