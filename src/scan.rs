//! Pemrosesan citra untuk pemindai dokumen.
//!
//! Alur kerja:
//!   1. deteksi tepi dokumen (quad)
//!   2. koreksi perspektif (warp) sehingga dokumen tampak difoto tegak lurus
//!   3. perataan cahaya + pemutihan kertas + peningkatan kontras
//!
//! Semua fungsi di sini murni (tidak menyentuh UI) agar mudah diuji.

use image::{imageops, Rgba, RgbaImage};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ScanPreset {
    /// Warna asli dipertahankan, kertas diputihkan.
    Color,
    /// Abu-abu kontras tinggi, cocok untuk dokumen teks.
    #[default]
    Document,
    /// Ambang hitam-putih tegas.
    BlackWhite,
}

impl ScanPreset {
    pub const ALL: [ScanPreset; 3] = [
        ScanPreset::Color,
        ScanPreset::Document,
        ScanPreset::BlackWhite,
    ];

    pub fn label(self) -> &'static str {
        match self {
            ScanPreset::Color => "Warna",
            ScanPreset::Document => "Dokumen",
            ScanPreset::BlackWhite => "Hitam Putih",
        }
    }

    pub fn hint(self) -> &'static str {
        match self {
            ScanPreset::Color => "Warna tetap, latar diputihkan",
            ScanPreset::Document => "Abu-abu tajam untuk teks",
            ScanPreset::BlackWhite => "Hitam-putih tegas",
        }
    }

    pub fn index(self) -> usize {
        match self {
            ScanPreset::Color => 0,
            ScanPreset::Document => 1,
            ScanPreset::BlackWhite => 2,
        }
    }

    pub fn from_index(i: usize) -> ScanPreset {
        Self::ALL.get(i).copied().unwrap_or_default()
    }
}

/// Sudut penuh gambar (urut: kiri-atas, kanan-atas, kanan-bawah, kiri-bawah).
pub fn full_frame() -> [[f32; 2]; 4] {
    [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]
}

/// Deteksi otomatis tepi dokumen. Mengembalikan sudut ternormalisasi 0..1,
/// atau `None` bila tidak ada kandidat yang cukup meyakinkan.
pub fn detect_quad(img: &RgbaImage) -> Option<[[f32; 2]; 4]> {
    let (w, h) = img.dimensions();
    if w < 32 || h < 32 {
        return None;
    }

    // Kecilkan agar deteksi tetap ringan dan tahan noise.
    let small = if w.max(h) > 360 {
        let scale = 360.0 / w.max(h) as f32;
        imageops::thumbnail(
            img,
            (w as f32 * scale).round().max(8.0) as u32,
            (h as f32 * scale).round().max(8.0) as u32,
        )
    } else {
        img.clone()
    };
    let (sw, sh) = small.dimensions();

    // 1) Dokumen dianggap berbeda warna dari latar di sekeliling bingkai.
    let border = border_color(&small);
    for threshold in [26u32, 38, 52, 70] {
        let mask = color_distance_mask(&small, border, threshold);
        if let Some(quad) = largest_quad(&mask, sw, sh) {
            return Some(normalize_quad(quad, sw, sh));
        }
    }

    // 2) Fallback: objek paling terang (kertas) atau paling gelap.
    let gray = luma_map(&small);
    let threshold = otsu_threshold(&histogram(&gray));
    let bright: Vec<bool> = gray.iter().map(|&v| v > threshold).collect();
    if let Some(quad) = largest_quad(&bright, sw, sh) {
        return Some(normalize_quad(quad, sw, sh));
    }
    let dark: Vec<bool> = gray.iter().map(|&v| v <= threshold).collect();
    largest_quad(&dark, sw, sh).map(|quad| normalize_quad(quad, sw, sh))
}

