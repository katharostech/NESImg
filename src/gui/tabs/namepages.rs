use crate::gui::ProjectState;

use super::NesimgGuiTab;

#[derive(Default)]
pub struct NamepagesTab;

impl NesimgGuiTab for NamepagesTab {
    fn show(
        &mut self,
        _project: &mut ProjectState,
        ctx: &egui::Context,
        _frame: &mut eframe::Frame,
    ) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("Namepages");
        });
    }

    fn help_text(&self) -> &'static str {
        include_str!("./namepages_help.txt")
    }
}
