use crate::NesimgGui;

use super::NesimgGuiTab;

#[derive(Default)]
pub struct MapsTab;

impl NesimgGuiTab for MapsTab {
    fn show(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("Maps");
        });
    }
}
