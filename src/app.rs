use eframe::{egui, epaint::textures::TextureFilter};
use egui::{Color32, ColorImage, Vec2};
use kmeans_colors::{get_kmeans_hamerly, Kmeans, Sort};
use once_cell::sync::Lazy;
use palette::{ColorDifference, FromColor, IntoColor, Lab};
use rand::Rng;
use std::{collections::HashMap, fs, io::Read, mem, sync::Arc};

use egui_extras::{RetainedImage, Size, StripBuilder};
use palette::Srgb;

use crate::components::image_viewer;

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
    target_reduced_texture: Option<RetainedImage>,
    #[serde(skip)]
    target_nes_texture: Option<RetainedImage>,
    #[serde(skip)]
    target_nes_tiled_texture: Option<RetainedImage>,
    #[serde(skip)]
    nes_pallet: Option<[Srgb<u8>; 16]>,
    #[serde(skip)]
    reduced_pallet: Option<[Srgb<u8>; 16]>,

    #[serde(skip)]
    other_colors: HashMap<&'static str, Srgb<u8>>,
    #[serde(skip)]
    orig_color_counts: Vec<(Srgb<u8>, usize)>,

    show_nes_palette: bool,

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
                println!("Got request for file load: {}", name);
                let dialog = rfd::FileDialog::new().add_filter("PNG", &["png"]);
                println!("Showing dialog");
                let file = dialog.pick_file();

                if let Some(file) = file {
                    println!("Picked file: {:?}", file);

                    println!("Opening file...");
                    let mut file = match fs::OpenOptions::new().read(true).open(file) {
                        Ok(f) => f,
                        Err(e) => {
                            println!("Error: {}", e);
                            continue;
                        }
                    };

                    let mut contents = Vec::new();
                    println!("Reading file...");
                    if let Err(e) = file.read_to_end(&mut contents) {
                        println!("Error: {}", e);
                        continue;
                    }

                    open_image_response_sender.send((name, contents)).ok();
                    println!("Image processed");
                } else {
                    println!("Warning no file picked");
                }
            }
        });

        Self {
            image_offset: Vec2::ZERO,
            image_zoom: 1.0,
            source_image: None,
            source_texture: None,
            target_reduced_texture: None,
            target_nes_texture: None,
            target_nes_tiled_texture: None,
            nes_pallet: None,
            reduced_pallet: None,
            orig_color_counts: Vec::new(),
            other_colors: HashMap::default(),
            show_nes_palette: false,
            open_image_request_sender,
            open_image_response_receiver,
        }
    }
}

impl NesimgGui {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customized the look at feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.
        cc.egui_ctx.set_visuals(egui::style::Visuals::dark());

        // Load previous app state (if any).
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }

    fn reset_images(&mut self) {
        self.source_image = None;
        self.source_texture = None;
        self.target_reduced_texture = None;
        self.target_nes_texture = None;
        self.target_nes_tiled_texture = None;
        self.nes_pallet = None;
        self.reduced_pallet = None;
        self.other_colors.clear();
        self.orig_color_counts.clear();
    }
}

