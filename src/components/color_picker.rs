use egui::{
    Align, Area, Color32, Frame, Id, Key, Layout, Order, Response, Sense,
    Ui, Vec2,
};

use crate::globals::NES_PALETTE_RGB;

/// The border radius used for rendering
const BORDER_RADIUS: f32 = 2.0;

/// Allows you to select a color from the NES pallet
pub fn nes_color_picker(ui: &mut Ui, nes_color_index: &mut u8) {
    let i = (*nes_color_index).min(63) as usize;
    let color = NES_PALETTE_RGB[i];
    let color = egui::Color32::from_rgb(color.red, color.green, color.blue);

    let response = color_button(ui, color);

    let popup_id = response.id.with("popup");
    if response.clicked() {
        ui.memory().open_popup(popup_id)
    }

    popup_under_widget(ui, popup_id, &response, |ui| {
        ui.horizontal_wrapped(|ui| {
            ui.set_max_width(800.0);
            for (i, color) in NES_PALETTE_RGB.iter().enumerate() {
                let color = egui::Color32::from_rgb(color.red, color.green, color.blue);
                let resp = color_button(ui, color);

                let resp = resp.on_hover_ui(|ui| {
                    ui.label(format!("NES Palette Index: ${:02X}", i));
                    ui.label(format!("srgb: ({}, {}, {})", color[0], color[1], color[2]));
                    ui.label(format!(
                        "srgb: #{:02X}{:02X}{:02X}",
                        color[0], color[1], color[2]
                    ));
                });

                if resp.clicked() {
                    *nes_color_index = i as u8;
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

/// Like `egui::popup_under_widget, but pops up to the left, so that the popup doesn't go off the screen
pub fn popup_under_widget<R>(
    ui: &Ui,
    popup_id: Id,
    widget_response: &Response,
    add_contents: impl FnOnce(&mut Ui) -> R,
) -> Option<R> {
    if ui.memory().is_popup_open(popup_id) {
        let inner = Area::new(popup_id)
            .order(Order::Foreground)
            .default_pos(widget_response.rect.left_bottom())
            .show(ui.ctx(), |ui| {
                // Note: we use a separate clip-rect for this area, so the popup can be outside the parent.
                // See https://github.com/emilk/egui/issues/825
                let frame = Frame::popup(ui.style());
                let frame_margin = frame.inner_margin + frame.outer_margin;
                frame
                    .show(ui, |ui| {
                        ui.with_layout(Layout::top_down_justified(Align::LEFT), |ui| {
                            ui.set_width(widget_response.rect.width() - frame_margin.sum().x);
                            add_contents(ui)
                        })
                        .inner
                    })
                    .inner
            })
            .inner;

        if ui.input().key_pressed(Key::Escape) || widget_response.clicked_elsewhere() {
            ui.memory().close_popup();
        }
        Some(inner)
    } else {
        None
    }
}
