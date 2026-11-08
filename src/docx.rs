use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

#[derive(Clone)]
pub struct DocxPageImage {
    pub png_bytes: Vec<u8>,
    pub width_px: u32,
    pub height_px: u32,
}

pub struct DocxPage {
    pub text: String,
    pub image: Option<DocxPageImage>,
}

fn xml_escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            c if c.is_control() && c != '\n' && c != '\t' => out.push(' '),
            c => out.push(c),
        }
    }
    out
}

fn paragraph(text: &str, bold: bool, size_half_pt: u32) -> String {
    let lines: Vec<&str> = text.split('\n').collect();
    let mut runs = String::new();
    for (i, line) in lines.iter().enumerate() {
        if i > 0 {
            runs.push_str("<w:br/>");
        }
        runs.push_str(&format!(
            "<w:t xml:space=\"preserve\">{}</w:t>",
            xml_escape(line)
        ));
    }
    format!(
        "<w:p><w:pPr><w:spacing w:after=\"100\" w:line=\"276\" w:lineRule=\"auto\"/></w:pPr>\
         <w:r><w:rPr><w:rFonts w:ascii=\"Arial\" w:hAnsi=\"Arial\"/>{}{}<w:sz w:val=\"{}\"/>\
         <w:szCs w:val=\"{}\"/></w:rPr>{}</w:r></w:p>",
        if bold { "<w:b/>" } else { "" },
        "",
        size_half_pt,
        size_half_pt,
        runs
    )
}

fn page_break() -> String {
    "<w:p><w:r><w:br w:type=\"page\"/></w:r></w:p>".to_string()
}

fn image_paragraph(rel_id: &str, doc_pr_id: u32, cx: u64, cy: u64) -> String {
    format!(
        "<w:p><w:pPr><w:jc w:val=\"center\"/><w:spacing w:before=\"0\" w:after=\"120\"/></w:pPr>\
         <w:r><w:drawing><wp:inline distT=\"0\" distB=\"0\" distL=\"0\" distR=\"0\">\
         <wp:extent cx=\"{cx}\" cy=\"{cy}\"/>\
         <wp:effectExtent l=\"0\" t=\"0\" r=\"0\" b=\"0\"/>\
         <wp:docPr id=\"{doc_pr_id}\" name=\"Halaman {doc_pr_id}\"/>\
         <wp:cNvGraphicFramePr><a:graphicFrameLocks noChangeAspect=\"1\"/></wp:cNvGraphicFramePr>\
         <a:graphic><a:graphicData uri=\"http://schemas.openxmlformats.org/drawingml/2006/picture\">\
         <pic:pic><pic:nvPicPr><pic:cNvPr id=\"{doc_pr_id}\" name=\"Halaman {doc_pr_id}\"/>\
         <pic:cNvPicPr/></pic:nvPicPr>\
         <pic:blipFill><a:blip r:embed=\"{rel_id}\"/><a:stretch><a:fillRect/></a:stretch></pic:blipFill>\
         <pic:spPr><a:xfrm><a:off x=\"0\" y=\"0\"/><a:ext cx=\"{cx}\" cy=\"{cy}\"/></a:xfrm>\
         <a:prstGeom prst=\"rect\"><a:avLst/></a:prstGeom></pic:spPr></pic:pic>\
         </a:graphicData></a:graphic></wp:inline></w:drawing></w:r></w:p>"
    )
}

