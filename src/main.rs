#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod app;
mod convert;
mod docx;
mod image_utils;
mod jobs;
mod layout;
mod model;
mod pdf_export;
mod photos;
mod prefs;
mod scan;
mod theme;
mod ui_chrome;
mod ui_common;
mod ui_convert;
mod ui_home;
mod ui_scan;
mod ui_sheet;
mod ui_tools;
mod util;

use std::sync::Arc;

use eframe::egui;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("cetakin. — cetak foto & konversi dokumen")
            .with_inner_size([1280.0, 840.0])
            .with_min_inner_size([1040.0, 700.0])
            .with_icon(Arc::new(app_icon())),
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };

    eframe::run_native(
        "cetakin",
        options,
        Box::new(|cc| Ok(Box::new(app::CetakinApp::new(cc)))),
    )
}

/// Ikon jendela sederhana: kotak hijau membulat dengan glyph printer.
fn app_icon() -> egui::IconData {
    const SIZE: usize = 64;
    const RADIUS: i32 = 14;
    let accent = [10u8, 132, 255, 255];
    let dark = [8u8, 24, 14, 255];
    let paper = [245u8, 248, 246, 255];

    let mut rgba = vec![0u8; SIZE * SIZE * 4];
    let mut put = |x: i32, y: i32, color: [u8; 4]| {
        if x < 0 || y < 0 || x >= SIZE as i32 || y >= SIZE as i32 {
            return;
        }
        let i = (y as usize * SIZE + x as usize) * 4;
        rgba[i..i + 4].copy_from_slice(&color);
    };

    // Latar kotak membulat.
    for y in 0..SIZE as i32 {
        for x in 0..SIZE as i32 {
            let cx = x.clamp(RADIUS, SIZE as i32 - 1 - RADIUS);
            let cy = y.clamp(RADIUS, SIZE as i32 - 1 - RADIUS);
            let dx = x - cx;
            let dy = y - cy;
            let inside = dx * dx + dy * dy <= RADIUS * RADIUS;
            if inside {
                put(x, y, accent);
            }
        }
    }

    // Badan printer.
    for y in 26..46 {
        for x in 16..48 {
            put(x, y, dark);
        }
    }
    // Kertas masuk & keluar.
    for y in 15..26 {
        for x in 23..41 {
            put(x, y, paper);
        }
    }
    for y in 45..54 {
        for x in 23..41 {
            put(x, y, paper);
        }
    }
    // Lampu indikator.
    for y in 31..35 {
        for x in 40..44 {
            put(x, y, accent);
        }
    }
    // Aksen garis pada kertas.
    for x in 26..38 {
        put(x, 20, accent);
        put(x, 49, dark);
    }

    egui::IconData {
        rgba,
        width: SIZE as u32,
        height: SIZE as u32,
    }
}
