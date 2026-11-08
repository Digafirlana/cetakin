use std::collections::HashMap;
use std::fs::File;
use std::io::BufWriter;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use image::DynamicImage;
use printpdf::{
    BuiltinFont, Color, Image, ImageTransform, Line, Mm, PdfDocument, PdfDocumentReference,
    PdfLayerReference, Point, Rgb,
};

use crate::image_utils::{load_image, prepare_for_cell};
use crate::layout::{cut_segments, SheetLayout};
use crate::model::{cut_style_uses_guides, ImageFit, PageOrientation, SheetSettings};

type Progress<'a> = dyn FnMut(f32, &str) + 'a;

fn line_color(layer: &PdfLayerReference) {
    layer.set_outline_color(Color::Rgb(Rgb::new(0.62, 0.64, 0.68, None)));
    layer.set_outline_thickness(0.35);
}

fn draw_cut_guides(layer: &PdfLayerReference, layout: &SheetLayout, settings: &SheetSettings) {
    if !cut_style_uses_guides(settings.cut_style) {
        return;
    }
    line_color(layer);
    for (x0, y0, x1, y1) in cut_segments(layout, settings.cut_style) {
        let line = Line {
            points: vec![
                (Point::new(Mm(x0), Mm(layout.page_h_mm - y0)), false),
                (Point::new(Mm(x1), Mm(layout.page_h_mm - y1)), false),
            ],
            is_closed: false,
        };
        layer.add_line(line);
    }
}

fn draw_missing_photo(layer: &PdfLayerReference, x: f32, y: f32, w: f32, h: f32, page_h: f32) {
    layer.set_outline_color(Color::Rgb(Rgb::new(0.78, 0.80, 0.84, None)));
    layer.set_outline_thickness(0.4);
    let rect = Line {
        points: vec![
            (Point::new(Mm(x), Mm(page_h - y)), false),
            (Point::new(Mm(x + w), Mm(page_h - y)), false),
            (Point::new(Mm(x + w), Mm(page_h - y - h)), false),
            (Point::new(Mm(x), Mm(page_h - y - h)), false),
            (Point::new(Mm(x), Mm(page_h - y)), false),
        ],
        is_closed: false,
    };
    layer.add_line(rect);
    let cross = Line {
        points: vec![
            (Point::new(Mm(x), Mm(page_h - y)), false),
            (Point::new(Mm(x + w), Mm(page_h - y - h)), false),
            (Point::new(Mm(x + w), Mm(page_h - y)), false),
            (Point::new(Mm(x), Mm(page_h - y - h)), false),
        ],
        is_closed: false,
    };
    layer.add_line(cross);
}

fn draw_footer(
    layer: &PdfLayerReference,
    font: Option<&printpdf::IndirectFontRef>,
    settings: &SheetSettings,
    page_w: f32,
    page_h: f32,
) {
    if !settings.footer_enabled {
        return;
    }
    let text = settings.footer.trim();
    if text.is_empty() {
        return;
    }
    let Some(font) = font else {
        return;
    };
    let size_pt = 8.0f32;
    let approx_char_mm = size_pt * 0.3528 * 0.52;
    let width = text.chars().count() as f32 * approx_char_mm;
    let x = ((page_w - width) / 2.0).max(5.0);
    let y = (page_h - settings.base_margins().bottom_mm / 2.0 - 3.0)
        .max(4.0)
        .min(page_h - 4.0);
    layer.set_fill_color(Color::Rgb(Rgb::new(0.35, 0.37, 0.40, None)));
    layer.use_text(sanitize_text(text), size_pt, Mm(x), Mm(y), font);
}