/// Tulis DOCX hasil impor PDF. Bila gambar halaman tersedia, halaman PDF
/// ditampilkan utuh sebagai gambar, lalu teks hasil ekstraksi diletakkan
/// pada lampiran agar tetap bisa diedit.
pub fn write_pdf_import(
    output: &Path,
    title: &str,
    pages: &[DocxPage],
    include_images: bool,
) -> Result<(), String> {
    let mut document = String::new();
    document.push_str(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
         <w:document xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\" \
         xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\" \
         xmlns:wp=\"http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing\" \
         xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\" \
         xmlns:pic=\"http://schemas.openxmlformats.org/drawingml/2006/picture\">\
         <w:body>",
    );

    if !title.trim().is_empty() {
        document.push_str(&paragraph(title, true, 32));
    }

    let mut image_rels = Vec::new();
    let mut rel_counter = 10u32;
    let mut pr_id = 1u32;
    let emu_per_inch = 914_400f64;
    let content_width_in = 6.5f64;
    let max_height_in = 8.9f64;

    if include_images {
        for (i, page) in pages.iter().enumerate() {
            if i > 0 {
                document.push_str(&page_break());
            }
            if let Some(img) = &page.image {
                let ratio = if img.width_px > 0 {
                    img.height_px as f64 / img.width_px as f64
                } else {
                    1.4
                };
                let mut w_in = content_width_in;
                let mut h_in = w_in * ratio;
                if h_in > max_height_in {
                    h_in = max_height_in;
                    w_in = h_in / ratio;
                }
                let rel = format!("rId{rel_counter}");
                rel_counter += 1;
                image_rels.push((rel.clone(), i + 1));
                document.push_str(&image_paragraph(
                    &rel,
                    pr_id,
                    (w_in * emu_per_inch) as u64,
                    (h_in * emu_per_inch) as u64,
                ));
                pr_id += 1;
            }
        }

        document.push_str(&page_break());
        document.push_str(&paragraph("Teks hasil ekstraksi", true, 28));
        for (i, page) in pages.iter().enumerate() {
            if i > 0 {
                document.push_str(&page_break());
            }
            document.push_str(&paragraph(
                &format!("Halaman {}", i + 1),
                true,
                22,
            ));
            for block in page.text.split('\n') {
                if !block.trim().is_empty() {
                    document.push_str(&paragraph(block, false, 20));
                }
            }
        }
    } else {
        for (i, page) in pages.iter().enumerate() {
            if i > 0 {
                document.push_str(&page_break());
            }
            for block in page.text.split('\n') {
                if !block.trim().is_empty() {
                    document.push_str(&paragraph(block, false, 22));
                }
            }
        }
    }

    document.push_str(
        "<w:sectPr><w:pgSz w:w=\"11906\" w:h=\"16838\"/>\
         <w:pgMar w:top=\"1134\" w:right=\"1134\" w:bottom=\"1134\" w:left=\"1134\" \
         w:header=\"708\" w:footer=\"708\" w:gutter=\"0\"/>\
         <w:cols w:space=\"708\"/><w:docGrid w:linePitch=\"360\"/></w:sectPr>\
         </w:body></w:document>",
    );

    let mut rels = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
         <Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\
         <Relationship Id=\"rId1\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles\" Target=\"styles.xml\"/>",
    );
    for (rel, page_no) in &image_rels {
        rels.push_str(&format!(
            "<Relationship Id=\"{rel}\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/image\" Target=\"media/halaman-{page_no}.png\"/>"
        ));
    }
    rels.push_str("</Relationships>");

    let styles = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
        <w:styles xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\
        <w:docDefaults><w:rPrDefault><w:rPr>\
        <w:rFonts w:ascii=\"Arial\" w:hAnsi=\"Arial\"/>\
        <w:sz w:val=\"22\"/><w:szCs w:val=\"22\"/>\
        </w:rPr></w:rPrDefault></w:docDefaults>\
        <w:style w:type=\"paragraph\" w:default=\"1\" w:styleId=\"Normal\">\
        <w:name w:val=\"Normal\"/><w:qFormat/></w:style></w:styles>";

    let content_types = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
        <Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\">\
        <Default Extension=\"rels\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/>\
        <Default Extension=\"xml\" ContentType=\"application/xml\"/>\
        <Default Extension=\"png\" ContentType=\"image/png\"/>\
        <Default Extension=\"jpeg\" ContentType=\"image/jpeg\"/>\
        <Default Extension=\"jpg\" ContentType=\"image/jpeg\"/>\
        <Override PartName=\"/word/document.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml\"/>\
        <Override PartName=\"/word/styles.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml\"/>\
        </Types>";

    let root_rels = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
        <Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\
        <Relationship Id=\"rId1\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument\" Target=\"word/document.xml\"/>\
        </Relationships>";

    let file = File::create(output).map_err(|e| format!("Gagal membuat DOCX: {e}"))?;
    let mut zip = ZipWriter::new(file);
    let opts = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    let write = |zip: &mut ZipWriter<File>, name: &str, data: &[u8]| -> Result<(), String> {
        zip.start_file(name, opts)
            .map_err(|e| format!("Gagal menulis DOCX: {e}"))?;
        zip.write_all(data)
            .map_err(|e| format!("Gagal menulis DOCX: {e}"))
    };

    write(&mut zip, "[Content_Types].xml", content_types.as_bytes())?;
    write(&mut zip, "_rels/.rels", root_rels.as_bytes())?;
    write(&mut zip, "word/document.xml", document.as_bytes())?;
    write(&mut zip, "word/styles.xml", styles.as_bytes())?;
    write(
        &mut zip,
        "word/_rels/document.xml.rels",
        rels.as_bytes(),
    )?;
    if include_images {
        for (i, page) in pages.iter().enumerate() {
            if let Some(img) = &page.image {
                write(
                    &mut zip,
                    &format!("word/media/halaman-{}.png", i + 1),
                    &img.png_bytes,
                )?;
            }
        }
    }
    zip.finish().map_err(|e| format!("Gagal finalisasi DOCX: {e}"))?;
    Ok(())
}

