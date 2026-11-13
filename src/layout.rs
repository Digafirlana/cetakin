use crate::model::{Margins, PageOrientation, SheetMode, SheetSettings};

/// Satu foto yang akan ditempatkan pada halaman. Koordinat dalam mm,
/// origin di kiri-atas halaman.
#[derive(Clone, Debug, PartialEq)]
pub struct Cell {
    pub x_mm: f32,
    pub y_mm: f32,
    pub w_mm: f32,
    pub h_mm: f32,
    /// Indeks baris pada daftar paket (None untuk mode satuan).
    pub item_index: Option<usize>,
    /// Foto pilihan khusus untuk sel ini (indeks pada daftar foto).
    pub photo: Option<usize>,
}

impl Cell {
    pub fn right_mm(&self) -> f32 {
        self.x_mm + self.w_mm
    }

    pub fn bottom_mm(&self) -> f32 {
        self.y_mm + self.h_mm
    }
}

#[derive(Clone, Debug)]
pub struct SheetLayout {
    pub page_w_mm: f32,
    pub page_h_mm: f32,
    pub cells: Vec<Cell>,
    /// Jumlah foto yang tidak muat di halaman.
    pub overflow: usize,
    pub cols: usize,
    pub rows: usize,
    pub capacity: usize,
    /// Kuantitas efektif tiap baris paket, termasuk tambahan otomatis.
    /// Kosong untuk mode satuan.
    pub packet_qty: Vec<usize>,
    /// Margin efektif (termasuk pergeseran auto-center).
    pub margins: Margins,
    pub used_w_mm: f32,
    pub used_h_mm: f32,
    pub warnings: Vec<String>,
}

impl SheetLayout {
    /// Sisa margin di kanan/bawah bila grid tidak mengisi penuh area cetak.
    pub fn margin_label(&self) -> String {
        format!(
            "Kiri {:.2} · Atas {:.2} · Kanan {:.2} · Bawah {:.2} cm",
            self.margins.left_mm / 10.0,
            self.margins.top_mm / 10.0,
            self.margins.right_mm / 10.0,
            self.margins.bottom_mm / 10.0,
        )
        .replace('.', ",")
    }
}

pub fn compute(settings: &SheetSettings) -> SheetLayout {
    let (base_w, base_h) = settings.base_paper_mm();
    let (portrait_w, portrait_h) = (base_w.min(base_h), base_h.max(base_w));
    let (landscape_w, landscape_h) = (portrait_h, portrait_w);

    let layout_for = |w: f32, h: f32| -> SheetLayout {
        match settings.mode {
            SheetMode::Single => layout_single(settings, w, h),
            SheetMode::Packet => layout_packet(settings, w, h),
        }
    };

    match settings.orientation {
        PageOrientation::Portrait => layout_for(portrait_w, portrait_h),
        PageOrientation::Landscape => layout_for(landscape_w, landscape_h),
        PageOrientation::Auto => {
            let a = layout_for(portrait_w, portrait_h);
            let b = layout_for(landscape_w, landscape_h);
            let score = |l: &SheetLayout| -> (usize, usize) {
                // Utamakan jumlah foto yang muat, lalu minimalkan overflow.
                (l.cells.len(), usize::MAX - l.overflow)
            };
            if score(&b) > score(&a) {
                b
            } else {
                a
            }
        }
    }
}

fn effective_margins(settings: &SheetSettings) -> Margins {
    let m = settings.base_margins();
    Margins {
        top_mm: m.top_mm.max(0.0),
        right_mm: m.right_mm.max(0.0),
        bottom_mm: m.bottom_mm.max(0.0),
        left_mm: m.left_mm.max(0.0),
    }
}

fn grid_count(available: f32, item: f32, gap: f32) -> usize {
    if item <= 0.0 {
        return 0;
    }
    if available < item {
        1
    } else {
        ((available + gap) / (item + gap)).floor().max(1.0) as usize
    }
}