/// Tulis satu lembar (satu halaman) foto. `photos` harus sejajar dengan
/// `layout.cells`.
#[allow(clippy::too_many_arguments)]
fn render_sheet_page(
    doc: &PdfDocumentReference,
    page_idx: printpdf::PdfPageIndex,
    layer_idx: printpdf::PdfLayerIndex,
    layout: &SheetLayout,
    photos: &[Option<PathBuf>],
    settings: &SheetSettings,
    image_cache: &mut HashMap<PathBuf, Rc<DynamicImage>>,
    prepared_cache: &mut HashMap<(PathBuf, i32, i32, u8), Rc<DynamicImage>>,
    footer_font: Option<&printpdf::IndirectFontRef>,
) -> Result<(), String> {
    let layer = doc.get_page(page_idx).get_layer(layer_idx);
    let dpi = settings.dpi.clamp(72, 1200);
    let fit = settings.fit;

    for (idx, cell) in layout.cells.iter().enumerate() {
        let photo = photos.get(idx).and_then(|p| p.as_ref());
        let Some(path) = photo else {
            draw_missing_photo(
                &layer,
                cell.x_mm,
                cell.y_mm,
                cell.w_mm,
                cell.h_mm,
                layout.page_h_mm,
            );
            continue;
        };

        let key = (
            path.clone(),
            (cell.w_mm * 10.0).round() as i32,
            (cell.h_mm * 10.0).round() as i32,
            if fit == ImageFit::Cover { 0 } else { 1 },
        );
        let prepared = if let Some(img) = prepared_cache.get(&key) {
            img.clone()
        } else {
            let source = if let Some(img) = image_cache.get(path) {
                img.clone()
            } else {
                let img = Rc::new(load_image(path)?);
                image_cache.insert(path.clone(), img.clone());
                img
            };
            let img = Rc::new(prepare_for_cell(
                &source, cell.w_mm, cell.h_mm, dpi, fit,
            ));
            prepared_cache.insert(key, img.clone());
            img
        };

        let image = Image::from_dynamic_image(&prepared);
        image.add_to_layer(
            layer.clone(),
            ImageTransform {
                translate_x: Some(Mm(cell.x_mm)),
                translate_y: Some(Mm(layout.page_h_mm - cell.y_mm - cell.h_mm)),
                dpi: Some(dpi as f32),
                ..Default::default()
            },
        );
    }

    draw_cut_guides(&layer, layout, settings);
    draw_footer(
        &layer,
        footer_font,
        settings,
        layout.page_w_mm,
        layout.page_h_mm,
    );
    Ok(())
}

/// Ekspor satu atau beberapa halaman lembar foto ke PDF.
pub fn export_sheet_pdf(
    output: &Path,
    settings: &SheetSettings,
    layout: &SheetLayout,
    pages: &[Vec<Option<PathBuf>>],
    progress: &mut Progress<'_>,
) -> Result<(), String> {
    if pages.is_empty() {
        return Err("Tidak ada foto untuk dicetak.".into());
    }

    let (pw, ph) = (layout.page_w_mm, layout.page_h_mm);
    let (doc, first_page, first_layer) =
        PdfDocument::new("Cetakin - Lembar Foto", Mm(pw), Mm(ph), "Foto");

    let mut image_cache: HashMap<PathBuf, Rc<DynamicImage>> = HashMap::new();
    let mut prepared_cache: HashMap<(PathBuf, i32, i32, u8), Rc<DynamicImage>> = HashMap::new();
    let footer_font = if settings.footer_enabled && !settings.footer.trim().is_empty() {
        doc.add_builtin_font(BuiltinFont::Helvetica).ok()
    } else {
        None
    };

    for (page_no, photos) in pages.iter().enumerate() {
        let (page_idx, layer_idx) = if page_no == 0 {
            (first_page, first_layer)
        } else {
            doc.add_page(Mm(pw), Mm(ph), "Foto")
        };
        render_sheet_page(
            &doc,
            page_idx,
            layer_idx,
            layout,
            photos,
            settings,
            &mut image_cache,
            &mut prepared_cache,
            footer_font.as_ref(),
        )?;
        progress(
            (page_no + 1) as f32 / pages.len() as f32,
            &format!("Merender halaman {} dari {}", page_no + 1, pages.len()),
        );
    }

    save_pdf(doc, output)
}

