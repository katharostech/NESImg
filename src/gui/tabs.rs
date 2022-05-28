use crate::NesimgGui;

pub trait NesimgGuiTab {
    fn show(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame);
}

pub mod maps;
pub mod metatiles;
pub mod namepages;
pub mod sources;
