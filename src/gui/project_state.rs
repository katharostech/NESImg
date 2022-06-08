use std::{path::PathBuf, sync::Arc};

use egui::util::undoer::Undoer;
use egui_extras::RetainedImage;
use indexmap::IndexMap;
use path_absolutize::Absolutize;
use ulid::Ulid;
use watch::WatchReceiver;

use crate::project::Project;

use super::util::load_and_watch_image;

#[derive(Clone)]
pub struct LoadedProject {
    pub data: Project,
    pub path: PathBuf,
}

#[derive(Clone)]
pub struct SourceImageData {
    /// The image texture for the source, which can be displayed by Egui and contains the image size
    /// info
    pub texture: Arc<RetainedImage>,
    /// The vector of pixels in the image, but instead of a color, the pixels contain the color
    /// index, 0-4
    pub indexes: Vec<u8>,
}

#[derive(Clone)]
pub enum SourceImageStatus {
    Loading,
    Error(String),
    Found(SourceImageData),
}

pub struct SourceImage {
    pub path: PathBuf,
    pub data: WatchReceiver<SourceImageStatus>,
}

pub struct ProjectState {
    pub data: Project,
    pub path: PathBuf,
    pub undoer: Undoer<Project>,
    pub source_images: IndexMap<Ulid, SourceImage>,
}

impl ProjectState {
    pub fn add_source(&mut self, path: PathBuf) {
        let id = Ulid::new();
        let absolute_path = path.absolutize().unwrap().to_path_buf();
        let relative_path = pathdiff::diff_paths(absolute_path, &self.path.absolutize().unwrap())
            .expect("Same filesystem");
        self.data.sources.insert(id, relative_path.clone());
        self.source_images.insert(
            id,
            SourceImage {
                data: load_and_watch_image(&path),
                path: relative_path,
            },
        );
    }

    pub fn update_source(&mut self, id: Ulid, path: PathBuf) {
        let absolute_path = path.absolutize().unwrap().to_path_buf();
        let relative_path = pathdiff::diff_paths(absolute_path, &self.path.absolutize().unwrap())
            .expect("Same filesystem");
        *self.data.sources.get_mut(&id).expect("missing source") = relative_path.clone();
        *self.source_images.get_mut(&id).expect("missing source") = SourceImage {
            data: load_and_watch_image(&path),
            path: relative_path,
        }
    }

    /// Reloads all the source images from the current project source list
    pub fn reload_source_images(&mut self) {
        self.source_images = self
            .data
            .sources
            .iter()
            .map(|(id, path)| {
                (
                    *id,
                    SourceImage {
                        data: load_and_watch_image(
                            &self
                                .path
                                .absolutize()
                                .unwrap()
                                .join(&path)
                                .absolutize()
                                .expect("Absoluteize"),
                        ),
                        path: path.clone(),
                    },
                )
            })
            .collect();
    }
}