fn used_extent(count: usize, item: f32, gap: f32) -> f32 {
    if count == 0 {
        0.0
    } else {
        count as f32 * item + (count.saturating_sub(1)) as f32 * gap
    }
}

fn layout_single(settings: &SheetSettings, page_w: f32, page_h: f32) -> SheetLayout {
    let margins = effective_margins(settings);
    let gap = settings.gap_mm.clamp(0.0, 30.0);
    let (pw, ph) = settings.photo.mm();
    let avail_w = (page_w - margins.horizontal()).max(0.0);
    let avail_h = (page_h - margins.vertical()).max(0.0);

    let cols = grid_count(avail_w, pw, gap);
    let rows = grid_count(avail_h, ph, gap);
    let capacity = cols * rows;

    let wanted = if settings.quantity == 0 {
        capacity
    } else {
        settings.quantity as usize
    };
    let count = wanted.min(capacity);
    let overflow = wanted.saturating_sub(capacity);

    let used_cols = if count == 0 { 0 } else { cols.min(count) };
    let used_rows = if count == 0 {
        0
    } else {
        (count + cols - 1) / cols
    };
    let used_w = used_extent(used_cols, pw, gap);
    let used_h = used_extent(used_rows, ph, gap);

    let start_x = if settings.auto_center {
        let centered = margins.left_mm + (avail_w - used_w) / 2.0;
        if used_w > avail_w {
            (page_w - used_w) / 2.0
        } else {
            centered
        }
    } else {
        margins.left_mm
    }
    .clamp(0.0, (page_w - used_w).max(0.0));

    let start_y = if settings.auto_center {
        let centered = margins.top_mm + (avail_h - used_h) / 2.0;
        if used_h > avail_h {
            (page_h - used_h) / 2.0
        } else {
            centered
        }
    } else {
        margins.top_mm
    }
    .clamp(0.0, (page_h - used_h).max(0.0));

    let mut cells = Vec::with_capacity(count);
    for i in 0..count {
        let col = i % cols;
        let row = i / cols;
        cells.push(Cell {
            x_mm: start_x + used_extent(col, pw, gap),
            y_mm: start_y + used_extent(row, ph, gap),
            w_mm: pw,
            h_mm: ph,
            item_index: None,
            photo: None,
        });
    }

    let mut warnings = Vec::new();
    if pw > avail_w + 0.01 || ph > avail_h + 0.01 {
        warnings.push(
            "Ukuran foto lebih besar dari area cetak. Kurangi margin atau pilih kertas lebih besar."
                .to_string(),
        );
    }
    if overflow > 0 {
        warnings.push(format!(
            "{} foto tidak muat dan tidak akan dicetak.",
            overflow
        ));
    }

    SheetLayout {
        page_w_mm: page_w,
        page_h_mm: page_h,
        cells,
        overflow,
        cols,
        rows,
        capacity,
        packet_qty: Vec::new(),
        margins: Margins {
            left_mm: start_x,
            top_mm: start_y,
            right_mm: (page_w - start_x - used_w).max(0.0),
            bottom_mm: (page_h - start_y - used_h).max(0.0),
        },
        used_w_mm: used_w,
        used_h_mm: used_h,
        warnings,
    }
}

