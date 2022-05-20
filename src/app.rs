use eframe::{egui, epaint::textures::TextureFilter};
use egui::{ColorImage, Layout, Vec2};
use std::{fs, io::Read};

use egui_extras::RetainedImage;
use tracing as trc;

use crate::components::{nes_color_picker, NesImageViewer};

use self::keyboard_shortcuts::KEYBOARD_SHORTCUTS;

mod keyboard_shortcuts;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct NesimgGui {
    #[serde(skip)]
    image_offset: Vec2,
    #[serde(skip)]
    image_zoom: f32,

    #[serde(skip)]
    source_image: Option<ColorImage>,
    #[serde(skip)]
    source_texture: Option<RetainedImage>,

    #[serde(skip)]
    palette: [u8; 13],

    #[serde(skip)]
    open_image_request_sender: flume::Sender<&'static str>,
    #[serde(skip)]
    open_image_response_receiver: flume::Receiver<(&'static str, Vec<u8>)>,
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
                    trc::trace!("Image processed");
                } else {
                    trc::trace!("No file picked");
                }
            }
        });

        Self {
            image_offset: Vec2::ZERO,
            image_zoom: 1.0,
            source_image: None,
            source_texture: None,
            palette: [0x0F; 13],
            open_image_request_sender,
            open_image_response_receiver,
        }
    }
}

impl NesimgGui {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Default to dark theme
        cc.egui_ctx.set_visuals(egui::style::Visuals::dark());

        // Load previous app state (if any).
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
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
        if let Ok((name, image)) = self.open_image_response_receiver.try_recv() {
            match name {
                "source_image" => {
                    trc::trace!("Uploading image to texture");
                    match egui_extras::image::load_image_bytes(&image) {
                        Ok(i) => {
                            self.source_image = Some(i.clone());
                            self.source_texture = Some(RetainedImage::from_color_image(
                                "source_image",
                                i,
                                TextureFilter::Nearest,
                            ));
                        }
                        Err(e) => {
                            println!("Error loading image: {}", e);
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
                                if ui.visuals().dark_mode {
                                    ui.ctx().set_visuals(egui::Visuals::light())
                                } else {
                                    ui.ctx().set_visuals(egui::Visuals::dark())
                                }
                            }
                        });
                    });
                });
            });
        });

        egui::SidePanel::right("side_panel").show(ctx, |ui| {
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
                    nes_color_picker(ui, &mut self.palette[0]);
                    nes_color_picker(ui, &mut self.palette[1]);
                    nes_color_picker(ui, &mut self.palette[2]);
                    nes_color_picker(ui, &mut self.palette[3]);
                });
                ui.horizontal(|ui| {
                    nes_color_picker(ui, &mut self.palette[0]);
                    nes_color_picker(ui, &mut self.palette[4]);
                    nes_color_picker(ui, &mut self.palette[5]);
                    nes_color_picker(ui, &mut self.palette[6]);
                });
                ui.horizontal(|ui| {
                    nes_color_picker(ui, &mut self.palette[0]);
                    nes_color_picker(ui, &mut self.palette[7]);
                    nes_color_picker(ui, &mut self.palette[8]);
                    nes_color_picker(ui, &mut self.palette[9]);
                });
                ui.horizontal(|ui| {
                    nes_color_picker(ui, &mut self.palette[0]);
                    nes_color_picker(ui, &mut self.palette[10]);
                    nes_color_picker(ui, &mut self.palette[11]);
                    nes_color_picker(ui, &mut self.palette[12]);
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(texture) = &self.source_texture {
                NesImageViewer::new("image_view", texture).show(ui, frame);
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