fn save_pdf(doc: PdfDocumentReference, output: &Path) -> Result<(), String> {
    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("Gagal membuat folder: {e}"))?;
    }
    let file = File::create(output).map_err(|e| format!("Gagal membuat PDF: {e}"))?;
    let mut writer = BufWriter::new(file);
    doc.save(&mut writer)
        .map_err(|e| format!("Gagal menulis PDF: {e}"))?;
    Ok(())
}

/// Gabungkan beberapa gambar menjadi PDF, satu gambar per halaman.
#[allow(clippy::too_many_arguments)]
pub fn images_to_pdf(
    output: &Path,
    images: &[PathBuf],
    settings: &SheetSettings,
    fit: ImageFit,
    margin_mm: f32,
    dpi: u32,
    progress: &mut Progress<'_>,
) -> Result<(), String> {
    if images.is_empty() {
        return Err("Belum ada gambar yang dipilih.".into());
    }
    let (base_w, base_h) = settings.base_paper_mm();
    let mut doc: Option<PdfDocumentReference> = None;
    let mut first_page = None;

    for (i, path) in images.iter().enumerate() {
        let src = load_image(path)?;
        let (iw, ih) = image::GenericImageView::dimensions(&src);
        let (pw, ph) = match settings.orientation {
            PageOrientation::Portrait => (base_w.min(base_h), base_w.max(base_h)),
            PageOrientation::Landscape => (base_h.max(base_w), base_h.min(base_w)),
            PageOrientation::Auto => {
                if iw > ih {
                    (base_h.max(base_w), base_h.min(base_w))
                } else {
                    (base_w.min(base_h), base_w.max(base_h))
                }
            }
        };
        let area_w = (pw - margin_mm * 2.0).max(10.0);
        let area_h = (ph - margin_mm * 2.0).max(10.0);
        let prepared = prepare_for_cell(&src, area_w, area_h, dpi, fit);

        if doc.is_none() {
            let (d, p, l) = PdfDocument::new("Cetakin - Gambar ke PDF", Mm(pw), Mm(ph), "Gambar");
            doc = Some(d);
            first_page = Some((p, l));
        }
        let d = doc.as_ref().unwrap();
        let (page_idx, layer_idx) = if i == 0 {
            first_page.unwrap()
        } else {
            d.add_page(Mm(pw), Mm(ph), "Gambar")
        };
        let layer = d.get_page(page_idx).get_layer(layer_idx);
        let img = Image::from_dynamic_image(&prepared);
        img.add_to_layer(
            layer,
            ImageTransform {
                translate_x: Some(Mm((pw - area_w) / 2.0)),
                translate_y: Some(Mm((ph - area_h) / 2.0)),
                dpi: Some(dpi as f32),
                ..Default::default()
            },
        );
        progress(
            (i + 1) as f32 / images.len() as f32,
            &format!("Menambahkan {} / {}", i + 1, images.len()),
        );
    }

    save_pdf(doc.ok_or("Tidak ada gambar.")?, output)
}