/// Koreksi perspektif: petakan quad pada gambar ke persegi panjang.
pub fn warp_quad(img: &RgbaImage, quad: &[[f32; 2]; 4], max_side: u32) -> RgbaImage {
    let (sw, sh) = img.dimensions();
    if sw == 0 || sh == 0 {
        return RgbaImage::new(1, 1);
    }
    let max_side = max_side.max(16);

    // Sudut dalam koordinat piksel sumber.
    let q = quad.map(|p| {
        [
            p[0].clamp(0.0, 1.0) * sw as f32,
            p[1].clamp(0.0, 1.0) * sh as f32,
        ]
    });

    let width = dist(q[0], q[1]).max(dist(q[3], q[2]));
    let height = dist(q[0], q[3]).max(dist(q[1], q[2]));
    let mut out_w = width.round().max(2.0) as u32;
    let mut out_h = height.round().max(2.0) as u32;
    let longest = out_w.max(out_h);
    if longest > max_side {
        let scale = max_side as f32 / longest as f32;
        out_w = ((out_w as f32 * scale).round() as u32).max(2);
        out_h = ((out_h as f32 * scale).round() as u32).max(2);
    }

    // `matrix` memetakan persegi satuan ke koordinat piksel sumber. Setiap
    // piksel keluaran dipetakan ke posisi ternormalisasi lalu ke sumber.
    let Some(matrix) = unit_to_quad(&q) else {
        return RgbaImage::new(out_w, out_h);
    };

    let mut out = RgbaImage::new(out_w, out_h);
    for y in 0..out_h {
        let v = (y as f64 + 0.5) / out_h as f64;
        for x in 0..out_w {
            let u = (x as f64 + 0.5) / out_w as f64;
            let den = matrix[2][0] * u + matrix[2][1] * v + matrix[2][2];
            if den.abs() < 1e-9 {
                continue;
            }
            let sx = (matrix[0][0] * u + matrix[0][1] * v + matrix[0][2]) / den;
            let sy = (matrix[1][0] * u + matrix[1][1] * v + matrix[1][2]) / den;
            out.put_pixel(x, y, sample_bilinear(img, sx as f32, sy as f32));
        }
    }
    out
}

/// Warp lalu ratakan pencahayaan sesuai preset.
pub fn process(
    img: &RgbaImage,
    quad: &[[f32; 2]; 4],
    preset: ScanPreset,
    max_side: u32,
) -> RgbaImage {
    let warped = warp_quad(img, quad, max_side);
    enhance(&warped, preset)
}

