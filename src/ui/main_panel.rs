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

pub struct MainPanel {
    tab: Tab,
    send_panel: SendPanel,
    receive_panel: ReceivePanel,
}

impl MainPanel {
    pub fn new() -> Self {
        Self {
            tab: Tab::default(),
            send_panel: SendPanel::default(),
            receive_panel: ReceivePanel::default(),
        }
    }
}

impl MainPanel {
    pub fn ui(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("tabs").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui
                    .selectable_label(
                        self.tab == Tab::Send,
                        egui::RichText::new("Send").size(20.0).strong(),
                    )
                    .clicked()
                {
                    self.tab = Tab::Send;
                }
                if ui
                    .selectable_label(
                        self.tab == Tab::Receive,
                        egui::RichText::new("Receive").size(20.0).strong(),
                    )
                    .clicked()
                {
                    self.tab = Tab::Receive;
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| match self.tab {
            Tab::Send => self.send_panel.ui(ui, ctx),
            Tab::Receive => self.receive_panel.ui(ui, ctx),
        });
    }
}