/// Kuantitas efektif tiap ukuran paket. Bila `packet_auto_fill` aktif, slot
/// kosong pada baris terakhir diisi lebih dulu, lalu baris tambahan ditambahkan
/// bergiliran selama masih muat tanpa melewati area cetak.
fn packet_quantities(settings: &SheetSettings, avail_w: f32, avail_h: f32, gap: f32) -> Vec<usize> {
    let mut qty: Vec<usize> = settings
        .packet
        .iter()
        .map(|item| item.qty as usize)
        .collect();
    if !settings.packet_auto_fill || qty.is_empty() {
        return qty;
    }

    // Jangan menambah apa pun bila paket dasar saja sudah melebihi halaman.
    if packet_used_h(settings, &qty, avail_w, gap) > avail_h + 0.01 {
        return qty;
    }

    // Isi slot kosong di baris terakhir tiap ukuran (tinggi tidak bertambah).
    for (i, item) in settings.packet.iter().enumerate() {
        if qty[i] == 0 {
            continue;
        }
        let cols = grid_count(avail_w, item.size.mm().0, gap).max(1);
        let rem = qty[i] % cols;
        if rem != 0 {
            qty[i] += cols - rem;
        }
    }

    let used = packet_used_h(settings, &qty, avail_w, gap);
    let mut remaining = avail_h - used;
    if remaining <= 0.0 {
        return qty;
    }

    // Tambah satu baris penuh secara bergiliran sampai tidak ada yang muat.
    loop {
        let mut added = false;
        for (i, item) in settings.packet.iter().enumerate() {
            if qty[i] == 0 {
                continue;
            }
            let (pw, ph) = item.size.mm();
            let cols = grid_count(avail_w, pw, gap).max(1);
            let cost = ph + gap;
            if remaining + 0.01 >= cost {
                qty[i] += cols;
                remaining -= cost;
                added = true;
            }
        }
        if !added {
            break;
        }
    }

    qty
}

/// Tinggi total (mm) yang dipakai sederet kuantitas paket pada satu halaman.
fn packet_used_h(settings: &SheetSettings, qty: &[usize], avail_w: f32, gap: f32) -> f32 {
    let mut used = 0.0;
    let mut any = false;
    for (i, item) in settings.packet.iter().enumerate() {
        let count = qty.get(i).copied().unwrap_or(0);
        if count == 0 {
            continue;
        }
        let (pw, ph) = item.size.mm();
        let cols = grid_count(avail_w, pw, gap).max(1);
        let rows = count.div_ceil(cols);
        used += rows as f32 * (ph + gap);
        any = true;
    }
    if any {
        used = (used - gap).max(0.0);
    }
    used
}

fn layout_packet(settings: &SheetSettings, page_w: f32, page_h: f32) -> SheetLayout {
    let margins = effective_margins(settings);
    let gap = settings.gap_mm.clamp(0.0, 30.0);
    let avail_w = (page_w - margins.horizontal()).max(0.0);
    let avail_h = (page_h - margins.vertical()).max(0.0);
    let packet_qty = packet_quantities(settings, avail_w, avail_h, gap);

    let start_x_base = margins.left_mm;
    let mut y = margins.top_mm;
    let mut cells = Vec::new();
    let mut overflow = 0usize;
    let mut max_cols = 0usize;
    let mut total_rows = 0usize;
    let mut stopped = false;

    for (item_idx, item) in settings.packet.iter().enumerate() {
        let count = packet_qty.get(item_idx).copied().unwrap_or(0);
        if count == 0 {
            continue;
        }
        let (pw, ph) = item.size.mm();
        let cols = grid_count(avail_w, pw, gap).max(1);
        let mut remaining = count;
        let mut item_rows = 0usize;

        while remaining > 0 {
            if stopped || y + ph > page_h - margins.bottom_mm + 0.01 {
                stopped = true;
                overflow += remaining;
                break;
            }
            let n = remaining.min(cols);
            let row_w = used_extent(n, pw, gap);
            let row_x = if settings.auto_center {
                start_x_base + ((avail_w - row_w) / 2.0).max(0.0)
            } else {
                start_x_base
            };
            for c in 0..n {
                cells.push(Cell {
                    x_mm: row_x + used_extent(c, pw, gap),
                    y_mm: y,
                    w_mm: pw,
                    h_mm: ph,
                    item_index: Some(item_idx),
                    photo: item.photo,
                });
            }
            remaining -= n;
            item_rows += 1;
            y += ph + gap;
        }
        max_cols = max_cols.max(cols);
        total_rows += item_rows;
    }

    // Tinggi terpakai dari sel-sel aktual.
    let used_h = cells.iter().map(|c| c.bottom_mm()).fold(0.0f32, f32::max) - margins.top_mm;

    let mut warnings = Vec::new();
    if overflow > 0 {
        warnings.push(format!(
            "{} foto tidak muat di halaman ini. Tambah halaman, kurangi margin, atau kecilkan ukuran.",
            overflow
        ));
    }
    if cells
        .iter()
        .any(|c| c.w_mm > avail_w + 0.01 || c.h_mm > avail_h + 0.01)
    {
        warnings.push(
            "Ada ukuran foto yang lebih besar dari area cetak. Kurangi margin atau ganti kertas."
                .to_string(),
        );
    }

    let left = cells
        .iter()
        .map(|c| c.x_mm)
        .fold(f32::MAX, f32::min)
        .min(page_w);
    let right = cells.iter().map(|c| c.right_mm()).fold(0.0, f32::max);
    let top = cells
        .iter()
        .map(|c| c.y_mm)
        .fold(f32::MAX, f32::min)
        .min(page_h);
    let bottom = cells.iter().map(|c| c.bottom_mm()).fold(0.0, f32::max);

    let effective = if cells.is_empty() {
        margins
    } else {
        Margins {
            left_mm: left.max(0.0),
            top_mm: top.max(0.0),
            right_mm: (page_w - right).max(0.0),
            bottom_mm: (page_h - bottom).max(0.0),
        }
    };

    SheetLayout {
        page_w_mm: page_w,
        page_h_mm: page_h,
        cells,
        overflow,
        cols: max_cols,
        rows: total_rows,
        capacity: packet_qty.iter().sum(),
        packet_qty,
        margins: effective,
        used_w_mm: (right - left.max(0.0)).max(0.0),
        used_h_mm: used_h.max(0.0),
        warnings,
    }
}

