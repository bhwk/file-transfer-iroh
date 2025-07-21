#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::{NativeOptions, egui};
use tracing_subscriber::fmt::init;
use ui::main_panel::MainPanel;

mod iroh_client;
mod ui;

fn main() -> eframe::Result<()> {
    init();
    let options = NativeOptions::default();
    eframe::run_native(
        "P2P File Transfer",
        options,
        Box::new(|_ctx| Ok(Box::new(MyApp::new()))),
    )
}

pub struct MyApp {
    main_panel: MainPanel,
}

impl MyApp {
    pub fn new() -> Self {
        let main_panel = MainPanel::new();
        Self { main_panel }
    }
}

impl Default for MyApp {
    fn default() -> Self {
        Self::new()
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.main_panel.ui(ctx);
    }
}
