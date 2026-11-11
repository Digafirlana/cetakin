use eframe::egui::{self, Align, Color32, CornerRadius, Layout, RichText, Stroke, Vec2};

use crate::model::ImageFit;
use crate::theme;

// ---------------------------------------------------------------------------

pub fn card_title(ui: &mut egui::Ui, icon: theme::Icon, title: &str) {
    ui.horizontal(|ui| {
        let (rect, _) = ui.allocate_exact_size(Vec2::splat(18.0), egui::Sense::hover());
        theme::icon(ui.painter(), rect, icon, theme::accent(), 1.5);
        ui.label(
            RichText::new(title)
                .size(13.5)
                .strong()
                .color(theme::text()),
        );
    });
}

pub fn drag_cm(ui: &mut egui::Ui, value_mm: &mut f32, width: f32) {
    let mut cm = *value_mm / 10.0;
    let response = ui.add_sized(
        [width, 28.0],
        egui::DragValue::new(&mut cm)
            .speed(0.02)
            .range(0.0..=50.0)
            .fixed_decimals(2)
            .suffix(" cm"),
    );
    if response.changed() {
        *value_mm = cm * 10.0;
    }
}

pub fn label_value(ui: &mut egui::Ui, label: &str, value: &str) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(label).size(11.5).color(theme::muted()));
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.label(RichText::new(value).size(11.5).color(theme::text()));
        });
    });
}

pub fn dashed_rect(painter: &egui::Painter, rect: egui::Rect, stroke: Stroke) {
    let points = [
        rect.left_top(),
        rect.right_top(),
        rect.right_bottom(),
        rect.left_bottom(),
        rect.left_top(),
    ];
    painter.extend(egui::Shape::dashed_line(&points, stroke, 5.0, 4.0));
}

pub fn cover_uv(texture_size: Vec2, target_aspect: f32) -> egui::Rect {
    let tex_aspect = (texture_size.x / texture_size.y.max(1.0)).max(0.01);
    if tex_aspect > target_aspect {
        let u = (target_aspect / tex_aspect).clamp(0.0, 1.0);
        egui::Rect::from_min_max(
            egui::pos2((1.0 - u) / 2.0, 0.0),
            egui::pos2((1.0 + u) / 2.0, 1.0),
        )
    } else {
        let v = (tex_aspect / target_aspect.max(0.01)).clamp(0.0, 1.0);
        egui::Rect::from_min_max(
            egui::pos2(0.0, (1.0 - v) / 2.0),
            egui::pos2(1.0, (1.0 + v) / 2.0),
        )
    }
}

pub fn paint_photo(
    painter: &egui::Painter,
    rect: egui::Rect,
    tex: &egui::TextureHandle,
    fit: ImageFit,
) {
    match fit {
        ImageFit::Cover => {
            let uv = cover_uv(tex.size_vec2(), rect.width() / rect.height().max(1.0));
            painter.image(tex.id(), rect, uv, Color32::WHITE);
        }
        ImageFit::Contain => {
            painter.rect_filled(rect, CornerRadius::ZERO, Color32::WHITE);
            let tex_size = tex.size_vec2();
            let scale =
                (rect.width() / tex_size.x.max(1.0)).min(rect.height() / tex_size.y.max(1.0));
            let size = tex_size * scale;
            let target = egui::Rect::from_center_size(rect.center(), size);
            painter.image(
                tex.id(),
                target,
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                Color32::WHITE,
            );
        }
    }
}