/// Segmen garis potong yang akan digambar pada halaman (mm).
/// `Frame` menggambar bingkai tiap foto, `Guillotine` meneruskan garis
/// sampai tepi kertas agar mudah dipotong massal.
pub fn cut_segments(
    layout: &SheetLayout,
    style: crate::model::CutStyle,
) -> Vec<(f32, f32, f32, f32)> {
    use crate::model::CutStyle;
    let mut segs = Vec::new();
    match style {
        CutStyle::None => {}
        CutStyle::Frame => {
            for c in &layout.cells {
                let (x0, y0, x1, y1) = (c.x_mm, c.y_mm, c.right_mm(), c.bottom_mm());
                segs.push((x0, y0, x1, y0));
                segs.push((x1, y0, x1, y1));
                segs.push((x1, y1, x0, y1));
                segs.push((x0, y1, x0, y0));
            }
        }
        CutStyle::Guillotine => {
            // Garis horizontal dan vertikal unik, diteruskan ke tepi halaman.
            let mut xs: Vec<f32> = Vec::new();
            let mut ys: Vec<f32> = Vec::new();
            for c in &layout.cells {
                for x in [c.x_mm, c.right_mm()] {
                    if !xs.iter().any(|v| (*v - x).abs() < 0.01) {
                        xs.push(x);
                    }
                }
                for y in [c.y_mm, c.bottom_mm()] {
                    if !ys.iter().any(|v| (*v - y).abs() < 0.01) {
                        ys.push(y);
                    }
                }
            }
            for x in xs {
                segs.push((x, 0.0, x, layout.page_h_mm));
            }
            for y in ys {
                segs.push((0.0, y, layout.page_w_mm, y));
            }
        }
    }
    segs
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{CutStyle, MarginPreset, PhotoPreset, PhotoSize};

    fn single(photo: PhotoPreset, orientation: PageOrientation) -> SheetSettings {
        SheetSettings {
            orientation,
            margin_preset: MarginPreset::Normal,
            photo: PhotoSize::from_preset(photo),
            gap_mm: 3.0,
            quantity: 0,
            ..Default::default()
        }
    }

    #[test]
    fn a4_fits_20_photos_3x4() {
        let s = single(PhotoPreset::Cm3x4, PageOrientation::Portrait);
        let l = compute(&s);
        assert_eq!(l.page_w_mm, 210.0);
        assert_eq!(l.page_h_mm, 297.0);
        assert_eq!(l.cols, 4);
        assert_eq!(l.rows, 5);
        assert_eq!(l.cells.len(), 20);
        assert!(l.warnings.is_empty());
    }

    #[test]
    fn auto_orientation_picks_better_page() {
        let mut s = single(PhotoPreset::Cm4x6, PageOrientation::Auto);
        s.photo.landscape = true;
        let l = compute(&s);
        // Dengan margin Normal, halaman potret memuat 10 foto 6×4 cm,
        // sedangkan halaman mendatar hanya 9. Auto memilih yang terbanyak.
        assert_eq!(l.page_w_mm, 210.0);
        assert_eq!(l.page_h_mm, 297.0);
        assert_eq!(l.cells.len(), 10);
    }

    #[test]
    fn quantity_is_limited_and_reported() {
        let mut s = single(PhotoPreset::Cm3x4, PageOrientation::Portrait);
        s.quantity = 24;
        let l = compute(&s);
        assert_eq!(l.cells.len(), 20);
        assert_eq!(l.overflow, 4);
        assert!(!l.warnings.is_empty());
    }

    #[test]
    fn grid_is_centered_on_page_with_symmetric_margins() {
        let s = single(PhotoPreset::Cm3x4, PageOrientation::Portrait);
        let l = compute(&s);
        assert!((l.margins.left_mm - l.margins.right_mm).abs() < 0.01);
        assert!((l.margins.top_mm - l.margins.bottom_mm).abs() < 0.01);
    }

    #[test]
    fn packet_fits_and_overflow_detected() {
        let mut s = SheetSettings::default();
        s.mode = SheetMode::Packet;
        let l = compute(&s);
        assert_eq!(l.cells.len(), 14);
        assert_eq!(l.overflow, 0);

        s.packet[0].qty = 200;
        let l = compute(&s);
        assert!(l.overflow > 0);
    }

    #[test]
    fn packet_auto_fill_uses_leftover_without_overflow() {
        let mut s = SheetSettings::default();
        s.mode = SheetMode::Packet;
        let base = compute(&s);

        s.packet_auto_fill = true;
        let filled = compute(&s);
        assert!(filled.cells.len() > base.cells.len());
        assert_eq!(filled.overflow, 0);
        assert_eq!(filled.cells.len(), filled.capacity);
        assert_eq!(filled.packet_qty.len(), s.packet.len());
        assert_eq!(filled.packet_qty.iter().sum::<usize>(), filled.cells.len());
        assert!(filled.used_h_mm <= 297.0 - 25.4 + 0.01);
    }

    #[test]
    fn packet_auto_fill_keeps_overflowing_package_untouched() {
        let mut s = SheetSettings::default();
        s.mode = SheetMode::Packet;
        s.packet[0].qty = 200;
        s.packet_auto_fill = true;
        let l = compute(&s);
        assert_eq!(l.packet_qty[0], 200);
        assert!(l.overflow > 0);
    }

    #[test]
    fn cut_guillotine_reaches_page_edges() {
        let s = single(PhotoPreset::Cm3x4, PageOrientation::Portrait);
        let l = compute(&s);
        let segs = cut_segments(&l, CutStyle::Guillotine);
        assert!(segs.iter().any(|(_, y0, _, y1)| *y0 == 0.0 && *y1 == 297.0));
        assert!(segs.iter().any(|(x0, _, x1, _)| *x0 == 0.0 && *x1 == 210.0));
    }

    #[test]
    fn zero_margin_packs_more() {
        let mut s = single(PhotoPreset::Cm2x3, PageOrientation::Portrait);
        s.margin_preset = MarginPreset::None;
        let l = compute(&s);
        // 210 / 23 = 9 kolom, 297 / 33 = 9 baris
        assert_eq!(l.cells.len(), 81);
    }
}
