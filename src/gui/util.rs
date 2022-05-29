use std::path::Path;

use native_dialog::FileDialog;
use watch::WatchReceiver;

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
