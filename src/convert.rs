use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

use lopdf::{dictionary, Document, Object};

use crate::docx::{self, DocxPage, DocxPageImage};
use crate::image_utils::{
    self, format_bytes, load_image, save_image, SUPPORTED_IMAGE_EXTS,
};
use crate::pdf_export;

pub const PDF_EXTS: &[&str] = &["pdf"];

#[derive(Clone, Debug, Default)]
pub struct ToolStatus {
    pub libreoffice: Option<PathBuf>,
    pub pdftoppm: Option<PathBuf>,
    pub pdftotext: Option<PathBuf>,
    pub mutool: Option<PathBuf>,
    pub magick: Option<PathBuf>,
    pub lp: Option<PathBuf>,
    pub lpstat: Option<PathBuf>,
}

impl ToolStatus {
    pub fn detect() -> Self {
        Self {
            libreoffice: find_soffice(),
            pdftoppm: find_in_path("pdftoppm"),
            pdftotext: find_in_path("pdftotext"),
            mutool: find_in_path("mutool"),
            magick: find_in_path("magick").or_else(|| find_in_path("convert")),
            lp: find_in_path("lp"),
            lpstat: find_in_path("lpstat"),
        }
    }

    pub fn pdf_renderer(&self) -> Option<&Path> {
        self.pdftoppm
            .as_deref()
            .or_else(|| self.mutool.as_deref())
            .or_else(|| self.magick.as_deref())
    }
}

