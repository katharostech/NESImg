use eframe::egui;
use egui::{Rect, Vec2};
use egui_extras::RetainedImage;

fn image_viewer_ui(
    ui: &mut egui::Ui,
    image: &mut RetainedImage,
    zoom: &mut f32,
    offset: &mut Vec2,
) -> egui::Response {
    let available_size = ui.available_size() - Vec2::new(0.0, ui.spacing().interact_size.y);

    let (rect, response) = ui.allocate_exact_size(available_size, egui::Sense::click_and_drag());

    ui.set_clip_rect(rect);

    let image_size = image.size_vec2();

    let drag_delta = response.drag_delta();
    let zoom_delta = if response.hovered() {
        ui.input().scroll_delta.y
    } else {
        0.0
    };

    *zoom += zoom_delta;
    *offset += drag_delta;

    *zoom = zoom.max(0.0);

    let widget = egui::Image::new(image.texture_id(ui.ctx()), available_size)
        .sense(egui::Sense::click_and_drag());

    let min = rect.min + *offset;
    let max = min + image_size;
    let mut draw_rect = Rect { min, max };
    let draw_aspect = draw_rect.aspect_ratio();

    draw_rect = draw_rect.expand2(Vec2::new(*zoom * draw_aspect, *zoom));

    widget.paint_at(ui, draw_rect);

    response
}

pub fn image_viewer<'a>(
    image: &'a mut RetainedImage,
    zoom: &'a mut f32,
    offset: &'a mut Vec2,
) -> impl egui::Widget + 'a {
    move |ui: &mut egui::Ui| image_viewer_ui(ui, image, zoom, offset)
}