impl eframe::App for NesimgGui {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Load the source image if the user has selected one
        if let Ok((name, image)) = self.open_image_response_receiver.try_recv() {
            match name {
                "source_image" => {
                    println!("Uploading image to texture");
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
            egui::menu::bar(ui, |ui| {
                if self.show_nes_palette {
                    if ui.button("Hide NES Palette").clicked() {
                        self.show_nes_palette = false;
                    }
                } else {
                    if ui.button("Show NES Palette").clicked() {
                        self.show_nes_palette = true;
                    }
                }

                egui::warn_if_debug_build(ui);
            });
        });

        egui::SidePanel::right("side_panel").show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.add_space(ui.spacing().interact_size.y);

                ui.heading("NES Palette");
                ui.separator();

                ui.horizontal_wrapped(|ui| {
                    if let Some(palette) = &mut self.nes_pallet {
                        for color in palette.iter() {
                            show_srgb(ui, color)
                        }
                    } else {
                        ui.colored_label(egui::Color32::DARK_RED, "No palette.");
                        ui.label("Convert image to generate palette.");
                    }
                });

                ui.add_space(ui.spacing().interact_size.y);

                ui.heading("Reduced Palette");
                ui.separator();

                ui.horizontal_wrapped(|ui| {
                    if let Some(palette) = &mut self.reduced_pallet {
                        for color in palette.iter() {
                            show_srgb(ui, color)
                        }
                    } else {
                        ui.colored_label(egui::Color32::DARK_RED, "No palette.");
                        ui.label("Convert image to generate palette.");
                    }
                });

                if !self.other_colors.is_empty() {
                    ui.add_space(ui.spacing().interact_size.y);

                    ui.heading("Other Colors");
                    ui.separator();

                    for (&name, color) in &self.other_colors {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(name).strong());
                            show_srgb(ui, color);
                        });
                    }
                }

                ui.add_space(ui.spacing().interact_size.y);

                ui.heading("Source Image Histogram");
                ui.separator();

                for (color, count) in &self.orig_color_counts {
                    ui.horizontal(|ui| {
                        show_srgb(ui, &color);
                        ui.label(format!("{}", count))
                            .on_hover_text("Pixel count with this color");
                    });
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            StripBuilder::new(ui)
                .size(Size::relative(0.5))
                .size(Size::relative(0.5))
                .horizontal(|mut strip| {
                    strip.strip(|builder| {
                        builder.sizes(Size::remainder(), 2).vertical(|mut strip| {
                            strip.cell(|ui| {
                                ui.horizontal(|ui| {
                                    if self.source_texture.is_some() {
                                        // Unload source file button
                                        if ui.button("🗙").clicked() {
                                            self.reset_images();
                                        }
                                    }
                                    ui.heading("Source Image");
                                });
                                egui::Frame::canvas(ui.style()).show(ui, |ui| {
                                    ui.set_min_size(ui.available_size());

                                    if let Some(image) = &mut self.source_texture {
                                        ui.add(image_viewer(
                                            image,
                                            &mut self.image_zoom,
                                            &mut self.image_offset,
                                        ));
                                    } else {
                                        ui.vertical_centered(|ui| {
                                            ui.add_space(20.0);
                                            if ui.button("Select Source File...").clicked() {
                                                self.open_image_request_sender
                                                    .send("source_image")
                                                    .expect("Open file");
                                            }
                                        });
                                    }
                                });
                            });

                            strip.cell(|ui| {
                                ui.heading("Reduced Pallet");
                                egui::Frame::canvas(ui.style()).show(ui, |ui| {
                                    ui.set_min_size(ui.available_size());

                                    if let Some(image) = &mut self.target_reduced_texture {
                                        ui.add(image_viewer(
                                            image,
                                            &mut self.image_zoom,
                                            &mut self.image_offset,
                                        ));
                                    } else {
                                        ui.vertical_centered(|ui| {
                                            ui.add_space(20.0);

                                            if self.source_texture.is_none() {
                                                ui.colored_label(
                                                    Color32::DARK_RED,
                                                    "No source file.",
                                                );
                                                ui.label("Load source file to convert");
                                            } else {
                                                if ui.button("Convert...").clicked() {
                                                    convert(self, ui, _frame);
                                                }
                                            }
                                        });
                                    }
                                });
                            });
                        });
                    });

                    strip.strip(|builder| {
                        builder.sizes(Size::remainder(), 2).vertical(|mut strip| {
                            strip.cell(|ui| {
                                ui.heading("NES");
                                egui::Frame::canvas(ui.style()).show(ui, |ui| {
                                    ui.set_min_size(ui.available_size());

                                    if let Some(image) = &mut self.target_nes_tiled_texture {
                                        ui.add(image_viewer(
                                            image,
                                            &mut self.image_zoom,
                                            &mut self.image_offset,
                                        ));
                                    } else {
                                        ui.vertical_centered(|ui| {
                                            ui.add_space(20.0);

                                            if self.source_texture.is_none() {
                                                ui.colored_label(
                                                    Color32::DARK_RED,
                                                    "No source file.",
                                                );
                                                ui.label("Load source file to convert");
                                            } else {
                                                if ui.button("Convert...").clicked() {
                                                    convert(self, ui, _frame);
                                                }
                                            }
                                        });
                                    }
                                });
                            });

                            strip.cell(|ui| {
                                ui.heading("NES ( Without Tile Enforcement )");
                                egui::Frame::canvas(ui.style()).show(ui, |ui| {
                                    ui.set_min_size(ui.available_size());

                                    if let Some(image) = &mut self.target_nes_texture {
                                        ui.add(image_viewer(
                                            image,
                                            &mut self.image_zoom,
                                            &mut self.image_offset,
                                        ));
                                    } else {
                                        ui.vertical_centered(|ui| {
                                            ui.add_space(20.0);

                                            if self.source_texture.is_none() {
                                                ui.colored_label(
                                                    Color32::DARK_RED,
                                                    "No source file.",
                                                );
                                                ui.label("Load source file to convert");
                                            } else {
                                                if ui.button("Convert...").clicked() {
                                                    convert(self, ui, _frame);
                                                }
                                            }
                                        });
                                    }
                                });
                            });
                        });
                    });
                });
        });

        egui::Window::new("Full NES Palette")
            .open(&mut self.show_nes_palette)
            .default_width(790.0)
            .show(ctx, |ui| {
                ui.horizontal_wrapped(|ui| {
                    for color in NES_PALETTE_RGB.iter() {
                        show_srgb(ui, color)
                    }
                });
            });
    }
}

