use indexmap::IndexSet;

use crate::{
    gui::{
        components::{nes_color_picker, MetatileGui, MetatileKind},
        project_state::SourceImageStatus,
        ProjectState,
    },
    project::Metatileset,
    Uid,
};

use super::NesimgGuiTab;

pub struct MetatilesetsTab {
    current_metatileset_id: Option<Uid<Metatileset>>,
    side_metatile_list_col_count: u8,
    central_metatile_list_col_count: u8,
    /// The currently selected pallet, 0-3 that will be used for painting on metatiles
    current_subpallet_pallet: usize,
}

impl Default for MetatilesetsTab {
    fn default() -> Self {
        Self {
            current_metatileset_id: Default::default(),
            side_metatile_list_col_count: 5,
            central_metatile_list_col_count: 10,
            current_subpallet_pallet: 0,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
enum SidebarTab {
    Tiles,
    Colors,
}

impl Default for SidebarTab {
    fn default() -> Self {
        Self::Tiles
    }
}

impl NesimgGuiTab for MetatilesetsTab {
    fn show(&mut self, project: &mut ProjectState, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if self.current_metatileset_id.is_none() && !project.data.metatilesets.is_empty() {
            self.current_metatileset_id = Some(*project.data.metatilesets.keys().next().unwrap());
        } else if let Some(id) = &self.current_metatileset_id {
            if !project.data.metatilesets.contains_key(id) {
                self.current_metatileset_id = None;
            }
        }

        let sidebar_tab_id = egui::Id::new("metatilesets_sidebar_tab");
        let mut sidebar_tab = *ctx
            .data()
            .get_persisted_mut_or(sidebar_tab_id, SidebarTab::Tiles);

        let central_frame = egui::Frame {
            fill: ctx.style().visuals.window_fill(),
            ..Default::default()
        };
        let sidebar_frame = egui::Frame {
            inner_margin: egui::style::Margin::symmetric(8.0, 0.0),
            fill: ctx.style().visuals.window_fill(),
            stroke: ctx.style().visuals.window_stroke(),
            ..Default::default()
        };
        egui::CentralPanel::default()
            .frame(central_frame)
            .show(ctx, |ui| {
                let default_spacing = ui.spacing().item_spacing;
                // Avoid putting space between top panel and side panels
                ui.spacing_mut().item_spacing = egui::Vec2::default();

                egui::TopBottomPanel::top("metatile_top_bar").show_inside(ui, |ui| {
                    ui.spacing_mut().item_spacing = default_spacing;
                    self.top_panel(ui, project);
                });

                // Reset the spacing to the default after adding the top panel
                ui.spacing_mut().item_spacing = default_spacing;

                egui::SidePanel::left("utility_sidebar")
                    .frame(sidebar_frame)
                    .min_width(150.0)
                    .max_width(400.0)
                    .show_inside(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.add_space(ui.spacing().window_margin.top);
                        ui.horizontal(|ui| {
                            ui.selectable_value(
                                &mut sidebar_tab,
                                SidebarTab::Tiles,
                                "Available Metatiles",
                            );
                            ui.selectable_value(
                                &mut sidebar_tab,
                                SidebarTab::Colors,
                                "Color Pallet",
                            );
                        });

                        ui.separator();

                        match sidebar_tab {
                            SidebarTab::Tiles => {
                                self.available_metatiles_sidebar(project, ui, frame);
                            }
                            SidebarTab::Colors => {
                                self.color_pallet_sidebar(project, ui, frame);
                            }
                        }
                    });

                egui::SidePanel::right("pattern_table")
                    .frame(sidebar_frame)
                    .default_width(100.0)
                    .show_inside(ui, |ui| {
                        self.pattern_table_sidebar(project, ui, frame);
                    });

                egui::CentralPanel::default()
                    .frame(egui::Frame::default())
                    .show_inside(ui, |ui| {
                        self.central_panel(project, &sidebar_tab, ui, frame);
                    });
            });

        ctx.data().insert_persisted(sidebar_tab_id, sidebar_tab);
    }

    fn help_text(&self) -> &'static str {
        include_str!("./metatilesets_help.txt")
    }

    fn tooltip(&self) -> &'static str {
        "Color and group metatiles"
    }
}

