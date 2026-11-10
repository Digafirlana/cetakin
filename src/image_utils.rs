use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use exif::{In, Reader as ExifReader, Tag};
use image::{DynamicImage, GenericImageView, ImageOutputFormat, Rgba, RgbaImage};

use crate::model::ImageFit;

pub const SUPPORTED_IMAGE_EXTS: &[&str] =
    &["jpg", "jpeg", "png", "webp", "bmp", "gif", "tif", "tiff"];

pub fn is_supported_image(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| SUPPORTED_IMAGE_EXTS.contains(&e.to_ascii_lowercase().as_str()))
        .unwrap_or(false)
}

pub fn is_docx(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| {
            let e = e.to_ascii_lowercase();
            e == "docx" || e == "doc" || e == "odt" || e == "rtf"
        })
        .unwrap_or(false)
}

/// Membaca EXIF orientation. Nilai 1..=8.
fn exif_orientation(path: &Path) -> u32 {
    let Ok(file) = File::open(path) else {
        return 1;
    };
    let mut reader = BufReader::new(file);
    match ExifReader::new().read_from_container(&mut reader) {
        Ok(exif) => exif
            .get_field(Tag::Orientation, In::PRIMARY)
            .and_then(|f| f.value.get_uint(0))
            .unwrap_or(1)
            .clamp(1, 8),
        Err(_) => 1,
    }
}

fn apply_orientation(img: DynamicImage, orientation: u32) -> DynamicImage {
    match orientation {
        2 => img.fliph(),
        3 => img.rotate180(),
        4 => img.flipv(),
        5 => img.rotate90().fliph(),
        6 => img.rotate90(),
        7 => img.rotate270().fliph(),
        8 => img.rotate270(),
        _ => img,
    }
}

/// Membuka gambar dan menerapkan rotasi EXIF (foto HP).
pub fn load_image(path: &Path) -> Result<DynamicImage, String> {
    let img = image::open(path).map_err(|e| format!("Gagal membaca {}: {e}", path.display()))?;
    let orientation = exif_orientation(path);
    Ok(apply_orientation(img, orientation))
}

/// Dimensi setelah orientasi EXIF diterapkan.
pub fn oriented_dimensions(path: &Path) -> Result<(u32, u32), String> {
    let (w, h) = image::image_dimensions(path)
        .map_err(|e| format!("Gagal membaca dimensi {}: {e}", path.display()))?;
    let o = exif_orientation(path);
    Ok(if (5..=8).contains(&o) { (h, w) } else { (w, h) })
}

/// Gambar kecil untuk pratinjau di UI. Tidak pernah melebihi `max_side` px.
pub fn thumbnail(path: &Path, max_side: u32) -> Result<image::RgbaImage, String> {
    let img = load_image(path)?;
    let thumb = img.thumbnail(max_side, max_side);
    Ok(thumb.to_rgba8())
}

/// Putar thumbnail sesuai sudut pengguna (0/90/180/270).
pub fn rotate_rgba(img: RgbaImage, rotation: u16) -> RgbaImage {
    match rotation % 360 {
        90 => image::imageops::rotate90(&img),
        180 => image::imageops::rotate180(&img),
        270 => image::imageops::rotate270(&img),
        _ => img,
    }
}

/// Putar gambar penuh sesuai sudut pengguna.
pub fn rotate_dynamic(img: &DynamicImage, rotation: u16) -> DynamicImage {
    match rotation % 360 {
        90 => img.rotate90(),
        180 => img.rotate180(),
        270 => img.rotate270(),
        _ => img.clone(),
    }
}

/// Seperti [`prepare_for_cell`], tetapi menerapkan rotasi pengguna lebih dulu.
pub fn prepare_for_cell_rotated(
    img: &DynamicImage,
    cell_w_mm: f32,
    cell_h_mm: f32,
    dpi: u32,
    fit: ImageFit,
    rotation: u16,
) -> DynamicImage {
    if rotation % 360 == 0 {
        return prepare_for_cell(img, cell_w_mm, cell_h_mm, dpi, fit);
    }
    let rotated = rotate_dynamic(img, rotation);
    prepare_for_cell(&rotated, cell_w_mm, cell_h_mm, dpi, fit)
}

/// Siapkan gambar agar pas dengan ukuran sel (mm) pada DPI tertentu.
/// Hasil selalu memiliki rasio = rasio sel, sehingga bisa ditempel 1:1 di PDF.
pub fn prepare_for_cell(
    img: &DynamicImage,
    cell_w_mm: f32,
    cell_h_mm: f32,
    dpi: u32,
    fit: ImageFit,
) -> DynamicImage {
    let cell_w_mm = cell_w_mm.max(1.0);
    let cell_h_mm = cell_h_mm.max(1.0);
    let dpi = dpi.clamp(72, 1200);
    let target_w = ((cell_w_mm / 25.4) * dpi as f32).round().clamp(1.0, 6000.0) as u32;
    let target_h = ((cell_h_mm / 25.4) * dpi as f32).round().clamp(1.0, 6000.0) as u32;

    let (sw, sh) = img.dimensions();
    if sw == 0 || sh == 0 {
        return DynamicImage::ImageRgba8(RgbaImage::from_pixel(
            target_w.max(1),
            target_h.max(1),
            Rgba([255, 255, 255, 255]),
        ));
    }

    match fit {
        ImageFit::Cover => {
            let target_aspect = target_w as f32 / target_h as f32;
            let src_aspect = sw as f32 / sh as f32;
            let (cw, ch) = if src_aspect > target_aspect {
                (((sh as f32) * target_aspect).round() as u32, sh)
            } else {
                (sw, ((sw as f32) / target_aspect).round() as u32)
            };
            let cw = cw.clamp(1, sw);
            let ch = ch.clamp(1, sh);
            let x = (sw - cw) / 2;
            let y = (sh - ch) / 2;
            let cropped = img.crop_imm(x, y, cw, ch);
            cropped.resize_exact(target_w, target_h, image::imageops::FilterType::Lanczos3)
        }
        ImageFit::Contain => {
            let scale = (target_w as f32 / sw as f32).min(target_h as f32 / sh as f32);
            let nw = ((sw as f32 * scale).round() as u32).clamp(1, target_w);
            let nh = ((sh as f32 * scale).round() as u32).clamp(1, target_h);
            let resized = img.resize_exact(nw, nh, image::imageops::FilterType::Lanczos3);
            let mut canvas = RgbaImage::from_pixel(target_w, target_h, Rgba([255, 255, 255, 255]));
            let ox = (target_w - nw) / 2;
            let oy = (target_h - nh) / 2;
            image::imageops::overlay(&mut canvas, &resized.to_rgba8(), ox as i64, oy as i64);
            DynamicImage::ImageRgba8(canvas)
        }
    }
}

