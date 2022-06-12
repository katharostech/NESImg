use egui::DragValue;

use crate::{
    constants::NES_PALLET,
    gui::{
        components::{MetatileGui, MetatileKind},
        ProjectState,
    },
    project::{Level, LevelTile, MetatilesetTile},
    Uid,
};

use super::NesimgGuiTab;

pub struct MapsTab {
    zoom: f32,
    pan: egui::Vec2,
    dragging_level: Option<Uid<Level>>,
    current_level: Option<Uid<Level>>,
    tile_list_col_count: u8,
    current_metatileset_tile: Option<Uid<MetatilesetTile>>,
}

impl Default for MapsTab {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            pan: egui::Vec2::ZERO,
            dragging_level: None,
            current_level: None,
            tile_list_col_count: 5,
            current_metatileset_tile: None,
        }
    }
}

impl NesimgGuiTab for MapsTab {
    fn show(&mut self, project: &mut ProjectState, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if self.current_level.is_none() && !project.data.levels.is_empty() {
            self.current_level = Some(*project.data.levels.keys().last().unwrap());
        } else if let Some(id) = &self.current_level {
            if !project.data.levels.contains_key(id) {
                self.current_level = None;
            }
        }

        let central_frame = egui::Frame {
            fill: ctx.style().visuals.window_fill(),
            ..Default::default()
        };
        egui::CentralPanel::default()
            .frame(central_frame)
            .show(ctx, |ui| {
                let default_spacing = ui.spacing().item_spacing;

                // Avoid putting space between panels
                ui.spacing_mut().item_spacing = egui::Vec2::default();

                egui::TopBottomPanel::top("maps_toolbar").show_inside(ui, |ui| {
                    ui.spacing_mut().item_spacing = default_spacing;

                    ui.with_layout(egui::Layout::right_to_left(), |ui| {
                        if ui.button("Reset View").clicked() {
                            self.zoom = 1.0;
                            self.pan = egui::Vec2::ZERO;
                        }
                        ui.add_space(10.0);
                        ui.monospace(format!("Zoom: {:>5.1}", self.zoom));
                        ui.add_space(10.0);
                        ui.monospace(format!("Pan: {:>15}", format!("{:?}", self.pan)));
                    });
                });

                let sidebar_frame = egui::Frame {
                    inner_margin: egui::style::Margin::symmetric(8.0, 0.0),
                    fill: ctx.style().visuals.window_fill(),
                    stroke: ctx.style().visuals.window_stroke(),
                    ..Default::default()
                };
                egui::SidePanel::left("level_sidebar")
                    .frame(sidebar_frame)
                    .min_width(150.0)
                    .max_width(400.0)
                    .show_inside(ui, |ui| {
                        ui.spacing_mut().item_spacing = default_spacing;
                        self.level_sidebar_gui(project, ui, frame);
                    });

                // Reset the spacing to the default after adding the panels
                ui.spacing_mut().item_spacing = default_spacing;

                egui::CentralPanel::default()
                    .frame(egui::Frame::canvas(&ctx.style()))
                    .show_inside(ui, |ui| {
                        self.map_canvas_gui(project, ui, frame);
                    });
            });
    }

    fn help_text(&self) -> &'static str {
        include_str!("./maps_help.txt")
    }

    fn tooltip(&self) -> &'static str {
        "Create maps and levels from metatiles"
    }
}