/// Ratakan pencahayaan, putihkan kertas, dan tingkatkan kontras.
pub fn enhance(img: &RgbaImage, preset: ScanPreset) -> RgbaImage {
    let (w, h) = img.dimensions();
    if w < 2 || h < 2 {
        return img.clone();
    }
    let count = (w * h) as usize;

    // Luminansi & histogram awal.
    let mut gray = vec![0u8; count];
    let mut hist = [0u32; 256];
    for (i, px) in img.pixels().enumerate() {
        let v = luma(px.0);
        gray[i] = v;
        hist[v as usize] += 1;
    }

    // Warna kertas diambil dari piksel paling terang → netralkan tint.
    let paper_level = percentile(&hist, 0.90);
    // Lantai pencahayaan: area gelap besar (logo/foto) tidak boleh dianggap
    // sebagai latar, jadi kecerahan kertas global dipakai sebagai batas bawah.
    let paper_floor = (paper_level as f32 * 0.88).max(8.0);
    let mut sums = [0u64; 3];
    let mut paper_count = 0u64;
    for px in img.pixels() {
        if luma(px.0) >= paper_level {
            for (c, sum) in sums.iter_mut().enumerate() {
                *sum += px.0[c] as u64;
            }
            paper_count += 1;
        }
    }
    let gains = if paper_count > 0 {
        let means = sums.map(|s| (s / paper_count).max(1) as f32);
        let max = means[0].max(means[1]).max(means[2]);
        [
            (max / means[0]).clamp(0.72, 1.38),
            (max / means[1]).clamp(0.72, 1.38),
            (max / means[2]).clamp(0.72, 1.38),
        ]
    } else {
        [1.0; 3]
    };

    // Peta pencahayaan lokal (kertas tetap terang walau ada bayangan).
    let bg = background_map(&gray, w, h, 320);

    // Pass 2: luma setelah diratakan.
    let mut flat = vec![0u8; count];
    let mut flat_hist = [0u32; 256];
    for y in 0..h {
        for x in 0..w {
            let i = (y * w + x) as usize;
            let local = bg.sample(x, y).max(paper_floor);
            let px = img.get_pixel(x, y).0;
            let r = px[0] as f32 * gains[0] * 255.0 / local;
            let g = px[1] as f32 * gains[1] * 255.0 / local;
            let b = px[2] as f32 * gains[2] * 255.0 / local;
            let v = (0.299 * r + 0.587 * g + 0.114 * b).clamp(0.0, 255.0) as u8;
            flat[i] = v;
            flat_hist[v as usize] += 1;
        }
    }

    let low = percentile(&flat_hist, 0.02) as f32;
    let high = (percentile(&flat_hist, 0.97) as f32).max(low + 8.0);

    let mut out = RgbaImage::new(w, h);
    match preset {
        ScanPreset::Color => {
            for y in 0..h {
                for x in 0..w {
                    let local = bg.sample(x, y).max(paper_floor);
                    let px = img.get_pixel(x, y).0;
                    let r = px[0] as f32 * gains[0] * 255.0 / local;
                    let g = px[1] as f32 * gains[1] * 255.0 / local;
                    let b = px[2] as f32 * gains[2] * 255.0 / local;
                    let l = (0.299 * r + 0.587 * g + 0.114 * b).max(1.0);
                    let target = level(l, low, high) as f32;
                    let k = target / l;
                    out.put_pixel(
                        x,
                        y,
                        Rgba([
                            (r * k).clamp(0.0, 255.0).round() as u8,
                            (g * k).clamp(0.0, 255.0).round() as u8,
                            (b * k).clamp(0.0, 255.0).round() as u8,
                            255,
                        ]),
                    );
                }
            }
        }
        ScanPreset::Document => {
            for y in 0..h {
                for x in 0..w {
                    let v = level(flat[(y * w + x) as usize] as f32, low, high);
                    out.put_pixel(x, y, Rgba([v, v, v, 255]));
                }
            }
        }
        ScanPreset::BlackWhite => {
            let threshold = otsu_threshold(&flat_hist) as f32;
            for y in 0..h {
                for x in 0..w {
                    let v = if (flat[(y * w + x) as usize] as f32) <= threshold {
                        0
                    } else {
                        255
                    };
                    out.put_pixel(x, y, Rgba([v, v, v, 255]));
                }
            }
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Geometri
// ---------------------------------------------------------------------------

type Mat3 = [[f64; 3]; 3];

fn dist(a: [f32; 2], b: [f32; 2]) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)).sqrt()
}

/// Homografi yang memetakan persegi satuan (0,0)…(1,1) ke quad.
fn unit_to_quad(quad: &[[f32; 2]; 4]) -> Option<Mat3> {
    let unit = [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)];
    let mut a = [[0.0f64; 9]; 8];
    for i in 0..4 {
        let (u, v) = unit[i];
        let x = quad[i][0] as f64;
        let y = quad[i][1] as f64;
        a[2 * i] = [u, v, 1.0, 0.0, 0.0, 0.0, -u * x, -v * x, x];
        a[2 * i + 1] = [0.0, 0.0, 0.0, u, v, 1.0, -u * y, -v * y, y];
    }

    // Eliminasi Gauss–Jordan pada matriks 8×9.
    for col in 0..8 {
        let mut pivot = col;
        for row in (col + 1)..8 {
            if a[row][col].abs() > a[pivot][col].abs() {
                pivot = row;
            }
        }
        if a[pivot][col].abs() < 1e-12 {
            return None;
        }
        a.swap(col, pivot);
        let div = a[col][col];
        for cell in a[col].iter_mut().skip(col) {
            *cell /= div;
        }
        let pivot_row = a[col];
        for (row, cells) in a.iter_mut().enumerate() {
            if row == col {
                continue;
            }
            let f = cells[col];
            if f.abs() > 1e-15 {
                for (cell, p) in cells.iter_mut().zip(pivot_row.iter()).skip(col) {
                    *cell -= f * *p;
                }
            }
        }
    }

    Some([
        [a[0][8], a[1][8], a[2][8]],
        [a[3][8], a[4][8], a[5][8]],
        [a[6][8], a[7][8], 1.0],
    ])
}

