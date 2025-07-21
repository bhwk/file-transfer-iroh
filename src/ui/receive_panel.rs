use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use eframe::egui;
use egui::{RichText, Spinner, Vec2};
use egui_file_dialog::FileDialog;

use crate::iroh_client;

pub struct DownloadStatus {
    pub message: String,
    pub progress: f32,
    pub in_progress: bool,
    pub done: bool,
}

pub struct ReceivePanel {
    directory_dialog: FileDialog,
    picked_directory: Option<PathBuf>,
    input_ticket: String,
    download_status: Arc<Mutex<DownloadStatus>>,
    runtime: Arc<tokio::runtime::Runtime>,
}

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
            download_status: Arc::new(Mutex::new(DownloadStatus {
                message: String::from("No download in progress."),
                progress: 0.0,
                in_progress: false,
                done: false,
            })),
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
                        lock.message = "Downloading file now...".to_string();
                    }

                    runtime_clone.spawn(async move {
                        {
                            let mut lock = download_clone.lock().unwrap();
                            lock.in_progress = true;
                            lock.done = false;
                            lock.progress = 0.0;
                            lock.message = "Starting download...".into();
                        }

                        match iroh_client::receive_file(
                            path_clone,
                            ticket_string.as_str(),
                            download_clone.clone(),
                            egui_ctx_clone.clone(),
                        )
                        .await
                        {
                            Ok(()) => {}
                            Err(e) => {
                                let mut lock = download_clone.lock().unwrap();
                                lock.message = format!("Error: {e}");
                            }
                        }
                        egui_ctx_clone.request_repaint();
                    });
                };
            });
            ui.add_space(10.0);
            let status = self.download_status.lock().unwrap();

            ui.horizontal(|ui| {
                if status.in_progress {
                    ui.add(Spinner::new());
                    ui.label(RichText::new(&status.message).italics());
                } else if status.done {
                    ui.label(RichText::new(&status.message).strong());
                } else {
                    ui.label(&status.message);
                }
            });
        }
    }
}
