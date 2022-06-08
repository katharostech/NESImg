use std::path::PathBuf;

use egui::Vec2;
use egui_extras::{Size, TableBuilder};
use ulid::Ulid;
use watch::WatchReceiver;

use crate::gui::{project_state::SourceImageStatus, ProjectState};

use super::NesimgGuiTab;

pub struct SourcesTab {
    new_source: WatchReceiver<Option<PathBuf>>,
    update_source: (Ulid, WatchReceiver<Option<PathBuf>>),
    preview_zoom: f32,
}

impl Default for SourcesTab {
    fn default() -> Self {
        Self {
            update_source: (Ulid::default(), watch::channel(None).1),
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
                project.add_source(source);
            }
        }
        if let Some(update) = self.update_source.1.get_if_new() {
            if let Some(path) = update {
                project.update_source(self.update_source.0, path);
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            if ui.button("➕ Add Source").clicked() {
                self.new_source = browse_for_image_path();
            }

            ui.separator();

            if project.data.sources.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.label("No Sources");
                });
            } else {
                const ROW_HEIGHT: f32 = 50.0;

                TableBuilder::new(ui)
                    .striped(true)
                    .cell_layout(
                        egui::Layout::left_to_right().with_cross_align(egui::Align::Center),
                    )
                    .column(Size::remainder()) // Source path
                    .column(Size::exact(ROW_HEIGHT * 2.0)) // Image
                    .column(Size::exact(ROW_HEIGHT)) // Delete button
                    .header(20.0, |mut header| {
                        header.col(|ui| {
                            ui.horizontal(|ui| {
                                ui.label("Source Path");
                                ui.label("ℹ").on_hover_text(
                                    "Paths are relative to the project file's parent folder",
                                );
                            });
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

                            body.row(ROW_HEIGHT, |mut row| {
                                row.col(|ui| {
                                    ui.horizontal(|ui| {
                                        ui.add_space(7.0);
                                        if ui.button("✏").on_hover_text("Edit file path").clicked()
                                        {
                                            self.update_source = (*id, browse_for_image_path());
                                        }
                                        ui.label(
                                            image
                                                .path
                                                .to_string_lossy()
                                                .as_ref()
                                                .strip_prefix('.')
                                                .unwrap(),
                                        );
                                    });
                                });
                                row.col(|ui| match image.data.get() {
                                    SourceImageStatus::Found(image) => {
                                        let orig_size = image.texture.size_vec2();
                                        let aspect = orig_size.x / orig_size.y;
                                        let width = aspect * ROW_HEIGHT;
                                        let size = Vec2::new(width, ROW_HEIGHT);

                                        ui.image(image.texture.texture_id(ctx), size)
                                            .on_hover_text("Scroll to zoom")
                                            .on_hover_ui(|ui| {
                                                self.preview_zoom +=
                                                    ui.input().scroll_delta.y * 0.01;
                                                ui.image(
                                                    image.texture.texture_id(ctx),
                                                    size * self.preview_zoom,
                                                );
                                            });
                                    }
                                    SourceImageStatus::Loading => {
                                        ui.spinner();
                                    }
                                    SourceImageStatus::Error(e) => {
                                        ui.colored_label(egui::Color32::RED, "Error ℹ")
                                            .on_hover_ui(|ui| {
                                                ui.colored_label(egui::Color32::RED, &e);
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

    fn help_text(&self) -> &'static str {
        include_str!("./sources_help.txt")
    }

    fn tooltip(&self) -> &'static str {
        "Select source images"
    }
}

fn browse_for_image_path() -> WatchReceiver<Option<PathBuf>> {
    let (path_sender, path_receiver) = watch::channel(None);

    std::thread::spawn(move || {
        let path = native_dialog::FileDialog::new()
            .add_filter("PNG Image", &["png"])
            .show_open_single_file()
            .expect("File dialog");

        path_sender.send(path);
    });

    path_receiver
}