impl MetatilesetsTab {
    fn top_panel(&mut self, ui: &mut egui::Ui, project: &mut ProjectState) {
        ui.add_space(1.0);
        ui.with_layout(egui::Layout::right_to_left(), |ui| {
            ui.add_enabled_ui(self.current_metatileset_id.is_some(), |ui| {
                if ui.button("🗑").on_hover_text("Delete Metatileset").clicked() {
                    project
                        .data
                        .metatilesets
                        .remove(&self.current_metatileset_id.unwrap());
                    self.current_metatileset_id = None;
                }
            });
            if ui
                .button("➕")
                .on_hover_text("Create metatileset")
                .clicked()
            {
                let id = Uid::new();
                project.data.metatilesets.insert(
                    id,
                    crate::project::Metatileset {
                        name: "New Metatileset".into(),
                        ..Default::default()
                    },
                );
                self.current_metatileset_id = Some(id);
            }

            egui::ComboBox::from_id_source("metatileset_select")
                .selected_text(
                    self.current_metatileset_id
                        .map(|id| {
                            project
                                .data
                                .metatilesets
                                .get(&id)
                                .map(|x| x.name.clone())
                                .unwrap()
                        })
                        .unwrap_or_else(|| "No Metatilesets".into()),
                )
                .show_ui(ui, |ui| {
                    for (id, metatileset) in &project.data.metatilesets {
                        ui.selectable_value(
                            &mut self.current_metatileset_id,
                            Some(*id),
                            &metatileset.name,
                        );
                    }
                });
            ui.label("Metatileset: ");

            ui.add_enabled_ui(self.current_metatileset_id.is_some(), |ui| {
                ui.horizontal(|ui| {
                    let mut name = &mut String::new();
                    if let Some(id) = self.current_metatileset_id {
                        if let Some(tileset) = project.data.metatilesets.get_mut(&id) {
                            name = &mut tileset.name;
                        }
                    }
                    ui.text_edit_singleline(name);
                    ui.label("Name: ");
                });
            });
        });
    }

    fn available_metatiles_sidebar(
        &mut self,
        project: &mut ProjectState,
        ui: &mut egui::Ui,
        frame: &mut eframe::Frame,
    ) {
        ui.horizontal(|ui| {
            ui.label("Zoom: ");
            ui.add(
                egui::Slider::new(&mut self.side_metatile_list_col_count, 16..=1).show_value(false),
            );
        });
        ui.separator();

        let item_spacing = egui::Vec2::splat(ui.spacing().item_spacing.x);
        ui.spacing_mut().item_spacing = item_spacing;
        let metatile_ids = project.data.metatiles.keys().cloned().collect::<Vec<_>>();
        let hovered_stroke_color = ui.visuals().widgets.hovered.fg_stroke.color;
        let tile_rounding = 2.0;

        // Render the metatile list
        ui.scope(|ui| {
            ui.set_height(ui.available_height());
            egui::ScrollArea::new([false, true]).show(ui, |ui| {
                ui.add_space(ui.spacing().item_spacing.y);
                ui.horizontal_wrapped(|ui| {
                    for id in metatile_ids {
                        let tile_region_display_size = egui::Vec2::splat(
                            ui.available_width() / self.side_metatile_list_col_count as f32,
                        ) - item_spacing;

                        let (rect, mut response) =
                            ui.allocate_exact_size(tile_region_display_size, egui::Sense::click());

                        if response.clicked() {
                            response.mark_changed();
                        }

                        MetatileGui::new(project, MetatileKind::Standalone(id))
                            .show_at(rect, ui, frame);

                        if response.hovered() {
                            ui.painter().rect_stroke(
                                rect,
                                tile_rounding,
                                (2.0, hovered_stroke_color),
                            );
                        }

                        if response.double_clicked() {
                            if let Some(metatileset) = self.current_metatileset(project) {
                                metatileset.tiles.insert(
                                    Uid::new(),
                                    crate::project::MetatilesetTile {
                                        metatile_id: id,
                                        sub_pallet_idx: 0,
                                    },
                                );

                                sort_project_metatileset(
                                    project,
                                    self.current_metatileset_id.unwrap(),
                                );
                            }
                        }

                        response.on_hover_text("Double-click to add to metatileset");
                    }
                });
                ui.add_space(ui.spacing().item_spacing.y);
            });
        });
    }