/// Perkiraan DPI efektif bila foto dicetak pada ukuran sel tertentu.
pub fn estimate_dpi(img_w: u32, img_h: u32, cell_w_mm: f32, cell_h_mm: f32, fit: ImageFit) -> f32 {
    if img_w == 0 || img_h == 0 || cell_w_mm <= 0.0 || cell_h_mm <= 0.0 {
        return 0.0;
    }
    let dpi_w = img_w as f32 * 25.4 / cell_w_mm;
    let dpi_h = img_h as f32 * 25.4 / cell_h_mm;
    // Saat cover, sisi yang dipotong tetap memakai piksel lebih banyak,
    // sehingga dimensi pembatas selalu yang bernilai DPI terkecil.
    let _ = fit;
    dpi_w.min(dpi_h)
}

/// Label kualitas berdasarkan DPI.
pub fn dpi_quality(dpi: f32) -> (&'static str, u8) {
    if dpi >= 300.0 {
        ("Sangat baik", 0)
    } else if dpi >= 200.0 {
        ("Baik", 1)
    } else if dpi >= 120.0 {
        ("Cukup", 2)
    } else {
        ("Rendah", 3)
    }
}

pub fn format_bytes(bytes: u64) -> String {
    if bytes >= 1 << 30 {
        format!("{:.1} GB", bytes as f64 / (1u64 << 30) as f64)
    } else if bytes >= 1 << 20 {
        format!("{:.1} MB", bytes as f64 / (1u64 << 20) as f64)
    } else if bytes >= 1 << 10 {
        format!("{:.0} KB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}

/// Simpan gambar dengan format berdasarkan ekstensi.
pub fn save_image(img: &DynamicImage, path: &Path, quality: u8) -> Result<(), String> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("png")
        .to_ascii_lowercase();
    let file = File::create(path).map_err(|e| format!("Gagal membuat file: {e}"))?;
    let mut writer = std::io::BufWriter::new(file);
    let format = match ext.as_str() {
        "jpg" | "jpeg" => ImageOutputFormat::Jpeg(quality.clamp(40, 100)),
        "webp" => ImageOutputFormat::WebP,
        "bmp" => ImageOutputFormat::Bmp,
        "tif" | "tiff" => ImageOutputFormat::Tiff,
        _ => ImageOutputFormat::Png,
    };
    img.write_to(&mut writer, format)
        .map_err(|e| format!("Gagal menyimpan gambar: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{Rgba, RgbaImage};

    #[test]
    fn prepare_cover_matches_cell_aspect() {
        let img = DynamicImage::ImageRgba8(RgbaImage::from_pixel(400, 800, Rgba([1, 2, 3, 255])));
        let out = prepare_for_cell(&img, 30.0, 40.0, 300, ImageFit::Cover);
        let (w, h) = out.dimensions();
        assert_eq!(w, 354);
        assert_eq!(h, 472);
    }

    #[test]
    fn prepare_contain_has_white_letterbox() {
        let img = DynamicImage::ImageRgba8(RgbaImage::from_pixel(400, 400, Rgba([0, 0, 0, 255])));
        let out = prepare_for_cell(&img, 30.0, 40.0, 300, ImageFit::Contain);
        let (w, h) = out.dimensions();
        assert_eq!((w, h), (354, 472));
        let px = out.to_rgba8().get_pixel(w / 2, 2).0;
        assert_eq!(px, [255, 255, 255, 255]);
    }

    #[test]
    fn rotation_swaps_dimensions() {
        let img = DynamicImage::ImageRgba8(RgbaImage::from_pixel(40, 20, Rgba([1, 2, 3, 255])));
        assert_eq!(rotate_dynamic(&img, 90).dimensions(), (20, 40));
        assert_eq!(rotate_dynamic(&img, 180).dimensions(), (40, 20));
        assert_eq!(rotate_dynamic(&img, 270).dimensions(), (20, 40));
        assert_eq!(rotate_dynamic(&img, 0).dimensions(), (40, 20));
    }

    #[test]
    fn dpi_estimate() {
        // 1200x1800 px untuk 4x6 inci = 300 dpi
        let d = estimate_dpi(1200, 1800, 101.6, 152.4, ImageFit::Cover);
        assert!((d - 300.0).abs() < 1.0);
    }
}
