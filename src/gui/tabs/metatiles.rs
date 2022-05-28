use crate::NesimgGui;

use super::NesimgGuiTab;

#[derive(Default)]
pub struct MetatilesTab;

impl NesimgGuiTab for MetatilesTab {
    fn show(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("Metatiles");
        });
    }
}