    fn color_pallet_sidebar(
        &mut self,
        project: &mut ProjectState,
        ui: &mut egui::Ui,
        _frame: &mut eframe::Frame,
    ) {
        let metatileset = if let Some(metatileset) = self.current_metatileset(project) {
            metatileset
        } else {
            return;
        };

        ui.horizontal(|ui| {
            ui.radio_value(&mut self.current_subpallet_pallet, 0, "")
                .on_hover_ui(|ui| {
                    ui.label("Select pallet");
                    ui.label("Shortcut: 1");
                });

            nes_color_picker(ui, &mut metatileset.pallet.colors[0]);
            nes_color_picker(ui, &mut metatileset.pallet.colors[1]);
            nes_color_picker(ui, &mut metatileset.pallet.colors[2]);
            nes_color_picker(ui, &mut metatileset.pallet.colors[3]);
        });
        ui.horizontal(|ui| {
            ui.radio_value(&mut self.current_subpallet_pallet, 1, "")
                .on_hover_ui(|ui| {
                    ui.label("Select pallet");
                    ui.label("Shortcut: 2");
                });
            nes_color_picker(ui, &mut metatileset.pallet.colors[0]);
            nes_color_picker(ui, &mut metatileset.pallet.colors[4]);
            nes_color_picker(ui, &mut metatileset.pallet.colors[5]);
            nes_color_picker(ui, &mut metatileset.pallet.colors[6]);
        });
        ui.horizontal(|ui| {
            ui.radio_value(&mut self.current_subpallet_pallet, 2, "")
                .on_hover_ui(|ui| {
                    ui.label("Select pallet");
                    ui.label("Shortcut: 3");
                });
            nes_color_picker(ui, &mut metatileset.pallet.colors[0]);
            nes_color_picker(ui, &mut metatileset.pallet.colors[7]);
            nes_color_picker(ui, &mut metatileset.pallet.colors[8]);
            nes_color_picker(ui, &mut metatileset.pallet.colors[9]);
        });
        ui.horizontal(|ui| {
            ui.radio_value(&mut self.current_subpallet_pallet, 3, "")
                .on_hover_ui(|ui| {
                    ui.label("Select pallet");
                    ui.label("Shortcut: 4");
                });
            nes_color_picker(ui, &mut metatileset.pallet.colors[0]);
            nes_color_picker(ui, &mut metatileset.pallet.colors[10]);
            nes_color_picker(ui, &mut metatileset.pallet.colors[11]);
            nes_color_picker(ui, &mut metatileset.pallet.colors[12]);
        });
    }

