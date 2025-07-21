#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::{NativeOptions, egui};
use egui::{RichText, Vec2};
use egui_file_dialog::FileDialog;
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};
use tracing::info;
use tracing_subscriber::fmt::init;

mod iroh_client;
mod ui;

use tokio_util::sync::CancellationToken;

fn main() -> eframe::Result<()> {
    init();
    let options = NativeOptions::default();
    eframe::run_native(
        "P2P File Transfer",
        options,
        Box::new(|ctx| Ok(Box::new(MyApp::new(ctx)))),
    )
}

struct MyApp {
    file_dialog: FileDialog,
    directory_dialog: FileDialog,
    picked_file: Option<PathBuf>,
    picked_directory: Option<PathBuf>,
    cancel_token: Option<CancellationToken>,
    ticket: Arc<Mutex<Option<String>>>,
    input_ticket: String,
    status: Arc<Mutex<String>>,
    download_status: Arc<Mutex<String>>,
    runtime: Arc<tokio::runtime::Runtime>,
}
impl MyApp {
    pub fn new(_cc: &eframe::CreationContext) -> Self {
        let runtime = Arc::new(
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .expect("Failed to build runtime"),
        );

        let align = egui::Align2::CENTER_CENTER;
        let offset = Vec2::new(0.0, 0.0);

        Self {
            file_dialog: FileDialog::new().anchor(align, offset),
            directory_dialog: FileDialog::new().anchor(align, offset),
            picked_file: None,
            picked_directory: None,
            cancel_token: None,
            ticket: Arc::new(Mutex::new(None)),
            input_ticket: String::from(""),
            status: Arc::new(Mutex::new("Waiting".into())),
            runtime,
            download_status: Arc::new(Mutex::new("No download in progress.".into())),
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Send File");
            ui.add_space(10.0);

            ui.horizontal(|ui| {
                if ui.button("Pick file").clicked() {
                    self.file_dialog.pick_file();
                }
            });

            ui.add_space(5.0);

            self.file_dialog.update(ctx);

            if let Some(path) = self.file_dialog.take_picked() {
                self.picked_file = Some(path.to_path_buf());
                // Reset status and ticket when a new file is picked
                *self.status.lock().unwrap() = "File selected. Ready to send.".into();
                *self.ticket.lock().unwrap() = None;
            }

            // Display picked file path if available
            if let Some(path) = &self.picked_file {
                ui.label(format!("Picked file: {}", path.display()));
                ui.add_space(5.0);

                ui.horizontal_wrapped(|ui| {
                    if ui.button("Send File").clicked() {
                        // cancel previous send operation
                        if let Some(cancel_token) = &self.cancel_token {
                            info!("Cancelling previous send operation.");
                            cancel_token.cancel();
                        }

                        let path_clone = path.clone();
                        let status_clone = self.status.clone();
                        let ticket_clone = self.ticket.clone();
                        let egui_ctx_clone = ctx.clone();

                        let new_cancel_token = CancellationToken::new();
                        let cancel_clone = new_cancel_token.clone();
                        self.cancel_token = Some(new_cancel_token);

                        *status_clone.lock().unwrap() = "Sending...".into();
                        egui_ctx_clone.request_repaint();

                        // we keep the runtime outside so we can actually run the tasks
                        let runtime_clone = self.runtime.clone();
                        runtime_clone.spawn(async move {
                            match iroh_client::send_file(path_clone, cancel_clone).await {
                                Ok(new_ticket) => {
                                    let mut lock = ticket_clone.lock().unwrap();
                                    *lock = Some(new_ticket);
                                    *status_clone.lock().unwrap() = "Now hosting file.".into();
                                }
                                Err(e) => {
                                    let mut lock = status_clone.lock().unwrap();
                                    *lock = format!("Error: {e}");
                                    *ticket_clone.lock().unwrap() = None;
                                }
                            }
                            egui_ctx_clone.request_repaint();
                        });
                    }
                    ui.add_space(5.0);

                    // Create a cancel button for the file transfer
                    if ui.button("Stop Sending").clicked() {
                        if let Some(cancel_token) = &self.cancel_token {
                            info!("Manually cancelling current send operation.");
                            cancel_token.cancel();
                            // clear all our fields
                            *self.status.lock().unwrap() = "Transfer stopped by user.".into();
                            *self.ticket.lock().unwrap() = None;
                            self.cancel_token = None;
                            ctx.request_repaint();
                        }
                    }
                });
                ui.add_space(5.0);

                if let Some(ticket) = self.ticket.lock().unwrap().as_ref() {
                    ui.horizontal_wrapped(|ui| {
                        ui.label("Ticket:");
                        ui.label(RichText::new(ticket).strong());
                        if ui.button("📋").clicked() {
                            ui.ctx().copy_text(ticket.clone().to_owned());
                        }
                    });
                }
                ui.label(self.status.lock().unwrap().as_str());
            } else {
                ui.label("No file selected.");
            }

            ui.add_space(20.0);

            // Section for Downloading Files
            ui.heading("Receive file");
            ui.add_space(10.0);
            ui.label("Select download location.");
            if ui.button("Select folder").clicked() {
                self.directory_dialog.pick_directory();
            }

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
                            match iroh_client::receive_file(path_clone, ticket_string.as_str())
                                .await
                            {
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
        });
    }
}
