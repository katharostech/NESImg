use ulid::Ulid;

use crate::gui::{components::MetatileGui, ProjectState};

use super::NesimgGuiTab;

pub struct MetatilesetsTab {
    current_metatileset: Option<Ulid>,
    current_tab: Tab,
    metatile_list_col_count: u8,
}

impl Default for MetatilesetsTab {
    fn default() -> Self {
        Self {
            current_metatileset: Default::default(),
            current_tab: Default::default(),
            metatile_list_col_count: 5,
        }
    }
}

#[derive(Debug, PartialEq)]
enum Tab {
    Tiles,
    Colors,
}

impl Default for Tab {
    fn default() -> Self {
        Self::Tiles
    }
}

impl NesimgGuiTab for MetatilesetsTab {
    fn show(&mut self, project: &mut ProjectState, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if self.current_metatileset.is_none() && !project.data.metatilesets.is_empty() {
            self.current_metatileset = Some(*project.data.metatilesets.keys().next().unwrap());
        }

        if let Tab::Tiles = self.current_tab {
            egui::SidePanel::left("available_metatiles")
                .min_width(150.0)
                .max_width(400.0)
                .show(ctx, |ui| {
                    ui.set_width(ui.available_width());
                    ui.add_space(ui.spacing().window_margin.top);
                    ui.horizontal(|ui| {
                        ui.label("Available Metatiles");
                        ui.add(
                            egui::Slider::new(&mut self.metatile_list_col_count, 16..=1)
                                .show_value(false),
                        )
                        .on_hover_text("Zoom");
                    });
                    ui.separator();

                    let item_spacing = egui::Vec2::splat(ui.spacing().item_spacing.x);
                    ui.spacing_mut().item_spacing = item_spacing;
                    let tile_ids = project.data.metatiles.keys().cloned().collect::<Vec<_>>();
                    let hovered_stroke_color = ui.visuals().widgets.hovered.fg_stroke.color;
                    let tile_rounding = 2.0;

                    // Render the metatile list
                    ui.scope(|ui| {
                        ui.set_height(ui.available_height());
                        egui::ScrollArea::new([false, true]).show(ui, |ui| {
                            ui.add_space(ui.spacing().item_spacing.y);
                            ui.horizontal_wrapped(|ui| {
                                for id in tile_ids {
                                    let tile_region_display_size = egui::Vec2::splat(
                                        ui.available_width() / self.metatile_list_col_count as f32,
                                    ) - item_spacing;

                                    let (rect, mut response) = ui.allocate_exact_size(
                                        tile_region_display_size,
                                        egui::Sense::click(),
                                    );

                                    if response.clicked() {
                                        response.mark_changed();
                                    }

                                    MetatileGui::new(project, id).show_at(rect, ui, frame);

                                    if response.hovered() {
                                        ui.painter().rect_stroke(
                                            rect,
                                            tile_rounding,
                                            (2.0, hovered_stroke_color),
                                        );
                                    }
                                }
                            });
                            ui.add_space(ui.spacing().item_spacing.y);
                        });
                    });
                });
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.add_enabled_ui(self.current_metatileset.is_some(), |ui| {
                    if ui.button("🗑").on_hover_text("Delete Metatileset").clicked() {
                        project
                            .data
                            .metatilesets
                            .remove(&self.current_metatileset.unwrap());
                        self.current_metatileset = None;
                    }
                });
                if ui
                    .button("➕")
                    .on_hover_text("Create metatileset")
                    .clicked()
                {
                    let id = Ulid::new();
                    project.data.metatilesets.insert(
                        id,
                        crate::project::Metatileset {
                            name: "New Metatileset".into(),
                            ..Default::default()
                        },
                    );
                    self.current_metatileset = Some(id);
                }

                ui.label("Metatileset: ");
                egui::ComboBox::from_id_source("metatileset_select")
                    .selected_text(
                        self.current_metatileset
                            .map(|id| {
                                project
                                    .data
                                    .metatilesets
                                    .get(&id)
                                    .map(|x| x.name.clone())
                                    .unwrap()
                            })
                            .unwrap_or("Select Metatileset...".into()),
                    )
                    .show_ui(ui, |ui| {
                        for (id, metatileset) in &project.data.metatilesets {
                            ui.selectable_value(
                                &mut self.current_metatileset,
                                Some(*id),
                                &metatileset.name,
                            );
                        }
                    });

                ui.with_layout(egui::Layout::right_to_left(), |ui| {
                    ui.selectable_value(&mut self.current_tab, Tab::Colors, "Colors");
                    ui.selectable_value(&mut self.current_tab, Tab::Tiles, "Tiles");
                    ui.separator();

                    ui.with_layout(egui::Layout::left_to_right(), |ui| {
                        ui.add_enabled_ui(self.current_metatileset.is_some(), |ui| {
                            ui.horizontal(|ui| {
                                let mut name = &mut String::new();
                                if let Some(id) = self.current_metatileset {
                                    if let Some(tileset) = project.data.metatilesets.get_mut(&id) {
                                        name = &mut tileset.name;
                                    }
                                }
                                ui.label("Name: ");
                                ui.text_edit_singleline(name);
                            });
                        });
                    });
                });
            });

            ui.separator();
        });
    }

    fn help_text(&self) -> &'static str {
        include_str!("./metatilesets_help.txt")
    }
}
