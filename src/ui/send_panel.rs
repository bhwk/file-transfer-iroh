use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use crate::iroh_client;
use eframe::egui;
use egui::{RichText, Vec2};
use egui_file_dialog::FileDialog;
use tokio_util::sync::CancellationToken;
use tracing::info;

pub struct SendPanel {
    file_dialog: FileDialog,
    picked_file: Option<PathBuf>,
    cancel_token: Option<CancellationToken>,
    ticket: Arc<Mutex<Option<String>>>,
    status: Arc<Mutex<String>>,
    runtime: Arc<tokio::runtime::Runtime>,
}

impl SendPanel {
    pub fn new(_cc: &eframe::CreationContext) -> Self {
        Self::default()
    }
}

impl Default for SendPanel {
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
            file_dialog: FileDialog::new().anchor(align, offset),
            picked_file: None,
            cancel_token: None,
            ticket: Default::default(),
            status: Arc::new(Mutex::new("Waiting".into())),
            runtime,
        }
    }
}

impl SendPanel {
    pub fn ui(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
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
    }
}