fn sample_bilinear(img: &RgbaImage, x: f32, y: f32) -> Rgba<u8> {
    let (w, h) = img.dimensions();
    if w == 0 || h == 0 {
        return Rgba([255, 255, 255, 255]);
    }
    let x = x.clamp(0.0, (w - 1) as f32);
    let y = y.clamp(0.0, (h - 1) as f32);
    let x0 = x.floor() as u32;
    let y0 = y.floor() as u32;
    let x1 = (x0 + 1).min(w - 1);
    let y1 = (y0 + 1).min(h - 1);
    let tx = x - x0 as f32;
    let ty = y - y0 as f32;

    let p00 = img.get_pixel(x0, y0).0;
    let p10 = img.get_pixel(x1, y0).0;
    let p01 = img.get_pixel(x0, y1).0;
    let p11 = img.get_pixel(x1, y1).0;

    let mut out = [0u8; 4];
    for c in 0..4 {
        let top = p00[c] as f32 * (1.0 - tx) + p10[c] as f32 * tx;
        let bottom = p01[c] as f32 * (1.0 - tx) + p11[c] as f32 * tx;
        out[c] = (top * (1.0 - ty) + bottom * ty).round() as u8;
    }
    Rgba(out)
}

// ---------------------------------------------------------------------------
// Deteksi dokumen
// ---------------------------------------------------------------------------

fn border_color(img: &RgbaImage) -> [u8; 3] {
    let (w, h) = img.dimensions();
    let band = (w.min(h) / 40).clamp(1, 5);
    let mut channels = [Vec::new(), Vec::new(), Vec::new()];
    for y in 0..h {
        for x in 0..w {
            let edge = x < band || y < band || x + band >= w || y + band >= h;
            if edge {
                let px = img.get_pixel(x, y).0;
                for c in 0..3 {
                    channels[c].push(px[c]);
                }
            }
        }
    }
    let median = |v: &mut Vec<u8>| -> u8 {
        if v.is_empty() {
            return 0;
        }
        v.sort_unstable();
        v[v.len() / 2]
    };
    [
        median(&mut channels[0]),
        median(&mut channels[1]),
        median(&mut channels[2]),
    ]
}

fn color_distance_mask(img: &RgbaImage, reference: [u8; 3], threshold: u32) -> Vec<bool> {
    let limit = threshold * threshold;
    let mut mask = Vec::with_capacity((img.width() * img.height()) as usize);
    for px in img.pixels() {
        let dr = px.0[0] as i32 - reference[0] as i32;
        let dg = px.0[1] as i32 - reference[1] as i32;
        let db = px.0[2] as i32 - reference[2] as i32;
        let distance = (dr * dr + dg * dg + db * db) as u32;
        mask.push(distance > limit);
    }
    mask
}

