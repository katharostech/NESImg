use egui::{Color32, ComboBox, Layout};
use ulid::Ulid;

use crate::{
    gui::{components::MetatileGui, project_state::SourceImageData, ProjectState},
    project::{Metatile, Tile},
};

use super::NesimgGuiTab;

pub struct MetatilesTab {
    current_source_image: Option<Ulid>,
    current_source_image_tile: Option<Tile>,
    current_metatile: Option<Ulid>,
    metatile_list_col_count: u32,
}

impl Default for MetatilesTab {
    fn default() -> Self {
        Self {
            current_source_image: Default::default(),
            current_source_image_tile: None,
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

        egui::SidePanel::right("metatiles_sidebar")
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
                            let id = Ulid::new();
                            project.data.metatiles.insert(
                                id,
                                Metatile {
                                    tiles: [None, None, None, None],
                                },
                            );
                            self.current_metatile = Some(id);
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

                                MetatileGui::new(project, id).show_at(rect, ui, frame);

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
                        ui.add_space(ui.spacing().item_spacing.y);
                    });
                });
            });

        egui::SidePanel::left("source_image_sidebar")
            .default_width(800.0)
            .min_width(200.0)
            .show(ctx, |ui| {
                ui.add_space(ui.spacing().window_margin.top);
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
                                source_image_viewer(
                                    id,
                                    &image,
                                    project,
                                    &mut self.current_source_image_tile,
                                    ui,
                                );
                            }
                        }
                    } else {
                        ui.label("No source file selected");
                    }
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.set_height(ui.spacing().interact_size.y);
                ui.label("Edit Metatile");
            });
            ui.separator();

            ui.centered_and_justified(|ui| {
                if let Some(id) = self.current_metatile {
                    metatile_editor(id, project, &self.current_source_image_tile, ui, frame);
                } else {
                    ui.label("No metatile selected");
                }
            });
        });
    }

    fn help_text(&self) -> &'static str {
        include_str!("./metatiles_help.txt")
    }
}

#[derive(Copy, Clone)]
struct MetatileEditorState {
    zoom: f32,
    pan: egui::Vec2,
}

impl Default for MetatileEditorState {
    fn default() -> Self {
        Self {
            zoom: 14.0,
            pan: egui::Vec2::ZERO,
        }
    }
}

const METATILE_SIZE: egui::Vec2 = egui::Vec2::splat(16.0);
const TILE_SIZE: egui::Vec2 = egui::Vec2::splat(8.0);

fn metatile_editor(
    metatile_id: Ulid,
    project: &mut ProjectState,
    current_source_image_tile: &Option<Tile>,
    ui: &mut egui::Ui,
    frame: &mut eframe::Frame,
) {
    let id = ui.id();
    let (rect, response) =
        ui.allocate_exact_size(ui.available_size(), egui::Sense::click_and_drag());
    let is_clicked = response.clicked_by(egui::PointerButton::Primary);

    ui.set_clip_rect(rect);

    let mut state: MetatileEditorState = *ui.data().get_temp_mut_or_default(id);

    // Handle zoom
    if response.hovered() {
        state.zoom += ui.input().scroll_delta.y * 0.01;
        state.zoom = state.zoom.max(0.5);
    }

    // Handle pan
    if response.dragged_by(egui::PointerButton::Middle) || ui.input().modifiers.command {
        state.pan += response.drag_delta();
    }

    // Handle cursor
    if response.dragged_by(egui::PointerButton::Middle)
        || (ui.input().modifiers.command && response.dragged_by(egui::PointerButton::Primary))
    {
        response.on_hover_cursor(egui::CursorIcon::Grabbing);
    } else if ui.input().modifiers.command {
        response.on_hover_cursor(egui::CursorIcon::Grab);
    } else {
        response.on_hover_cursor(egui::CursorIcon::Crosshair);
    }

    // Calculate image rect render
    let min = rect.center() - METATILE_SIZE / 2.0 * state.zoom;
    let max = min + METATILE_SIZE * state.zoom;
    let image_rect = egui::Rect { min, max }.translate(state.pan);

    // Render metatile
    MetatileGui::new(project, metatile_id).show_at(image_rect, ui, frame);

    /// How wide a metatile is in tiles
    const TILES_WIDE: u8 = 2;

    // Render the tile hover stroke
    let tile_size = TILE_SIZE * state.zoom;
    let mouse_pos = ui.input().pointer.interact_pos();
    if let Some(mouse_pos) = mouse_pos {
        if image_rect.contains(mouse_pos) {
            let mouse_image_pos = mouse_pos - image_rect.min;
            let tile_pos = (mouse_image_pos / tile_size).floor();
            let min = tile_pos * tile_size;
            let max = min + tile_size;
            let hover_rect = egui::Rect {
                min: min.to_pos2(),
                max: max.to_pos2(),
            }
            .translate(image_rect.min.to_vec2());

            let hover_stroke_color = ui.visuals().widgets.hovered.fg_stroke.color;
            ui.painter()
                .rect_stroke(hover_rect, 1.0, (2.0, hover_stroke_color));

            // Paint a tile on the metatile
            if is_clicked {
                if let Some(metatile) = project.data.metatiles.get_mut(&metatile_id) {
                    let tile_idx = tile_pos.y as u8 * TILES_WIDE + tile_pos.x as u8;
                    metatile.tiles[tile_idx as usize] = current_source_image_tile.clone();
                }
            }
        }
    }

    *ui.data().get_temp_mut_or_default(id) = state;
}