/// Render teks sederhana ke PDF (fallback bila LibreOffice tidak tersedia).
pub fn text_to_pdf(
    output: &Path,
    title: &str,
    pages_text: &[String],
    progress: &mut Progress<'_>,
) -> Result<(), String> {
    let (doc, first_page, first_layer) =
        PdfDocument::new(title, Mm(210.0), Mm(297.0), "Teks");
    let font = doc
        .add_builtin_font(BuiltinFont::Helvetica)
        .map_err(|e| format!("Font PDF gagal: {e}"))?;
    let bold = doc
        .add_builtin_font(BuiltinFont::HelveticaBold)
        .map_err(|e| format!("Font PDF gagal: {e}"))?;

    let margin = 20.0;
    let max_width_mm = 210.0 - margin * 2.0;
    let line_h: f32 = 5.2;
    let max_lines = ((297.0 - margin * 2.0 - 12.0) / line_h).floor() as usize;

    let mut total_pages = 0usize;
    for (i, text) in pages_text.iter().enumerate() {
        let (page_idx, layer_idx) = if i == 0 {
            (first_page, first_layer)
        } else {
            doc.add_page(Mm(210.0), Mm(297.0), "Teks")
        };
        total_pages += 1;
        let layer = doc.get_page(page_idx).get_layer(layer_idx);
        let mut y = 297.0 - margin;
        if i == 0 && !title.is_empty() {
            layer.set_fill_color(Color::Rgb(Rgb::new(0.12, 0.13, 0.15, None)));
            layer.use_text(sanitize_text(title), 15.0, Mm(margin), Mm(y - 4.0), &bold);
            y -= 12.0;
        }
        layer.set_fill_color(Color::Rgb(Rgb::new(0.15, 0.16, 0.18, None)));
        let mut line_count = 0usize;
        for line in wrap_text(text, max_width_mm) {
            if line_count >= max_lines {
                break;
            }
            layer.use_text(sanitize_text(&line), 10.0, Mm(margin), Mm(y), &font);
            y -= line_h;
            line_count += 1;
        }
        progress(
            (i + 1) as f32 / pages_text.len().max(1) as f32,
            &format!("Menulis halaman {}", i + 1),
        );
    }
    if total_pages == 0 {
        return Err("Tidak ada teks untuk ditulis.".into());
    }
    save_pdf(doc, output)
}

/// Bungkus teks berdasarkan perkiraan lebar huruf Helvetica 10pt.
fn wrap_text(text: &str, max_width_mm: f32) -> Vec<String> {
    let char_mm = 10.0 * 0.3528 * 0.50;
    let max_chars = ((max_width_mm / char_mm).floor() as usize).max(20);
    let mut out = Vec::new();
    for raw_line in text.replace('\r', "").split('\n') {
        if raw_line.trim().is_empty() {
            out.push(String::new());
            continue;
        }
        let mut current = String::new();
        for word in raw_line.split_whitespace() {
            let extra = if current.is_empty() {
                word.chars().count()
            } else {
                word.chars().count() + 1
            };
            if current.chars().count() + extra > max_chars && !current.is_empty() {
                out.push(std::mem::take(&mut current));
            }
            if !current.is_empty() {
                current.push(' ');
            }
            // Kata yang sangat panjang dipotong paksa.
            let mut word = word.to_string();
            while word.chars().count() > max_chars {
                let head: String = word.chars().take(max_chars).collect();
                out.push(head);
                word = word.chars().skip(max_chars).collect();
            }
            current.push_str(&word);
        }
        if !current.is_empty() {
            out.push(current);
        }
    }
    if out.is_empty() {
        out.push(String::new());
    }
    out
}

/// Builtin font printpdf memakai Windows-1252; ganti karakter di luar itu.
pub fn sanitize_text(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        let mapped = match c {
            '\u{2013}' | '\u{2014}' => '-',
            '\u{2018}' | '\u{2019}' => '\'',
            '\u{201c}' | '\u{201d}' => '"',
            '\u{2022}' => '-',
            '\u{2026}' => "...".to_string().chars().next().unwrap_or('.'),
            '\u{00a0}' => ' ',
            _ if c.is_ascii() => c,
            _ if (c as u32) <= 0xff => c,
            _ => '?',
        };
        out.push(mapped);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wraps_long_text() {
        let text = "a ".repeat(300);
        let lines = wrap_text(&text, 170.0);
        assert!(lines.len() > 2);
        assert!(lines.iter().all(|l| l.chars().count() <= 100));
    }

    #[test]
    fn sanitize_non_latin() {
        assert_eq!(sanitize_text("Halo — dunia 日本語"), "Halo - dunia ???");
    }
}
