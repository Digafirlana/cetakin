use serde::{Deserialize, Serialize};


#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaperPreset {
    A4,
    A3,
    A5,
    Letter,
    Legal,
    F4,
    Paper4R,
    Paper5R,
    Paper6R,
    Custom,
}

impl PaperPreset {
    pub const ALL: [PaperPreset; 10] = [
        PaperPreset::A4,
        PaperPreset::A3,
        PaperPreset::A5,
        PaperPreset::Letter,
        PaperPreset::Legal,
        PaperPreset::F4,
        PaperPreset::Paper4R,
        PaperPreset::Paper5R,
        PaperPreset::Paper6R,
        PaperPreset::Custom,
    ];

    pub fn label(self) -> &'static str {
        match self {
            PaperPreset::A4 => "A4  (21 × 29,7 cm)",
            PaperPreset::A3 => "A3  (29,7 × 42 cm)",
            PaperPreset::A5 => "A5  (14,8 × 21 cm)",
            PaperPreset::Letter => "Letter  (21,6 × 27,9 cm)",
            PaperPreset::Legal => "Legal  (21,6 × 35,6 cm)",
            PaperPreset::F4 => "F4 / Folio  (21 × 33 cm)",
            PaperPreset::Paper4R => "Kertas 4R  (10,2 × 15,2 cm)",
            PaperPreset::Paper5R => "Kertas 5R  (12,7 × 17,8 cm)",
            PaperPreset::Paper6R => "Kertas 6R  (15,2 × 20,3 cm)",
            PaperPreset::Custom => "Ukuran khusus",
        }
    }

    /// Ukuran potret dalam milimeter.
    pub fn base_mm(self) -> (f32, f32) {
        match self {
            PaperPreset::A4 => (210.0, 297.0),
            PaperPreset::A3 => (297.0, 420.0),
            PaperPreset::A5 => (148.0, 210.0),
            PaperPreset::Letter => (215.9, 279.4),
            PaperPreset::Legal => (215.9, 355.6),
            PaperPreset::F4 => (210.0, 330.0),
            PaperPreset::Paper4R => (101.6, 152.4),
            PaperPreset::Paper5R => (127.0, 177.8),
            PaperPreset::Paper6R => (152.4, 203.2),
            PaperPreset::Custom => (210.0, 297.0),
        }
    }
}


// Photo size


#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PhotoPreset {
    Cm2x3,
    Cm3x4,
    Cm4x6,
    R2,
    R3,
    R4,
    R5,
    R6,
    Passport,
    Custom,
}

impl PhotoPreset {
    pub const ALL: [PhotoPreset; 10] = [
        PhotoPreset::Cm2x3,
        PhotoPreset::Cm3x4,
        PhotoPreset::Cm4x6,
        PhotoPreset::R2,
        PhotoPreset::R3,
        PhotoPreset::R4,
        PhotoPreset::R5,
        PhotoPreset::R6,
        PhotoPreset::Passport,
        PhotoPreset::Custom,
    ];

