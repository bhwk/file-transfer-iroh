use eframe::egui;

#[derive(Default)]
pub struct ReceivePanel;

impl ReceivePanel {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("📥 Receive File");

        if ui.button("Paste ticket").clicked() {
            // ticket logic here
        }

        // More receive UI...
    }
}
