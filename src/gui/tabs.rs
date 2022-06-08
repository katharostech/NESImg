use super::ProjectState;

pub trait NesimgGuiTab {
    fn show(&mut self, project: &mut ProjectState, ctx: &egui::Context, frame: &mut eframe::Frame);

    fn help_text(&self) -> &'static str {
        ""
    }

    fn tooltip(&self) -> &'static str;
}

pub mod maps;
pub mod metatiles;
pub mod metatilesets;
pub mod sources;
