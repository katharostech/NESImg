use std::{path::PathBuf, sync::Arc};

use egui::util::undoer::Undoer;
use egui_extras::RetainedImage;
use indexmap::IndexMap;
use path_absolutize::Absolutize;
use watch::WatchReceiver;

use crate::{project::Project, Uid};

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
    pub source_images: IndexMap<Uid<PathBuf>, SourceImage>,
}

impl ProjectState {
    pub fn add_source(&mut self, path: PathBuf) {
        let id = Uid::new();
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

    pub fn update_source(&mut self, id: Uid<PathBuf>, path: PathBuf) {
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

    /// Cleans up items with UID's pointing to non-existent objects. This happens when, for
    /// instance, we delete a metatile that is contained in a metatileset or other similar
    /// scenarios.
    pub fn cleanup_dead_refs(&mut self) {
        for metatile in self.data.metatiles.values_mut() {
            for possible_tile in &mut metatile.tiles {
                if let Some(tile) = possible_tile {
                    if !self.data.sources.contains_key(&tile.source_id) {
                        *possible_tile = None;
                    }
                }
            }
        }

        for metatileset in self.data.metatilesets.values_mut() {
            metatileset
                .tiles
                .retain(|_, tile| self.data.metatiles.contains_key(&tile.metatile_id));
        }

        for level in self.data.levels.values_mut() {
            if let Some(metatileset) = self.data.metatilesets.get(&level.metatileset_id) {
                level
                    .tiles
                    .retain(|_, tile| metatileset.tiles.contains_key(&tile.metatileset_tile_id));
            } else {
                // TODO: metatileset_id should be an option I think
                level.metatileset_id = Uid::default();
                level.tiles = Default::default();
            }
        }
    }
}
