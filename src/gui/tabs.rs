use crate::NesimgGui;

use super::RootState;

pub trait NesimgGuiTab {
    fn show(&mut self, root_state: &mut RootState, ctx: &egui::Context, frame: &mut eframe::Frame);
}

pub mod maps;
pub mod metatiles;
pub mod namepages;
pub mod sources;
