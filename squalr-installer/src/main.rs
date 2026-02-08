#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::NativeOptions;
use eframe::egui;
use eframe::egui::{Align, Layout, RichText, ScrollArea, ViewportBuilder};
use log::{Level, Log, Metadata, Record, SetLoggerError};
use squalr_engine::app_provisioner::app_provisioner_config::AppProvisionerConfig;
use squalr_engine::app_provisioner::installer::app_installer::AppInstaller;
use squalr_engine::app_provisioner::installer::install_phase::InstallPhase;
use squalr_engine::app_provisioner::installer::install_progress::InstallProgress;
use squalr_engine::app_provisioner::operations::launch::update_operation_launch::UpdateOperationLaunch;
use squalr_engine::app_provisioner::progress_tracker::ProgressTracker;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

const APP_NAME: &str = "Squalr Installer";
const MAX_LOG_BUFFER_BYTES: usize = 256 * 1024;

#[derive(Clone)]
struct InstallerUiState {
    installer_progress: f32,
    installer_progress_string: String,
    install_complete: bool,
    installer_logs: String,
}

impl InstallerUiState {
    fn new() -> Self {
        Self {
            installer_progress: 0.0,
            installer_progress_string: "0%".to_string(),
            install_complete: false,
            installer_logs: String::new(),
        }
    }

    fn set_progress(
        &mut self,
        install_progress: InstallProgress,
    ) {
        self.installer_progress = install_progress.progress_percent.clamp(0.0, 1.0);
        self.installer_progress_string = format!("{:.0}%", self.installer_progress * 100.0);
        self.install_complete = install_progress.phase == InstallPhase::Complete;
    }

    fn append_log(
        &mut self,
        log_message: &str,
    ) {
        self.installer_logs.push_str(log_message);
        trim_log_buffer(&mut self.installer_logs, MAX_LOG_BUFFER_BYTES);
    }
}

struct InstallerLogger {
    ui_state: Arc<Mutex<InstallerUiState>>,
}

impl InstallerLogger {
    fn new(ui_state: Arc<Mutex<InstallerUiState>>) -> Self {
        Self { ui_state }
    }
}

impl Log for InstallerLogger {
    fn enabled(
        &self,
        metadata: &Metadata,
    ) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(
        &self,
        record: &Record,
    ) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let log_message = format!("[{}] {}\n", record.level(), record.args());

        if let Ok(mut ui_state) = self.ui_state.lock() {
            ui_state.append_log(&log_message);
        }
    }

    fn flush(&self) {}
}

struct InstallerApp {
    ui_state: Arc<Mutex<InstallerUiState>>,
}

impl InstallerApp {
    fn new(ui_state: Arc<Mutex<InstallerUiState>>) -> Self {
        start_installer(ui_state.clone());
        Self { ui_state }
    }

    fn launch_app(&self) {
        match AppProvisionerConfig::get_default_install_dir() {
            Ok(app_install_directory) => {
                let executable_path = app_install_directory.join("squalr.exe");
                UpdateOperationLaunch::launch_app(&executable_path);
            }
            Err(error) => {
                log::error!("Failed to resolve install directory: {}", error);
            }
        }
    }
}