    pub fn label(self) -> &'static str {
        match self {
            PhotoPreset::Cm2x3 => "2 × 3 cm",
            PhotoPreset::Cm3x4 => "3 × 4 cm",
            PhotoPreset::Cm4x6 => "4 × 6 cm",
            PhotoPreset::R2 => "2R · 2,5 × 3,5 in",
            PhotoPreset::R3 => "3R · 3,5 × 5 in",
            PhotoPreset::R4 => "4R · 4 × 6 in",
            PhotoPreset::R5 => "5R · 5 × 7 in",
            PhotoPreset::R6 => "6R · 6 × 8 in",
            PhotoPreset::Passport => "Paspor · 3,5 × 4,5 cm",
            PhotoPreset::Custom => "Khusus",
        }
    }

    pub fn short_label(self) -> &'static str {
        match self {
            PhotoPreset::Cm2x3 => "2×3",
            PhotoPreset::Cm3x4 => "3×4",
            PhotoPreset::Cm4x6 => "4×6",
            PhotoPreset::R2 => "2R",
            PhotoPreset::R3 => "3R",
            PhotoPreset::R4 => "4R",
            PhotoPreset::R5 => "5R",
            PhotoPreset::R6 => "6R",
            PhotoPreset::Passport => "PASPOR",
            PhotoPreset::Custom => "KUSTOM",
        }
    }

    /// Basis ukuran potret dalam milimeter.
    pub fn base_mm(self) -> (f32, f32) {
        match self {
            PhotoPreset::Cm2x3 => (20.0, 30.0),
            PhotoPreset::Cm3x4 => (30.0, 40.0),
            PhotoPreset::Cm4x6 => (40.0, 60.0),
            PhotoPreset::R2 => (63.5, 88.9),
            PhotoPreset::R3 => (88.9, 127.0),
            PhotoPreset::R4 => (101.6, 152.4),
            PhotoPreset::R5 => (127.0, 177.8),
            PhotoPreset::R6 => (152.4, 203.2),
            PhotoPreset::Passport => (35.0, 45.0),
            PhotoPreset::Custom => (40.0, 60.0),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct PhotoSize {
    pub preset: PhotoPreset,
    pub custom_w_mm: f32,
    pub custom_h_mm: f32,
    pub landscape: bool,
}

impl Default for PhotoSize {
    fn default() -> Self {
        Self::from_preset(PhotoPreset::Cm3x4)
    }
}

impl PhotoSize {
    pub fn from_preset(preset: PhotoPreset) -> Self {
        let (w, h) = preset.base_mm();
        Self {
            preset,
            custom_w_mm: w,
            custom_h_mm: h,
            landscape: false,
        }
    }

    pub fn portrait_mm(self) -> (f32, f32) {
        match self.preset {
            PhotoPreset::Custom => {
                let (a, b) = (self.custom_w_mm.max(5.0), self.custom_h_mm.max(5.0));
                if a <= b {
                    (a, b)
                } else {
                    (b, a)
                }
            }
            _ => self.preset.base_mm(),
        }
    }

    /// Ukuran efektif setelah opsi mendatar.
    pub fn mm(self) -> (f32, f32) {
        let (w, h) = self.portrait_mm();
        if self.landscape {
            (h, w)
        } else {
            (w, h)
        }
    }

    pub fn short_label(self) -> &'static str {
        self.preset.short_label()
    }
}

// Margins


#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Margins {
    pub top_mm: f32,
    pub right_mm: f32,
    pub bottom_mm: f32,
    pub left_mm: f32,
}

impl Margins {
    pub fn all(mm: f32) -> Self {
        Self {
            top_mm: mm,
            right_mm: mm,
            bottom_mm: mm,
            left_mm: mm,
        }
    }

    pub fn horizontal(self) -> f32 {
        self.left_mm + self.right_mm
    }

    pub fn vertical(self) -> f32 {
        self.top_mm + self.bottom_mm
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MarginPreset {
    Normal,
    Narrow,
    Moderate,
    Wide,
    None,
    Custom,
}

impl MarginPreset {
    pub const ALL: [MarginPreset; 6] = [
        MarginPreset::Normal,
        MarginPreset::Narrow,
        MarginPreset::Moderate,
        MarginPreset::Wide,
        MarginPreset::None,
        MarginPreset::Custom,
    ];

    pub fn label(self) -> &'static str {
        match self {
            MarginPreset::Normal => "Normal  ·  2,54 cm",
            MarginPreset::Narrow => "Sempit  ·  1,27 cm",
            MarginPreset::Moderate => "Sedang  ·  2,54 / 1,91 cm",
            MarginPreset::Wide => "Lebar  ·  2,54 / 5,08 cm",
            MarginPreset::None => "Tanpa margin",
            MarginPreset::Custom => "Kustom",
        }
    }

    pub fn margins(self) -> Margins {
        match self {
            MarginPreset::Normal => Margins::all(25.4),
            MarginPreset::Narrow => Margins::all(12.7),
            MarginPreset::Moderate => Margins {
                top_mm: 25.4,
                right_mm: 19.05,
                bottom_mm: 25.4,
                left_mm: 19.05,
            },
            MarginPreset::Wide => Margins {
                top_mm: 25.4,
                right_mm: 50.8,
                bottom_mm: 25.4,
                left_mm: 50.8,
            },
            MarginPreset::None => Margins::all(0.0),
            MarginPreset::Custom => Margins::all(25.4),
        }
    }
}

// ---------------------------------------------------------------------------
// Layout settings
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PageOrientation {
    Portrait,
    Landscape,
    Auto,
}

impl PageOrientation {
    pub const ALL: [PageOrientation; 3] = [
        PageOrientation::Portrait,
        PageOrientation::Landscape,
        PageOrientation::Auto,
    ];

