use crate::gui::ProjectState;

use super::NesimgGuiTab;

#[derive(Default)]
pub struct MetatilesTab;

impl NesimgGuiTab for MetatilesTab {
    fn show(
        &mut self,
        _project: &mut ProjectState,
        ctx: &egui::Context,
        _frame: &mut eframe::Frame,
    ) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("Metatiles");
        });
    }

    fn help_text(&self) -> &'static str {
        include_str!("./metatiles_help.txt")
    }
}
