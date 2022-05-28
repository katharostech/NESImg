use crate::{gui::RootState, NesimgGui};

use super::NesimgGuiTab;

#[derive(Default)]
pub struct NamepagesTab;

impl NesimgGuiTab for NamepagesTab {
    fn show(&mut self, root_state: &mut RootState, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("Namepages");
        });
    }
}