/// Cari komponen terhubung terbesar dan ambil 4 titik ekstremnya.
fn largest_quad(mask: &[bool], w: u32, h: u32) -> Option<[[f32; 2]; 4]> {
    let count = (w * h) as usize;
    if mask.len() != count || w < 16 || h < 16 {
        return None;
    }
    let mut seen = vec![false; count];
    let mut stack: Vec<u32> = Vec::new();
    let mut best: Option<(f64, [[f32; 2]; 4])> = None;
    let border_total = (2 * w + 2 * h).max(1) as u64;
    let image_area = count as f64;

    for start in 0..count {
        if !mask[start] || seen[start] {
            continue;
        }
        seen[start] = true;
        stack.clear();
        stack.push(start as u32);

        let mut area = 0u64;
        let mut border_hits = 0u64;
        let mut min_sum = f32::MAX;
        let mut max_diff = f32::MIN;
        let mut max_sum = f32::MIN;
        let mut min_diff = f32::MAX;
        let mut tl = [0.0f32; 2];
        let mut tr = [0.0f32; 2];
        let mut br = [0.0f32; 2];
        let mut bl = [0.0f32; 2];

        while let Some(idx) = stack.pop() {
            let x = idx % w;
            let y = idx / w;
            area += 1;

            if x == 0 || y == 0 || x + 1 == w || y + 1 == h {
                border_hits += 1;
            }

            let (fx, fy) = (x as f32, y as f32);
            let sum = fx + fy;
            let diff = fx - fy;
            if sum < min_sum {
                min_sum = sum;
                tl = [fx, fy];
            }
            if diff > max_diff {
                max_diff = diff;
                tr = [fx, fy];
            }
            if sum > max_sum {
                max_sum = sum;
                br = [fx, fy];
            }
            if diff < min_diff {
                min_diff = diff;
                bl = [fx, fy];
            }

            if x > 0 {
                let n = idx - 1;
                if mask[n as usize] && !seen[n as usize] {
                    seen[n as usize] = true;
                    stack.push(n);
                }
            }
            if x + 1 < w {
                let n = idx + 1;
                if mask[n as usize] && !seen[n as usize] {
                    seen[n as usize] = true;
                    stack.push(n);
                }
            }
            if y > 0 {
                let n = idx - w;
                if mask[n as usize] && !seen[n as usize] {
                    seen[n as usize] = true;
                    stack.push(n);
                }
            }
            if y + 1 < h {
                let n = idx + w;
                if mask[n as usize] && !seen[n as usize] {
                    seen[n as usize] = true;
                    stack.push(n);
                }
            }
        }

        // Abaikan bercak kecil dan objek yang menempel penuh ke bingkai.
        if (area as f64) < image_area * 0.05 {
            continue;
        }
        if area as f64 > image_area * 0.985 {
            continue;
        }
        if border_hits * 20 > border_total {
            continue;
        }

        let quad = [tl, tr, br, bl];
        let quad_area = polygon_area(&quad);
        if quad_area < image_area * 0.06 {
            continue;
        }
        if !plausible_quad(&quad, w, h) {
            continue;
        }

        match &best {
            Some((best_area, _)) if *best_area >= quad_area => {}
            _ => best = Some((quad_area, quad)),
        }
    }

    best.map(|(_, quad)| quad)
}

fn plausible_quad(quad: &[[f32; 2]; 4], w: u32, h: u32) -> bool {
    for i in 0..4 {
        let a = quad[i];
        let b = quad[(i + 1) % 4];
        if dist(a, b) < (w.min(h) as f32) * 0.08 {
            return false;
        }
    }
    // Sudut harus terurut (kiri-atas … kiri-bawah) dan tidak menyilang.
    polygon_area(quad) > 0.0
}

fn polygon_area(points: &[[f32; 2]; 4]) -> f64 {
    let mut sum = 0.0f64;
    for i in 0..4 {
        let a = points[i];
        let b = points[(i + 1) % 4];
        sum += a[0] as f64 * b[1] as f64 - b[0] as f64 * a[1] as f64;
    }
    (sum.abs() / 2.0).max(0.0)
}

fn normalize_quad(quad: [[f32; 2]; 4], w: u32, h: u32) -> [[f32; 2]; 4] {
    let nw = w.max(1) as f32;
    let nh = h.max(1) as f32;
    quad.map(|p| [(p[0] / nw).clamp(0.0, 1.0), (p[1] / nh).clamp(0.0, 1.0)])
}

// ---------------------------------------------------------------------------
// Pencahayaan & warna
// ---------------------------------------------------------------------------

struct BackgroundMap {
    data: Vec<f32>,
    w: u32,
    h: u32,
    step: u32,
}

impl BackgroundMap {
    fn sample(&self, x: u32, y: u32) -> f32 {
        if self.w == 0 || self.h == 0 {
            return 255.0;
        }
        let fx = x as f32 / self.step as f32;
        let fy = y as f32 / self.step as f32;
        let x0 = (fx.floor() as u32).min(self.w - 1);
        let y0 = (fy.floor() as u32).min(self.h - 1);
        let x1 = (x0 + 1).min(self.w - 1);
        let y1 = (y0 + 1).min(self.h - 1);
        let tx = (fx - x0 as f32).clamp(0.0, 1.0);
        let ty = (fy - y0 as f32).clamp(0.0, 1.0);
        let get = |x: u32, y: u32| self.data[(y * self.w + x) as usize];
        let top = get(x0, y0) * (1.0 - tx) + get(x1, y0) * tx;
        let bottom = get(x0, y1) * (1.0 - tx) + get(x1, y1) * tx;
        top * (1.0 - ty) + bottom * ty
    }
}

