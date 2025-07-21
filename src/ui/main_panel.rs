use super::{receive_panel::ReceivePanel, send_panel::SendPanel};
use eframe::egui;

#[derive(PartialEq)]
enum Tab {
    Send,
    Receive,
}

impl Default for Tab {
    fn default() -> Self {
        Self::Send
    }
}

#[derive(Default)]
pub struct MainPanel {
    tab: Tab,
    send_panel: SendPanel,
    receive_panel: ReceivePanel,
}

impl MainPanel {
    pub fn ui(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("tabs").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.selectable_label(self.tab == Tab::Send, "Send").clicked() {
                    self.tab = Tab::Send;
                }
                if ui
                    .selectable_label(self.tab == Tab::Receive, "Receive")
                    .clicked()
                {
                    self.tab = Tab::Receive;
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| match self.tab {
            Tab::Send => self.send_panel.ui(ui, ctx),
            Tab::Receive => self.receive_panel.ui(ui),
        });
    }
}
