use crate::gui::ProjectState;

use super::NesimgGuiTab;

#[derive(Default)]
pub struct MapsTab;

impl NesimgGuiTab for MapsTab {
    fn show(
        &mut self,
        _project: &mut ProjectState,
        ctx: &egui::Context,
        _frame: &mut eframe::Frame,
    ) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("Maps");
        });
    }

    fn help_text(&self) -> &'static str {
        include_str!("./maps_help.txt")
    }

    fn tooltip(&self) -> &'static str {
        "Create maps and levels from metatiles"
    }
}