    pub fn label(self) -> &'static str {
        match self {
            PageOrientation::Portrait => "Potret",
            PageOrientation::Landscape => "Mendatar",
            PageOrientation::Auto => "Otomatis",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImageFit {
    /// Penuhi bingkai, sisi berlebih dipotong (paling umum untuk pas foto).
    Cover,
    /// Tampilkan seluruh foto, sisa ruang diberi putih.
    Contain,
}

impl ImageFit {
    pub fn label(self) -> &'static str {
        match self {
            ImageFit::Cover => "Penuhi bingkai",
            ImageFit::Contain => "Utuh (tanpa potong)",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CutStyle {
    None,
    Frame,
    Guillotine,
}

impl CutStyle {
    pub fn label(self) -> &'static str {
        match self {
            CutStyle::None => "Tanpa garis",
            CutStyle::Frame => "Garis bingkai",
            CutStyle::Guillotine => "Garis penuh (guillotine)",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PacketItem {
    pub size: PhotoSize,
    pub qty: u32,
    pub photo: Option<usize>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum SheetMode {
    Single,
    Packet,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SheetSettings {
    pub paper: PaperPreset,
    pub orientation: PageOrientation,
    pub custom_paper_w_mm: f32,
    pub custom_paper_h_mm: f32,

    pub mode: SheetMode,
    pub photo: PhotoSize,
    /// 0 = isi halaman otomatis.
    pub quantity: u32,
    pub batch_per_photo: bool,
    pub fit: ImageFit,
    pub packet: Vec<PacketItem>,

    pub margin_preset: MarginPreset,
    pub custom_margins: Margins,
    pub auto_center: bool,
    pub gap_mm: f32,
    pub cut_style: CutStyle,

    pub dpi: u32,
    pub footer_enabled: bool,
    pub footer: String,
}

impl Default for SheetSettings {
    fn default() -> Self {
        Self {
            paper: PaperPreset::A4,
            orientation: PageOrientation::Portrait,
            custom_paper_w_mm: 210.0,
            custom_paper_h_mm: 297.0,
            mode: SheetMode::Single,
            photo: PhotoSize::from_preset(PhotoPreset::Cm3x4),
            quantity: 0,
            batch_per_photo: false,
            fit: ImageFit::Cover,
            packet: vec![
                PacketItem {
                    size: PhotoSize::from_preset(PhotoPreset::Cm2x3),
                    qty: 8,
                    photo: None,
                },
                PacketItem {
                    size: PhotoSize::from_preset(PhotoPreset::Cm3x4),
                    qty: 4,
                    photo: None,
                },
                PacketItem {
                    size: PhotoSize::from_preset(PhotoPreset::Cm4x6),
                    qty: 2,
                    photo: None,
                },
            ],
            margin_preset: MarginPreset::Normal,
            custom_margins: Margins::all(25.4),
            auto_center: true,
            gap_mm: 3.0,
            cut_style: CutStyle::Frame,
            dpi: 300,
            footer_enabled: false,
            footer: String::new(),
        }
    }
}

impl SheetSettings {
    pub fn base_paper_mm(&self) -> (f32, f32) {
        match self.paper {
            PaperPreset::Custom => (
                self.custom_paper_w_mm.clamp(20.0, 2000.0),
                self.custom_paper_h_mm.clamp(20.0, 2000.0),
            ),
            other => other.base_mm(),
        }
    }

    pub fn base_margins(&self) -> Margins {
        match self.margin_preset {
            MarginPreset::Custom => Margins {
                top_mm: self.custom_margins.top_mm.clamp(0.0, 100.0),
                right_mm: self.custom_margins.right_mm.clamp(0.0, 100.0),
                bottom_mm: self.custom_margins.bottom_mm.clamp(0.0, 100.0),
                left_mm: self.custom_margins.left_mm.clamp(0.0, 100.0),
            },
            other => other.margins(),
        }
    }
}

// ---------------------------------------------------------------------------
// Small helpers
// ---------------------------------------------------------------------------

pub fn cut_style_uses_guides(style: CutStyle) -> bool {
    !matches!(style, CutStyle::None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paper_sizes_are_portrait() {
        for p in PaperPreset::ALL {
            let (w, h) = p.base_mm();
            assert!(w <= h, "{:?} harus potret", p);
        }
    }

    #[test]
    fn landscape_photo_swaps() {
        let mut s = PhotoSize::from_preset(PhotoPreset::Cm4x6);
        assert_eq!(s.mm(), (40.0, 60.0));
        s.landscape = true;
        assert_eq!(s.mm(), (60.0, 40.0));
    }

    #[test]
    fn word_margins() {
        assert!((MarginPreset::Normal.margins().left_mm - 25.4).abs() < 0.001);
        assert!((MarginPreset::Narrow.margins().top_mm - 12.7).abs() < 0.001);
        assert!((MarginPreset::None.margins().top_mm - 0.0).abs() < 0.001);
    }
}
