use crate::gui::RootState;

use super::NesimgGuiTab;

#[derive(Default)]
pub struct SourcesTab;

impl NesimgGuiTab for SourcesTab {
    fn show(&mut self, root_state: &mut RootState, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("Sources");
        });
    }
}
