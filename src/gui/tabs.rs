use super::ProjectState;

pub trait NesimgGuiTab {
    fn show(&mut self, project: &mut ProjectState, ctx: &egui::Context, frame: &mut eframe::Frame);
    #[allow(unused_variables)]
    fn help_text(&self) -> &'static str {
        ""
    }
}

pub mod maps;
pub mod metatiles;
pub mod namepages;
pub mod sources;
