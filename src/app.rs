use anyhow::Context;
use eframe::{egui, epaint::textures::TextureFilter};
use egui::{Color32, ColorImage, Layout, RichText, Ui};
use egui_extras::RetainedImage;
use image::GenericImageView;
use notify::Watcher;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    fs::{self, OpenOptions},
    io::Read,
    path::{Path, PathBuf},
    time::Duration,
};

use tracing as trc;

use crate::{
    components::{nes_color_picker, NesImageViewer},
    globals::{NES_PALLET_RGB, TILE_SIZE, TILE_SIZE_INT},
};

use self::keyboard_shortcuts::KEYBOARD_SHORTCUTS;

mod keyboard_shortcuts;

struct SourceImage {
    path: PathBuf,
    image: ColorImage,
    texture: RetainedImage,
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct NesimgGui {
    current_pallet: Pallet,

    #[serde(skip)]
    pallet: ImagePalletData,

    #[serde(skip)]
    source_image: Option<SourceImage>,

    dark_mode: bool,
    export_on_save: bool,

    #[serde(skip)]
    error_message: Option<String>,

    #[serde(skip)]
    open_image_request_sender: flume::Sender<&'static str>,
    #[serde(skip)]
    open_image_response_receiver: flume::Receiver<(&'static str, PathBuf)>,
    #[serde(skip)]
    file_watcher_path_change_sender: std::sync::mpsc::Sender<PathBuf>,
    #[serde(skip)]
    file_watcher_file_changed_receiver: std::sync::mpsc::Receiver<notify::DebouncedEvent>,
}

#[derive(Copy, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
enum Pallet {
    First,
    Second,
    Third,
    Fourth,
}

impl Into<u8> for Pallet {
    fn into(self) -> u8 {
        match self {
            Pallet::First => 0,
            Pallet::Second => 1,
            Pallet::Third => 2,
            Pallet::Fourth => 3,
        }
    }
}

impl Default for NesimgGui {
    fn default() -> Self {
        let (open_image_request_sender, open_image_request_receiver) = flume::bounded(1);
        let (open_image_response_sender, open_image_response_receiver) = flume::bounded(1);

        // Spawn the file dialog thread
        std::thread::spawn(move || {
            while let Ok(name) = open_image_request_receiver.recv() {
                trc::trace!("Got request for file load: {}", name);
                let dialog = rfd::FileDialog::new().add_filter("PNG", &["png"]);
                trc::trace!("Showing file dialog");
                let file = dialog.pick_file();

                if let Some(path) = file {
                    open_image_response_sender.send((name, path)).ok();
                } else {
                    trc::trace!("No file picked");
                }
            }
        });

        let (file_watcher_path_change_sender, file_watcher_path_change_receiver) =
            std::sync::mpsc::channel();
        let (file_watcher_file_change_sender, file_watcher_file_changed_receiver) =
            std::sync::mpsc::channel();

        // Spawn the file watcher thread
        std::thread::spawn(move || {
            // This is used to keep the watcher in scope while it listens for changes
            let mut watcher: Option<notify::RecommendedWatcher> = None;
            let mut prev_path = None;

            while let Ok(path) = file_watcher_path_change_receiver.recv() {
                if let Some(mut watcher) = watcher.take() {
                    if let Some(prev_path) = prev_path.take() {
                        watcher.unwatch(prev_path).expect("Failed to unwatch file");
                    }
                }

                let mut new_watcher = notify::watcher(
                    file_watcher_file_change_sender.clone(),
                    Duration::from_secs(1),
                )
                .expect("Start file watcher");

                new_watcher
                    .watch(&path, notify::RecursiveMode::NonRecursive)
                    .expect("Watch filesystem");

                prev_path = Some(path);
                watcher = Some(new_watcher);
            }
        });

        Self {
            dark_mode: true,
            source_image: None,
            error_message: None,
            export_on_save: true,
            pallet: ImagePalletData::default(),
            current_pallet: Pallet::First,
            open_image_request_sender,
            open_image_response_receiver,
            file_watcher_path_change_sender,
            file_watcher_file_changed_receiver,
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
    Save,
    Export,
}

impl Action {
    fn perform(&self, data: &mut NesimgGui, _ctx: &egui::Context, frame: &mut eframe::Frame) {
        if let Err(e) = match self {
            Action::Quit => Ok(frame.quit()),
            Action::LoadImage => Ok(data
                .open_image_request_sender
                .send("source_image")
                .expect("Open file")),
            Action::Save => save_project(data),
            Action::Export => export_project(data),
        } {
            trc::error!("{}", e);
            data.error_message = Some(e.to_string());
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

        handle_file_loads(self);

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

                            let save_shortcut = KEYBOARD_SHORTCUTS
                                .get(&Action::Save)
                                .map_or(String::new(), |x| format!("\t{}", x));

                            let export_shortcut = KEYBOARD_SHORTCUTS
                                .get(&Action::Export)
                                .map_or(String::new(), |x| format!("\t{}", x));

                            let quit_shortcut = KEYBOARD_SHORTCUTS
                                .get(&Action::Quit)
                                .map_or(String::new(), |x| format!("\t{}", x));

                            if ui.button(format!("Load Image{}", open_shortcut)).clicked() {
                                Action::LoadImage.perform(self, ctx, frame);
                            }

                            if ui.button(format!("Save Pallet{}", save_shortcut)).clicked() {
                                Action::Save.perform(self, ctx, frame);
                            }

                            if ui.button(format!("Export{}", export_shortcut)).clicked() {
                                Action::Export.perform(self, ctx, frame);
                            }

                            ui.separator();

                            ui.checkbox(&mut self.export_on_save, "Export on Save");

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
                        // if ui.button("ℹ").on_hover_text("Full NES Pallet").clicked() {
                        // }

                        ui.with_layout(Layout::left_to_right(), |ui| {
                            ui.heading("Pallet");
                        });
                    });
                });

                ui.separator();

                ui.horizontal(|ui| {
                    ui.radio_value(&mut self.current_pallet, Pallet::First, "");
                    nes_color_picker(ui, &mut self.pallet.colors[0]);
                    nes_color_picker(ui, &mut self.pallet.colors[1]);
                    nes_color_picker(ui, &mut self.pallet.colors[2]);
                    nes_color_picker(ui, &mut self.pallet.colors[3]);
                });
                ui.horizontal(|ui| {
                    ui.radio_value(&mut self.current_pallet, Pallet::Second, "");
                    nes_color_picker(ui, &mut self.pallet.colors[0]);
                    nes_color_picker(ui, &mut self.pallet.colors[4]);
                    nes_color_picker(ui, &mut self.pallet.colors[5]);
                    nes_color_picker(ui, &mut self.pallet.colors[6]);
                });
                ui.horizontal(|ui| {
                    ui.radio_value(&mut self.current_pallet, Pallet::Third, "");
                    nes_color_picker(ui, &mut self.pallet.colors[0]);
                    nes_color_picker(ui, &mut self.pallet.colors[7]);
                    nes_color_picker(ui, &mut self.pallet.colors[8]);
                    nes_color_picker(ui, &mut self.pallet.colors[9]);
                });
                ui.horizontal(|ui| {
                    ui.radio_value(&mut self.current_pallet, Pallet::Fourth, "");
                    nes_color_picker(ui, &mut self.pallet.colors[0]);
                    nes_color_picker(ui, &mut self.pallet.colors[10]);
                    nes_color_picker(ui, &mut self.pallet.colors[11]);
                    nes_color_picker(ui, &mut self.pallet.colors[12]);
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.set_min_width(100.0);
            if let Some(source) = &self.source_image {
                NesImageViewer::new(
                    "image_view",
                    &source.texture,
                    self.current_pallet.into(),
                    &mut self.pallet,
                )
                .show(ui, frame);
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

fn handle_file_loads(data: &mut NesimgGui) {
    // Load the source image if the user has selected one
    if let Ok((name, path)) = data.open_image_response_receiver.try_recv() {
        match name {
            "source_image" => {
                data.file_watcher_path_change_sender
                    .send(path.clone())
                    .expect("Update file watcher");

                match load_image(&path) {
                    Ok(loaded) => {
                        let image_size_tiles = loaded.source_image.texture.size_vec2() / TILE_SIZE;
                        let tile_count = (image_size_tiles.x * image_size_tiles.y).floor() as u32;
                        data.pallet.tile_pallets = (0..tile_count).into_iter().map(|_| 0).collect();
                        data.source_image = Some(loaded.source_image);

                        if let Some(pallet) = loaded.pallet_save {
                            data.pallet = pallet;
                        }
                    }
                    Err(e) => {
                        trc::error!("Error loading image: {}", e);
                        data.error_message = Some(e.to_string());
                    }
                };
            }
            _ => panic!("Unrecognized file loaded"),
        }
    }

    // Reload the source image if it has been changed on disk
    if let Ok(event) = data.file_watcher_file_changed_receiver.try_recv() {
        if let notify::DebouncedEvent::Write(path) = event {
            match load_image(&path) {
                Ok(loaded) => {
                    let new_source = loaded.source_image;
                    if let Some(old_source) = &data.source_image {
                        if old_source.image.size != new_source.image.size {
                            let image_size_tiles = new_source.texture.size_vec2() / TILE_SIZE;
                            let tile_count =
                                (image_size_tiles.x * image_size_tiles.y).floor() as u32;
                            data.pallet.tile_pallets =
                                (0..tile_count).into_iter().map(|_| 0).collect();
                        }
                    }

                    data.source_image = Some(new_source);
                }
                Err(e) => {
                    trc::error!("Error re-loading image: {}", e);
                    data.error_message = Some(e.to_string());
                }
            };
        }
    }
}

struct LoadedImage {
    source_image: SourceImage,
    pallet_save: Option<ImagePalletData>,
}

/// The four colors used to represent the different pallets internally in the source image
static GRAYSCALE_COLORS: [Color32; 4] = [
    Color32::from_rgb(0, 0, 0),
    Color32::from_rgb(85, 85, 85),
    Color32::from_rgb(170, 170, 170),
    Color32::from_rgb(255, 255, 255),
];

fn load_image(path: &Path) -> anyhow::Result<LoadedImage> {
    let mut file = fs::OpenOptions::new().read(true).open(path)?;

    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;

    let image = image::load_from_memory(&bytes)?;

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
                GRAYSCALE_COLORS[0]
            } else if &x == colors_sorted[1] {
                GRAYSCALE_COLORS[1]
            } else if &x == colors_sorted[2] {
                GRAYSCALE_COLORS[2]
            } else if &x == colors_sorted[3] {
                GRAYSCALE_COLORS[3]
            } else {
                unreachable!()
            }
        })
        .collect();

    let image = ColorImage {
        size: [image.width() as usize, image.height() as usize],
        pixels,
    };

    let texture = RetainedImage::from_color_image("source_image", image.clone())
        .with_texture_filter(TextureFilter::Nearest);

    let source_image = SourceImage {
        image,
        texture,
        path: path.to_owned(),
    };

    let pallet_file_path = get_pallet_file_path_for_image(&path);

    let pallet_save = if pallet_file_path.exists() {
        let file = OpenOptions::new()
            .read(true)
            .open(pallet_file_path)
            .context("Open pallet file")?;

        Some(serde_json::from_reader(&file).context("Deserialize pallet file")?)
    } else {
        None
    };

    Ok(LoadedImage {
        source_image,
        pallet_save,
    })
}

#[derive(Serialize, Deserialize)]
pub(crate) struct ImagePalletData {
    pub colors: [u8; 13],
    pub tile_pallets: Vec<u8>,
}

impl ImagePalletData {
    pub(crate) fn get_full_pallet_16(&self) -> [u32; 16] {
        [
            self.colors[0] as u32,
            self.colors[1] as u32,
            self.colors[2] as u32,
            self.colors[3] as u32,
            self.colors[0] as u32,
            self.colors[4] as u32,
            self.colors[5] as u32,
            self.colors[6] as u32,
            self.colors[0] as u32,
            self.colors[7] as u32,
            self.colors[8] as u32,
            self.colors[9] as u32,
            self.colors[0] as u32,
            self.colors[10] as u32,
            self.colors[11] as u32,
            self.colors[12] as u32,
        ]
    }
}

impl Default for ImagePalletData {
    fn default() -> Self {
        Self {
            colors: [
                // Default to simple grayscale pallet
                0x0F, 0x1D, 0x2D, 0x3D, 0x1D, 0x2D, 0x3D, 0x1D, 0x2D, 0x3D, 0x1D, 0x2D, 0x3D,
            ],
            tile_pallets: Default::default(),
        }
    }
}

fn save_project(data: &mut NesimgGui) -> anyhow::Result<()> {
    let source = if let Some(image) = &data.source_image {
        image
    } else {
        return Ok(());
    };

    let pallet_file_path = get_pallet_file_path_for_image(&source.path);

    let pallet_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(pallet_file_path)
        .context("Couldn't write to image pallet file")?;

    serde_json::to_writer_pretty(pallet_file, &data.pallet)?;

    if data.export_on_save {
        export_project(data)?;
    }

    Ok(())
}

fn get_pallet_file_path_for_image(path: &Path) -> PathBuf {
    let base_filename = path.file_name().expect("File without filename");
    path.parent().expect("File without parent").join(format!(
        "{}.pallet.json",
        base_filename.to_str().expect("Non-unicode filename")
    ))
}

fn export_project(data: &mut NesimgGui) -> anyhow::Result<()> {
    let source = if let Some(image) = &data.source_image {
        image
    } else {
        return Ok(());
    };

    let get_tile_idx = |pixel_idx: usize| {
        let image_y = pixel_idx / source.image.width();
        let image_x = pixel_idx % source.image.width();
        let tile_x = image_x / TILE_SIZE_INT[0];
        let tile_y = image_y / TILE_SIZE_INT[1];
        let tile_idx = tile_y * source.image.width() / TILE_SIZE_INT[0] + tile_x;
        tile_idx
    };

    let pixels = source
        .image
        .pixels
        .iter()
        .enumerate()
        .map(|(pixel_i, c)| {
            let tile_i = get_tile_idx(pixel_i);
            let color_i = GRAYSCALE_COLORS
                .iter()
                .enumerate()
                .find_map(|(i, x)| if c == x { Some(i) } else { None })
                .expect("Color map error");
            let pallet_i = data.pallet.tile_pallets[tile_i] as usize;
            let nes_i = data.pallet.get_full_pallet_16()[pallet_i * 4 + color_i] as usize;
            NES_PALLET_RGB[nes_i]
        })
        .map(|x| [x[0], x[1], x[2], 255])
        .flatten()
        .collect();

    let image_buffer: image::RgbaImage = image::ImageBuffer::from_vec(
        source.image.width() as _,
        source.image.height() as _,
        pixels,
    )
    .unwrap();

    let base_filename = source.path.file_name().expect("File without name");
    let export_path = source
        .path
        .parent()
        .expect("File without parent")
        .join(format!(
            "{}.export.png",
            base_filename.to_str().expect("Non-unicode filename")
        ));

    image_buffer.save(export_path).context("Save export file")?;

    Ok(())
}
