# cetakin.

Aplikasi desktop **digital printing** untuk menyusun pas foto di atas kertas, mengekspor PDF siap cetak, dan mengonversi dokumen (PDF, Word, gambar) dalam satu alur kerja.

Dibuat dengan **Rust + eframe/egui** (renderer glow/OpenGL). Berjalan di **Windows** dan **Linux**.

---

## Fitur

### 1. Cetak Foto
- Ukuran foto: **2×3, 3×4, 4×6**, 2R–6R, paspor, dan ukuran kustom.
- Mode **satuan** dan **paket campuran** (beberapa ukuran dalam satu lembar).
- Kertas A3/A4/A5, Letter, Legal, F4/Folio, kertas 4R/5R/6R, dan kustom.
- Orientasi potret, mendatar, atau otomatis.
- Margin gaya Word: **Normal, Sempit, Sedang, Lebar, Tanpa, Kustom** + pusatkan grid otomatis.
- Jarak antar foto, garis potong (bingkai / guillotine), footer nama studio.
- Rotasi per foto 0°/90°/180°/270°; pratinjau dan PDF mengikuti.
- Pratinjau halaman real-time dengan zoom, pan, dan panduan margin.
- Batch **satu halaman per foto** untuk mencetak banyak pelanggan sekaligus.
- Ekspor PDF 150/300/600 DPI dengan gambar ter-crop sesuai bingkai.
- Cetak langsung: Linux via `lp` (CUPS), Windows membuka PDF di penampil untuk Ctrl+P.
- Statistik lembar: jumlah foto, margin efektif, area terpakai, estimasi DPI minimum.

### 2. Pemindai Dokumen
- **Koreksi perspektif** otomatis: foto dokumen miring dirapikan menjadi persegi.
- Deteksi tepi kertas otomatis, dengan **4 titik sudut yang bisa digeser** manual.
- **Pemutihan kertas** dan perataan pencahayaan, termasuk bayangan dan tint warna.
- Preset **Warna**, **Dokumen** (abu-abu tajam), dan **Hitam Putih**.
- Rotasi 90°, pratinjau hasil real-time, simpan **PNG/JPEG/PDF**.
- Hasil scan bisa langsung **ditambahkan ke Cetak Foto**.

### 3. Konversi
- **Gambar → PDF** (satu gambar per halaman).
- **PDF → PNG** (render 150/300/600 DPI, rentang halaman opsional).
- **PDF → Word** (teks yang bisa diedit + gambar halaman opsional).
- **Word → PDF** (LibreOffice; fallback renderer teks bawaan).
- **Combine menjadi PDF** — gabungkan gambar, PDF, dan Word/ODT (bisa campur) menjadi satu PDF sesuai urutan.
- **Kompres gambar** (resize sisi terpanjang, kualitas, JPEG/PNG/WebP).
- Daftar file drag & drop, folder output, progress non-blocking, hasil bisa langsung dibuka.

### 4. Alat & Diagnostik
- **Inspektur kualitas foto**: dimensi, ukuran file, estimasi DPI untuk 2×3/3×4/4×6 + rekomendasi.
- **Kalkulator kapasitas kertas**: grid & jumlah foto per lembar, tabel perbandingan cepat.
- **Daftar printer** dan cetak PDF terakhir.
- **Status komponen pendukung** (LibreOffice, Poppler, MuPDF, ImageMagick, CUPS).

### 5. Antarmuka
- **Dashboard modern**: latar lavender bergradasi, sidebar kartu melayang, banner sambutan,
  kartu statistik, dan permukaan putih membulat dengan bayangan lembut.
- **Dua mode tampilan**: Terang dan Gelap bergaya macOS Tahoe, aksen biru sistem —
  pilih langsung di tab **Pengaturan → Mode Tampilan**. Pilihan tersimpan bersama
  preferensi dan langsung diterapkan tanpa perlu restart.
- **Dynamic Island** di tengah atas: progress job serta toast sukses/error/info.
- **Riwayat sesi**: hasil cetak, simpan PDF, konversi, dan pindai bisa dibuka kembali.
- **Notifikasi adaptif**: kapsul membulat penuh, warna lembut, aksen hijau/merah/biru sesuai status.
- Ringkas saat berjalan, membesar saat hover/klik; error tetap terbuka sampai ditutup.
- Animasi buka/tutup halus tanpa dependency tambahan.

### 5. Pengaturan
- **Mode tampilan** Terang / Gelap (gaya macOS Tahoe).
- Nama studio dan footer.
- Bawaan kertas, orientasi, ukuran foto, margin, DPI, auto-center, batch.
- Simpan/reset preferensi, lokasi file konfigurasi, hasil terakhir.
- Panduan dependensi dan daftar shortcut.

