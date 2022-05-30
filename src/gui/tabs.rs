use super::ProjectState;

pub trait NesimgGuiTab {
    fn show(&mut self, project: &mut ProjectState, ctx: &egui::Context, frame: &mut eframe::Frame);
    #[allow(unused_variables)]
    fn show_help(&mut self, ui: &mut egui::Ui) {}
}

pub mod maps;
pub mod metatiles;
pub mod namepages;
pub mod sources;