fn show_srgb(ui: &mut egui::Ui, color: &Srgb<u8>) {
    egui::widgets::color_picker::show_color(
        ui,
        egui::color::Rgba::from_srgba_unmultiplied(color.red, color.green, color.blue, 255),
        ui.spacing().interact_size,
    )
    .on_hover_ui(|ui| {
        if let Some(idx) =
            NES_PALETTE_RGB.iter().enumerate().find_map(
                |(i, x)| {
                    if x == color {
                        Some(i)
                    } else {
                        None
                    }
                },
            )
        {
            ui.label(format!("NES Palette Index: ${:02X}", idx));
        } else {
            ui.colored_label(Color32::DARK_RED, "Not an NES Color");
        }
        ui.label(format!(
            "srgb: ({}, {}, {})",
            color.red, color.green, color.blue
        ));
        ui.label(format!(
            "srgb: #{:02X}{:02X}{:02X}",
            color.red, color.green, color.blue
        ));
    });
}

fn convert(data: &mut NesimgGui, _ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
    let source_image = if let Some(image) = &mut data.source_image {
        image
    } else {
        return;
    };

    // Convert image pixels to Lab encoding
    let pixels = source_image
        .pixels
        .iter()
        .map(|x| Srgb::new(x.r(), x.g(), x.b()))
        .map(|x| x.into_format::<f32>())
        .map(|x| x.into_linear().into_color())
        .collect::<Vec<Lab>>();

    // Loop through k-means algorithm a few times with a different seed and keep the best result
    let mut kmeans = Kmeans::new();
    for i in 0..5 {
        let r = get_kmeans_hamerly(13, 20, 5.0, false, &pixels, i ^ 42);
        if r.score < kmeans.score {
            kmeans = r;
        }
    }

    // Identify the background color
    let sorted = Lab::sort_indexed_colors(&kmeans.centroids, &kmeans.indices);
    let background_color = Lab::get_dominant_color(&sorted).expect("No dominant color");

    // Collect the other colors into a list of non-background colors
    let mut non_background_reduced_colors: [Lab; 12] = Default::default();
    let mut i = 0;
    for color in kmeans.centroids {
        if color != background_color {
            non_background_reduced_colors[i] = color;
            i += 1;
        }
    }

    // Set the reduced pallet in the UI
    data.reduced_pallet = Some([
        Srgb::from_color(background_color).into_format(),
        Srgb::from_color(non_background_reduced_colors[0]).into_format(),
        Srgb::from_color(non_background_reduced_colors[1]).into_format(),
        Srgb::from_color(non_background_reduced_colors[2]).into_format(),
        Srgb::from_color(background_color).into_format(),
        Srgb::from_color(non_background_reduced_colors[3]).into_format(),
        Srgb::from_color(non_background_reduced_colors[4]).into_format(),
        Srgb::from_color(non_background_reduced_colors[5]).into_format(),
        Srgb::from_color(background_color).into_format(),
        Srgb::from_color(non_background_reduced_colors[6]).into_format(),
        Srgb::from_color(non_background_reduced_colors[7]).into_format(),
        Srgb::from_color(non_background_reduced_colors[8]).into_format(),
        Srgb::from_color(background_color).into_format(),
        Srgb::from_color(non_background_reduced_colors[9]).into_format(),
        Srgb::from_color(non_background_reduced_colors[10]).into_format(),
        Srgb::from_color(non_background_reduced_colors[11]).into_format(),
    ]);

    // Create the list of unique colors in the reduced pallet
    let reduced_pallet_unique = [
        background_color,
        non_background_reduced_colors[0],
        non_background_reduced_colors[1],
        non_background_reduced_colors[2],
        non_background_reduced_colors[3],
        non_background_reduced_colors[4],
        non_background_reduced_colors[5],
        non_background_reduced_colors[6],
        non_background_reduced_colors[7],
        non_background_reduced_colors[8],
        non_background_reduced_colors[9],
        non_background_reduced_colors[10],
        non_background_reduced_colors[11],
    ];

    // Convert the image pixels to use only the colors in the reduced pallet
    let target_reduced_pixels = pixels
        .iter()
        .map(|x| find_closest_in_pallet(*x, reduced_pallet_unique))
        .collect::<Vec<_>>();
    // Convert reduced pallet pixels to srgba
    let target_reduced_pixels_color32 = target_reduced_pixels
        .iter()
        .map(|x| Srgb::from_color(*x).into_format::<u8>())
        .map(|x| Color32::from_rgb(x.red, x.green, x.blue))
        .collect::<Vec<_>>();
    // Create image from pixels
    let target_reduced_image = egui::ColorImage {
        size: source_image.size,
        pixels: target_reduced_pixels_color32,
    };
    // Upload to GPU
    let target_reduced_texture = RetainedImage::from_color_image(
        "target_imge",
        target_reduced_image,
        TextureFilter::Nearest,
    );
    // Set in UI
    data.target_reduced_texture = Some(target_reduced_texture);

    // Count and sort source image colors
    let mut color_counts = Vec::<(Lab, usize)>::new();
    for pixel in &pixels {
        let mut found_color = false;
        for (color, count) in &mut color_counts {
            if color == pixel {
                *count += 1;
                found_color = true;
                break;
            }
        }
        if !found_color {
            color_counts.push((pixel.clone(), 1));
        }
    }
    color_counts.sort_by_key(|x| x.1);
    color_counts.reverse();

    // Set the histogram in the UI
    data.orig_color_counts = color_counts
        .iter()
        .map(|x| (Srgb::from_linear(x.0.into_color()).into_format(), x.1))
        .collect();

    //
    let background_color_nes = find_closest_nes(background_color);
    let mut non_background_nes_colors: [Lab; 12] = Default::default();
    for (i, color) in non_background_reduced_colors.iter().enumerate() {
        non_background_nes_colors[i] = find_closest_nes(*color);
    }

    // The set of unique NES colors
    let nes_palette_unique = [
        background_color_nes,
        non_background_nes_colors[0],
        non_background_nes_colors[1],
        non_background_nes_colors[2],
        non_background_nes_colors[3],
        non_background_nes_colors[4],
        non_background_nes_colors[5],
        non_background_nes_colors[6],
        non_background_nes_colors[7],
        non_background_nes_colors[8],
        non_background_nes_colors[9],
        non_background_nes_colors[10],
        non_background_nes_colors[11],
    ];

    // Convert reduced_pallet pixels to their closest NES equivalents
    let target_nes_pixels = target_reduced_pixels
        .iter()
        .map(|x| find_closest_in_pallet(*x, nes_palette_unique))
        .map(|x| Srgb::from_color(x).into_format::<u8>())
        .map(|x| Color32::from_rgb(x.red, x.green, x.blue))
        .collect::<Vec<_>>();
    // Create image from pixels
    let target_nes_image = egui::ColorImage {
        size: source_image.size,
        pixels: target_nes_pixels,
    };
    // Upload to texture
    let target_nes_texture =
        RetainedImage::from_color_image("target_imge", target_nes_image, TextureFilter::Nearest);
    // Set in UI
    data.target_nes_texture = Some(target_nes_texture);

    // Collect reduced pixels into 8x8 tiles
    let mut tiles: Vec<[Lab; 64]> = Vec::new();
    assert_eq!(
        source_image.width() % 8,
        0,
        "Image width must be a multiple of 8"
    );
    assert_eq!(
        source_image.height() % 8,
        0,
        "Image height must be a multiple of 8"
    );
    for y in 0..(source_image.height() / 8) {
        for x in 0..(source_image.width() / 8) {
            let mut tile: [Lab; 64] = [Lab::default(); 64];
            let mut i = 0;
            for row in 0..8 {
                for col in 0..8 {
                    tile[i] =
                        target_reduced_pixels[(y * 8 + row) * source_image.width() + x * 8 + col];

                    i += 1;
                }
            }
            tiles.push(tile);
        }
    }

    // // Make sure tiles are being extracted right by dumping them to a directory
    // for (i, tile) in tiles.iter().enumerate() {
    //     use palette::Pixel;
    //     let mut srgb_tile = Vec::<u8>::new();

    //     for pixel in tile {
    //         srgb_tile.extend(
    //             Srgb::<f32>::from_color(*pixel)
    //                 .into_format::<u8>()
    //                 .into_raw::<[u8; 3]>()
    //                 .iter(),
    //         );
    //     }
    //     let img = image::RgbImage::from_raw(8, 8, srgb_tile).expect("Create image");
    //     img.save(format!("./gitignore/tile_{}.png", i))
    //         .expect("Save test tile");
    // }

    // Loop through the tiles and record how often each color in the reduced pallet appears with the
    // other colors in the pallet

    // A vector of tiles and for each tile a vector of non-background reduced tile color indices
    let mut tile_colors = Vec::<Vec<usize>>::new();
    for tile in &tiles {
        let mut colors = Vec::new();
        for pixel in tile {
            for (i, color) in non_background_reduced_colors.iter().enumerate() {
                if color == pixel && !colors.contains(&i) {
                    colors.push(i);
                }
            }
        }
        tile_colors.push(colors);
    }

    let mut color_adjacent_count = HashMap::<(usize, usize), usize>::new();
    for color1 in 0..non_background_reduced_colors.len() {
        for tile in &tile_colors {
            if !tile.contains(&color1) {
                continue;
            }

            for color2 in 0..non_background_reduced_colors.len() {
                if color2 == color1 {
                    continue;
                }

                if tile.contains(&color2) {
                    let key = if color1 > color2 {
                        (color1, color2)
                    } else {
                        (color2, color1)
                    };
                    *color_adjacent_count.entry(key).or_default() += 1;
                }
            }
        }
    }

    let mut available_colors = Vec::new();
    for color in 0..non_background_reduced_colors.len() {
        available_colors.push(color);
    }
    let mut pallets: [[usize; 3]; 4] = Default::default();

    let get_pallet_color_dist = |color: usize, other: usize| {
        let key = if color > other {
            (color, other)
        } else {
            (other, color)
        };
        match color_adjacent_count.get(&key) {
            Some(count) => 1.0 / *count as f32,
            None => f32::INFINITY,
        }
    };

    for pallet in &mut pallets {
        pallet[0] = available_colors.pop().unwrap();

        for i in 1..=2 {
            let mut closest_accompanying = available_colors.pop().unwrap();
            let mut closest_accompanying_diff =
                get_pallet_color_dist(closest_accompanying, pallet[0]);

            for color in &mut available_colors {
                let diff = get_pallet_color_dist(*color, pallet[0]);
                if diff < closest_accompanying_diff {
                    mem::swap(color, &mut closest_accompanying);
                    closest_accompanying_diff = diff;
                }
            }

            pallet[i] = closest_accompanying;
        }
    }

    let sorted_pallet = [
        background_color,
        non_background_reduced_colors[pallets[0][0]],
        non_background_reduced_colors[pallets[0][1]],
        non_background_reduced_colors[pallets[0][2]],
        background_color,
        non_background_reduced_colors[pallets[1][0]],
        non_background_reduced_colors[pallets[1][1]],
        non_background_reduced_colors[pallets[1][2]],
        background_color,
        non_background_reduced_colors[pallets[2][0]],
        non_background_reduced_colors[pallets[2][1]],
        non_background_reduced_colors[pallets[2][2]],
        background_color,
        non_background_reduced_colors[pallets[3][0]],
        non_background_reduced_colors[pallets[3][1]],
        non_background_reduced_colors[pallets[3][2]],
    ];

    // Set the reduced pallet in the UI
    data.reduced_pallet = Some([
        Srgb::from_color(sorted_pallet[0]).into_format(),
        Srgb::from_color(sorted_pallet[1]).into_format(),
        Srgb::from_color(sorted_pallet[2]).into_format(),
        Srgb::from_color(sorted_pallet[3]).into_format(),
        Srgb::from_color(sorted_pallet[4]).into_format(),
        Srgb::from_color(sorted_pallet[5]).into_format(),
        Srgb::from_color(sorted_pallet[6]).into_format(),
        Srgb::from_color(sorted_pallet[7]).into_format(),
        Srgb::from_color(sorted_pallet[8]).into_format(),
        Srgb::from_color(sorted_pallet[9]).into_format(),
        Srgb::from_color(sorted_pallet[10]).into_format(),
        Srgb::from_color(sorted_pallet[11]).into_format(),
        Srgb::from_color(sorted_pallet[12]).into_format(),
        Srgb::from_color(sorted_pallet[13]).into_format(),
        Srgb::from_color(sorted_pallet[14]).into_format(),
        Srgb::from_color(sorted_pallet[15]).into_format(),
    ]);

    let sorted_pallet_nes = [
        find_closest_nes(sorted_pallet[0]),
        find_closest_nes(sorted_pallet[1]),
        find_closest_nes(sorted_pallet[2]),
        find_closest_nes(sorted_pallet[3]),
        find_closest_nes(sorted_pallet[4]),
        find_closest_nes(sorted_pallet[5]),
        find_closest_nes(sorted_pallet[6]),
        find_closest_nes(sorted_pallet[7]),
        find_closest_nes(sorted_pallet[8]),
        find_closest_nes(sorted_pallet[9]),
        find_closest_nes(sorted_pallet[10]),
        find_closest_nes(sorted_pallet[11]),
        find_closest_nes(sorted_pallet[12]),
        find_closest_nes(sorted_pallet[13]),
        find_closest_nes(sorted_pallet[14]),
        find_closest_nes(sorted_pallet[15]),
    ];

    // Set the NES pallet in the UI
    data.nes_pallet = Some([
        Srgb::from_color(sorted_pallet_nes[0]).into_format(),
        Srgb::from_color(sorted_pallet_nes[1]).into_format(),
        Srgb::from_color(sorted_pallet_nes[2]).into_format(),
        Srgb::from_color(sorted_pallet_nes[3]).into_format(),
        Srgb::from_color(sorted_pallet_nes[4]).into_format(),
        Srgb::from_color(sorted_pallet_nes[5]).into_format(),
        Srgb::from_color(sorted_pallet_nes[6]).into_format(),
        Srgb::from_color(sorted_pallet_nes[7]).into_format(),
        Srgb::from_color(sorted_pallet_nes[8]).into_format(),
        Srgb::from_color(sorted_pallet_nes[9]).into_format(),
        Srgb::from_color(sorted_pallet_nes[10]).into_format(),
        Srgb::from_color(sorted_pallet_nes[11]).into_format(),
        Srgb::from_color(sorted_pallet_nes[12]).into_format(),
        Srgb::from_color(sorted_pallet_nes[13]).into_format(),
        Srgb::from_color(sorted_pallet_nes[14]).into_format(),
        Srgb::from_color(sorted_pallet_nes[15]).into_format(),
    ]);

    let reduced_tile_pallets = [
        [
            sorted_pallet[0],
            sorted_pallet[1],
            sorted_pallet[2],
            sorted_pallet[3],
        ],
        [
            sorted_pallet[4],
            sorted_pallet[5],
            sorted_pallet[6],
            sorted_pallet[7],
        ],
        [
            sorted_pallet[8],
            sorted_pallet[9],
            sorted_pallet[10],
            sorted_pallet[11],
        ],
        [
            sorted_pallet[12],
            sorted_pallet[13],
            sorted_pallet[14],
            sorted_pallet[15],
        ],
    ];
    let nes_tile_pallets = [
        [
            sorted_pallet_nes[0],
            sorted_pallet_nes[1],
            sorted_pallet_nes[2],
            sorted_pallet_nes[3],
        ],
        [
            sorted_pallet_nes[4],
            sorted_pallet_nes[5],
            sorted_pallet_nes[6],
            sorted_pallet_nes[7],
        ],
        [
            sorted_pallet_nes[8],
            sorted_pallet_nes[9],
            sorted_pallet_nes[10],
            sorted_pallet_nes[11],
        ],
        [
            sorted_pallet_nes[12],
            sorted_pallet_nes[13],
            sorted_pallet_nes[14],
            sorted_pallet_nes[15],
        ],
    ];

    let mut nes_final_pixels = Vec::new();

    for tile in &mut tiles {
        let get_pallet_dist = |tile: &[Lab; 64], pallet: [Lab; 4]| {
            let mut dist = 4;
            for pixel in tile {
                if pallet.contains(pixel) {
                    dist -= 1;
                }
            }

            dist
        };

        let mut best_pallet_idx = 0;
        let mut best_pallet_dist = get_pallet_dist(tile, reduced_tile_pallets[0]);

        for i in 1..4 {
            let dist = get_pallet_dist(tile, reduced_tile_pallets[i]);
            if dist < best_pallet_dist {
                best_pallet_dist = dist;
                best_pallet_idx = i;
            }
        }

        for pixel in tile {
            *pixel = find_closest_in_pallet(*pixel, nes_tile_pallets[best_pallet_idx]);
        }
    }

    let tile_height = source_image.height() / 8;
    let tile_width = source_image.width() / 8;
    for row in 0..tile_height {
        for y in 0..8 {
            for col in 0..tile_width {
                for x in 0..8 {
                    nes_final_pixels.push(tiles[row * tile_width + col][y * 8 + x]);
                }
            }
        }
    }

    // Convert reduced pallet pixels to srgba
    let nes_pixels_color32 = nes_final_pixels
        .iter()
        .map(|x| Srgb::from_color(*x).into_format::<u8>())
        .map(|x| Color32::from_rgb(x.red, x.green, x.blue))
        .collect::<Vec<_>>();
    // Create image from pixels
    let target_reduced_image = egui::ColorImage {
        size: source_image.size,
        pixels: nes_pixels_color32,
    };
    // Upload to GPU
    let nes_texture = RetainedImage::from_color_image(
        "target_imge",
        target_reduced_image,
        TextureFilter::Nearest,
    );
    // Set in UI
    data.target_nes_tiled_texture = Some(nes_texture);
}

