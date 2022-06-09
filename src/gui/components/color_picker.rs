use egui::{Color32, Response, Sense, Ui, Vec2};

use crate::constants::NES_PALLET;

use super::popup_under_widget;

/// The border radius used for rendering
const BORDER_RADIUS: f32 = 2.0;

/// Allows you to select a color from the NES pallet
pub fn nes_color_picker(ui: &mut Ui, nes_color_index: &mut u32) {
    let i = (*nes_color_index).min(63) as usize;
    let color = NES_PALLET[i];
    let color = egui::Color32::from_rgb(color[0], color[1], color[2]);

    let response = color_button(ui, color);

    let popup_id = response.id.with("popup");
    if response.clicked() {
        ui.memory().toggle_popup(popup_id)
    }
    let response = response.on_hover_ui(|ui| {
        ui.label(format!("NES Pallet Index: ${:02X}", i));
        ui.label(format!("srgb: ({}, {}, {})", color[0], color[1], color[2]));
        ui.label(format!(
            "srgb: #{:02X}{:02X}{:02X}",
            color[0], color[1], color[2]
        ));
    });

    popup_under_widget(ui, popup_id, &response, |ui| {
        ui.horizontal_wrapped(|ui| {
            ui.set_max_width(800.0);
            for (i, color) in NES_PALLET.iter().enumerate() {
                let color = egui::Color32::from_rgb(color[0], color[1], color[2]);
                let resp = color_button(ui, color);

                let resp = resp.on_hover_ui(|ui| {
                    ui.label(format!("NES Pallet Index: ${:02X}", i));
                    ui.label(format!("srgb: ({}, {}, {})", color[0], color[1], color[2]));
                    ui.label(format!(
                        "srgb: #{:02X}{:02X}{:02X}",
                        color[0], color[1], color[2]
                    ));
                });

                if resp.clicked() {
                    *nes_color_index = i as u32;
                    ui.memory().close_popup();
                }
            }
        });
    });
}

/// Displays a clickable color button
pub fn color_button(ui: &mut Ui, color: Color32) -> Response {
    let padding = Vec2::splat(2.0);
    let (rect, response) =
        ui.allocate_exact_size(ui.spacing().interact_size + padding, Sense::click());
    let color_rect = rect.expand2(-padding);

    let painter = ui.painter();
    painter.rect_filled(color_rect, BORDER_RADIUS, color);

    if response.hovered() {
        painter.rect_stroke(
            rect,
            BORDER_RADIUS,
            ui.visuals().widgets.noninteractive.fg_stroke,
        );
    }

    response
}
