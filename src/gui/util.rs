use std::{io::Read, path::Path, sync::Arc};

use egui_extras::RetainedImage;
use native_dialog::FileDialog;
use notify::Watcher;
use watch::WatchReceiver;

use super::project_state::SourceTextureStatus;

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

/// Load an image and watch for changes
pub fn load_and_watch_image(path: &Path) -> WatchReceiver<SourceTextureStatus> {
    let path = path.to_owned();
    let (sender, receiver) = watch::channel(SourceTextureStatus::Loading);
    let error_sender = sender.clone();

    std::thread::spawn(move || {
        let inner = move || -> anyhow::Result<()> {
            let load_image = || -> anyhow::Result<_> {
                let mut file = std::fs::OpenOptions::new().read(true).open(&path)?;
                let mut bytes = Vec::new();
                file.read_to_end(&mut bytes)?;

                let image = RetainedImage::from_image_bytes("a_source_image", &bytes)
                    .map(|x| x.with_texture_filter(egui::TextureFilter::Nearest))
                    .map_err(|e| anyhow::format_err!("{}", e))?;

                Ok(Arc::new(image))
            };

            let image = load_image()?;

            sender.send(SourceTextureStatus::Found(image));

            let (watch_sender, watch_receiver) = std::sync::mpsc::channel();
            let mut watcher = notify::watcher(watch_sender, std::time::Duration::from_secs(1))
                .expect("Watch file");

            watcher.watch(&path, notify::RecursiveMode::NonRecursive)?;

            // TODO: Clean up file watcher once all receivers have been dropped:
            //       https://github.com/Darksonn/watch/issues/3

            while let Ok(event) = watch_receiver.recv() {
                if let notify::DebouncedEvent::Write(_) = event {
                    let image = load_image()?;

                    sender.send(SourceTextureStatus::Found(image));
                }
            }

            Ok(())
        };

        if let Err(e) = inner() {
            error_sender.send(SourceTextureStatus::Error(e.to_string()));
        }
    });

    receiver
}
