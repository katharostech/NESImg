use crate::{gui::RootState, NesimgGui};

use super::NesimgGuiTab;

#[derive(Default)]
pub struct MetatilesTab;

impl NesimgGuiTab for MetatilesTab {
    fn show(&mut self, root_state: &mut RootState, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("Metatiles");
        });
    }
}
