//! Preferensi aplikasi: penyimpanan, pemuatan, dan reset.

use std::path::PathBuf;

use eframe::egui::Vec2;
use serde::{Deserialize, Serialize};

use crate::app::CetakinApp;
use crate::model::SheetSettings;
use crate::theme::Flavor;

#[derive(Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Prefs {
    pub sheet: SheetSettings,
    pub studio: String,
    pub last_dir: Option<PathBuf>,
    /// Mode tampilan yang dipakai (Tahoe Terang / Tahoe Gelap).
    pub theme: Flavor,
}

impl CetakinApp {
    // -------------------------------------------------------------- prefs

    pub fn save_prefs(&mut self) {
        let prefs = Prefs {
            sheet: self.settings.clone(),
            studio: self.studio.clone(),
            last_dir: self.convert.output_dir.clone(),
            theme: crate::theme::flavor(),
        };
        if let Some(dir) = self.prefs_path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        if let Ok(raw) = serde_json::to_string_pretty(&prefs) {
            if std::fs::write(&self.prefs_path, raw).is_ok() {
                self.prefs_dirty = false;
            }
        }
    }

    pub fn save_prefs_manual(&mut self) {
        self.save_prefs();
        let path = self.prefs_path.to_string_lossy().to_string();
        self.toast_success("Preferensi disimpan", &path);
    }

    pub fn reset_settings(&mut self) {
        self.settings = SheetSettings::default();
        self.studio = String::new();
        self.preview_pan = Vec2::ZERO;
        self.preview_zoom = 1.0;
        self.confirm_reset = false;
        self.prefs_dirty = true;
        self.toast_info("Pengaturan direset", "Semua pilihan kembali ke bawaan.");
    }
}
