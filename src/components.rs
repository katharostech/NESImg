mod image_viewer;
use egui::{Align, Area, Frame, Id, Key, Layout, Order, Response, Ui};
pub use image_viewer::*;

mod color_picker;
pub use color_picker::*;

/// Like `egui::popup_under_widget, but pops up to the left, so that the popup doesn't go off the screen
pub(crate) fn popup_under_widget<R>(
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