#[derive(Copy, Clone)]
struct SourceImageViewerState {
    zoom: f32,
    pan: egui::Vec2,
    drag_start: Option<egui::Vec2>,
}

impl Default for SourceImageViewerState {
    fn default() -> Self {
        Self {
            zoom: 2.0,
            pan: egui::Vec2::ZERO,
            drag_start: None,
        }
    }
}

fn source_image_viewer(
    source_image_id: Ulid,
    source_image_data: &SourceImageData,
    project: &mut ProjectState,
    current_source_image_tile: &mut Option<Tile>,
    ui: &mut egui::Ui,
) {
    let id = ui.id();
    let (rect, response) =
        ui.allocate_exact_size(ui.available_size(), egui::Sense::click_and_drag());

    let is_clicked = response.clicked_by(egui::PointerButton::Primary);
    let is_dragged_by_primary = response.dragged_by(egui::PointerButton::Primary);
    let select_drag_started = response.drag_started() && is_dragged_by_primary;
    let drag_released = response.drag_released() || !response.dragged();

    let image_size = source_image_data.texture.size_vec2();

    ui.set_clip_rect(rect);

    let mut state: SourceImageViewerState = *ui.data().get_temp_mut_or_default(id);

    // Make sure selected tile is within image bounds and for the current source image
    if let Some(tile) = current_source_image_tile {
        tile.source_id = source_image_id;
        tile.x = tile.x.min(image_size.x as u16 / 8 - 1);
        tile.y = tile.y.min(image_size.y as u16 / 8 - 1);
    }

    // Handle zoom
    if response.hovered() {
        state.zoom += ui.input().scroll_delta.y * 0.01;
        state.zoom = state.zoom.max(0.5);
    }

    // Handle pan
    if response.dragged_by(egui::PointerButton::Middle) || ui.input().modifiers.command {
        state.pan += response.drag_delta();
    }

    // Handle cursor
    if response.dragged_by(egui::PointerButton::Middle)
        || (ui.input().modifiers.command && response.dragged_by(egui::PointerButton::Primary))
    {
        response.on_hover_cursor(egui::CursorIcon::Grabbing);
    } else if ui.input().modifiers.command {
        response.on_hover_cursor(egui::CursorIcon::Grab);
    } else {
        response.on_hover_cursor(egui::CursorIcon::Crosshair);
    }

    // Calculate image rect render
    let min = rect.center() - image_size / 2.0 * state.zoom;
    let max = min + image_size * state.zoom;
    let image_rect = egui::Rect { min, max }.translate(state.pan);

    // Render the image
    egui::Image::new(
        source_image_data.texture.texture_id(ui.ctx()),
        image_rect.size(),
    )
    .paint_at(ui, image_rect);

    let mut drag_selecting = false;

    let tile_size = TILE_SIZE * state.zoom;
    let mouse_pos = ui.input().pointer.interact_pos();
    if let Some(mouse_pos) = mouse_pos {
        if image_rect.contains(mouse_pos) {
            let mouse_image_pos = mouse_pos - image_rect.min;
            let hover_xy_idx = (mouse_image_pos / TILE_SIZE / state.zoom).floor();

            // Helper to get the egui rect for the a tile in the image, based on it's tile xy
            // location
            let get_tile_rect_from_xy_idx = |xy_idx: egui::Vec2| -> egui::Rect {
                let tile_image_pos =
                    egui::Pos2::new(xy_idx.x as f32 * tile_size.x, xy_idx.y as f32 * tile_size.y);
                let tile_rect = egui::Rect {
                    min: tile_image_pos,
                    max: tile_image_pos + tile_size,
                }
                .translate(image_rect.min.to_vec2());

                tile_rect
            };

            // Handle drag selecting
            if select_drag_started {
                state.drag_start = Some(hover_xy_idx);
            }

            // Calculate drag region
            if let Some(drag_start_xy_idx) = state.drag_start {
                // Stop dragging
                if drag_released {
                    state.drag_start = None;
                } else if drag_start_xy_idx != hover_xy_idx {
                    drag_selecting = true;
                }

                // Calculate the selected metatiles
                let tiles_min_x_idx = (drag_start_xy_idx.x as u16).min(hover_xy_idx.x as u16);
                let tiles_min_y_idx = (drag_start_xy_idx.y as u16).min(hover_xy_idx.y as u16);
                let mut tiles_max_x_idx = (drag_start_xy_idx.x as u16).max(hover_xy_idx.x as u16);
                let mut tiles_max_y_idx = (drag_start_xy_idx.y as u16).max(hover_xy_idx.y as u16);

                // Shift the select region to the left or top, if we are selecting in that direction
                if hover_xy_idx.x < drag_start_xy_idx.x {
                    tiles_max_x_idx -= 1;
                }
                if hover_xy_idx.y < drag_start_xy_idx.y {
                    tiles_max_y_idx -= 1;
                }

                let metatiles_wide = (tiles_max_x_idx - tiles_min_x_idx) / 2 + 1;
                let metatiles_high = (tiles_max_y_idx - tiles_min_y_idx) / 2 + 1;

                for metatile_x_idx in 0..metatiles_wide {
                    for metatile_y_idx in 0..metatiles_high {
                        let tile_0 = Tile {
                            source_id: source_image_id,
                            x: tiles_min_x_idx + metatile_x_idx * 2,
                            y: tiles_min_y_idx + metatile_y_idx * 2,
                        };
                        let tile_1 = Tile {
                            source_id: source_image_id,
                            x: tiles_min_x_idx + metatile_x_idx * 2 + 1,
                            y: tiles_min_y_idx + metatile_y_idx * 2,
                        };
                        let tile_2 = Tile {
                            source_id: source_image_id,
                            x: tiles_min_x_idx + metatile_x_idx * 2,
                            y: tiles_min_y_idx + metatile_y_idx * 2 + 1,
                        };
                        let tile_3 = Tile {
                            source_id: source_image_id,
                            x: tiles_min_x_idx + metatile_x_idx * 2 + 1,
                            y: tiles_min_y_idx + metatile_y_idx * 2 + 1,
                        };

                        let min_tile_rect = get_tile_rect_from_xy_idx(egui::Vec2::new(
                            tile_0.x as f32,
                            tile_0.y as f32,
                        ));
                        let max_tile_rect = get_tile_rect_from_xy_idx(egui::Vec2::new(
                            tile_3.x as f32,
                            tile_3.y as f32,
                        ));

                        // Skip out-of-bounds metatiles
                        if max_tile_rect.max.x > image_rect.max.x
                            || max_tile_rect.max.y > image_rect.max.y
                            || min_tile_rect.min.x < image_rect.min.x
                            || min_tile_rect.min.y < image_rect.min.y
                        {
                            continue;
                        }

                        // Paint the drag region
                        if drag_selecting {
                            let highlight_rect = egui::Rect {
                                min: min_tile_rect.min,
                                max: max_tile_rect.max,
                            };
                            ui.painter().rect_stroke(
                                highlight_rect,
                                1.0,
                                (2.0, egui::Color32::BLUE),
                            );
                        }

                        // Add the selected metatiles
                        if drag_released && drag_start_xy_idx != hover_xy_idx {
                            let metatile = Metatile {
                                tiles: [Some(tile_0), Some(tile_1), Some(tile_2), Some(tile_3)],
                            };
                            project.data.metatiles.insert(Ulid::new(), metatile);
                        }
                    }
                }
            }

            // Render the hover stroke
            if !drag_selecting {
                let hover_stroke_color = ui.visuals().widgets.hovered.fg_stroke.color;

                let hover_rect = get_tile_rect_from_xy_idx(hover_xy_idx);
                ui.painter()
                    .rect_stroke(hover_rect, 1.0, (2.0, hover_stroke_color));
            }

            // Select the clicked tile
            if is_clicked {
                *current_source_image_tile = Some(Tile {
                    source_id: source_image_id,
                    x: hover_xy_idx.x as u16,
                    y: hover_xy_idx.y as u16,
                })
            }
        }
    }

    // Render the selected tile border
    if !drag_selecting {
        if let Some(tile) = current_source_image_tile {
            let tile_pos = egui::Vec2::new(tile.x as f32, tile.y as f32);
            let min = tile_pos * tile_size;
            let max = min + tile_size;
            let select_rect = egui::Rect {
                min: min.to_pos2(),
                max: max.to_pos2(),
            }
            .translate(image_rect.min.to_vec2());

            let selected_stroke_color = egui::Color32::GREEN;
            ui.painter()
                .rect_stroke(select_rect, 1.0, (2.0, selected_stroke_color));
        }
    }

    *ui.data().get_temp_mut_or_default(id) = state;
}