    fn pattern_table_sidebar(
        &mut self,
        project: &mut ProjectState,
        ui: &mut egui::Ui,
        _frame: &mut eframe::Frame,
    ) {
        ui.set_width(ui.available_width());
        ui.add_space(ui.spacing().window_margin.top);
        ui.horizontal(|ui| {
            ui.set_height(ui.spacing().interact_size.y);
            ui.label("Pattern Table");
        });
        ui.separator();

        let max_tiles = 16 * 16;
        let mut tiles = if let Some(metatileset) = self.current_metatileset(project) {
            let mut tiles = IndexSet::with_capacity(max_tiles);

            let metatile_ids = metatileset
                .tiles
                .values()
                .map(|x| x.metatile_id)
                .collect::<Vec<_>>();

            for id in metatile_ids {
                let metatile = project.data.metatiles.get(&id).unwrap();
                for i in 0..4 {
                    if let Some(tile) = metatile.tiles[i].as_ref() {
                        tiles.insert(tile);
                    }
                }
            }

            tiles
        } else {
            return;
        };

        let progress = tiles.len() as f32 / max_tiles as f32;
        if progress > 1.0 {
            let dark = ui.style().visuals.dark_mode;
            ui.style_mut().visuals.selection.bg_fill = if dark {
                egui::Color32::DARK_RED
            } else {
                egui::Color32::RED
            };
        }
        let progress_resp = ui.add(
            egui::ProgressBar::new(progress)
                .show_percentage()
                .text(format!(
                    "{}Percentage Used: {:.1}%",
                    if progress > 1.0 { "⚠ " } else { "" },
                    progress * 100.0
                ))
                .desired_width(ui.available_width()),
        );
        if progress > 1.0 {
            progress_resp.on_hover_text("⚠ Too many tiles to fit into NES pattern table!");
        } else {
            progress_resp.on_hover_text(
                "The percentage of the pattern table that has been used by the metatileset.",
            );
        }

        ui.separator();

        ui.add_space(ui.spacing().item_spacing.y);

        let size = ui.available_width().min(ui.available_height());

        let (rect, _response) =
            ui.allocate_exact_size(egui::Vec2::splat(size), egui::Sense::hover());

        ui.painter().rect_filled(rect, 0.0, egui::Color32::BLACK);

        let pattern_table_width = 128;
        let pattern_table_height = pattern_table_width;
        let tile_size = 8;
        let tiles_wide = pattern_table_width / tile_size;
        let tiles_high = pattern_table_height / tile_size;
        let physical_tile_size = egui::Vec2::splat(rect.width() / tiles_wide as f32 + 0.1);

        for y in 0..tiles_high {
            for x in 0..tiles_wide {
                let min = rect.min + egui::Vec2::new(x as f32, y as f32) * physical_tile_size;
                let max = min + physical_tile_size;
                let rect = egui::Rect { min, max };
                if let Some(tile) = tiles.shift_remove_index(0) {
                    let source_image = project.source_images.get_mut(&tile.source_id).unwrap();
                    let source_data =
                        if let SourceImageStatus::Found(data) = source_image.data.get() {
                            data
                        } else {
                            ui.painter().rect_filled(rect, 0.0, egui::Color32::BLACK);
                            continue;
                        };
                    let source_size = source_data.texture.size_vec2();
                    let uv_start = egui::Vec2::new(
                        tile.x as f32 * 8.0 / source_size.x,
                        tile.y as f32 * 8.0 / source_size.y,
                    );
                    let uv_end = uv_start + egui::Vec2::splat(8.0) / source_size;
                    let uv = egui::Rect {
                        min: uv_start.to_pos2(),
                        max: uv_end.to_pos2(),
                    };
                    egui::Image::new(source_data.texture.texture_id(ui.ctx()), rect.size())
                        .uv(uv)
                        .paint_at(ui, rect);
                }
            }
        }
    }

