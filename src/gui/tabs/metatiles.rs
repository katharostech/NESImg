use egui::{Color32, ComboBox, Layout};
use ulid::Ulid;

use crate::{
    gui::{components::MetatileGui, ProjectState},
    project::Metatile,
};

use super::NesimgGuiTab;

pub struct MetatilesTab {
    current_source_image: Option<Ulid>,
    current_metatile: Option<Ulid>,
    metatile_list_col_count: u32,
}

impl Default for MetatilesTab {
    fn default() -> Self {
        Self {
            current_source_image: Default::default(),
            current_metatile: Default::default(),
            metatile_list_col_count: 4,
        }
    }
}

impl NesimgGuiTab for MetatilesTab {
    fn show(&mut self, project: &mut ProjectState, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if let Some(id) = &self.current_metatile {
            if !project.data.metatiles.contains_key(id) {
                self.current_metatile = None;
            }
        }

        egui::SidePanel::right("sidebar")
            .min_width(200.0)
            .show(ctx, |ui| {
                ui.set_width(ui.available_width());
                ui.add_space(ui.spacing().window_margin.top);
                ui.horizontal(|ui| {
                    ui.label("Metatiles");
                    ui.add(
                        egui::Slider::new(&mut self.metatile_list_col_count, 16..=1)
                            .show_value(false),
                    )
                    .on_hover_text("Zoom");

                    ui.with_layout(Layout::right_to_left(), |ui| {
                        if ui
                            .button("➕")
                            .on_hover_text("Create a new metatile")
                            .clicked()
                        {
                            project.data.metatiles.insert(
                                Ulid::new(),
                                Metatile {
                                    tiles: [None, None, None, None],
                                },
                            );
                        }
                    });
                });
                ui.separator();

                let item_spacing = egui::Vec2::splat(ui.spacing().item_spacing.x);
                ui.spacing_mut().item_spacing = item_spacing;
                let tile_ids = project.data.metatiles.keys().cloned().collect::<Vec<_>>();
                let hovered_stroke_color = ui.visuals().widgets.hovered.fg_stroke.color;
                let active_stroke_color = egui::Color32::GREEN;
                let tile_rounding = 2.0;

                ui.scope(|ui| {
                    ui.set_height(ui.available_height());
                    egui::ScrollArea::new([false, true]).show(ui, |ui| {
                        ui.horizontal_wrapped(|ui| {
                            for id in tile_ids {
                                let tile_region_display_size = egui::Vec2::splat(
                                    ui.available_width() / self.metatile_list_col_count as f32,
                                ) - item_spacing;

                                let (rect, mut response) = ui.allocate_exact_size(
                                    tile_region_display_size,
                                    egui::Sense::click(),
                                );

                                response = response.context_menu(|ui| {
                                    ui.set_width(75.0);
                                    if ui.button("🗑 Delete").clicked() {
                                        project.data.metatiles.remove(&id);
                                        ui.close_menu();

                                        if self.current_metatile == Some(id) {
                                            self.current_metatile = None;
                                        }
                                    }
                                });

                                if response.clicked() {
                                    self.current_metatile = Some(id);
                                    response.mark_changed();
                                }

                                MetatileGui::new(project, id, tile_region_display_size)
                                    .show_at(rect, ui, frame);

                                if self.current_metatile == Some(id) {
                                    ui.painter().rect_stroke(
                                        rect,
                                        tile_rounding,
                                        (2.0, active_stroke_color),
                                    );
                                } else if response.hovered() {
                                    ui.painter().rect_stroke(
                                        rect,
                                        tile_rounding,
                                        (2.0, hovered_stroke_color),
                                    );
                                }
                            }
                        });
                    });
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.columns(2, |cols| {
                {
                    let ui = &mut cols[0];

                    ui.horizontal(|ui| {
                        ui.label("Source Image: ");
                        ComboBox::from_id_source("source_image")
                            .width(ui.available_width() - ui.spacing().item_spacing.x)
                            .selected_text(
                                self.current_source_image
                                    .map(|id| {
                                        project
                                            .data
                                            .sources
                                            .get(&id)
                                            .unwrap()
                                            .to_string_lossy()
                                            .to_string()
                                    })
                                    .unwrap_or("None".to_string()),
                            )
                            .show_ui(ui, |ui| {
                                for (id, path) in project.data.sources.iter() {
                                    ui.selectable_value(
                                        &mut self.current_source_image,
                                        Some(*id),
                                        path.to_string_lossy().as_ref(),
                                    );
                                }
                            });
                    });

                    ui.separator();

                    ui.centered_and_justified(|ui| {
                        if let Some(id) = self.current_source_image {
                            let source_image = project.source_images.get_mut(&id).unwrap();

                            match source_image.data.get() {
                                crate::gui::project_state::SourceImageStatus::Loading => {
                                    ui.spinner();
                                }
                                crate::gui::project_state::SourceImageStatus::Error(e) => {
                                    ui.colored_label(Color32::RED, e.to_string());
                                }
                                crate::gui::project_state::SourceImageStatus::Found(image) => {
                                    ui.image(image.texture.texture_id(ctx), ui.available_size());
                                }
                            }
                        } else {
                            ui.label("No source file selected");
                        }
                    });
                }

                {
                    let ui = &mut cols[1];

                    ui.horizontal(|ui| {
                        ui.set_height(ui.spacing().interact_size.y);
                        ui.label("Edit Metatile");
                    });
                    ui.separator();

                    ui.centered_and_justified(|ui| {
                        if let Some(id) = self.current_metatile {
                            MetatileGui::new(project, id, ui.available_size()).show(ui, frame);
                        } else {
                            ui.label("No metatile selected");
                        }
                    });
                }
            });
        });
    }

    fn help_text(&self) -> &'static str {
        include_str!("./metatiles_help.txt")
    }
}