/// Peta pencahayaan dari blok-blok kecil. Nilai blok memakai luma maksimum
/// agar teks gelap tidak ikut menurunkan perkiraan terang kertas.
fn background_map(gray: &[u8], w: u32, h: u32, target: u32) -> BackgroundMap {
    let step = ((w.max(h) as f32 / target.max(1) as f32).ceil() as u32).max(1);
    let bw = w.div_ceil(step).max(1);
    let bh = h.div_ceil(step).max(1);
    let mut small = vec![0f32; (bw * bh) as usize];

    for by in 0..bh {
        let y0 = by * step;
        let y1 = ((by + 1) * step).min(h);
        for bx in 0..bw {
            let x0 = bx * step;
            let x1 = ((bx + 1) * step).min(w);
            let mut max = 0u8;
            for y in y0..y1 {
                for x in x0..x1 {
                    max = max.max(gray[(y * w + x) as usize]);
                }
            }
            small[(by * bw + bx) as usize] = max as f32;
        }
    }

    BackgroundMap {
        data: box_blur(&small, bw as usize, bh as usize, 2),
        w: bw,
        h: bh,
        step,
    }
}

fn box_blur(src: &[f32], w: usize, h: usize, radius: usize) -> Vec<f32> {
    let mut out = vec![0f32; src.len()];
    if w == 0 || h == 0 {
        return out;
    }
    for y in 0..h {
        for x in 0..w {
            let mut sum = 0f32;
            let mut n = 0f32;
            let y0 = y.saturating_sub(radius);
            let y1 = (y + radius).min(h - 1);
            let x0 = x.saturating_sub(radius);
            let x1 = (x + radius).min(w - 1);
            for yy in y0..=y1 {
                for xx in x0..=x1 {
                    sum += src[yy * w + xx];
                    n += 1.0;
                }
            }
            out[y * w + x] = sum / n.max(1.0);
        }
    }
    out
}

fn luma(px: [u8; 4]) -> u8 {
    ((77 * px[0] as u32 + 150 * px[1] as u32 + 29 * px[2] as u32) >> 8) as u8
}

fn luma_map(img: &RgbaImage) -> Vec<u8> {
    img.pixels().map(|p| luma(p.0)).collect()
}

fn histogram(values: &[u8]) -> [u32; 256] {
    let mut hist = [0u32; 256];
    for &v in values {
        hist[v as usize] += 1;
    }
    hist
}

fn percentile(hist: &[u32; 256], q: f32) -> u8 {
    let total: u64 = hist.iter().map(|&c| c as u64).sum();
    if total == 0 {
        return 0;
    }
    let target = (total as f64 * q as f64).round().max(1.0) as u64;
    let mut acc = 0u64;
    for (i, &count) in hist.iter().enumerate() {
        acc += count as u64;
        if acc >= target {
            return i as u8;
        }
    }
    255
}

fn otsu_threshold(hist: &[u32; 256]) -> u8 {
    let total: u64 = hist.iter().map(|&c| c as u64).sum();
    if total == 0 {
        return 160;
    }
    let sum: u64 = hist
        .iter()
        .enumerate()
        .map(|(i, &c)| i as u64 * c as u64)
        .sum();
    let mut sum_b = 0u64;
    let mut weight_b = 0u64;
    let mut best = -1.0f64;
    let mut threshold = 128u8;
    for (i, &count) in hist.iter().enumerate() {
        weight_b += count as u64;
        if weight_b == 0 {
            continue;
        }
        let weight_f = total - weight_b;
        if weight_f == 0 {
            break;
        }
        sum_b += i as u64 * count as u64;
        let mean_b = sum_b as f64 / weight_b as f64;
        let mean_f = (sum - sum_b) as f64 / weight_f as f64;
        let between = weight_b as f64 * weight_f as f64 * (mean_b - mean_f).powi(2);
        if between > best {
            best = between;
            threshold = i as u8;
        }
    }
    threshold
}

