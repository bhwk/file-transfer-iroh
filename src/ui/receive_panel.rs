use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use eframe::egui;
use egui::Vec2;
use egui_file_dialog::FileDialog;

use crate::iroh_client;

pub struct ReceivePanel {
    directory_dialog: FileDialog,
    picked_directory: Option<PathBuf>,
    input_ticket: String,
    download_status: Arc<Mutex<String>>,
    runtime: Arc<tokio::runtime::Runtime>,
}
impl ReceivePanel {}

impl Default for ReceivePanel {
    fn default() -> Self {
        let runtime = Arc::new(
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .expect("Failed to build runtime"),
        );
        let align = egui::Align2::CENTER_CENTER;
        let offset = Vec2::new(0.0, 0.0);
        Self {
            directory_dialog: FileDialog::new().anchor(align, offset),
            picked_directory: None,
            input_ticket: String::from(""),
            runtime,
            download_status: Arc::new(Mutex::new("No download in progress.".into())),
        }
    }
}

impl ReceivePanel {
    pub fn ui(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.heading("Receive file");

        ui.add_space(10.0);

        if ui.button("Select folder").clicked() {
            self.directory_dialog.pick_directory();
        }

        ui.add_space(5.0);

        ui.label("Select download location.");

        self.directory_dialog.update(ctx);

        if let Some(path) = self.directory_dialog.take_picked() {
            self.picked_directory = Some(path.to_path_buf());
        }

        ui.add_space(5.0);

        // wait until user picks a download location before
        // allowing them to input a ticket
        if let Some(dir_path) = &self.picked_directory {
            ui.label(format!("Download location: {}", dir_path.display()));
            ui.add_space(5.0);

            ui.horizontal_wrapped(|ui| {
                let input_ticket_label = ui.label("Enter Ticket: ");
                ui.text_edit_singleline(&mut self.input_ticket)
                    .labelled_by(input_ticket_label.id);
                if ui.button("Download").clicked() {
                    // we keep the runtime outside so we can actually run the tasks
                    let download_clone = self.download_status.clone();
                    let runtime_clone = self.runtime.clone();
                    let egui_ctx_clone = ctx.clone();
                    let ticket_string = self.input_ticket.clone();
                    let path_clone = dir_path.clone();

                    {
                        let mut lock = download_clone.lock().unwrap();
                        *lock = "Downloading file now...".to_string();
                    }

                    runtime_clone.spawn(async move {
                        match iroh_client::receive_file(path_clone, ticket_string.as_str()).await {
                            Ok(()) => {
                                let mut lock = download_clone.lock().unwrap();
                                *lock = "File downloaded.".to_string();
                            }
                            Err(e) => {
                                let mut lock = download_clone.lock().unwrap();
                                *lock = format!("Error: {e}");
                            }
                        }
                        egui_ctx_clone.request_repaint();
                    });
                };
            });
            ui.add_space(5.0);
            ui.label(self.download_status.lock().unwrap().to_string());
        }
    }
}