impl MapsTab {
    fn level_sidebar_gui(
        &mut self,
        project: &mut ProjectState,
        ui: &mut egui::Ui,
        frame: &mut eframe::Frame,
    ) {
        ui.add_space(ui.spacing().window_margin.top);
        ui.horizontal(|ui| {
            ui.horizontal(|ui| {
                ui.set_height(ui.spacing().interact_size.y);
                ui.label("Level");

                ui.with_layout(egui::Layout::right_to_left(), |ui| {
                    ui.set_enabled(self.current_level.is_some());
                    if ui.button("🗑").on_hover_text("Delete level.").clicked() {
                        project.data.levels.remove(&self.current_level.unwrap());
                        self.current_level = None;
                    }
                });
            });
        });
        ui.separator();

        let level_id = if let Some(id) = self.current_level {
            id
        } else {
            // If we don't have a selected level, that means there are no levels yet
            ui.label("Right-click on the canvas to create a new level.");
            return;
        };

        let level = project.data.levels.get_mut(&level_id).unwrap();

        ui.horizontal(|ui| {
            ui.label("Name: ");
            ui.text_edit_singleline(&mut level.name);
        });

        let metatileset = project.data.metatilesets.get(&level.metatileset_id);
        ui.horizontal(|ui| {
            ui.label("Metatileset: ");
            egui::ComboBox::from_id_source("metatileset")
                .selected_text(metatileset.map(|x| x.name.as_str()).unwrap_or("None"))
                .show_ui(ui, |ui| {
                    for (metatileset_id, metatileset) in &project.data.metatilesets {
                        ui.selectable_value(
                            &mut level.metatileset_id,
                            *metatileset_id,
                            &metatileset.name,
                        );
                    }
                });
        });

        ui.separator();

        ui.label("Margins:");
        ui.indent("margin", |ui| {
            ui.horizontal(|ui| {
                ui.label("Top");
                ui.with_layout(egui::Layout::right_to_left(), |ui| {
                    ui.add(DragValue::new(&mut level.margin.top).speed(-0.25));
                });
            });
            ui.horizontal(|ui| {
                ui.label("Left");
                ui.with_layout(egui::Layout::right_to_left(), |ui| {
                    ui.add(DragValue::new(&mut level.margin.left).speed(-0.25));
                });
            });
            ui.horizontal(|ui| {
                ui.label("Right");
                ui.with_layout(egui::Layout::right_to_left(), |ui| {
                    ui.add(DragValue::new(&mut level.margin.right));
                });
            });
            ui.horizontal(|ui| {
                ui.label("Bottom");
                ui.with_layout(egui::Layout::right_to_left(), |ui| {
                    ui.add(DragValue::new(&mut level.margin.bottom));
                });
            });
        });
        ui.separator();

        let metatileset_id = level.metatileset_id;
        let metatileset = if let Some(metatileset) = project.data.metatilesets.get(&metatileset_id)
        {
            metatileset
        } else {
            return;
        };

        ui.horizontal(|ui| {
            ui.set_height(ui.spacing().interact_size.y);
            ui.label("Tiles");
        });
        ui.separator();

        let item_spacing = egui::Vec2::splat(ui.spacing().item_spacing.x);
        ui.spacing_mut().item_spacing = item_spacing;
        let hovered_stroke_color = ui.visuals().widgets.hovered.fg_stroke.color;
        let tile_rounding = 2.0;

        let tile_ids = metatileset.tiles.keys().cloned().collect::<Vec<_>>();
        ui.scope(|ui| {
            egui::ScrollArea::new([false, true]).show(ui, |ui| {
                ui.add_space(ui.spacing().item_spacing.y);
                ui.horizontal_wrapped(|ui| {
                    for id in tile_ids {
                        let tile_region_display_size = egui::Vec2::splat(
                            ui.available_width() / self.tile_list_col_count as f32,
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
                        .paint_at(rect, ui, frame);

                        if self.current_metatileset_tile == Some(id) {
                            ui.painter().rect_stroke(
                                rect,
                                tile_rounding,
                                (2.0, egui::Color32::GREEN),
                            );
                        } else if response.hovered() {
                            ui.painter().rect_stroke(
                                rect,
                                tile_rounding,
                                (2.0, hovered_stroke_color),
                            );
                        }

                        if response.clicked() {
                            self.current_metatileset_tile = Some(id);
                        }
                    }
                });
                ui.add_space(ui.spacing().item_spacing.y);
            });
        });
    }

    fn map_canvas_gui(
        &mut self,
        project: &mut ProjectState,
        ui: &mut egui::Ui,
        frame: &mut eframe::Frame,
    ) {
        let canvas_rect = ui.max_rect();
        let canvas_center = canvas_rect.center();
        ui.set_clip_rect(canvas_rect);

        let mut response = ui.interact(
            canvas_rect,
            egui::Id::new("map_canvas"),
            egui::Sense::click_and_drag(),
        );

        // Handle zoom
        if response.hovered() {
            self.zoom *= 1000.0;
            self.zoom += ui.input().scroll_delta.y * 5.0;
            self.zoom /= 1000.0;
            self.zoom = 0.1_f32.max(self.zoom);
        }

        // Handle pan
        let panning_map_view =
            response.dragged_by(egui::PointerButton::Middle) || ui.input().modifiers.command;
        if response.dragged_by(egui::PointerButton::Middle) || ui.input().modifiers.command {
            self.pan += response.drag_delta();
        }

        // Handle cursor
        if response.dragged_by(egui::PointerButton::Middle)
            || (ui.input().modifiers.command && response.dragged_by(egui::PointerButton::Primary))
        {
            response = response.on_hover_cursor(egui::CursorIcon::Grabbing);
        } else if ui.input().modifiers.command {
            response = response.on_hover_cursor(egui::CursorIcon::Grab);
        }

        let pointer_pos = ui.input().pointer.interact_pos();

        // Add context menu
        response = response.context_menu(|ui| {
            if ui.button("➕ Create Level").clicked() {
                ui.close_menu();

                if let Some(pos) = pointer_pos {
                    let world_pos =
                        (pos.to_vec2() - canvas_center.to_vec2() - self.pan) / self.zoom;

                    // Add new level
                    let id = Uid::new();
                    project.data.levels.insert(
                        id,
                        Level {
                            world_offset: world_pos,
                            ..Default::default()
                        },
                    );
                    self.current_level = Some(id);
                }
            }
        });

        let mut new_tile = None;
        let level_ids = project.data.levels.keys().cloned().collect::<Vec<_>>();
        for id in level_ids {
            let level = project.data.levels.get(&id).unwrap();
            let level_margin = level.margin;

            let tile_size = 16_f32;
            let canvas_pos = canvas_center + level.world_offset * self.zoom + self.pan;
            let min = canvas_pos
                - egui::Vec2::new(
                    level_margin.left as f32 * tile_size * self.zoom,
                    level_margin.top as f32 * tile_size * self.zoom,
                );
            let max = canvas_pos
                + egui::Vec2::new(
                    level_margin.right as f32 * tile_size * self.zoom,
                    level_margin.bottom as f32 * tile_size * self.zoom,
                );
            let level_rect = egui::Rect { min, max };

            // Render the map label
            let label_pos = level_rect.center_top();
            let label_rect = ui.painter().text(
                label_pos,
                egui::Align2::CENTER_BOTTOM,
                if level.name.is_empty() {
                    "Untitled"
                } else {
                    &level.name
                },
                egui::FontId::monospace(20.0),
                ui.style().visuals.text_color(),
            );

            // Render the map
            let metatileset_id = level.metatileset_id;
            if let Some(metatileset) = project.data.metatilesets.get(&metatileset_id) {
                // Paint the background color
                let background_color = NES_PALLET[metatileset.pallet.colors[0] as usize];
                ui.painter().rect_filled(level_rect, 2.0, background_color);

                let tiles = level
                    .tiles
                    .iter()
                    .map(|(pos, tile)| (pos.clone(), tile.clone()))
                    .collect::<Vec<_>>();

                for ((level_x, level_y), tile) in tiles {
                    let metatileset_tile_id = tile.metatileset_tile_id;
                    if level_x > -level_margin.left - 1
                        && level_x < level_margin.right
                        && level_y > -level_margin.top - 1
                        && level_y < level_margin.bottom
                    {
                        let x = level_x + level_margin.left;
                        let y = level_y + level_margin.top;
                        let tile_size = 16.0 * self.zoom;
                        let tile_rect = egui::Rect::from_min_size(
                            level_rect.min
                                + egui::Vec2::new(x as f32 * tile_size, y as f32 * tile_size),
                            egui::Vec2::splat(tile_size),
                        );

                        MetatileGui::new(
                            project,
                            MetatileKind::Metatileset {
                                metatileset_id,
                                metatileset_tile_id,
                            },
                        )
                        .paint_at(tile_rect, ui, frame);
                    }
                }
            }

            let pointer_within_label = pointer_pos.map(|x| label_rect.contains(x)).unwrap_or(false);
            let pointer_within_level = pointer_pos.map(|x| level_rect.contains(x)).unwrap_or(false);

            // Check drag state and update cursor
            if response.dragged_by(egui::PointerButton::Primary)
                && response.drag_started()
                && pointer_within_label
                && !panning_map_view
            {
                self.dragging_level = Some(id);
            } else if self.dragging_level.is_some() {
                response = response.on_hover_cursor(egui::CursorIcon::Grabbing);
            } else if pointer_within_label {
                response = response.on_hover_cursor(egui::CursorIcon::Grab);
            }

            // Focus level if clicked
            if response.clicked_by(egui::PointerButton::Primary)
                && (pointer_within_level || pointer_within_label)
            {
                self.current_level = Some(id);
            }

            // Render the hover cursor
            if pointer_within_level && self.current_level == Some(id) {
                let pointer_pos = pointer_pos.unwrap().to_vec2(); // We know it's within the level rect
                let rel_pointer_pos = pointer_pos - level_rect.min.to_vec2();

                let level_size_in_tiles =
                    egui::Vec2::new(level_margin.width() as f32, level_margin.height() as f32);
                let tile_size = level_rect.size() / level_size_in_tiles;

                let pointer_uv = rel_pointer_pos / level_rect.size();
                let pointer_xy =
                    level_rect.min + (pointer_uv * level_size_in_tiles).floor() * tile_size;

                let tile_rect = egui::Rect::from_min_size(pointer_xy, tile_size);

                if let Some(metatileset_tile_id) = self.current_metatileset_tile {
                    MetatileGui::new(
                        project,
                        MetatileKind::Metatileset {
                            metatileset_id,
                            metatileset_tile_id,
                        },
                    )
                    .paint_at(tile_rect, ui, frame);

                    if ui.input().pointer.button_down(egui::PointerButton::Primary) {
                        let tile_xy_idx = (pointer_uv * level_size_in_tiles).floor();
                        let level_x_idx = -level_margin.left + tile_xy_idx.x as i32;
                        let level_y_idx = -level_margin.top + tile_xy_idx.y as i32;

                        new_tile = Some((
                            id,
                            (level_x_idx, level_y_idx),
                            LevelTile {
                                metatileset_tile_id,
                            },
                        ));
                    }
                }
            }

            // Render the level stroke
            ui.painter().rect_stroke(
                level_rect,
                1.0,
                if self.current_level == Some(id) {
                    ui.style().visuals.widgets.active.fg_stroke
                } else {
                    ui.style().visuals.widgets.inactive.fg_stroke
                },
            );
        }

        // Add a new tile if one was place
        if let Some((level_id, pos, tile)) = new_tile {
            let level = project.data.levels.get_mut(&level_id).unwrap();
            level.tiles.insert(pos, tile);
        }

        // Clear drag state if not dragging
        if !response.dragged() {
            self.dragging_level = None;
        }

        // Bring selected level to the front of the stack ( last in render order ) if it isn't already
        if let Some(id) = &self.current_level {
            if let Some(idx) = project.data.levels.get_index_of(id) {
                let last_idx = project.data.levels.len() - 1;
                if idx != last_idx {
                    project.data.levels.swap_indices(last_idx, idx);
                }
            }
        }

        // Drag the level if dragged
        if let Some(id) = self.dragging_level {
            let level = project.data.levels.get_mut(&id).unwrap();

            level.world_offset += response.drag_delta() / self.zoom;
        }
    }
}