fn level(value: f32, low: f32, high: f32) -> u8 {
    let range = (high - low).max(1.0);
    (((value - low) / range).clamp(0.0, 1.0) * 255.0).round() as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    fn solid(w: u32, h: u32, color: [u8; 4]) -> RgbaImage {
        RgbaImage::from_pixel(w, h, Rgba(color))
    }

    #[test]
    fn full_frame_is_clockwise() {
        assert_eq!(
            full_frame(),
            [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]
        );
    }

    #[test]
    fn warp_full_frame_keeps_pixels() {
        let mut img = RgbaImage::from_pixel(64, 48, Rgba([200, 100, 50, 255]));
        for y in 12..16 {
            for x in 12..16 {
                img.put_pixel(x, y, Rgba([0, 255, 0, 255]));
            }
        }
        let out = warp_quad(&img, &full_frame(), 4096);
        assert_eq!(out.dimensions(), (64, 48));
        let px = out.get_pixel(14, 14).0;
        assert!(px[1] > 200, "piksel hijau harus tetap hijau: {px:?}");
    }

    #[test]
    fn warp_quad_maps_region_to_full_frame() {
        let mut img = RgbaImage::from_pixel(80, 60, Rgba([200, 100, 50, 255]));
        for y in 12..48 {
            for x in 24..64 {
                img.put_pixel(x, y, Rgba([0, 255, 0, 255]));
            }
        }
        let quad = [
            [24.0 / 80.0, 12.0 / 60.0],
            [64.0 / 80.0, 12.0 / 60.0],
            [64.0 / 80.0, 48.0 / 60.0],
            [24.0 / 80.0, 48.0 / 60.0],
        ];
        let out = warp_quad(&img, &quad, 4096);
        assert_eq!(out.dimensions(), (40, 36));
        for y in 3..out.height() - 3 {
            for x in 3..out.width() - 3 {
                let px = out.get_pixel(x, y).0;
                assert!(px[1] > 200, "isi quad harus hijau di ({x},{y}): {px:?}");
            }
        }
    }

    #[test]
    fn detection_finds_paper_on_dark_table() {
        let mut img = solid(320, 400, [30, 30, 32, 255]);
        for y in 30..370 {
            for x in 40..280 {
                img.put_pixel(x, y, Rgba([235, 232, 225, 255]));
            }
        }
        let quad = detect_quad(&img).expect("dokumen harus terdeteksi");
        let expected = [
            [40.0 / 320.0, 30.0 / 400.0],
            [279.0 / 320.0, 30.0 / 400.0],
            [279.0 / 320.0, 369.0 / 400.0],
            [40.0 / 320.0, 369.0 / 400.0],
        ];
        for (got, want) in quad.iter().zip(expected.iter()) {
            assert!(
                (got[0] - want[0]).abs() < 0.05 && (got[1] - want[1]).abs() < 0.05,
                "sudut {got:?} tidak dekat {want:?}"
            );
        }
    }

    #[test]
    fn document_preset_whitens_paper_and_darkens_ink() {
        let mut img = solid(160, 160, [198, 190, 172, 255]);
        for y in 60..110 {
            for x in 50..115 {
                img.put_pixel(x, y, Rgba([55, 50, 45, 255]));
            }
        }
        let out = enhance(&img, ScanPreset::Document);
        let paper = out.get_pixel(10, 10).0;
        let ink = out.get_pixel(80, 80).0;
        assert!(paper[0] > 240, "kertas harus putih: {paper:?}");
        assert!(ink[0] < 60, "teks harus gelap: {ink:?}");
    }

    #[test]
    fn black_white_preset_is_binary() {
        let mut img = solid(100, 100, [210, 205, 195, 255]);
        for y in 30..70 {
            for x in 30..70 {
                img.put_pixel(x, y, Rgba([70, 65, 60, 255]));
            }
        }
        let out = enhance(&img, ScanPreset::BlackWhite);
        for px in out.pixels() {
            assert!(px.0[0] == 0 || px.0[0] == 255);
            assert_eq!(px.0[0], px.0[1]);
            assert_eq!(px.0[1], px.0[2]);
        }
    }

    #[test]
    fn perspective_output_respects_max_side() {
        let img = solid(800, 600, [240, 240, 240, 255]);
        let out = warp_quad(&img, &full_frame(), 200);
        assert!(out.width().max(out.height()) <= 200);
    }
}
