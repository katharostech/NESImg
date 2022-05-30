use std::{collections::HashSet, io::Read, path::Path, sync::Arc};

use egui::Color32;
use egui_extras::RetainedImage;
use image::GenericImageView;
use native_dialog::FileDialog;
use notify::Watcher;
use watch::WatchReceiver;

use super::project_state::{SourceImageData, SourceImageStatus};

/// Ask the user to pick a file, and then optionally watch it for changes
pub fn pick_file<F, R>(filters: &'static [FileFilter], load_fn: F) -> WatchReceiver<R>
where
    F: Fn(&Path) -> R + Sync + Send + 'static,
    R: Default + Clone + Sync + Send + 'static,
{
    let (sender, receiver) = watch::channel(R::default());

    std::thread::spawn(move || {
        let mut dialog = FileDialog::new();

        for filter in filters {
            dialog = dialog.add_filter(filter.name, filter.extensions);
        }

        let path = if let Some(path) = dialog.show_open_single_file().expect("Show file dialog") {
            path
        } else {
            return;
        };

        let data = load_fn(&path);
        sender.send(data);
    });

    receiver
}

pub struct FileFilter {
    pub name: &'static str,
    pub extensions: &'static [&'static str],
}

/// The four colors used to represent the different pallets internally in the source image
static GRAYSCALE_COLORS: [Color32; 4] = [
    Color32::from_rgb(0, 0, 0),
    Color32::from_rgb(85, 85, 85),
    Color32::from_rgb(170, 170, 170),
    Color32::from_rgb(255, 255, 255),
];

/// Load an image and watch for changes
pub fn load_and_watch_image(path: &Path) -> WatchReceiver<SourceImageStatus> {
    let path = path.to_owned();
    let (sender, receiver) = watch::channel(SourceImageStatus::Loading);

    std::thread::spawn(move || {
        let load_texture = || -> anyhow::Result<_> {
            let mut file = std::fs::OpenOptions::new().read(true).open(&path)?;
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

            // Sort colors by brightness ( or luminocity, I'm not sure what the difference is )
            let mut colors_sorted = colors.iter().collect::<Vec<_>>();
            colors_sorted.sort_unstable_by(|x, y| {
                let x = x[0] as u16 + x[1] as u16 + x[2] as u16;
                let y = y[0] as u16 + y[1] as u16 + y[2] as u16;
                x.cmp(&y)
            });

            let (pixels, indexes) = image
                .pixels()
                .map(|(_, _, x)| {
                    if &x == colors_sorted[0] {
                        (GRAYSCALE_COLORS[0], 0)
                    } else if &x == colors_sorted[1] {
                        (GRAYSCALE_COLORS[1], 1)
                    } else if &x == colors_sorted[2] {
                        (GRAYSCALE_COLORS[2], 2)
                    } else if &x == colors_sorted[3] {
                        (GRAYSCALE_COLORS[3], 3)
                    } else {
                        unreachable!()
                    }
                })
                .unzip();

            let image = egui::ColorImage {
                size: [image.width() as usize, image.height() as usize],
                pixels,
            };

            let texture = RetainedImage::from_color_image("source_image", image.clone())
                .with_texture_filter(egui::TextureFilter::Nearest);

            Ok(SourceImageData {
                texture: Arc::new(texture),
                indexes,
            })
        };

        match load_texture() {
            Ok(image) => sender.send(SourceImageStatus::Found(image)),
            Err(e) => sender.send(SourceImageStatus::Error(e.to_string())),
        }

        let (watch_sender, watch_receiver) = std::sync::mpsc::channel();
        let mut watcher =
            notify::watcher(watch_sender, std::time::Duration::from_secs(1)).expect("Watch file");

        if let Err(e) = watcher.watch(&path, notify::RecursiveMode::NonRecursive) {
            sender.send(SourceImageStatus::Error(e.to_string()));
            return;
        }

        // TODO: Clean up file watcher once all receivers have been dropped:
        //       https://github.com/Darksonn/watch/issues/3

        while let Ok(event) = watch_receiver.recv() {
            if let notify::DebouncedEvent::Write(_) = event {
                match load_texture() {
                    Ok(image) => sender.send(SourceImageStatus::Found(image)),
                    Err(e) => sender.send(SourceImageStatus::Error(e.to_string())),
                }
            }
        }
    });

    receiver
}