---

## Persyaratan

### Linux (Debian/Ubuntu)
```bash
sudo apt update
sudo apt install build-essential pkg-config libgtk-3-dev \
  poppler-utils libreoffice imagemagick cups-client
```

- `libgtk-3-dev` dibutuhkan oleh dialog file (`rfd`).
- `poppler-utils` menyediakan `pdftoppm` dan `pdftotext`.
- `libreoffice` untuk konversi Word → PDF dengan layout asli.
- `imagemagick`, `cups-client` bersifat opsional.

### Windows
- **Rust toolchain** (MSVC): https://rustup.rs
- **Build Tools for Visual Studio** (C++ workload).
- Opsional: LibreOffice, Poppler for Windows, ImageMagick — tambahkan foldernya ke `PATH`.

---

## Build & Jalankan

```bash
# Jalankan langsung (development)
cargo run

# Build release
cargo build --release

# Uji unit
cargo test
```

Binary hasil build:

- Linux: `target/release/cetakin`
- Windows: `target\release\cetakin.exe`

Untuk build yang lebih cepat di laptop/PC ringan:

```bash
CARGO_BUILD_JOBS=1 cargo build --release
```

### Ukuran binary

Profil `dev` tidak menyertakan debug info, sehingga `cargo run` menghasilkan
binary sekitar **25 MB** (sebelumnya ~380 MB). Profil `release` juga sudah
memakai `strip`, jadi binary distribusi jauh lebih kecil lagi.

Jika butuh backtrace dengan nomor baris saat debugging, jalankan sekali dengan
debug info tanpa mengubah pengaturan permanen:

```bash
CARGO_PROFILE_DEV_DEBUG=2 CARGO_PROFILE_DEV_STRIP=none cargo run
```

---

## Shortcut

| Tombol | Aksi |
| --- | --- |
| `Ctrl + O` | Impor foto |
| `Ctrl + S` | Simpan PDF |
| `Ctrl + P` | Cetak |
| `Esc` | Batalkan dialog konfirmasi |

Drag & drop file ke jendela:
- Tab **Cetak Foto** → gambar masuk ke daftar foto, dokumen dialihkan ke Konversi.
- Tab **Pemindai** → gambar pertama dimuat sebagai sumber scan.
- Tab **Konversi** → semua file didukung ditambahkan.
- Tab **Alat** → gambar pertama dipilih sebagai objek inspeksi.

---

## Catatan Teknis

- PDF dibuat dengan `printpdf`; PDF → Word memakai `lopdf` + penulis OOXML minimal (`zip`).
- Konversi berat dijalankan di thread latar agar UI tetap responsif.
- Preferensi disimpan sebagai JSON di:
  - Linux: `~/.config/cetakin/prefs.json`
  - Windows: `%APPDATA%\cetakin\prefs.json`
- Tanpa komponen eksternal, aplikasi tetap dapat menyusun foto, membuat PDF, dan mengompres gambar. Konversi Word akan memakai renderer teks internal.

---

## Struktur Proyek

```
src/
  main.rs         Bootstrap jendela, ikon, eframe
  app.rs          State aplikasi, Tab, handler shortcut & drag-drop
  photos.rs       Daftar foto, rotasi, thumbnail, pemetaan ke layout
  jobs.rs         Job latar belakang & toast notifikasi
  prefs.rs        Muat, simpan, dan reset preferensi
  ui_chrome.rs    Sidebar, header, banner hero, dan Dynamic Island
  ui_common.rs    Widget UI bersama (kartu, slider, dashed rect)
  util.rs         Helper path & filesystem
  model.rs        Tipe pengaturan, preset kertas/foto/margin
  layout.rs       Mesin tata letak halaman & garis potong
  image_utils.rs  EXIF, thumbnail, DPI, crop/contain
  pdf_export.rs   Ekspor lembar foto & PDF teks
  convert.rs      Konversi gambar/PDF/Word, printer, deteksi tool
  docx.rs         Penulis & pembaca DOCX
  scan.rs         Deteksi tepi, koreksi perspektif, pemutihan dokumen
  theme.rs        Palet Tahoe terang/gelap, kartu, tombol, ikon garis
  ui_sheet.rs     Tab Cetak Foto
  ui_history.rs   Tab Riwayat
  ui_scan.rs      Tab Pemindai Dokumen
  ui_convert.rs   Tab Konversi
  ui_tools.rs     Tab Alat & Pengaturan
```

---

## Lisensi

MIT