/// Ekstrak teks dari DOCX (fallback ketika LibreOffice tidak terpasang).
/// Mengembalikan teks dengan `\n` untuk paragraf dan `\u{000c}` untuk
/// perpindahan halaman.
pub fn extract_docx_text(path: &Path) -> Result<String, String> {
    let file = File::open(path).map_err(|e| format!("Gagal membuka DOCX: {e}"))?;
    let mut zip = ZipArchive::new(file).map_err(|e| format!("DOCX tidak valid: {e}"))?;
    let mut xml = String::new();
    {
        let mut doc = zip
            .by_name("word/document.xml")
            .map_err(|_| "File bukan DOCX yang dikenali.".to_string())?;
        doc.read_to_string(&mut xml)
            .map_err(|e| format!("Gagal membaca DOCX: {e}"))?;
    }
    Ok(decode_word_text(&xml))
}

fn decode_word_text(xml: &str) -> String {
    let mut out = String::new();
    let mut rest = xml;
    let mut in_text = false;
    loop {
        if !in_text {
            let Some(pos) = rest.find('<') else {
                break;
            };
            // Teks di luar tag diabaikan (spasi antar tag).
            rest = &rest[pos..];
            if let Some(end) = rest.find('>') {
                let tag = &rest[..=end];
                if tag.starts_with("<w:t") {
                    in_text = true;
                } else if tag.starts_with("<w:br") && tag.contains("page") {
                    out.push('\u{000c}');
                } else if tag.starts_with("</w:p") {
                    out.push('\n');
                } else if tag.starts_with("<w:tab") {
                    out.push('\t');
                }
                rest = &rest[end + 1..];
            } else {
                break;
            }
        } else {
            let Some(end) = rest.find("</w:t>") else {
                break;
            };
            out.push_str(&decode_entities(&rest[..end]));
            rest = &rest[end + 6..];
            in_text = false;
        }
    }
    // Rapikan baris kosong beruntun.
    let mut cleaned = String::new();
    let mut blank = 0;
    for line in out.lines() {
        if line.trim().is_empty() {
            blank += 1;
            if blank > 1 {
                continue;
            }
        } else {
            blank = 0;
        }
        cleaned.push_str(line);
        cleaned.push('\n');
    }
    cleaned.trim().to_string()
}

fn decode_entities(text: &str) -> String {
    let mut out = text
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'");
    // &#123; dan &#x1F600;
    while let Some(start) = out.find("&#") {
        let Some(end_rel) = out[start..].find(';') else {
            break;
        };
        let end = start + end_rel;
        let entity = &out[start + 2..end];
        let parsed = if let Some(hex) = entity.strip_prefix('x').or_else(|| entity.strip_prefix('X')) {
            u32::from_str_radix(hex, 16).ok()
        } else {
            entity.parse::<u32>().ok()
        };
        if let Some(code) = parsed.and_then(char::from_u32) {
            out.replace_range(start..=end, &code.to_string());
        } else {
            out.replace_range(start..=end, "");
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_docx_style_text() {
        let xml = "<w:body><w:p><w:r><w:t>Halo &amp; dunia</w:t></w:r></w:p>\
                   <w:p><w:r><w:br w:type=\"page\"/><w:t>Halaman 2</w:t></w:r></w:p></w:body>";
        let text = decode_word_text(xml);
        assert!(text.contains("Halo & dunia"));
        assert!(text.contains('\u{000c}'));
        assert!(text.contains("Halaman 2"));
    }

    #[test]
    fn decodes_numeric_entities() {
        assert_eq!(decode_entities("A&#65;B&#x42;C"), "ABBC");
    }
}
