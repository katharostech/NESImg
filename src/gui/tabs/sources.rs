use std::path::PathBuf;

use egui::Vec2;
use egui_extras::{Size, TableBuilder};
use watch::WatchReceiver;

use crate::gui::ProjectState;

use super::NesimgGuiTab;

pub struct SourcesTab {
    new_source: WatchReceiver<Option<PathBuf>>,
    preview_zoom: f32,
}

impl Default for SourcesTab {
    fn default() -> Self {
        Self {
            new_source: watch::channel(None).1,
            preview_zoom: 3.0,
        }
    }
}

impl NesimgGuiTab for SourcesTab {
    fn show(
        &mut self,
        project: &mut ProjectState,
        ctx: &egui::Context,
        _frame: &mut eframe::Frame,
    ) {
        if let Some(source) = self.new_source.get_if_new() {
            if let Some(source) = source {
                project.add_source(ctx, source);
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            if ui.button("➕ Add Source").clicked() {
                let (source_sender, source_receiver) = watch::channel(None);
                self.new_source = source_receiver;

                std::thread::spawn(move || {
                    let path = native_dialog::FileDialog::new()
                        .add_filter("PNG Image", &["png"])
                        .show_open_single_file()
                        .expect("File dialog");

                    source_sender.send(path);
                });
            }

            ui.separator();

            if project.data.sources.len() == 0 {
                ui.vertical_centered(|ui| {
                    ui.label("No Sources");
                });
            } else {
                TableBuilder::new(ui)
                    .striped(true)
                    .cell_layout(
                        egui::Layout::left_to_right().with_cross_align(egui::Align::Center),
                    )
                    .column(Size::remainder().at_least(60.0))
                    .column(Size::exact(100.))
                    .column(Size::exact(50.))
                    .header(20.0, |mut header| {
                        header.col(|ui| {
                            ui.label("Source Path");
                        });
                        header.col(|ui| {
                            ui.label("Image");
                        });
                        header.col(|ui| {
                            ui.label("Remove");
                        });
                    })
                    .body(|mut body| {
                        project.source_images.retain(|id, image| {
                            let mut keep = true;

                            const ROW_HEIGHT: f32 = 50.0;

                            body.row(ROW_HEIGHT, |mut row| {
                                row.col(|ui| {
                                    ui.label(image.path.to_string_lossy().as_ref());
                                });
                                row.col(|ui| {
                                    if let Some(image) = image.texture.get() {
                                        let orig_size = image.size_vec2();
                                        let aspect = orig_size.x / orig_size.y;
                                        let width = aspect * ROW_HEIGHT;
                                        let size = Vec2::new(width, ROW_HEIGHT);

                                        ui.image(image.texture_id(ctx), size)
                                            .on_hover_text("Scroll to zoom")
                                            .on_hover_ui(|ui| {
                                                self.preview_zoom +=
                                                    ui.input().scroll_delta.y * 0.01;
                                                ui.image(
                                                    image.texture_id(ctx),
                                                    size * self.preview_zoom,
                                                );
                                            });
                                    }
                                });
                                row.col(|ui| {
                                    if ui.button("🗙").clicked() {
                                        keep = false;
                                        project.data.sources.remove(id);
                                    };
                                });
                            });

                            keep
                        });
                    });
            }
        });
    }

    fn show_help(&mut self, ui: &mut egui::Ui) {
        ui.label(include_str!("./sources_help.txt"));
    }
}