    fn central_panel(
        &mut self,
        project: &mut ProjectState,
        sidebar_tab: &SidebarTab,
        ui: &mut egui::Ui,
        frame: &mut eframe::Frame,
    ) {
        ui.add_space(ui.spacing().window_margin.top);
        ui.horizontal(|ui| {
            ui.set_height(ui.spacing().interact_size.y);
            ui.label("Metatileset");

            ui.add(
                egui::Slider::new(&mut self.central_metatile_list_col_count, 16..=1)
                    .show_value(false),
            )
            .on_hover_text("Zoom");

            ui.with_layout(egui::Layout::right_to_left(), |ui| {
                if ui
                    .button("⬍ Sort")
                    .on_hover_text("Sort tiles according to their order in the Metatiles tab")
                    .clicked()
                {
                    if let Some(id) = self.current_metatileset_id {
                        sort_project_metatileset(project, id);
                    }
                }
            });
        });
        ui.separator();

        let metatileset = if let Some(metatileset) = self.current_metatileset(project) {
            metatileset
        } else {
            return;
        };
        let metatileset_id = self.current_metatileset_id.unwrap();

        if metatileset.tiles.is_empty() {
            ui.vertical_centered(|ui| {
                ui.add_space(5.0);
                ui.label("No metatiles. Add them by clicking them in the left sidebar.");
            });
            return;
        }

        let item_spacing = egui::Vec2::splat(ui.spacing().item_spacing.x);
        ui.spacing_mut().item_spacing = item_spacing;
        let tile_ids = metatileset.tiles.keys().cloned().collect::<Vec<_>>();
        let hovered_stroke_color = ui.visuals().widgets.hovered.fg_stroke.color;
        let tile_rounding = 2.0;

        ui.scope(|ui| {
            egui::ScrollArea::new([false, true]).show(ui, |ui| {
                ui.add_space(ui.spacing().item_spacing.y);
                ui.horizontal_wrapped(|ui| {
                    for id in tile_ids {
                        let tile_region_display_size = egui::Vec2::splat(
                            ui.available_width() / self.central_metatile_list_col_count as f32,
                        ) - item_spacing;

                        let (rect, mut response) =
                            ui.allocate_exact_size(tile_region_display_size, egui::Sense::click());

                        if response.clicked() {
                            response.mark_changed();
                        }

                        MetatileGui::new(
                            project,
                            MetatileKind::Metatileset {
                                metatileset_id,
                                metatileset_tile_id: id,
                            },
                        )
                        .show_at(rect, ui, frame);

                        if response.hovered() {
                            ui.painter().rect_stroke(
                                rect,
                                tile_rounding,
                                (2.0, hovered_stroke_color),
                            );
                        }

                        // Paint the active pallet onto the tile
                        if sidebar_tab == &SidebarTab::Colors {
                            response = response.on_hover_cursor(egui::CursorIcon::Crosshair);

                            if response.clicked() {
                                let tile = self
                                    .current_metatileset(project)
                                    .unwrap()
                                    .tiles
                                    .get_mut(&id)
                                    .unwrap();
                                tile.sub_pallet_idx = self.current_subpallet_pallet;
                            }
                        }

                        response.context_menu(|ui| {
                            if ui.button("🗑 Remove").clicked() {
                                self.current_metatileset(project).unwrap().tiles.remove(&id);
                                sort_project_metatileset(
                                    project,
                                    self.current_metatileset_id.unwrap(),
                                );
                                ui.close_menu();
                            }
                        });
                    }
                });
                ui.add_space(ui.spacing().item_spacing.y);
            });
        });
    }
}

/// Sort the metatileset based on the order the tiles are in in the corresponding sources
fn sort_project_metatileset(project: &mut ProjectState, id: Uid<Metatileset>) {
    let metatileset = project.data.metatilesets.get_mut(&id).unwrap();

    metatileset.tiles.sort_unstable_by(|_, a, _, b| {
        let a_idx = project.data.metatiles.get_index_of(&a.metatile_id).unwrap();
        let b_idx = project.data.metatiles.get_index_of(&b.metatile_id).unwrap();
        a_idx.cmp(&b_idx)
    });
}

// Helper methods
impl MetatilesetsTab {
    fn current_metatileset<'a, 'b>(
        &'a self,
        project: &'b mut ProjectState,
    ) -> Option<&'b mut Metatileset> {
        self.current_metatileset_id
            .and_then(|id| project.data.metatilesets.get_mut(&id))
    }
}
