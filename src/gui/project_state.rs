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

pub type SourceTexture = Arc<RetainedImage>;

#[derive(Clone)]
pub enum SourceTextureStatus {
    Loading,
    Error(String),
    Found(SourceTexture),
}

pub struct SourceImage {
    pub path: PathBuf,
    pub texture: WatchReceiver<SourceTextureStatus>,
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
        self.data.sources.insert(id.clone(), relative_path.clone());
        self.source_images.insert(
            id,
            SourceImage {
                texture: load_and_watch_image(&path),
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
            texture: load_and_watch_image(&path),
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
                    id.clone(),
                    SourceImage {
                        texture: load_and_watch_image(
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
