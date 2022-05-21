use eframe::{egui, epaint::textures::TextureFilter};
use egui::{Color32, ColorImage, Layout, RichText, Ui};
use egui_extras::RetainedImage;
use image::GenericImageView;
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, fs, io::Read};

use tracing as trc;

use crate::components::{nes_color_picker, NesImageViewer};

use self::keyboard_shortcuts::KEYBOARD_SHORTCUTS;

mod keyboard_shortcuts;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct NesimgGui {
    palette: [u8; 13],
    current_palette: Pallet,

    #[serde(skip)]
    source_image: Option<ColorImage>,
    #[serde(skip)]
    source_texture: Option<RetainedImage>,

    dark_mode: bool,

    #[serde(skip)]
    error_message: Option<String>,

    #[serde(skip)]
    open_image_request_sender: flume::Sender<&'static str>,
    #[serde(skip)]
    open_image_response_receiver: flume::Receiver<(&'static str, Vec<u8>)>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
enum Pallet {
    First,
    Second,
    Third,
    Fourth,
}

impl Default for NesimgGui {
    fn default() -> Self {
        let (open_image_request_sender, open_image_request_receiver) = flume::bounded(1);
        let (open_image_response_sender, open_image_response_receiver) = flume::bounded(1);

        // TODO: Image loading on WASM
        #[cfg(not(target_arch = "wasm32"))]
        std::thread::spawn(move || {
            while let Ok(name) = open_image_request_receiver.recv() {
                trc::trace!("Got request for file load: {}", name);
                let dialog = rfd::FileDialog::new().add_filter("PNG", &["png"]);
                trc::trace!("Showing file dialog");
                let file = dialog.pick_file();

                if let Some(file) = file {
                    trc::debug!(?file, "Picked file");

                    trc::trace!("Opening file...");
                    let mut file = match fs::OpenOptions::new().read(true).open(file) {
                        Ok(f) => f,
                        Err(e) => {
                            println!("Error: {}", e);
                            continue;
                        }
                    };

                    let mut contents = Vec::new();
                    trc::trace!("Reading file...");
                    if let Err(e) = file.read_to_end(&mut contents) {
                        println!("Error: {}", e);
                        continue;
                    }

                    open_image_response_sender.send((name, contents)).ok();
                    trc::trace!("File loaded");
                } else {
                    trc::trace!("No file picked");
                }
            }
        });

        Self {
            dark_mode: true,
            source_image: None,
            source_texture: None,
            error_message: None,
            palette: [
                // Default to simple grayscale pallet
                0x0F, 0x1D, 0x2D, 0x3D, 0x1D, 0x2D, 0x3D, 0x1D, 0x2D, 0x3D, 0x1D, 0x2D, 0x3D,
            ],
            current_palette: Pallet::First,
            open_image_request_sender,
            open_image_response_receiver,
        }
    }
}

impl NesimgGui {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Load previous app state (if any).
        if let Some(storage) = cc.storage {
            let gui: NesimgGui = eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();

            if gui.dark_mode {
                cc.egui_ctx.set_visuals(egui::style::Visuals::dark());
            }

            gui
        } else {
            // Default to dark theme
            cc.egui_ctx.set_visuals(egui::style::Visuals::dark());

            Default::default()
        }
    }

    fn toggle_dark_mode(&mut self, ui: &mut Ui) {
        if ui.visuals().dark_mode {
            self.dark_mode = false;
            ui.ctx().set_visuals(egui::Visuals::light())
        } else {
            self.dark_mode = true;
            ui.ctx().set_visuals(egui::Visuals::dark())
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub(crate) enum Action {
    Quit,
    LoadImage,
}

impl Action {
    fn perform(&self, data: &mut NesimgGui, _ctx: &egui::Context, frame: &mut eframe::Frame) {
        match self {
            Action::Quit => frame.quit(),
            Action::LoadImage => {
                data.open_image_request_sender
                    .send("source_image")
                    .expect("Open file");
            }
        }
    }
}

impl eframe::App for NesimgGui {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        for (action, shortcut) in &*KEYBOARD_SHORTCUTS {
            if ctx
                .input_mut()
                .consume_key(shortcut.modifiers, shortcut.key)
            {
                action.perform(self, ctx, frame);
            }
        }

        // Load the source image if the user has selected one
        if let Ok((name, bytes)) = self.open_image_response_receiver.try_recv() {
            match name {
                "source_image" => {
                    trc::trace!("Uploading image to texture");
                    match load_image(&bytes) {
                        Ok(i) => {
                            self.source_texture = Some(i);
                        }
                        Err(e) => {
                            trc::error!("Error loading image: {}", e);
                            self.error_message = Some(e.to_string());
                        }
                    };
                }
                _ => panic!("Unrecognized file loaded"),
            }
        }

        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            ui.add_space(1.0);
            ui.horizontal(|ui| {
                ui.with_layout(Layout::right_to_left(), |ui| {
                    egui::warn_if_debug_build(ui);
                    if let Some(message) = &self.error_message {
                        if ui
                            .selectable_label(
                                false,
                                RichText::new(format!("Error: {}", message)).color(Color32::RED),
                            )
                            .on_hover_text("Dismiss")
                            .clicked()
                        {
                            self.error_message = None;
                        }
                    }

                    ui.with_layout(Layout::left_to_right(), |ui| {
                        ui.menu_button("File", |ui| {
                            let open_shortcut = KEYBOARD_SHORTCUTS
                                .get(&Action::LoadImage)
                                .map_or(String::new(), |x| format!("\t{}", x));

                            let quit_shortcut = KEYBOARD_SHORTCUTS
                                .get(&Action::Quit)
                                .map_or(String::new(), |x| format!("\t{}", x));

                            if ui.button(format!("Load Image{}", open_shortcut)).clicked() {
                                Action::LoadImage.perform(self, ctx, frame);
                            }

                            ui.separator();

                            if ui.button(format!("Quit{}", quit_shortcut)).clicked() {
                                frame.quit();
                            }
                        });

                        ui.menu_button("UI", |ui| {
                            if ui.button("Toggle Dark Mode").clicked() {
                                self.toggle_dark_mode(ui);
                            }
                        });
                    });
                });
            });
        });

        egui::SidePanel::right("side_panel").show(ctx, |ui| {
            ui.set_width_range(230.0..=f32::INFINITY);
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.add_space(ui.spacing().item_spacing.y);

                ui.horizontal(|ui| {
                    ui.with_layout(Layout::right_to_left(), |ui| {
                        // We might want a button here later
                        // if ui.button("ℹ").on_hover_text("Full NES Palette").clicked() {
                        // }

                        ui.with_layout(Layout::left_to_right(), |ui| {
                            ui.heading("Pallet");
                        });
                    });
                });

                ui.separator();

                ui.horizontal(|ui| {
                    ui.radio_value(&mut self.current_palette, Pallet::First, "");
                    nes_color_picker(ui, &mut self.palette[0]);
                    nes_color_picker(ui, &mut self.palette[1]);
                    nes_color_picker(ui, &mut self.palette[2]);
                    nes_color_picker(ui, &mut self.palette[3]);
                });
                ui.horizontal(|ui| {
                    ui.radio_value(&mut self.current_palette, Pallet::Second, "");
                    nes_color_picker(ui, &mut self.palette[0]);
                    nes_color_picker(ui, &mut self.palette[4]);
                    nes_color_picker(ui, &mut self.palette[5]);
                    nes_color_picker(ui, &mut self.palette[6]);
                });
                ui.horizontal(|ui| {
                    ui.radio_value(&mut self.current_palette, Pallet::Third, "");
                    nes_color_picker(ui, &mut self.palette[0]);
                    nes_color_picker(ui, &mut self.palette[7]);
                    nes_color_picker(ui, &mut self.palette[8]);
                    nes_color_picker(ui, &mut self.palette[9]);
                });
                ui.horizontal(|ui| {
                    ui.radio_value(&mut self.current_palette, Pallet::Fourth, "");
                    nes_color_picker(ui, &mut self.palette[0]);
                    nes_color_picker(ui, &mut self.palette[10]);
                    nes_color_picker(ui, &mut self.palette[11]);
                    nes_color_picker(ui, &mut self.palette[12]);
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.set_min_width(100.0);
            if let Some(image) = &self.source_texture {
                NesImageViewer::new("image_view", image, &self.palette).show(ui, frame);
            } else {
                ui.vertical_centered(|ui| {
                    if ui.button("Load Image...").clicked() {
                        Action::LoadImage.perform(self, ctx, frame);
                    }
                });
            }
        });
    }
}

pub(crate) fn load_image(bytes: &[u8]) -> anyhow::Result<RetainedImage> {
    let image = image::load_from_memory(bytes)?;

    if image.width() % 16 != 0 || image.height() % 16 != 0 {
        anyhow::bail!("Image width and height must be a multiple of 16");
    }

    let mut colors = HashSet::new();

    for (_, _, pixel) in image.pixels() {
        colors.insert(pixel);
    }

    if colors.len() != 4 {
        anyhow::bail!(
            "Image must have only 4 colors, but found {} colors",
            colors.len()
        );
    }

    let mut colors_sorted = colors.iter().collect::<Vec<_>>();
    colors_sorted.sort_unstable_by(|x, y| {
        let x = x[0] as u16 + x[1] as u16 + x[2] as u16;
        let y = y[0] as u16 + y[1] as u16 + y[2] as u16;
        x.cmp(&y)
    });

    let pixels = image
        .pixels()
        .map(|(_, _, x)| {
            if &x == colors_sorted[0] {
                Color32::from_rgb(0, 0, 0)
            } else if &x == colors_sorted[1] {
                Color32::from_rgb(85, 85, 85)
            } else if &x == colors_sorted[2] {
                Color32::from_rgb(170, 170, 170)
            } else if &x == colors_sorted[3] {
                Color32::from_rgb(255, 255, 255)
            } else {
                unreachable!()
            }
        })
        .collect();

    let final_image = ColorImage {
        size: [image.width() as usize, image.height() as usize],
        pixels,
    };

    Ok(RetainedImage::from_color_image(
        "source_image",
        final_image,
        TextureFilter::Nearest,
    ))
}
