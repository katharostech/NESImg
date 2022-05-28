use crate::NesimgGui;

use super::NesimgGuiTab;

#[derive(Default)]
pub struct NamepagesTab;

impl NesimgGuiTab for NamepagesTab {
    fn show(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("Namepages");
        });
    }
}
