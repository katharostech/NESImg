use crate::gui::ProjectState;

use super::NesimgGuiTab;

#[derive(Default)]
pub struct MetatilesetsTab;

impl NesimgGuiTab for MetatilesetsTab {
    fn show(
        &mut self,
        _project: &mut ProjectState,
        ctx: &egui::Context,
        _frame: &mut eframe::Frame,
    ) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("Metatilesets");
        });
    }

    fn help_text(&self) -> &'static str {
        include_str!("./metatilesets_help.txt")
    }
}
