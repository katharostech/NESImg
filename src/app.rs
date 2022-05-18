use eframe::{egui, epaint::textures::TextureFilter};
use egui::{Color32, ColorImage, Vec2};
use once_cell::sync::Lazy;
use palette::{ColorDifference, FromColor, IntoColor, Lab};
use std::{collections::HashMap, fs, io::Read};

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

                ui.add_space(ui.spacing().interact_size.y);

                ui.heading("Other Colors");
                ui.separator();

                if self.other_colors.is_empty() {
                    ui.label("No other colors.");
                } else {
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

    // Compute the average color
    let l = pixels.iter().map(|x| x.l).sum::<f32>() / pixels.len() as f32;
    let a = pixels.iter().map(|x| x.a).sum::<f32>() / pixels.len() as f32;
    let b = pixels.iter().map(|x| x.b).sum::<f32>() / pixels.len() as f32;
    let average_color = Lab::new(l, a, b);
    let average_srgb: Srgb = average_color.into_color();
    data.other_colors
        .insert("Average", average_srgb.into_format());

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

    // The background color is the most common color
    let background_color = color_counts.first().unwrap().0;

    // Start creating the reduced pallet
    let mut reduced_pallet = [Default::default(); 12];
    let mut pallet_diffs = [0.0; 12];
    let mut lowest_diff = (0, f32::NEG_INFINITY);

    // Creat the initial pallet by grabbing the most-used colors from the image
    for (i, color) in color_counts
        .iter()
        .skip(1)
        .map(|x| x.0)
        .enumerate()
        .take(12)
    {
        reduced_pallet[i] = color;
    }

    // // Make sure tiles are being extracted right
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

    // Helper to get the sum of the difference that one color has from all the other colors in the
    // pallet
    let get_pallet_diff = |pallet: &[Lab; 12], color: &Lab| {
        let mut diff = 0.0;
        for c in pallet {
            diff += color.get_color_difference(c);
        }
        diff
    };
    // Helper to update the pallet diff of each color in the pallet, and take note of the least
    // different color in the pallet
    let calculate_pallet_diffs =
        |pallet: &mut [Lab; 12], diffs: &mut [f32; 12], lowest_diff: &mut (usize, f32)| {
            for (i, c) in pallet.iter().enumerate() {
                let diff = get_pallet_diff(pallet, c);

                diffs[i] = diff;
                if diff < lowest_diff.1 {
                    *lowest_diff = (i, diff);
                }
            }
        };

    calculate_pallet_diffs(&mut reduced_pallet, &mut pallet_diffs, &mut lowest_diff);

    for pixel in color_counts.iter().map(|x| x.0) {
        let diff = get_pallet_diff(&reduced_pallet, &pixel);
        if diff > lowest_diff.1 {
            reduced_pallet[lowest_diff.0] = pixel;
        }

        calculate_pallet_diffs(&mut reduced_pallet, &mut pallet_diffs, &mut lowest_diff);
    }

    // Set the reduced pallet
    data.reduced_pallet = Some([
        Srgb::from_color(background_color).into_format(),
        Srgb::from_color(reduced_pallet[0]).into_format(),
        Srgb::from_color(reduced_pallet[1]).into_format(),
        Srgb::from_color(reduced_pallet[2]).into_format(),
        Srgb::from_color(background_color).into_format(),
        Srgb::from_color(reduced_pallet[3]).into_format(),
        Srgb::from_color(reduced_pallet[4]).into_format(),
        Srgb::from_color(reduced_pallet[5]).into_format(),
        Srgb::from_color(background_color).into_format(),
        Srgb::from_color(reduced_pallet[6]).into_format(),
        Srgb::from_color(reduced_pallet[7]).into_format(),
        Srgb::from_color(reduced_pallet[8]).into_format(),
        Srgb::from_color(background_color).into_format(),
        Srgb::from_color(reduced_pallet[9]).into_format(),
        Srgb::from_color(reduced_pallet[10]).into_format(),
        Srgb::from_color(reduced_pallet[11]).into_format(),
    ]);

    let mut nes_reduced_pallet: [Lab; 12] = Default::default();
    for (i, color) in reduced_pallet.iter().enumerate() {
        nes_reduced_pallet[i] = find_closest_nes(*color);
    }

    // Set the NES pallet
    data.nes_pallet = Some([
        Srgb::from_color(find_closest_nes(background_color)).into_format(),
        Srgb::from_color(nes_reduced_pallet[0]).into_format(),
        Srgb::from_color(nes_reduced_pallet[1]).into_format(),
        Srgb::from_color(nes_reduced_pallet[2]).into_format(),
        Srgb::from_color(find_closest_nes(background_color)).into_format(),
        Srgb::from_color(nes_reduced_pallet[3]).into_format(),
        Srgb::from_color(nes_reduced_pallet[4]).into_format(),
        Srgb::from_color(nes_reduced_pallet[5]).into_format(),
        Srgb::from_color(find_closest_nes(background_color)).into_format(),
        Srgb::from_color(nes_reduced_pallet[6]).into_format(),
        Srgb::from_color(nes_reduced_pallet[7]).into_format(),
        Srgb::from_color(nes_reduced_pallet[8]).into_format(),
        Srgb::from_color(find_closest_nes(background_color)).into_format(),
        Srgb::from_color(nes_reduced_pallet[9]).into_format(),
        Srgb::from_color(nes_reduced_pallet[10]).into_format(),
        Srgb::from_color(nes_reduced_pallet[11]).into_format(),
    ]);

    let target_reduced_palette = [
        background_color,
        reduced_pallet[0],
        reduced_pallet[1],
        reduced_pallet[2],
        reduced_pallet[3],
        reduced_pallet[4],
        reduced_pallet[5],
        reduced_pallet[6],
        reduced_pallet[7],
        reduced_pallet[8],
        reduced_pallet[9],
        reduced_pallet[10],
        reduced_pallet[11],
    ];

    let target_nes_palette = [
        background_color,
        nes_reduced_pallet[0],
        nes_reduced_pallet[1],
        nes_reduced_pallet[2],
        nes_reduced_pallet[3],
        nes_reduced_pallet[4],
        nes_reduced_pallet[5],
        nes_reduced_pallet[6],
        nes_reduced_pallet[7],
        nes_reduced_pallet[8],
        nes_reduced_pallet[9],
        nes_reduced_pallet[10],
        nes_reduced_pallet[11],
    ];

    let target_reduced_pixels = pixels
        .iter()
        .map(|x| find_closest_in_pallet(*x, target_reduced_palette))
        .collect::<Vec<_>>();
    let target_reduced_pixels_color32 = target_reduced_pixels
        .iter()
        .map(|x| Srgb::from_color(*x).into_format::<u8>())
        .map(|x| Color32::from_rgb(x.red, x.green, x.blue))
        .collect::<Vec<_>>();

    let target_reduced_image = egui::ColorImage {
        size: source_image.size,
        pixels: target_reduced_pixels_color32,
    };
    let target_reduced_texture = RetainedImage::from_color_image(
        "target_imge",
        target_reduced_image,
        TextureFilter::Nearest,
    );
    data.target_reduced_texture = Some(target_reduced_texture);

    let target_nes_pixels = target_reduced_pixels
        .iter()
        .map(|x| find_closest_in_pallet(*x, target_nes_palette))
        .map(|x| Srgb::from_color(x).into_format::<u8>())
        .map(|x| Color32::from_rgb(x.red, x.green, x.blue))
        .collect::<Vec<_>>();

    let target_reduced_image = egui::ColorImage {
        size: source_image.size,
        pixels: target_nes_pixels,
    };
    let target_nes_texture = RetainedImage::from_color_image(
        "target_imge",
        target_reduced_image,
        TextureFilter::Nearest,
    );
    data.target_nes_texture = Some(target_nes_texture);
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