fn find_closest_nes(color: Lab) -> Lab {
    find_closest_in_pallet(color, *NES_PALETTE_LAB)
}

fn find_closest_in_pallet<'a, I: IntoIterator<Item = Lab>>(color: Lab, pallet: I) -> Lab {
    let mut iter = pallet.into_iter();
    let mut closest_color = iter.next().unwrap();
    let mut diff = color.get_color_difference(&closest_color);

    for other_color in iter {
        let next_diff = color.get_color_difference(&other_color);
        if next_diff < diff {
            diff = next_diff;
            closest_color = other_color.clone();
        }
    }

    closest_color
}

static NES_PALETTE_RGB: Lazy<[Srgb<u8>; 64]> = Lazy::new(|| {
    [
        // 00
        Srgb::new(128, 128, 128),
        Srgb::new(0, 61, 166),
        Srgb::new(0, 18, 176),
        Srgb::new(68, 0, 150),
        Srgb::new(161, 0, 94),
        Srgb::new(199, 0, 40),
        Srgb::new(186, 6, 0),
        Srgb::new(140, 23, 0),
        Srgb::new(92, 47, 0),
        Srgb::new(16, 69, 0),
        Srgb::new(5, 74, 0),
        Srgb::new(0, 71, 46),
        Srgb::new(0, 65, 102),
        Srgb::new(3, 3, 3),
        Srgb::new(3, 3, 3),
        Srgb::new(3, 3, 3),
        // 10
        Srgb::new(199, 199, 199),
        Srgb::new(0, 119, 255),
        Srgb::new(33, 85, 255),
        Srgb::new(130, 55, 250),
        Srgb::new(235, 47, 181),
        Srgb::new(255, 41, 80),
        Srgb::new(255, 34, 0),
        Srgb::new(214, 50, 0),
        Srgb::new(196, 98, 0),
        Srgb::new(53, 128, 0),
        Srgb::new(5, 143, 0),
        Srgb::new(0, 138, 85),
        Srgb::new(0, 153, 204),
        Srgb::new(33, 33, 33),
        Srgb::new(3, 3, 3),
        Srgb::new(3, 3, 3),
        // 20
        Srgb::new(255, 255, 255),
        Srgb::new(15, 215, 255),
        Srgb::new(105, 162, 255),
        Srgb::new(212, 128, 255),
        Srgb::new(255, 69, 243),
        Srgb::new(255, 97, 139),
        Srgb::new(255, 136, 51),
        Srgb::new(255, 156, 18),
        Srgb::new(250, 188, 32),
        Srgb::new(159, 227, 14),
        Srgb::new(43, 240, 53),
        Srgb::new(12, 240, 164),
        Srgb::new(5, 251, 255),
        Srgb::new(94, 94, 94),
        Srgb::new(13, 13, 13),
        Srgb::new(13, 13, 13),
        // 30
        Srgb::new(255, 255, 255),
        Srgb::new(166, 252, 255),
        Srgb::new(179, 236, 255),
        Srgb::new(218, 171, 235),
        Srgb::new(255, 168, 249),
        Srgb::new(255, 171, 179),
        Srgb::new(255, 210, 176),
        Srgb::new(255, 239, 166),
        Srgb::new(255, 247, 156),
        Srgb::new(215, 232, 149),
        Srgb::new(166, 237, 157),
        Srgb::new(162, 242, 218),
        Srgb::new(153, 255, 252),
        Srgb::new(221, 221, 221),
        Srgb::new(17, 17, 17),
        Srgb::new(17, 17, 17),
    ]
});

static NES_PALETTE_LAB: Lazy<[Lab; 64]> = Lazy::new(|| {
    let mut colors = [Lab::default(); 64];
    for (i, color) in NES_PALETTE_RGB.iter().enumerate() {
        colors[i] = color.into_format().into_color();
    }

    colors
});