fn find_in_path(name: &str) -> Option<PathBuf> {
    let path_var = std::env::var_os("PATH")?;
    let exts: Vec<String> = if cfg!(windows) {
        std::env::var("PATHEXT")
            .unwrap_or_else(|_| ".EXE;.CMD;.BAT".to_string())
            .split(';')
            .map(|s| s.to_ascii_lowercase())
            .collect()
    } else {
        vec![String::new()]
    };
    for dir in std::env::split_paths(&path_var) {
        for ext in &exts {
            let candidate = if ext.is_empty() {
                dir.join(name)
            } else {
                dir.join(format!("{name}{ext}"))
            };
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

fn find_soffice() -> Option<PathBuf> {
    if let Some(p) = find_in_path("soffice") {
        return Some(p);
    }
    let mut candidates: Vec<&str> = vec![
        "/usr/bin/soffice",
        "/usr/lib/libreoffice/program/soffice",
        "/opt/libreoffice/program/soffice",
        "/snap/bin/libreoffice",
        "/Applications/LibreOffice.app/Contents/MacOS/soffice",
    ];
    if cfg!(windows) {
        candidates.extend_from_slice(&[
            r"C:\Program Files\LibreOffice\program\soffice.exe",
            r"C:\Program Files (x86)\LibreOffice\program\soffice.exe",
        ]);
    }
    candidates
        .into_iter()
        .map(PathBuf::from)
        .find(|p| p.is_file())
}

fn command(program: &Path) -> Command {
    let mut cmd = Command::new(program);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    cmd.stdin(Stdio::null());
    cmd
}

fn run_capture(mut cmd: Command) -> Result<String, String> {
    let output = cmd
        .output()
        .map_err(|e| format!("Gagal menjalankan program: {e}"))?;
    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "Program gagal ({}): {}",
            output.status,
            err.trim().chars().take(500).collect::<String>()
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn unique_work_dir(tag: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let dir = std::env::temp_dir().join(format!(
        "cetakin-{}-{}-{}",
        tag,
        std::process::id(),
        nanos
    ));
    let _ = fs::create_dir_all(&dir);
    dir
}

fn collect_images_sorted(dir: &Path) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.is_file()
                && p.extension()
                    .and_then(|e| e.to_str())
                    .map(|e| {
                        let e = e.to_ascii_lowercase();
                        e == "png" || e == "jpg" || e == "jpeg"
                    })
                    .unwrap_or(false)
        })
        .collect();
    files.sort_by_key(|p| {
        let name = p.file_stem().and_then(OsStr::to_str).unwrap_or("");
        let num: String = name
            .chars()
            .rev()
            .take_while(|c| c.is_ascii_digit())
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();
        num.parse::<u32>().unwrap_or(u32::MAX)
    });
    files
}

#[derive(Clone, Debug, Default)]
pub struct ConvertOutcome {
    pub files: Vec<PathBuf>,
    pub note: String,
}

/// Render PDF ke PNG dengan pdftoppm / mutool / ImageMagick.
pub fn pdf_to_images(
    input: &Path,
    output_dir: &Path,
    dpi: u32,
    first_page: Option<u32>,
    last_page: Option<u32>,
    tools: &ToolStatus,
    progress: &mut dyn FnMut(f32, &str),
) -> Result<ConvertOutcome, String> {
    fs::create_dir_all(output_dir).map_err(|e| format!("Gagal membuat folder output: {e}"))?;
    let work = unique_work_dir("pdfimg");
    progress(0.15, "Merender halaman PDF…");

    let result = (|| -> Result<Vec<PathBuf>, String> {
        if let Some(pdftoppm) = &tools.pdftoppm {
            let mut cmd = command(pdftoppm);
            cmd.arg("-png").arg("-r").arg(dpi.to_string());
            if let Some(f) = first_page {
                cmd.arg("-f").arg(f.to_string());
            }
            if let Some(l) = last_page {
                cmd.arg("-l").arg(l.to_string());
            }
            cmd.arg(input).arg(work.join("halaman"));
            run_capture(cmd)?;
        } else if let Some(mutool) = &tools.mutool {
            let mut cmd = command(mutool);
            cmd.arg("draw")
                .arg("-r")
                .arg(dpi.to_string())
                .arg("-o")
                .arg(work.join("halaman-%d.png"));
            if let Some(f) = first_page {
                cmd.arg("-F").arg(f.to_string());
            }
            if let Some(l) = last_page {
                cmd.arg("-l").arg(l.to_string());
            }
            cmd.arg(input);
            run_capture(cmd)?;
        } else if let Some(magick) = &tools.magick {
            let mut cmd = command(magick);
            cmd.arg("-density").arg(dpi.to_string());
            if let Some(f) = first_page {
                cmd.arg("-density").arg(dpi.to_string());
                let _ = f;
            }
            // ImageMagick menulis semua halaman dengan pola nama.
            cmd.arg(input)
                .arg(work.join("halaman-%d.png"));
            run_capture(cmd)?;
        } else {
            return Err(
                "Tidak ada renderer PDF. Pasang Poppler (pdftoppm) atau MuPDF (mutool)."
                    .to_string(),
            );
        }
        Ok(collect_images_sorted(&work))
    })();

    let rendered = match result {
        Ok(files) => files,
        Err(e) => {
            let _ = fs::remove_dir_all(&work);
            return Err(e);
        }
    };

    if rendered.is_empty() {
        let _ = fs::remove_dir_all(&work);
        return Err("Tidak ada halaman yang berhasil dirender.".to_string());
    }

    let stem = input
        .file_stem()
        .and_then(OsStr::to_str)
        .unwrap_or("dokumen");
    let mut outputs = Vec::new();
    for (i, src) in rendered.iter().enumerate() {
        let name = format!("{stem}-hal-{:02}.png", i + 1);
        let dest = unique_destination(output_dir.join(name));
        fs::copy(src, &dest).map_err(|e| format!("Gagal menyalin hasil render: {e}"))?;
        outputs.push(dest);
        progress(
            0.2 + 0.8 * (i + 1) as f32 / rendered.len() as f32,
            &format!("Menyimpan halaman {}", i + 1),
        );
    }
    let _ = fs::remove_dir_all(&work);
    Ok(ConvertOutcome {
        files: outputs,
        note: format!("{} halaman dirender pada {} DPI.", rendered.len(), dpi),
    })
}

fn unique_destination(path: PathBuf) -> PathBuf {
    if !path.exists() {
        return path;
    }
    let stem = path
        .file_stem()
        .and_then(OsStr::to_str)
        .unwrap_or("hasil");
    let ext = path
        .extension()
        .and_then(OsStr::to_str)
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

/// Ekstrak teks per halaman. Memakai pdftotext bila ada, lalu pdf-extract.
pub fn extract_pdf_pages_text(input: &Path, tools: &ToolStatus) -> Vec<String> {
    if let Some(pdftotext) = &tools.pdftotext {
        let mut cmd = command(pdftotext);
        cmd.arg("-layout").arg("-enc").arg("UTF-8").arg(input).arg("-");
        if let Ok(text) = run_capture(cmd) {
            let pages: Vec<String> = text
                .split('\u{000c}')
                .map(|p| p.trim_end().to_string())
                .collect();
            if pages.iter().any(|p| !p.trim().is_empty()) {
                return trim_trailing_empty(pages);
            }
        }
    }
    match pdf_extract::extract_text(input) {
        Ok(text) => trim_trailing_empty(
            text.split('\u{000c}')
                .map(|p| p.to_string())
                .collect::<Vec<_>>(),
        ),
        Err(_) => vec![String::new()],
    }
}

fn trim_trailing_empty(mut pages: Vec<String>) -> Vec<String> {
    while pages.len() > 1 && pages.last().map(|p| p.trim().is_empty()).unwrap_or(false) {
        pages.pop();
    }
    if pages.is_empty() {
        pages.push(String::new());
    }
    pages
}

pub fn pdf_to_docx(
    input: &Path,
    output: &Path,
    include_images: bool,
    dpi: u32,
    tools: &ToolStatus,
    progress: &mut dyn FnMut(f32, &str),
) -> Result<ConvertOutcome, String> {
    progress(0.05, "Mengekstrak teks…");
    let texts = extract_pdf_pages_text(input, tools);
    let page_count = texts.len().max(1);

    let mut page_images: Vec<Option<DocxPageImage>> = vec![None; page_count];
    let mut note = if include_images {
        "Menyiapkan gambar halaman…".to_string()
    } else {
        String::new()
    };

    if include_images {
        if tools.pdf_renderer().is_none() {
            note = "Renderer PDF tidak ditemukan; DOCX dibuat dari teks saja.".to_string();
        } else {
            let work = unique_work_dir("pdfdocx");
            let render = (|| -> Result<(), String> {
                if let Some(pdftoppm) = &tools.pdftoppm {
                    let mut cmd = command(pdftoppm);
                    cmd.arg("-png")
                        .arg("-r")
                        .arg(dpi.to_string())
                        .arg(input)
                        .arg(work.join("hal"));
                    run_capture(cmd)?;
                } else if let Some(mutool) = &tools.mutool {
                    let mut cmd = command(mutool);
                    cmd.arg("draw")
                        .arg("-r")
                        .arg(dpi.to_string())
                        .arg("-o")
                        .arg(work.join("hal-%d.png"))
                        .arg(input);
                    run_capture(cmd)?;
                } else if let Some(magick) = &tools.magick {
                    let mut cmd = command(magick);
                    cmd.arg("-density")
                        .arg(dpi.to_string())
                        .arg(input)
                        .arg(work.join("hal-%d.png"));
                    run_capture(cmd)?;
                }
                Ok(())
            })();
            if let Err(e) = render {
                note = format!("Gambar halaman gagal dirender ({e}); DOCX berisi teks saja.");
            } else {
                let files = collect_images_sorted(&work);
                for (i, file) in files.iter().enumerate() {
                    if i >= page_images.len() {
                        break;
                    }
                    if let (Ok(bytes), Ok((w, h))) =
                        (fs::read(file), image::image_dimensions(file))
                    {
                        page_images[i] = Some(DocxPageImage {
                            png_bytes: bytes,
                            width_px: w,
                            height_px: h,
                        });
                    }
                    progress(
                        0.1 + 0.75 * (i + 1) as f32 / files.len().max(1) as f32,
                        &format!("Merender halaman {}", i + 1),
                    );
                }
            }
            let _ = fs::remove_dir_all(&work);
        }
    }

    progress(0.9, "Menulis DOCX…");
    let pages: Vec<DocxPage> = texts
        .into_iter()
        .enumerate()
        .map(|(i, text)| DocxPage {
            text,
            image: page_images.get(i).cloned().flatten(),
        })
        .collect();

    let title = input
        .file_stem()
        .and_then(OsStr::to_str)
        .unwrap_or("Dokumen");
    docx::write_pdf_import(output, title, &pages, include_images)?;
    progress(1.0, "Selesai");
    if note.is_empty() {
        note = "Teks berhasil diekstrak ke Word.".to_string();
    }
    Ok(ConvertOutcome {
        files: vec![output.to_path_buf()],
        note,
    })
}

/// Word / ODT → PDF melalui LibreOffice. Bila tidak ada LibreOffice,
/// DOCX sederhana dirender ulang sebagai teks.
pub fn docx_to_pdf(
    input: &Path,
    output_dir: &Path,
    tools: &ToolStatus,
    progress: &mut dyn FnMut(f32, &str),
) -> Result<ConvertOutcome, String> {
    fs::create_dir_all(output_dir).map_err(|e| format!("Gagal membuat folder output: {e}"))?;
    progress(0.1, "Menjalankan konversi…");

    let ext = input
        .extension()
        .and_then(OsStr::to_str)
        .unwrap_or("")
        .to_ascii_lowercase();

    if let Some(soffice) = &tools.libreoffice {
        let mut cmd = command(soffice);
        cmd.arg("--headless")
            .arg("--norestore")
            .arg("--convert-to")
            .arg("pdf")
            .arg("--outdir")
            .arg(output_dir)
            .arg(input);
        run_capture(cmd)?;
        let stem = input
            .file_stem()
            .and_then(OsStr::to_str)
            .unwrap_or("dokumen");
        let pdf = output_dir.join(format!("{stem}.pdf"));
        if pdf.is_file() {
            progress(1.0, "Selesai");
            return Ok(ConvertOutcome {
                files: vec![pdf],
                note: "Dikonversi dengan LibreOffice.".to_string(),
            });
        }
        return Err("LibreOffice selesai tetapi PDF tidak ditemukan.".to_string());
    }

    if ext == "docx" {
        progress(0.4, "LibreOffice tidak ada — memakai renderer teks…");
        let text = docx::extract_docx_text(input)?;
        let pages: Vec<String> = text
            .split('\u{000c}')
            .map(|s| s.to_string())
            .collect();
        let stem = input
            .file_stem()
            .and_then(OsStr::to_str)
            .unwrap_or("dokumen");
        let output = unique_destination(output_dir.join(format!("{stem}.pdf")));
        let mut cb = |p: f32, msg: &str| progress(0.4 + p * 0.6, msg);
        pdf_export::text_to_pdf(&output, stem, &pages, &mut cb)?;
        return Ok(ConvertOutcome {
            files: vec![output],
            note: "LibreOffice tidak terpasang. PDF dibuat dari teks saja (format asli tidak dipertahankan). Pasang LibreOffice untuk hasil terbaik.".to_string(),
        });
    }

    Err("Konversi format ini membutuhkan LibreOffice. Pasang LibreOffice lalu coba lagi."
        .to_string())
}

/// Gabungkan beberapa PDF menjadi satu.
pub fn merge_pdfs(
    inputs: &[PathBuf],
    output: &Path,
    progress: &mut dyn FnMut(f32, &str),
) -> Result<ConvertOutcome, String> {
    if inputs.len() < 2 {
        return Err("Pilih minimal dua file PDF untuk digabung.".to_string());
    }
    let mut docs = Vec::new();
    for (i, input) in inputs.iter().enumerate() {
        progress(0.1 + 0.4 * i as f32 / inputs.len() as f32, "Membaca PDF…");
        let doc = Document::load(input)
            .map_err(|e| format!("Gagal membaca {}: {e}", input.display()))?;
        docs.push(doc);
    }

    let mut out = Document::with_version("1.5");
    let mut max_id = 1u32;
    let mut page_ids = Vec::new();

    for mut doc in docs {
        doc.renumber_objects_with(max_id);
        max_id = doc.max_id + 1;
        let pages = doc.get_pages();
        for (_, pid) in &pages {
            page_ids.push(*pid);
        }
        for (id, object) in doc.objects {
            match object.type_name().unwrap_or("") {
                "Catalog" | "Pages" => {}
                _ => {
                    out.objects.insert(id, object);
                }
            }
        }
    }

    let pages_id = out.new_object_id();
    for pid in &page_ids {
        if let Ok(Object::Dictionary(dict)) = out.get_object_mut(*pid) {
            dict.set("Parent", pages_id);
        }
    }
    let kids: Vec<Object> = page_ids.iter().map(|id| Object::Reference(*id)).collect();
    out.objects.insert(
        pages_id,
        Object::Dictionary(dictionary! {
            "Type" => "Pages",
            "Kids" => kids,
            "Count" => page_ids.len() as i64,
        }),
    );
    let catalog_id = out.add_object(dictionary! {
        "Type" => "Catalog",
        "Pages" => pages_id,
    });
    out.trailer.set("Root", catalog_id);
    out.compress();
    progress(0.85, "Menulis PDF gabungan…");
    out.save(output)
        .map_err(|e| format!("Gagal menyimpan PDF gabungan: {e}"))?;
    progress(1.0, "Selesai");
    Ok(ConvertOutcome {
        files: vec![output.to_path_buf()],
        note: format!("{} halaman dari {} PDF digabung.", page_ids.len(), inputs.len()),
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResizeFormat {
    Jpeg,
    Png,
    WebP,
}

impl ResizeFormat {
    pub fn ext(self) -> &'static str {
        match self {
            ResizeFormat::Jpeg => "jpg",
            ResizeFormat::Png => "png",
            ResizeFormat::WebP => "webp",
        }
    }
}

/// Kompres & ubah ukuran banyak gambar sekaligus.
pub fn compress_images(
    inputs: &[PathBuf],
    output_dir: &Path,
    max_side_px: u32,
    quality: u8,
    format: ResizeFormat,
    progress: &mut dyn FnMut(f32, &str),
) -> Result<ConvertOutcome, String> {
    fs::create_dir_all(output_dir).map_err(|e| format!("Gagal membuat folder output: {e}"))?;
    let mut files = Vec::new();
    for (i, input) in inputs.iter().enumerate() {
        progress(
            i as f32 / inputs.len().max(1) as f32,
            &format!("Memproses {}", input.file_name().and_then(OsStr::to_str).unwrap_or("gambar")),
        );
        let mut img = load_image(input)?;
        if max_side_px > 0 {
            let (w, h) = image::GenericImageView::dimensions(&img);
            if w.max(h) > max_side_px {
                img = img.thumbnail(max_side_px, max_side_px);
            }
        }
        let stem = input
            .file_stem()
            .and_then(OsStr::to_str)
            .unwrap_or("gambar");
        let dest = unique_destination(output_dir.join(format!(
            "{stem}-kompres.{}",
            format.ext()
        )));
        save_image(&img, &dest, quality)?;
        files.push(dest);
    }
    progress(1.0, "Selesai");
    Ok(ConvertOutcome {
        files,
        note: format!(
            "{} gambar diproses. Kualitas {}%.",
            inputs.len(),
            quality
        ),
    })
}

/// Daftar printer (Linux CUPS / Windows PowerShell).
pub fn list_printers(tools: &ToolStatus) -> Vec<String> {
    #[cfg(windows)]
    {
        let mut cmd = Command::new("powershell");
        cmd.args([
            "-NoProfile",
            "-Command",
            "Get-Printer | Select-Object -ExpandProperty Name",
        ]);
        if let Ok(out) = run_capture(cmd) {
            return out
                .lines()
                .map(|l| l.trim().to_string())
                .filter(|l| !l.is_empty())
                .collect();
        }
        Vec::new()
    }
    #[cfg(not(windows))]
    {
        if let Some(lpstat) = &tools.lpstat {
            let mut cmd = command(lpstat);
            cmd.arg("-p");
            if let Ok(out) = run_capture(cmd) {
                return out
                    .lines()
                    .filter_map(|l| {
                        let rest = l.strip_prefix("printer ")?;
                        rest.split_whitespace().next().map(|s| s.to_string())
                    })
                    .collect();
            }
        }
        Vec::new()
    }
}

/// Kirim PDF ke printer default (Linux `lp`, Windows membuka dialog cetak).
pub fn print_pdf(pdf: &Path, tools: &ToolStatus) -> Result<String, String> {
    if cfg!(windows) {
        let status = Command::new("cmd")
            .args(["/C", "start", "", &pdf.to_string_lossy()])
            .status()
            .map_err(|e| format!("Gagal membuka PDF: {e}"))?;
        if status.success() {
            return Ok("PDF dibuka. Gunakan Ctrl+P untuk mencetak.".to_string());
        }
        return Err("Gagal membuka PDF.".to_string());
    }
    if let Some(lp) = &tools.lp {
        let mut cmd = command(lp);
        cmd.arg(pdf);
        let out = run_capture(cmd)?;
        return Ok(format!("Dikirim ke printer default. {}", out.trim()));
    }
    open::that_detached(pdf).map_err(|e| format!("Tidak ada printer/lp: {e}"))?;
    Ok("PDF dibuka. Gunakan Ctrl+P untuk mencetak.".to_string())
}

pub fn is_pdf_file(path: &Path) -> bool {
    path.extension()
        .and_then(OsStr::to_str)
        .map(|e| PDF_EXTS.contains(&e.to_ascii_lowercase().as_str()))
        .unwrap_or(false)
}

pub fn is_image_file(path: &Path) -> bool {
    path.extension()
        .and_then(OsStr::to_str)
        .map(|e| SUPPORTED_IMAGE_EXTS.contains(&e.to_ascii_lowercase().as_str()))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_extensions() {
        assert!(is_pdf_file(Path::new("a.PDF")));
        assert!(is_image_file(Path::new("a.JpEg")));
        assert!(!is_pdf_file(Path::new("a.docx")));
        assert!(!is_pdf_file(Path::new("pdf")));
    }

    #[test]
    fn unique_names() {
        let dir = unique_work_dir("test-unique");
        let first = unique_destination(dir.join("hasil.png"));
        fs::write(&first, b"x").unwrap();
        let second = unique_destination(dir.join("hasil.png"));
        assert_ne!(first, second);
        assert!(second.to_string_lossy().contains("hasil-2"));
        let _ = fs::remove_dir_all(dir);
    }
}