impl eframe::App for InstallerApp {
    fn update(
        &mut self,
        context: &egui::Context,
        _frame: &mut eframe::Frame,
    ) {
        context.request_repaint_after(Duration::from_millis(50));

        let state_snapshot = match self.ui_state.lock() {
            Ok(ui_state) => ui_state.clone(),
            Err(_) => InstallerUiState::new(),
        };

        egui::TopBottomPanel::top("header").show(context, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(APP_NAME).strong().size(18.0));
            });
        });

        egui::TopBottomPanel::bottom("footer")
            .resizable(false)
            .show(context, |ui| {
                ui.horizontal(|ui| {
                    ui.label(" ");
                });
            });

        egui::CentralPanel::default().show(context, |ui| {
            ui.with_layout(Layout::top_down(Align::Center), |ui| {
                let status_text = if state_snapshot.install_complete {
                    "Squalr installed successfully!"
                } else {
                    "Installing Squalr, please wait..."
                };
                ui.add_space(8.0);
                ui.label(status_text);
                ui.add_space(8.0);

                ui.add(egui::ProgressBar::new(state_snapshot.installer_progress).text(state_snapshot.installer_progress_string));
                ui.add_space(8.0);

                ui.separator();
                ScrollArea::vertical().stick_to_bottom(true).show(ui, |ui| {
                    ui.with_layout(Layout::top_down(Align::LEFT), |ui| {
                        ui.label(
                            RichText::new(state_snapshot.installer_logs.as_str())
                                .monospace()
                                .size(12.0),
                        );
                    });
                });
                ui.separator();

                if state_snapshot.install_complete && ui.button("Launch Squalr").clicked() {
                    self.launch_app();
                }
            });
        });
    }
}

fn trim_log_buffer(
    log_buffer: &mut String,
    max_buffer_bytes: usize,
) {
    if log_buffer.len() <= max_buffer_bytes {
        return;
    }

    let mut trim_start_index = log_buffer.len().saturating_sub(max_buffer_bytes);
    while trim_start_index < log_buffer.len() && !log_buffer.is_char_boundary(trim_start_index) {
        trim_start_index += 1;
    }

    log_buffer.drain(..trim_start_index);
}

fn initialize_logger(ui_state: Arc<Mutex<InstallerUiState>>) -> Result<(), SetLoggerError> {
    let logger = InstallerLogger::new(ui_state);
    log::set_boxed_logger(Box::new(logger))?;
    log::set_max_level(log::LevelFilter::Info);
    Ok(())
}

fn start_installer(ui_state: Arc<Mutex<InstallerUiState>>) {
    let progress_tracker = ProgressTracker::new();
    let progress_receiver = progress_tracker.subscribe();
    let progress_ui_state = ui_state.clone();

    thread::spawn(move || {
        for install_progress in progress_receiver {
            if let Ok(mut state) = progress_ui_state.lock() {
                state.set_progress(install_progress);
            }
        }
    });

    match AppProvisionerConfig::get_default_install_dir() {
        Ok(install_directory) => {
            AppInstaller::run_installation(install_directory, progress_tracker);
        }
        Err(error) => {
            log::error!("Failed to resolve install directory: {}", error);
        }
    }
}

pub fn main() {
    let ui_state = Arc::new(Mutex::new(InstallerUiState::new()));

    if let Err(error) = initialize_logger(ui_state.clone()) {
        eprintln!("Failed to initialize installer logger: {}", error);
    }

    let native_options = NativeOptions {
        viewport: ViewportBuilder::default()
            .with_inner_size([640.0, 640.0])
            .with_min_inner_size([480.0, 420.0]),
        ..NativeOptions::default()
    };

    let run_result = eframe::run_native(
        APP_NAME,
        native_options,
        Box::new(move |_creation_context| Ok(Box::new(InstallerApp::new(ui_state.clone())))),
    );

    if let Err(error) = run_result {
        eprintln!("Fatal error starting installer GUI: {}", error);
    }
}

#[cfg(test)]
mod tests {
    use super::trim_log_buffer;

    #[test]
    fn trim_log_buffer_keeps_recent_log_data() {
        let mut log_buffer = "line-1\nline-2\nline-3\n".to_string();
        trim_log_buffer(&mut log_buffer, 8);
        assert_eq!(log_buffer, "\nline-3\n");
    }

    #[test]
    fn trim_log_buffer_does_not_modify_when_under_limit() {
        let mut log_buffer = "line-1\n".to_string();
        trim_log_buffer(&mut log_buffer, 1024);
        assert_eq!(log_buffer, "line-1\n");
    }
}
